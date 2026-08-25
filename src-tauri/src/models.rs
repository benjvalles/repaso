use std::sync::Mutex;

use serde::{Deserialize, Serialize};

/// Clave en `app_settings` para el hash del PIN del tutor.
pub const PIN_SETTING_KEY: &str = "guardian_pin_hash";
/// Clave en `app_settings` para el proveedor LLM seleccionado.
pub const LLM_PROVIDER_KEY: &str = "llm_provider";
/// Clave en `app_settings` para el modelo LLM seleccionado.
pub const LLM_MODEL_KEY: &str = "llm_model";
/// Clave en `app_settings` para la URL base del LLM.
pub const LLM_BASE_URL_KEY: &str = "llm_base_url";
/// Clave en `app_settings` para la clave de API del LLM.
pub const LLM_API_KEY_KEY: &str = "llm_api_key";
/// Clave en `app_settings` para el ID de usuario de sesion en la nube.
pub const CLOUD_SESSION_KEY: &str = "cloud_session_user_id";
/// Clave en `app_settings` para la marca de ultima sincronizacion en la nube.
pub const CLOUD_LAST_SYNC_KEY: &str = "cloud_last_sync";
/// Clave en `app_settings` para el flag de inicio de sesion automatico en la nube.
pub const CLOUD_AUTO_LOGIN_KEY: &str = "cloud_auto_login";
/// Clave en `app_settings` para el nombre de usuario de sesion en la nube.
pub const CLOUD_USER_NAME_KEY: &str = "cloud_session_user_name";
/// Clave en `app_settings` para el email de sesion en la nube.
pub const CLOUD_EMAIL_KEY: &str = "cloud_session_email";
/// Clave en `app_settings` para el codigo de verificacion de email.
pub const CLOUD_VERIFICATION_CODE_KEY: &str = "cloud_verification_code";
/// Clave en `app_settings` para el flag de email verificado.
pub const CLOUD_EMAIL_VERIFIED_KEY: &str = "cloud_email_verified";

/// Configuracion del proveedor LLM (Ollama, Gemini, OpenAI-compatible).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMConfig {
    pub provider: String,
    pub model: String,
    pub base_url: String,
    pub api_key: String,
}

/// Valores por defecto para `LLMConfig` (Ollama, llama3, localhost:11434).
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

/// Estado general de la aplicacion devuelto al frontend al iniciar.
#[derive(Debug, Serialize)]
pub struct AppStatus {
    pub guardian_pin_set: bool,
    pub adult_unlocked: bool,
    pub profiles: Vec<Profile>,
    pub llm_config: LLMConfig,
    pub cloud_status: crate::cloud::CloudStatus,
}

/// Perfil de estudiante almacenado en la tabla `profiles`.
#[derive(Debug, Serialize, Deserialize)]
pub struct Profile {
    pub id: String,
    pub display_name: String,
    pub school_year: u8,
    pub age: Option<u8>,
    pub level_mode: LevelMode,
    pub current_level: u8,
    pub manual_prompt: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// Datos de entrada para crear un nuevo perfil de estudiante.
#[derive(Debug, Deserialize)]
pub struct CreateProfileRequest {
    pub display_name: String,
    pub school_year: u8,
    pub age: Option<u8>,
    pub level_mode: LevelMode,
    pub manual_level: Option<u8>,
    pub manual_prompt: Option<String>,
}

/// Datos de entrada para actualizar un perfil de estudiante existente.
#[derive(Debug, Deserialize)]
pub struct UpdateProfileRequest {
    pub id: String,
    pub display_name: String,
    pub school_year: u8,
    pub age: Option<u8>,
    pub level_mode: LevelMode,
    pub manual_level: Option<u8>,
    pub manual_prompt: Option<String>,
}

/// Modo de nivel del perfil: automatico (sigue el curso) o manual (configurado por el tutor).
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LevelMode {
    Automatic,
    Manual,
}

impl LevelMode {
    /// Devuelve la representacion en string del modo de nivel.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Automatic => "automatic",
            Self::Manual => "manual",
        }
    }

    /// Construye un `LevelMode` desde el valor almacenado en base de datos.
    pub fn from_db(value: String) -> Result<Self, String> {
        match value.as_str() {
            "automatic" => Ok(Self::Automatic),
            "manual" => Ok(Self::Manual),
            _ => Err("Modo de nivel desconocido".to_string()),
        }
    }
}

/// Sesion de practica almacenada en la tabla `sessions`.
#[derive(Debug, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub profile_id: String,
    pub status: String,
    pub total_questions: u8,
    pub questions_answered: u8,
    pub correct_count: u8,
    pub current_question_index: u8,
    pub started_at: String,
    pub ended_at: Option<String>,
}

