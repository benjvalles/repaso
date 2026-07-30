/// Modulo de integracion con Baserow (nube): cliente, comandos y sincronizacion.
pub mod baserow;
/// Comandos Tauri para operaciones en la nube (registro, login, sync).
pub mod commands;
/// Logica de sincronizacion de datos locales con Baserow.
pub mod sync;

/// Reexport del cliente Baserow para uso externo.
pub use baserow::BaserowClient;

use serde::{Deserialize, Serialize};

/// Sesion activa en la nube, con datos del usuario autenticado.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudSession {
    pub user_id: String,
    pub user_name: String,
    pub email: String,
}

/// Estado de conexion con la nube, devuelto al frontend en `AppStatus`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudStatus {
    pub connected: bool,
    pub user_name: Option<String>,
    pub email: Option<String>,
    pub last_sync: Option<String>,
    pub auto_login: bool,
    pub email_verified: bool,
}
