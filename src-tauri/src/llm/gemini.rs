use super::{LLMQuestion, LLMExplanation};

use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Deserialize, Serialize)]
struct GeminiRequest {
    contents: Vec<GeminiContent>,
    generation_config: Option<GeminiGenConfig>,
}

#[derive(Debug, Deserialize, Serialize)]
struct GeminiContent {
    parts: Vec<GeminiPart>,
}

#[derive(Debug, Deserialize, Serialize)]
struct GeminiPart {
    text: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct GeminiGenConfig {
    temperature: Option<f32>,
    max_output_tokens: Option<u32>,
}

#[derive(Debug, Deserialize)]
struct GeminiResponse {
    candidates: Vec<GeminiCandidate>,
}

#[derive(Debug, Deserialize)]
struct GeminiCandidate {
    content: GeminiContent,
}

#[derive(Debug, Clone)]
pub struct GeminiProvider {
    api_key: String,
    model: String,
    timeout: Duration,
}

impl GeminiProvider {
    pub fn new(api_key: Option<String>, model: Option<String>, timeout_secs: Option<u64>) -> Self {
        Self {
            api_key: api_key.unwrap_or_default(),
            model: model.unwrap_or_else(|| "gemini-1.5-flash".to_string()),
            timeout: Duration::from_secs(timeout_secs.unwrap_or(60)),
        }
    }

    async fn generate_text(&self, system_prompt: &str, user_prompt: &str) -> Result<String, String> {
        if self.api_key.is_empty() {
            return Err("No se ha configurado la API key de Gemini".to_string());
        }

        let client = Client::builder().timeout(self.timeout).build().map_err(|e| e.to_string())?;

        let full_text = format!("{system_prompt}\n\n{user_prompt}");
        let request = GeminiRequest {
            contents: vec![GeminiContent { parts: vec![GeminiPart { text: full_text }] }],
            generation_config: Some(GeminiGenConfig { temperature: Some(0.3), max_output_tokens: Some(1024) }),
        };

        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
            self.model, self.api_key
        );

        let response = client.post(&url).json(&request).send().await.map_err(|e| format!("Error de conexion: {e}"))?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let text = response.text().await.unwrap_or_default();
            return match status {
                401 | 403 => Err("API key de Gemini invalida".to_string()),
                429 => Err("Limite de peticiones de Gemini alcanzado".to_string()),
                _ => Err(format!("Gemini error {status}: {text}")),
            };
        }

        let body: GeminiResponse = response.json().await.map_err(|e| format!("Error de parseo: {e}"))?;
        body.candidates.first()
            .and_then(|c| c.content.parts.first())
            .map(|p| p.text.clone())
            .ok_or_else(|| "Respuesta vacia de Gemini".to_string())
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