/// Pregunta individual dentro de una sesion, almacenada en `session_questions`.
#[derive(Debug, Serialize, Deserialize)]
pub struct SessionQuestion {
    pub id: String,
    pub session_id: String,
    pub question_text: String,
    pub correct_answer: String,
    pub student_answer: Option<String>,
    pub concept: String,
    pub difficulty: String,
    pub is_correct: Option<bool>,
    pub explanation: Option<String>,
    pub needs_reformulation: Option<bool>,
    pub reformulated_text: Option<String>,
    pub question_number: u8,
    pub time_spent_secs: Option<u32>,
    pub created_at: String,
    pub answered_at: Option<String>,
}

/// Resumen de una sesion completada, incluyendo preguntas y metricas por concepto.
#[derive(Debug, Serialize)]
pub struct SessionSummary {
    pub session: Session,
    pub questions: Vec<SessionQuestion>,
    pub concepts_worked: Vec<String>,
    pub concepts_mastered: Vec<String>,
    pub concepts_to_practice: Vec<String>,
    pub accuracy_pct: f64,
    pub avg_time_per_question: f64,
    pub total_time_secs: u32,
}

/// Respuesta al iniciar una sesion, con el ID y la primera pregunta.
#[derive(Debug, Serialize)]
pub struct StartSessionResponse {
    pub session_id: String,
    pub total_questions: u8,
    pub first_question: Option<CurrentQuestion>,
}

/// Pregunta actual presentada al estudiante durante una sesion.
#[derive(Debug, Serialize)]
pub struct CurrentQuestion {
    pub question_id: String,
    pub question_text: String,
    pub question_number: u8,
    pub total_questions: u8,
    pub concept: String,
    pub difficulty: String,
}

/// Datos de entrada para enviar la respuesta del estudiante a una pregunta.
#[derive(Debug, Deserialize)]
pub struct SubmitAnswerRequest {
    pub session_id: String,
    pub question_id: String,
    pub answer: String,
    pub time_spent_secs: u32,
}

/// Respuesta tras evaluar la respuesta del estudiante, con retroalimentacion y siguiente paso.
#[derive(Debug, Serialize)]
pub struct SubmitAnswerResponse {
    pub is_correct: bool,
    pub feedback: String,
    pub correct_answer: String,
    pub explanation_needed: bool,
    pub next_question: Option<CurrentQuestion>,
    pub session_finished: bool,
}

/// Explicacion detallada de una pregunta, generada por el LLM.
#[derive(Debug, Serialize)]
pub struct ExplanationResponse {
    pub explanation: String,
    pub key_points: Vec<String>,
    pub next_steps: Vec<String>,
    pub reformulated_question: Option<String>,
}

/// Datos de entrada para solicitar una nueva pregunta adaptativa.
#[derive(Debug, Deserialize)]
pub struct GetQuestionRequest {
    pub profile_id: String,
}

/// Estadisticas globales del panel de control de un perfil.
#[derive(Debug, Serialize)]
#[allow(dead_code)]
pub struct DashboardStats {
    pub total_sessions: u32,
    pub total_questions_answered: u32,
    pub total_correct: u32,
    pub overall_accuracy_pct: f64,
    pub total_time_secs: u32,
    pub avg_time_per_question: f64,
    pub concepts_mastered: Vec<String>,
    pub concepts_in_progress: Vec<String>,
    pub concepts_needing_practice: Vec<String>,
}

/// Estadisticas de rendimiento agrupadas por concepto.
#[derive(Debug, Serialize)]
#[allow(dead_code)]
pub struct ConceptStat {
    pub concept: String,
    pub total_attempts: u32,
    pub correct_attempts: u32,
    pub accuracy_pct: f64,
    pub last_practiced: String,
}

/// Punto de evolucion temporal de una sesion para graficos de progreso.
#[derive(Debug, Serialize)]
#[allow(dead_code)]
pub struct EvolutionPoint {
    pub session_id: String,
    pub started_at: String,
    pub accuracy_pct: f64,
    pub questions_answered: u8,
    pub correct_count: u8,
}

/// Fila de exportacion CSV con los datos de una pregunta respondida.
#[derive(Debug, Serialize)]
#[allow(dead_code)]
pub struct ExportSessionRow {
    pub session_id: String,
    pub started_at: String,
    pub ended_at: Option<String>,
    pub question_number: u8,
    pub question_text: String,
    pub concept: String,
    pub difficulty: String,
    pub student_answer: Option<String>,
    pub correct_answer: String,
    pub is_correct: Option<bool>,
    pub time_spent_secs: Option<u32>,
}

/// Usuario del sistema (tutor, padre o administrador), almacenado en `users`.
#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub display_name: String,
    pub role: String,
    pub created_at: String,
}

