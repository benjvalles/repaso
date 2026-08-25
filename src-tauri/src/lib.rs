mod cloud;
mod email;
mod helpers;
mod llm;
mod models;

use std::env;
use std::fs;
use std::sync::Mutex;

use argon2::password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString};
use argon2::Argon2;
use chrono::Utc;
use cloud::{BaserowClient, CloudSession, CloudStatus};
use email::{
    delete_scheduled_email, get_email_content, get_email_status, list_transac_emails,
    send_transac_email, EmailClient,
};
use llm::{LLMExplanation, LLMProviderEnum, LLMQuestion};
use llm::commands::{build_provider, load_llm_config};
use rusqlite::{params, Connection};
use tauri::{Manager, State};
use uuid::Uuid;

use helpers::*;
use models::*;

struct AppState {
    db: Mutex<Connection>,
    adult_unlocked: Mutex<bool>,
    llm_provider: Mutex<Option<LLMProviderEnum>>,
    llm_config: Mutex<LLMConfig>,
    locale: Mutex<String>,
    baserow_client: Mutex<Option<BaserowClient>>,
    cloud_session: Mutex<Option<CloudSession>>,
    email_client: Mutex<Option<EmailClient>>,
}

impl HasAdultUnlocked for AppState {
    fn adult_unlocked(&self) -> &Mutex<bool> {
        &self.adult_unlocked
    }
}

// ==================== COMMANDS ====================

#[tauri::command]
fn get_app_status(state: State<'_, AppState>) -> Result<AppStatus, String> {
    let db = state.db.lock().map_err(|_| "No se pudo acceder a la base de datos")?;
    let adult_unlocked = *state
        .adult_unlocked
        .lock()
        .map_err(|_| "No se pudo comprobar la sesion adulta")?;
    let llm_config = state
        .llm_config
        .lock()
        .map_err(|_| "No se pudo leer la configuracion LLM")?
        .clone();

    let cloud_session = state
        .cloud_session
        .lock()
        .map_err(|_| "No se pudo leer la sesion cloud")?
        .clone();

    let baserow_ok = state
        .baserow_client
        .lock()
        .map_err(|_| "Error interno")?
        .is_some();

    let last_sync = get_setting(&db, CLOUD_LAST_SYNC_KEY)?;
    let auto_login = get_setting(&db, CLOUD_AUTO_LOGIN_KEY)?.unwrap_or_default();
    let email_verified =
        get_setting(&db, CLOUD_EMAIL_VERIFIED_KEY)?.unwrap_or_default() == "true";

    Ok(AppStatus {
        guardian_pin_set: get_setting(&db, PIN_SETTING_KEY)?.is_some(),
        adult_unlocked,
        profiles: list_profiles_from_db(&db)?,
        llm_config,
        cloud_status: CloudStatus {
            connected: cloud_session.is_some() && baserow_ok,
            user_name: cloud_session.as_ref().map(|s| s.user_name.clone()),
            email: cloud_session.as_ref().map(|s| s.email.clone()),
            last_sync,
            auto_login: auto_login == "true",
            email_verified,
        },
    })
}

#[tauri::command]
/// Configura el PIN de adulto si aun no existe y desbloquea el area adulta.
///
/// # Parámetros
/// - `pin`: PIN numerico de 4 a 6 digitos.
/// - `state`: Contexto de estado compartido `AppState`.
fn setup_guardian_pin(pin: String, state: State<'_, AppState>) -> Result<(), String> {
    validate_pin(&pin)?;
    let db = state.db.lock().map_err(|_| "No se pudo acceder a la base de datos")?;
    if get_setting(&db, PIN_SETTING_KEY)?.is_some() {
        return Err("El PIN adulto ya esta configurado".to_string());
    }
    let pin_hash = hash_pin(&pin)?;
    set_setting(&db, PIN_SETTING_KEY, &pin_hash)?;
    *state.adult_unlocked.lock().map_err(|_| "No se pudo iniciar la sesion adulta")? = true;
    Ok(())
}

#[tauri::command]
/// Verifica el PIN de adulto y desbloquea el area adulta si es correcto.
///
/// # Parámetros
/// - `pin`: PIN numerico a verificar.
/// - `state`: Contexto de estado compartido `AppState`.
///
/// # Retorna
/// `true` si el PIN coincide, `false` en caso contrario.
fn verify_guardian_pin(pin: String, state: State<'_, AppState>) -> Result<bool, String> {
    let db = state.db.lock().map_err(|_| "No se pudo acceder a la base de datos")?;
    let Some(pin_hash) = get_setting(&db, PIN_SETTING_KEY)? else {
        return Ok(false);
    };
    let valid = verify_pin(&pin, &pin_hash)?;
    if valid {
        *state.adult_unlocked.lock().map_err(|_| "No se pudo iniciar la sesion adulta")? = true;
    }
    Ok(valid)
}

#[tauri::command]
/// Cierra forzosamente el área adulta, bloqueando todas las operaciones sensibles.
///
/// # Parámetros
/// - `state`: Contexto de estado compartido `AppState`.
fn lock_adult_area(state: State<'_, AppState>) -> Result<(), String> {
    *state.adult_unlocked.lock().map_err(|_| "No se pudo bloquear la zona adulta")? = false;
    Ok(())
}

#[tauri::command]
/// Establece el locale del usuario para mensajes LLM y UI (ej. `"es-ES"`, `"en-US"`).
///
/// # Parámetros
/// - `locale`: código IETF BCP47 (ej. `"es-ES"`)
/// - `state`: Contexto de estado compartido `AppState`.
fn set_locale(locale: String, state: State<'_, AppState>) -> Result<(), String> {
    let mut loc = state.locale.lock().map_err(|_| "No se pudo establecer locale")?;
    *loc = locale;
    Ok(())
}

#[tauri::command]
/// Elimina todos los datos locales (perfiles, sesiones, settings, PIN).
///
/// - `confirm_phrase`: debe ser exactamente `"RESET"` para confirmar
/// - `state`: Contexto de estado compartido `AppState`.
fn reset_local_data(confirm_phrase: String, state: State<'_, AppState>) -> Result<(), String> {
    if confirm_phrase != "RESET" {
        return Err("Escribe RESET para confirmar el borrado local".to_string());
    }
    let db = state.db.lock().map_err(|_| "No se pudo acceder a la base de datos")?;
    db.execute("DELETE FROM session_questions", []).map_err(|err| format!("No se pudieron borrar preguntas: {err}"))?;
    db.execute("DELETE FROM sessions", []).map_err(|err| format!("No se pudieron borrar sesiones: {err}"))?;
    db.execute("DELETE FROM profiles", []).map_err(|err| format!("No se pudieron borrar los perfiles: {err}"))?;
    db.execute("DELETE FROM app_settings", []).map_err(|err| format!("No se pudo borrar la configuracion: {err}"))?;
    *state.adult_unlocked.lock().map_err(|_| "No se pudo reiniciar la sesion adulta")? = false;
    Ok(())
}

#[tauri::command]
/// Devuelve todos los perfiles de alumno registrados localmente.
///
/// # Parámetros
/// - `state`: Contexto de estado compartido `AppState`.
///
/// # Retorna
/// Lista de `Profile` ordenada por fecha de creacion.
fn list_profiles(state: State<'_, AppState>) -> Result<Vec<Profile>, String> {
    let db = state.db.lock().map_err(|_| "No se pudo acceder a la base de datos")?;
    list_profiles_from_db(&db)
}

#[tauri::command]
/// Crea un nuevo perfil de alumno.
///
/// # Parámetros
/// - `request`: Datos del perfil a crear (`CreateProfileRequest`).
/// - `state`: Contexto de estado compartido `AppState`.
///
/// # Retorna
/// El perfil recien creado con su ID asignado.
fn create_profile(request: CreateProfileRequest, state: State<'_, AppState>) -> Result<Profile, String> {
    require_adult_unlocked(&state)?;
    validate_profile_input(&request.display_name, request.school_year, request.age, request.level_mode, request.manual_level, request.manual_prompt.as_deref())?;
    let db = state.db.lock().map_err(|_| "No se pudo acceder a la base de datos")?;
    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();
    let current_level = resolve_current_level(request.school_year, request.level_mode, request.manual_level);
    let manual_prompt = resolve_manual_prompt(request.level_mode, request.manual_prompt);
    let display_name = request.display_name.trim();
    db.execute(
        "INSERT INTO profiles (id, display_name, school_year, age, level_mode, current_level, manual_prompt, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![id, display_name, request.school_year, request.age, request.level_mode.as_str(), current_level, manual_prompt, now, now],
    ).map_err(|err| format!("No se pudo crear el perfil: {err}"))?;
    get_profile_by_id(&db, &id)
}

