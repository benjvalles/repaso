use super::baserow::BaserowClient;
use chrono::Utc;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::Mutex;

/// Resultados de la sincronización. Contiene contadores por cada tabla y los errores encontrados.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncResult {
    pub config_synced: u32,
    pub profiles_synced: u32,
    pub sessions_synced: u32,
    pub session_questions_synced: u32,
    pub errors: Vec<String>,
}

const FIELD_PROFILES_DELETED_AT: &str = "field_9679183";
const FIELD_SESSIONS_DELETED_AT: &str = "field_9679269";
const FIELD_QUESTIONS_DELETED_AT: &str = "field_9679279";
const FIELD_SESSIONS_UPDATED_AT: &str = "field_9679263";
const FIELD_QUESTIONS_UPDATED_AT: &str = "field_9679275";

/// ID de la tabla de configuración de usuario en Baserow
///
/// Almacena la configuración global de la aplicación por usuario
const TABLE_USER_CONFIG: i64 = 1071740;

/// ID de la tabla de perfiles de usuario en Baserow
///
/// Almacena los datos de los perfiles de usuario (display_name, school_year, nivel, etc.)
const TABLE_USER_PROFILES: i64 = 1071741;

/// ID de la tabla de sesiones de usuario en Baserow
///
/// Almacena los metadatos de las sesiones de aprendizaje del usuario
const TABLE_USER_SESSIONS: i64 = 1071742;

/// ID de la tabla de preguntas de sesión de usuario en Baserow
///
/// Almacena el registro histórico de preguntas respondidas en cada sesión
const TABLE_USER_SESSION_QUESTIONS: i64 = 1071743;

/// Marca de tiempo UTC actual en formato RFC3339 (ej. "2024-01-15T14:30:00Z").
fn now_rfc3339() -> String {
    Utc::now().to_rfc3339()
}

/// Extrae un valor numérico de un campo JSON de Baserow, aceptando tanto Number como String.
fn value_to_i64(v: &Value) -> Option<i64> {
    v.as_i64().or_else(|| v.as_str()?.parse::<i64>().ok())
}

/// Extrae el campo ID de una fila JSON de Baserow. Devuelve 0 si falta o es inválido.
fn row_id(value: &Value) -> i64 {
    value["id"].as_i64().unwrap_or(0)
}

/// Lee todas las configuraciones de la aplicación desde la base de datos local
///
/// # Argumentos
/// * `db` - Referencia a la conexión SQLite local
///
/// # Proceso
/// 1. Prepara una sentencia SQL para seleccionar key, value y updated_at desde app_settings
/// 2. Ejecuta la consulta y mapea cada fila a una tupla (String, String, String)
/// 3. Filtra las filas donde la clave está vacía
/// 4. Retorna el vector de configuraciones leídas
///
/// # Retorna
/// - Vec<(key, value, updated_at)> - Lista de configuraciones locales
/// - Error - Si ocurre un error SQL
fn read_local_app_settings(db: &Connection) -> Result<Vec<(String, String, String)>, String> {
    let mut stmt = db
        .prepare("SELECT key, value, updated_at FROM app_settings ORDER BY key")
        .map_err(|e| format!("Error leyendo config local: {e}"))?;
    let rows = stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
            ))
        })
        .map_err(|e| format!("Error leyendo config local: {e}"))?
        .filter_map(|r| r.ok())
        .filter(|(k, _, _)| !k.is_empty())
        .collect();
    Ok(rows)
}

/// Escribe filas de configuración de Baserow en SQLite local (solo inserción).
fn write_remote_config(
    db: &Connection,
    remote_rows: &[Value],
    local_rows: &[(String, String, String)],
) -> Result<u32, String> {
    let mut synced = 0u32;
    for rem in remote_rows {
        let rem_key = rem["field_9480705"].as_str().unwrap_or("");
        if rem_key.is_empty() { continue; }
        let exists = local_rows.iter().any(|(k, _, _)| k == rem_key);
        if !exists {
            let rem_val = rem["field_9480706"].as_str().unwrap_or("");
            let rem_updated = rem["field_9480707"].as_str().unwrap_or("");
            db.execute(
                "INSERT INTO app_settings (key, value, updated_at)
                 VALUES (?1, ?2, ?3)
                 ON CONFLICT(key) DO UPDATE
                 SET value = excluded.value, updated_at = excluded.updated_at",
                params![rem_key, rem_val, rem_updated],
            )
            .map_err(|e| format!("Error insertando config remota: {e}"))?;
            synced += 1;
        }
    }
    Ok(synced)
}

/// Sube configuraciones locales a la tabla de configuración de Baserow.
async fn upload_local_config(
    client: &BaserowClient,
    local_rows: &[(String, String, String)],
    remote_rows: &[Value],
    user_id: &str,
) -> Result<u32, String> {
    let mut synced = 0u32;
    for (key, value, updated_at) in local_rows {
        let remote = remote_rows
            .iter()
            .find(|r| r["field_9480705"].as_str() == Some(key.as_str()));
        let now = now_rfc3339();
        match remote {
            Some(rem) => {
                let rem_updated = rem["field_9480707"].as_str().unwrap_or("");
                if updated_at.as_str() > rem_updated {
                    client
                        .update_row(
                            TABLE_USER_CONFIG,
                            row_id(rem),
                            serde_json::json!({
                                "field_9480689": user_id,
                                "field_9480705": key,
                                "field_9480706": value,
                                "field_9480707": now,
                            }),
                        )
                        .await?;
                    synced += 1;
                }
            }
            None => {
                client
                    .create_row(
                        TABLE_USER_CONFIG,
                        serde_json::json!({
                            "field_9480689": user_id,
                            "field_9480705": key,
                            "field_9480706": value,
                            "field_9480707": now,
                        }),
                    )
                    .await?;
                synced += 1;
            }
        }
    }
    Ok(synced)
}

/// Sincroniza configuraciones de la aplicación entre SQLite local y Baserow.
async fn sync_config_table(
    client: &BaserowClient,
    db: &Mutex<Connection>,
    user_id: &str,
) -> Result<u32, String> {
    let local_rows = {
        let db = db.lock().map_err(|e| format!("Error de base de datos: {e}"))?;
        read_local_app_settings(&db)?
    };

    let remote_rows: Vec<Value> = client
        .list_rows(
            TABLE_USER_CONFIG,
            &[("filter__field_9480689__equal", user_id)],
        )
        .await?
        .into_iter()
        .filter(|r| {
            r["field_9480689"].as_str().map(|s| !s.is_empty()).unwrap_or(false)
            && r["field_9480705"].as_str().map(|s| !s.is_empty()).unwrap_or(false)
        })
        .collect();

    let mut synced = upload_local_config(client, &local_rows, &remote_rows, user_id).await?;

    {
        let db = db.lock().map_err(|e| format!("Error de base de datos: {e}"))?;
        synced += write_remote_config(&db, &remote_rows, &local_rows)?;
    }

    Ok(synced)
}

/// Lee todos los perfiles desde la base de datos local
///
/// # Argumentos
/// * `db` - Referencia a la conexión SQLite local
///
/// # Proceso
/// 1. Prepara una sentencia SQL para seleccionar todos los campos de la tabla profiles
/// 2. Ejecuta la consulta y mapea cada fila a una tupla con todos los campos:
///    - id, display_name, school_year (u8), age (Option<u8>), level_mode,
///    - current_level (u8), manual_prompt (Option<String>), created_at, updated_at
/// 3. Filtra las filas donde el ID está vacío
///
/// # Retorna
/// - Vec<(id, display_name, school_year, age, level_mode, current_level, manual_prompt, created_at, updated_at)> - Lista de perfiles locales
/// - Error - Si ocurre un error SQL
/// Lee todos los perfiles desde SQLite local.
fn read_local_profiles(
    db: &Connection,
) -> Result<Vec<(String, String, u8, Option<u8>, String, u8, Option<String>, String, String, Option<String>)>, String> {
    let mut stmt = db
        .prepare(
            "SELECT id, display_name, school_year, age, level_mode,
                    current_level, manual_prompt, created_at, updated_at, deleted_at
             FROM profiles ORDER BY id",
        )
        .map_err(|e| format!("Error leyendo perfiles locales: {e}"))?;
    let rows = stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, u8>(2)?,
                row.get::<_, Option<u8>>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, u8>(5)?,
                row.get::<_, Option<String>>(6)?,
                row.get::<_, String>(7)?,
                row.get::<_, String>(8)?,
                row.get::<_, Option<String>>(9)?,
            ))
        })
        .map_err(|e| format!("Error leyendo perfiles locales: {e}"))?
        .filter_map(|r| r.ok())
        .filter(|(id, _, _, _, _, _, _, _, _, _)| !id.is_empty())
        .collect();
    Ok(rows)
}

