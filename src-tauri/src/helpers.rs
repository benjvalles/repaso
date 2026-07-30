
use argon2::password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString};
use argon2::Argon2;
use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension};

use crate::models::*;

/// Crea todas las tablas de la base de datos si no existen (esquema completo).
///
/// # Argumentos
///
/// * `db` - Conexion a la base de datos SQLite.
///
/// # Retorna
///
/// * `Ok(())` si el esquema se creo correctamente.
/// * `Err(String)` si ocurrio un error SQL.
pub fn setup_database(db: &Connection) -> Result<(), String> {
    db.execute_batch(
        "PRAGMA foreign_keys = ON;

         CREATE TABLE IF NOT EXISTS app_settings (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL,
            updated_at TEXT NOT NULL
         );

         CREATE TABLE IF NOT EXISTS profiles (
            id TEXT PRIMARY KEY,
            display_name TEXT NOT NULL,
            school_year INTEGER NOT NULL CHECK (school_year BETWEEN 1 AND 6),
            age INTEGER CHECK (age IS NULL OR age BETWEEN 6 AND 12),
            level_mode TEXT NOT NULL CHECK (level_mode IN ('automatic', 'manual')),
            current_level INTEGER NOT NULL CHECK (current_level BETWEEN 1 AND 6),
            manual_prompt TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            deleted_at TEXT
         );

         CREATE TABLE IF NOT EXISTS sessions (
            id TEXT PRIMARY KEY,
            profile_id TEXT NOT NULL REFERENCES profiles(id),
            status TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'completed', 'abandoned')),
            total_questions INTEGER NOT NULL DEFAULT 10,
            questions_answered INTEGER NOT NULL DEFAULT 0,
            correct_count INTEGER NOT NULL DEFAULT 0,
            current_question_index INTEGER NOT NULL DEFAULT 0,
            started_at TEXT NOT NULL,
            ended_at TEXT,
            updated_at TEXT NOT NULL DEFAULT '',
            deleted_at TEXT
         );

         CREATE TABLE IF NOT EXISTS session_questions (
            id TEXT PRIMARY KEY,
            session_id TEXT NOT NULL REFERENCES sessions(id),
            question_text TEXT NOT NULL,
            correct_answer TEXT NOT NULL,
            student_answer TEXT,
            concept TEXT NOT NULL,
            difficulty TEXT NOT NULL,
            is_correct INTEGER,
            explanation TEXT,
            needs_reformulation INTEGER DEFAULT 0,
            reformulated_text TEXT,
            question_number INTEGER NOT NULL,
            time_spent_secs INTEGER,
            created_at TEXT NOT NULL,
            answered_at TEXT,
            updated_at TEXT NOT NULL DEFAULT '',
            deleted_at TEXT
         );

         CREATE TABLE IF NOT EXISTS users (
            id TEXT PRIMARY KEY,
            display_name TEXT NOT NULL,
            pin_hash TEXT NOT NULL,
            role TEXT NOT NULL CHECK (role IN ('parent', 'tutor', 'admin')),
            created_at TEXT NOT NULL
         );

         CREATE TABLE IF NOT EXISTS student_groups (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            owner_user_id TEXT NOT NULL REFERENCES users(id),
            created_at TEXT NOT NULL
         );

         CREATE TABLE IF NOT EXISTS student_group_members (
            group_id TEXT NOT NULL REFERENCES student_groups(id) ON DELETE CASCADE,
            student_id TEXT NOT NULL REFERENCES profiles(id) ON DELETE CASCADE,
            PRIMARY KEY (group_id, student_id)
         );

         CREATE TABLE IF NOT EXISTS tutor_student (
            tutor_user_id TEXT NOT NULL REFERENCES users(id),
            student_id TEXT NOT NULL REFERENCES profiles(id),
            assigned_at TEXT NOT NULL,
            status TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'inactive')),
            PRIMARY KEY (tutor_user_id, student_id)
         );

         CREATE TABLE IF NOT EXISTS parent_student (
            parent_user_id TEXT NOT NULL REFERENCES users(id),
            student_id TEXT NOT NULL REFERENCES profiles(id),
            assigned_at TEXT NOT NULL,
            PRIMARY KEY (parent_user_id, student_id)
         );

         CREATE TABLE IF NOT EXISTS assignments (
            id TEXT PRIMARY KEY,
            tutor_user_id TEXT NOT NULL REFERENCES users(id),
            student_id TEXT NOT NULL REFERENCES profiles(id),
            concept TEXT NOT NULL,
            difficulty TEXT NOT NULL,
            due_date TEXT,
            status TEXT NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'in_progress', 'completed', 'cancelled')),
            created_at TEXT NOT NULL
         );

         CREATE TABLE IF NOT EXISTS reports (
            id TEXT PRIMARY KEY,
            tutor_user_id TEXT NOT NULL REFERENCES users(id),
            student_id TEXT NOT NULL REFERENCES profiles(id),
            period TEXT NOT NULL,
            report_data TEXT NOT NULL,
            generated_at TEXT NOT NULL
         );",
    ).map_err(|err| format!("No se pudo preparar SQLite: {err}"))?;
    ensure_profile_manual_prompt_column(db)?;
    ensure_soft_delete_columns(db)?;
    Ok(())
}