#[tauri::command]
/// Actualiza los datos de un perfil existente.
///
/// # Parámetros
/// - `request`: Datos actualizados del perfil (`UpdateProfileRequest`).
/// - `state`: Contexto de estado compartido `AppState`.
///
/// # Retorna
/// El perfil actualizado.
fn update_profile(request: UpdateProfileRequest, state: State<'_, AppState>) -> Result<Profile, String> {
    require_adult_unlocked(&state)?;
    validate_profile_input(&request.display_name, request.school_year, request.age, request.level_mode, request.manual_level, request.manual_prompt.as_deref())?;
    let db = state.db.lock().map_err(|_| "No se pudo acceder a la base de datos")?;
    let current = get_profile_by_id(&db, &request.id)?;
    let requested_level = resolve_current_level(request.school_year, request.level_mode, request.manual_level);
    let current_level = requested_level.max(current.current_level);
    let manual_prompt = resolve_manual_prompt(request.level_mode, request.manual_prompt);
    let now = Utc::now().to_rfc3339();
    let display_name = request.display_name.trim();
    db.execute(
        "UPDATE profiles SET display_name = ?1, school_year = ?2, age = ?3, level_mode = ?4, current_level = ?5, manual_prompt = ?6, updated_at = ?7 WHERE id = ?8",
        params![display_name, request.school_year, request.age, request.level_mode.as_str(), current_level, manual_prompt, now, request.id],
    ).map_err(|err| format!("No se pudo actualizar el perfil: {err}"))?;
    get_profile_by_id(&db, &request.id)
}

#[tauri::command]
/// Elimina un perfil de alumno y todos sus datos asociados (sesiones, preguntas).
///
/// # Parámetros
/// - `id`: ID del perfil a eliminar.
/// - `state`: Contexto de estado compartido `AppState`.
fn delete_profile(id: String, state: State<'_, AppState>) -> Result<(), String> {
    require_adult_unlocked(&state)?;
    let db = state.db.lock().map_err(|_| "No se pudo acceder a la base de datos")?;
    let now = Utc::now().to_rfc3339();
    db.execute(
        "UPDATE session_questions SET deleted_at = ?1, updated_at = ?1 WHERE session_id IN (SELECT id FROM sessions WHERE profile_id = ?2) AND deleted_at IS NULL",
        params![now, id],
    ).map_err(|err| format!("No se pudieron borrar preguntas: {err}"))?;
    db.execute(
        "UPDATE sessions SET deleted_at = ?1, updated_at = ?1 WHERE profile_id = ?2 AND deleted_at IS NULL",
        params![now, id],
    ).map_err(|err| format!("No se pudieron borrar sesiones: {err}"))?;
    db.execute(
        "UPDATE profiles SET deleted_at = ?1, updated_at = ?1 WHERE id = ?2 AND deleted_at IS NULL",
        params![now, id],
    ).map_err(|err| format!("No se pudo eliminar el perfil: {err}"))?;
    Ok(())
}

#[tauri::command]
/// Recupera un perfil eliminado (soft-delete) y todos sus datos asociados (sesiones, preguntas).
/// Requiere la zona adulta desbloqueada.
///
/// # Parámetros
/// - `id`: ID del perfil a recuperar.
/// - `state`: Contexto de estado compartido `AppState`.
fn recover_profile(id: String, state: State<'_, AppState>) -> Result<(), String> {
    require_adult_unlocked(&state)?;
    let db = state.db.lock().map_err(|_| "No se pudo acceder a la base de datos")?;
    let now = Utc::now().to_rfc3339();
    db.execute(
        "UPDATE session_questions SET deleted_at = NULL, updated_at = ?1 WHERE session_id IN (SELECT id FROM sessions WHERE profile_id = ?2) AND deleted_at IS NOT NULL",
        params![now, id],
    ).map_err(|err| format!("No se pudieron recuperar preguntas: {err}"))?;
    db.execute(
        "UPDATE sessions SET deleted_at = NULL, updated_at = ?1 WHERE profile_id = ?2 AND deleted_at IS NOT NULL",
        params![now, id],
    ).map_err(|err| format!("No se pudieron recuperar sesiones: {err}"))?;
    db.execute(
        "UPDATE profiles SET deleted_at = NULL, updated_at = ?1 WHERE id = ?2 AND deleted_at IS NOT NULL",
        params![now, id],
    ).map_err(|err| format!("No se pudo recuperar el perfil: {err}"))?;
    Ok(())
}

#[tauri::command]
/// Elimina una sesión individual (soft-delete).
/// Requiere la zona adulta desbloqueada.
///
/// # Parámetros
/// - `id`: ID de la sesión a eliminar.
/// - `state`: Contexto de estado compartido `AppState`.
fn delete_session(id: String, state: State<'_, AppState>) -> Result<(), String> {
    require_adult_unlocked(&state)?;
    let db = state.db.lock().map_err(|_| "No se pudo acceder a la base de datos")?;
    let now = Utc::now().to_rfc3339();
    db.execute(
        "UPDATE session_questions SET deleted_at = ?1, updated_at = ?1 WHERE session_id = ?2 AND deleted_at IS NULL",
        params![now, id],
    ).map_err(|err| format!("No se pudieron borrar preguntas: {err}"))?;
    db.execute(
        "UPDATE sessions SET deleted_at = ?1, updated_at = ?1 WHERE id = ?2 AND deleted_at IS NULL",
        params![now, id],
    ).map_err(|err| format!("No se pudo eliminar la sesión: {err}"))?;
    Ok(())
}

#[tauri::command]
/// Recupera una sesión individual eliminada (soft-delete).
/// Requiere la zona adulta desbloqueada.
///
/// # Parámetros
/// - `id`: ID de la sesión a recuperar.
/// - `state`: Contexto de estado compartido `AppState`.
fn recover_session(id: String, state: State<'_, AppState>) -> Result<(), String> {
    require_adult_unlocked(&state)?;
    let db = state.db.lock().map_err(|_| "No se pudo acceder a la base de datos")?;
    let now = Utc::now().to_rfc3339();
    db.execute(
        "UPDATE session_questions SET deleted_at = NULL, updated_at = ?1 WHERE session_id = ?2 AND deleted_at IS NOT NULL",
        params![now, id],
    ).map_err(|err| format!("No se pudieron recuperar preguntas: {err}"))?;
    db.execute(
        "UPDATE sessions SET deleted_at = NULL, updated_at = ?1 WHERE id = ?2 AND deleted_at IS NOT NULL",
        params![now, id],
    ).map_err(|err| format!("No se pudo recuperar la sesión: {err}"))?;
    Ok(())
}

#[tauri::command]
/// Elimina permanentemente sesiones y sus preguntas que llevan más de un mes eliminadas (soft-delete).
/// No sincroniza con la nube — es una limpieza local.
fn purge_old_sessions(state: State<'_, AppState>) -> Result<u32, String> {
    let db = state.db.lock().map_err(|_| "No se pudo acceder a la base de datos")?;
    let cutoff = Utc::now()
        .checked_sub_signed(chrono::Duration::days(30))
        .ok_or("Error calculando fecha de corte")?
        .to_rfc3339();
    let deleted_questions = db
        .execute(
            "DELETE FROM session_questions WHERE deleted_at IS NOT NULL AND deleted_at < ?1",
            params![cutoff],
        )
        .map_err(|err| format!("Error purgando preguntas: {err}"))?;
    let deleted_sessions = db
        .execute(
            "DELETE FROM sessions WHERE deleted_at IS NOT NULL AND deleted_at < ?1",
            params![cutoff],
        )
        .map_err(|err| format!("Error purgando sesiones: {err}"))?;
    Ok((deleted_questions + deleted_sessions) as u32)
}

#[tauri::command]
/// Devuelve todos los perfiles eliminados (soft-delete). Requiere la zona adulta desbloqueada.
///
/// # Parámetros
/// - `state`: Contexto de estado compartido `AppState`.
///
/// # Retorna
/// Lista de `Profile` eliminados ordenados por fecha de eliminacion descendente.
fn list_deleted_profiles(state: State<'_, AppState>) -> Result<Vec<Profile>, String> {
    require_adult_unlocked(&state)?;
    let db = state.db.lock().map_err(|_| "No se pudo acceder a la base de datos")?;
    list_deleted_profiles_from_db(&db)
}

// ==================== LLM CONFIG ====================

#[tauri::command]
/// Devuelve la configuracion actual del proveedor LLM.
///
/// # Parámetros
/// - `state`: Contexto de estado compartido `AppState`.
///
/// # Retorna
/// Configuracion LLM actual (`LLMConfig`).
fn get_llm_config(state: State<'_, AppState>) -> Result<LLMConfig, String> {
    let config = state.llm_config.lock().map_err(|_| "No se pudo leer la configuracion LLM")?;
    Ok(config.clone())
}

#[tauri::command]
/// Actualiza la configuracion del proveedor LLM y reconstruye el cliente.
///
/// # Parámetros
/// - `request`: Datos de configuracion (`LLMConfigRequest`).
/// - `state`: Contexto de estado compartido `AppState`.
///
/// # Retorna
/// La configuracion guardada.
fn set_llm_config(request: LLMConfigRequest, state: State<'_, AppState>) -> Result<LLMConfig, String> {
    require_adult_unlocked(&state)?;
    let db = state.db.lock().map_err(|_| "No se pudo acceder a la base de datos")?;
    set_setting(&db, LLM_PROVIDER_KEY, &request.provider)?;
    set_setting(&db, LLM_MODEL_KEY, &request.model)?;
    set_setting(&db, LLM_BASE_URL_KEY, &request.base_url)?;
    set_setting(&db, LLM_API_KEY_KEY, &request.api_key)?;

    let config = LLMConfig {
        provider: request.provider,
        model: request.model,
        base_url: request.base_url,
        api_key: request.api_key,
    };
    *state.llm_config.lock().map_err(|_| "No se pudo guardar la configuracion LLM")? = config.clone();

    let provider = build_provider(&config);
    *state.llm_provider.lock().map_err(|_| "No se pudo asignar el proveedor LLM")? = Some(provider);

    Ok(config)
}

