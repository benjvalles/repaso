use super::{LLMQuestion, LLMExplanation};
use reqwest::Client;
use serde::Serialize;

/// Convierte un código de locale ISO (como `es-ES`, `ca-ES`, `eu-ES`)
/// al nombre del idioma en español para usar en los prompts.
fn locale_to_language(locale: &str) -> &str {
    match &locale[..2] {
        "es" => "espanol",
        "ca" => "catalan",
        "eu" => "euskera",
        "gl" => "gallego",
        "en" => "ingles",
        _ => "espanol",
    }
}

/// Muestra en stderr el prompt exacto enviado al LLM.
///
/// # Parámetros
/// - `provider`: Nombre del proveedor LLM.
/// - `model`: Modelo configurado.
/// - `system_prompt`: Prompt de sistema enviado al proveedor.
/// - `user_prompt`: Prompt de usuario enviado al proveedor.
pub fn log_llm_prompt(provider: &str, model: &str, system_prompt: &str, user_prompt: &str) {
    eprintln!(
        "[LLM PROMPT][{provider}][{model}]\n--- system ---\n{system_prompt}\n--- user ---\n{user_prompt}\n--- end ---"
    );
}

/// Mensaje de chat compartido entre proveedores (formato OpenAI-compatible).
#[derive(Debug, Serialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

/// Muestra en stderr la respuesta cruda del servidor LLM.
///
/// # Parámetros
/// - `provider`: Nombre del proveedor.
/// - `model`: Modelo utilizado.
/// - `response`: Respuesta cruda del servidor.
pub fn log_llm_response(provider: &str, model: &str, response: &str) {
    let snippet = if response.len() > 2000 {
        format!("{}... (truncated, {} total chars)", &response[..2000], response.len())
    } else {
        response.to_string()
    };
    eprintln!("[LLM RESPONSE][{provider}][{model}]\n{snippet}\n--- end ---");
}

/// Elimina recursivamente claves `reasoning_content` de un `serde_json::Value`.
///
/// Algunos modelos (DeepSeek, etc.) incluyen cadenas de razonamiento enormes
/// en la respuesta que no nos interesan ni en logs ni en el parsing posterior.
fn strip_reasoning_content(value: &mut serde_json::Value) {
    match value {
        serde_json::Value::Object(map) => {
            map.remove("reasoning_content");
            for (_, v) in map.iter_mut() {
                strip_reasoning_content(v);
            }
        }
        serde_json::Value::Array(arr) => {
            for v in arr.iter_mut() {
                strip_reasoning_content(v);
            }
        }
        _ => {}
    }
}

/// Realiza la llamada HTTP a una API de chat LLM, registrando prompt y respuesta.
///
/// # Parámetros
/// - `client`: Cliente HTTP reutilizable.
/// - `url`: URL completa del endpoint.
/// - `api_key`: API key (se envía como Bearer si no está vacía).
/// - `body`: Cuerpo serializable de la petición.
/// - `provider`: Nombre del proveedor para logs.
/// - `model`: Nombre del modelo para logs.
///
/// # Retorna
/// El JSON completo de la respuesta del servidor (sin `reasoning_content`).
pub async fn chat_request(
    client: &Client,
    url: &str,
    api_key: &str,
    body: &impl Serialize,
    provider: &str,
    model: &str,
) -> Result<serde_json::Value, String> {
    let mut builder = client.post(url)
        .header("Content-Type", "application/json");
    if !api_key.is_empty() {
        builder = builder.header("Authorization", format!("Bearer {api_key}"));
    }
    let response = builder.json(body).send().await
        .map_err(|e| {
            if e.is_timeout() {
                format!("Tiempo de espera agotado al conectar con {provider}. Revisa tu conexion a internet o la configuracion del servidor.")
            } else {
                format!("Error de conexion: {e}")
            }
        })?;
    if !response.status().is_success() {
        let status = response.status().as_u16();
        let text = response.text().await.unwrap_or_default();
        return match status {
            401 | 403 => Err(format!("API key invalida ({provider})")),
            429 => Err(format!("Limite de peticiones alcanzado ({provider})")),
            _ => Err(format!("{provider} error {status}: {text}")),
        };
    }
    let mut json: serde_json::Value = response.json().await
        .map_err(|e| format!("Error de parseo: {e}"))?;
    strip_reasoning_content(&mut json);
    log_llm_response(provider, model, &json.to_string());
    Ok(json)
}