/// Asegura que bases de datos existentes tengan el campo de contexto pedagogico manual.
///
/// # Argumentos
///
/// * `db` - Conexion a la base de datos SQLite.
///
/// # Retorna
///
/// * `Ok(())` si la migracion fue exitosa o no era necesaria.
/// * `Err(String)` si fallo la comprobacion o la alteracion.
fn ensure_profile_manual_prompt_column(db: &Connection) -> Result<(), String> {
    let mut stmt = db.prepare("PRAGMA table_info(profiles)")
        .map_err(|err| format!("No se pudo revisar perfiles: {err}"))?;
    let rows = stmt.query_map([], |row| row.get::<_, String>(1))
        .map_err(|err| format!("No se pudieron leer columnas de perfiles: {err}"))?;
    let mut has_column = false;
    for row in rows {
        if row.map_err(|err| format!("Columna de perfil invalida: {err}"))? == "manual_prompt" {
            has_column = true;
            break;
        }
    }
    if !has_column {
        db.execute("ALTER TABLE profiles ADD COLUMN manual_prompt TEXT", [])
            .map_err(|err| format!("No se pudo migrar perfiles: {err}"))?;
    }
    Ok(())
}

/// Asegura que las tablas tengan las columnas `deleted_at` y `updated_at` para soft-delete.
///
/// # Argumentos
///
/// * `db` - Conexion a la base de datos SQLite.
///
/// # Retorna
///
/// * `Ok(())` si la migracion fue exitosa o no era necesaria.
/// * `Err(String)` si fallo la comprobacion o la alteracion.
fn ensure_soft_delete_columns(db: &Connection) -> Result<(), String> {
    // Migrar perfiles: agregar deleted_at si no existe
    if !has_column(db, "profiles", "deleted_at")? {
        db.execute("ALTER TABLE profiles ADD COLUMN deleted_at TEXT", [])
            .map_err(|err| format!("No se pudo migrar perfiles (deleted_at): {err}"))?;
    }
    // Migrar sesiones: agregar updated_at si no existe
    if !has_column(db, "sessions", "updated_at")? {
        db.execute("ALTER TABLE sessions ADD COLUMN updated_at TEXT NOT NULL DEFAULT ''", [])
            .map_err(|err| format!("No se pudo migrar sesiones (updated_at): {err}"))?;
    }
    // Migrar sesiones: agregar deleted_at si no existe
    if !has_column(db, "sessions", "deleted_at")? {
        db.execute("ALTER TABLE sessions ADD COLUMN deleted_at TEXT", [])
            .map_err(|err| format!("No se pudo migrar sesiones (deleted_at): {err}"))?;
    }
    // Migrar preguntas: agregar updated_at si no existe
    if !has_column(db, "session_questions", "updated_at")? {
        db.execute("ALTER TABLE session_questions ADD COLUMN updated_at TEXT NOT NULL DEFAULT ''", [])
            .map_err(|err| format!("No se pudo migrar preguntas (updated_at): {err}"))?;
    }
    // Migrar preguntas: agregar deleted_at si no existe
    if !has_column(db, "session_questions", "deleted_at")? {
        db.execute("ALTER TABLE session_questions ADD COLUMN deleted_at TEXT", [])
            .map_err(|err| format!("No se pudo migrar preguntas (deleted_at): {err}"))?;
    }
    Ok(())
}

