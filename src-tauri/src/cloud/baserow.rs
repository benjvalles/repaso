use reqwest::Client;
use serde_json::Value;
use std::future::Future;
use std::time::Duration;

/// Reintenta una operación hasta 3 veces con 500ms de espera entre fallos.
async fn retry<F, Fut, T>(op: F) -> Result<T, String>
where
    F: Fn() -> Fut,
    Fut: Future<Output = Result<T, String>>,
{
    let mut last_err = String::new();
    for attempt in 1..=3 {
        match op().await {
            Ok(val) => return Ok(val),
            Err(e) => {
                last_err = e;
                if attempt < 3 {
                    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                }
            }
        }
    }
    Err(format!("{last_err} (tras 3 intentos)"))
}

const BASEROW_API_BASE: &str = "https://api.baserow.io/api/database";

#[derive(Clone)]
pub struct BaserowClient {
    client: Client,
    api_token: String,
}

impl BaserowClient {
    /// Crea un nuevo cliente Baserow con timeout de 30 segundos por petición.
    pub fn new(api_token: String) -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(30))
                .build()
                .expect("No se pudo crear el cliente HTTP para Baserow"),
            api_token,
        }
    }

    /// HeaderMap con autorización `Token {api_token}`.
    fn headers(&self) -> reqwest::header::HeaderMap {
        let mut h = reqwest::header::HeaderMap::new();
        let val = format!("Token {}", self.api_token);
        let header_val = reqwest::header::HeaderValue::from_str(&val).unwrap();
        h.insert(reqwest::header::AUTHORIZATION, header_val);
        h
    }

    /// URL base `/api/database/rows/table/{table_id}/` para la tabla dada.
    fn url(&self, table_id: i64) -> String {
        format!("{BASEROW_API_BASE}/rows/table/{table_id}/")
    }

    /// URL completa `/api/database/rows/table/{table_id}/{row_id}/` para la fila dada.
    fn url_row(&self, table_id: i64, row_id: i64) -> String {
        format!("{}/rows/table/{table_id}/{row_id}/", BASEROW_API_BASE)
    }

    /// Lista paginada de filas de una tabla Baserow (100 filas por página).
    ///
    /// Reintenta hasta 3 veces en caso de error de red, autenticación o límite de tasa.
    pub async fn list_rows(
        &self,
        table_id: i64,
        params: &[(&str, &str)],
    ) -> Result<Vec<Value>, String> {
        retry(|| async {
            let mut all = Vec::new();
            let mut page = 1i64;
            loop {
                let page_str = page.to_string();
                let mut p: Vec<(&str, &str)> = params.to_vec();
                p.push(("page", &page_str));
                p.push(("size", "100"));
                let resp = self.client.get(self.url(table_id))
                    .headers(self.headers())
                    .query(&p)
                    .send()
                    .await
                    .map_err(|e| {
                        if e.is_timeout() {
                            "Tiempo de espera agotado al conectar con Baserow. Revisa tu conexion a internet.".to_string()
                        } else {
                            format!("Error de conexion Baserow: {e}")
                        }
                    })?;
                if !resp.status().is_success() {
                    let status = resp.status().as_u16();
                    let text = resp.text().await.unwrap_or_default();
                    return Err(format!("Baserow error {status}: {text}"));
                }
                let json: Value = resp.json().await.map_err(|e| format!("Error parseando respuesta Baserow: {e}"))?;
                let results = json["results"].as_array().cloned().unwrap_or_default();
                let count = results.len();
                all.extend(results);
                if count < 100 {
                    break;
                }
                page += 1;
            }
            Ok(all)
        }).await
    }

    /// Crea una nueva fila en una tabla de Baserow.
    pub async fn create_row(&self, table_id: i64, fields: Value) -> Result<Value, String> {
        retry(|| async {
            let resp = self.client.post(self.url(table_id))
                .headers(self.headers())
                .json(&fields)
                .send()
                .await
                .map_err(|e| {
                    if e.is_timeout() {
                        "Tiempo de espera agotado al conectar con Baserow. Revisa tu conexion a internet.".to_string()
                    } else {
                        format!("Error de conexion Baserow: {e}")
                    }
                })?;
            if !resp.status().is_success() {
                let status = resp.status().as_u16();
                let text = resp.text().await.unwrap_or_default();
                return Err(format!("Baserow error {status}: {text}"));
            }
            resp.json().await.map_err(|e| format!("Error parseando respuesta Baserow: {e}"))
        }).await
    }

    /// Actualiza una fila de Baserow (actualización parcial). Solo cambia los campos proporcionados.
    pub async fn update_row(&self, table_id: i64, row_id: i64, fields: Value) -> Result<Value, String> {
        retry(|| async {
            let resp = self.client.patch(self.url_row(table_id, row_id))
                .headers(self.headers())
                .json(&fields)
                .send()
                .await
                .map_err(|e| {
                    if e.is_timeout() {
                        "Tiempo de espera agotado al conectar con Baserow. Revisa tu conexion a internet.".to_string()
                    } else {
                        format!("Error de conexion Baserow: {e}")
                    }
                })?;
            if !resp.status().is_success() {
                let status = resp.status().as_u16();
                let text = resp.text().await.unwrap_or_default();
                return Err(format!("Baserow error {status}: {text}"));
            }
            resp.json().await.map_err(|e| format!("Error parseando respuesta Baserow: {e}"))
        }).await
    }

    /// Elimina una fila de una tabla de Baserow.
    pub async fn delete_row(&self, table_id: i64, row_id: i64) -> Result<(), String> {
        retry(|| async {
            let resp = self.client
                .delete(self.url_row(table_id, row_id))
                .headers(self.headers())
                .send()
                .await
                .map_err(|e| {
                    if e.is_timeout() {
                        "Tiempo de espera agotado al conectar con Baserow. Revisa tu conexion a internet.".to_string()
                    } else {
                        format!("Error de conexion Baserow: {e}")
                    }
                })?;
            if !resp.status().is_success() {
                let status = resp.status().as_u16();
                let text = resp.text().await.unwrap_or_default();
                return Err(format!("Baserow error {status}: {text}"));
            }
            Ok(())
        }).await
    }

    /// Busca una cuenta de usuario por email (útil para login).
    pub async fn find_account_by_email(&self, email: &str) -> Result<Option<Value>, String> {
        let results = self.list_rows(1071739, &[("filter__field_9480686__equal", email)]).await?;
        Ok(results.into_iter().next())
    }
}
