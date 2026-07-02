# Contexto Del Proyecto

## Proposito

Aplicacion de escritorio para que ninos repasen matematicas mediante sesiones cortas, explicaciones motivadoras y seguimiento de progreso. La app debe servir para uso domestico y, en una fase posterior, profesional.

## Stack

- Aplicacion de escritorio: Tauri 2.
- Frontend: Svelte plano + TypeScript + Vite.
- Backend local: Rust.
- Persistencia: SQLite local.
- Idioma inicial: espanol.
- Primer objetivo: entorno de desarrollo, no instalador final.

## Alcance Inicial

- Primera version enfocada en Primaria.
- Cursos: 1o a 6o de primaria.
- Asignatura inicial: aritmetica basica segun curso.
- Sesiones por defecto de 10 preguntas.
- Preguntas de tipo numerico y texto libre.
- Dashboard inicial simple para adultos.
- Datos locales en primera version.

## Privacidad

- Nunca se enviara al LLM el nombre, alias, identificador ni datos personales del nino.
- Al LLM solo se enviara contexto pedagogico minimo: curso, edad si aplica, nivel, concepto, pregunta, respuesta y tipo de ejercicio.
- El historial completo queda local en SQLite.
- La arquitectura debe dejar preparada una futura sincronizacion en nube, pero no implementarla en el MVP.
- La configuracion/debug podra mostrar al adulto el texto exacto enviado al LLM para transparencia.

## Lenguaje Pedagogico

- No usar etiquetas injustas o negativas como "lento", "malo" o similares.
- Usar lenguaje motivador: "esta consolidando", "necesita practicar", "progresa con apoyo", "concepto en desarrollo".
- El dashboard debe informar sin etiquetar negativamente al nino.

## Zonas

### Zona Infantil

- El nino puede elegir perfil.
- El nino no puede cambiar a un nivel anterior o mas facil que el nivel minimo asociado a su edad/curso/perfil.
- Puede escoger un nivel superior si quiere practicar mas dificultad.
- Interfaz inicialmente simple y funcional, visualmente amable.
- Feedback futuro inmediato, claro y motivador.

### Zona Adulta

- Protegida por PIN de 4 a 6 digitos.
- El PIN se define en el primer arranque.
- El PIN se guarda localmente con hash Argon2, no en claro.
- Reset implementado como borrado local confirmado con la palabra `RESET`, porque no hay recuperacion segura de un PIN hasheado sin cuentas externas.
- Permite configurar perfiles y, mas adelante, proveedor LLM, parametros y dashboard.

## Proveedores LLM Planificados

La app debe tener una capa de proveedores intercambiables:

- OllamaProvider: local/offline si hay modelo instalado.
- GeminiProvider: cloud/API, util para calidad inicial.
- OpenAICompatibleProvider: endpoints compatibles con OpenAI, como LM Studio, OpenRouter, llama.cpp server u otros.
- Futuro posible: proveedor mock/local para desarrollo y pruebas sin coste ni red.

La seleccion del proveedor sera configurable desde la zona adulta. Las respuestas del LLM deben pedirse en formato estructurado y validarse antes de usarse o persistirse.

## Correccion Y Flujo De Sesion Futuro

- La app genera o solicita preguntas segun curso/nivel/concepto.
- Para respuestas numericas simples, se intentara correccion determinista.
- Para texto libre o razonamiento, se usara LLM con salida estructurada.
- Si el nino falla, se ofrece explicacion motivadora.
- Despues del fallo, se reformula el mismo concepto de otra manera para comprobar si lo ha comprendido.
- Al final se genera resumen detallado y se guardan metricas.

## Nivel Y Progreso

- El padre/tutor puede ajustar nivel manualmente.
- Por defecto, el nivel sera automatico.
- El nivel inicial se estima por curso y/o edad.
- En modo automatico nunca se retrocede de nivel; solo se mantiene o sube.
- En la gestion actual de perfiles, el nivel actual no baja al editar.

## Metricas A Guardar En Fases Posteriores

- Aciertos por sesion.
- Errores por sesion.
- Porcentaje de acierto.
- Respuesta dada.
- Respuesta esperada.
- Tipo de pregunta.
- Concepto trabajado.
- Dificultad.
- Curso/nivel asociado.
- Tiempo por pregunta.
- Numero de intentos.
- Si necesito explicacion.
- Si acerto tras explicacion.
- Numero de reformulaciones.
- Conceptos dominados.
- Conceptos en consolidacion.
- Conceptos que necesitan practica.
- Evolucion por sesion.
- Nivel estimado.
- Nivel manual, si existe.
- Proveedor LLM usado.
- Modelo usado.
- Latencia aproximada del proveedor.
- Errores de generacion/evaluacion.
- Resumen pedagogico de sesion.

## Fase 1 Implementada

- Proyecto Tauri 2 creado.
- Convertido a Svelte plano + TypeScript + Vite.
- Backend Rust configurado.
- `frontendDist` configurado a `dist`.
- Nombre de app: Mates.
- Identificador: `dev.mates.desktop`.

