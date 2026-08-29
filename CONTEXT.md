# Contexto Del Proyecto

## Propósito

Aplicación multiplataforma (escritorio + Android) para que niños repasen matemáticas mediante sesiones cortas, explicaciones motivadoras y seguimiento de progreso. La app debe servir para uso domestico y, en una fase posterior, profesional.

## Stack

- Escritorio: Tauri 2 (Windows, Linux, macOS).
- Móvil: Tauri 2 + Android (aarch64, APK).
- Frontend: Svelte plano + TypeScript + Vite.
- Backend local: Rust.
- Persistencia: SQLite local.
- Idioma: el sistema detecta el locale del dispositivo (`navigator.language`) y fuerza al LLM a responder en ese idioma.
- Primer objetivo: entorno de desarrollo, no instalador final.

## Alcance Inicial

- Primera version enfocada en Primaria.
- Cursos: 1o a 6o de primaria.
- Asignatura inicial: aritmética básica según curso.
- Sesiones por defecto de 10 preguntas.
- Preguntas de tipo numérico y texto libre.
- Dashboard inicial simple para adultos.
- Datos locales en primera version.

## Privacidad

- Nunca se enviara al LLM el nombre, alias, identificador ni datos personales del niño.
- Al LLM solo se enviara contexto pedagógico mínimo: curso, edad si aplica, nivel, concepto, pregunta, respuesta y tipo de ejercicio.
- En modo manual, el adulto puede definir un contexto pedagogico adicional por perfil para orientar preguntas, explicaciones y reformulaciones.
- El contexto pedagogico manual no debe contener nombres ni datos personales.
- El historial completo queda local en SQLite.
- La arquitectura tiene una capa de sincronización en nube implementada (Fase 7 WIP).
- Las API keys de Baserow y Brevo nunca están en la app; viven exclusivamente en el Cloudflare Worker proxy.
- La configuración/debug podrá mostrar al adulto el texto exacto enviado al LLM para transparencia.

## Lenguaje Pedagogico

- No usar etiquetas injustas o negativas como "lento", "malo" o similares.
- Usar lenguaje motivador: "esta consolidando", "necesita practicar", "progresa con apoyo", "concepto en desarrollo".
- El dashboard debe informar sin etiquetar negativamente al niño.

## Zonas

### Zona Infantil

- El niño puede elegir perfil.
- El niño no puede cambiar a un nivel anterior o mas fácil que el nivel mínimo asociado a su edad/curso/perfil.
- Puede escoger un nivel superior si quiere practicar mas dificultad.
- Interfaz inicialmente simple y funcional, visualmente amable.
- Feedback futuro inmediato, claro y motivador.

### Zona Adulta

- Protegida por PIN de 4 a 6 dígitos.
- El PIN se define en el primer arranque.
- El PIN se guarda localmente con hash Argon2, no en claro.
- Reset implementado como borrado local confirmado con la palabra `RESET`, porque no hay recuperación segura de un PIN hasheado sin cuentas externas.
- Permite configurar perfiles y, mas adelante, proveedor LLM, parámetros y dashboard.
- Los elementos eliminados (soft-delete) se muestran en un `<details class="deleted-profiles">` con botón "Recuperar". El patrón está documentado en `AGENTS.md`.

## Proveedores LLM Planificados

La app debe tener una capa de proveedores intercambiables:

- OllamaProvider: local/offline si hay modelo instalado.
- GeminiProvider: cloud/API, util para calidad inicial.
- OpenAICompatibleProvider: endpoints compatibles con OpenAI, como LM Studio, OpenRouter, llama.cpp server u otros.
- Futuro posible: proveedor mock/local para desarrollo y pruebas sin coste ni red.

La selección del proveedor sera configurable desde la zona adulta. Las respuestas del LLM deben pedirse en formato estructurado y validarse antes de usarse o persistirse.

### Locale

El idioma de las respuestas del LLM se fuerza mediante el locale del dispositivo. El frontend envía `navigator.language` (ej. `"es-ES"`, `"ca-ES"`, `"eu-ES"`) al backend al arrancar. El system prompt de cada llamada al LLM incluye `"Responde siempre en {idioma}."` con el mapeo:

