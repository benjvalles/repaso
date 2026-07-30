use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;

const BREVO_API_BASE: &str = "https://api.brevo.com/v3";

#[derive(Clone)]
pub struct EmailClient {
    client: Client,
    api_key: String,
}

impl EmailClient {
    pub fn from_env() -> Option<Self> {
        let api_key = env::var("BREVO_API_KEY").ok()?;
        if api_key.is_empty() {
            return None;
        }
        Some(Self {
            client: Client::new(),
            api_key,
        })
    }

    fn headers(&self) -> reqwest::header::HeaderMap {
        let mut h = reqwest::header::HeaderMap::new();
        h.insert(
            reqwest::header::HeaderName::from_static("api-key"),
            reqwest::header::HeaderValue::from_str(&self.api_key).unwrap(),
        );
        h.insert(
            reqwest::header::CONTENT_TYPE,
            reqwest::header::HeaderValue::from_static("application/json"),
        );
        h
    }

    /// POST /smtp/email — Enviar un email transaccional
    pub async fn send_transac_email(
        &self,
        body: SendEmailRequest,
    ) -> Result<SendEmailResponse, String> {
        let url = format!("{}/smtp/email", BREVO_API_BASE);
        let resp = self
            .client
            .post(&url)
            .headers(self.headers())
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("Error de conexion con Brevo: {e}"))?;
        let status = resp.status();
        if status.is_success() {
            resp.json::<SendEmailResponse>()
                .await
                .map_err(|e| format!("Error decodificando respuesta de Brevo: {e}"))
        } else {
            let text = resp
                .text()
                .await
                .unwrap_or_else(|_| "Respuesta vacia".to_string());
            Err(format_error("send_transac_email", status.as_u16(), &text))
        }
    }

    /// GET /smtp/emails — Listar emails enviados
    pub async fn list_transac_emails(
        &self,
        filters: EmailListFilters,
    ) -> Result<EmailListResponse, String> {
        let url = format!("{}/smtp/emails", BREVO_API_BASE);
        let resp = self
            .client
            .get(&url)
            .headers(self.headers())
            .query(&filters)
            .send()
            .await
            .map_err(|e| format!("Error de conexion con Brevo: {e}"))?;
        let status = resp.status();
        if status.is_success() {
            resp.json::<EmailListResponse>()
                .await
                .map_err(|e| format!("Error decodificando respuesta de Brevo: {e}"))
        } else {
            let text = resp
                .text()
                .await
                .unwrap_or_else(|_| "Respuesta vacia".to_string());
            Err(format_error("list_transac_emails", status.as_u16(), &text))
        }
    }

    /// GET /smtp/emails/{uuid} — Obtener contenido completo de un email enviado
    pub async fn get_email_content(&self, uuid: &str) -> Result<EmailContentResponse, String> {
        let url = format!("{}/smtp/emails/{}", BREVO_API_BASE, uuid);
        let resp = self
            .client
            .get(&url)
            .headers(self.headers())
            .send()
            .await
            .map_err(|e| format!("Error de conexion con Brevo: {e}"))?;
        let status = resp.status();
        if status.is_success() {
            resp.json::<EmailContentResponse>()
                .await
                .map_err(|e| format!("Error decodificando respuesta de Brevo: {e}"))
        } else {
            let text = resp
                .text()
                .await
                .unwrap_or_else(|_| "Respuesta vacia".to_string());
            Err(format_error("get_email_content", status.as_u16(), &text))
        }
    }

    /// GET /smtp/emailStatus/{identifier} — Consultar estado de envio programado
    pub async fn get_email_status(
        &self,
        identifier: &str,
        filters: StatusFilters,
    ) -> Result<EmailStatusResponse, String> {
        let url = format!("{}/smtp/emailStatus/{}", BREVO_API_BASE, identifier);
        let resp = self
            .client
            .get(&url)
            .headers(self.headers())
            .query(&filters)
            .send()
            .await
            .map_err(|e| format!("Error de conexion con Brevo: {e}"))?;
        let status = resp.status();
        if status.is_success() {
            resp.json::<EmailStatusResponse>()
                .await
                .map_err(|e| format!("Error decodificando respuesta de Brevo: {e}"))
        } else {
            let text = resp
                .text()
                .await
                .unwrap_or_else(|_| "Respuesta vacia".to_string());
            Err(format_error("get_email_status", status.as_u16(), &text))
        }
    }

    /// DELETE /smtp/email/{identifier} — Cancelar envio programado
    pub async fn delete_scheduled_email(&self, identifier: &str) -> Result<(), String> {
        let url = format!("{}/smtp/email/{}", BREVO_API_BASE, identifier);
        let resp = self
            .client
            .delete(&url)
            .headers(self.headers())
            .send()
            .await
            .map_err(|e| format!("Error de conexion con Brevo: {e}"))?;
        let status = resp.status();
        if status.is_success() {
            Ok(())
        } else {
            let text = resp
                .text()
                .await
                .unwrap_or_else(|_| "Respuesta vacia".to_string());
            Err(format_error(
                "delete_scheduled_email",
                status.as_u16(),
                &text,
            ))
        }
    }
}

fn format_error(operation: &str, status: u16, body: &str) -> String {
    match status {
        400 => format!("Solicitud invalida a Brevo ({operation}): {body}"),
        401 => "La API key de Brevo no es valida".to_string(),
        402 => "No hay creditos suficientes en Brevo para enviar el email".to_string(),
        404 => format!("Recurso no encontrado en Brevo ({operation}): {body}"),
        429 => "Demasiadas requests a Brevo. Intenta mas tarde".to_string(),
        _ => format!("Error en Brevo (HTTP {status}) en {operation}: {body}"),
    }
}