#[tauri::command]
/// Prueba la conexion con el proveedor LLM generando una pregunta de ejemplo.
///
/// # Parámetros
/// - `state`: Contexto de estado compartido `AppState`.
///
/// # Retorna
/// Mensaje de confirmacion con la pregunta generada, o un error.
fn test_llm_connection(state: State<'_, AppState>) -> Result<String, String> {
    let provider_guard = state.llm_provider.lock().map_err(|_| "No se pudo acceder al proveedor LLM")?;
    let provider = provider_guard.as_ref().ok_or("No hay proveedor LLM configurado")?;
    let locale = state.locale.lock().map_err(|_| "No se pudo acceder al locale")?.clone();

    let rt = tokio::runtime::Runtime::new().map_err(|err| format!("No se pudo crear runtime: {err}"))?;
    rt.block_on(provider.generate_question(1, 1, Some("suma basica".to_string()), None, &locale))
        .map(|q| format!("Conexion OK. Pregunta: {}", q.question))
        .map_err(|err| format!("Error: {err}"))
}

// ==================== CHAT ====================

#[tauri::command]
/// Envía un mensaje de chat libre al LLM y devuelve la respuesta.
/// El tono es amigable y adaptado a la edad del niño.
///
/// # Parámetros
/// - `request`: Datos del mensaje (`ChatMessageRequest`).
/// - `state`: Contexto de estado compartido `AppState`.
///
/// # Retorna
/// Respuesta del asistente (`ChatMessageResponse`).
async fn chat_message(request: ChatMessageRequest, state: State<'_, AppState>) -> Result<ChatMessageResponse, String> {
    let (profile, provider, locale) = {
        let db = state.db.lock().map_err(|_| "No se pudo acceder a la base de datos")?;
        let profile = get_profile_by_id(&db, &request.profile_id)?;
        let provider_guard = state.llm_provider.lock().map_err(|_| "No se pudo acceder al proveedor LLM")?;
        let provider = provider_guard.as_ref()
            .ok_or("No hay proveedor LLM configurado")?
            .clone();
        let locale = state.locale.lock().map_err(|_| "No se pudo acceder al locale")?.clone();
        (profile, provider, locale)
    };

    let language = match &locale[..2] {
        "es" => "espanol",
        "ca" => "catalan",
        "eu" => "euskera",
        "gl" => "gallego",
        "en" => "ingles",
        _ => "espanol",
    };

    let age_text = profile.age.map(|a| format!("Edad del nino: {a} anios. ")).unwrap_or_default();
    let year_text = format!("Curso: {}o de primaria. ", profile.school_year);

    let manual_context = profile.manual_prompt.as_deref()
        .map(str::trim)
        .filter(|p| !p.is_empty())
        .map(|p| format!("\nContexto pedagogico adicional del perfil:\n{p}"))
        .unwrap_or_default();

    let system_prompt = format!(
        "Eres un companion de aprendizaje para ninos. Hablas de forma amigable y sencilla. \
         {age_text}{year_text}\
         Puedes hablar de matematicas, curiosidades, juegos, y ayudar con dudas. \
         NO hables de temas inapropiados para ninos. \
         Responde siempre en {language}. Sé breve y motivador.{manual_context}"
    );

    let messages = vec![
        llm::commands::ChatMessage { role: "system".to_string(), content: system_prompt },
        llm::commands::ChatMessage { role: "user".to_string(), content: request.message },
    ];

    let response_text = provider.chat_completion(&messages).await
        .map_err(|e| format!("Error del LLM: {e}"))?;

    Ok(ChatMessageResponse { response: response_text })
}

// ==================== SESSIONS ====================

#[tauri::command]
/// Inicia una nueva sesion de practica para un perfil y genera la primera pregunta.
///
/// # Parámetros
/// - `profile_id`: ID del perfil de alumno.
/// - `state`: Contexto de estado compartido `AppState`.
///
/// # Retorna
/// Informacion de la sesion recien creada y la primera pregunta.
async fn start_session(profile_id: String, state: State<'_, AppState>) -> Result<StartSessionResponse, String> {
    let (profile, session_id) = {
        let db = state.db.lock().map_err(|_| "No se pudo acceder a la base de datos")?;
        let profile = get_profile_by_id(&db, &profile_id)?;
        let session_id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();
        db.execute(
            "INSERT INTO sessions (id, profile_id, status, total_questions, questions_answered, correct_count, current_question_index, started_at)
             VALUES (?1, ?2, 'active', 10, 0, 0, 0, ?3)",
            params![session_id, profile_id, now],
        ).map_err(|err| format!("No se pudo crear la sesion: {err}"))?;
        (profile, session_id)
    };

    let first_question = generate_question_for_session(&state, &session_id, &profile).await?;

    Ok(StartSessionResponse {
        session_id,
        total_questions: 10,
        first_question: Some(first_question),
    })
}

#[tauri::command]
/// Crea una sesion temporal y genera una pregunta para el perfil indicado.
///
/// # Parámetros
/// - `request`: Datos de la solicitud (`GetQuestionRequest`).
/// - `state`: Contexto de estado compartido `AppState`.
///
/// # Retorna
/// La pregunta generada.
async fn generate_question(request: GetQuestionRequest, state: State<'_, AppState>) -> Result<CurrentQuestion, String> {
    let (profile, session_id) = {
        let db = state.db.lock().map_err(|_| "No se pudo acceder a la base de datos")?;
        let profile = get_profile_by_id(&db, &request.profile_id)?;
        let session_id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();
        db.execute(
            "INSERT INTO sessions (id, profile_id, status, total_questions, questions_answered, correct_count, current_question_index, started_at)
             VALUES (?1, ?2, 'active', 10, 0, 0, 0, ?3)",
            params![session_id, request.profile_id, now],
        ).map_err(|err| format!("No se pudo crear sesion temporal: {err}"))?;
        (profile, session_id)
    };

    generate_question_for_session(&state, &session_id, &profile).await
}

#[tauri::command]
/// Evalua la respuesta del alumno, actualiza la sesion y devuelve la siguiente pregunta si corresponde.
///
/// # Parámetros
/// - `request`: Datos de la respuesta (`SubmitAnswerRequest`).
/// - `state`: Contexto de estado compartido `AppState`.
///
/// # Retorna
/// Resultado de la evaluacion y siguiente paso de la sesion.
async fn submit_answer(request: SubmitAnswerRequest, state: State<'_, AppState>) -> Result<SubmitAnswerResponse, String> {
    let (is_correct, feedback, correct_answer, finished, profile) = {
        let db = state.db.lock().map_err(|_| "No se pudo acceder a la base de datos")?;
        let question = get_question_by_id(&db, &request.question_id)?;

        let (is_correct, feedback) = evaluate_answer_local(&question.correct_answer, &request.answer);

        let now = Utc::now().to_rfc3339();
        db.execute(
            "UPDATE session_questions SET student_answer = ?1, is_correct = ?2, explanation = ?3, answered_at = ?4, time_spent_secs = ?5 WHERE id = ?6",
            params![request.answer, is_correct, feedback, now, request.time_spent_secs, request.question_id],
        ).map_err(|err| format!("No se pudo guardar la respuesta: {err}"))?;

        let session = get_session_by_id(&db, &request.session_id)?;
        let new_answered = session.questions_answered + 1;
        let new_correct = session.correct_count + if is_correct { 1 } else { 0 };
        let new_index = session.current_question_index + 1;
        let finished = new_answered >= session.total_questions;

        let new_status = if finished { "completed" } else { "active" };
        db.execute(
            "UPDATE sessions SET questions_answered = ?1, correct_count = ?2, current_question_index = ?3, status = ?4 WHERE id = ?5",
            params![new_answered, new_correct, new_index, new_status, request.session_id],
        ).map_err(|err| format!("No se pudo actualizar la sesion: {err}"))?;

        let profile = if !finished {
            Some(get_profile_by_id(&db, &session.profile_id)?)
        } else {
            None
        };

        (is_correct, feedback, question.correct_answer, finished, profile)
    };

    let next_question = if !finished {
        let profile = profile.unwrap();
        Some(generate_question_for_session(&state, &request.session_id, &profile).await?)
    } else {
        None
    };

    Ok(SubmitAnswerResponse {
        is_correct,
        feedback,
        correct_answer,
        explanation_needed: !is_correct,
        next_question,
        session_finished: finished,
    })
}

