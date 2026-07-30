use tauri::State;

use argon2::password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString};
use argon2::Argon2;
use chrono::Utc;

use rand_core::RngCore;

use crate::email::{Recipient, SendEmailRequest, Sender};
use crate::helpers::{get_setting, set_setting};
use crate::models::{
    CloudLoginRequest, RegisterRequest,
    CLOUD_AUTO_LOGIN_KEY, CLOUD_EMAIL_KEY, CLOUD_EMAIL_VERIFIED_KEY,
    CLOUD_LAST_SYNC_KEY, CLOUD_SESSION_KEY, CLOUD_USER_NAME_KEY,
    CLOUD_VERIFICATION_CODE_KEY,
};
use crate::AppState;

use super::sync::SyncResult;
use super::{CloudSession, CloudStatus};

/// ID de la tabla de cuentas en Baserow.
const BASEROW_TABLE_ACCOUNTS: i64 = 1071739;

/// Genera un codigo de verificacion numerico de 6 digitos.
fn generate_verification_code() -> String {
    let code: u32 = OsRng.next_u32() % 1_000_000;
    format!("{:06}", code)
}

/// Registra una nueva cuenta en la nube (Baserow) desde la aplicacion.
///
/// Valida los campos, verifica que el email no exista, hashea la contrasena con Argon2,
/// crea el registro en Baserow, guarda la sesion localmente y envia un codigo de
/// verificacion por email (best-effort).
///
/// # Argumentos
///
/// * `request` - Datos de registro: nombre, email, contrasena y consentimiento.
/// * `state` - Estado compartido de Tauri con acceso a base de datos, Baserow y email.
///
/// # Retorna
///
/// * `Ok(CloudSession)` con los datos de la sesion creada.
/// * `Err(String)` si falla la validacion, la creacion en Baserow o el acceso interno.
#[tauri::command]
pub async fn register_account(
    request: RegisterRequest,
    state: State<'_, AppState>,
) -> Result<CloudSession, String> {
    if request.name.len() < 2 || request.name.len() > 100 {
        return Err("El nombre debe tener entre 2 y 100 caracteres".to_string());
    }
    if !request.email.contains('@') {
        return Err("Email invalido".to_string());
    }
    if request.password.len() < 8 {
        return Err("La contrasena debe tener al menos 8 caracteres".to_string());
    }
    if !request.consent {
        return Err("Debes aceptar el consentimiento de privacidad".to_string());
    }

    let client = state
        .baserow_client
        .lock()
        .map_err(|_| "Error interno")?
        .clone()
        .ok_or("Baserow no esta configurado. Revisa el archivo .env")?;

    let exists = client
        .find_account_by_email(&request.email)
        .await
        .map_err(|e| format!("Error verificando email: {e}"))?;
    if exists.is_some() {
        return Err("El email ya esta registrado. Quieres iniciar sesion?".to_string());
    }

    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(request.password.as_bytes(), &salt)
        .map_err(|e| format!("Error hasheando contrasena: {e}"))?
        .to_string();

    let now = Utc::now().to_rfc3339();

    let created = client
        .create_row(
            BASEROW_TABLE_ACCOUNTS,
            serde_json::json!({
                "field_9480686": request.email,
                "field_9480701": request.name,
                "field_9480702": password_hash,
                "field_9480703": now,
                "field_9645672": false,
            }),
        )
        .await
        .map_err(|e| format!("Error creando cuenta en Baserow: {e}"))?;

    let user_id_str = created["id"]
        .as_i64()
        .ok_or("Error al obtener el id de la cuenta creada")?
        .to_string();
    let user_id = user_id_str.clone();

    let session = CloudSession {
        user_id,
        user_name: request.name,
        email: request.email,
    };

    let code = generate_verification_code();
    {
        let db = state.db.lock().map_err(|_| "No se pudo acceder a la base de datos")?;
        set_setting(&db, CLOUD_SESSION_KEY, &user_id_str)?;
        set_setting(&db, CLOUD_USER_NAME_KEY, &session.user_name)?;
        set_setting(&db, CLOUD_EMAIL_KEY, &session.email)?;
        set_setting(&db, CLOUD_AUTO_LOGIN_KEY, "true")?;
        set_setting(&db, CLOUD_LAST_SYNC_KEY, "")?;
        set_setting(&db, CLOUD_VERIFICATION_CODE_KEY, &code)?;
        set_setting(&db, CLOUD_EMAIL_VERIFIED_KEY, "false")?;
    }

    // Enviar email con el código (best-effort)
    let email_client = state.email_client.lock().ok().and_then(|g| g.as_ref().cloned());
    if let Some(client) = email_client {
        let email_body = SendEmailRequest {
            sender: Some(Sender {
                email: Some("noreply@brevosend.com".to_string()),
                name: Some("Mates".to_string()),
                id: None,
            }),
            to: Some(vec![Recipient {
                email: session.email.clone(),
                name: Some(session.user_name.clone()),
            }]),
            subject: Some("Tu código de verificación de Mates".to_string()),
            html_content: Some(format!(
                "<h2>Verifica tu cuenta</h2><p>Tu código de verificación es: <strong>{}</strong></p>",
                code
            )),
            text_content: None,
            template_id: None,
            cc: None,
            bcc: None,
            reply_to: None,
            attachment: None,
            headers: None,
            tags: Some(vec!["verification".to_string()]),
            params: None,
            scheduled_at: None,
            batch_id: None,
            message_versions: None,
        };
        if let Err(e) = client.send_transac_email(email_body).await {
            eprintln!("[mates] No se pudo enviar email de verificación: {e}");
        }
    }

    *state
        .cloud_session
        .lock()
        .map_err(|_| "Error interno")? = Some(session.clone());

    Ok(session)
}