/// Datos de entrada para crear un nuevo usuario.
#[derive(Debug, Deserialize)]
pub struct CreateUserRequest {
    pub display_name: String,
    pub pin: String,
    pub role: String,
}

/// Datos de entrada para iniciar sesion con PIN.
#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub pin: String,
}

/// Grupo de estudiantes creado por un tutor, almacenado en `student_groups`.
#[derive(Debug, Serialize)]
pub struct StudentGroup {
    pub id: String,
    pub name: String,
    pub owner_user_id: String,
    pub created_at: String,
}

/// Datos de entrada para crear un grupo de estudiantes.
#[derive(Debug, Deserialize)]
pub struct CreateStudentGroupRequest {
    pub name: String,
}

/// Relacion tutor-estudiante almacenada en `tutor_student`.
#[derive(Debug, Serialize, Deserialize)]
pub struct TutorStudent {
    pub tutor_user_id: String,
    pub student_id: String,
    pub assigned_at: String,
    pub status: String,
}

/// Datos de entrada para asignar un estudiante a un tutor.
#[derive(Debug, Deserialize)]
pub struct AssignStudentRequest {
    pub student_id: String,
}

/// Tarea asignada por un tutor a un estudiante, almacenada en `assignments`.
#[derive(Debug, Serialize, Deserialize)]
pub struct Assignment {
    pub id: String,
    pub tutor_user_id: String,
    pub student_id: String,
    pub concept: String,
    pub difficulty: String,
    pub due_date: Option<String>,
    pub status: String,
    pub created_at: String,
}

/// Datos de entrada para crear una nueva tarea asignada.
#[derive(Debug, Deserialize)]
pub struct CreateAssignmentRequest {
    pub student_id: String,
    pub concept: String,
    pub difficulty: String,
    pub due_date: Option<String>,
}

/// Informe generado por un tutor sobre el progreso de un estudiante, almacenado en `reports`.
#[derive(Debug, Serialize)]
pub struct Report {
    pub id: String,
    pub tutor_user_id: String,
    pub student_id: String,
    pub period: String,
    pub report_data: String,
    pub generated_at: String,
}

/// Datos de entrada para generar un informe de progreso.
#[derive(Debug, Deserialize)]
pub struct GenerateReportRequest {
    pub student_id: String,
    pub period: String,
}

/// Panel de control del tutor con resumen de estudiantes y tareas activas.
#[derive(Debug, Serialize)]
pub struct TutorDashboard {
    pub total_students: u32,
    pub active_assignments: u32,
    pub reports_generated: u32,
    pub students: Vec<TutorStudentInfo>,
}

/// Informacion resumida de un estudiante para el panel del tutor.
#[derive(Debug, Serialize)]
pub struct TutorStudentInfo {
    pub student_id: String,
    pub display_name: String,
    pub school_year: u8,
    pub current_level: u8,
    pub last_session: Option<String>,
    pub accuracy_pct: f64,
}

/// Datos de entrada para actualizar la configuracion del LLM.
#[derive(Debug, Deserialize)]
pub struct LLMConfigRequest {
    pub provider: String,
    pub model: String,
    pub base_url: String,
    pub api_key: String,
}

/// Trait para acceder al flag `adult_unlocked` desde helpers sin acoplar a `AppState`.
///
/// # Metodos requeridos
///
/// * `adult_unlocked` - Referencia al `Mutex<bool>` que indica si la zona adulta esta desbloqueada.
pub trait HasAdultUnlocked {
    fn adult_unlocked(&self) -> &Mutex<bool>;
}

/// Solicitud de registro en la nube (Baserow).
///
/// # Campos
///
/// * `name` - Nombre del usuario.
/// * `email` - Correo electronico del usuario.
/// * `password` - Contrasena de acceso.
/// * `consent` - Consentimiento para el tratamiento de datos.
#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub name: String,
    pub email: String,
    pub password: String,
    pub consent: bool,
}

/// Solicitud de inicio de sesion en la nube (Baserow).
///
/// # Campos
///
/// * `email` - Correo electronico del usuario.
/// * `password` - Contrasena de acceso.
#[derive(Debug, Deserialize)]
pub struct CloudLoginRequest {
    pub email: String,
    pub password: String,
}

/// Solicitud de mensaje de chat libre con el LLM.
///
/// # Campos
///
/// * `message` - Mensaje del usuario.
/// * `profile_id` - ID del perfil del niño (para conocer edad y curso).
#[derive(Debug, Deserialize)]
pub struct ChatMessageRequest {
    pub message: String,
    pub profile_id: String,
}

/// Respuesta del LLM en el chat libre.
///
/// # Campos
///
/// * `response` - Texto de respuesta del asistente.
#[derive(Debug, Serialize)]
pub struct ChatMessageResponse {
    pub response: String,
}