#[tauri::command]
/// Genera una explicacion detallada y una reformulacion para una pregunta fallida usando el LLM.
///
/// # Parámetros
/// - `question_id`: ID de la pregunta a explicar.
/// - `state`: Contexto de estado compartido `AppState`.
///
/// # Retorna
/// Explicacion, puntos clave, pasos siguientes y pregunta reformulada (opcional).
fn get_explanation(question_id: String, state: State<'_, AppState>) -> Result<ExplanationResponse, String> {
    let db = state.db.lock().map_err(|_| "No se pudo acceder a la base de datos")?;
    let question = get_question_by_id(&db, &question_id)?;
    let session = get_session_by_id(&db, &question.session_id)?;
    let profile = get_profile_by_id(&db, &session.profile_id)?;
    let manual_prompt = manual_prompt_for_profile(&profile);
    let locale = state.locale.lock()
        .map_err(|_| "No se pudo acceder al locale")?
        .clone();

    let explanation = {
        let provider_guard = state.llm_provider.lock().map_err(|_| "No se pudo acceder al proveedor LLM")?;
        if let Some(provider) = provider_guard.as_ref() {
            let llm_question = LLMQuestion {
                question: question.question_text.clone(),
                correct_answer: question.correct_answer.clone(),
                concept: question.concept.clone(),
                difficulty: question.difficulty.clone(),
            };
            let rt = tokio::runtime::Runtime::new().map_err(|err| format!("No se pudo crear runtime: {err}"))?;
            match rt.block_on(provider.provide_explanation(
                &llm_question,
                question.student_answer.as_deref().unwrap_or(""),
                &question.correct_answer,
                &question.concept,
                manual_prompt.as_deref(),
                &locale,
            )) {
                Ok(exp) => exp,
                Err(_) => LLMExplanation {
                    explanation: generate_default_explanation(&question),
                    key_points: vec![question.concept.clone()],
                    next_steps: vec!["Practica mas ejercicios de este tipo".to_string()],
                },
            }
        } else {
            LLMExplanation {
                explanation: generate_default_explanation(&question),
                key_points: vec![question.concept.clone()],
                next_steps: vec!["Practica mas ejercicios de este tipo".to_string()],
            }
        }
    };

    let reformulated = {
        let provider_guard = state.llm_provider.lock().map_err(|_| "No se pudo acceder al proveedor LLM")?;
        if let Some(provider) = provider_guard.as_ref() {
            let llm_question = LLMQuestion {
                question: question.question_text.clone(),
                correct_answer: question.correct_answer.clone(),
                concept: question.concept.clone(),
                difficulty: question.difficulty.clone(),
            };
            let rt = tokio::runtime::Runtime::new().map_err(|err| format!("No se pudo crear runtime: {err}"))?;
            rt.block_on(provider.reformulate_concept(&question.concept, &llm_question, manual_prompt.as_deref(), &locale)).ok()
        } else {
            None
        }
    };

    Ok(ExplanationResponse {
        explanation: explanation.explanation,
        key_points: explanation.key_points,
        next_steps: explanation.next_steps,
        reformulated_question: reformulated,
    })
}

#[tauri::command]
/// Finaliza una sesion activa y genera un resumen con estadisticas y conceptos trabajados.
///
/// # Parámetros
/// - `session_id`: ID de la sesion a finalizar.
/// - `state`: Contexto de estado compartido `AppState`.
///
/// # Retorna
/// Resumen completo de la sesion (`SessionSummary`).
fn end_session(session_id: String, state: State<'_, AppState>) -> Result<SessionSummary, String> {
    let db = state.db.lock().map_err(|_| "No se pudo acceder a la base de datos")?;
    let mut session = get_session_by_id(&db, &session_id)?;
    let now = Utc::now().to_rfc3339();

    if session.status == "active" {
        db.execute(
            "UPDATE sessions SET status = 'completed', ended_at = ?1 WHERE id = ?2",
            params![now, session_id],
        ).map_err(|err| format!("No se pudo finalizar la sesion: {err}"))?;
        session.status = "completed".to_string();
        session.ended_at = Some(now);
    }

    let questions = list_questions_for_session(&db, &session_id)?;
    let accuracy_pct = if session.total_questions > 0 {
        (session.correct_count as f64 / session.total_questions as f64) * 100.0
    } else {
        0.0
    };

    let total_time: u32 = questions.iter().filter_map(|q| q.time_spent_secs).sum();
    let avg_time = if !questions.is_empty() {
        total_time as f64 / questions.len() as f64
    } else {
        0.0
    };

    let mut concepts_worked: Vec<String> = questions.iter().map(|q| q.concept.clone()).collect();
    concepts_worked.sort();
    concepts_worked.dedup();

    let mut concepts_mastered = Vec::new();
    let mut concepts_to_practice = Vec::new();
    for concept in &concepts_worked {
        let concept_questions: Vec<_> = questions.iter().filter(|q| q.concept == *concept).collect();
        let concept_correct = concept_questions.iter().filter(|q| q.is_correct == Some(true)).count();
        let concept_total = concept_questions.len();
        if concept_total > 0 && concept_correct as f64 / concept_total as f64 >= 0.7 {
            concepts_mastered.push(concept.clone());
        } else {
            concepts_to_practice.push(concept.clone());
        }
    }

    Ok(SessionSummary {
        session,
        questions,
        concepts_worked,
        concepts_mastered,
        concepts_to_practice,
        accuracy_pct,
        avg_time_per_question: avg_time,
        total_time_secs: total_time,
    })
}

#[tauri::command]
/// Calcula estadisticas globales del dashboard para un perfil (sesiones, aciertos, conceptos).
///
/// # Parámetros
/// - `profile_id`: ID del perfil de alumno.
/// - `state`: Contexto de estado compartido `AppState`.
///
/// # Retorna
/// Estadisticas agregadas del dashboard.
fn get_dashboard_stats(profile_id: String, state: State<'_, AppState>) -> Result<DashboardStats, String> {
    let db = state.db.lock().map_err(|_| "No se pudo acceder a la base de datos")?;
    let mut stmt = db.prepare(
         "SELECT total_questions, questions_answered, correct_count, started_at, ended_at
          FROM sessions WHERE profile_id = ?1 AND status = 'completed' AND deleted_at IS NULL ORDER BY started_at",
    ).map_err(|err| format!("No se pudieron preparar las estadisticas: {err}"))?;

    let rows = stmt.query_map(params![profile_id], |row| {
        Ok((
            row.get::<_, u8>(0)?,
            row.get::<_, u8>(1)?,
            row.get::<_, u8>(2)?,
            row.get::<_, String>(3)?,
            row.get::<_, Option<String>>(4)?,
        ))
    }).map_err(|err| format!("No se pudieron leer las estadisticas: {err}"))?;

    let mut total_sessions: u32 = 0;
    let mut total_questions: u32 = 0;
    let mut total_correct: u32 = 0;
    let total_time: u32 = 0;

    for row in rows {
        let (tq, _qa, cc, _started, _ended) = row.map_err(|err| format!("Sesion invalida: {err}"))?;
        total_sessions += 1;
        total_questions += tq as u32;
        total_correct += cc as u32;
    }

    let overall_accuracy = if total_questions > 0 {
        (total_correct as f64 / total_questions as f64) * 100.0
    } else {
        0.0
    };

    let avg_time = if total_questions > 0 {
        total_time as f64 / total_questions as f64
    } else {
        0.0
    };

    let concept_stats = get_concept_stats_for_profile(&db, &profile_id)
        .map_err(|err| format!("Error al obtener estadisticas: {err}"))?;
    let mut concepts_mastered = Vec::new();
    let mut concepts_in_progress = Vec::new();
    let mut concepts_needing_practice = Vec::new();

    for cs in &concept_stats {
        if cs.accuracy_pct >= 80.0 {
            concepts_mastered.push(cs.concept.clone());
        } else if cs.accuracy_pct >= 50.0 {
            concepts_in_progress.push(cs.concept.clone());
        } else {
            concepts_needing_practice.push(cs.concept.clone());
        }
    }

    Ok(DashboardStats {
        total_sessions,
        total_questions_answered: total_questions,
        total_correct,
        overall_accuracy_pct: overall_accuracy,
        total_time_secs: total_time,
        avg_time_per_question: avg_time,
        concepts_mastered,
        concepts_in_progress,
        concepts_needing_practice,
    })
}

#[tauri::command]
/// Devuelve estadisticas detalladas por concepto para un perfil.
///
/// # Parámetros
/// - `profile_id`: ID del perfil de alumno.
/// - `state`: Contexto de estado compartido `AppState`.
///
/// # Retorna
/// Lista de `ConceptStat` con intentos, aciertos y ultima practica.
fn get_concept_stats(profile_id: String, state: State<'_, AppState>) -> Result<Vec<ConceptStat>, String> {
    let db = state.db.lock().map_err(|_| "No se pudo acceder a la base de datos")?;
    get_concept_stats_for_profile(&db, &profile_id).map_err(|err| format!("Error al obtener estadisticas: {err}"))
}

#[tauri::command]
/// Devuelve la evolucion historica de sesiones completadas para un perfil.
///
/// # Parámetros
/// - `profile_id`: ID del perfil de alumno.
/// - `state`: Contexto de estado compartido `AppState`.
///
/// # Retorna
/// Lista de puntos de evolucion ordenados cronologicamente.
fn get_evolution(profile_id: String, state: State<'_, AppState>) -> Result<Vec<EvolutionPoint>, String> {
    let db = state.db.lock().map_err(|_| "No se pudo acceder a la base de datos")?;
    let mut stmt = db.prepare(
         "SELECT id, started_at, total_questions, correct_count
          FROM sessions WHERE profile_id = ?1 AND status = 'completed' AND deleted_at IS NULL
          ORDER BY started_at ASC",
    ).map_err(|err| format!("No se pudo preparar la evolucion: {err}"))?;

    let rows = stmt.query_map(params![profile_id], |row| {
        let id: String = row.get(0)?;
        let started: String = row.get(1)?;
        let total: u8 = row.get(2)?;
        let correct: u8 = row.get(3)?;
        let accuracy = if total > 0 { (correct as f64 / total as f64) * 100.0 } else { 0.0 };
        Ok(EvolutionPoint {
            session_id: id,
            started_at: started,
            accuracy_pct: accuracy,
            questions_answered: total,
            correct_count: correct,
        })
    }).map_err(|err| format!("No se pudo leer la evolucion: {err}"))?;

    let mut points = Vec::new();
    for row in rows {
        points.push(row.map_err(|err| format!("Punto invalido: {err}"))?);
    }
    Ok(points)
}