/// Comprueba si una columna existe en una tabla.
fn has_column(db: &Connection, table: &str, column: &str) -> Result<bool, String> {
    let mut stmt = db
        .prepare(&format!("PRAGMA table_info({table})"))
        .map_err(|err| format!("No se pudo revisar {table}: {err}"))?;
    let rows = stmt
        .query_map([], |row| row.get::<_, String>(1))
        .map_err(|err| format!("No se pudieron leer columnas de {table}: {err}"))?;
    for row in rows {
        if row.map_err(|err| format!("Columna invalida en {table}: {err}"))? == column {
            return Ok(true);
        }
    }
    Ok(false)
}

/// Obtiene el valor de una clave de configuracion en `app_settings`.
///
/// # Argumentos
///
/// * `db` - Conexion a la base de datos SQLite.
/// * `key` - Clave de configuracion a consultar.
///
/// # Retorna
///
/// * `Ok(Some(String))` si la clave existe.
/// * `Ok(None)` si la clave no existe.
/// * `Err(String)` si fallo la consulta.
pub fn get_setting(db: &Connection, key: &str) -> Result<Option<String>, String> {
    db.query_row("SELECT value FROM app_settings WHERE key = ?1", params![key], |row| row.get(0))
        .optional()
        .map_err(|err| format!("No se pudo leer la configuracion: {err}"))
}

/// Guarda o actualiza una clave de configuracion en `app_settings`.
///
/// # Argumentos
///
/// * `db` - Conexion a la base de datos SQLite.
/// * `key` - Clave de configuracion.
/// * `value` - Valor a almacenar.
///
/// # Retorna
///
/// * `Ok(())` si se guardo correctamente.
/// * `Err(String)` si fallo la operacion.
pub fn set_setting(db: &Connection, key: &str, value: &str) -> Result<(), String> {
    let now = Utc::now().to_rfc3339();
    db.execute(
        "INSERT INTO app_settings (key, value, updated_at) VALUES (?1, ?2, ?3)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = excluded.updated_at",
        params![key, value, now],
    ).map_err(|err| format!("No se pudo guardar la configuracion: {err}"))?;
    Ok(())
}

/// Construye un `Profile` desde una fila de la tabla `profiles`.
///
/// # Argumentos
///
/// * `row` - Fila de resultado de SQLite con las columnas del perfil.
///
/// # Retorna
///
/// * `Ok(Profile)` si la conversion fue exitosa.
/// * `Err(rusqlite::Error)` si fallo la lectura de algun campo.
pub fn profile_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Profile> {
    let level_mode: String = row.get(4)?;
    Ok(Profile {
        id: row.get(0)?,
        display_name: row.get(1)?,
        school_year: row.get(2)?,
        age: row.get(3)?,
        level_mode: LevelMode::from_db(level_mode).map_err(|message| {
            rusqlite::Error::FromSqlConversionFailure(4, rusqlite::types::Type::Text, Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, message)))
        })?,
        current_level: row.get(5)?,
        manual_prompt: row.get(6)?,
        created_at: row.get(7)?,
        updated_at: row.get(8)?,
    })
}

/// Obtiene un perfil por su identificador.
///
/// # Argumentos
///
/// * `db` - Conexion a la base de datos SQLite.
/// * `id` - Identificador unico del perfil.
///
/// # Retorna
///
/// * `Ok(Profile)` si se encontro el perfil.
/// * `Err(String)` si no existe o fallo la consulta.
pub fn get_profile_by_id(db: &Connection, id: &str) -> Result<Profile, String> {
    db.query_row(
         "SELECT id, display_name, school_year, age, level_mode, current_level, manual_prompt, created_at, updated_at
          FROM profiles WHERE id = ?1 AND deleted_at IS NULL",
        params![id],
        profile_from_row,
    ).optional()
    .map_err(|err| format!("No se pudo leer el perfil: {err}"))?
    .ok_or_else(|| "Perfil no encontrado".to_string())
}

