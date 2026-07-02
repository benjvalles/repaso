use super::{LLMQuestion, LLMExplanation};

use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Deserialize, Serialize)]
struct OllamaChatRequest {
    model: String,
    messages: Vec<OllamaMessage>,
    stream: bool,
}

#[derive(Debug, Deserialize, Serialize)]
struct OllamaMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct OllamaChatResponse {
    message: Option<OllamaMessage>,
    done: Option<bool>,
}

#[derive(Debug, Clone)]
pub struct OllamaProvider {
    base_url: String,
    model: String,
    timeout: Duration,
}

impl OllamaProvider {
    pub fn new(base_url: Option<String>, model: Option<String>, timeout_secs: Option<u64>) -> Self {
        Self {
            base_url: base_url.unwrap_or_else(|| "http://localhost:11434".to_string()),
            model: model.unwrap_or_else(|| "llama3".to_string()),
            timeout: Duration::from_secs(timeout_secs.unwrap_or(120)),
        }
    }

    pub async fn generate_text(&self, system_prompt: &str, user_prompt: &str) -> Result<String, String> {
        let client = Client::builder().timeout(self.timeout).build().map_err(|e| e.to_string())?;

        let request = OllamaChatRequest {
            model: self.model.clone(),
            messages: vec![
                OllamaMessage { role: "system".to_string(), content: system_prompt.to_string() },
                OllamaMessage { role: "user".to_string(), content: user_prompt.to_string() },
            ],
            stream: false,
        };

        let response = client.post(format!("{}/api/chat", self.base_url))
            .json(&request).send().await.map_err(|e| format!("Error de conexion: {e}"))?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(format!("Ollama error {status}: {text}"));
        }

        let body: OllamaChatResponse = response.json().await.map_err(|e| format!("Error de parseo: {e}"))?;
        body.message.map(|m| m.content).ok_or_else(|| "Respuesta vacia de Ollama".to_string())
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
            "Genera una pregunta de matematicas para un alumno de {year}o de primaria nivel {level}. Tema: {concept_text}.
            Responde SOLO con este JSON:
            {{\"question\":\"la pregunta\",\"correct_answer\":\"la respuesta\",\"concept\":\"concepto\",\"difficulty\":\"easy|medium|hard\"}}"
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
        let system = "Eres un tutor de matematicas para ninos. Responde SOLO con JSON valido. Usa lenguaje motivador y sencillo.";
        let prompt = format!(
            "Pregunta: {}\nRespuesta del alumno: {}\nRespuesta correcta: {}\nConcepto: {}
            Explica por que se equivoco y como resolverlo. Responde SOLO con JSON:
            {{\"explanation\":\"explicacion\",\"key_points\":[\"punto1\",\"punto2\"],\"next_steps\":[\"paso1\",\"paso2\"]}}",
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
        let system = "Eres un tutor de matematicas para ninos. Responde con una explicacion sencilla y motivadora.";
        let prompt = format!(
            "El concepto '{}' para la pregunta '{}' debe explicarse de otra manera. Usa una analogia o ejemplo diferente. Sé breve.",
            concept, question.question
        );
        self.generate_text(system, &prompt).await
    }
}