/// Aplica cambios remotos de perfiles a SQLite local.
///
/// - Inserta perfiles remotos que no existen localmente (a menos que esten borrados)
/// - Si un perfil remoto tiene `deleted_at` mas reciente que el local, lo marca borrado localmente
/// - Si un perfil remoto esta activo y el local esta borrado, lo recupera con los datos remotos
/// - Si el remoto es mas reciente (updated_at), actualiza todos los campos del perfil local
fn write_remote_profiles(
    db: &Connection,
    remote_rows: &[Value],
    local_rows: &[(String, String, u8, Option<u8>, String, u8, Option<String>, String, String, Option<String>)],
) -> Result<u32, String> {
    let mut synced = 0u32;
    for rem in remote_rows {
        let rem_profile_id = rem["field_9480692"].as_str().unwrap_or("");
        if rem_profile_id.is_empty() { continue; }
        let rem_deleted = rem[FIELD_PROFILES_DELETED_AT].as_str().unwrap_or("");
        let rem_updated = rem["field_9480716"].as_str().unwrap_or("");
        let rem_display = rem["field_9480709"].as_str().unwrap_or("");
        let rem_school = value_to_i64(&rem["field_9480710"]).unwrap_or(1) as u8;
        let rem_age = value_to_i64(&rem["field_9480711"]).map(|v| v as u8);
        let rem_lm = rem["field_9480712"].as_str().unwrap_or("automatic");
        let rem_curr = value_to_i64(&rem["field_9480713"]).unwrap_or(1) as u8;
        let rem_manual = rem["field_9480714"].as_str().map(String::from);
        let rem_cr = rem["field_9480715"].as_str().unwrap_or("");

        let local = local_rows
            .iter()
            .find(|(id, _, _, _, _, _, _, _, _, _)| id == rem_profile_id);

        match (local, rem_deleted.is_empty()) {
            // No existe local, no esta borrado remoto -> insertar
            (None, true) => {
                db.execute(
                    "INSERT OR IGNORE INTO profiles
                     (id, display_name, school_year, age, level_mode,
                      current_level, manual_prompt, created_at, updated_at)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                    params![
                        rem_profile_id, rem_display, rem_school, rem_age,
                        rem_lm, rem_curr, rem_manual, rem_cr, rem_updated,
                    ],
                )
                .map_err(|e| format!("Error insertando perfil remoto: {e}"))?;
                synced += 1;
            }
            // No existe local, esta borrado remoto -> saltar
            (None, false) => {}
            // Existe local, esta borrado remoto -> si la eliminacion remota es mas reciente, borrar local
            (Some(local), false) => {
                let local_updated = &local.8;
                if local_updated.as_str() < rem_updated {
                    db.execute(
                        "UPDATE profiles SET deleted_at = ?1, updated_at = ?1 WHERE id = ?2",
                        params![rem_deleted, rem_profile_id],
                    )
                    .map_err(|e| format!("Error aplicando eliminacion remota de perfil: {e}"))?;
                    synced += 1;
                }
            }
            // Existe local, no esta borrado remoto -> actualizar si remoto es mas reciente
            (Some(local), true) => {
                let local_updated = &local.8;
                if rem_updated > local_updated.as_str() {
                    db.execute(
                        "UPDATE profiles SET display_name = ?1, school_year = ?2, age = ?3,
                         level_mode = ?4, current_level = ?5, manual_prompt = ?6,
                         created_at = ?7, updated_at = ?8, deleted_at = NULL
                         WHERE id = ?9",
                        params![
                            rem_display, rem_school, rem_age, rem_lm, rem_curr,
                            rem_manual, rem_cr, rem_updated, rem_profile_id,
                        ],
                    )
                    .map_err(|e| format!("Error actualizando perfil desde remoto: {e}"))?;
                    synced += 1;
                }
            }
        }
    }
    Ok(synced)
}

/// Sube perfiles locales a Baserow (actualiza si la version local es mas reciente).
/// Incluye el estado `deleted_at` para propagar eliminaciones a la nube.
async fn upload_local_profiles(
    client: &BaserowClient,
    local_rows: &[(String, String, u8, Option<u8>, String, u8, Option<String>, String, String, Option<String>)],
    remote_rows: &[Value],
    user_id: &str,
) -> Result<u32, String> {
    let mut synced = 0u32;
    for (id, display_name, school_year, age, level_mode, current_level, manual_prompt, created_at, updated_at, deleted_at) in local_rows
    {
        let remote = remote_rows
            .iter()
            .find(|r| r["field_9480692"].as_str() == Some(id.as_str()));
        let now = now_rfc3339();
        let deleted_field: Value = match deleted_at {
            Some(s) => Value::String(s.clone()),
            None => Value::Null,
        };
        match remote {
            Some(rem) => {
                let rem_updated = rem["field_9480716"].as_str().unwrap_or("");
                if updated_at.as_str() > rem_updated {
                    client
                        .update_row(
                            TABLE_USER_PROFILES,
                            row_id(rem),
                            json!({
                                "field_9480692": id,
                                "field_9480708": user_id,
                                "field_9480709": display_name,
                                "field_9480710": school_year,
                                "field_9480711": age,
                                "field_9480712": level_mode,
                                "field_9480713": current_level,
                                "field_9480714": manual_prompt,
                                "field_9480715": created_at,
                                "field_9480716": now,
                                FIELD_PROFILES_DELETED_AT: deleted_field,
                            }),
                        )
                        .await?;
                    synced += 1;
                }
            }
            None => {
                client
                    .create_row(
                        TABLE_USER_PROFILES,
                        json!({
                            "field_9480692": id,
                            "field_9480708": user_id,
                            "field_9480709": display_name,
                            "field_9480710": school_year,
                            "field_9480711": age,
                            "field_9480712": level_mode,
                            "field_9480713": current_level,
                            "field_9480714": manual_prompt,
                            "field_9480715": created_at,
                            "field_9480716": now,
                            FIELD_PROFILES_DELETED_AT: deleted_field,
                        }),
                    )
                    .await?;
                synced += 1;
            }
        }
    }
    Ok(synced)
}

/// Sincroniza la tabla de perfiles entre SQLite local y Baserow.
async fn sync_profiles_table(
    client: &BaserowClient,
    db: &Mutex<Connection>,
    user_id: &str,
) -> Result<u32, String> {
    let local_rows = {
        let db = db.lock().map_err(|e| format!("Error de base de datos: {e}"))?;
        read_local_profiles(&db)?
    };

    let remote_rows: Vec<Value> = client
        .list_rows(
            TABLE_USER_PROFILES,
            &[("filter__field_9480708__equal", user_id)],
        )
        .await?
        .into_iter()
        .filter(|r| r["field_9480692"].as_str().map(|s| !s.is_empty()).unwrap_or(false))
        .collect();

    let mut synced = upload_local_profiles(client, &local_rows, &remote_rows, user_id).await?;

    {
        let db = db.lock().map_err(|e| format!("Error de base de datos: {e}"))?;
        synced += write_remote_profiles(&db, &remote_rows, &local_rows)?;
    }

    Ok(synced)
}

/// Tipo que representa una fila de sesión local
///
/// Contiene todos los campos de la tabla de sesiones desde SQLite:
/// - id: ID único de la sesión
/// - profile_id: ID del perfil asociado
/// - status: Estado de la sesión (ej. "en_progreso", "completada")
/// - total_questions: Número total de preguntas en la sesión
/// - questions_answered: Número de preguntas respondidas
/// - correct_count: Número de respuestas correctas
/// - current_question_index: Índice de la pregunta actual (0-based)
/// - started_at: Timestamp ISO del inicio de la sesión
/// - ended_at: Timestamp ISO de fin de sesión (opcional, None si está en curso)
///
/// # Campos
/// * id: String, ID único de la sesión
/// * profile_id: String, ID del perfil que pertenece a esta sesión
/// * status: String, estado actual de la sesión
/// * total_questions: i64, número total de preguntas planificadas
/// * questions_answered: i64, número de preguntas completadas
/// * correct_count: i64, número de respuestas correctas
/// * current_question_index: i64, índice de la pregunta actual (0-based)
/// * started_at: String, fecha/hora de inicio en formato ISO
/// * ended_at: Option<String>, fecha/hora de finalización opcional en formato ISO
///
type LocalSessionRow = (String, String, String, i64, i64, i64, i64, String, Option<String>, String, Option<String>);