| Código | Idioma   |
|--------|----------|
| `es`   | espanol  |
| `ca`   | catalan  |
| `eu`   | euskera  |
| `gl`   | gallego  |
| `en`   | ingles   |
| otro   | espanol  |

## corrección Y Flujo De sesión Futuro

- La app genera o solicita preguntas según curso/nivel/concepto.
- Para respuestas numéricas simples, se intentara corrección determinista.
- Para texto libre o razonamiento, se usara LLM con salida estructurada.
- Si el niño falla, se ofrece explicación motivadora.
- Después del fallo, se reformula el mismo concepto de otra manera para comprobar si lo ha comprendido.
- Al final se genera resumen detallado y se guardan métricas.

## Nivel Y Progreso

- El padre/tutor puede ajustar nivel manualmente.
- Por defecto, el nivel sera automático.
- El nivel inicial se estima por curso y/o edad.
- En modo automático nunca se retrocede de nivel; solo se mantiene o sube.
- En modo manual se puede definir un contexto pedagogico para la IA asociado al perfil.
- En modo manual, la generación de preguntas prioriza el contexto pedagogico manual y no fuerza el concepto débil automático del historial.
- En la gestión actual de perfiles, el nivel actual no baja al editar.

## Metricas A Guardar En Fases Posteriores

- Aciertos por sesión.
- Errores por sesión.
- Porcentaje de acierto.
- Respuesta dada.
- Respuesta esperada.
- Tipo de pregunta.
- Concepto trabajado.
- Dificultad.
- Curso/nivel asociado.
- Tiempo por pregunta.
- Numero de intentos.
- Si necesito explicación.
- Si acierto tras explicación.
- Numero de reformulaciones.
- Conceptos dominados.
- Conceptos en consolidación.
- Conceptos que necesitan practica.
- Evolución por sesión.
- Nivel estimado.
- Nivel manual, si existe.
- Proveedor LLM usado.
- Modelo usado.
- Latencia aproximada del proveedor.
- Errores de generación/evaluación.
- Resumen pedagógico de sesión.
- Contexto pedagogico manual usado, si existe.

## Fase 1 Implementada

- Proyecto Tauri 2 creado.
- Convertido a Svelte plano + TypeScript + Vite.
- Backend Rust configurado.
- `frontendDist` configurado a `dist`.
- Nombre de app: Mates.
- Identificador: `es.benjamin.mates`.

## Fase 2 Implementada

- SQLite local en el directorio de datos de la aplicación.
- Tablas actuales:
  - `app_settings` para configuración local.
  - `profiles` para perfiles infantiles.
- Primer arranque con PIN adulto.
- Verificación de PIN adulto.
- Bloqueo/desbloqueo de zona adulta en sesión.
- Reset local con confirmación `RESET`.
- Crear, editar, listar y eliminar perfiles.
- Perfil con nombre visible, curso 1o-6o, edad opcional, modo automático/manual, nivel actual y contexto pedagogico manual opcional.
- Validación: edad 6-12, curso 1-6, PIN 4-6 dígitos, contexto pedagogico manual máximo 1000 caracteres.
- La UI separa zona infantil y zona adulta.
- La zona infantil permite seleccionar perfil y queda preparada para sesiones futuras.

## Fase 3 Implementada