// ---------------------------------------------------------------------------
// Schemas de request
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SendEmailRequest {
    pub sender: Option<Sender>,
    pub to: Option<Vec<Recipient>>,
    pub subject: Option<String>,
    pub html_content: Option<String>,
    pub text_content: Option<String>,
    pub template_id: Option<i64>,
    pub cc: Option<Vec<Recipient>>,
    pub bcc: Option<Vec<Recipient>>,
    pub reply_to: Option<Recipient>,
    pub attachment: Option<Vec<Attachment>>,
    pub headers: Option<HashMap<String, String>>,
    pub tags: Option<Vec<String>>,
    pub params: Option<HashMap<String, String>>,
    pub scheduled_at: Option<String>,
    pub batch_id: Option<String>,
    pub message_versions: Option<Vec<MessageVersion>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Sender {
    pub email: Option<String>,
    pub name: Option<String>,
    pub id: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Recipient {
    pub email: String,
    pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Attachment {
    pub name: Option<String>,
    pub content: Option<String>,
    pub url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MessageVersion {
    pub to: Vec<Recipient>,
    pub params: Option<HashMap<String, String>>,
    pub subject: Option<String>,
    pub html_content: Option<String>,
    pub text_content: Option<String>,
    pub cc: Option<Vec<Recipient>>,
    pub bcc: Option<Vec<Recipient>>,
    pub reply_to: Option<Recipient>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EmailListFilters {
    pub email: Option<String>,
    #[serde(rename = "templateId")]
    pub template_id: Option<i64>,
    #[serde(rename = "messageId")]
    pub message_id: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub sort: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StatusFilters {
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub sort: Option<String>,
    pub status: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

// ---------------------------------------------------------------------------
// Schemas de response
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SendEmailResponse {
    pub message_id: Option<String>,
    pub message_ids: Option<Vec<String>>,
    pub batch_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailListResponse {
    pub count: i64,
    #[serde(rename = "transactionalEmails")]
    pub transactional_emails: Vec<EmailSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailSummary {
    pub date: String,
    pub email: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from: Option<String>,
    #[serde(rename = "messageId")]
    pub message_id: String,
    pub subject: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "templateId")]
    pub template_id: Option<i64>,
    pub uuid: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailContentResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
    pub date: String,
    pub email: String,
    pub events: Vec<EmailEvent>,
    pub subject: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "templateId")]
    pub template_id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "attachmentCount")]
    pub attachment_count: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailEvent {
    pub name: String,
    pub time: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum EmailStatusResponse {
    Batch {
        batches: Vec<BatchStatus>,
        count: i64,
    },
    Single {
        #[serde(rename = "createdAt")]
        created_at: String,
        #[serde(rename = "scheduledAt")]
        scheduled_at: String,
        status: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchStatus {
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "scheduledAt")]
    pub scheduled_at: String,
    pub status: String,
}

// ---------------------------------------------------------------------------
// Comandos Tauri
// ---------------------------------------------------------------------------

#[tauri::command]
pub async fn send_transac_email(
    request: SendEmailRequest,
    email_client: tauri::State<'_, crate::AppState>,
) -> Result<SendEmailResponse, String> {
    let client = email_client
        .email_client
        .lock()
        .map_err(|_| "Error interno del servidor".to_string())?
        .clone()
        .ok_or("BREVO_API_KEY no configurada. Revisa el archivo .env")?;
    client.send_transac_email(request).await
}

#[tauri::command]
pub async fn list_transac_emails(
    filters: EmailListFilters,
    email_client: tauri::State<'_, crate::AppState>,
) -> Result<EmailListResponse, String> {
    let client = email_client
        .email_client
        .lock()
        .map_err(|_| "Error interno del servidor".to_string())?
        .clone()
        .ok_or("BREVO_API_KEY no configurada. Revisa el archivo .env")?;
    client.list_transac_emails(filters).await
}

#[tauri::command]
pub async fn get_email_content(
    uuid: String,
    email_client: tauri::State<'_, crate::AppState>,
) -> Result<EmailContentResponse, String> {
    let client = email_client
        .email_client
        .lock()
        .map_err(|_| "Error interno del servidor".to_string())?
        .clone()
        .ok_or("BREVO_API_KEY no configurada. Revisa el archivo .env")?;
    client.get_email_content(&uuid).await
}

#[tauri::command]
pub async fn get_email_status(
    identifier: String,
    filters: StatusFilters,
    email_client: tauri::State<'_, crate::AppState>,
) -> Result<EmailStatusResponse, String> {
    let client = email_client
        .email_client
        .lock()
        .map_err(|_| "Error interno del servidor".to_string())?
        .clone()
        .ok_or("BREVO_API_KEY no configurada. Revisa el archivo .env")?;
    client.get_email_status(&identifier, filters).await
}

#[tauri::command]
pub async fn delete_scheduled_email(
    identifier: String,
    email_client: tauri::State<'_, crate::AppState>,
) -> Result<(), String> {
    let client = email_client
        .email_client
        .lock()
        .map_err(|_| "Error interno del servidor".to_string())?
        .clone()
        .ok_or("BREVO_API_KEY no configurada. Revisa el archivo .env")?;
    client.delete_scheduled_email(&identifier).await
}
