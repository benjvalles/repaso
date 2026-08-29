# Especificacion: Nube — Autenticacion, verificación, sincronización y email transaccional

## Resumen

El adulto/padre/tutor podra crear una cuenta o iniciar sesion usando nombre, email y
contraseña contra una tabla remota en Baserow.io. La cuenta de Baserow es propiedad del
desarrollador; las tablas ya estan creadas en su cuenta.

Al registrarse, se envia un codigo de verificacion de 6 digitos por email (via Brevo).
Hasta que el email no se verifique, la sincronizacion de datos permanece bloqueada.

Tras autenticarse y verificar el email, los datos locales (configuracion, perfiles
infantiles, sesiones) se sincronizan con el servidor usando "last-writer-wins" segun la
fecha de modificacion.

La sesion se persiste localmente con auto-login opcional para evitar introducir
credenciales en cada reinicio. El backend incluye un modulo de email transaccional
(Brevo) para enviar notificaciones al adulto.

Se añade una pestaña "Nube" dentro de la zona adulta para gestionar la cuenta, y un
indicador global de estado cloud en la barra superior de la app.

## Usuarios / Actores involucrados

- **Adulto (guardian/padre/tutor)**: persona que gestiona la app, sus perfiles infantiles,
  su configuración y destinatario de emails de notificacion.
- **Sistema Baserow.io**: backend remoto propiedad del desarrollador, con tablas
  pre-creadas que almacenan cuentas de usuario, configuraciones, perfiles y sesiones.
- **Sistema Brevo**: API externa (ex Sendinblue) que procesa el envio de correos
  transaccionales (codigos de verificacion y notificaciones).

## Funcionalidad

1. El adulto puede **crear una cuenta** desde la zona adulta proporcionando nombre,
   email y contraseña (8+ caracteres) mas consentimiento de privacidad.
2. Al registrarse, se genera un **codigo de verificacion de 6 digitos** que se envia
   al email del adulto via Brevo.
3. El adulto puede **verificar su email** introduciendo el codigo de 6 digitos.
4. El adulto puede **reenviar el codigo de verificacion** si no lo recibe.
5. El adulto puede **iniciar sesion** con email + contraseña si ya tiene cuenta.
6. El login refleja el estado de verificacion del email (`email_verified`) desde Baserow.
7. La sesion autenticada se persiste localmente y se puede **restaurar al reiniciar**
   la app mediante auto-login opcional, sin necesidad de introducir credenciales.
8. El adulto puede activar/desactivar el **interruptor de auto-login** en la pestaña Nube.
9. Tras autenticarse y verificar el email, todos los datos locales se sincronizan con el
   servidor (subida y bajada).
10. La sincronizacion usa "last modified wins": para cada registro, el que tenga
    `updated_at` mas reciente prevalece.
11. Los datos que se sincronizan son:
    - Configuracion LLM (`app_settings`)
    - Perfiles infantiles (`profiles`)
    - Sesiones y preguntas (`sessions`, `session_questions`)
12. La sincronizacion esta **bloqueada** hasta que el email este verificado.
13. Si hay conflictos en la primera sincronizacion (datos locales y remotos sin
    relacion previa), se fusionan sumando ambos conjuntos (no se descartan registros).
14. El adulto puede **cerrar sesion** en cualquier momento.
15. El adulto puede **eliminar su cuenta** en Baserow, lo que borra el registro y limpia
    la sesion local.
16. El adulto puede **cambiar su email**, lo que genera un nuevo codigo de verificacion.
17. El backend expone **5 operaciones de email transaccional** via Brevo: enviar,
    listar historial, obtener contenido, consultar estado programado y cancelar envio.
18. Existe un **indicador global** en la barra superior (icono de nube con punto de
    color: verde conectado, rojo validacion pendiente, gris desconectado).

## Criterios de aceptacion

- [x] Existe una pestaña "Nube" en la zona adulta (junto a Perfiles, IA/LLM,
      Historial, Dashboard, Profesional).
- [x] La pestaña "Nube" muestra un formulario de login (email + password) o registro
      (nombre + email + password + confirmar + consentimiento).
- [x] El registro valida que la contraseña tenga al menos 8 caracteres (frontend y
      backend).
- [x] El registro envia los datos a la tabla `accounts` de Baserow. El `id` es el
      row_id autoincremental que asigna Baserow (usado como `user_id`).
- [x] El login verifica email y contraseña contra la tabla `accounts` de Baserow usando
      Argon2.
- [x] Tras login/registro exitoso, se persisten localmente `user_id`, `user_name`,
      `email` y se activa auto-login implicitamente.
- [x] `register_account` envia codigo de 6 digitos y marca email como no verificado.
- [x] `login_account` lee `field_9645672` de Baserow y lo refleja en DB local.
- [x] `verify_email_code` valida codigo, marca verificado en local y actualiza Baserow.
- [x] `resend_verification_code` genera nuevo codigo y lo reenvia por email.
- [x] `sync_all_data` bloquea sincronizacion si `email_verified == false`.
- [x] `delete_cloud_account` borra registro en Baserow via `delete_row` y limpia sesion local.
- [x] `change_cloud_email` verifica duplicado, actualiza Baserow, genera nuevo codigo y lo envia.
- [x] Los datos sincronizados incluyen: `app_settings`, `profiles`, `sessions`,
      `session_questions`.
- [x] La sincronizacion respeta "last modified wins" comparando `updated_at` en todas
      las tablas (config, profiles, sessions, session_questions).
- [x] El usuario puede cerrar sesion desde la pestaña "Nube".
- [x] Existe un interruptor "Auto-login" en la pestaña Nube, visible solo cuando hay
      sesion activa.
- [x] Al activar/desactivar el interruptor, la preferencia se persiste en `app_settings`.
- [x] Al reiniciar la app con auto-login activado, la sesion se restaura sin pedir
      credenciales.
- [x] Al reiniciar la app con auto-login desactivado, la sesion no se restaura.
- [x] Al hacer login/registro, los datos de sesion (user_name, email) se persisten
      localmente y el auto-login se activa implicitamente.
- [x] Al cerrar sesion, los datos de sesion se limpian pero la preferencia de
      auto-login se conserva.
- [x] Al arrancar con auto-login, se ejecuta una sincronizacion en segundo plano
      (silenciosa si falla).
