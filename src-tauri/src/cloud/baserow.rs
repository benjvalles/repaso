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

/// URL del proxy Cloudflare Worker para Baserow.
/// Se inyecta en compile time desde `.env` (variable PROXY_BASEROW_URL).
const PROXY_BASEROW: &str = env!("PROXY_BASEROW_URL");

/// Shared secret para autenticación contra el proxy.
/// Se inyecta en compile time desde `.env` (variable SHARED_SECRETS).
/// Se usa solo el primer valor de la lista (el más reciente).
const SHARED_SECRET: &str = env!("SHARED_SECRETS");

#[derive(Clone)]
pub struct BaserowClient {
    client: Client,
    user_id: Option<String>,
}

impl BaserowClient {
    /// Crea un nuevo cliente del proxy Baserow con timeout de 30 segundos por petición.
    pub fn new() -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(30))
                .build()
                .expect("No se pudo crear el cliente HTTP para el proxy"),
            user_id: None,
        }
    }

    /// Establece el user_id de la sesión para enviarlo como header `X-User-Id` al proxy.
    pub fn set_user_id(&mut self, id: String) {
        self.user_id = if id.is_empty() { None } else { Some(id) };
    }

    /// URL base `{proxy}/database/rows/table/{table_id}/` para la tabla dada.
    fn url(&self, table_id: i64) -> String {
        format!("{PROXY_BASEROW}/database/rows/table/{table_id}/")
    }

    /// URL completa `{proxy}/database/rows/table/{table_id}/{row_id}/` para la fila dada.
    fn url_row(&self, table_id: i64, row_id: i64) -> String {
        format!("{PROXY_BASEROW}/database/rows/table/{table_id}/{row_id}/")
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
                let mut req = self.client.get(self.url(table_id)).query(&p);
                if let Some(ref uid) = self.user_id {
                    req = req.header("X-User-Id", uid.as_str());
                }
                req = req.header("X-Proxy-Key", SHARED_SECRET);
                let resp = req.send()
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
            let mut req = self.client.post(self.url(table_id)).json(&fields);
            if let Some(ref uid) = self.user_id {
                req = req.header("X-User-Id", uid.as_str());
            }
            req = req.header("X-Proxy-Key", SHARED_SECRET);
            let resp = req.send()
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
            let mut req = self.client.patch(self.url_row(table_id, row_id)).json(&fields);
            if let Some(ref uid) = self.user_id {
                req = req.header("X-User-Id", uid.as_str());
            }
            req = req.header("X-Proxy-Key", SHARED_SECRET);
            let resp = req.send()
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
            let mut req = self.client.delete(self.url_row(table_id, row_id));
            if let Some(ref uid) = self.user_id {
                req = req.header("X-User-Id", uid.as_str());
            }
            req = req.header("X-Proxy-Key", SHARED_SECRET);
            let resp = req.send()
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