/// Lee todas las sesiones desde la base de datos local
///
/// # Argumentos
/// * `db` - Referencia a la conexión SQLite local
///
/// # Proceso
/// 1. Prepara una sentencia SQL para seleccionar todos los campos de la tabla sessions
/// 2. Ejecuta la consulta y mapea cada fila a una tupla LocalSessionRow
///    (id, profile_id, status, total_questions, questions_answered, correct_count,
///     current_question_index, started_at, ended_at, updated_at, deleted_at)
/// 3. Filtra las filas donde el ID está vacío
///
/// # Retorna
/// - Vec<LocalSessionRow> - Lista de sesiones locales
/// - Error - Si ocurre un error SQL
fn read_local_sessions(db: &Connection) -> Result<Vec<LocalSessionRow>, String> {
    let mut stmt = db
        .prepare(
            "SELECT id, profile_id, status, total_questions, questions_answered,
                    correct_count, current_question_index, started_at, ended_at,
                    updated_at, deleted_at
             FROM sessions ORDER BY id",
        )
        .map_err(|e| format!("Error leyendo sesiones locales: {e}"))?;
    let rows = stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, i64>(3)?,
                row.get::<_, i64>(4)?,
                row.get::<_, i64>(5)?,
                row.get::<_, i64>(6)?,
                row.get::<_, String>(7)?,
                row.get::<_, Option<String>>(8)?,
                row.get::<_, String>(9)?,
                row.get::<_, Option<String>>(10)?,
            ))
        })
        .map_err(|e| format!("Error leyendo sesiones locales: {e}"))?
        .filter_map(|r| r.ok())
        .filter(|(id, _, _, _, _, _, _, _, _, _, _)| !id.is_empty())
        .collect();
    Ok(rows)
}

/// Sube sesiones locales a Baserow
///
/// Para sesiones no borradas, las crea en remoto si no existen (upload-only).
/// Para sesiones borradas, las actualiza en remoto con `deleted_at` (PATCH)
/// o las crea con `deleted_at` si aun no existen.
///
/// # Argumentos
/// * `client` - Referencia al cliente HTTP de Baserow
/// * `local_rows` - Referencia a los tuples LocalSessionRow
/// * `remote_rows` - Referencia a los valores JSON de Baserow (tabla sesiones) por usuario
/// * `user_id` - ID del usuario que realiza la sincronización
///
/// # Retorna
/// - u32 - Número de filas sincronizadas
/// - Error - Si ocurre un error
async fn upload_local_sessions(
    client: &BaserowClient,
    local_rows: &[LocalSessionRow],
    remote_rows: &[Value],
    user_id: &str,
) -> Result<u32, String> {
    let mut synced = 0u32;
    for (id, profile_id, status, total_q, answered, correct, idx, started, ended, updated_at, deleted_at) in local_rows {
        let remote = remote_rows
            .iter()
            .find(|r| r["field_9480695"].as_str() == Some(id.as_str()));
        let now = now_rfc3339();
        let (is_deleted, deleted_field) = match deleted_at {
            Some(s) => (true, Value::String(s.clone())),
            None => (false, Value::Null),
        };
        match (remote, is_deleted) {
            // Borrada localmente y existe remoto: PATCH con deleted_at
            (Some(rem), true) => {
                let rem_updated = rem[FIELD_SESSIONS_UPDATED_AT].as_str().unwrap_or("");
                if updated_at.as_str() > rem_updated {
                    client
                        .update_row(
                            TABLE_USER_SESSIONS,
                            row_id(rem),
                            json!({
                                "field_9480695": id,
                                "field_9480717": user_id,
                                "field_9480718": profile_id,
                                "field_9480719": status,
                                "field_9480720": total_q,
                                "field_9480721": answered,
                                "field_9480722": correct,
                                "field_9480723": idx,
                                "field_9480724": started,
                                "field_9480725": ended,
                                FIELD_SESSIONS_UPDATED_AT: now,
                                FIELD_SESSIONS_DELETED_AT: deleted_field,
                            }),
                        )
                        .await?;
                    synced += 1;
                }
            }
            // Borrada localmente y no existe remoto: POST con deleted_at
            (None, true) => {
                client
                    .create_row(
                        TABLE_USER_SESSIONS,
                        json!({
                            "field_9480695": id,
                            "field_9480717": user_id,
                            "field_9480718": profile_id,
                            "field_9480719": status,
                            "field_9480720": total_q,
                            "field_9480721": answered,
                            "field_9480722": correct,
                            "field_9480723": idx,
                            "field_9480724": started,
                            "field_9480725": ended,
                            FIELD_SESSIONS_UPDATED_AT: now,
                            FIELD_SESSIONS_DELETED_AT: deleted_field,
                        }),
                    )
                    .await?;
                synced += 1;
            }
            // No borrada y no existe remoto: POST normal
            (None, false) => {
                client
                    .create_row(
                        TABLE_USER_SESSIONS,
                        json!({
                            "field_9480695": id,
                            "field_9480717": user_id,
                            "field_9480718": profile_id,
                            "field_9480719": status,
                            "field_9480720": total_q,
                            "field_9480721": answered,
                            "field_9480722": correct,
                            "field_9480723": idx,
                            "field_9480724": started,
                            "field_9480725": ended,
                            FIELD_SESSIONS_UPDATED_AT: now,
                        }),
                    )
                    .await?;
                synced += 1;
            }
            // No borrada y existe remoto: ganar si local es mas reciente
            (Some(rem), false) => {
                let rem_updated = rem[FIELD_SESSIONS_UPDATED_AT].as_str().unwrap_or("");
                if updated_at.as_str() > rem_updated {
                    client
                        .update_row(
                            TABLE_USER_SESSIONS,
                            row_id(rem),
                            json!({
                                "field_9480695": id,
                                "field_9480717": user_id,
                                "field_9480718": profile_id,
                                "field_9480719": status,
                                "field_9480720": total_q,
                                "field_9480721": answered,
                                "field_9480722": correct,
                                "field_9480723": idx,
                                "field_9480724": started,
                                "field_9480725": ended,
                                FIELD_SESSIONS_UPDATED_AT: now,
                            }),
                        )
                        .await?;
                    synced += 1;
                }
            }
        }
    }
    Ok(synced)
}

/// Aplica cambios remotos de sesiones a SQLite local (last-writer-wins por updated_at).
///
/// - Inserta sesiones remotas que no existen localmente (a menos que esten borradas)
/// - Si una sesion remota esta borrada y la local activa, marca borrada local si remota es mas reciente
/// - Si la local esta borrada y la remota activa, recupera local si remota es mas reciente
/// - Si ambas estan activas, gana la que tenga updated_at mas reciente
fn write_remote_sessions(
    db: &Connection,
    remote_rows: &[Value],
    local_rows: &[LocalSessionRow],
) -> Result<u32, String> {
    let mut synced = 0u32;
    for rem in remote_rows {
        let rem_session_id = rem["field_9480695"].as_str().unwrap_or("");
        if rem_session_id.is_empty() { continue; }
        let rem_deleted = rem[FIELD_SESSIONS_DELETED_AT].as_str().unwrap_or("");
        let rem_updated = rem[FIELD_SESSIONS_UPDATED_AT].as_str().unwrap_or("");

        let local = local_rows
            .iter()
            .find(|(id, _, _, _, _, _, _, _, _, _, _)| id == rem_session_id);

        match (local, rem_deleted.is_empty()) {
            // No existe local, no esta borrada remoto -> insertar con updated_at
            (None, true) => {
                let rem_profile_id = rem["field_9480718"].as_str().unwrap_or("");
                if rem_profile_id.is_empty() { continue; }
                let rem_status = rem["field_9480719"].as_str().unwrap_or("active");
                let rem_total = value_to_i64(&rem["field_9480720"]).unwrap_or(10);
                let rem_answered = value_to_i64(&rem["field_9480721"]).unwrap_or(0);
                let rem_correct = value_to_i64(&rem["field_9480722"]).unwrap_or(0);
                let rem_idx = value_to_i64(&rem["field_9480723"]).unwrap_or(0);
                let rem_started = rem["field_9480724"].as_str().unwrap_or("");
                let rem_ended = rem["field_9480725"].as_str().map(String::from);

                db.execute(
                    "INSERT OR IGNORE INTO sessions
                     (id, profile_id, status, total_questions, questions_answered,
                      correct_count, current_question_index, started_at, ended_at, updated_at)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                    params![
                        rem_session_id, rem_profile_id, rem_status, rem_total,
                        rem_answered, rem_correct, rem_idx, rem_started, rem_ended, rem_updated,
                    ],
                )
                .map_err(|e| format!("Error insertando sesion remota: {e}"))?;
                synced += 1;
            }
            // No existe local, esta borrada remoto -> saltar
            (None, false) => {}
            // Existe local, esta borrada remoto -> si la eliminacion remota es mas reciente, borrar local
            (Some(local), false) => {
                let local_updated = &local.9;
                if rem_updated > local_updated.as_str() {
                    db.execute(
                        "UPDATE sessions SET deleted_at = ?1, updated_at = ?1 WHERE id = ?2",
                        params![rem_deleted, rem_session_id],
                    )
                    .map_err(|e| format!("Error aplicando eliminacion remota de sesion: {e}"))?;
                    synced += 1;
                }
            }
            // Existe local, no esta borrada remoto -> ganar si remoto es mas reciente (incondicional)
            (Some(local), true) => {
                let local_updated = &local.9;
                if rem_updated > local_updated.as_str() {
                    let rem_profile_id = rem["field_9480718"].as_str().unwrap_or("");
                    let rem_status = rem["field_9480719"].as_str().unwrap_or("active");
                    let rem_total = value_to_i64(&rem["field_9480720"]).unwrap_or(10);
                    let rem_answered = value_to_i64(&rem["field_9480721"]).unwrap_or(0);
                    let rem_correct = value_to_i64(&rem["field_9480722"]).unwrap_or(0);
                    let rem_idx = value_to_i64(&rem["field_9480723"]).unwrap_or(0);
                    let rem_started = rem["field_9480724"].as_str().unwrap_or("");
                    let rem_ended = rem["field_9480725"].as_str().map(String::from);
                    db.execute(
                        "UPDATE sessions SET profile_id = ?1, status = ?2, total_questions = ?3,
                         questions_answered = ?4, correct_count = ?5, current_question_index = ?6,
                         started_at = ?7, ended_at = ?8, updated_at = ?9, deleted_at = NULL
                         WHERE id = ?10",
                        params![
                            rem_profile_id, rem_status, rem_total, rem_answered, rem_correct,
                            rem_idx, rem_started, rem_ended, rem_updated, rem_session_id,
                        ],
                    )
                    .map_err(|e| format!("Error actualizando sesion desde remoto: {e}"))?;
                    synced += 1;
                }
            }
        }
    }
    Ok(synced)
}