/// Lista todos los perfiles desde la base de datos, ordenados por curso y nombre.
///
/// # Argumentos
///
/// * `db` - Conexion a la base de datos SQLite.
///
/// # Retorna
///
/// * `Ok(Vec<Profile>)` con la lista de perfiles.
/// * `Err(String)` si falla la preparacion o lectura de la consulta.
pub fn list_profiles_from_db(db: &Connection) -> Result<Vec<Profile>, String> {
    let mut stmt = db.prepare(
        "SELECT id, display_name, school_year, age, level_mode, current_level, manual_prompt, created_at, updated_at
         FROM profiles WHERE deleted_at IS NULL ORDER BY school_year, display_name COLLATE NOCASE",
    ).map_err(|err| format!("No se pudieron preparar los perfiles: {err}"))?;

    let rows = stmt.query_map([], profile_from_row)
        .map_err(|err| format!("No se pudieron leer los perfiles: {err}"))?;

    let mut profiles = Vec::new();
    for row in rows {
        profiles.push(row.map_err(|err| format!("Perfil invalido: {err}"))?);
    }
    Ok(profiles)
}

/// Lista los perfiles eliminados (soft-delete) desde la base de datos, ordenados por fecha de eliminacion.
///
/// # Argumentos
///
/// * `db` - Conexion a la base de datos SQLite.
///
/// # Retorna
///
/// * `Ok(Vec<Profile>)` con la lista de perfiles eliminados.
/// * `Err(String)` si falla la preparacion o lectura de la consulta.
pub fn list_deleted_profiles_from_db(db: &Connection) -> Result<Vec<Profile>, String> {
    let mut stmt = db.prepare(
        "SELECT id, display_name, school_year, age, level_mode, current_level, manual_prompt, created_at, updated_at
         FROM profiles WHERE deleted_at IS NOT NULL ORDER BY deleted_at DESC",
    ).map_err(|err| format!("No se pudieron preparar los perfiles eliminados: {err}"))?;

    let rows = stmt.query_map([], profile_from_row)
        .map_err(|err| format!("No se pudieron leer los perfiles eliminados: {err}"))?;

    let mut profiles = Vec::new();
    for row in rows {
        profiles.push(row.map_err(|err| format!("Perfil invalido: {err}"))?);
    }
    Ok(profiles)
}

/// Obtiene una sesion por su identificador.
///
/// # Argumentos
///
/// * `db` - Conexion a la base de datos SQLite.
/// * `id` - Identificador unico de la sesion.
///
/// # Retorna
///
/// * `Ok(Session)` si se encuentra la sesion.
/// * `Err(String)` si no existe o falla la consulta.
pub fn get_session_by_id(db: &Connection, id: &str) -> Result<Session, String> {
    db.query_row(
        "SELECT id, profile_id, status, total_questions, questions_answered, correct_count, current_question_index, started_at, ended_at
         FROM sessions WHERE id = ?1 AND deleted_at IS NULL",
        params![id],
        |row| Ok(Session {
            id: row.get(0)?,
            profile_id: row.get(1)?,
            status: row.get(2)?,
            total_questions: row.get(3)?,
            questions_answered: row.get(4)?,
            correct_count: row.get(5)?,
            current_question_index: row.get(6)?,
            started_at: row.get(7)?,
            ended_at: row.get(8)?,
        }),
    ).optional()
    .map_err(|err| format!("No se pudo leer la sesion: {err}"))?
    .ok_or_else(|| "Sesion no encontrada".to_string())
}

/// Obtiene una pregunta de sesion por su identificador.
///
/// # Argumentos
///
/// * `db` - Conexion a la base de datos SQLite.
/// * `id` - Identificador unico de la pregunta.
///
/// # Retorna
///
/// * `Ok(SessionQuestion)` si se encuentra la pregunta.
/// * `Err(String)` si no existe o falla la consulta.
pub fn get_question_by_id(db: &Connection, id: &str) -> Result<SessionQuestion, String> {
    db.query_row(
        "SELECT id, session_id, question_text, correct_answer, student_answer, concept, difficulty, is_correct, explanation, needs_reformulation, reformulated_text, question_number, time_spent_secs, created_at, answered_at
         FROM session_questions WHERE id = ?1 AND deleted_at IS NULL",
        params![id],
        |row| Ok(SessionQuestion {
            id: row.get(0)?,
            session_id: row.get(1)?,
            question_text: row.get(2)?,
            correct_answer: row.get(3)?,
            student_answer: row.get(4)?,
            concept: row.get(5)?,
            difficulty: row.get(6)?,
            is_correct: row.get(7)?,
            explanation: row.get(8)?,
            needs_reformulation: row.get(9)?,
            reformulated_text: row.get(10)?,
            question_number: row.get(11)?,
            time_spent_secs: row.get(12)?,
            created_at: row.get(13)?,
            answered_at: row.get(14)?,
        }),
    ).optional()
    .map_err(|err| format!("No se pudo leer la pregunta: {err}"))?
    .ok_or_else(|| "Pregunta no encontrada".to_string())
}