/// Inicia sesion en la nube validando email y contrasena contra Baserow.
///
/// Verifica la contrasena con Argon2, recupera el nombre y el estado de verificacion
/// del email, guarda la sesion localmente y la retorna.
///
/// # Argumentos
///
/// * `request` - Credenciales: email y contrasena.
/// * `state` - Estado compartido de Tauri.
///
/// # Retorna
///
/// * `Ok(CloudSession)` con los datos de la sesion iniciada.
/// * `Err(String)` si las credenciales son invalidas o falla la conexion.
#[tauri::command]
pub async fn login_account(
    request: CloudLoginRequest,
    state: State<'_, AppState>,
) -> Result<CloudSession, String> {
    if !request.email.contains('@') {
        return Err("Email invalido".to_string());
    }
    if request.password.is_empty() {
        return Err("La contrasena no puede estar vacia".to_string());
    }

    let client = state
        .baserow_client
        .lock()
        .map_err(|_| "Error interno")?
        .clone()
        .ok_or("Baserow no esta configurado. Revisa el archivo .env")?;

    let account = client
        .find_account_by_email(&request.email)
        .await
        .map_err(|e| format!("Error buscando cuenta: {e}"))?
        .ok_or("No hay ninguna cuenta con este email".to_string())?;

    let stored_hash = account["field_9480702"]
        .as_str()
        .ok_or("Error leyendo datos de la cuenta")?;

    let parsed_hash = PasswordHash::new(stored_hash)
        .map_err(|e| format!("Error al verificar contrasena: {e}"))?;
    let argon2 = Argon2::default();
    argon2
        .verify_password(request.password.as_bytes(), &parsed_hash)
        .map_err(|_| "Email o contrasena incorrectos".to_string())?;

    let email_verified = account["field_9645672"].as_bool().unwrap_or(false);
    let name = account["field_9480701"].as_str().unwrap_or("Usuario");
    let user_id_str = account["id"]
        .as_i64()
        .ok_or("Error al obtener el id de la cuenta")?
        .to_string();
    let user_id = user_id_str.clone();

    let session = CloudSession {
        user_id,
        user_name: name.to_string(),
        email: request.email,
    };

    let db = state.db.lock().map_err(|_| "No se pudo acceder a la base de datos")?;
    set_setting(&db, CLOUD_SESSION_KEY, &user_id_str)?;
    set_setting(&db, CLOUD_USER_NAME_KEY, &session.user_name)?;
    set_setting(&db, CLOUD_EMAIL_KEY, &session.email)?;
    set_setting(&db, CLOUD_AUTO_LOGIN_KEY, "true")?;
    set_setting(&db, CLOUD_LAST_SYNC_KEY, "")?;
    set_setting(
        &db,
        CLOUD_EMAIL_VERIFIED_KEY,
        if email_verified { "true" } else { "false" },
    )?;
    drop(db);

    *state
        .cloud_session
        .lock()
        .map_err(|_| "Error interno")? = Some(session.clone());

    Ok(session)
}

