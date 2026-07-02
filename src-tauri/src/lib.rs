mod llm;

use std::fs;
use std::sync::Mutex;

use argon2::password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString};
use argon2::Argon2;
use chrono::Utc;
use llm::{LLMExplanation, LLMProviderEnum, LLMQuestion};
use llm::ollama::OllamaProvider;
use llm::gemini::GeminiProvider;
use llm::openai_compatible::OpenAICompatibleProvider;
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use tauri::{Manager, State};
use uuid::Uuid;

const PIN_SETTING_KEY: &str = "guardian_pin_hash";
const LLM_PROVIDER_KEY: &str = "llm_provider";
const LLM_MODEL_KEY: &str = "llm_model";
const LLM_BASE_URL_KEY: &str = "llm_base_url";
const LLM_API_KEY_KEY: &str = "llm_api_key";

struct AppState {
    db: Mutex<Connection>,
    adult_unlocked: Mutex<bool>,
    llm_provider: Mutex<Option<LLMProviderEnum>>,
    llm_config: Mutex<LLMConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct LLMConfig {
    provider: String,
    model: String,
    base_url: String,
    api_key: String,
}

impl Default for LLMConfig {
    fn default() -> Self {
        Self {
            provider: "ollama".to_string(),
            model: "llama3".to_string(),
            base_url: "http://localhost:11434".to_string(),
            api_key: String::new(),
        }
    }
}

#[derive(Debug, Serialize)]
struct AppStatus {
    guardian_pin_set: bool,
    adult_unlocked: bool,
    profiles: Vec<Profile>,
    llm_config: LLMConfig,
}

#[derive(Debug, Serialize, Deserialize)]
struct Profile {
    id: String,
    display_name: String,
    school_year: u8,
    age: Option<u8>,
    level_mode: LevelMode,
    current_level: u8,
    created_at: String,
    updated_at: String,
}

#[derive(Debug, Deserialize)]
struct CreateProfileRequest {
    display_name: String,
    school_year: u8,
    age: Option<u8>,
    level_mode: LevelMode,
    manual_level: Option<u8>,
}

#[derive(Debug, Deserialize)]
struct UpdateProfileRequest {
    id: String,
    display_name: String,
    school_year: u8,
    age: Option<u8>,
    level_mode: LevelMode,
    manual_level: Option<u8>,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
enum LevelMode {
    Automatic,
    Manual,
}

impl LevelMode {
    fn as_str(self) -> &'static str {
        match self {
            Self::Automatic => "automatic",
            Self::Manual => "manual",
        }
    }