/// Sincroniza la tabla de sesiones
///
/// # Argumentos
/// * `client` - Referencia al cliente HTTP de Baserow
/// * `db` - Referencia con mutex a la conexión SQLite local
/// * `user_id` - ID del usuario que realiza la sincronización
///
/// # Proceso
/// 1. Lee todas las sesiones desde la base de datos local usando read_local_sessions
/// 2. Obtiene las filas remotas filtradas por usuario desde Baserow (tabla sesiones)
/// 3. Sube filas locales a Baserow usando upload_local_sessions
/// 4. Escribe filas remotas a SQLite local usando write_remote_sessions (bidireccional)
///
/// # Retorna
/// - u32 - Número de filas sincronizadas
/// - Error - Si ocurre un error en cualquier paso de sincronización
async fn sync_sessions_table(
    client: &BaserowClient,
    db: &Mutex<Connection>,
    user_id: &str,
) -> Result<u32, String> {
    let local_rows = {
        let db = db.lock().map_err(|e| format!("Error de base de datos: {e}"))?;
        read_local_sessions(&db)?
    };

    let remote_rows: Vec<Value> = client
        .list_rows(
            TABLE_USER_SESSIONS,
            &[("filter__field_9480717__equal", user_id)],
        )
        .await?
        .into_iter()
        .filter(|r| r["field_9480695"].as_str().map(|s| !s.is_empty()).unwrap_or(false))
        .collect();

    let mut synced = upload_local_sessions(client, &local_rows, &remote_rows, user_id).await?;

    {
        let db = db.lock().map_err(|e| format!("Error de base de datos: {e}"))?;
        synced += write_remote_sessions(&db, &remote_rows, &local_rows)?;
    }

    Ok(synced)
}

/// Tipo que representa una fila de pregunta de sesión local
///
/// Contiene todos los campos de la tabla de preguntas de sesión desde SQLite:
/// - id: ID único de la pregunta
/// - session_id: ID de la sesión a la que pertenece la pregunta
/// - question_text: Texto de la pregunta
/// - correct_answer: Respuesta correcta
/// - student_answer: Respuesta del estudiante (opcional durante lectura)
/// - concept: Concepto asociado a la pregunta
/// - difficulty: Dificultad de la pregunta (ej. "facil", "medio", "dificil")
/// - is_correct: Si la respuesta del estudiante fue correcta (opcional durante lectura)
/// - explanation: Explicación de la respuesta (opcional)
/// - question_number: Número de orden de la pregunta en la sesión
/// - time_spent_secs: Tiempo en segundos que el estudiante tardó en responder (opcional)
/// - created_at: Timestamp ISO de creación
/// - answered_at: Timestamp ISO de respuesta (opcional, None si no respondida)
///
/// # Campos
/// * id: String, ID único de la pregunta
/// * session_id: String, ID de la sesión que contiene esta pregunta
/// * question_text: String, texto completo de la pregunta
/// * correct_answer: String, respuesta correcta
/// * student_answer: Option<String>, respuesta del estudiante (opcional)
/// * concept: String, concepto educativo asociado
/// * difficulty: String, nivel de dificultad de la pregunta
/// * is_correct: Option<i64>, si la respuesta fue correcta (opcional)
/// * explanation: Option<String>, explicación de por qué es correcta (opcional)
/// * question_number: i64, posición de la pregunta en la sesión (1-indexed)
/// * time_spent_secs: Option<i64>, tiempo en segundos usado (opcional)
/// * created_at: String, timestamp de creación en formato ISO
/// * answered_at: Option<String>, timestamp de respuesta en formato ISO (opcional)
///
type LocalQuestionRow = (
    String, String, String, String, Option<String>, String, String,
    Option<i64>, Option<String>, i64, Option<i64>, String, Option<String>,
    String, Option<String>,
);

/// Lee todas las preguntas de sesión desde la base de datos local
///
/// # Argumentos
/// * `db` - Referencia a la conexión SQLite local
///
/// # Proceso
/// 1. Prepara una sentencia SQL para seleccionar todos los campos de la tabla session_questions
/// 2. Ejecuta la consulta y mapea cada fila a una tupla LocalQuestionRow
/// 3. Filtra las filas donde el ID está vacío
///
/// # Retorna
/// - Vec<LocalQuestionRow> - Lista de preguntas de sesión locales
/// - Error - Si ocurre un error SQL
fn read_local_session_questions(db: &Connection) -> Result<Vec<LocalQuestionRow>, String> {
    let mut stmt = db
        .prepare(
            "SELECT id, session_id, question_text, correct_answer,
                    student_answer, concept, difficulty, is_correct,
                    explanation, question_number, time_spent_secs,
                    created_at, answered_at, updated_at, deleted_at
             FROM session_questions ORDER BY id",
        )
        .map_err(|e| format!("Error leyendo preguntas locales: {e}"))?;
    let rows = stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, Option<String>>(4)?,
                row.get::<_, String>(5)?,
                row.get::<_, String>(6)?,
                row.get::<_, Option<i64>>(7)?,
                row.get::<_, Option<String>>(8)?,
                row.get::<_, i64>(9)?,
                row.get::<_, Option<i64>>(10)?,
                row.get::<_, String>(11)?,
                row.get::<_, Option<String>>(12)?,
                row.get::<_, String>(13)?,
                row.get::<_, Option<String>>(14)?,
            ))
        })
        .map_err(|e| format!("Error leyendo preguntas locales: {e}"))?
        .filter_map(|r| r.ok())
        .filter(|(id, _, _, _, _, _, _, _, _, _, _, _, _, _, _)| !id.is_empty())
        .collect();
    Ok(rows)
}