#[tauri::command]
/// Exporta todas las sesiones completadas de un perfil en formato plano para CSV.
///
/// # Parámetros
/// - `profile_id`: ID del perfil de alumno.
/// - `state`: Contexto de estado compartido `AppState`.
///
/// # Retorna
/// Lista de filas exportables (`ExportSessionRow`).
fn export_sessions(profile_id: String, state: State<'_, AppState>) -> Result<Vec<ExportSessionRow>, String> {
    let db = state.db.lock().map_err(|_| "No se pudo acceder a la base de datos")?;
    let mut stmt = db.prepare(
         "SELECT sq.session_id, s.started_at, s.ended_at, sq.question_number, sq.question_text,
                 sq.concept, sq.difficulty, sq.student_answer, sq.correct_answer, sq.is_correct, sq.time_spent_secs
          FROM session_questions sq
          JOIN sessions s ON sq.session_id = s.id
          WHERE s.profile_id = ?1 AND s.status = 'completed'
          AND s.deleted_at IS NULL AND sq.deleted_at IS NULL
          ORDER BY s.started_at DESC, sq.question_number ASC",
    ).map_err(|err| format!("No se pudo preparar la exportacion: {err}"))?;

    let rows = stmt.query_map(params![profile_id], |row| {
        Ok(ExportSessionRow {
            session_id: row.get(0)?,
            started_at: row.get(1)?,
            ended_at: row.get(2)?,
            question_number: row.get(3)?,
            question_text: row.get(4)?,
            concept: row.get(5)?,
            difficulty: row.get(6)?,
            student_answer: row.get(7)?,
            correct_answer: row.get(8)?,
            is_correct: row.get(9)?,
            time_spent_secs: row.get(10)?,
        })
    }).map_err(|err| format!("No se pudieron exportar los datos: {err}"))?;

    let mut result = Vec::new();
    for row in rows {
        result.push(row.map_err(|err| format!("Fila invalida: {err}"))?);
    }
    Ok(result)
}

#[tauri::command]
/// Obtiene el resumen completo de una sesion (preguntas, respuestas, estadisticas)
/// sin modificar su estado.
///
/// # Parametros
/// - `session_id`: ID de la sesion.
/// - `state`: Contexto de estado compartido `AppState`.
///
/// # Retorna
/// Resumen completo de la sesion (`SessionSummary`).
fn get_session_summary(session_id: String, state: State<'_, AppState>) -> Result<SessionSummary, String> {
    let db = state.db.lock().map_err(|_| "No se pudo acceder a la base de datos")?;
    let session = get_session_by_id(&db, &session_id)?;
    let questions = list_questions_for_session(&db, &session_id)?;
    let accuracy_pct = if session.total_questions > 0 {
        (session.correct_count as f64 / session.total_questions as f64) * 100.0
    } else {
        0.0
    };

    let total_time: u32 = questions.iter().filter_map(|q| q.time_spent_secs).sum();
    let avg_time = if !questions.is_empty() {
        total_time as f64 / questions.len() as f64
    } else {
        0.0
    };

    let mut concepts_worked: Vec<String> = questions.iter().map(|q| q.concept.clone()).collect();
    concepts_worked.sort();
    concepts_worked.dedup();

    let mut concepts_mastered = Vec::new();
    let mut concepts_to_practice = Vec::new();
    for concept in &concepts_worked {
        let concept_questions: Vec<_> = questions.iter().filter(|q| q.concept == *concept).collect();
        let concept_correct = concept_questions.iter().filter(|q| q.is_correct == Some(true)).count();
        let concept_total = concept_questions.len();
        if concept_total > 0 && concept_correct as f64 / concept_total as f64 >= 0.7 {
            concepts_mastered.push(concept.clone());
        } else {
            concepts_to_practice.push(concept.clone());
        }
    }

    Ok(SessionSummary {
        session,
        questions,
        concepts_worked,
        concepts_mastered,
        concepts_to_practice,
        accuracy_pct,
        avg_time_per_question: avg_time,
        total_time_secs: total_time,
    })
}

#[tauri::command]
/// Lista todas las sesiones de un perfil ordenadas por fecha descendente.
///
/// # Parámetros
/// - `profile_id`: ID del perfil de alumno.
/// - `state`: Contexto de estado compartido `AppState`.
///
/// # Retorna
/// Lista de sesiones del perfil.
fn list_sessions(profile_id: String, state: State<'_, AppState>) -> Result<Vec<Session>, String> {
    let db = state.db.lock().map_err(|_| "No se pudo acceder a la base de datos")?;
    let mut stmt = db.prepare(
         "SELECT id, profile_id, status, total_questions, questions_answered, correct_count, current_question_index, started_at, ended_at
          FROM sessions WHERE profile_id = ?1 AND deleted_at IS NULL ORDER BY started_at DESC",
    ).map_err(|err| format!("No se pudieron preparar las sesiones: {err}"))?;

    let rows = stmt.query_map(params![profile_id], |row| {
        Ok(Session {
            id: row.get(0)?,
            profile_id: row.get(1)?,
            status: row.get(2)?,
            total_questions: row.get(3)?,
            questions_answered: row.get(4)?,
            correct_count: row.get(5)?,
            current_question_index: row.get(6)?,
            started_at: row.get(7)?,
            ended_at: row.get(8)?,
        })
    }).map_err(|err| format!("No se pudieron leer las sesiones: {err}"))?;

    let mut sessions = Vec::new();
    for row in rows {
        sessions.push(row.map_err(|err| format!("Sesion invalida: {err}"))?);
    }
    Ok(sessions)
}

#[tauri::command]
/// Lista las sesiones eliminadas (soft-delete) de un perfil.
/// Requiere la zona adulta desbloqueada.
///
/// # Parámetros
/// - `profile_id`: ID del perfil.
/// - `state`: Contexto de estado compartido `AppState`.
fn list_deleted_sessions(profile_id: String, state: State<'_, AppState>) -> Result<Vec<Session>, String> {
    let db = state.db.lock().map_err(|_| "No se pudo acceder a la base de datos")?;
    let mut stmt = db.prepare(
        "SELECT id, profile_id, status, total_questions, questions_answered, correct_count, current_question_index, started_at, ended_at
         FROM sessions WHERE profile_id = ?1 AND deleted_at IS NOT NULL ORDER BY deleted_at DESC",
    ).map_err(|err| format!("No se pudieron preparar las sesiones eliminadas: {err}"))?;

    let rows = stmt.query_map(params![profile_id], |row| {
        Ok(Session {
            id: row.get(0)?,
            profile_id: row.get(1)?,
            status: row.get(2)?,
            total_questions: row.get(3)?,
            questions_answered: row.get(4)?,
            correct_count: row.get(5)?,
            current_question_index: row.get(6)?,
            started_at: row.get(7)?,
            ended_at: row.get(8)?,
        })
    }).map_err(|err| format!("No se pudieron leer las sesiones eliminadas: {err}"))?;

    let mut sessions = Vec::new();
    for row in rows {
        sessions.push(row.map_err(|err| format!("Sesion eliminada invalida: {err}"))?);
    }
    Ok(sessions)
}

// ==================== PROFESSIONAL LAYER ====================

#[tauri::command]
/// Crea un nuevo usuario profesional (tutor/profesor) con PIN hasheado.
///
/// # Parámetros
/// - `request`: Datos del usuario (`CreateUserRequest`).
/// - `state`: Contexto de estado compartido `AppState`.
///
/// # Retorna
/// El usuario creado.
fn create_user(request: CreateUserRequest, state: State<'_, AppState>) -> Result<User, String> {
    let db = state.db.lock().map_err(|_| "No se pudo acceder a la base de datos")?;
    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();

    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let pin_hash = argon2.hash_password(request.pin.as_bytes(), &salt)
        .map_err(|err| format!("No se pudo hashear el PIN: {err}"))?
        .to_string();

    db.execute(
        "INSERT INTO users (id, display_name, pin_hash, role, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![id, request.display_name, pin_hash, request.role, now],
    ).map_err(|err| format!("No se pudo crear el usuario: {err}"))?;

    Ok(User { id, display_name: request.display_name, role: request.role, created_at: now })
}

#[tauri::command]
/// Autentica un usuario profesional (tutor/profesor) comparando el PIN contra el hash almacenado.
///
/// # Parámetros
/// - `request`: Credenciales de inicio de sesion (`LoginRequest`).
/// - `state`: Contexto de estado compartido `AppState`.
///
/// # Retorna
/// El usuario si la autenticacion es correcta.
fn login_user(request: LoginRequest, state: State<'_, AppState>) -> Result<User, String> {
    let db = state.db.lock().map_err(|_| "No se pudo acceder a la base de datos")?;
    let mut stmt = db.prepare(
        "SELECT id, display_name, pin_hash, role, created_at FROM users ORDER BY created_at ASC LIMIT 1",
    ).map_err(|err| format!("No se pudo preparar la consulta: {err}"))?;

    let user = stmt.query_row([], |row| {
        let pin_hash: String = row.get(2)?;
        Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?, pin_hash, row.get::<_, String>(3)?, row.get::<_, String>(4)?))
    }).map_err(|_| "No hay usuarios registrados".to_string())?;

    let (id, display_name, pin_hash, role, created_at) = user;

    let parsed_hash = PasswordHash::new(&pin_hash)
        .map_err(|err| format!("Error al verificar PIN: {err}"))?;
    let argon2 = Argon2::default();
    argon2.verify_password(request.pin.as_bytes(), &parsed_hash)
        .map_err(|_| "PIN incorrecto".to_string())?;

    Ok(User { id, display_name, role, created_at })
}

