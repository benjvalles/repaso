pub mod gemini;
pub mod ollama;
pub mod openai_compatible;

#[derive(Debug, Clone)]
pub struct LLMQuestion {
    pub question: String,
    pub correct_answer: String,
    pub concept: String,
    pub difficulty: String,
}

#[derive(Debug, Clone)]
pub struct LLMExplanation {
    pub explanation: String,
    pub key_points: Vec<String>,
    pub next_steps: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum LLMProviderEnum {
    Ollama(ollama::OllamaProvider),
    Gemini(gemini::GeminiProvider),
    OpenAICompatible(openai_compatible::OpenAICompatibleProvider),
}

impl LLMProviderEnum {
    pub async fn generate_question(&self, year: u8, level: u8, concept: Option<String>) -> Result<LLMQuestion, String> {
        match self {
            Self::Ollama(p) => p.generate_question(year, level, concept).await,
            Self::Gemini(p) => p.generate_question(year, level, concept).await,
            Self::OpenAICompatible(p) => p.generate_question(year, level, concept).await,
        }
    }

    pub async fn provide_explanation(
        &self,
        question: &LLMQuestion,
        student_answer: &str,
        expected_answer: &str,
        concept: &str,
    ) -> Result<LLMExplanation, String> {
        match self {
            Self::Ollama(p) => p.provide_explanation(question, student_answer, expected_answer, concept).await,
            Self::Gemini(p) => p.provide_explanation(question, student_answer, expected_answer, concept).await,
            Self::OpenAICompatible(p) => p.provide_explanation(question, student_answer, expected_answer, concept).await,
        }
    }

    pub async fn reformulate_concept(&self, concept: &str, question: &LLMQuestion) -> Result<String, String> {
        match self {
            Self::Ollama(p) => p.reformulate_concept(concept, question).await,
            Self::Gemini(p) => p.reformulate_concept(concept, question).await,
            Self::OpenAICompatible(p) => p.reformulate_concept(concept, question).await,
        }
    }
}