- Modulo `src-tauri/src/llm/` con trait y tres proveedores.
- `LLMProviderEnum` para evitar `Box<dyn Trait>` (async no dyn-compatible).
- OllamaProvider: `/api/chat`, timeout configurable.
- GeminiProvider: API REST Gemini, deteccion de errores 401/429.
- OpenAICompatibleProvider: `/v1/chat/completions`, con o sin API key.
- Los tres proveedores devuelven `Result<T, String>`.
- Modulo `common.rs` con funciones compartidas: parsing JSON, prompts y parseo de respuestas LLM.
- Cada provider solo implementa `generate_text()` con su transporte HTTP especifico.
- Todos los métodos públicos documentados con docstrings Rust (`///`), incluyendo parámetros y retorno.
- Sistema de prompts en espanol para niños.
- Los prompts pueden recibir contexto pedagogico manual del perfil para preguntas, explicaciones y reformulaciones.
- Los proveedores registran en stderr el prompt exacto enviado al LLM para depuración local.
- Configuración LLM persistida en `app_settings` (proveedor, modelo, URL, API key).
- Comandos Tauri: `get_llm_config`, `set_llm_config`, `test_llm_connection`.
- UI adulta con pestaña "IA / LLM" para configurar proveedor.
- Dependencias: `reqwest`, `tokio`, `async-trait`, `thiserror`, `derive_more`.
- Cuando el historial no tiene datos suficientes (<2 preguntas por concepto), se usa un concepto predeterminado por curso (`get_default_concept_for_year()`): sumas/restas (1o), multiplicación (3o), fracciones/decimales (5o), porcentajes (6o), etc.
- Las respuestas vacías del LLM se detectan antes del parseo JSON y devuelven un error claro en vez de "EOF while parsing a value".
- Los errores de parseo JSON incluyen los primeros 300 caracteres de la respuesta cruda del LLM para facilitar la depuración.

## Fase 4 Implementada

- Tablas SQLite: `sessions` y `session_questions`.
- Comandos Tauri: `start_session`, `submit_answer`, `get_explanation`, `end_session`, `list_sessions`.
- Flujo completo: perfil selecciona → sesión de 10 preguntas → feedback → explicación → resumen.
- Evaluación local determinista para respuestas numéricas (normalización, tolerancia).
- Fallback a LLM para explicaciones y reformulación del concepto.
- Detección de conceptos débiles del perfil (`get_weakest_concept`) en modo automático.
- En modo manual, la generación de preguntas omite el concepto débil automático y prioriza el contexto pedagogico manual.
- UI sesión infantil: barra de progreso, pregunta, respuesta, feedback, explicación.
- UI resumen: aciertos, precisión, tiempo, conceptos dominados/practicar, detalle por pregunta.
- Historial de sesiones en zona adulta.
- Reset de datos cascade (session_questions → sessions → profiles → app_settings).

## Fase 5 Implementada

- Dashboard adulto con pestaña dedicada.
- Comandos Tauri: `get_dashboard_stats`, `get_concept_stats`, `get_evolution`, `export_sessions`.
- Resumen general: total sesiones, precision promedio, tiempo total, tiempo promedio/pregunta.
- Análisis de conceptos: dominados (>=80%), en progreso (50-79%), necesitan practica (<50%).
- Detalle por concepto con barra de progreso, porcentajes y filtro por concepto.
- Evolución de precision por sesión (gráfica de barras).
- Sesiones recientes con barra de precision para comparativa temporal.
- Exportación de datos en CSV y JSON.
- Filtro por perfil en todas las vistas del dashboard.
- Estilos CSS completos para el dashboard.

## Fase 6 Implementada

- Tablas SQLite: `users`, `student_groups`, `student_group_members`, `tutor_student`, `parent_student`, `assignments`, `reports`.
- Roles de usuario: "parent", "tutor", "admin".
- Comandos Tauri: `create_user`, `login_user`, `list_users`, `create_student_group`, `list_student_groups`, `add_student_to_group`, `remove_student_from_group`, `list_group_students`, `assign_student_to_tutor`, `remove_student_from_tutor`, `list_tutor_students`, `create_assignment`, `list_assignments`, `generate_report`, `list_reports`, `get_tutor_dashboard`.
- Pestaña "Profesional" en zona adulta con subpestañas: Resumen, Estudiantes, Groups, Tareas, Reportes.
- Dashboard profesional: total estudiantes, tareas activas, reportes generados, lista de estudiantes con precision.
- Asignación de estudiantes a tutores (N:N).
- Creación de grupos/clases (nombre local, NO se envía al LLM).
- Creación de tareas (concepto, dificultad, fecha limite).
- Generación de reportes por estudiante y periodo.
- Reglas de privacidad: nombre de colegio/clase NUNCA se envía al LLM.
- Estilos CSS completos para la capa profesional.

## Fase 7 (WIP — capa cloud)

- Modulo `src-tauri/src/cloud/` con cliente HTTP para Baserow API REST.
- `BaserowClient`: listar, crear, actualizar y buscar filas en tablas predefinidas (5 tablas).
  - Cliente ya no recibe `database_id` (se eliminó por no usarse).
  - Métodos eliminados por no usarse: `get_row()`, `delete_row()`.