- [x] Indicador permanente de nube en la barra superior (SVG + punto verde/gris/rojo).
- [x] Al arrancar con auto-login, se muestra notice "Sesion de nube restaurada".
- [x] Las variables de configuracion (`PROXY_BASEROW_URL`, `PROXY_BREVO_URL`,
      `SHARED_SECRETS`) se inyectan en compile time desde `.env` via `build.rs`.
- [x] El backend tiene un comando `send_transac_email` que llama a `POST /smtp/email`.
- [x] El backend tiene un comando `list_transac_emails` que llama a `GET /smtp/emails`.
- [x] El backend tiene un comando `get_email_content` que llama a `GET /smtp/emails/{uuid}`.
- [x] El backend tiene un comando `get_email_status` que llama a `GET /smtp/emailStatus/{identifier}`.
- [x] El backend tiene un comando `delete_scheduled_email` que llama a `DELETE /smtp/email/{identifier}`.
- [x] Todos los comandos Tauri manejan errores de red, timeouts y respuestas HTTP
      no exitosas (400, 404, 500) devolviendo un `Result<..., String>`.
- [x] Los comandos se registran en el `invoke_handler` de Tauri.
- [x] `CloudStatus` expone `email_verified: bool` y `auto_login: bool` al frontend.
- [x] `get_app_status` y `get_cloud_status` incluyen `email_verified` y `auto_login`.
- [ ] Compatibilidad hacia atras: usuarios existentes no pierden datos ni tienen
      errores al actualizar.
- [ ] Si la red no esta disponible, la app sigue funcionando con datos locales pero
      no muestra un indicador de "sin conexion".

## Reglas de negocio

1. **Contraseña**: minimo 8 caracteres, sin limite maximo explicito (se recomienda 128).
2. **Email**: unico por cuenta en la tabla `accounts`. Si el email ya existe, el
   registro se rechaza con mensaje claro.
3. **Nombre**: obligatorio, entre 2 y 100 caracteres.
4. **Privacidad**: los datos de los niños (perfiles, sesiones) viajan a Baserow.
   El adulto debe aceptar explicitamente (consentimiento) al registrar o conectar la cuenta.
   Los correos de los adultos destinatarios viajan a Brevo para su procesamiento.
5. **Sin conexion**: la app nunca debe bloquearse por falta de red. Las operaciones
   locales continuan normalmente. La sincronizacion falla silenciosamente.
6. **Sesion**: el `user_id` (row_id numerico de Baserow) se guarda en `app_settings`
   con clave `cloud_session_user_id`. No se genera ningun token UUID.
7. **Cuenta Baserow del desarrollador**: las credenciales de Baserow (Database Token)
   viven exclusivamente en el Cloudflare Worker proxy como secrets. La app nunca las conoce.
8. **API Key Brevo**: vive exclusivamente en el Cloudflare Worker proxy como secret.
   La app nunca la conoce.
9. **Endpoint base Brevo**: `https://api.brevo.com/v3` (configurado en el proxy).
   La app usa `PROXY_BREVO_URL` que apunta al proxy, no directamente a Brevo.
10. **Autenticacion Brevo**: header `api-key: <valor>` inyectado por el proxy.
11. **Rate limit**: Brevo permite ~300 requests/minuto. Sin rate limiting local en
    esta fase; se confia en el manejo de errores HTTP 429.
12. **Idempotencia**: el campo `Idempotency-Key` en `headers` permite evitar envios
    duplicados. No es obligatorio.
13. **Sin almacenamiento local de emails**: no se guardan logs de envio en SQLite en
    esta fase. El historial se consulta via API.
14. **Costos Brevo**: el plan de Brevo determina cuantos emails se pueden enviar.
    No hay control local de creditos.
15. **Persistencia local de sesion**: los datos de sesion se guardan en `app_settings`
    con las claves `cloud_session_user_id`, `cloud_session_user_name`,
    `cloud_session_email`, `cloud_auto_login`, `cloud_last_sync`.
16. **Restauracion de sesion**: solo se restaura si `cloud_auto_login = "true"` Y
    `cloud_session_user_id` tiene valor no vacio.
17. **Cierre de sesion**: al cerrar sesion se limpian los datos de sesion pero se
    conserva la preferencia `cloud_auto_login`.
18. **Verificacion de email**: la sincronizacion de datos (`sync_all_data`) esta
    bloqueada hasta que `email_verified == true`.

## Flujos principales

### 6.1 Registro con verificacion de email

1. El adulto abre la zona adulta y selecciona la pestaña "Nube".
2. No hay sesion activa, se muestra el formulario de registro/login.
3. El adulto selecciona "Crear cuenta".
4. Introduce nombre, email y contraseña (8+ caracteres). Confirma la contraseña.
5. Marca el checkbox de consentimiento de privacidad.
6. El frontend valida: contraseña >= 8 caracteres, contraseñas coinciden, email
   con formato valido.
7. El backend calcula el hash Argon2 de la contraseña localmente (nunca se envia
   en texto plano a Baserow, solo el hash).
8. El backend envia POST a Baserow para crear un registro en `accounts` con:
   `id` (autoincremental), `email`, `name`, `password_hash`, `created_at`,
   `field_9645672: false` (email no verificado).
9. Si el email ya existe, Baserow devuelve error y se muestra "El email ya esta
   registrado. Quieres iniciar sesion?".
10. Si es exitoso, se usa el `id` del registro creado (row_id numerico) como
    `user_id` y se guarda en `app_settings` (`cloud_session_user_id`,
    `cloud_session_user_name`, `cloud_session_email`).
11. Se activa auto-login implicitamente: `cloud_auto_login = "true"`.
12. Se genera un codigo de verificacion de 6 digitos (`OsRng.next_u32() % 1_000_000`
    formateado como `{:06}`).
13. El codigo se almacena en DB local con clave `CLOUD_VERIFICATION_CODE_KEY`.
14. Se marca `CLOUD_EMAIL_VERIFIED_KEY = "false"` en DB local.
15. Se envia un email transaccional via Brevo con el codigo (best-effort; si falla
    el envio, el registro continua y el usuario puede usar `resend_verification_code`).
16. Se muestra "Cuenta creada. Te hemos enviado un codigo de verificacion a tu email."
    con un campo para introducir el codigo de 6 digitos.
17. La sincronizacion de datos queda bloqueada hasta verificar el email.

### 6.2 Verificacion de email