/// Sube preguntas de sesión locales a Baserow
///
/// # Argumentos
/// * `client` - Referencia al cliente HTTP de Baserow
/// * `local_rows` - Referencia a los tuples LocalQuestionRow (id, session_id, question_text, correct_answer, student_answer, concept, difficulty, is_correct, explanation, question_number, time_spent_secs, created_at, answered_at)
/// * `remote_rows` - Referencia a los valores JSON de Baserow (tabla session_questions) por usuario
/// * `user_id` - ID del usuario que realiza la sincronización
///
/// # Proceso
/// 1. Para cada pregunta local, verifica si ya existe en Baserow por ID (field_9480698)
/// 2. Si NO existe, crea una nueva fila remota con todos los datos locales
///    - ID de usuario (field_9480726), ID de sesión (field_9480727), texto de la pregunta (field_9480728), etc.
/// 3. Solo crea filas que no existan (para evitar duplicados)
///
/// # Retorna
/// - u32 - Número de nuevas filas remotas creadas
/// - Error - Si ocurre un error al crear
async fn upload_local_session_questions(
    client: &BaserowClient,
    local_rows: &[LocalQuestionRow],
    remote_rows: &[Value],
    user_id: &str,
) -> Result<u32, String> {
    let mut synced = 0u32;
    for (id, session_id, question_text, correct_answer, student_answer, concept, difficulty, is_correct, explanation, question_number, time_spent_secs, created_at, answered_at, updated_at, deleted_at) in local_rows
    {
        let remote = remote_rows
            .iter()
            .find(|r| r["field_9480698"].as_str() == Some(id.as_str()));
        let now = now_rfc3339();
        let (is_deleted, deleted_field) = match deleted_at {
            Some(s) => (true, Value::String(s.clone())),
            None => (false, Value::Null),
        };
        match (remote, is_deleted) {
            (Some(rem), true) => {
                let rem_updated = rem[FIELD_QUESTIONS_UPDATED_AT].as_str().unwrap_or("");
                if updated_at.as_str() > rem_updated {
                    client
                        .update_row(
                            TABLE_USER_SESSION_QUESTIONS,
                            row_id(rem),
                            json!({
                                "field_9480698": id,
                                "field_9480726": user_id,
                                "field_9480727": session_id,
                                "field_9480728": question_text,
                                "field_9480729": correct_answer,
                                "field_9480730": student_answer,
                                "field_9480731": concept,
                                "field_9480732": difficulty,
                                "field_9480733": is_correct,
                                "field_9480734": explanation,
                                "field_9480735": question_number,
                                "field_9480736": time_spent_secs,
                                "field_9480737": created_at,
                                "field_9480738": answered_at,
                                FIELD_QUESTIONS_UPDATED_AT: now,
                                FIELD_QUESTIONS_DELETED_AT: deleted_field,
                            }),
                        )
                        .await?;
                    synced += 1;
                }
            }
            (None, true) => {
                client
                    .create_row(
                        TABLE_USER_SESSION_QUESTIONS,
                        json!({
                            "field_9480698": id,
                            "field_9480726": user_id,
                            "field_9480727": session_id,
                            "field_9480728": question_text,
                            "field_9480729": correct_answer,
                            "field_9480730": student_answer,
                            "field_9480731": concept,
                            "field_9480732": difficulty,
                            "field_9480733": is_correct,
                            "field_9480734": explanation,
                            "field_9480735": question_number,
                            "field_9480736": time_spent_secs,
                            "field_9480737": created_at,
                            "field_9480738": answered_at,
                            FIELD_QUESTIONS_UPDATED_AT: now,
                            FIELD_QUESTIONS_DELETED_AT: deleted_field,
                        }),
                    )
                    .await?;
                synced += 1;
            }
            (None, false) => {
                client
                    .create_row(
                        TABLE_USER_SESSION_QUESTIONS,
                        json!({
                            "field_9480698": id,
                            "field_9480726": user_id,
                            "field_9480727": session_id,
                            "field_9480728": question_text,
                            "field_9480729": correct_answer,
                            "field_9480730": student_answer,
                            "field_9480731": concept,
                            "field_9480732": difficulty,
                            "field_9480733": is_correct,
                            "field_9480734": explanation,
                            "field_9480735": question_number,
                            "field_9480736": time_spent_secs,
                            "field_9480737": created_at,
                            "field_9480738": answered_at,
                            FIELD_QUESTIONS_UPDATED_AT: now,
                        }),
                    )
                    .await?;
                synced += 1;
            }
            (Some(rem), false) => {
                let rem_updated = rem[FIELD_QUESTIONS_UPDATED_AT].as_str().unwrap_or("");
                if updated_at.as_str() > rem_updated {
                    client
                        .update_row(
                            TABLE_USER_SESSION_QUESTIONS,
                            row_id(rem),
                            json!({
                                "field_9480698": id,
                                "field_9480726": user_id,
                                "field_9480727": session_id,
                                "field_9480728": question_text,
                                "field_9480729": correct_answer,
                                "field_9480730": student_answer,
                                "field_9480731": concept,
                                "field_9480732": difficulty,
                                "field_9480733": is_correct,
                                "field_9480734": explanation,
                                "field_9480735": question_number,
                                "field_9480736": time_spent_secs,
                                "field_9480737": created_at,
                                "field_9480738": answered_at,
                                FIELD_QUESTIONS_UPDATED_AT: now,
                            }),
                        )
                        .await?;
                    synced += 1;
                }
            }
        }
    }
    Ok(synced)
}

/// Aplica cambios remotos de preguntas a SQLite local (last-writer-wins por updated_at).
///
/// - Inserta preguntas remotas que no existen localmente (a menos que esten borradas)
/// - Si una pregunta remota esta borrada y la local activa, marca borrada local si remota es mas reciente
/// - Si la local esta borrada y la remota activa, recupera local si remota es mas reciente
/// - Si ambas estan activas, gana la que tenga updated_at mas reciente
fn write_remote_session_questions(
    db: &Connection,
    remote_rows: &[Value],
    local_rows: &[LocalQuestionRow],
) -> Result<u32, String> {
    let mut synced = 0u32;
    for rem in remote_rows {
        let rem_question_id = rem["field_9480698"].as_str().unwrap_or("");
        if rem_question_id.is_empty() { continue; }
        let rem_deleted = rem[FIELD_QUESTIONS_DELETED_AT].as_str().unwrap_or("");
        let rem_updated = rem[FIELD_QUESTIONS_UPDATED_AT].as_str().unwrap_or("");

        let local = local_rows
            .iter()
            .find(|(id, _, _, _, _, _, _, _, _, _, _, _, _, _, _)| id == rem_question_id);

        match (local, rem_deleted.is_empty()) {
            // No existe local, no esta borrada remoto -> insertar con updated_at
            (None, true) => {
                let rem_session = rem["field_9480727"].as_str().unwrap_or("");
                if rem_session.is_empty() { continue; }
                let rem_text = rem["field_9480728"].as_str().unwrap_or("");
                let rem_correct = rem["field_9480729"].as_str().unwrap_or("");
                let rem_student = rem["field_9480730"].as_str().map(String::from);
                let rem_concept = rem["field_9480731"].as_str().unwrap_or("");
                let rem_diff = rem["field_9480732"].as_str().unwrap_or("");
                let rem_is_correct = value_to_i64(&rem["field_9480733"]).map(|v| v != 0);
                let rem_expl = rem["field_9480734"].as_str().map(String::from);
                let rem_qnum = value_to_i64(&rem["field_9480735"]).unwrap_or(0);
                let rem_time = value_to_i64(&rem["field_9480736"]).map(|v| v as u32);
                let rem_cr = rem["field_9480737"].as_str().unwrap_or("");
                let rem_answered = rem["field_9480738"].as_str().map(String::from);

                db.execute(
                    "INSERT OR IGNORE INTO session_questions
                     (id, session_id, question_text, correct_answer, student_answer,
                      concept, difficulty, is_correct, explanation, question_number,
                      time_spent_secs, created_at, answered_at, updated_at)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
                    params![
                        rem_question_id, rem_session, rem_text, rem_correct,
                        rem_student, rem_concept, rem_diff, rem_is_correct,
                        rem_expl, rem_qnum, rem_time, rem_cr, rem_answered, rem_updated,
                    ],
                )
                .map_err(|e| format!("Error insertando pregunta remota: {e}"))?;
                synced += 1;
            }
            // No existe local, esta borrada remoto -> saltar
            (None, false) => {}
            // Existe local, esta borrada remoto -> si la eliminacion remota es mas reciente, borrar local
            (Some(local), false) => {
                let local_updated = &local.13;
                if rem_updated > local_updated.as_str() {
                    db.execute(
                        "UPDATE session_questions SET deleted_at = ?1, updated_at = ?1 WHERE id = ?2",
                        params![rem_deleted, rem_question_id],
                    )
                    .map_err(|e| format!("Error aplicando eliminacion remota de pregunta: {e}"))?;
                    synced += 1;
                }
            }
            // Existe local, no esta borrada remoto -> ganar si remoto es mas reciente (incondicional)
            (Some(local), true) => {
                let local_updated = &local.13;
                if rem_updated > local_updated.as_str() {
                    let rem_session = rem["field_9480727"].as_str().unwrap_or("");
                    let rem_text = rem["field_9480728"].as_str().unwrap_or("");
                    let rem_correct = rem["field_9480729"].as_str().unwrap_or("");
                    let rem_student = rem["field_9480730"].as_str().map(String::from);
                    let rem_concept = rem["field_9480731"].as_str().unwrap_or("");
                    let rem_diff = rem["field_9480732"].as_str().unwrap_or("");
                    let rem_is_correct = value_to_i64(&rem["field_9480733"]).map(|v| v != 0);
                    let rem_expl = rem["field_9480734"].as_str().map(String::from);
                    let rem_qnum = value_to_i64(&rem["field_9480735"]).unwrap_or(0);
                    let rem_time = value_to_i64(&rem["field_9480736"]).map(|v| v as u32);
                    let rem_cr = rem["field_9480737"].as_str().unwrap_or("");
                    let rem_answered = rem["field_9480738"].as_str().map(String::from);
                    db.execute(
                        "UPDATE session_questions SET session_id = ?1, question_text = ?2,
                         correct_answer = ?3, student_answer = ?4, concept = ?5,
                         difficulty = ?6, is_correct = ?7, explanation = ?8,
                         question_number = ?9, time_spent_secs = ?10, created_at = ?11,
                         answered_at = ?12, updated_at = ?13, deleted_at = NULL
                         WHERE id = ?14",
                        params![
                            rem_session, rem_text, rem_correct, rem_student, rem_concept,
                            rem_diff, rem_is_correct, rem_expl, rem_qnum, rem_time,
                            rem_cr, rem_answered, rem_updated, rem_question_id,
                        ],
                    )
                    .map_err(|e| format!("Error actualizando pregunta desde remoto: {e}"))?;
                    synced += 1;
                }
            }
        }
    }
    Ok(synced)
}