#[tauri::command]
/// Lista todos los usuarios profesionales registrados, ordenados por fecha de creacion.
///
/// # Parámetros
/// - `state`: Contexto de estado compartido `AppState`.
///
/// # Retorna
/// Vector de usuarios.
fn list_users(state: State<'_, AppState>) -> Result<Vec<User>, String> {
    let db = state.db.lock().map_err(|_| "No se pudo acceder a la base de datos")?;
    let mut stmt = db.prepare(
        "SELECT id, display_name, role, created_at FROM users ORDER BY created_at ASC",
    ).map_err(|err| format!("No se pudieron preparar los usuarios: {err}"))?;

    let rows = stmt.query_map([], |row| {
        Ok(User {
            id: row.get(0)?,
            display_name: row.get(1)?,
            role: row.get(2)?,
            created_at: row.get(3)?,
        })
    }).map_err(|err| format!("No se pudieron leer los usuarios: {err}"))?;

    let mut users = Vec::new();
    for row in rows {
        users.push(row.map_err(|err| format!("Usuario invalido: {err}"))?);
    }
    Ok(users)
}

#[tauri::command]
/// Crea un nuevo grupo de estudiantes para el usuario profesional.
///
/// # Parámetros
/// - `request`: Datos del grupo (`CreateStudentGroupRequest`).
/// - `user_id`: Identificador del usuario propietario.
/// - `state`: Contexto de estado compartido `AppState`.
///
/// # Retorna
/// El grupo de estudiantes creado.
fn create_student_group(request: CreateStudentGroupRequest, user_id: String, state: State<'_, AppState>) -> Result<StudentGroup, String> {
    let db = state.db.lock().map_err(|_| "No se pudo acceder a la base de datos")?;
    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();

    db.execute(
        "INSERT INTO student_groups (id, name, owner_user_id, created_at) VALUES (?1, ?2, ?3, ?4)",
        params![id, request.name, user_id, now],
    ).map_err(|err| format!("No se pudo crear el grupo: {err}"))?;

    Ok(StudentGroup { id, name: request.name, owner_user_id: user_id, created_at: now })
}

#[tauri::command]
/// Lista los grupos de estudiantes propiedad del usuario indicado.
///
/// # Parámetros
/// - `user_id`: Identificador del usuario propietario.
/// - `state`: Contexto de estado compartido `AppState`.
///
/// # Retorna
/// Vector de grupos de estudiantes.
fn list_student_groups(user_id: String, state: State<'_, AppState>) -> Result<Vec<StudentGroup>, String> {
    let db = state.db.lock().map_err(|_| "No se pudo acceder a la base de datos")?;
    let mut stmt = db.prepare(
        "SELECT id, name, owner_user_id, created_at FROM student_groups WHERE owner_user_id = ?1 ORDER BY created_at ASC",
    ).map_err(|err| format!("No se pudieron preparar los grupos: {err}"))?;

    let rows = stmt.query_map(params![user_id], |row| {
        Ok(StudentGroup {
            id: row.get(0)?,
            name: row.get(1)?,
            owner_user_id: row.get(2)?,
            created_at: row.get(3)?,
        })
    }).map_err(|err| format!("No se pudieron leer los grupos: {err}"))?;

    let mut groups = Vec::new();
    for row in rows {
        groups.push(row.map_err(|err| format!("Grupo invalido: {err}"))?);
    }
    Ok(groups)
}

#[tauri::command]
/// Agrega un perfil (estudiante) a un grupo existente.
///
/// # Parámetros
/// - `group_id`: Identificador del grupo.
/// - `student_id`: Identificador del perfil del estudiante.
/// - `state`: Contexto de estado compartido `AppState`.
fn add_student_to_group(group_id: String, student_id: String, state: State<'_, AppState>) -> Result<(), String> {
    let db = state.db.lock().map_err(|_| "No se pudo acceder a la base de datos")?;
    db.execute(
        "INSERT OR IGNORE INTO student_group_members (group_id, student_id) VALUES (?1, ?2)",
        params![group_id, student_id],
    ).map_err(|err| format!("No se pudo agregar el estudiante al grupo: {err}"))?;
    Ok(())
}

#[tauri::command]
/// Elimina un estudiante de un grupo.
///
/// # Parámetros
/// - `group_id`: Identificador del grupo.
/// - `student_id`: Identificador del perfil del estudiante.
/// - `state`: Contexto de estado compartido `AppState`.
fn remove_student_from_group(group_id: String, student_id: String, state: State<'_, AppState>) -> Result<(), String> {
    let db = state.db.lock().map_err(|_| "No se pudo acceder a la base de datos")?;
    db.execute(
        "DELETE FROM student_group_members WHERE group_id = ?1 AND student_id = ?2",
        params![group_id, student_id],
    ).map_err(|err| format!("No se pudo remover el estudiante del grupo: {err}"))?;
    Ok(())
}

#[tauri::command]
/// Lista los perfiles de estudiantes pertenecientes a un grupo.
///
/// # Parámetros
/// - `group_id`: Identificador del grupo.
/// - `state`: Contexto de estado compartido `AppState`.
///
/// # Retorna
/// Vector de perfiles.
fn list_group_students(group_id: String, state: State<'_, AppState>) -> Result<Vec<Profile>, String> {
    let db = state.db.lock().map_err(|_| "No se pudo acceder a la base de datos")?;
    let mut stmt = db.prepare(
         "SELECT p.id, p.display_name, p.school_year, p.age, p.level_mode, p.current_level, p.manual_prompt, p.created_at, p.updated_at
          FROM profiles p
          JOIN student_group_members sgm ON p.id = sgm.student_id
          WHERE sgm.group_id = ?1 AND p.deleted_at IS NULL
          ORDER BY p.display_name ASC",
    ).map_err(|err| format!("No se pudieron preparar los estudiantes: {err}"))?;

    let rows = stmt.query_map(params![group_id], |row| {
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
    }).map_err(|err| format!("No se pudieron leer los estudiantes: {err}"))?;

    let mut students = Vec::new();
    for row in rows {
        students.push(row.map_err(|err| format!("Estudiante invalido: {err}"))?);
    }
    Ok(students)
}

#[tauri::command]
/// Asigna un estudiante a un tutor.
///
/// # Parámetros
/// - `tutor_user_id`: Identificador del usuario tutor.
/// - `request`: Datos de la asignacion (`AssignStudentRequest`).
/// - `state`: Contexto de estado compartido `AppState`.
///
/// # Retorna
/// La relacion tutor-estudiante creada.
fn assign_student_to_tutor(tutor_user_id: String, request: AssignStudentRequest, state: State<'_, AppState>) -> Result<TutorStudent, String> {
    let db = state.db.lock().map_err(|_| "No se pudo acceder a la base de datos")?;
    let now = Utc::now().to_rfc3339();

    db.execute(
        "INSERT OR IGNORE INTO tutor_student (tutor_user_id, student_id, assigned_at, status) VALUES (?1, ?2, ?3, 'active')",
        params![tutor_user_id, request.student_id, now],
    ).map_err(|err| format!("No se pudo asignar el estudiante: {err}"))?;

    Ok(TutorStudent { tutor_user_id, student_id: request.student_id, assigned_at: now, status: "active".to_string() })
}

#[tauri::command]
/// Desactiva la relacion tutor-estudiante (borrado logico).
///
/// # Parámetros
/// - `tutor_user_id`: Identificador del usuario tutor.
/// - `student_id`: Identificador del perfil del estudiante.
/// - `state`: Contexto de estado compartido `AppState`.
fn remove_student_from_tutor(tutor_user_id: String, student_id: String, state: State<'_, AppState>) -> Result<(), String> {
    let db = state.db.lock().map_err(|_| "No se pudo acceder a la base de datos")?;
    db.execute(
        "UPDATE tutor_student SET status = 'inactive' WHERE tutor_user_id = ?1 AND student_id = ?2",
        params![tutor_user_id, student_id],
    ).map_err(|err| format!("No se pudo remover la asignacion: {err}"))?;
    Ok(())
}

#[tauri::command]
/// Lista los estudiantes activos asignados a un tutor.
///
/// # Parámetros
/// - `tutor_user_id`: Identificador del usuario tutor.
/// - `state`: Contexto de estado compartido `AppState`.
///
/// # Retorna
/// Vector de perfiles de estudiantes.
fn list_tutor_students(tutor_user_id: String, state: State<'_, AppState>) -> Result<Vec<Profile>, String> {
    let db = state.db.lock().map_err(|_| "No se pudo acceder a la base de datos")?;
    let mut stmt = db.prepare(
         "SELECT p.id, p.display_name, p.school_year, p.age, p.level_mode, p.current_level, p.manual_prompt, p.created_at, p.updated_at
          FROM profiles p
          JOIN tutor_student ts ON p.id = ts.student_id
          WHERE ts.tutor_user_id = ?1 AND ts.status = 'active' AND p.deleted_at IS NULL
          ORDER BY p.display_name ASC",
    ).map_err(|err| format!("No se pudieron preparar los estudiantes: {err}"))?;

    let rows = stmt.query_map(params![tutor_user_id], |row| {
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
    }).map_err(|err| format!("No se pudieron leer los estudiantes: {err}"))?;

    let mut students = Vec::new();
    for row in rows {
        students.push(row.map_err(|err| format!("Estudiante invalido: {err}"))?);
    }
    Ok(students)
}