1. El usuario ingresa el codigo de 6 digitos en la UI.
2. `verify_email_code` compara el codigo ingresado contra `CLOUD_VERIFICATION_CODE_KEY`
   en DB local.
3. Si coincide:
   - Marca `CLOUD_EMAIL_VERIFIED_KEY = "true"` en DB local.
   - Limpia `CLOUD_VERIFICATION_CODE_KEY` en DB local.
   - Actualiza `field_9645672: true` en Baserow via `update_row` (PATCH).
   - Se muestra "Email verificado correctamente".
   - Se habilita la sincronizacion.
4. Si no coincide, devuelve error "Codigo incorrecto".
5. El usuario puede solicitar `resend_verification_code` para recibir un nuevo codigo.

### 6.3 Login

1. El adulto introduce email y contraseña.
2. El backend consulta la tabla `accounts` en Baserow filtrando por email.
3. Si no encuentra el email: "No hay ninguna cuenta con este email".
4. Si encuentra, verifica la contraseña contra el hash almacenado usando Argon2.
5. Lee `field_9645672` de la respuesta JSON de Baserow.
6. Almacena `CLOUD_EMAIL_VERIFIED_KEY = "true"/"false"` en DB local segun el valor.
7. Si es correcto, usa el `id` del registro (row_id numerico) como `user_id` y lo
   guarda en `app_settings` (`cloud_session_user_id`, `cloud_session_user_name`,
   `cloud_session_email`), activando auto-login implicitamente.
8. Si el email NO esta verificado, se muestra el formulario de verificacion de codigo.
9. Si el email esta verificado, se ejecuta la sincronizacion (subida y bajada segun
   corresponda).
10. Se muestra "Sesion iniciada" con contadores de sincronizacion.

### 6.4 Auto-login

#### Activacion

1. El adulto abre la zona adulta y selecciona la pestaña "Nube".
2. Ya esta conectado (sesion activa).
3. Ve un interruptor "Auto-login al iniciar" debajo de los datos de conexion.
4. Activa el interruptor.
5. El frontend llama al comando Tauri `set_cloud_auto_login(true)`.
6. El backend persiste `cloud_auto_login = "true"` en `app_settings`.

#### Arranque con auto-login

1. El usuario abre la app.
2. En `setup()` (Rust), tras abrir la base de datos:
   - Se lee `cloud_auto_login` de `app_settings`.
   - Si es `"true"`, se leen `cloud_session_user_id`, `cloud_session_user_name`,
     `cloud_session_email`.
   - Si `cloud_session_user_id` tiene valor, se crea un `CloudSession` en memoria.
3. En el frontend, `App.svelte.onMount`:
   - Llama a `await refreshStatus()`.
   - `refreshStatus()` actualiza `cloudStatus` desde la respuesta de `get_app_status`.
   - Si `cloudStatus.connected && cloudStatus.auto_login` son true:
     - Se muestra notice "Sesion de nube restaurada".
     - Se dispara `invoke<SyncResult>("sync_all_data")` sin `await` (silenciosa si falla).
4. El usuario ve el indicador de nube con punto verde en la barra superior desde el
   primer momento, sin haber introducido credenciales.

#### Cierre de sesion

1. El adulto hace click en "Cerrar sesion" en la pestaña Nube.
2. `logout_account()` se ejecuta:
   - Limpia `cloud_session_user_id`, `cloud_session_user_name`,
     `cloud_session_email` y `cloud_last_sync` en `app_settings`.
   - Pone `cloud_session` a `None`.
   - NO toca `cloud_auto_login` (la preferencia se conserva).
3. El frontend muestra el formulario de login/registro.

### 6.5 Sincronizacion

1. `sync_all_data` verifica `CLOUD_EMAIL_VERIFIED_KEY` antes de sincronizar.
2. Si `email_verified == false`, devuelve error: "Debes verificar tu email antes de
   sincronizar los datos".
3. Para cada tipo de dato se ejecuta una fase de lectura local, una fase de subida
   remota y una fase de escritura local, sin retener el Mutex de la DB a traves de
   llamadas `.await`.
4. **Config (`app_settings`)**: sincronizacion bidireccional con
    last-modified-wins. Solo se insertan configs remotas que no existen localmente
    (cada dispositivo es origen de su propia configuracion LLM/nube).
 5. **Profiles**: sincronizacion bidireccional con last-modified-wins (`updated_at`).
    - Si el local es mas reciente -> se sube al servidor (PATCH).
    - Si el remoto no existe localmente y no esta borrado -> se crea en local (INSERT).
    - Si el remoto es mas reciente (`updated_at`) -> se actualizan todos los campos
      en local (display_name, school_year, nivel, etc.) y se marca como activo
      (`deleted_at = NULL`), recuperandolo si estaba borrado localmente.
    - **Borrados (soft-delete)**: la propagacion de eliminaciones y recuperaciones
      tambien usa last-modified-wins. Se compara `local.updated_at` contra
      `rem.updated_at` (o `rem.deleted_at` en sesiones/preguntas) para decidir
      que direccion prevalece. Si el local se recupero mas recientemente, la
      recuperacion se sube al remoto; si la eliminacion remota es mas reciente,
      se replica localmente.
 6. **Sessions** y **session_questions**: sincronizacion bidireccional.
    - Si el local no existe en remoto -> se crea alla (POST).
    - Si el local existe en remoto y esta borrado localmente -> se actualiza (PATCH)
      con `deleted_at`.
    - Si el remoto no existe localmente y no esta borrado -> se crea en local (INSERT).
      Esto permite que al recuperar un perfil en un dispositivo, las sesiones y
      preguntas de ese perfil aparezcan en los demas dispositivos tras la sincronizacion.
    - Si el remoto esta borrado y el local no -> se replica el borrado localmente.
    - Si ambas versiones existen y estan activas -> gana la version con `updated_at`
      mas reciente (last-writer-wins). Al subir, se usa `now` como `updated_at`
      remoto para que la proxima sincronizacion reconozca el cambio.
    - Las sesiones y preguntas se insertan aunque el perfil/sesion asociado no exista
      localmente, para evitar que un error de sincronizacion previo las descarte.
  7. Se registra un resumen de lo sincronizado (contadores por tipo).
  8. Tras la sincronizacion, el frontend refresca el estado completo (`refreshStatus`)
     para reflejar los datos descargados en la UI (perfiles, sesiones, configuracion).
  9. Existe un modo **"Forzar sincronizacion desde la nube"** (`force_sync_from_cloud`)
     que difiere del normal en:
     - **Upload**: solo crea datos locales que no existen en remoto (nunca actualiza).
     - **Download**: sobrescribe todo lo local con datos remotos incondicionalmente
       (sin comparar `updated_at`). Esto permite recuperar un dispositivo desde cero
       a partir de los datos de la nube.

