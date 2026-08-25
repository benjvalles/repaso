pub mod commands;
pub mod common;
pub mod gemini;
pub mod ollama;
pub mod openai_compatible;

/// Representa una pregunta generada por el LLM.
#[derive(Debug, Clone)]
pub struct LLMQuestion {
    /// Texto de la pregunta matemática.
    pub question: String,
    /// Respuesta correcta esperada.
    pub correct_answer: String,
    /// Concepto matemático evaluado (ej: "suma con carry").
    pub concept: String,
    /// Nivel de dificultad: "easy", "medium" o "hard".
    pub difficulty: String,
}

/// Representa una explicación generada por el LLM para un error del estudiante.
#[derive(Debug, Clone)]
pub struct LLMExplanation {
    /// Explicación del error y cómo resolverlo.
    pub explanation: String,
    /// Puntos clave del concepto.
    pub key_points: Vec<String>,
    /// Siguientes pasos para mejorar.
    pub next_steps: Vec<String>,
}

/// Enum que dispatcha llamadas al proveedor LLM configurado.
/// Se usa enum en vez de trait object porque `async fn` no es dyn-compatible en Rust.
#[derive(Debug, Clone)]
pub enum LLMProviderEnum {
    /// Proveedor Ollama (local/offline).
    Ollama(ollama::OllamaProvider),
    /// Proveedor Google Gemini (cloud/API).
    Gemini(gemini::GeminiProvider),
    /// Proveedor compatible con API de OpenAI (LM Studio, OpenRouter, etc.).
    OpenAICompatible(openai_compatible::OpenAICompatibleProvider),
}

impl LLMProviderEnum {
    /// Genera una pregunta de matemáticas para un curso y nivel dado.
    ///
    /// # Parámetros
    /// - `year`: Curso del alumno (1-6 Primaria).
    /// - `level`: Nivel de dificultad actual del alumno.
    /// - `concept`: Concepto específico a evaluar. Si es `None`, se elige automáticamente.
    /// - `manual_prompt`: Contexto pedagogico adicional definido por el adulto.
    ///
    /// # Retorna
    /// `LLMQuestion` con la pregunta, respuesta correcta, concepto y dificultad.
    pub async fn generate_question(&self, year: u8, level: u8, concept: Option<String>, manual_prompt: Option<&str>, locale: &str) -> Result<LLMQuestion, String> {
        match self {
            Self::Ollama(p) => p.generate_question(year, level, concept, manual_prompt, locale).await,
            Self::Gemini(p) => p.generate_question(year, level, concept, manual_prompt, locale).await,
            Self::OpenAICompatible(p) => p.generate_question(year, level, concept, manual_prompt, locale).await,
        }
    }

    /// Genera una explicación motivadora cuando el estudiante falla una pregunta.
    ///
    /// # Parámetros
    /// - `question`: Pregunta que falló el alumno.
    /// - `student_answer`: Respuesta dada por el alumno.
    /// - `expected_answer`: Respuesta correcta.
    /// - `concept`: Concepto matemático evaluado.
    /// - `manual_prompt`: Contexto pedagogico adicional definido por el adulto.
    ///
    /// # Retorna
    /// `LLMExplanation` con la explicación, puntos clave y siguientes pasos.
    pub async fn provide_explanation(
        &self,
        question: &LLMQuestion,
        student_answer: &str,
        expected_answer: &str,
        concept: &str,
        manual_prompt: Option<&str>,
        locale: &str,
    ) -> Result<LLMExplanation, String> {
        match self {
            Self::Ollama(p) => p.provide_explanation(question, student_answer, expected_answer, concept, manual_prompt, locale).await,
            Self::Gemini(p) => p.provide_explanation(question, student_answer, expected_answer, concept, manual_prompt, locale).await,
            Self::OpenAICompatible(p) => p.provide_explanation(question, student_answer, expected_answer, concept, manual_prompt, locale).await,
        }
    }

    /// Reformula un concepto de otra manera para reforzar la comprensión.
    ///
    /// # Parámetros
    /// - `concept`: Concepto matemático a reformular.
    /// - `question`: Pregunta original que causó la dificultad.
    /// - `manual_prompt`: Contexto pedagogico adicional definido por el adulto.
    ///
    /// # Retorna
    /// `String` con la explicación reformulada en lenguaje sencillo.
    pub async fn reformulate_concept(&self, concept: &str, question: &LLMQuestion, manual_prompt: Option<&str>, locale: &str) -> Result<String, String> {
        match self {
            Self::Ollama(p) => p.reformulate_concept(concept, question, manual_prompt, locale).await,
            Self::Gemini(p) => p.reformulate_concept(concept, question, manual_prompt, locale).await,
            Self::OpenAICompatible(p) => p.reformulate_concept(concept, question, manual_prompt, locale).await,
        }
    }

    /// Envía una lista de mensajes al proveedor LLM y retorna la respuesta como texto.
    /// Usado para chat libre (no estructurado como las sesiones de práctica).
    ///
    /// # Parámetros
    /// - `messages`: Lista de mensajes del chat (sistema, historial, usuario).
    ///
    /// # Retorna
    /// Texto de respuesta del asistente.
    pub async fn chat_completion(&self, messages: &[common::ChatMessage]) -> Result<String, String> {
        match self {
            Self::Ollama(p) => p.chat_completion(messages).await,
            Self::Gemini(p) => p.chat_completion(messages).await,
            Self::OpenAICompatible(p) => p.chat_completion(messages).await,
        }
    }
}