- `sync.rs`: sincronizacion bidireccional (last-writer-wins) de configuracion, perfiles, sesiones y preguntas.
- Comandos Tauri: `register_account`, `login_account`, `logout_account`, `sync_all_data`, `get_cloud_status`, `set_cloud_auto_login`.
- UI: pestana "Nube" en zona adulta con formularios de registro/login y panel de estado conectado.
- Persistencia de sesion local en `app_settings` (clave `cloud_session_user_id`).
- **Auto-login persistente**:
  - Claves en `app_settings`: `cloud_session_user_name`, `cloud_session_email`, `cloud_auto_login`.
  - Al hacer login/registro, el auto-login se activa implícitamente.
  - Al arrancar la app, si `cloud_auto_login = true` y existe `user_id`, la sesión se
    restaura en memoria sin llamar a Baserow.
  - Interruptor "Auto-login al iniciar" en la pestaña Nube para activar/desactivar.
  - Indicador permanente de estado de nube en la barra superior (icono SVG de nube +
    punto verde si conectado, gris si desconectado), visible desde cualquier pantalla.
  - Al arrancar con auto-login, se muestra notice "Sesion de nube restaurada" y se
    ejecuta sincronización en segundo plano (silenciosa si falla).
- `refreshStatus()` en el frontend actualiza `cloudStatus` desde `get_app_status`.
- `get_app_status` y `get_cloud_status` devuelven `auto_login` en el `CloudStatus`.
- Contrasena hasheada con Argon2 localmente antes de enviar a Baserow.
- Las operaciones HTTP contra Baserow se reintentan hasta 3 veces con 500ms entre intentos ante fallos de conexión o errores HTTP
- **Proxy Cloudflare Worker** como capa intermedia entre la app y las APIs externas:
  - La app nunca conoce las API keys de Baserow ni Brevo; el proxy las inyecta.
  - URLs del proxy configurables via `.env` (`PROXY_BASEROW_URL`, `PROXY_BREVO_URL`), inyectadas en compile time via `build.rs`.
  - **Autenticación del proxy (3 capas)**:
    1. `X-Proxy-Key`: shared secret validado contra `SHARED_SECRETS` (lista separada por comas para rotación).
    2. `X-User-Id`: ID de usuario validado contra tabla de cuentas (1071739). Excepción: lecturas de la tabla de cuentas sin user_id (login/registro).
    3. Token inyectado por el Worker (`BASEROW_API_TOKEN` / `BREVO_API_KEY`).

## Errores Conocidos / Trabajos Futuros

### Truncamiento de JSON del LLM por `max_tokens`

**Síntoma**: Ocasionalmente, al generar preguntas, el LLM devuelve el JSON truncado a mitad de un string, provocando `"Error JSON: EOF while parsing a string at line 1 column N"`. El mensaje de error muestra los primeros 300 caracteres de la respuesta cruda.

**Causa**: El parámetro `max_tokens` / `max_output_tokens` estaba en 1024 tanto en OpenAI-compatible como en Gemini. Si el LLM genera una pregunta/explicación larga (p.ej. describiendo un patrón matemático), la respuesta se corta antes de cerrar el JSON.

**Solución aplicada (2026-07-06)**:
- `src-tauri/src/llm/openai_compatible.rs:80`: `Some(1024)` → `Some(2048)`
- `src-tauri/src/llm/gemini.rs:81`: `Some(1024)` → `Some(2048)`

**Próximo paso si el error persiste**: Implementar un reintento automático en `generate_question_for_session` (`lib.rs:1400-1413`) con un bucle de hasta 3 intentos que solo reintente si el error contiene `"Error JSON"` (truncamiento/parseo). Si el error no es de parseo (red, DB, etc.), fallar inmediatamente.

## Decisiones Pendientes Futuras

- Elegir modelo recomendado de Ollama.
- Definir reglas exactas de promoción de nivel.
- Definir diseño visual infantil final.
- Definir empaquetado/instalador.
- FUTURO: Login con email+password para tutores.
- FUTURO: OAuth (Google, GitHub) para tutores.