## Fase 2 Implementada

- SQLite local en el directorio de datos de la aplicacion.
- Tablas actuales:
  - `app_settings` para configuracion local.
  - `profiles` para perfiles infantiles.
- Primer arranque con PIN adulto.
- Verificacion de PIN adulto.
- Bloqueo/desbloqueo de zona adulta en sesion.
- Reset local con confirmacion `RESET`.
- Crear, editar, listar y eliminar perfiles.
- Perfil con nombre visible, curso 1o-6o, edad opcional, modo automatico/manual y nivel actual.
- Validacion: edad 6-12, curso 1-6, PIN 4-6 digitos.
- La UI separa zona infantil y zona adulta.
- La zona infantil permite seleccionar perfil y queda preparada para sesiones futuras.

## Fase 3 Implementada

- Modulo `src-tauri/src/llm/` con trait y tres proveedores.
- `LLMProviderEnum` para evitar `Box<dyn Trait>` (async no dyn-compatible).
- OllamaProvider: `/api/chat`, timeout configurable.
- GeminiProvider: API REST Gemini, deteccion de errores 401/429.
- OpenAICompatibleProvider: `/v1/chat/completions`, con o sin API key.
- Los tres proveedores devuelven `Result<T, String>`.
- Parseo JSON con limpieza de code blocks markdown.
- Sistema de prompts en espanol para ninos.
- Configuracion LLM persistida en `app_settings` (proveedor, modelo, URL, API key).
- Comandos Tauri: `get_llm_config`, `set_llm_config`, `test_llm_connection`.
- UI adulta con pestana "IA / LLM" para configurar proveedor.
- Dependencias: `reqwest`, `tokio`, `async-trait`, `thiserror`, `derive_more`.

## Fase 4 Implementada

- Tablas SQLite: `sessions` y `session_questions`.
- Comandos Tauri: `start_session`, `submit_answer`, `get_explanation`, `end_session`, `list_sessions`.
- Flujo completo: perfil selecciona → sesión de 10 preguntas → feedback → explicación → resumen.
- Evaluación local determinista para respuestas numéricas (normalización, tolerancia).
- Fallback a LLM para explicaciones y reformulación del concepto.
- Detección de conceptos débiles del perfil (`get_weakest_concept`).
- UI sesión infantil: barra de progreso, pregunta, respuesta, feedback, explicación.
- UI resumen: aciertos, precisión, tiempo, conceptos dominados/practicar, detalle por pregunta.
- Historial de sesiones en zona adulta.
- Reset de datos cascade (session_questions → sessions → profiles → app_settings).

## Fase 5 Implementada

- Dashboard adulto con pestana dedicada.
- Comandos Tauri: `get_dashboard_stats`, `get_concept_stats`, `get_evolution`, `export_sessions`.
- Resumen general: total sesiones, precision promedio, tiempo total, tiempo promedio/pregunta.
- Analisis de conceptos: dominados (>=80%), en progreso (50-79%), necesitan practica (<50%).
- Detalle por concepto con barra de progreso, porcentajes y filtro por concepto.
- Evolucion de precision por sesion (grafica de barras).
- Sesiones recientes con barra de precision para comparativa temporal.
- Exportacion de datos en CSV y JSON.
- Filtro por perfil en todas las vistas del dashboard.
- Estilos CSS completos para el dashboard.

## Fase 6 Implementada

- Tablas SQLite: `users`, `student_groups`, `student_group_members`, `tutor_student`, `parent_student`, `assignments`, `reports`.
- Roles de usuario: "parent", "tutor", "admin".
- Comandos Tauri: `create_user`, `login_user`, `list_users`, `create_student_group`, `list_student_groups`, `add_student_to_group`, `remove_student_from_group`, `list_group_students`, `assign_student_to_tutor`, `remove_student_from_tutor`, `list_tutor_students`, `create_assignment`, `list_assignments`, `generate_report`, `list_reports`, `get_tutor_dashboard`.
- Pestaña "Profesional" en zona adulta con subpestanas: Resumen, Estudiantes, Groups, Tareas, Reportes.
- Dashboard profesional: total estudiantes, tareas activas, reportes generados, lista de estudiantes con precision.
- Asignacion de estudiantes a tutores (N:N).
- Creacion de grupos/clases (nombre local, NO se envia al LLM).
- Creacion de tareas (concepto, dificultad, fecha limite).
- Generacion de reportes por estudiante y periodo.
- Reglas de privacidad: nombre de colegio/clase NUNCA se envia al LLM.
- Estilos CSS completos para la capa profesional.

## Decisiones Pendientes Futuras

- Elegir modelo recomendado de Ollama.
- Definir reglas exactas de promocion de nivel.
- Definir diseno visual infantil final.
- Definir empaquetado/instalador.
- FUTURO: Login con email+password para tutores.
- FUTURO: OAuth (Google, GitHub) para tutores.