### 6.6 API Email (Brevo)

Se exponen 5 operaciones contra la API REST de Brevo (`https://api.brevo.com/v3`):

| # | Operacion | Metodo | Endpoint |
|---|-----------|--------|----------|
| 1 | `send_transac_email` | POST | `/smtp/email` |
| 2 | `list_transac_emails` | GET | `/smtp/emails` |
| 3 | `get_email_content` | GET | `/smtp/emails/{uuid}` |
| 4 | `get_email_status` | GET | `/smtp/emailStatus/{identifier}` |
| 5 | `delete_scheduled_email` | DELETE | `/smtp/email/{identifier}` |

#### POST /smtp/email — Enviar email transaccional

**Request body** (camelCase a traves de `#[serde(rename_all = "camelCase")]`):

```json
{
  "sender": { "name": "Mates App", "email": "no-reply@mates.app" },
  "to": [{ "email": "adulto@ejemplo.com", "name": "Nombre del adulto" }],
  "subject": "Progreso semanal de {child_name}",
  "htmlContent": "<html><body><p>{{params.message}}</p></body></html>",
  "textContent": "Progreso semanal: {{params.message}}",
  "cc": [{ "email": "otro@ejemplo.com", "name": "Otro adulto" }],
  "bcc": [{ "email": "copia@ejemplo.com", "name": "Copia oculta" }],
  "replyTo": { "email": "soporte@mates.app", "name": "Soporte Mates" },
  "attachment": [{ "name": "reporte.pdf", "url": "https://cdn.mates.app/reportes/123.pdf" }],
  "headers": { "X-Mates-Campaign": "weekly_report" },
  "tags": ["progreso", "semanal", "mates"],
  "params": { "message": "Juan ha completado 15 ejercicios esta semana" },
  "templateId": 5,
  "scheduledAt": "2026-07-20T10:00:00.000Z",
  "batchId": "5c6cfa04-eed9-42c2-8b5c-6d470d978e9d",
  "messageVersions": [
    {
      "to": [{ "email": "adulto1@ejemplo.com", "name": "Adulto 1" }],
      "params": { "message": "Progreso individualizado 1" },
      "subject": "Progreso de {child1}"
    }
  ]
}
```

**Response 201** (enviado inmediatamente):
```json
{ "messageId": "<201798300811.5787683@relay.domain.com>", "messageIds": [] }
```

**Response 202** (programado):
```json
{ "batchId": "5c6cfa04-...", "messageId": "<201798300811.5787683@relay.domain.com>", "messageIds": [] }
```

#### GET /smtp/emails — Listar emails enviados

**Query params**: `email`, `templateId`, `messageId`, `startDate`, `endDate`, `sort`, `limit`, `offset`.

**Response 200**:
```json
{ "count": 120, "transactionalEmails": [{ "date": "...", "email": "abc@xyz.com", "from": "no-reply@mates.app", "messageId": "<...>", "subject": "Progreso semanal", "tags": ["progreso"], "templateId": 15, "uuid": "5a78c-..." }] }
```

#### GET /smtp/emails/{uuid} — Obtener contenido de un email

**Response 200**:
```json
{ "body": "<html>...", "date": "2016-02-25T11:53:26Z", "email": "adulto@ejemplo.com", "events": [{ "name": "sent", "time": "..." }, { "name": "delivered", "time": "..." }], "subject": "Progreso semanal", "templateId": 12, "attachmentCount": 2 }
```

#### GET /smtp/emailStatus/{identifier} — Consultar estado de envio programado

`identifier` puede ser `batchId` (UUIDv4) o `messageId` (`<...@domain>`).

**Response batch**:
```json
{ "batches": [{ "createdAt": "...", "scheduledAt": "...", "status": "queued" }], "count": 3 }
```

**Response individual**:
```json
{ "createdAt": "...", "scheduledAt": "...", "status": "queued" }
```

#### DELETE /smtp/email/{identifier} — Cancelar envio programado

**Response**: `204 No Content`.

### 6.7 Gestion de cuenta

#### Eliminar cuenta

1. El usuario confirma la eliminacion desde la UI.
2. `delete_cloud_account` lee `CLOUD_SESSION_KEY` de la DB local para obtener `user_id`.
3. Llama a `BaserowClient.delete_row(BASEROW_TABLE_ACCOUNTS, user_id_int)`.
4. Limpia todas las claves de sesion en DB local:
   - `CLOUD_SESSION_KEY`, `CLOUD_USER_NAME_KEY`, `CLOUD_EMAIL_KEY`
   - `CLOUD_LAST_SYNC_KEY`, `CLOUD_VERIFICATION_CODE_KEY`, `CLOUD_EMAIL_VERIFIED_KEY`
5. Pone `cloud_session` en memoria a `None`.

#### Cambiar email

1. El usuario ingresa un nuevo email en la UI.
2. `change_cloud_email` valida que el email contenga `@`.
3. Verifica que el nuevo email no exista ya en Baserow via `find_account_by_email`.
4. Obtiene `user_id`, email antiguo y nombre de la DB local.
5. Actualiza Baserow con `update_row` seteando `field_9480686` (email) y
   `field_9645672: false` (email no verificado).
6. Genera un nuevo codigo de verificacion y lo guarda en DB local con
   `CLOUD_VERIFICATION_CODE_KEY`.
7. Marca `CLOUD_EMAIL_VERIFIED_KEY = "false"` en DB local.
8. Actualiza la sesion en memoria con el nuevo email.
9. Envia el codigo por email via Brevo (best-effort; si falla se loguea el error
   pero el cambio de email ya se completo).

## Flujos alternativos / errores

- **Email ya registrado**: "Ya existe una cuenta con este email. Quieres iniciar sesion?"
  con enlace a login.
- **Credenciales incorrectas**: "Email o contraseña incorrectos".
- **Red no disponible**: la sincronizacion falla silenciosamente; la app funciona
  localmente. No hay indicador visual de "Sin conexion".