#[tauri::command]
/// Crea una nueva tarea asignada por un tutor a un estudiante.
///
/// # Parámetros
/// - `request`: Datos de la tarea (`CreateAssignmentRequest`).
/// - `tutor_user_id`: Identificador del usuario tutor.
/// - `state`: Contexto de estado compartido `AppState`.
///
/// # Retorna
/// La tarea creada.
fn create_assignment(request: CreateAssignmentRequest, tutor_user_id: String, state: State<'_, AppState>) -> Result<Assignment, String> {
    let db = state.db.lock().map_err(|_| "No se pudo acceder a la base de datos")?;
    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();

    db.execute(
        "INSERT INTO assignments (id, tutor_user_id, student_id, concept, difficulty, due_date, status, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, 'pending', ?7)",
        params![id, tutor_user_id, request.student_id, request.concept, request.difficulty, request.due_date, now],
    ).map_err(|err| format!("No se pudo crear la tarea: {err}"))?;

    Ok(Assignment {
        id,
        tutor_user_id,
        student_id: request.student_id,
        concept: request.concept,
        difficulty: request.difficulty,
        due_date: request.due_date,
        status: "pending".to_string(),
        created_at: now,
    })
}

#[tauri::command]
/// Lista las tareas activas asignadas por un tutor, ordenadas por creacion descendente.
///
/// # Parámetros
/// - `tutor_user_id`: Identificador del usuario tutor.
/// - `state`: Contexto de estado compartido `AppState`.
///
/// # Retorna
/// Vector de tareas.
fn list_assignments(tutor_user_id: String, state: State<'_, AppState>) -> Result<Vec<Assignment>, String> {
    let db = state.db.lock().map_err(|_| "No se pudo acceder a la base de datos")?;
    let mut stmt = db.prepare(
        "SELECT id, tutor_user_id, student_id, concept, difficulty, due_date, status, created_at
         FROM assignments WHERE tutor_user_id = ?1 AND status != 'cancelled'
         ORDER BY created_at DESC",
    ).map_err(|err| format!("No se pudieron preparar las tareas: {err}"))?;

    let rows = stmt.query_map(params![tutor_user_id], |row| {
        Ok(Assignment {
            id: row.get(0)?,
            tutor_user_id: row.get(1)?,
            student_id: row.get(2)?,
            concept: row.get(3)?,
            difficulty: row.get(4)?,
            due_date: row.get(5)?,
            status: row.get(6)?,
            created_at: row.get(7)?,
        })
    }).map_err(|err| format!("No se pudieron leer las tareas: {err}"))?;

    let mut assignments = Vec::new();
    for row in rows {
        assignments.push(row.map_err(|err| format!("Tarea invalida: {err}"))?);
    }
    Ok(assignments)
}

#[tauri::command]
/// Genera un reporte de progreso para un estudiante en un periodo determinado.
///
/// # Parámetros
/// - `request`: Datos del reporte (`GenerateReportRequest`).
/// - `tutor_user_id`: Identificador del usuario tutor.
/// - `state`: Contexto de estado compartido `AppState`.
///
/// # Retorna
/// El reporte generado.
fn generate_report(request: GenerateReportRequest, tutor_user_id: String, state: State<'_, AppState>) -> Result<Report, String> {
    let db = state.db.lock().map_err(|_| "No se pudo acceder a la base de datos")?;
    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();

    let concept_stats = get_concept_stats_for_profile(&db, &request.student_id)
        .map_err(|err| format!("No se pudieron obtener las estadisticas: {err}"))?;

    let report_data = serde_json::json!({
        "student_id": request.student_id,
        "period": request.period,
        "generated_at": now,
        "concepts": concept_stats.iter().map(|cs| serde_json::json!({
            "concept": cs.concept,
            "accuracy_pct": cs.accuracy_pct,
            "total_attempts": cs.total_attempts,
            "correct_attempts": cs.correct_attempts,
        })).collect::<Vec<_>>(),
    }).to_string();

    db.execute(
        "INSERT INTO reports (id, tutor_user_id, student_id, period, report_data, generated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![id, tutor_user_id, request.student_id, request.period, report_data, now],
    ).map_err(|err| format!("No se pudo guardar el reporte: {err}"))?;

    Ok(Report { id, tutor_user_id, student_id: request.student_id, period: request.period, report_data, generated_at: now })
}

#[tauri::command]
/// Lista los reportes generados por un tutor, ordenados por fecha descendente.
///
/// # Parámetros
/// - `tutor_user_id`: Identificador del usuario tutor.
/// - `state`: Contexto de estado compartido `AppState`.
///
/// # Retorna
/// Vector de reportes.
fn list_reports(tutor_user_id: String, state: State<'_, AppState>) -> Result<Vec<Report>, String> {
    let db = state.db.lock().map_err(|_| "No se pudo acceder a la base de datos")?;
    let mut stmt = db.prepare(
        "SELECT id, tutor_user_id, student_id, period, report_data, generated_at
         FROM reports WHERE tutor_user_id = ?1
         ORDER BY generated_at DESC",
    ).map_err(|err| format!("No se pudieron preparar los reportes: {err}"))?;

    let rows = stmt.query_map(params![tutor_user_id], |row| {
        Ok(Report {
            id: row.get(0)?,
            tutor_user_id: row.get(1)?,
            student_id: row.get(2)?,
            period: row.get(3)?,
            report_data: row.get(4)?,
            generated_at: row.get(5)?,
        })
    }).map_err(|err| format!("No se pudieron leer los reportes: {err}"))?;

    let mut reports = Vec::new();
    for row in rows {
        reports.push(row.map_err(|err| format!("Reporte invalido: {err}"))?);
    }
    Ok(reports)
}

#[tauri::command]
/// Obtiene el panel de control del tutor con resumen de estudiantes, tareas y reportes.
///
/// # Parámetros
/// - `tutor_user_id`: Identificador del usuario tutor.
/// - `state`: Contexto de estado compartido `AppState`.
///
/// # Retorna
/// Datos del panel de tutor (`TutorDashboard`).
fn get_tutor_dashboard(tutor_user_id: String, state: State<'_, AppState>) -> Result<TutorDashboard, String> {
    let db = state.db.lock().map_err(|_| "No se pudo acceder a la base de datos")?;

    let mut stmt_students = db.prepare(
         "SELECT p.id, p.display_name, p.school_year, p.current_level
          FROM profiles p
          JOIN tutor_student ts ON p.id = ts.student_id
          WHERE ts.tutor_user_id = ?1 AND ts.status = 'active' AND p.deleted_at IS NULL",
    ).map_err(|err| format!("No se pudieron preparar los estudiantes: {err}"))?;

    let student_rows = stmt_students.query_map(params![tutor_user_id], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?, row.get::<_, u8>(2)?, row.get::<_, u8>(3)?))
    }).map_err(|err| format!("No se pudieron leer los estudiantes: {err}"))?;

    let mut students = Vec::new();
    let mut total_students: u32 = 0;

    for row in student_rows {
        let (id, name, year, level) = row.map_err(|err| format!("Estudiante invalido: {err}"))?;
        total_students += 1;

        let last_session: Option<String> = db.query_row(
            "SELECT started_at FROM sessions WHERE profile_id = ?1 AND status = 'completed' AND deleted_at IS NULL ORDER BY started_at DESC LIMIT 1",
            params![id],
            |row| row.get(0),
        ).unwrap_or(None);

        let accuracy: f64 = db.query_row(
            "SELECT COALESCE(SUM(CASE WHEN is_correct = 1 THEN 1 ELSE 0 END) * 100.0 / NULLIF(COUNT(*), 0), 0)
              FROM session_questions sq
              JOIN sessions s ON sq.session_id = s.id
              WHERE s.profile_id = ?1 AND s.status = 'completed' AND sq.is_correct IS NOT NULL
              AND s.deleted_at IS NULL AND sq.deleted_at IS NULL",
            params![id],
            |row| row.get(0),
        ).unwrap_or(0.0);

        students.push(TutorStudentInfo {
            student_id: id,
            display_name: name,
            school_year: year,
            current_level: level,
            last_session,
            accuracy_pct: accuracy,
        });
    }

    let active_assignments: u32 = db.query_row(
        "SELECT COUNT(*) FROM assignments WHERE tutor_user_id = ?1 AND status IN ('pending', 'in_progress')",
        params![tutor_user_id],
        |row| row.get(0),
    ).unwrap_or(0);

    let reports_generated: u32 = db.query_row(
        "SELECT COUNT(*) FROM reports WHERE tutor_user_id = ?1",
        params![tutor_user_id],
        |row| row.get(0),
    ).unwrap_or(0);

    Ok(TutorDashboard {
        total_students,
        active_assignments,
        reports_generated,
        students,
    })
}