/// Cierra la sesion de nube: limpia la configuracion local y el estado en memoria.
///
/// # Argumentos
///
/// * `state` - Estado compartido de Tauri.
///
/// # Retorna
///
/// * `Ok(())` si se cerro la sesion correctamente.
/// * `Err(String)` si fallo el acceso a la base de datos.
#[tauri::command]
pub fn logout_account(state: State<'_, AppState>) -> Result<(), String> {
    let db = state.db.lock().map_err(|_| "No se pudo acceder a la base de datos")?;
    set_setting(&db, CLOUD_SESSION_KEY, "")?;
    set_setting(&db, CLOUD_USER_NAME_KEY, "")?;
    set_setting(&db, CLOUD_EMAIL_KEY, "")?;
    set_setting(&db, CLOUD_LAST_SYNC_KEY, "")?;

    *state
        .cloud_session
        .lock()
        .map_err(|_| "Error interno")? = None;

    Ok(())
}

/// Sincroniza todos los datos locales con la nube (Baserow).
///
/// Requiere una sesion activa. Tras la sincronizacion actualiza la marca
/// de ultima sincronizacion en la configuracion local.
///
/// # Argumentos
///
/// * `state` - Estado compartido de Tauri.
///
/// # Retorna
///
/// * `Ok(SyncResult)` con el resumen de la sincronizacion.
/// * `Err(String)` si no hay sesion, Baserow no esta configurado o falla la sync.
#[tauri::command]
pub async fn sync_all_data(state: State<'_, AppState>) -> Result<SyncResult, String> {
    {
        let db = state.db.lock().map_err(|_| "No se pudo acceder a la base de datos")?;
        let email_verified = get_setting(&db, CLOUD_EMAIL_VERIFIED_KEY)?
            .unwrap_or_default() == "true";
        if !email_verified {
            return Err("Debes verificar tu email antes de sincronizar los datos".to_string());
        }
    }

    let client = state
        .baserow_client
        .lock()
        .map_err(|_| "Error interno")?
        .clone()
        .ok_or("Baserow no esta configurado")?;

    let session = state
        .cloud_session
        .lock()
        .map_err(|_| "Error interno")?
        .clone()
        .ok_or("No hay sesion activa. Inicia sesion primero")?;

    let result = super::sync::sync_all(&client, &state.db, &session.user_id).await;

    let now = Utc::now().to_rfc3339();
    {
        let db = state.db.lock().map_err(|_| "No se pudo acceder a la base de datos")?;
        set_setting(&db, CLOUD_LAST_SYNC_KEY, &now)?;
    }

    Ok(result)
}

/// Fuerza la sincronización desde la nube: sube datos locales que no existen
/// en remoto y luego sobrescribe todo lo local con los datos remotos.
#[tauri::command]
pub async fn force_sync_from_cloud(state: State<'_, AppState>) -> Result<SyncResult, String> {
    {
        let db = state.db.lock().map_err(|_| "No se pudo acceder a la base de datos")?;
        let email_verified = get_setting(&db, CLOUD_EMAIL_VERIFIED_KEY)?
            .unwrap_or_default() == "true";
        if !email_verified {
            return Err("Debes verificar tu email antes de sincronizar los datos".to_string());
        }
    }

    let client = state
        .baserow_client
        .lock()
        .map_err(|_| "Error interno")?
        .clone()
        .ok_or("Baserow no esta configurado")?;

    let session = state
        .cloud_session
        .lock()
        .map_err(|_| "Error interno")?
        .clone()
        .ok_or("No hay sesion activa. Inicia sesion primero")?;

    let result = super::sync::force_sync_all(&client, &state.db, &session.user_id).await;

    let now = Utc::now().to_rfc3339();
    {
        let db = state.db.lock().map_err(|_| "No se pudo acceder a la base de datos")?;
        set_setting(&db, CLOUD_LAST_SYNC_KEY, &now)?;
    }

    Ok(result)
}