- **Baserow no responde (timeout, 500)**: "El servidor no esta disponible. Intenta
  mas tarde". Sin reintentos automaticos.
- **Sincronizacion parcial**: si un lote falla, se informa al usuario sin bloquear
  la app (los errores se acumulan en `SyncResult.errors`).
- **BREVO_API_KEY no configurada**: los comandos de email devuelven "BREVO_API_KEY
  no configurada en el archivo .env".
- **Red no disponible (Brevo)**: "Error de conexion con el servidor de email".
- **400 Bad Request (Brevo)**: parametros invalidos. Se devuelve el mensaje de error.
- **401 Unauthorized (Brevo)**: "La API key de Brevo no es valida".
- **402 Not enough credit (Brevo)**: "No hay creditos suficientes en Brevo".
- **404 Not found (Brevo)**: el UUID o identifier no existe.
- **429 Too Many Requests (Brevo)**: "Demasiadas requests a Brevo. Intenta mas tarde".
- **500/502/503 (Brevo)**: "El servidor de email no esta disponible".
- **Timeout (Brevo)**: 30 segundos por request. "La request a Brevo excedio el tiempo
  de espera".
- **Scheduled email cancelado**: si el email ya fue enviado, DELETE devuelve 404.
- **Sin red al arrancar (auto-login)**: la sesion se restaura localmente (usuario ve
  "Conectado"). La sincronizacion en segundo plano falla silenciosamente.
- **Usuario borra datos locales (RESET)**: todas las claves de sesion se borran,
  incluyendo `cloud_auto_login`. Al reiniciar, no hay sesion ni auto-login.
- **Auto-login activado pero sin user_id en BD**: no se restaura sesion. El
  interruptor de auto-login queda en ON pero sin efecto hasta que el usuario haga login.
- **Migracion desde version anterior**: usuarios con `cloud_session_user_id` existente
  pero sin `cloud_session_user_name`/`cloud_session_email` ni `cloud_auto_login` veran
  el auto-login desactivado por defecto. Al hacer login manualmente, se persistiran
  todos los datos.

## Consideraciones tecnicas

### 8.1 Tablas en Baserow (creadas por el desarrollador)

| # | Tabla (ID) | ID Baserow | Campos |
|---|------------|------------|--------|
| 1 | `accounts` | 1071739 | `id` (autonumerico), `field_9480686` (email, texto), `field_9480701` (name, texto), `field_9480702` (password_hash, texto, Argon2), `field_9480703` (created_at, texto, RFC 3339), `field_9645672` (email_verified, boolean, false por defecto) |
| 2 | `user_config` | 1071740 | `id` (autonumerico), `field_9480689` (user_id, numero), `field_9480705` (key, texto), `field_9480706` (value, texto), `field_9480707` (updated_at, texto) |
| 3 | `user_profiles` | 1071741 | `id` (autonumerico), `field_9480692` (profile_id, texto), `field_9480708` (user_id, numero), `field_9480709` (display_name, texto), `field_9480710` (school_year, numero), `field_9480711` (age, numero), `field_9480712` (level_mode, texto), `field_9480713` (current_level, numero), `field_9480714` (manual_prompt, texto_largo), `field_9480715` (created_at, texto), `field_9480716` (updated_at, texto), `field_9679183` (deleted_at, texto, null si activo) |
| 4 | `user_sessions` | 1071742 | `id` (autonumerico), `field_9480695` (session_id, texto), `field_9480717` (user_id, numero), `field_9480718` (profile_id, texto), `field_9480719` (status, texto), `field_9480720` (total_questions, numero), `field_9480721` (questions_answered, numero), `field_9480722` (correct_count, numero), `field_9480723` (current_question_index, numero), `field_9480724` (started_at, texto), `field_9480725` (ended_at, texto), `field_9679263` (updated_at, texto), `field_9679269` (deleted_at, texto, null si activo) |
| 5 | `user_session_questions` | 1071743 | `id` (autonumerico), `field_9480698` (question_id, texto), `field_9480726` (user_id, numero), `field_9480727` (session_id, texto), `field_9480728` (question_text, texto), `field_9480729` (correct_answer, texto), `field_9480730` (student_answer, texto), `field_9480731` (concept, texto), `field_9480732` (difficulty, texto), `field_9480733` (is_correct, boolean), `field_9480734` (explanation, texto), `field_9480735` (question_number, numero), `field_9480736` (time_spent_secs, numero), `field_9480737` (created_at, texto), `field_9480738` (answered_at, texto), `field_9679275` (updated_at, texto), `field_9679279` (deleted_at, texto, null si activo) |

### 8.2 Backend (Rust)

#### Modulos

| Archivo | Proposito |
|---------|-----------|
| `src-tauri/src/cloud/mod.rs` | Reexportacion, `CloudSession`, `CloudStatus` |
| `src-tauri/src/cloud/baserow.rs` | Cliente HTTP para la API REST de Baserow via proxy |
| `src-tauri/src/cloud/commands.rs` | Comandos Tauri para operaciones cloud |
| `src-tauri/src/cloud/sync.rs` | Logica de sincronizacion |
| `src-tauri/src/email.rs` | Modulo completo: `EmailClient`, schemas, 5 comandos Tauri |

#### Configuracion

Todas las URLs y secretos se inyectan en compile time desde `.env` via `build.rs`:

| Variable `.env` | Compile time | Valor local (pruebas) | Valor produccion |
|-----------------|--------------|----------------------|------------------|
| `PROXY_BASEROW_URL` | `env!("PROXY_BASEROW_URL")` | `http://localhost:8787/baserow` | `https://baserow-proxy.baserow-proxy.workers.dev/baserow` |
| `PROXY_BREVO_URL` | `env!("PROXY_BREVO_URL")` | `http://localhost:8787/brevo` | `https://baserow-proxy.baserow-proxy.workers.dev/brevo` |
| `SHARED_SECRETS` | `env!("SHARED_SECRETS")` | `v1_<hex>` | `v1_<hex>` (mismo valor) |

- `build.rs` lee `.env` y exporta las variables como `cargo:rustc-env`.
- `baserow.rs` usa `const PROXY_BASEROW: &str = env!("PROXY_BASEROW_URL")`.
- `email.rs` usa `const PROXY_BREVO: &str = env!("PROXY_BREVO_URL")`.
- `baserow.rs` usa `const SHARED_SECRET: &str = env!("SHARED_SECRETS")` (primer valor de la lista).
- `BASEROW_API_TOKEN` y `BREVO_API_KEY` viven exclusivamente en el Cloudflare Worker proxy;
  la app nunca los conoce.

