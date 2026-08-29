use super::{LLMQuestion, LLMExplanation};
use super::common;

use reqwest::Client;
use serde::Serialize;
use std::time::Duration;

#[derive(Debug, Serialize)]
struct OllamaChatRequest {
    model: String,
    messages: Vec<common::ChatMessage>,
    stream: bool,
}

/// Proveedor LLM local que se comunica con Ollama vía `/api/chat`.
#[derive(Debug, Clone)]
pub struct OllamaProvider {
    base_url: String,
    model: String,
    timeout: Duration,
}

impl OllamaProvider {
    /// Crea una nueva instancia del proveedor Ollama.
    ///
    /// # Parámetros
    /// - `base_url`: URL base de Ollama. Por defecto `http://localhost:11434`.
    /// - `model`: Nombre del modelo a usar. Por defecto `llama3`.
    /// - `timeout_secs`: Timeout en segundos. Por defecto 60.
    pub fn new(base_url: Option<String>, model: Option<String>, timeout_secs: Option<u64>) -> Self {
        Self {
            base_url: base_url.unwrap_or_else(|| "http://localhost:11434".to_string()),
            model: model.unwrap_or_else(|| "llama3".to_string()),
            timeout: Duration::from_secs(timeout_secs.unwrap_or(60)),
        }
    }

    /// Envía un prompt al servidor Ollama y retorna la respuesta como texto.
    ///
    /// # Parámetros
    /// - `system_prompt`: Instrucciones del sistema (rol del LLM).
    /// - `user_prompt`: Pregunta o tarea específica.
    ///
    /// # Retorna
    /// El texto generado por Ollama.
    async fn generate_text(&self, system_prompt: &str, user_prompt: &str) -> Result<String, String> {
        let client = Client::builder().timeout(self.timeout).build().map_err(|e| e.to_string())?;

        let request = OllamaChatRequest {
            model: self.model.clone(),
            messages: vec![
                common::ChatMessage { role: "system".to_string(), content: system_prompt.to_string() },
                common::ChatMessage { role: "user".to_string(), content: user_prompt.to_string() },
            ],
            stream: false,
        };

        let url = format!("{}/api/chat", self.base_url);

        common::log_llm_prompt("ollama", &self.model, system_prompt, user_prompt);

        let json = common::chat_request(
            &client,
            &url,
            "",
            &request,
            "ollama",
            &self.model,
        ).await?;

        json["message"]["content"]
            .as_str()
            .map(String::from)
            .ok_or_else(|| "Respuesta vacia de Ollama".to_string())
    }

    /// Genera una pregunta de matemáticas usando Ollama.
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

    /// Envía una lista de mensajes al servidor Ollama y retorna la respuesta como texto.
    ///
    /// # Parámetros
    /// - `messages`: Lista de mensajes del chat (sistema, historial, usuario).
    ///
    /// # Retorna
    /// El texto generado por Ollama.
    pub async fn chat_completion(&self, messages: &[common::ChatMessage]) -> Result<String, String> {
        let client = Client::builder().timeout(self.timeout).build().map_err(|e| e.to_string())?;

        let request = OllamaChatRequest {
            model: self.model.clone(),
            messages: messages.to_vec(),
            stream: false,
        };

        let url = format!("{}/api/chat", self.base_url);

        let json = common::chat_request(
            &client,
            &url,
            "",
            &request,
            "ollama",
            &self.model,
        ).await?;

        json["message"]["content"]
            .as_str()
            .map(String::from)
            .ok_or_else(|| "Respuesta vacia de Ollama".to_string())
    }
}