/// Devuelve el estado actual de conexion con la nube.
///
/// Lee la sesion en memoria y las configuraciones locales de auto-login,
/// ultima sincronizacion y verificacion de email.
///
/// # Argumentos
///
/// * `state` - Estado compartido de Tauri.
///
/// # Retorna
///
/// * `Ok(CloudStatus)` con el estado de conexion.
/// * `Err(String)` si falla el acceso a la base de datos.
#[tauri::command]
pub fn get_cloud_status(state: State<'_, AppState>) -> Result<CloudStatus, String> {
    let db = state.db.lock().map_err(|_| "No se pudo acceder a la base de datos")?;
    let session = state
        .cloud_session
        .lock()
        .map_err(|_| "Error interno")?
        .clone();

    let baserow_ok = state
        .baserow_client
        .lock()
        .map_err(|_| "Error interno")?
        .is_some();

    let last_sync = get_setting(&db, CLOUD_LAST_SYNC_KEY)?;
    let auto_login = get_setting(&db, CLOUD_AUTO_LOGIN_KEY)?.unwrap_or_default();
    let email_verified =
        get_setting(&db, CLOUD_EMAIL_VERIFIED_KEY)?.unwrap_or_default() == "true";

    Ok(CloudStatus {
        connected: session.is_some() && baserow_ok,
        user_name: session.as_ref().map(|s| s.user_name.clone()),
        email: session.as_ref().map(|s| s.email.clone()),
        last_sync,
        auto_login: auto_login == "true",
        email_verified,
    })
}

/// Activa o desactiva el inicio de sesion automatico en la nube.
///
/// # Argumentos
///
/// * `enabled` - `true` para activar auto-login, `false` para desactivarlo.
/// * `state` - Estado compartido de Tauri.
///
/// # Retorna
///
/// * `Ok(())` si se actualizo la configuracion.
/// * `Err(String)` si falla el acceso a la base de datos.
#[tauri::command]
pub fn set_cloud_auto_login(enabled: bool, state: State<'_, AppState>) -> Result<(), String> {
    let db = state
        .db
        .lock()
        .map_err(|_| "No se pudo acceder a la base de datos")?;
    set_setting(&db, CLOUD_AUTO_LOGIN_KEY, if enabled { "true" } else { "false" })?;
    Ok(())
}

/// Verifica el codigo de verificacion enviado por email.
///
/// Compara el codigo recibido con el almacenado localmente. Si coincide,
/// marca el email como verificado tanto en local como en Baserow.
///
/// # Argumentos
///
/// * `code` - Codigo de verificacion de 6 digitos.
/// * `state` - Estado compartido de Tauri.
///
/// # Retorna
///
/// * `Ok(())` si el codigo es correcto y se verifico el email.
/// * `Err(String)` si el codigo es incorrecto, no hay codigo pendiente o falla Baserow.
#[tauri::command]
pub async fn verify_email_code(
    code: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    {
        let db = state.db.lock().map_err(|_| "No se pudo acceder a la base de datos")?;
        let stored = get_setting(&db, CLOUD_VERIFICATION_CODE_KEY)?
            .ok_or("No hay codigo de verificacion pendiente")?;
        if code.trim() != stored.trim() {
            return Err("Codigo incorrecto".to_string());
        }
        set_setting(&db, CLOUD_EMAIL_VERIFIED_KEY, "true")?;
        set_setting(&db, CLOUD_VERIFICATION_CODE_KEY, "")?;
    }

    let user_id = {
        let session_guard = state.cloud_session.lock().map_err(|_| "Error interno")?;
        session_guard
            .as_ref()
            .map(|s| s.user_id.clone())
            .ok_or("No hay sesion activa")?
    };

    let client = state
        .baserow_client
        .lock()
        .map_err(|_| "Error interno")?
        .clone()
        .ok_or("Baserow no esta configurado")?;
    let user_id_int: i64 = user_id.parse().map_err(|_| "Error interno")?;
    client
        .update_row(BASEROW_TABLE_ACCOUNTS, user_id_int, serde_json::json!({
            "field_9645672": true,
        }))
        .await
        .map_err(|e| format!("Error actualizando verificacion en Baserow: {e}"))?;

    Ok(())
}