/// Limpia la respuesta del LLM eliminando code blocks markdown y parsea el JSON.
///
/// # Parámetros
/// - `response`: Texto crudo devuelto por el LLM (puede contener ` ```json `).
///
/// # Retorna
/// El objeto deserializado de tipo genérico `T`.
pub fn parse_json_response<T: serde::de::DeserializeOwned>(response: &str) -> Result<T, String> {
    if response.trim().is_empty() {
        return Err("Respuesta vacia del LLM".to_string());
    }
    let cleaned = response.trim();
    let cleaned = if let Some(s) = cleaned.strip_prefix("```json") {
        s.strip_suffix("```").unwrap_or(s).trim()
    } else if let Some(s) = cleaned.strip_prefix("```") {
        s.strip_suffix("```").unwrap_or(s).trim()
    } else {
        cleaned
    };
    serde_json::from_str(cleaned).map_err(|e| {
        let snippet = response.chars().take(300).collect::<String>();
        format!("Error JSON: {e}. Respuesta cruda (primeros 300 chars): {snippet:?}")
    })
}

/// Construye el prompt (sistema + usuario) para generar una pregunta de matemáticas.
///
/// # Parámetros
/// - `year`: Curso del alumno (1-6 Primaria).
/// - `level`: Nivel de dificultad.
/// - `concept`: Concepto específico. Si es `None`, se usa un tema genérico por curso/nivel.
/// - `manual_prompt`: Contexto pedagogico adicional definido por el adulto.
///
/// # Retorna
/// Tupla `(system_prompt, user_prompt)` lista para enviar al LLM.
pub fn build_question_prompt(year: u8, level: u8, concept: Option<String>, manual_prompt: Option<&str>, locale: &str) -> (String, String) {
    let manual_context = build_manual_context(manual_prompt, "Usalo para orientar el tipo de ejercicios, concepto y dificultad.");
    let language = locale_to_language(locale);
    let system = format!("Eres un tutor de matematicas para ninos. Responde SOLO con JSON valido. Responde siempre en {language}.");
    let mut prompt = if manual_context.is_empty() {
        let concept_text = concept.unwrap_or_else(|| format!("mate para {year}o primaria nivel {level}"));
        format!("Genera una pregunta de matematicas para {year}o primaria nivel {level}. Tema: {concept_text}.")
    } else {
        format!("Genera una pregunta de matematicas para {year}o primaria nivel {level}.")
    };
    if !manual_context.is_empty() {
        prompt.push_str(&manual_context);
    }
    prompt.push_str(
            "\n        Responde SOLO con este JSON:\n        {\"question\":\"la pregunta\",\"correct_answer\":\"la respuesta\",\"concept\":\"concepto\",\"difficulty\":\"easy|medium|hard\"}"
    );
    (system, prompt)
}

/// Parsea la respuesta del LLM en un `LLMQuestion`.
///
/// # Parámetros
/// - `response`: Texto JSON devuelto por el LLM.
///
/// # Retorna
/// `LLMQuestion` con pregunta, respuesta, concepto y dificultad.
pub fn parse_question_response(response: &str) -> Result<LLMQuestion, String> {
    let json: serde_json::Value = parse_json_response(response)?;
    Ok(LLMQuestion {
        question: json["question"].as_str().unwrap_or("Error").to_string(),
        correct_answer: json["correct_answer"].as_str().unwrap_or("0").to_string(),
        concept: json["concept"].as_str().unwrap_or("desconocido").to_string(),
        difficulty: json["difficulty"].as_str().unwrap_or("easy").to_string(),
    })
}