/// Lista todas las preguntas de una sesion, ordenadas por numero de pregunta.
///
/// # Argumentos
///
/// * `db` - Conexion a la base de datos SQLite.
/// * `session_id` - Identificador unico de la sesion.
///
/// # Retorna
///
/// * `Ok(Vec<SessionQuestion>)` con la lista de preguntas.
/// * `Err(String)` si falla la preparacion o lectura de la consulta.
pub fn list_questions_for_session(db: &Connection, session_id: &str) -> Result<Vec<SessionQuestion>, String> {
    let mut stmt = db.prepare(
        "SELECT id, session_id, question_text, correct_answer, student_answer, concept, difficulty, is_correct, explanation, needs_reformulation, reformulated_text, question_number, time_spent_secs, created_at, answered_at
         FROM session_questions WHERE session_id = ?1 AND deleted_at IS NULL ORDER BY question_number",
    ).map_err(|err| format!("No se pudieron preparar las preguntas: {err}"))?;

    let rows = stmt.query_map(params![session_id], |row| {
        Ok(SessionQuestion {
            id: row.get(0)?,
            session_id: row.get(1)?,
            question_text: row.get(2)?,
            correct_answer: row.get(3)?,
            student_answer: row.get(4)?,
            concept: row.get(5)?,
            difficulty: row.get(6)?,
            is_correct: row.get(7)?,
            explanation: row.get(8)?,
            needs_reformulation: row.get(9)?,
            reformulated_text: row.get(10)?,
            question_number: row.get(11)?,
            time_spent_secs: row.get(12)?,
            created_at: row.get(13)?,
            answered_at: row.get(14)?,
        })
    }).map_err(|err| format!("No se pudieron leer las preguntas: {err}"))?;

    let mut questions = Vec::new();
    for row in rows {
        questions.push(row.map_err(|err| format!("Pregunta invalida: {err}"))?);
    }
    Ok(questions)
}

