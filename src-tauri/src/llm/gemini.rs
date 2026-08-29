use super::{LLMQuestion, LLMExplanation};
use super::common;

use reqwest::Client;
use std::time::Duration;

/// Proveedor LLM cloud que se comunica con la API de Google Gemini.
#[derive(Debug, Clone)]
pub struct GeminiProvider {
    api_key: String,
    model: String,
    timeout: Duration,
}

impl GeminiProvider {
    /// Crea una nueva instancia del proveedor Gemini.
    ///
    /// # Parámetros
    /// - `api_key`: API key de Google AI Studio.
    /// - `model`: Nombre del modelo. Por defecto `gemini-1.5-flash`.
    /// - `timeout_secs`: Timeout en segundos. Por defecto 60.
    pub fn new(api_key: Option<String>, model: Option<String>, timeout_secs: Option<u64>) -> Self {
        Self {
            api_key: api_key.unwrap_or_default(),
            model: model.unwrap_or_else(|| "gemini-1.5-flash".to_string()),
            timeout: Duration::from_secs(timeout_secs.unwrap_or(60)),
        }
    }

    /// Envía un prompt a la API de Gemini y retorna la respuesta como texto.
    ///
    /// # Parámetros
    /// - `system_prompt`: Instrucciones del sistema.
    /// - `user_prompt`: Pregunta o tarea específica.
    ///
    /// # Retorna
    /// El texto generado por Gemini.
    async fn generate_text(&self, system_prompt: &str, user_prompt: &str) -> Result<String, String> {
        if self.api_key.is_empty() {
            return Err("No se ha configurado la API key de Gemini".to_string());
        }

        let client = Client::builder().timeout(self.timeout).build().map_err(|e| e.to_string())?;

        let full_text = format!("{system_prompt}\n\n{user_prompt}");
        let request = serde_json::json!({
            "contents": [{"parts": [{"text": full_text}]}],
            "generation_config": {"temperature": 0.3, "max_output_tokens": 2048}
        });

        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
            self.model, self.api_key
        );

        common::log_llm_prompt("gemini", &self.model, system_prompt, user_prompt);

        let json = common::chat_request(
            &client,
            &url,
            "",
            &request,
            "gemini",
            &self.model,
        ).await?;

        json["candidates"][0]["content"]["parts"][0]["text"]
            .as_str()
            .map(String::from)
            .ok_or_else(|| "Respuesta vacia de Gemini".to_string())
    }

    /// Genera una pregunta de matemáticas usando Gemini.
    ///
    /// # Parámetros
    /// - `year`: Curso del alumno (1-6 Primaria).
    /// - `level`: Nivel de dificultad actual.
    /// - `concept`: Concepto específico. Si es `None`, se elige automáticamente.
    /// - `manual_prompt`: Contexto pedagogico adicional definido por el adulto.
    ///
    /// # Retorna
    /// `LLMQuestion` con la pregunta, respuesta correcta, concepto y dificultad.
    pub async fn generate_question(&self, year: u8, level: u8, concept: Option<String>, manual_prompt: Option<&str>, locale: &str) -> Result<LLMQuestion, String> {
        let (system, prompt) = common::build_question_prompt(year, level, concept, manual_prompt, locale);
        let response = self.generate_text(&system, &prompt).await?;
        common::parse_question_response(&response)
    }

    /// Genera una explicación motivadora cuando el estudiante falla.
    ///
    /// # Parámetros
    /// - `question`: Pregunta que falló.
    /// - `student_answer`: Respuesta del alumno.
    /// - `expected_answer`: Respuesta correcta.
    /// - `concept`: Concepto evaluado.
    /// - `manual_prompt`: Contexto pedagogico adicional definido por el adulto.
    ///
    /// # Retorna
    /// `LLMExplanation` con explicación, puntos clave y siguientes pasos.
    pub async fn provide_explanation(&self, question: &LLMQuestion, student_answer: &str, expected_answer: &str, concept: &str, manual_prompt: Option<&str>, locale: &str) -> Result<LLMExplanation, String> {
        let (system, prompt) = common::build_explanation_prompt(&question.question, student_answer, expected_answer, concept, manual_prompt, locale);
        let response = self.generate_text(&system, &prompt).await?;
        common::parse_explanation_response(&response)
    }

    /// Reformula un concepto de otra manera para reforzar la comprensión.
    ///
    /// # Parámetros
    /// - `concept`: Concepto a reformular.
    /// - `question`: Pregunta original.
    /// - `manual_prompt`: Contexto pedagogico adicional definido por el adulto.
    ///
    /// # Retorna
    /// Explicación reformulada en lenguaje sencillo.
    pub async fn reformulate_concept(&self, concept: &str, question: &LLMQuestion, manual_prompt: Option<&str>, locale: &str) -> Result<String, String> {
        let (system, prompt) = common::build_reformulation_prompt(concept, &question.question, manual_prompt, locale);
        self.generate_text(&system, &prompt).await
    }

    /// Envía una lista de mensajes a la API de Gemini y retorna la respuesta como texto.
    ///
    /// # Parámetros
    /// - `messages`: Lista de mensajes del chat (sistema, historial, usuario).
    ///
    /// # Retorna
    /// El texto generado por Gemini.
    pub async fn chat_completion(&self, messages: &[common::ChatMessage]) -> Result<String, String> {
        if self.api_key.is_empty() {
            return Err("No se ha configurado la API key de Gemini".to_string());
        }

        let client = Client::builder().timeout(self.timeout).build().map_err(|e| e.to_string())?;

        let full_text: String = messages.iter()
            .map(|m| format!("{}: {}", m.role, m.content))
            .collect::<Vec<_>>()
            .join("\n\n");

        let request = serde_json::json!({
            "contents": [{"parts": [{"text": full_text}]}],
            "generation_config": {"temperature": 0.3, "max_output_tokens": 2048}
        });

        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
            self.model, self.api_key
        );

        let json = common::chat_request(
            &client,
            &url,
            "",
            &request,
            "gemini",
            &self.model,
        ).await?;

        json["candidates"][0]["content"]["parts"][0]["text"]
            .as_str()
            .map(String::from)
            .ok_or_else(|| "Respuesta vacia de Gemini".to_string())
    }
}