#### Autenticacion del proxy (3 capas)

```
Capa 1: X-Proxy-Key  -> valida contra SHARED_SECRETS en el Worker
Capa 2: X-User-Id    -> valida que el user_id existe en tabla de cuentas (1071739)
Capa 3: Token         -> inyectado por el Worker (BASEROW_API_TOKEN / BREVO_API_KEY)
```

- `BaserowClient` envia `X-Proxy-Key` (shared secret) y `X-User-Id` (sesion activa) en
  cada peticion a Baserow.
- `find_account_by_email` no envia `X-User-Id` (se usa en login/registro antes de tener sesion).
- El proxy permite lecturas (GET/HEAD) de la tabla de cuentas sin `X-User-Id` para
  soportar login y registro.
- `SHARED_SECRETS` acepta una lista separada por comas para rotacion de secretos
  (backward compatibility con versiones anteriores de la app).

#### AppState

```rust
struct AppState {
    db: Mutex<Connection>,
    adult_unlocked: Mutex<bool>,
    llm_provider: Mutex<Option<Box<dyn LlmProvider>>>,
    llm_config: Mutex<LlmConfig>,
    locale: Mutex<String>,
    baserow_client: Mutex<Option<BaserowClient>>,
    cloud_session: Mutex<Option<CloudSession>>,
    email_client: Mutex<Option<EmailClient>>,
}
```

Inicializacion en `setup()`:
- `baserow_client` se crea via `BaserowClient::new()`. Si hay auto-login restaurado,
  se establece el `user_id` para las siguientes peticiones al proxy.
- `cloud_session` se crea leyendo `app_settings` (posible restauracion de auto-login).
- `email_client` se crea via `EmailClient::new()`.

### 8.3 Comandos Tauri (todos `async fn`)

```rust
// Cloud account
register_account(request: RegisterRequest) -> Result<CloudSession, String>
login_account(request: CloudLoginRequest) -> Result<CloudSession, String>
logout_account() -> Result<(), String>

// Sync
sync_all_data() -> Result<SyncResult, String>
force_sync_from_cloud() -> Result<SyncResult, String>
get_cloud_status() -> Result<CloudStatus, String>

// Auto-login
set_cloud_auto_login(enabled: bool) -> Result<(), String>

// Email verification
verify_email_code(code: String) -> Result<(), String>
resend_verification_code() -> Result<(), String>

// Account management
delete_cloud_account() -> Result<(), String>
change_cloud_email(new_email: String) -> Result<(), String>

// Brevo transactional email
send_transac_email(request: SendEmailRequest) -> Result<SendEmailResponse, String>
list_transac_emails(filters: EmailListFilters) -> Result<EmailListResponse, String>
get_email_content(uuid: String) -> Result<EmailContentResponse, String>
get_email_status(identifier: String, filters: StatusFilters) -> Result<EmailStatusResponse, String>
delete_scheduled_email(identifier: String) -> Result<(), String>
```

### 8.4 Constantes

```rust
// Sesion
pub const CLOUD_SESSION_KEY: &str = "cloud_session_user_id";
pub const CLOUD_USER_NAME_KEY: &str = "cloud_session_user_name";
pub const CLOUD_EMAIL_KEY: &str = "cloud_session_email";
pub const CLOUD_AUTO_LOGIN_KEY: &str = "cloud_auto_login";
pub const CLOUD_LAST_SYNC_KEY: &str = "cloud_last_sync";

// Verificacion email
pub const CLOUD_VERIFICATION_CODE_KEY: &str = "cloud_verification_code";
pub const CLOUD_EMAIL_VERIFIED_KEY: &str = "cloud_email_verified";

// Proxy (compile time desde .env via build.rs)
// const PROXY_BASEROW: &str = env!("PROXY_BASEROW_URL");
// const PROXY_BREVO: &str = env!("PROXY_BREVO_URL");
// const SHARED_SECRET: &str = env!("SHARED_SECRETS");
```

### 8.5 CloudStatus (Rust y TypeScript)

```rust
pub struct CloudStatus {
    pub connected: bool,
    pub user_name: Option<String>,
    pub email: Option<String>,
    pub last_sync: Option<String>,
    pub auto_login: bool,
    pub email_verified: bool,
}
```

```typescript
type CloudStatus = {
  connected: boolean
  user_name: string | null
  email: string | null
  last_sync: string | null
  auto_login: boolean
  email_verified: boolean
}
```

### 8.6 Schemas Rust (email.rs)

```rust
struct SendEmailRequest {
    sender: Sender,
    to: Vec<Recipient>,
    subject: Option<String>,
    html_content: Option<String>,
    text_content: Option<String>,
    template_id: Option<i64>,
    cc: Option<Vec<Recipient>>,
    bcc: Option<Vec<Recipient>>,
    reply_to: Option<Recipient>,
    attachment: Option<Vec<Attachment>>,
    headers: Option<HashMap<String, String>>,
    tags: Option<Vec<String>>,
    params: Option<HashMap<String, String>>,
    scheduled_at: Option<String>,
    batch_id: Option<String>,
    message_versions: Option<Vec<MessageVersion>>,
}

struct Sender { email: Option<String>, name: Option<String>, id: Option<i64> }
struct Recipient { email: String, name: Option<String> }
struct Attachment { name: Option<String>, content: Option<String>, url: Option<String> }
struct MessageVersion { to: Vec<Recipient>, params: Option<HashMap<String, String>>, subject: Option<String>, html_content: Option<String>, text_content: Option<String>, cc: Option<Vec<Recipient>>, bcc: Option<Vec<Recipient>>, reply_to: Option<Recipient> }
struct SendEmailResponse { message_id: Option<String>, message_ids: Option<Vec<String>>, batch_id: Option<String> }
struct EmailListFilters { email: Option<String>, template_id: Option<i64>, message_id: Option<String>, start_date: Option<String>, end_date: Option<String>, sort: Option<String>, limit: Option<i64>, offset: Option<i64> }
struct EmailListResponse { count: i64, transactional_emails: Vec<EmailSummary> }
struct EmailSummary { date: String, email: String, from: Option<String>, message_id: String, subject: String, tags: Option<Vec<String>>, template_id: Option<i64>, uuid: String }
struct EmailContentResponse { body: Option<String>, date: String, email: String, events: Vec<EmailEvent>, subject: String, template_id: Option<i64>, attachment_count: Option<i64> }
struct EmailEvent { name: String, time: String }
struct StatusFilters { start_date: Option<String>, end_date: Option<String>, sort: Option<String>, status: Option<String>, limit: Option<i64>, offset: Option<i64> }

#[serde(untagged)]
enum EmailStatusResponse {
    Batch { batches: Vec<BatchStatus>, count: i64 },
    Single { created_at: String, scheduled_at: String, status: String },
}
struct BatchStatus { created_at: String, scheduled_at: String, status: String }
```