/// Sincroniza la tabla de preguntas de sesión
///
/// # Argumentos
/// * `client` - Referencia al cliente HTTP de Baserow
/// * `db` - Referencia con mutex a la conexión SQLite local
/// * `user_id` - ID del usuario que realiza la sincronización
///
/// # Proceso
/// 1. Lee todas las preguntas de sesión desde la base de datos local usando read_local_session_questions
/// 2. Obtiene las filas remotas filtradas por usuario desde Baserow (tabla session_questions)
/// 3. Sube filas locales a Baserow usando upload_local_session_questions
/// 4. Escribe filas remotas a SQLite local usando write_remote_session_questions (bidireccional)
///
/// # Retorna
/// - u32 - Número de filas sincronizadas
/// - Error - Si ocurre un error en cualquier paso de sincronización
async fn sync_session_questions_table(
    client: &BaserowClient,
    db: &Mutex<Connection>,
    user_id: &str,
) -> Result<u32, String> {
    let local_rows = {
        let db = db.lock().map_err(|e| format!("Error de base de datos: {e}"))?;
        read_local_session_questions(&db)?
    };

    let remote_rows: Vec<Value> = client
        .list_rows(
            TABLE_USER_SESSION_QUESTIONS,
            &[("filter__field_9480726__equal", user_id)],
        )
        .await?
        .into_iter()
        .filter(|r| r["field_9480698"].as_str().map(|s| !s.is_empty()).unwrap_or(false))
        .collect();

    let mut synced = upload_local_session_questions(client, &local_rows, &remote_rows, user_id).await?;

    {
        let db = db.lock().map_err(|e| format!("Error de base de datos: {e}"))?;
        synced += write_remote_session_questions(&db, &remote_rows, &local_rows)?;
    }

    Ok(synced)
}

/// Ejecuta la sincronización completa de todas las tablas entre el almacenamiento local y Baserow
///
/// # Argumentos
/// * `client` - Referencia al cliente HTTP de Baserow para comunicación con la API
/// * `db` - Referencia con mutex a la conexión SQLite local que contiene los datos del usuario
/// * `user_id` - ID del usuario cuya información será sincronizada
///
/// # Proceso
/// 1. Sincroniza la tabla de configuración (configuración de la aplicación, unidireccional desde local hacia remoto)
/// 2. Sincroniza la tabla de perfiles (bidireccional con overwrite por fecha de actualización)
/// 3. Sincroniza la tabla de sesiones (unidireccional desde local hacia remoto)
/// 4. Sincroniza la tabla de preguntas de sesión (unidireccional desde local hacia remoto)
/// 5. Agrega todos los errores encontrados a un vector de errores
/// 6. Retorna una estructura SyncResult con estadísticas de lo sincronizado y cualquier error
///
/// # Retorna
/// - SyncResult - Estructurado con:
///   - config_synced: número de filas sincronizadas en la tabla de configuración
///   - profiles_synced: número de perfiles sincronizados
///   - sessions_synced: número de sesiones sincronizadas
///   - session_questions_synced: número de preguntas de sesión sincronizadas
///   - errors: vector de mensajes de error encontrados durante la sincronización
///
/// # Errores
/// Retorna SyncResult con los errores en el campo `errors`, en lugar de fallar abruptamente.
/// - Errores de red o de API durante llamadas a Baserow
/// - Errores de bases de datos SQLite durante operaciones de lectura/escritura
/// - Errores de deserialización de JSON
pub async fn sync_all(
    client: &BaserowClient,
    db: &Mutex<Connection>,
    user_id: &str,
) -> SyncResult {
    let mut errors = Vec::new();

    let config = sync_config_table(client, db, user_id).await.unwrap_or_else(|e| {
        errors.push(format!("config: {e}"));
        0
    });

    let profiles = sync_profiles_table(client, db, user_id).await.unwrap_or_else(|e| {
        errors.push(format!("profiles: {e}"));
        0
    });

    let sessions = sync_sessions_table(client, db, user_id).await.unwrap_or_else(|e| {
        errors.push(format!("sessions: {e}"));
        0
    });

    let questions = sync_session_questions_table(client, db, user_id).await.unwrap_or_else(|e| {
        errors.push(format!("session_questions: {e}"));
        0
    });

    SyncResult {
        config_synced: config,
        profiles_synced: profiles,
        sessions_synced: sessions,
        session_questions_synced: questions,
        errors,
    }
}

// ============================================================================
// Funciones para "Forzar desde nube"
// ============================================================================

/// Sube configuraciones locales que no existen en remoto (solo create).
async fn upload_new_local_config(
    client: &BaserowClient,
    local_rows: &[(String, String, String)],
    remote_rows: &[Value],
    user_id: &str,
) -> Result<u32, String> {
    let mut synced = 0u32;
    for (key, value, _) in local_rows {
        let exists = remote_rows
            .iter()
            .any(|r| r["field_9480705"].as_str() == Some(key.as_str()));
        if !exists {
            let now = now_rfc3339();
            client
                .create_row(
                    TABLE_USER_CONFIG,
                    serde_json::json!({
                        "field_9480689": user_id,
                        "field_9480705": key,
                        "field_9480706": value,
                        "field_9480707": now,
                    }),
                )
                .await?;
            synced += 1;
        }
    }
    Ok(synced)
}

/// Sube perfiles locales que no existen en remoto (solo create).
async fn upload_new_local_profiles(
    client: &BaserowClient,
    local_rows: &[(String, String, u8, Option<u8>, String, u8, Option<String>, String, String, Option<String>)],
    remote_rows: &[Value],
    user_id: &str,
) -> Result<u32, String> {
    let mut synced = 0u32;
    for (id, display_name, school_year, age, level_mode, current_level, manual_prompt, created_at, _updated_at, deleted_at) in local_rows
    {
        let exists = remote_rows
            .iter()
            .any(|r| r["field_9480692"].as_str() == Some(id.as_str()));
        if !exists {
            let now = now_rfc3339();
            let deleted_field = match deleted_at {
                Some(s) => Value::String(s.clone()),
                None => Value::Null,
            };
            client
                .create_row(
                    TABLE_USER_PROFILES,
                    json!({
                        "field_9480692": id,
                        "field_9480708": user_id,
                        "field_9480709": display_name,
                        "field_9480710": school_year,
                        "field_9480711": age,
                        "field_9480712": level_mode,
                        "field_9480713": current_level,
                        "field_9480714": manual_prompt,
                        "field_9480715": created_at,
                        "field_9480716": now,
                        FIELD_PROFILES_DELETED_AT: deleted_field,
                    }),
                )
                .await?;
            synced += 1;
        }
    }
    Ok(synced)
}

/// Sube sesiones locales que no existen en remoto (solo create).
async fn upload_new_local_sessions(
    client: &BaserowClient,
    local_rows: &[LocalSessionRow],
    remote_rows: &[Value],
    user_id: &str,
) -> Result<u32, String> {
    let mut synced = 0u32;
    for (id, profile_id, status, total_q, answered, correct, idx, started, ended, _, deleted_at) in local_rows {
        let exists = remote_rows
            .iter()
            .any(|r| r["field_9480695"].as_str() == Some(id.as_str()));
        if !exists {
            let now = now_rfc3339();
            let deleted_field = match deleted_at {
                Some(s) => Value::String(s.clone()),
                None => Value::Null,
            };
            client
                .create_row(
                    TABLE_USER_SESSIONS,
                    json!({
                        "field_9480695": id,
                        "field_9480717": user_id,
                        "field_9480718": profile_id,
                        "field_9480719": status,
                        "field_9480720": total_q,
                        "field_9480721": answered,
                        "field_9480722": correct,
                        "field_9480723": idx,
                        "field_9480724": started,
                        "field_9480725": ended,
                        FIELD_SESSIONS_UPDATED_AT: now,
                        FIELD_SESSIONS_DELETED_AT: deleted_field,
                    }),
                )
                .await?;
            synced += 1;
        }
    }
    Ok(synced)
}