/// Reenvia el codigo de verificacion al email asociado a la sesion.
///
/// Genera un nuevo codigo de 6 digitos, lo almacena localmente y lo envia
/// por email. Si el cliente de email no esta disponible, falla con error.
///
/// # Argumentos
///
/// * `state` - Estado compartido de Tauri.
///
/// # Retorna
///
/// * `Ok(())` si se reenvio el codigo correctamente.
/// * `Err(String)` si no hay email asociado, no hay cliente de email o falla el envio.
#[tauri::command]
pub async fn resend_verification_code(
    state: State<'_, AppState>,
) -> Result<(), String> {
    let code = generate_verification_code();
    let (email, name) = {
        let db = state.db.lock().map_err(|_| "No se pudo acceder a la base de datos")?;
        let email = get_setting(&db, CLOUD_EMAIL_KEY)?
            .ok_or("No hay email asociado a la sesion")?;
        let name = get_setting(&db, CLOUD_USER_NAME_KEY)?
            .unwrap_or_else(|| "Usuario".to_string());
        set_setting(&db, CLOUD_VERIFICATION_CODE_KEY, &code)?;
        (email, name)
    };

    let client = state.email_client.lock().ok().and_then(|g| g.as_ref().cloned())
        .ok_or("El cliente de email no esta disponible")?;
    let email_body = SendEmailRequest {
        sender: Some(Sender {
            email: Some("noreply@brevosend.com".to_string()),
            name: Some("Mates".to_string()),
            id: None,
        }),
        to: Some(vec![Recipient {
            email: email.clone(),
            name: Some(name.clone()),
        }]),
        subject: Some("Tu codigo de verificacion de Mates".to_string()),
        html_content: Some(format!(
            "<h2>Verifica tu cuenta</h2><p>Tu codigo de verificacion es: <strong>{}</strong></p>",
            code
        )),
        text_content: None,
        template_id: None,
        cc: None,
        bcc: None,
        reply_to: None,
        attachment: None,
        headers: None,
        tags: Some(vec!["verification".to_string()]),
        params: None,
        scheduled_at: None,
        batch_id: None,
        message_versions: None,
    };
    client.send_transac_email(email_body).await.map_err(|e| format!("No se pudo enviar el email: {e}"))?;

    Ok(())
}

/// Elimina la cuenta de nube: borra el registro en Baserow y limpia la sesion local.
///
/// # Argumentos
///
/// * `state` - Estado compartido de Tauri.
///
/// # Retorna
///
/// * `Ok(())` si se elimino la cuenta correctamente.
/// * `Err(String)` si no hay sesion, falla Baserow o falla el acceso a DB.
#[tauri::command]
pub async fn delete_cloud_account(state: State<'_, AppState>) -> Result<(), String> {
    let (_user_id, user_id_int) = {
        let db = state.db.lock().map_err(|_| "No se pudo acceder a la base de datos")?;
        let uid = get_setting(&db, CLOUD_SESSION_KEY)?
            .ok_or("No hay sesion activa")?;
        let uid_int: i64 = uid.parse().map_err(|_| "Error interno")?;
        (uid, uid_int)
    };

    let client = state
        .baserow_client
        .lock()
        .map_err(|_| "Error interno")?
        .clone()
        .ok_or("Baserow no esta configurado. Revisa el archivo .env")?;
    client
        .delete_row(BASEROW_TABLE_ACCOUNTS, user_id_int)
        .await
        .map_err(|e| format!("Error eliminando cuenta en Baserow: {e}"))?;

    let db = state.db.lock().map_err(|_| "No se pudo acceder a la base de datos")?;
    set_setting(&db, CLOUD_SESSION_KEY, "")?;
    set_setting(&db, CLOUD_USER_NAME_KEY, "")?;
    set_setting(&db, CLOUD_EMAIL_KEY, "")?;
    set_setting(&db, CLOUD_LAST_SYNC_KEY, "")?;
    set_setting(&db, CLOUD_VERIFICATION_CODE_KEY, "")?;
    set_setting(&db, CLOUD_EMAIL_VERIFIED_KEY, "")?;
    drop(db);

    *state
        .cloud_session
        .lock()
        .map_err(|_| "Error interno")? = None;

    Ok(())
}