Todos los structs con `#[derive(Debug, Clone, Serialize, Deserialize)]` y
`#[serde(rename_all = "camelCase")]` para serializar a camelCase JSON.

### 8.7 Mapeo de errores HTTP (Brevo)

Funcion `format_error(operation, status, body)` en `email.rs`:

| Codigo | Mensaje |
|--------|---------|
| 400 | "Solicitud invalida a Brevo ({operation}): {body}" |
| 401 | "La API key de Brevo no es valida" |
| 402 | "No hay creditos suficientes en Brevo para enviar el email" |
| 404 | "Recurso no encontrado en Brevo ({operation}): {body}" |
| 429 | "Demasiadas requests a Brevo. Intenta mas tarde" |
| otros | "Error en Brevo (HTTP {status}) en {operation}: {body}" |

### 8.8 Frontend

#### Componentes

- Nueva pestaña "Nube" en `AdultPanelView.svelte` (boton entre Dashboard y Profesional).
- Nuevo componente `CloudTab.svelte`.
- Tipos actualizados en `types.ts` (`CloudStatus` con `auto_login` y `email_verified`).
- Estado actualizado en `app-state.svelte.ts` (`cloudStatus`, `setAutoLogin()`).
- `App.svelte`: indicador global de nube en barra superior.

#### Indicador global de estado cloud

En la barra superior de la app (`App.svelte`), icono de nube con punto de estado:

| Estado | Punto | Texto tooltip |
|--------|-------|---------------|
| Desconectado | Gris (`#aaa`) | "Nube: Desconectado" |
| Conectado + email verificado | Verde (`#2ecc71`) | "Nube: Conectado ({email})" |
| Conectado + email no verificado | Rojo (`#e74c3c`) | "Nube: Validacion pendiente ({email})" |

El punto se actualiza reactivamente via `cs.email_verified` derivado de
`appState.cloudStatus`.

#### Sincronización en segundo plano al arrancar

```typescript
// En App.svelte onMount
if (appState.cloudStatus.connected && appState.cloudStatus.auto_login) {
  appState.notice = "Sesion de nube restaurada"
  invoke<SyncResult>("sync_all_data")
    .then(() => { appState.loadCloudStatus(); appState.refreshStatus() })
    .catch(() => {})
}
```

## Mockups / UI

### Pestaña Nube — Sin sesion

```
+-----------------------------------+
|  Nube / Sincronizacion            |
|                                   |
|  +- Iniciar sesion -------------+ |
|  | Email:    [_____________]    | |
|  | Password: [_____________]    | |
|  | [Iniciar sesion]             | |
|  |                              | |
|  | No tienes cuenta?            | |
|  | [Crear cuenta]               | |
|  +------------------------------+ |
+-----------------------------------+
```

### Pestaña Nube — Conectado

```
+-----------------------------------+
|  Nube / Sincronizacion            |
|                                   |
|  Conectado como:                  |
|     Benjamin (ben@ejemplo.com)    |
|  Ultima sincronizacion: hoy       |
|  Email verificado: Si             |
|                                   |
|  [x] Auto-login al iniciar        |
|                                   |
|  [Sincronizar ahora]              |
|  [Cerrar sesion]                  |
|                                   |
|  Estado: Todo sincronizado        |
+-----------------------------------+
```

### Formulario de registro

```
+-----------------------------------+
|  Crear cuenta                     |
|                                   |
|  Nombre:     [_____________]     |
|  Email:      [_____________]     |
|  Contrasena: [_____________]     |
|  Confirmar:  [_____________]     |
|                                   |
|  [ ] Acepto que mis datos se     |
|      almacenen en la nube        |
|                                   |
|  [Crear cuenta]                  |
|  [Ya tengo cuenta]               |
+-----------------------------------+
```

### Verificacion de email (post-registro)

```
+-----------------------------------+
|  Verifica tu email                |
|                                   |
|  Te hemos enviado un codigo de    |
|  6 digitos a ben@ejemplo.com      |
|                                   |
|  Codigo: [__ __ __ __ __ __]     |
|                                   |
|  [Verificar]                      |
|  [Reenviar codigo]                |
+-----------------------------------+
```

### Indicador global (barra superior)

```
+-----------------------------------+
|  Mates    [nombre niño]  [☁️ ●]  |
+-----------------------------------+
         Punto: verde (#2ecc71) conectado
                rojo (#e74c3c) pendiente verificacion
                gris (#aaa)   desconectado
```

## Dependencias

- **Baserow.io**: cuenta del desarrollador, tablas ya creadas (IDs 1071739-1071743).
  El `BASEROW_API_TOKEN` vive exclusivamente en el Cloudflare Worker proxy.
- **Brevo API**: API key obtenida desde https://app.brevo.com/settings/keys/api.
  La `BREVO_API_KEY` vive exclusivamente en el Cloudflare Worker proxy.
- **Cloudflare Worker proxy**: inyecta tokens de Baserow y Brevo en las peticiones.
  Valida `X-Proxy-Key` (shared secret) y `X-User-Id` antes de proxyar.
- **Librerias Rust**: `reqwest`, `serde`, `serde_json`, `tokio` ya en `Cargo.toml`.
  `rand_core` (con feature `getrandom`) se usa para generar codigos de verificacion.
- **Sin nuevas dependencias npm**: frontend no requiere cambios.
- **Configuracion `.env`** (injectada en compile time via `build.rs`):