/// Construye el prompt (sistema + usuario) para generar una explicación de error.
///
/// # Parámetros
/// - `question_text`: Texto de la pregunta que falló.
/// - `student_answer`: Respuesta dada por el alumno.
/// - `expected_answer`: Respuesta correcta.
/// - `concept`: Concepto matemático evaluado.
/// - `manual_prompt`: Contexto pedagogico adicional definido por el adulto.
///
/// # Retorna
/// Tupla `(system_prompt, user_prompt)` lista para enviar al LLM.
pub fn build_explanation_prompt(question_text: &str, student_answer: &str, expected_answer: &str, concept: &str, manual_prompt: Option<&str>, locale: &str) -> (String, String) {
    let manual_context = build_manual_context(manual_prompt, "Usalo para adaptar la explicacion, consejos y siguientes pasos.");
    let language = locale_to_language(locale);
    let system = format!("Eres un tutor de matematicas para ninos. Responde SOLO con JSON valido. Usa lenguaje motivador y sencillo. Responde siempre en {language}.");
    let prompt = format!(
        "Pregunta: {}\nRespuesta del alumno: {}\nRespuesta correcta: {}\nConcepto: {}{manual_context}
        Explica por que se equivoco y como resolverlo. Responde SOLO con JSON:
        {{\"explanation\":\"explicacion\",\"key_points\":[\"punto1\",\"punto2\"],\"next_steps\":[\"paso1\",\"paso2\"]}}",
        question_text, student_answer, expected_answer, concept
    );
    (system, prompt)
}

/// Parsea la respuesta del LLM en un `LLMExplanation`.
///
/// # Parámetros
/// - `response`: Texto JSON devuelto por el LLM.
///
/// # Retorna
/// `LLMExplanation` con explicación, puntos clave y siguientes pasos.
pub fn parse_explanation_response(response: &str) -> Result<LLMExplanation, String> {
    let json: serde_json::Value = parse_json_response(response)?;
    Ok(LLMExplanation {
        explanation: json["explanation"].as_str().unwrap_or("Revisa el proceso paso a paso.").to_string(),
        key_points: json["key_points"].as_array().map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect()).unwrap_or_default(),
        next_steps: json["next_steps"].as_array().map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect()).unwrap_or_default(),
    })
}

/// Construye el prompt (sistema + usuario) para reformular un concepto.
///
/// # Parámetros
/// - `concept`: Concepto matemático a reformular.
/// - `question_text`: Pregunta original que causó la dificultad.
/// - `manual_prompt`: Contexto pedagogico adicional definido por el adulto.
///
/// # Retorna
/// Tupla `(system_prompt, user_prompt)` lista para enviar al LLM.
pub fn build_reformulation_prompt(concept: &str, question_text: &str, manual_prompt: Option<&str>, locale: &str) -> (String, String) {
    let manual_context = build_manual_context(manual_prompt, "Usalo para reformular el concepto de forma mas adaptada.");
    let language = locale_to_language(locale);
    let system = format!("Eres un tutor de matematicas para ninos. Responde con una explicacion sencilla y motivadora. Responde siempre en {language}.");
    let prompt = format!(
        "El concepto '{}' para la pregunta '{}' debe explicarse de otra manera.{manual_context} Usa una analogia o ejemplo diferente. Sé breve.",
        concept, question_text
    );
    (system, prompt)
}

/// Construye el bloque de contexto pedagogico adicional para un prompt.
///
/// # Parámetros
/// - `manual_prompt`: Contexto pedagogico escrito por el adulto.
/// - `instruction`: Instruccion concreta para el objetivo del prompt.
///
/// # Retorna
/// Texto listo para insertar en el prompt o cadena vacia si no hay contexto.
fn build_manual_context(manual_prompt: Option<&str>, instruction: &str) -> String {
    manual_prompt
        .map(str::trim)
        .filter(|prompt| !prompt.is_empty())
        .map(|prompt| format!("\nContexto pedagogico adicional del perfil:\n{prompt}\n{instruction}"))
        .unwrap_or_default()
}