/// Cambia el email de la cuenta en la nube.
///
/// Verifica que el nuevo email no exista ya en Baserow, lo actualiza,
/// genera un nuevo codigo de verificacion, lo envia por email y marca
/// el email como no verificado.
///
/// # Argumentos
///
/// * `new_email` - Nuevo correo electronico.
/// * `state` - Estado compartido de Tauri.
///
/// # Retorna
///
/// * `Ok(())` si se actualizo el email correctamente.
/// * `Err(String)` si el email ya existe, no hay sesion o falla la operacion.
#[tauri::command]
pub async fn change_cloud_email(
    new_email: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    if !new_email.contains('@') {
        return Err("Email invalido".to_string());
    }

    let client = state
        .baserow_client
        .lock()
        .map_err(|_| "Error interno")?
        .clone()
        .ok_or("Baserow no esta configurado. Revisa el archivo .env")?;

    let exists = client
        .find_account_by_email(&new_email)
        .await
        .map_err(|e| format!("Error verificando email: {e}"))?;
    if exists.is_some() {
        return Err("El email ya esta registrado".to_string());
    }

    let (user_id_str, _old_email, name) = {
        let db = state.db.lock().map_err(|_| "No se pudo acceder a la base de datos")?;
        let uid = get_setting(&db, CLOUD_SESSION_KEY)?
            .ok_or("No hay sesion activa")?;
        let old = get_setting(&db, CLOUD_EMAIL_KEY)?
            .ok_or("No hay email asociado a la sesion")?;
        let nm = get_setting(&db, CLOUD_USER_NAME_KEY)?
            .unwrap_or_else(|| "Usuario".to_string());
        (uid, old, nm)
    };

    let user_id_int: i64 = user_id_str.parse().map_err(|_| "Error interno")?;
    client
        .update_row(
            BASEROW_TABLE_ACCOUNTS,
            user_id_int,
            serde_json::json!({
                "field_9480686": new_email,
                "field_9645672": false,
            }),
        )
        .await
        .map_err(|e| format!("Error actualizando email en Baserow: {e}"))?;

    let code = generate_verification_code();
    {
        let db = state.db.lock().map_err(|_| "No se pudo acceder a la base de datos")?;
        set_setting(&db, CLOUD_EMAIL_KEY, &new_email)?;
        set_setting(&db, CLOUD_VERIFICATION_CODE_KEY, &code)?;
        set_setting(&db, CLOUD_EMAIL_VERIFIED_KEY, "false")?;
    }

    {
        let mut session = state.cloud_session.lock().map_err(|_| "Error interno")?;
        if let Some(ref mut s) = *session {
            s.email = new_email.clone();
        }
    }

    let email_client = state.email_client.lock().ok().and_then(|g| g.as_ref().cloned());
    if let Some(client) = email_client {
        let email_body = SendEmailRequest {
            sender: Some(Sender {
                email: Some("noreply@brevosend.com".to_string()),
                name: Some("Mates".to_string()),
                id: None,
            }),
            to: Some(vec![Recipient {
                email: new_email,
                name: Some(name),
            }]),
            subject: Some("Tu codigo de verificacion de Mates".to_string()),
            html_content: Some(format!(
                "<h2>Verifica tu nueva cuenta</h2><p>Tu codigo de verificacion es: <strong>{}</strong></p>",
                code
            )),
            text_content: None,
            template_id: None,
            cc: None,
            bcc: None,
            reply_to: None,
            attachment: None,
            headers: None,
            tags: Some(vec!["verification".to_string()]),
            params: None,
            scheduled_at: None,
            batch_id: None,
            message_versions: None,
        };
        if let Err(e) = client.send_transac_email(email_body).await {
            eprintln!("[mates] No se pudo enviar email de verificacion: {e}");
        }
    }

    Ok(())
}