/// Verifica que la zona adulta este desbloqueada; error si no.
///
/// # Argumentos
///
/// * `state` - Estado compartido de Tauri que implementa `HasAdultUnlocked`.
///
/// # Retorna
///
/// * `Ok(())` si la zona adulta esta desbloqueada.
/// * `Err(String)` si esta bloqueada o no se pudo comprobar.
pub fn require_adult_unlocked<T>(state: &tauri::State<'_, T>) -> Result<(), String>
where
    T: HasAdultUnlocked + Send + Sync,
{
    let unlocked = *state.adult_unlocked().lock().map_err(|_| "No se pudo comprobar la sesion adulta")?;
    if unlocked { Ok(()) } else { Err("La zona adulta esta bloqueada".to_string()) }
}

/// Valida que un PIN sea numerico y tenga entre 4 y 6 digitos.
///
/// # Argumentos
///
/// * `pin` - PIN en texto plano a validar.
///
/// # Retorna
///
/// * `Ok(())` si el PIN es valido.
/// * `Err(String)` si no cumple el formato requerido.
pub fn validate_pin(pin: &str) -> Result<(), String> {
    if !(4..=6).contains(&pin.len()) || !pin.chars().all(|c| c.is_ascii_digit()) {
        return Err("El PIN debe tener entre 4 y 6 digitos".to_string());
    }
    Ok(())
}

/// Genera un hash seguro del PIN usando Argon2.
///
/// # Argumentos
///
/// * `pin` - PIN en texto plano a hashear.
///
/// # Retorna
///
/// * `Ok(String)` con el hash generado.
/// * `Err(String)` si falla el proceso de hasheo.
pub fn hash_pin(pin: &str) -> Result<String, String> {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(pin.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|err| format!("No se pudo proteger el PIN: {err}"))
}

/// Verifica un PIN contra su hash almacenado usando Argon2.
///
/// # Argumentos
///
/// * `pin` - PIN en texto plano a verificar.
/// * `pin_hash` - Hash almacenado contra el que comparar.
///
/// # Retorna
///
/// * `Ok(true)` si el PIN coincide con el hash.
/// * `Ok(false)` si no coincide.
/// * `Err(String)` si el hash almacenado es invalido.
pub fn verify_pin(pin: &str, pin_hash: &str) -> Result<bool, String> {
    let parsed_hash = PasswordHash::new(pin_hash).map_err(|err| format!("PIN guardado invalido: {err}"))?;
    Ok(Argon2::default().verify_password(pin.as_bytes(), &parsed_hash).is_ok())
}

/// Valida los campos de entrada al crear o actualizar un perfil de estudiante.
///
/// # Argumentos
///
/// * `display_name` - Nombre visible del perfil.
/// * `school_year` - Curso escolar (1 a 6).
/// * `age` - Edad opcional del estudiante (6 a 12).
/// * `level_mode` - Modo de nivel (automatico o manual).
/// * `manual_level` - Nivel manual opcional.
/// * `manual_prompt` - Contexto pedagogico opcional.
///
/// # Retorna
///
/// * `Ok(())` si todos los campos son validos.
/// * `Err(String)` con el mensaje de error del primer campo invalido.
pub fn validate_profile_input(
    display_name: &str,
    school_year: u8,
    age: Option<u8>,
    level_mode: LevelMode,
    manual_level: Option<u8>,
    manual_prompt: Option<&str>,
) -> Result<(), String> {
    let name = display_name.trim();
    if name.len() < 2 || name.len() > 40 {
        return Err("El nombre del perfil debe tener entre 2 y 40 caracteres".to_string());
    }
    if !(1..=6).contains(&school_year) {
        return Err("El curso debe estar entre 1o y 6o de primaria".to_string());
    }
    if let Some(age) = age {
        if !(6..=12).contains(&age) {
            return Err("La edad debe estar entre 6 y 12 anos para Primaria".to_string());
        }
    }
    if matches!(level_mode, LevelMode::Manual) && manual_level.is_none() {
        return Err("El modo manual necesita un nivel".to_string());
    }
    if let Some(manual_level) = manual_level {
        if !(school_year..=6).contains(&manual_level) {
            return Err("El nivel manual no puede ser inferior al curso del perfil".to_string());
        }
    }
    if let Some(manual_prompt) = manual_prompt {
        if manual_prompt.trim().len() > 1000 {
            return Err("El contexto pedagogico no puede superar 1000 caracteres".to_string());
        }
    }
    Ok(())
}

/// Normaliza el contexto pedagogico manual y lo descarta en modo automatico.
///
/// # Argumentos
///
/// * `level_mode` - Modo de nivel del perfil.
/// * `manual_prompt` - Contexto pedagogico opcional a normalizar.
///
/// # Retorna
///
/// * `Some(String)` con el texto normalizado si el modo es manual y el texto no esta vacio.
/// * `None` si el modo es automatico o el texto esta vacio.
pub fn resolve_manual_prompt(level_mode: LevelMode, manual_prompt: Option<String>) -> Option<String> {
    if level_mode != LevelMode::Manual {
        return None;
    }
    manual_prompt.map(|prompt| prompt.trim().to_string()).filter(|prompt| !prompt.is_empty())
}

/// Devuelve el contexto pedagogico aplicable a llamadas LLM del perfil.
///
/// # Argumentos
///
/// * `profile` - Perfil del estudiante.
///
/// # Retorna
///
/// * `Some(String)` con el contexto pedagogico si el modo es manual.
/// * `None` si el modo es automatico.
pub fn manual_prompt_for_profile(profile: &Profile) -> Option<String> {
    if profile.level_mode == LevelMode::Manual {
        profile.manual_prompt.clone()
    } else {
        None
    }
}

/// Resuelve el nivel actual segun el modo: automatico usa el curso, manual usa el nivel manual.
///
/// # Argumentos
///
/// * `school_year` - Curso escolar del perfil.
/// * `level_mode` - Modo de nivel (automatico o manual).
/// * `manual_level` - Nivel manual opcional.
///
/// # Retorna
///
/// * `u8` con el nivel resuelto.
pub fn resolve_current_level(school_year: u8, level_mode: LevelMode, manual_level: Option<u8>) -> u8 {
    match level_mode {
        LevelMode::Automatic => school_year,
        LevelMode::Manual => manual_level.unwrap_or(school_year).max(school_year),
    }
}

/// Obtiene el concepto con peor rendimiento para un perfil, basado en el historial de sesiones.
///
/// # Argumentos
///
/// * `db` - Conexion a la base de datos SQLite.
/// * `profile_id` - Identificador unico del perfil.
///
/// # Retorna
///
/// * `Some(String)` con el nombre del concepto mas debil.
/// * `None` si no hay suficientes datos o falla la consulta.
pub fn get_weakest_concept(db: &Connection, profile_id: &str) -> Option<String> {
    let mut stmt = db.prepare(
         "SELECT concept, COUNT(*) as total, SUM(CASE WHEN is_correct = 1 THEN 1 ELSE 0 END) as correct
          FROM session_questions sq
          JOIN sessions s ON sq.session_id = s.id
          WHERE s.profile_id = ?1 AND sq.is_correct IS NOT NULL
          AND s.deleted_at IS NULL AND sq.deleted_at IS NULL
         GROUP BY concept
         HAVING total >= 2
         ORDER BY (correct * 1.0 / total) ASC
         LIMIT 1",
    ).ok()?;

    let mut rows = stmt.query_map(params![profile_id], |row| {
        Ok(row.get::<_, String>(0)?)
    }).ok()?;

    rows.next().and_then(|r| r.ok())
}

/// Devuelve un concepto predeterminado para un curso cuando no hay historial.
///
/// # Argumentos
///
/// * `year` - Curso escolar (1 a 6).
///
/// # Retorna
///
/// * `String` con el concepto predeterminado para el curso.
pub fn get_default_concept_for_year(year: u8) -> String {
    match year {
        1 => "sumas y restas sencillas".to_string(),
        2 => "sumas con llevada y restas sin llevada".to_string(),
        3 => "multiplicacion y division sencilla".to_string(),
        4 => "multiplicacion por varias cifras y division con resto".to_string(),
        5 => "fracciones y decimales".to_string(),
        6 => "porcentajes y fracciones equivalentes".to_string(),
        _ => "operaciones basicas".to_string(),
    }
}

/// Evalua localmente una respuesta comparandola con la correcta usando normalizacion de texto y numerica.
///
/// # Argumentos
///
/// * `correct_answer` - Respuesta correcta esperada.
/// * `student_answer` - Respuesta proporcionada por el estudiante.
///
/// # Retorna
///
/// * `(bool, String)` donde el booleano indica acierto y el string contiene la retroalimentacion.
pub fn evaluate_answer_local(correct_answer: &str, student_answer: &str) -> (bool, String) {
    let normalize = |s: &str| -> String {
        s.trim().to_lowercase()
            .replace(",", ".")
            .replace(" ", "")
    };

    let normalized_correct = normalize(correct_answer);
    let normalized_student = normalize(student_answer);

    if normalized_correct == normalized_student {
        return (true, "¡Correcto! Bien hecho.".to_string());
    }

    if let (Ok(correct_num), Ok(student_num)) = (
        normalized_correct.parse::<f64>(),
        normalized_student.parse::<f64>(),
    ) {
        if (correct_num - student_num).abs() < 0.001 {
            return (true, "¡Correcto! Bien hecho.".to_string());
        }
    }

    if let (Some(correct_num), Some(student_num)) = (
        expected_numeric_result(correct_answer),
        final_numeric_result(student_answer),
    ) {
        if (correct_num - student_num).abs() < 0.001 {
            return (true, "¡Correcto! Bien hecho.".to_string());
        }
    }

    let feedback = match normalized_correct.parse::<f64>() {
        Ok(correct_num) => {
            let diff = correct_num - normalized_student.parse::<f64>().unwrap_or(0.0);
            if diff.abs() < 2.0 {
                "¡Casi! Muy cerca. Revisa tu calculo.".to_string()
            } else {
                format!("La respuesta correcta es {}. No te preocupes, revisa el proceso y vuelve a intentarlo.", correct_answer)
            }
        }
        Err(_) => {
            format!("La respuesta correcta es {}. Intentalo de nuevo con calma.", correct_answer)
        }
    };

    (false, feedback)
}

/// Extrae el resultado numerico esperado de una respuesta correcta con texto.
///
/// # Argumentos
///
/// * `answer` - Respuesta correcta con posible texto y numeros.
///
/// # Retorna
///
/// * `Some(f64)` con el numero extraido segun la posicion.
/// * `None` si no se encuentra ningun numero.
fn expected_numeric_result(answer: &str) -> Option<f64> {
    let numbers = extract_numbers(answer);
    if numbers.is_empty() {
        return None;
    }

    if answer.trim_start().chars().next().is_some_and(|c| c.is_ascii_digit() || c == '-' || c == '+') {
        numbers.first().copied()
    } else {
        numbers.last().copied()
    }
}

/// Extrae el resultado numerico final escrito por el alumno.
///
/// # Argumentos
///
/// * `answer` - Respuesta del estudiante.
///
/// # Retorna
///
/// * `Some(f64)` con el ultimo numero encontrado.
/// * `None` si no se encuentra ningun numero.
fn final_numeric_result(answer: &str) -> Option<f64> {
    extract_numbers(answer).last().copied()
}

/// Extrae numeros enteros o decimales de un texto usando coma o punto decimal.
///
/// # Argumentos
///
/// * `answer` - Texto del cual extraer numeros.
///
/// # Retorna
///
/// * `Vec<f64>` con los numeros encontrados en orden de aparicion.
fn extract_numbers(answer: &str) -> Vec<f64> {
    let mut numbers = Vec::new();
    let mut current = String::new();
    let mut has_digit = false;

    for c in answer.chars() {
        if c.is_ascii_digit() {
            current.push(c);
            has_digit = true;
        } else if (c == ',' || c == '.') && has_digit && !current.contains('.') {
            current.push('.')
        } else if (c == '-' || c == '+') && current.is_empty() {
            current.push(c)
        } else if has_digit {
            if let Ok(number) = current.parse::<f64>() {
                numbers.push(number);
            }
            current.clear();
            has_digit = false;
        } else {
            current.clear();
        }
    }

    if has_digit {
        if let Ok(number) = current.parse::<f64>() {
            numbers.push(number);
        }
    }

    numbers
}

/// Genera una explicacion generica para una pregunta cuando el LLM no proporciona una personalizada.
///
/// # Argumentos
///
/// * `question` - Pregunta de sesion con texto, concepto y respuesta correcta.
///
/// # Retorna
///
/// * `String` con la explicacion generada.
pub fn generate_default_explanation(question: &SessionQuestion) -> String {
    format!(
        "El problema \"{}\" se resuelve con el concepto de {}. La respuesta correcta es {}.",
        question.question_text, question.concept, question.correct_answer
    )
}

/// Obtiene estadisticas por concepto para un perfil desde la base de datos.
///
/// # Argumentos
///
/// * `db` - Conexion a la base de datos SQLite.
/// * `profile_id` - Identificador unico del perfil.
///
/// # Retorna
///
/// * `Ok(Vec<ConceptStat>)` con las estadisticas ordenadas por precision ascendente.
/// * `Err(rusqlite::Error)` si falla la consulta.
pub fn get_concept_stats_for_profile(db: &rusqlite::Connection, profile_id: &str) -> Result<Vec<ConceptStat>, rusqlite::Error> {
    let mut stmt = db.prepare(
        "SELECT sq.concept,
                COUNT(*) as total,
                SUM(CASE WHEN sq.is_correct = 1 THEN 1 ELSE 0 END) as correct,
                MAX(sq.answered_at) as last_practiced
         FROM session_questions sq
         JOIN sessions s ON sq.session_id = s.id
         WHERE s.profile_id = ?1 AND s.status = 'completed' AND sq.is_correct IS NOT NULL
         AND s.deleted_at IS NULL AND sq.deleted_at IS NULL
         GROUP BY sq.concept
         ORDER BY correct * 1.0 / COUNT(*) ASC",
    )?;

    let rows = stmt.query_map(params![profile_id], |row| {
        let concept: String = row.get(0)?;
        let total: u32 = row.get(1)?;
        let correct: u32 = row.get(2)?;
        let last_practiced: String = row.get(3)?;
        let accuracy = if total > 0 { (correct as f64 / total as f64) * 100.0 } else { 0.0 };
        Ok(ConceptStat {
            concept,
            total_attempts: total,
            correct_attempts: correct,
            accuracy_pct: accuracy,
            last_practiced,
        })
    })?;

    let mut stats = Vec::new();
    for row in rows {
        stats.push(row?);
    }
    Ok(stats)
}
