use super::{LLMQuestion, LLMExplanation};

use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Serialize)]
struct OpenAIChatRequest {
    model: String,
    messages: Vec<OpenAIMessage>,
    temperature: f32,
    max_tokens: Option<u32>,
    stream: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenAIMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct OpenAIResponse {
    choices: Vec<OpenAIChoice>,
}

#[derive(Debug, Deserialize)]
struct OpenAIChoice {
    message: OpenAIMessage,
}

#[derive(Debug, Clone)]
pub struct OpenAICompatibleProvider {
    base_url: String,
    api_key: String,
    model: String,
    timeout: Duration,
}

impl OpenAICompatibleProvider {
    pub fn new(base_url: Option<String>, api_key: Option<String>, model: Option<String>, timeout_secs: Option<u64>) -> Self {
        Self {
            base_url: base_url.unwrap_or_else(|| "http://localhost:1234".to_string()),
            api_key: api_key.unwrap_or_default(),
            model: model.unwrap_or_else(|| "llama3".to_string()),
            timeout: Duration::from_secs(timeout_secs.unwrap_or(60)),
        }
    }

    async fn generate_text(&self, system_prompt: &str, user_prompt: &str) -> Result<String, String> {
        let client = Client::builder().timeout(self.timeout).build().map_err(|e| e.to_string())?;

        let messages = vec![
            OpenAIMessage { role: "system".to_string(), content: system_prompt.to_string() },
            OpenAIMessage { role: "user".to_string(), content: user_prompt.to_string() },
        ];

        let request = OpenAIChatRequest {
            model: self.model.clone(),
            messages,
            temperature: 0.3,
            max_tokens: Some(1024),
            stream: false,
        };

        let mut builder = client.post(format!("{}/v1/chat/completions", self.base_url))
            .header("Content-Type", "application/json");

        if !self.api_key.is_empty() {
            builder = builder.header("Authorization", format!("Bearer {}", self.api_key));
        }

        let response = builder.json(&request).send().await.map_err(|e| format!("Error de conexion: {e}"))?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let text = response.text().await.unwrap_or_default();
            return match status {
                401 | 403 => Err("API key invalida".to_string()),
                429 => Err("Limite de peticiones alcanzado".to_string()),
                _ => Err(format!("API error {status}: {text}")),
            };
        }

        let body: OpenAIResponse = response.json().await.map_err(|e| format!("Error de parseo: {e}"))?;
        body.choices.first()
            .map(|c| c.message.content.clone())
            .ok_or_else(|| "Respuesta vacia".to_string())
    }

    fn parse_json_response<T: serde::de::DeserializeOwned>(&self, response: &str) -> Result<T, String> {
        let cleaned = response.trim();
        let cleaned = if let Some(s) = cleaned.strip_prefix("```json") {
            s.strip_suffix("```").unwrap_or(s).trim()
        } else if let Some(s) = cleaned.strip_prefix("```") {
            s.strip_suffix("```").unwrap_or(s).trim()
        } else {
            cleaned
        };
        serde_json::from_str(cleaned).map_err(|e| format!("Error JSON: {e}"))
    }

    pub async fn generate_question(&self, year: u8, level: u8, concept: Option<String>) -> Result<LLMQuestion, String> {
        let concept_text = concept.unwrap_or_else(|| format!("mate para {year}o primaria nivel {level}"));
        let system = "Eres un tutor de matematicas para ninos. Responde SOLO con JSON valido.";
        let prompt = format!(
            "Genera una pregunta de matematicas para {year}o primaria nivel {level}. Tema: {concept_text}.
            JSON: {{\"question\":\"...\",\"correct_answer\":\"...\",\"concept\":\"...\",\"difficulty\":\"easy|medium|hard\"}}"
        );

        let response = self.generate_text(system, &prompt).await?;
        let json: serde_json::Value = self.parse_json_response(&response)?;

        Ok(LLMQuestion {
            question: json["question"].as_str().unwrap_or("Error").to_string(),
            correct_answer: json["correct_answer"].as_str().unwrap_or("0").to_string(),
            concept: json["concept"].as_str().unwrap_or("desconocido").to_string(),
            difficulty: json["difficulty"].as_str().unwrap_or("easy").to_string(),
        })
    }

    pub async fn provide_explanation(&self, question: &LLMQuestion, student_answer: &str, expected_answer: &str, concept: &str) -> Result<LLMExplanation, String> {
        let system = "Eres un tutor de matematicas para ninos. Responde SOLO con JSON valido. Lenguaje motivador.";
        let prompt = format!(
            "Pregunta: {}\nAlumno: {}\nCorrecta: {}\nConcepto: {}\nExplica el error. JSON: {{\"explanation\":\"...\",\"key_points\":[\"...\"],\"next_steps\":[\"...\"]}}",
            question.question, student_answer, expected_answer, concept
        );

        let response = self.generate_text(system, &prompt).await?;
        let json: serde_json::Value = self.parse_json_response(&response)?;

        Ok(LLMExplanation {
            explanation: json["explanation"].as_str().unwrap_or("Revisa el proceso paso a paso.").to_string(),
            key_points: json["key_points"].as_array().map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect()).unwrap_or_default(),
            next_steps: json["next_steps"].as_array().map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect()).unwrap_or_default(),
        })
    }

    pub async fn reformulate_concept(&self, concept: &str, question: &LLMQuestion) -> Result<String, String> {
        let system = "Eres un tutor de matematicas para ninos. Explica con una analogia sencilla.";
        let prompt = format!("Concepto '{}', pregunta '{}'. Explica de otra manera breve.", concept, question.question);
        self.generate_text(system, &prompt).await
    }
}
