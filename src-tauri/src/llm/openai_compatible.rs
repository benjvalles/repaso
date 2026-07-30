use super::{LLMQuestion, LLMExplanation};
use super::common;

use reqwest::Client;
use serde::Serialize;
use std::time::Duration;

#[derive(Debug, Serialize)]
struct OpenAIChatRequest {
    model: String,
    messages: Vec<common::ChatMessage>,
    temperature: f32,
    max_tokens: Option<u32>,
    stream: bool,
}

/// Proveedor LLM compatible con la API de OpenAI.
/// Funciona con LM Studio, OpenRouter, llama.cpp server, vLLM, etc.
#[derive(Debug, Clone)]
pub struct OpenAICompatibleProvider {
    base_url: String,
    api_key: String,
    model: String,
    timeout: Duration,
}

impl OpenAICompatibleProvider {
    /// Crea una nueva instancia del proveedor OpenAI-compatible.
    ///
    /// # Parámetros
    /// - `base_url`: URL base del servidor. Por defecto `http://localhost:1234`.
    /// - `api_key`: API key opcional. Algunos servidores locales no la requieren.
    /// - `model`: Nombre del modelo. Por defecto `llama3`.
    /// - `timeout_secs`: Timeout en segundos. Por defecto 60.
    pub fn new(base_url: Option<String>, api_key: Option<String>, model: Option<String>, timeout_secs: Option<u64>) -> Self {
        Self {
            base_url: base_url.unwrap_or_else(|| "http://localhost:1234".to_string()),
            api_key: api_key.unwrap_or_default(),
            model: model.unwrap_or_else(|| "llama3".to_string()),
            timeout: Duration::from_secs(timeout_secs.unwrap_or(60)),
        }
    }

    /// Envía un prompt al servidor OpenAI-compatible y retorna la respuesta como texto.
    ///
    /// # Parámetros
    /// - `system_prompt`: Instrucciones del sistema.
    /// - `user_prompt`: Pregunta o tarea específica.
    ///
    /// # Retorna
    /// El texto generado por el modelo.
    async fn generate_text(&self, system_prompt: &str, user_prompt: &str) -> Result<String, String> {
        let client = Client::builder().timeout(self.timeout).build().map_err(|e| e.to_string())?;

        let messages = vec![
            common::ChatMessage { role: "system".to_string(), content: system_prompt.to_string() },
            common::ChatMessage { role: "user".to_string(), content: user_prompt.to_string() },
        ];

        let request = OpenAIChatRequest {
            model: self.model.clone(),
            messages,
            temperature: 0.3,
            max_tokens: Some(2048),
            stream: false,
        };

        let url = format!("{}/v1/chat/completions", self.base_url);

        common::log_llm_prompt("openai-compatible", &self.model, system_prompt, user_prompt);

        let json = common::chat_request(
            &client,
            &url,
            &self.api_key,
            &request,
            "openai-compatible",
            &self.model,
        ).await?;

        json["choices"][0]["message"]["content"]
            .as_str()
            .map(String::from)
            .ok_or_else(|| "Respuesta vacia".to_string())
    }

    /// Genera una pregunta de matemáticas usando el proveedor OpenAI-compatible.
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
}