| Variable | Descripcion | Ejemplo |
|----------|-------------|---------|
| `PROXY_BASEROW_URL` | URL del proxy para Baserow | `http://localhost:8787/baserow` |
| `PROXY_BREVO_URL` | URL del proxy para Brevo | `http://localhost:8787/brevo` |
| `SHARED_SECRETS` | Secreto(s) compartido(s) para autenticar contra el proxy | `v1_<hex>` |
| `BASEROW_DATABASE_ID` | ID de la base de datos en Baserow (solo referencia) | `490809` |
| `BASEROW_API_TOKEN` | Token de Baserow (solo para `wrangler secret put`, no en la app) | `Token ...` |
| `BREVO_API_KEY` | API key de Brevo (solo para `wrangler secret put`, no en la app) | `xkeysib-...` |

## Definicion de Done

- [x] Codigo implementado (frontend + backend)
- [x] Existe una pestaña "Nube" en la zona adulta con formularios de login/registro
- [x] Validacion de contraseña (8+ caracteres) en frontend y backend
- [x] Confirmacion de contraseña en registro (frontend)
- [x] Consentimiento de privacidad en registro (frontend y backend)
- [x] Todos los mensajes en espanol
- [x] Existe `src-tauri/src/cloud/` con baserow.rs, commands.rs, sync.rs, mod.rs
- [x] Existe `src-tauri/src/email.rs` con el cliente HTTP y todas las funciones
- [x] `BREVO_API_KEY` y `BASEROW_API_TOKEN` viven exclusivamente en el proxy
      (Cloudflare Worker secrets), nunca en la app.
- [x] Comando `send_transac_email` implementado y registrado en Tauri
- [x] Comando `list_transac_emails` implementado y registrado en Tauri
- [x] Comando `get_email_content` implementado y registrado en Tauri
- [x] Comando `get_email_status` implementado y registrado en Tauri
- [x] Comando `delete_scheduled_email` implementado y registrado en Tauri
- [x] Todos los comandos devuelven `Result<..., String>` con mensajes en espanol
- [x] Manejo de errores: red, timeouts, HTTP 400/401/402/404/429/500
- [x] `pnpm check` no reporta errores
- [x] `cargo build` compila sin errores ni warnings
- [x] `register_account` envia codigo de 6 digitos y marca email como no verificado
- [x] `login_account` lee `field_9645672` de Baserow y lo refleja en DB local
- [x] `verify_email_code` valida codigo, marca verificado en local y actualiza Baserow
- [x] `resend_verification_code` genera nuevo codigo y lo reenvia por email
- [x] `CloudStatus` expone `email_verified: bool` y `auto_login: bool` al frontend
- [x] `get_app_status` y `get_cloud_status` incluyen `email_verified` y `auto_login`
- [x] `verify_email_code` y `resend_verification_code` registrados en `invoke_handler`
- [x] `delete_cloud_account` borra registro en Baserow via `delete_row` y limpia sesion local
- [x] `change_cloud_email` verifica duplicado, actualiza Baserow, genera nuevo codigo y lo envia
- [x] `sync_all_data` bloquea sincronizacion si `email_verified == false`
- [x] Indicador visual en CloudTab: punto rojo + "Validacion pendiente" si email no verificado
- [x] Interruptor "Auto-login" en la pestaña Nube cuando hay sesion activa
- [x] Al activar/desactivar, la preferencia se persiste en `app_settings`
- [x] Al reiniciar con auto-login activado, la sesion se restaura sin pedir credenciales
- [x] Al hacer login/registro, datos de sesion persisten y auto-login se activa
- [x] Al cerrar sesion, datos se limpian pero preferencia auto-login se conserva
- [x] Al arrancar con auto-login, sync en segundo plano (silenciosa si falla)
- [x] Indicador permanente de nube en barra superior (SVG + punto verde/gris/rojo)
- [x] Al arrancar con auto-login, notice "Sesion de nube restaurada"
- [x] Sincronizacion bidireccional funcional probada contra Baserow real
  - config: bidireccional (insercion de configs remotas nuevas)
  - profiles: bidireccional con last-modified-wins y soft-delete propagado
  - sessions: bidireccional (insercion de sesiones remotas nuevas y borrado propagado)
  - session_questions: bidireccional (insercion de preguntas remotas nuevas y borrado propagado)
- [x] Recuperacion de perfiles eliminados visibles en zona adulta con boton "Recuperar"
- [x] `recover_profile` recupera perfil + sesiones + preguntas (`deleted_at = NULL`)
- [x] `list_deleted_profiles` devuelve perfiles con `deleted_at IS NOT NULL` (solo zona adulta)
- [x] `write_remote_profiles` actualiza perfiles locales desde remoto (nombre, nivel, etc.)
- [x] `write_remote_profiles` recupera perfil local si remoto esta activo y es mas reciente
- [x] Las eliminaciones y recuperaciones respetan last-modified-wins en todas las tablas
      (no se pierde una recuperacion local por una eliminacion remota antigua, ni viceversa)
- [x] `syncNow`, `deleteProfile`, `recoverProfile` y `saveProfile` refrescan la UI tras sync
- [x] Proxy valida `X-Proxy-Key` (shared secret) contra `SHARED_SECRETS`
- [x] Proxy valida `X-User-Id` contra tabla de cuentas (excepto GET/HEAD de cuentas)
- [x] `BaserowClient` envia `X-Proxy-Key` y `X-User-Id` en cada peticion
- [x] URLs del proxy configurables via `.env` (`PROXY_BASEROW_URL`, `PROXY_BREVO_URL`)
- [x] `SHARED_SECRETS` soporta lista separada por comas para rotacion de secretos
- [x] `build.rs` inyecta variables del `.env` como `cargo:rustc-env`
- [x] Soft-delete individual de sesiones (`delete_session`/`recover_session`) se propaga via sync
- [x] `purge_old_sessions` elimina fisicamente sesiones con `deleted_at > 30 días` — solo local, no toca Baserow
- [x] `write_remote_sessions` inserta sesiones remotas nuevas en local
- [x] `write_remote_session_questions` inserta preguntas remotas nuevas en local
- [ ] Manejo de errores de red implementado (sin bloqueo de app)
  - Errores capturados y propagados como strings
  - Falta indicador visual "Sin conexion"
- [x] Persistencia de sesion entre reinicios
  - `user_id`, `user_name` y `email` se guardan en `app_settings` y se restauran
    en memoria al arrancar si `cloud_auto_login = "true"`
- [ ] Compatibilidad hacia atras: usuarios existentes no pierden datos ni tienen
      errores al actualizar