/// Sube preguntas de sesion locales que no existen en remoto (solo create).
async fn upload_new_local_session_questions(
    client: &BaserowClient,
    local_rows: &[LocalQuestionRow],
    remote_rows: &[Value],
    user_id: &str,
) -> Result<u32, String> {
    let mut synced = 0u32;
    for (id, session_id, question_text, correct_answer, student_answer, concept, difficulty, is_correct, explanation, question_number, time_spent_secs, created_at, answered_at, _, deleted_at) in local_rows
    {
        let exists = remote_rows
            .iter()
            .any(|r| r["field_9480698"].as_str() == Some(id.as_str()));
        if !exists {
            let now = now_rfc3339();
            let deleted_field = match deleted_at {
                Some(s) => Value::String(s.clone()),
                None => Value::Null,
            };
            client
                .create_row(
                    TABLE_USER_SESSION_QUESTIONS,
                    json!({
                        "field_9480698": id,
                        "field_9480726": user_id,
                        "field_9480727": session_id,
                        "field_9480728": question_text,
                        "field_9480729": correct_answer,
                        "field_9480730": student_answer,
                        "field_9480731": concept,
                        "field_9480732": difficulty,
                        "field_9480733": is_correct,
                        "field_9480734": explanation,
                        "field_9480735": question_number,
                        "field_9480736": time_spent_secs,
                        "field_9480737": created_at,
                        "field_9480738": answered_at,
                        FIELD_QUESTIONS_UPDATED_AT: now,
                        FIELD_QUESTIONS_DELETED_AT: deleted_field,
                    }),
                )
                .await?;
            synced += 1;
        }
    }
    Ok(synced)
}

/// Sobrescribe configuracion local con datos remotos (incondicional).
fn force_write_remote_config(
    db: &Connection,
    remote_rows: &[Value],
    _local_rows: &[(String, String, String)],
) -> Result<u32, String> {
    let mut synced = 0u32;
    for rem in remote_rows {
        let rem_key = rem["field_9480705"].as_str().unwrap_or("");
        if rem_key.is_empty() { continue; }
        let rem_val = rem["field_9480706"].as_str().unwrap_or("");
        let rem_updated = rem["field_9480707"].as_str().unwrap_or("");
        db.execute(
            "INSERT INTO app_settings (key, value, updated_at)
             VALUES (?1, ?2, ?3)
             ON CONFLICT(key) DO UPDATE
             SET value = excluded.value, updated_at = excluded.updated_at",
            params![rem_key, rem_val, rem_updated],
        )
        .map_err(|e| format!("Error sobrescribiendo config desde remoto: {e}"))?;
        synced += 1;
    }
    Ok(synced)
}

/// Sobrescribe perfiles locales con datos remotos (incondicional).
fn force_write_remote_profiles(
    db: &Connection,
    remote_rows: &[Value],
    local_rows: &[(String, String, u8, Option<u8>, String, u8, Option<String>, String, String, Option<String>)],
) -> Result<u32, String> {
    let mut synced = 0u32;
    for rem in remote_rows {
        let rem_profile_id = rem["field_9480692"].as_str().unwrap_or("");
        if rem_profile_id.is_empty() { continue; }
        let rem_deleted = rem[FIELD_PROFILES_DELETED_AT].as_str().unwrap_or("");
        let rem_updated = rem["field_9480716"].as_str().unwrap_or("");
        let rem_display = rem["field_9480709"].as_str().unwrap_or("");
        let rem_school = value_to_i64(&rem["field_9480710"]).unwrap_or(1) as u8;
        let rem_age = value_to_i64(&rem["field_9480711"]).map(|v| v as u8);
        let rem_lm = rem["field_9480712"].as_str().unwrap_or("automatic");
        let rem_curr = value_to_i64(&rem["field_9480713"]).unwrap_or(1) as u8;
        let rem_manual = rem["field_9480714"].as_str().map(String::from);
        let rem_cr = rem["field_9480715"].as_str().unwrap_or("");

        let local = local_rows
            .iter()
            .find(|(id, _, _, _, _, _, _, _, _, _)| id == rem_profile_id);

        // Si remoto esta borrado pero local se recupero mas tarde, preservar recuperacion
        if !rem_deleted.is_empty() {
            if let Some(local) = local {
                let local_updated = &local.8;
                if local_updated.as_str() > rem_deleted {
                    continue
                }
            }
            db.execute(
                "UPDATE profiles SET deleted_at = ?1, updated_at = ?1 WHERE id = ?2",
                params![rem_deleted, rem_profile_id],
            )
            .map_err(|e| format!("Error aplicando eliminacion remota de perfil: {e}"))?;
            synced += 1;
        } else {
            db.execute(
                "INSERT INTO profiles (id, display_name, school_year, age, level_mode,
                 current_level, manual_prompt, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
                 ON CONFLICT(id) DO UPDATE SET
                 display_name = excluded.display_name, school_year = excluded.school_year,
                 age = excluded.age, level_mode = excluded.level_mode,
                 current_level = excluded.current_level, manual_prompt = excluded.manual_prompt,
                 created_at = excluded.created_at, updated_at = excluded.updated_at,
                 deleted_at = NULL",
                params![
                    rem_profile_id, rem_display, rem_school, rem_age,
                    rem_lm, rem_curr, rem_manual, rem_cr, rem_updated,
                ],
            )
            .map_err(|e| format!("Error sobrescribiendo perfil desde remoto: {e}"))?;
            synced += 1;
        }
    }
    Ok(synced)
}

/// Sobrescribe sesiones locales con datos remotos (incondicional).
fn force_write_remote_sessions(
    db: &Connection,
    remote_rows: &[Value],
    local_rows: &[LocalSessionRow],
) -> Result<u32, String> {
    let mut synced = 0u32;
    for rem in remote_rows {
        let rem_session_id = rem["field_9480695"].as_str().unwrap_or("");
        if rem_session_id.is_empty() { continue; }
        let rem_deleted = rem[FIELD_SESSIONS_DELETED_AT].as_str().unwrap_or("");
        let rem_updated = rem[FIELD_SESSIONS_UPDATED_AT].as_str().unwrap_or("");

        if !rem_deleted.is_empty() {
            // Preservar recuperacion local mas reciente
            if let Some(local) = local_rows.iter().find(|(id, _, _, _, _, _, _, _, _, _, _)| id == rem_session_id) {
                if local.9.as_str() > rem_deleted {
                    continue
                }
            }
            db.execute(
                "UPDATE sessions SET deleted_at = ?1, updated_at = ?1 WHERE id = ?2",
                params![rem_deleted, rem_session_id],
            )
            .map_err(|e| format!("Error marcando sesion borrada desde remoto: {e}"))?;
            synced += 1;
        } else {
            let rem_profile_id = rem["field_9480718"].as_str().unwrap_or("");
            let rem_status = rem["field_9480719"].as_str().unwrap_or("active");
            let rem_total = value_to_i64(&rem["field_9480720"]).unwrap_or(10);
            let rem_answered = value_to_i64(&rem["field_9480721"]).unwrap_or(0);
            let rem_correct = value_to_i64(&rem["field_9480722"]).unwrap_or(0);
            let rem_idx = value_to_i64(&rem["field_9480723"]).unwrap_or(0);
            let rem_started = rem["field_9480724"].as_str().unwrap_or("");
            let rem_ended = rem["field_9480725"].as_str().map(String::from);
            db.execute(
                "INSERT INTO sessions (id, profile_id, status, total_questions,
                 questions_answered, correct_count, current_question_index,
                 started_at, ended_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
                 ON CONFLICT(id) DO UPDATE SET
                 profile_id = excluded.profile_id, status = excluded.status,
                 total_questions = excluded.total_questions,
                 questions_answered = excluded.questions_answered,
                 correct_count = excluded.correct_count,
                 current_question_index = excluded.current_question_index,
                 started_at = excluded.started_at, ended_at = excluded.ended_at,
                 updated_at = excluded.updated_at, deleted_at = NULL",
                params![
                    rem_session_id, rem_profile_id, rem_status, rem_total,
                    rem_answered, rem_correct, rem_idx, rem_started, rem_ended, rem_updated,
                ],
            )
            .map_err(|e| format!("Error sobrescribiendo sesion desde remoto: {e}"))?;
            synced += 1;
        }
    }
    Ok(synced)
}

