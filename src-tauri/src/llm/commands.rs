use rusqlite::Connection;

use crate::helpers::get_setting;
use crate::llm::gemini::GeminiProvider;
use crate::llm::ollama::OllamaProvider;
use crate::llm::openai_compatible::OpenAICompatibleProvider;
use crate::llm::LLMProviderEnum;
use crate::models::{
    LLM_API_KEY_KEY, LLM_BASE_URL_KEY, LLMConfig, LLM_MODEL_KEY, LLM_PROVIDER_KEY,
};

/// Construye el proveedor LLM adecuado (Ollama, Gemini u OpenAI-compatible) segun la configuracion.
///
/// # Parámetros
/// - `config`: Configuracion LLM actual (`LLMConfig`).
///
/// # Retorna
/// Enum `LLMProviderEnum` con el proveedor configurado.
pub fn build_provider(config: &LLMConfig) -> LLMProviderEnum {
    match config.provider.as_str() {
        "gemini" => LLMProviderEnum::Gemini(GeminiProvider::new(
            Some(config.api_key.clone()),
            Some(config.model.clone()),
            None,
        )),
        "openai" => LLMProviderEnum::OpenAICompatible(OpenAICompatibleProvider::new(
            Some(config.base_url.clone()),
            Some(config.api_key.clone()),
            Some(config.model.clone()),
            None,
        )),
        _ => LLMProviderEnum::Ollama(OllamaProvider::new(
            Some(config.base_url.clone()),
            Some(config.model.clone()),
            None,
        )),
    }
}

/// Carga la configuracion LLM desde `app_settings`, con valores por defecto.
///
/// # Parámetros
/// - `db`: Conexion SQLite.
///
/// # Retorna
/// Configuracion LLM (`LLMConfig`).
pub fn load_llm_config(db: &Connection) -> LLMConfig {
    LLMConfig {
        provider: get_setting(db, LLM_PROVIDER_KEY)
            .ok()
            .flatten()
            .unwrap_or_else(|| "ollama".to_string()),
        model: get_setting(db, LLM_MODEL_KEY)
            .ok()
            .flatten()
            .unwrap_or_else(|| "llama3".to_string()),
        base_url: get_setting(db, LLM_BASE_URL_KEY)
            .ok()
            .flatten()
            .unwrap_or_else(|| "http://localhost:11434".to_string()),
        api_key: get_setting(db, LLM_API_KEY_KEY)
            .ok()
            .flatten()
            .unwrap_or_default(),
    }
}