// ==================== HELPERS ====================

/// Genera una nueva pregunta para la sesion usando el LLM y la guarda en la base de datos.
///
/// # Parámetros
/// - `db`: Conexion SQLite.
/// - `state`: Contexto de estado compartido `AppState`.
/// - `session_id`: Identificador de la sesion activa.
/// - `profile`: Perfil del estudiante.
///
/// # Retorna
/// La pregunta generada (`CurrentQuestion`).
async fn generate_question_for_session(
    state: &AppState,
    session_id: &str,
    profile: &Profile,
) -> Result<CurrentQuestion, String> {
    let (question_number, total_questions, provider, locale, manual_prompt, concept) = {
        let db = state.db.lock()
            .map_err(|e| format!("[generate_question] Error lockeando db: {e}"))?;
        let session = get_session_by_id(&db, session_id)
            .map_err(|e| format!("[generate_question] Error obteniendo sesion: {e}"))?;
        let question_number = session.questions_answered + 1;
        let total_questions = session.total_questions;

        let manual_prompt = manual_prompt_for_profile(profile);
        let concept = if manual_prompt.is_some() {
            None
        } else {
            Some(get_weakest_concept(&db, &profile.id)
                .unwrap_or_else(|| get_default_concept_for_year(profile.school_year)))
        };

        let provider_guard = state.llm_provider.lock()
            .map_err(|e| format!("[generate_question] Error lockeando provider: {e}"))?;
        let provider = provider_guard.as_ref()
            .ok_or_else(|| "[generate_question] No hay proveedor LLM configurado".to_string())?
            .clone();

        let locale = state.locale.lock()
            .map_err(|e| format!("[generate_question] Error lockeando locale: {e}"))?
            .clone();

        (question_number, total_questions, provider, locale, manual_prompt, concept)
    };

    let llm_question = provider.generate_question(
        profile.school_year,
        profile.current_level,
        concept,
        manual_prompt.as_deref(),
        &locale,
    ).await.map_err(|e| {
        eprintln!("[generate_question] Error del LLM al generar pregunta: {e}");
        format!("El asistente no pudo generar una pregunta: {e}")
    })?;

    let question_id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();

    {
        let db = state.db.lock()
            .map_err(|e| format!("[generate_question] Error lockeando db: {e}"))?;
        db.execute(
            "INSERT INTO session_questions (id, session_id, question_text, correct_answer, concept, difficulty, question_number, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                question_id,
                session_id,
                llm_question.question,
                llm_question.correct_answer,
                llm_question.concept,
                llm_question.difficulty,
                question_number,
                now,
            ],
        ).map_err(|e| format!("[generate_question] Error guardando pregunta en DB: {e}"))?;
    }

    Ok(CurrentQuestion {
        question_id,
        question_text: llm_question.question,
        question_number,
        total_questions,
        concept: llm_question.concept,
        difficulty: llm_question.difficulty,
    })
}

#[cfg(test)]
mod tests {
    use super::evaluate_answer_local;

    #[test]
    fn acepta_respuesta_explicada_con_resultado_final_correcto() {
        let correct_answer = "44 manzanas (7 x 8 = 56 manzanas en total; 56 - 12 = 44 manzanas que le quedan)";
        let student_answer = "Lo primero es multiplicar 7 por 8 = 56 si regala 12 manzanas le tienes que quitar 12 a las 56 manzanas que el resultado es 44 manzanas le quedan";

        let (is_correct, _) = evaluate_answer_local(correct_answer, student_answer);

        assert!(is_correct);
    }

    #[test]
    fn acepta_respuesta_con_unidades_cuando_el_resultado_coincide() {
        let (is_correct, _) = evaluate_answer_local("44", "44 manzanas");

        assert!(is_correct);
    }

    #[test]
    fn rechaza_respuesta_explicada_con_resultado_final_incorrecto() {
        let correct_answer = "44 manzanas (7 x 8 = 56 manzanas en total; 56 - 12 = 44 manzanas que le quedan)";
        let student_answer = "7 por 8 son 56 y si quita 12 creo que quedan 45 manzanas";

        let (is_correct, _) = evaluate_answer_local(correct_answer, student_answer);

        assert!(!is_correct);
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let _ = dotenvy::dotenv();

            for dir in [
                std::env::current_exe().ok().and_then(|p| p.parent().map(|p| p.to_path_buf())),
                app.path().resource_dir().ok(),
                app.path().app_data_dir().ok(),
            ]
            .into_iter()
            .flatten()
            {
                let env_path = dir.join(".env");
                if env_path.exists() {
                    let _ = dotenvy::from_path(&env_path);
                    break;
                }
            }

            let baserow_database_id: i64 = env::var("BASEROW_DATABASE_ID")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(0);
            let baserow_api_token = env::var("BASEROW_API_TOKEN").ok().unwrap_or_default();

            let mut baserow_client = if baserow_database_id > 0 && !baserow_api_token.is_empty() {
                Some(BaserowClient::new(baserow_api_token.clone()))
            } else {
                None
            };

            let email_client = EmailClient::from_env();

            let app_data_dir = app
                .path()
                .app_data_dir()
                .map_err(|err| format!("No se pudo localizar el directorio de datos: {err}"))?;
            fs::create_dir_all(&app_data_dir)
                .map_err(|err| format!("No se pudo crear el directorio de datos: {err}"))?;
            let db_path = app_data_dir.join("mates.sqlite3");
            let db = Connection::open(db_path).map_err(|err| format!("No se pudo abrir SQLite: {err}"))?;
            setup_database(&db)?;

            if baserow_client.is_some() {
                let _ = set_setting(&db, CLOUD_BASEROW_TOKEN_KEY, &baserow_api_token);
                let _ = set_setting(&db, CLOUD_BASEROW_DB_ID_KEY, &baserow_database_id.to_string());
            } else {
                let stored_token = get_setting(&db, CLOUD_BASEROW_TOKEN_KEY).ok().flatten();
                let stored_db_id = get_setting(&db, CLOUD_BASEROW_DB_ID_KEY).ok().flatten();
                if let (Some(token), Some(db_id_str)) = (stored_token, stored_db_id) {
                    if let Ok(db_id) = db_id_str.parse::<i64>() {
                        if db_id > 0 && !token.is_empty() {
                            baserow_client = Some(BaserowClient::new(token));
                        }
                    }
                }
            }

            let llm_config = load_llm_config(&db);
            let provider = build_provider(&llm_config);

            let cloud_session = {
                let auto_login = get_setting(&db, CLOUD_AUTO_LOGIN_KEY).ok().flatten();
                if auto_login.as_deref() == Some("true") {
                    let user_id = get_setting(&db, CLOUD_SESSION_KEY).ok().flatten();
                    if get_setting(&db, CLOUD_EMAIL_VERIFIED_KEY).ok().flatten().is_none() {
                        set_setting(&db, CLOUD_EMAIL_VERIFIED_KEY, "false")
                            .map_err(|err| format!("No se pudo inicializar la configuracion: {err}"))?;
                    }
                    user_id.map(|uid| CloudSession {
                        user_id: uid,
                        user_name: get_setting(&db, CLOUD_USER_NAME_KEY)
                            .ok().flatten().unwrap_or_else(|| "Usuario".to_string()),
                        email: get_setting(&db, CLOUD_EMAIL_KEY)
                            .ok().flatten().unwrap_or_default(),
                    })
                } else {
                    None
                }
            };

            app.manage(AppState {
                db: Mutex::new(db),
                adult_unlocked: Mutex::new(false),
                llm_provider: Mutex::new(Some(provider)),
                llm_config: Mutex::new(llm_config),
                locale: Mutex::new("es-ES".to_string()),
                baserow_client: Mutex::new(baserow_client),
                cloud_session: Mutex::new(cloud_session),
                email_client: Mutex::new(email_client),
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_app_status,
            setup_guardian_pin,
            verify_guardian_pin,
            lock_adult_area,
            set_locale,
            reset_local_data,
            list_profiles,
            create_profile,
            update_profile,
            delete_profile,
            recover_profile,
            list_deleted_profiles,
            delete_session,
            recover_session,
            purge_old_sessions,
            get_llm_config,
            set_llm_config,
            test_llm_connection,
            chat_message,
            start_session,
            generate_question,
            submit_answer,
            get_explanation,
            end_session,
            list_sessions,
            list_deleted_sessions,
            get_session_summary,
            get_dashboard_stats,
            get_concept_stats,
            get_evolution,
            export_sessions,
            create_user,
            login_user,
            list_users,
            create_student_group,
            list_student_groups,
            add_student_to_group,
            remove_student_from_group,
            list_group_students,
            assign_student_to_tutor,
            remove_student_from_tutor,
            list_tutor_students,
            create_assignment,
            list_assignments,
            generate_report,
            list_reports,
            get_tutor_dashboard,
            cloud::commands::register_account,
            cloud::commands::login_account,
            cloud::commands::logout_account,
            cloud::commands::sync_all_data,
            cloud::commands::force_sync_from_cloud,
            cloud::commands::get_cloud_status,
            cloud::commands::set_cloud_auto_login,
            cloud::commands::verify_email_code,
            cloud::commands::resend_verification_code,
            cloud::commands::delete_cloud_account,
            cloud::commands::change_cloud_email,
            send_transac_email,
            list_transac_emails,
            get_email_content,
            get_email_status,
            delete_scheduled_email,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