    fn from_db(value: String) -> Result<Self, String> {
        match value.as_str() {
            "automatic" => Ok(Self::Automatic),
            "manual" => Ok(Self::Manual),
            _ => Err("Modo de nivel desconocido".to_string()),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct Session {
    id: String,
    profile_id: String,
    status: String,
    total_questions: u8,
    questions_answered: u8,
    correct_count: u8,
    current_question_index: u8,
    started_at: String,
    ended_at: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct SessionQuestion {
    id: String,
    session_id: String,
    question_text: String,
    correct_answer: String,
    student_answer: Option<String>,
    concept: String,
    difficulty: String,
    is_correct: Option<bool>,
    explanation: Option<String>,
    needs_reformulation: Option<bool>,
    reformulated_text: Option<String>,
    question_number: u8,
    time_spent_secs: Option<u32>,
    created_at: String,
    answered_at: Option<String>,
}

#[derive(Debug, Serialize)]
struct SessionSummary {
    session: Session,
    questions: Vec<SessionQuestion>,
    concepts_worked: Vec<String>,
    concepts_mastered: Vec<String>,
    concepts_to_practice: Vec<String>,
    accuracy_pct: f64,
    avg_time_per_question: f64,
    total_time_secs: u32,
}

#[derive(Debug, Serialize)]
struct StartSessionResponse {
    session_id: String,
    total_questions: u8,
    first_question: Option<CurrentQuestion>,
}

#[derive(Debug, Serialize)]
struct CurrentQuestion {
    question_id: String,
    question_text: String,
    question_number: u8,
    total_questions: u8,
    concept: String,
    difficulty: String,
}

#[derive(Debug, Deserialize)]
struct SubmitAnswerRequest {
    session_id: String,
    question_id: String,
    answer: String,
    time_spent_secs: u32,
}

#[derive(Debug, Serialize)]
struct SubmitAnswerResponse {
    is_correct: bool,
    feedback: String,
    correct_answer: String,
    explanation_needed: bool,
    next_question: Option<CurrentQuestion>,
    session_finished: bool,
}

#[derive(Debug, Serialize)]
struct ExplanationResponse {
    explanation: String,
    key_points: Vec<String>,
    next_steps: Vec<String>,
    reformulated_question: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GetQuestionRequest {
    profile_id: String,
}

#[derive(Debug, Serialize)]
#[allow(dead_code)]
struct DashboardStats {
    total_sessions: u32,
    total_questions_answered: u32,
    total_correct: u32,
    overall_accuracy_pct: f64,
    total_time_secs: u32,
    avg_time_per_question: f64,
    concepts_mastered: Vec<String>,
    concepts_in_progress: Vec<String>,
    concepts_needing_practice: Vec<String>,
}

#[derive(Debug, Serialize)]
#[allow(dead_code)]
struct ConceptStat {
    concept: String,
    total_attempts: u32,
    correct_attempts: u32,
    accuracy_pct: f64,
    last_practiced: String,
}

#[derive(Debug, Serialize)]
#[allow(dead_code)]
struct EvolutionPoint {
    session_id: String,
    started_at: String,
    accuracy_pct: f64,
    questions_answered: u8,
    correct_count: u8,
}

#[derive(Debug, Serialize)]
#[allow(dead_code)]
struct ExportSessionRow {
    session_id: String,
    started_at: String,
    ended_at: Option<String>,
    question_number: u8,
    question_text: String,
    concept: String,
    difficulty: String,
    student_answer: Option<String>,
    correct_answer: String,
    is_correct: Option<bool>,
    time_spent_secs: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
struct User {
    id: String,
    display_name: String,
    role: String,
    created_at: String,
}

#[derive(Debug, Deserialize)]
struct CreateUserRequest {
    display_name: String,
    pin: String,
    role: String,
}

#[derive(Debug, Deserialize)]
struct LoginRequest {
    pin: String,
}

#[derive(Debug, Serialize)]
struct StudentGroup {
    id: String,
    name: String,
    owner_user_id: String,
    created_at: String,
}

#[derive(Debug, Deserialize)]
struct CreateStudentGroupRequest {
    name: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct TutorStudent {
    tutor_user_id: String,
    student_id: String,
    assigned_at: String,
    status: String,
}

#[derive(Debug, Deserialize)]
struct AssignStudentRequest {
    student_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Assignment {
    id: String,
    tutor_user_id: String,
    student_id: String,
    concept: String,
    difficulty: String,
    due_date: Option<String>,
    status: String,
    created_at: String,
}

#[derive(Debug, Deserialize)]
struct CreateAssignmentRequest {
    student_id: String,
    concept: String,
    difficulty: String,
    due_date: Option<String>,
}

#[derive(Debug, Serialize)]
struct Report {
    id: String,
    tutor_user_id: String,
    student_id: String,
    period: String,
    report_data: String,
    generated_at: String,
}

#[derive(Debug, Deserialize)]
struct GenerateReportRequest {
    student_id: String,
    period: String,
}

#[derive(Debug, Serialize)]
struct TutorDashboard {
    total_students: u32,
    active_assignments: u32,
    reports_generated: u32,
    students: Vec<TutorStudentInfo>,
}

#[derive(Debug, Serialize)]
struct TutorStudentInfo {
    student_id: String,
    display_name: String,
    school_year: u8,
    current_level: u8,
    last_session: Option<String>,
    accuracy_pct: f64,
}

#[derive(Debug, Deserialize)]
struct LLMConfigRequest {
    provider: String,
    model: String,
    base_url: String,
    api_key: String,
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

    Ok(AppStatus {
        guardian_pin_set: get_setting(&db, PIN_SETTING_KEY)?.is_some(),
        adult_unlocked,
        profiles: list_profiles_from_db(&db)?,
        llm_config,
    })
}

#[tauri::command]
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
fn lock_adult_area(state: State<'_, AppState>) -> Result<(), String> {
    *state.adult_unlocked.lock().map_err(|_| "No se pudo bloquear la zona adulta")? = false;
    Ok(())
}

#[tauri::command]
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
fn list_profiles(state: State<'_, AppState>) -> Result<Vec<Profile>, String> {
    let db = state.db.lock().map_err(|_| "No se pudo acceder a la base de datos")?;
    list_profiles_from_db(&db)
}

#[tauri::command]
fn create_profile(request: CreateProfileRequest, state: State<'_, AppState>) -> Result<Profile, String> {
    require_adult_unlocked(&state)?;
    validate_profile_input(&request.display_name, request.school_year, request.age, request.level_mode, request.manual_level)?;
    let db = state.db.lock().map_err(|_| "No se pudo acceder a la base de datos")?;
    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();
    let current_level = resolve_current_level(request.school_year, request.level_mode, request.manual_level);
    let display_name = request.display_name.trim();
    db.execute(
        "INSERT INTO profiles (id, display_name, school_year, age, level_mode, current_level, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![id, display_name, request.school_year, request.age, request.level_mode.as_str(), current_level, now, now],
    ).map_err(|err| format!("No se pudo crear el perfil: {err}"))?;
    get_profile_by_id(&db, &id)
}

#[tauri::command]
fn update_profile(request: UpdateProfileRequest, state: State<'_, AppState>) -> Result<Profile, String> {
    require_adult_unlocked(&state)?;
    validate_profile_input(&request.display_name, request.school_year, request.age, request.level_mode, request.manual_level)?;
    let db = state.db.lock().map_err(|_| "No se pudo acceder a la base de datos")?;
    let current = get_profile_by_id(&db, &request.id)?;
    let requested_level = resolve_current_level(request.school_year, request.level_mode, request.manual_level);
    let current_level = requested_level.max(current.current_level);
    let now = Utc::now().to_rfc3339();
    let display_name = request.display_name.trim();
    db.execute(
        "UPDATE profiles SET display_name = ?1, school_year = ?2, age = ?3, level_mode = ?4, current_level = ?5, updated_at = ?6 WHERE id = ?7",
        params![display_name, request.school_year, request.age, request.level_mode.as_str(), current_level, now, request.id],
    ).map_err(|err| format!("No se pudo actualizar el perfil: {err}"))?;
    get_profile_by_id(&db, &request.id)
}

#[tauri::command]
fn delete_profile(id: String, state: State<'_, AppState>) -> Result<(), String> {
    require_adult_unlocked(&state)?;
    let db = state.db.lock().map_err(|_| "No se pudo acceder a la base de datos")?;
    db.execute("DELETE FROM session_questions WHERE session_id IN (SELECT id FROM sessions WHERE profile_id = ?1)", params![id])
        .map_err(|err| format!("No se pudieron borrar preguntas: {err}"))?;
    db.execute("DELETE FROM sessions WHERE profile_id = ?1", params![id])
        .map_err(|err| format!("No se pudieron borrar sesiones: {err}"))?;
    db.execute("DELETE FROM profiles WHERE id = ?1", params![id])
        .map_err(|err| format!("No se pudo eliminar el perfil: {err}"))?;
    Ok(())
}

// ==================== LLM CONFIG ====================

#[tauri::command]
fn get_llm_config(state: State<'_, AppState>) -> Result<LLMConfig, String> {
    let config = state.llm_config.lock().map_err(|_| "No se pudo leer la configuracion LLM")?;
    Ok(config.clone())
}

#[tauri::command]
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
fn test_llm_connection(state: State<'_, AppState>) -> Result<String, String> {
    let provider_guard = state.llm_provider.lock().map_err(|_| "No se pudo acceder al proveedor LLM")?;
    let provider = provider_guard.as_ref().ok_or("No hay proveedor LLM configurado")?;

    let rt = tokio::runtime::Runtime::new().map_err(|err| format!("No se pudo crear runtime: {err}"))?;
    rt.block_on(provider.generate_question(1, 1, Some("suma basica".to_string())))
        .map(|q| format!("Conexion OK. Pregunta: {}", q.question))
        .map_err(|err| format!("Error: {err}"))
}

// ==================== SESSIONS ====================

#[tauri::command]
fn start_session(profile_id: String, state: State<'_, AppState>) -> Result<StartSessionResponse, String> {
    let db = state.db.lock().map_err(|_| "No se pudo acceder a la base de datos")?;
    let profile = get_profile_by_id(&db, &profile_id)?;
    let session_id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();

    db.execute(
        "INSERT INTO sessions (id, profile_id, status, total_questions, questions_answered, correct_count, current_question_index, started_at)
         VALUES (?1, ?2, 'active', 10, 0, 0, 0, ?3)",
        params![session_id, profile_id, now],
    ).map_err(|err| format!("No se pudo crear la sesion: {err}"))?;

    let first_question = generate_question_for_session(&db, &state, &session_id, &profile);

    Ok(StartSessionResponse {
        session_id,
        total_questions: 10,
        first_question,
    })
}

#[tauri::command]
fn generate_question(request: GetQuestionRequest, state: State<'_, AppState>) -> Result<CurrentQuestion, String> {
    let db = state.db.lock().map_err(|_| "No se pudo acceder a la base de datos")?;
    let profile = get_profile_by_id(&db, &request.profile_id)?;
    let session_id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();

    db.execute(
        "INSERT INTO sessions (id, profile_id, status, total_questions, questions_answered, correct_count, current_question_index, started_at)
         VALUES (?1, ?2, 'active', 10, 0, 0, 0, ?3)",
        params![session_id, request.profile_id, now],
    ).map_err(|err| format!("No se pudo crear sesion temporal: {err}"))?;

    generate_question_for_session(&db, &state, &session_id, &profile)
        .ok_or_else(|| "No se pudo generar la pregunta".to_string())
}

#[tauri::command]
fn submit_answer(request: SubmitAnswerRequest, state: State<'_, AppState>) -> Result<SubmitAnswerResponse, String> {
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

    let next_question = if !finished {
        let profile = get_profile_by_id(&db, &session.profile_id)?;
        generate_question_for_session(&db, &state, &request.session_id, &profile)
    } else {
        None
    };

    Ok(SubmitAnswerResponse {
        is_correct,
        feedback,
        correct_answer: question.correct_answer,
        explanation_needed: !is_correct,
        next_question,
        session_finished: finished,
    })
}

#[tauri::command]
fn get_explanation(question_id: String, state: State<'_, AppState>) -> Result<ExplanationResponse, String> {
    let db = state.db.lock().map_err(|_| "No se pudo acceder a la base de datos")?;
    let question = get_question_by_id(&db, &question_id)?;

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
            rt.block_on(provider.reformulate_concept(&question.concept, &llm_question)).ok()
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
fn get_dashboard_stats(profile_id: String, state: State<'_, AppState>) -> Result<DashboardStats, String> {
    let db = state.db.lock().map_err(|_| "No se pudo acceder a la base de datos")?;
    let mut stmt = db.prepare(
        "SELECT total_questions, questions_answered, correct_count, started_at, ended_at
         FROM sessions WHERE profile_id = ?1 AND status = 'completed' ORDER BY started_at",
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
fn get_concept_stats(profile_id: String, state: State<'_, AppState>) -> Result<Vec<ConceptStat>, String> {
    let db = state.db.lock().map_err(|_| "No se pudo acceder a la base de datos")?;
    get_concept_stats_for_profile(&db, &profile_id).map_err(|err| format!("Error al obtener estadisticas: {err}"))
}

fn get_concept_stats_for_profile(db: &rusqlite::Connection, profile_id: &str) -> Result<Vec<ConceptStat>, rusqlite::Error> {
    let mut stmt = db.prepare(
        "SELECT sq.concept,
                COUNT(*) as total,
                SUM(CASE WHEN sq.is_correct = 1 THEN 1 ELSE 0 END) as correct,
                MAX(sq.answered_at) as last_practiced
         FROM session_questions sq
         JOIN sessions s ON sq.session_id = s.id
         WHERE s.profile_id = ?1 AND s.status = 'completed' AND sq.is_correct IS NOT NULL
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

#[tauri::command]
fn get_evolution(profile_id: String, state: State<'_, AppState>) -> Result<Vec<EvolutionPoint>, String> {
    let db = state.db.lock().map_err(|_| "No se pudo acceder a la base de datos")?;
    let mut stmt = db.prepare(
        "SELECT id, started_at, total_questions, correct_count
         FROM sessions WHERE profile_id = ?1 AND status = 'completed'
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
fn export_sessions(profile_id: String, state: State<'_, AppState>) -> Result<Vec<ExportSessionRow>, String> {
    let db = state.db.lock().map_err(|_| "No se pudo acceder a la base de datos")?;
    let mut stmt = db.prepare(
        "SELECT sq.session_id, s.started_at, s.ended_at, sq.question_number, sq.question_text,
                sq.concept, sq.difficulty, sq.student_answer, sq.correct_answer, sq.is_correct, sq.time_spent_secs
         FROM session_questions sq
         JOIN sessions s ON sq.session_id = s.id
         WHERE s.profile_id = ?1 AND s.status = 'completed'
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
fn list_sessions(profile_id: String, state: State<'_, AppState>) -> Result<Vec<Session>, String> {
    let db = state.db.lock().map_err(|_| "No se pudo acceder a la base de datos")?;
    let mut stmt = db.prepare(
        "SELECT id, profile_id, status, total_questions, questions_answered, correct_count, current_question_index, started_at, ended_at
         FROM sessions WHERE profile_id = ?1 ORDER BY started_at DESC",
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

// ==================== PROFESSIONAL LAYER ====================

#[tauri::command]
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
fn add_student_to_group(group_id: String, student_id: String, state: State<'_, AppState>) -> Result<(), String> {
    let db = state.db.lock().map_err(|_| "No se pudo acceder a la base de datos")?;
    db.execute(
        "INSERT OR IGNORE INTO student_group_members (group_id, student_id) VALUES (?1, ?2)",
        params![group_id, student_id],
    ).map_err(|err| format!("No se pudo agregar el estudiante al grupo: {err}"))?;
    Ok(())
}

#[tauri::command]
fn remove_student_from_group(group_id: String, student_id: String, state: State<'_, AppState>) -> Result<(), String> {
    let db = state.db.lock().map_err(|_| "No se pudo acceder a la base de datos")?;
    db.execute(
        "DELETE FROM student_group_members WHERE group_id = ?1 AND student_id = ?2",
        params![group_id, student_id],
    ).map_err(|err| format!("No se pudo remover el estudiante del grupo: {err}"))?;
    Ok(())
}

#[tauri::command]
fn list_group_students(group_id: String, state: State<'_, AppState>) -> Result<Vec<Profile>, String> {
    let db = state.db.lock().map_err(|_| "No se pudo acceder a la base de datos")?;
    let mut stmt = db.prepare(
        "SELECT p.id, p.display_name, p.school_year, p.age, p.level_mode, p.current_level, p.created_at, p.updated_at
         FROM profiles p
         JOIN student_group_members sgm ON p.id = sgm.student_id
         WHERE sgm.group_id = ?1
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
            created_at: row.get(6)?,
            updated_at: row.get(7)?,
        })
    }).map_err(|err| format!("No se pudieron leer los estudiantes: {err}"))?;

    let mut students = Vec::new();
    for row in rows {
        students.push(row.map_err(|err| format!("Estudiante invalido: {err}"))?);
    }
    Ok(students)
}

#[tauri::command]
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
fn remove_student_from_tutor(tutor_user_id: String, student_id: String, state: State<'_, AppState>) -> Result<(), String> {
    let db = state.db.lock().map_err(|_| "No se pudo acceder a la base de datos")?;
    db.execute(
        "UPDATE tutor_student SET status = 'inactive' WHERE tutor_user_id = ?1 AND student_id = ?2",
        params![tutor_user_id, student_id],
    ).map_err(|err| format!("No se pudo remover la asignacion: {err}"))?;
    Ok(())
}

#[tauri::command]
fn list_tutor_students(tutor_user_id: String, state: State<'_, AppState>) -> Result<Vec<Profile>, String> {
    let db = state.db.lock().map_err(|_| "No se pudo acceder a la base de datos")?;
    let mut stmt = db.prepare(
        "SELECT p.id, p.display_name, p.school_year, p.age, p.level_mode, p.current_level, p.created_at, p.updated_at
         FROM profiles p
         JOIN tutor_student ts ON p.id = ts.student_id
         WHERE ts.tutor_user_id = ?1 AND ts.status = 'active'
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
            created_at: row.get(6)?,
            updated_at: row.get(7)?,
        })
    }).map_err(|err| format!("No se pudieron leer los estudiantes: {err}"))?;

    let mut students = Vec::new();
    for row in rows {
        students.push(row.map_err(|err| format!("Estudiante invalido: {err}"))?);
    }
    Ok(students)
}

#[tauri::command]
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
fn get_tutor_dashboard(tutor_user_id: String, state: State<'_, AppState>) -> Result<TutorDashboard, String> {
    let db = state.db.lock().map_err(|_| "No se pudo acceder a la base de datos")?;

    let mut stmt_students = db.prepare(
        "SELECT p.id, p.display_name, p.school_year, p.current_level
         FROM profiles p
         JOIN tutor_student ts ON p.id = ts.student_id
         WHERE ts.tutor_user_id = ?1 AND ts.status = 'active'",
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
            "SELECT started_at FROM sessions WHERE profile_id = ?1 AND status = 'completed' ORDER BY started_at DESC LIMIT 1",
            params![id],
            |row| row.get(0),
        ).unwrap_or(None);

        let accuracy: f64 = db.query_row(
            "SELECT COALESCE(SUM(CASE WHEN is_correct = 1 THEN 1 ELSE 0 END) * 100.0 / NULLIF(COUNT(*), 0), 0)
             FROM session_questions sq
             JOIN sessions s ON sq.session_id = s.id
             WHERE s.profile_id = ?1 AND s.status = 'completed' AND sq.is_correct IS NOT NULL",
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

fn build_provider(config: &LLMConfig) -> LLMProviderEnum {
    match config.provider.as_str() {
        "gemini" => LLMProviderEnum::Gemini(GeminiProvider::new(
            Some(config.api_key.clone()),
            Some(config.model.clone()),
            None,
        )),
        "openai" => LLMProviderEnum::OpenAICompatible(OpenAICompatibleProvider::new(
            Some(config.base_url.clone()),
            Some(config.api_key.clone()),
            Some(config.model.clone()),
            None,
        )),
        _ => LLMProviderEnum::Ollama(OllamaProvider::new(
            Some(config.base_url.clone()),
            Some(config.model.clone()),
            None,
        )),
    }
}

fn generate_question_for_session(
    db: &Connection,
    state: &State<'_, AppState>,
    session_id: &str,
    profile: &Profile,
) -> Option<CurrentQuestion> {
    let session = get_session_by_id(db, session_id).ok()?;
    let question_number = session.questions_answered + 1;

    let provider_guard = state.llm_provider.lock().ok()?;
    let provider = provider_guard.as_ref()?;

    let concept = get_weakest_concept(db, &profile.id);

    let llm_question = {
        let rt = tokio::runtime::Runtime::new().ok()?;
        rt.block_on(provider.generate_question(
            profile.school_year,
            profile.current_level,
            concept,
        )).ok()?
    };

    let question_id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();

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
    ).ok()?;

    Some(CurrentQuestion {
        question_id,
        question_text: llm_question.question,
        question_number,
        total_questions: session.total_questions,
        concept: llm_question.concept,
        difficulty: llm_question.difficulty,
    })
}

fn evaluate_answer_local(correct_answer: &str, student_answer: &str) -> (bool, String) {
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

fn generate_default_explanation(question: &SessionQuestion) -> String {
    format!(
        "El problema \"{}\" se resuelve con el concepto de {}. La respuesta correcta es {}.",
        question.question_text, question.concept, question.correct_answer
    )
}

fn get_weakest_concept(db: &Connection, profile_id: &str) -> Option<String> {
    let mut stmt = db.prepare(
        "SELECT concept, COUNT(*) as total, SUM(CASE WHEN is_correct = 1 THEN 1 ELSE 0 END) as correct
         FROM session_questions sq
         JOIN sessions s ON sq.session_id = s.id
         WHERE s.profile_id = ?1 AND sq.is_correct IS NOT NULL
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

fn get_session_by_id(db: &Connection, id: &str) -> Result<Session, String> {
    db.query_row(
        "SELECT id, profile_id, status, total_questions, questions_answered, correct_count, current_question_index, started_at, ended_at
         FROM sessions WHERE id = ?1",
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

fn get_question_by_id(db: &Connection, id: &str) -> Result<SessionQuestion, String> {
    db.query_row(
        "SELECT id, session_id, question_text, correct_answer, student_answer, concept, difficulty, is_correct, explanation, needs_reformulation, reformulated_text, question_number, time_spent_secs, created_at, answered_at
         FROM session_questions WHERE id = ?1",
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

fn list_questions_for_session(db: &Connection, session_id: &str) -> Result<Vec<SessionQuestion>, String> {
    let mut stmt = db.prepare(
        "SELECT id, session_id, question_text, correct_answer, student_answer, concept, difficulty, is_correct, explanation, needs_reformulation, reformulated_text, question_number, time_spent_secs, created_at, answered_at
         FROM session_questions WHERE session_id = ?1 ORDER BY question_number",
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

fn require_adult_unlocked(state: &State<'_, AppState>) -> Result<(), String> {
    let unlocked = *state.adult_unlocked.lock().map_err(|_| "No se pudo comprobar la sesion adulta")?;
    if unlocked { Ok(()) } else { Err("La zona adulta esta bloqueada".to_string()) }
}

fn validate_pin(pin: &str) -> Result<(), String> {
    if !(4..=6).contains(&pin.len()) || !pin.chars().all(|c| c.is_ascii_digit()) {
        return Err("El PIN debe tener entre 4 y 6 digitos".to_string());
    }
    Ok(())
}

fn hash_pin(pin: &str) -> Result<String, String> {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(pin.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|err| format!("No se pudo proteger el PIN: {err}"))
}

fn verify_pin(pin: &str, pin_hash: &str) -> Result<bool, String> {
    let parsed_hash = PasswordHash::new(pin_hash).map_err(|err| format!("PIN guardado invalido: {err}"))?;
    Ok(Argon2::default().verify_password(pin.as_bytes(), &parsed_hash).is_ok())
}

fn validate_profile_input(display_name: &str, school_year: u8, age: Option<u8>, level_mode: LevelMode, manual_level: Option<u8>) -> Result<(), String> {
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
    Ok(())
}

fn resolve_current_level(school_year: u8, level_mode: LevelMode, manual_level: Option<u8>) -> u8 {
    match level_mode {
        LevelMode::Automatic => school_year,
        LevelMode::Manual => manual_level.unwrap_or(school_year).max(school_year),
    }
}

fn get_setting(db: &Connection, key: &str) -> Result<Option<String>, String> {
    db.query_row("SELECT value FROM app_settings WHERE key = ?1", params![key], |row| row.get(0))
        .optional()
        .map_err(|err| format!("No se pudo leer la configuracion: {err}"))
}

fn set_setting(db: &Connection, key: &str, value: &str) -> Result<(), String> {
    let now = Utc::now().to_rfc3339();
    db.execute(
        "INSERT INTO app_settings (key, value, updated_at) VALUES (?1, ?2, ?3)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = excluded.updated_at",
        params![key, value, now],
    ).map_err(|err| format!("No se pudo guardar la configuracion: {err}"))?;
    Ok(())
}

fn list_profiles_from_db(db: &Connection) -> Result<Vec<Profile>, String> {
    let mut stmt = db.prepare(
        "SELECT id, display_name, school_year, age, level_mode, current_level, created_at, updated_at
         FROM profiles ORDER BY school_year, display_name COLLATE NOCASE",
    ).map_err(|err| format!("No se pudieron preparar los perfiles: {err}"))?;

    let rows = stmt.query_map([], profile_from_row)
        .map_err(|err| format!("No se pudieron leer los perfiles: {err}"))?;

    let mut profiles = Vec::new();
    for row in rows {
        profiles.push(row.map_err(|err| format!("Perfil invalido: {err}"))?);
    }
    Ok(profiles)
}

fn get_profile_by_id(db: &Connection, id: &str) -> Result<Profile, String> {
    db.query_row(
        "SELECT id, display_name, school_year, age, level_mode, current_level, created_at, updated_at
         FROM profiles WHERE id = ?1",
        params![id],
        profile_from_row,
    ).optional()
    .map_err(|err| format!("No se pudo leer el perfil: {err}"))?
    .ok_or_else(|| "Perfil no encontrado".to_string())
}

fn profile_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Profile> {
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
        created_at: row.get(6)?,
        updated_at: row.get(7)?,
    })
}

fn setup_database(db: &Connection) -> Result<(), String> {
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
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
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
            ended_at TEXT
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
            answered_at TEXT
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
    Ok(())
}

fn load_llm_config(db: &Connection) -> LLMConfig {
    LLMConfig {
        provider: get_setting(db, LLM_PROVIDER_KEY).ok().flatten().unwrap_or_else(|| "ollama".to_string()),
        model: get_setting(db, LLM_MODEL_KEY).ok().flatten().unwrap_or_else(|| "llama3".to_string()),
        base_url: get_setting(db, LLM_BASE_URL_KEY).ok().flatten().unwrap_or_else(|| "http://localhost:11434".to_string()),
        api_key: get_setting(db, LLM_API_KEY_KEY).ok().flatten().unwrap_or_default(),
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let app_data_dir = app
                .path()
                .app_data_dir()
                .map_err(|err| format!("No se pudo localizar el directorio de datos: {err}"))?;
            fs::create_dir_all(&app_data_dir)
                .map_err(|err| format!("No se pudo crear el directorio de datos: {err}"))?;
            let db_path = app_data_dir.join("mates.sqlite3");
            let db = Connection::open(db_path).map_err(|err| format!("No se pudo abrir SQLite: {err}"))?;
            setup_database(&db)?;

            let llm_config = load_llm_config(&db);
            let provider = build_provider(&llm_config);

            app.manage(AppState {
                db: Mutex::new(db),
                adult_unlocked: Mutex::new(false),
                llm_provider: Mutex::new(Some(provider)),
                llm_config: Mutex::new(llm_config),
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_app_status,
            setup_guardian_pin,
            verify_guardian_pin,
            lock_adult_area,
            reset_local_data,
            list_profiles,
            create_profile,
            update_profile,
            delete_profile,
            get_llm_config,
            set_llm_config,
            test_llm_connection,
            start_session,
            generate_question,
            submit_answer,
            get_explanation,
            end_session,
            list_sessions,
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
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