/// Sobrescribe preguntas de sesion locales con datos remotos (incondicional).
fn force_write_remote_session_questions(
    db: &Connection,
    remote_rows: &[Value],
    local_rows: &[LocalQuestionRow],
) -> Result<u32, String> {
    let mut synced = 0u32;
    for rem in remote_rows {
        let rem_question_id = rem["field_9480698"].as_str().unwrap_or("");
        if rem_question_id.is_empty() { continue; }
        let rem_deleted = rem[FIELD_QUESTIONS_DELETED_AT].as_str().unwrap_or("");
        let rem_updated = rem[FIELD_QUESTIONS_UPDATED_AT].as_str().unwrap_or("");

        if !rem_deleted.is_empty() {
            // Preservar recuperacion local mas reciente
            if let Some(local) = local_rows.iter().find(|(id, _, _, _, _, _, _, _, _, _, _, _, _, _, _)| id == rem_question_id) {
                if local.13.as_str() > rem_deleted {
                    continue
                }
            }
            db.execute(
                "UPDATE session_questions SET deleted_at = ?1, updated_at = ?1 WHERE id = ?2",
                params![rem_deleted, rem_question_id],
            )
            .map_err(|e| format!("Error marcando pregunta borrada desde remoto: {e}"))?;
            synced += 1;
        } else {
            let rem_session = rem["field_9480727"].as_str().unwrap_or("");
            let rem_text = rem["field_9480728"].as_str().unwrap_or("");
            let rem_correct = rem["field_9480729"].as_str().unwrap_or("");
            let rem_student = rem["field_9480730"].as_str().map(String::from);
            let rem_concept = rem["field_9480731"].as_str().unwrap_or("");
            let rem_diff = rem["field_9480732"].as_str().unwrap_or("");
            let rem_is_correct = value_to_i64(&rem["field_9480733"]).map(|v| v != 0);
            let rem_expl = rem["field_9480734"].as_str().map(String::from);
            let rem_qnum = value_to_i64(&rem["field_9480735"]).unwrap_or(0);
            let rem_time = value_to_i64(&rem["field_9480736"]).map(|v| v as u32);
            let rem_cr = rem["field_9480737"].as_str().unwrap_or("");
            let rem_answered = rem["field_9480738"].as_str().map(String::from);
            db.execute(
                "INSERT INTO session_questions
                 (id, session_id, question_text, correct_answer, student_answer,
                  concept, difficulty, is_correct, explanation, question_number,
                  time_spent_secs, created_at, answered_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)
                 ON CONFLICT(id) DO UPDATE SET
                 session_id = excluded.session_id, question_text = excluded.question_text,
                 correct_answer = excluded.correct_answer, student_answer = excluded.student_answer,
                 concept = excluded.concept, difficulty = excluded.difficulty,
                 is_correct = excluded.is_correct, explanation = excluded.explanation,
                 question_number = excluded.question_number,
                 time_spent_secs = excluded.time_spent_secs,
                 created_at = excluded.created_at, answered_at = excluded.answered_at,
                 updated_at = excluded.updated_at, deleted_at = NULL",
                params![
                    rem_question_id, rem_session, rem_text, rem_correct,
                    rem_student, rem_concept, rem_diff, rem_is_correct,
                    rem_expl, rem_qnum, rem_time, rem_cr, rem_answered, rem_updated,
                ],
            )
            .map_err(|e| format!("Error sobrescribiendo pregunta desde remoto: {e}"))?;
            synced += 1;
        }
    }
    Ok(synced)
}

/// Sincroniza config forzadamente: sube solo locales nuevas, descarga todo sobrescribiendo.
async fn force_sync_config_table(
    client: &BaserowClient,
    db: &Mutex<Connection>,
    user_id: &str,
) -> Result<u32, String> {
    let local_rows = {
        let db = db.lock().map_err(|e| format!("Error de base de datos: {e}"))?;
        read_local_app_settings(&db)?
    };
    let remote_rows: Vec<Value> = client
        .list_rows(
            TABLE_USER_CONFIG,
            &[("filter__field_9480689__equal", user_id)],
        )
        .await?
        .into_iter()
        .filter(|r| {
            r["field_9480689"].as_str().map(|s| !s.is_empty()).unwrap_or(false)
            && r["field_9480705"].as_str().map(|s| !s.is_empty()).unwrap_or(false)
        })
        .collect();

    let mut synced = upload_new_local_config(client, &local_rows, &remote_rows, user_id).await?;
    {
        let db = db.lock().map_err(|e| format!("Error de base de datos: {e}"))?;
        synced += force_write_remote_config(&db, &remote_rows, &local_rows)?;
    }
    Ok(synced)
}

/// Sincroniza perfiles forzadamente: sube solo locales nuevos, descarga todo sobrescribiendo.
async fn force_sync_profiles_table(
    client: &BaserowClient,
    db: &Mutex<Connection>,
    user_id: &str,
) -> Result<u32, String> {
    let local_rows = {
        let db = db.lock().map_err(|e| format!("Error de base de datos: {e}"))?;
        read_local_profiles(&db)?
    };
    let remote_rows: Vec<Value> = client
        .list_rows(
            TABLE_USER_PROFILES,
            &[("filter__field_9480708__equal", user_id)],
        )
        .await?
        .into_iter()
        .filter(|r| r["field_9480692"].as_str().map(|s| !s.is_empty()).unwrap_or(false))
        .collect();

    let mut synced = upload_new_local_profiles(client, &local_rows, &remote_rows, user_id).await?;
    {
        let db = db.lock().map_err(|e| format!("Error de base de datos: {e}"))?;
        synced += force_write_remote_profiles(&db, &remote_rows, &local_rows)?;
    }
    Ok(synced)
}

/// Sincroniza sesiones forzadamente: sube solo locales nuevas, descarga todo sobrescribiendo.
async fn force_sync_sessions_table(
    client: &BaserowClient,
    db: &Mutex<Connection>,
    user_id: &str,
) -> Result<u32, String> {
    let local_rows = {
        let db = db.lock().map_err(|e| format!("Error de base de datos: {e}"))?;
        read_local_sessions(&db)?
    };
    let remote_rows: Vec<Value> = client
        .list_rows(
            TABLE_USER_SESSIONS,
            &[("filter__field_9480717__equal", user_id)],
        )
        .await?
        .into_iter()
        .filter(|r| r["field_9480695"].as_str().map(|s| !s.is_empty()).unwrap_or(false))
        .collect();

    let mut synced = upload_new_local_sessions(client, &local_rows, &remote_rows, user_id).await?;
    {
        let db = db.lock().map_err(|e| format!("Error de base de datos: {e}"))?;
        synced += force_write_remote_sessions(&db, &remote_rows, &local_rows)?;
    }
    Ok(synced)
}

/// Sincroniza preguntas forzadamente: sube solo locales nuevas, descarga todo sobrescribiendo.
async fn force_sync_session_questions_table(
    client: &BaserowClient,
    db: &Mutex<Connection>,
    user_id: &str,
) -> Result<u32, String> {
    let local_rows = {
        let db = db.lock().map_err(|e| format!("Error de base de datos: {e}"))?;
        read_local_session_questions(&db)?
    };
    let remote_rows: Vec<Value> = client
        .list_rows(
            TABLE_USER_SESSION_QUESTIONS,
            &[("filter__field_9480726__equal", user_id)],
        )
        .await?
        .into_iter()
        .filter(|r| r["field_9480698"].as_str().map(|s| !s.is_empty()).unwrap_or(false))
        .collect();

    let mut synced = upload_new_local_session_questions(client, &local_rows, &remote_rows, user_id).await?;
    {
        let db = db.lock().map_err(|e| format!("Error de base de datos: {e}"))?;
        synced += force_write_remote_session_questions(&db, &remote_rows, &local_rows)?;
    }
    Ok(synced)
}

/// Ejecuta sincronizacion forzada desde la nube.
///
/// 1. Sube datos locales que no existen en remoto (solo create)
/// 2. Sobrescribe datos locales con datos remotos (incondicional)
pub async fn force_sync_all(
    client: &BaserowClient,
    db: &Mutex<Connection>,
    user_id: &str,
) -> SyncResult {
    let mut errors = Vec::new();

    let config = force_sync_config_table(client, db, user_id).await.unwrap_or_else(|e| {
        errors.push(format!("config: {e}"));
        0
    });

    let profiles = force_sync_profiles_table(client, db, user_id).await.unwrap_or_else(|e| {
        errors.push(format!("profiles: {e}"));
        0
    });

    let sessions = force_sync_sessions_table(client, db, user_id).await.unwrap_or_else(|e| {
        errors.push(format!("sessions: {e}"));
        0
    });

    let questions = force_sync_session_questions_table(client, db, user_id).await.unwrap_or_else(|e| {
        errors.push(format!("session_questions: {e}"));
        0
    });

    SyncResult {
        config_synced: config,
        profiles_synced: profiles,
        sessions_synced: sessions,
        session_questions_synced: questions,
        errors,
    }
}
