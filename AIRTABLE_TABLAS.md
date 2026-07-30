# Esquema de tablas en Airtable

> Documento de referencia para crear las tablas en Airtable.
> Cada tabla debe estar dentro de la misma Base.

---

## Tabla 1: `accounts`

Almacena las cuentas de los usuarios finales de la app.

### Campos

| Nombre del campo | Tipo Airtable | Descripción | Requerido | Único |
|---|---|---|---|---|
| `email` | Email | Email del usuario | Si | Si |
| `name` | Single line text | Nombre completo del usuario | Si | No |
| `password_hash` | Long text | Hash Argon2 de la contraseña | Si | No |
| `created_at` | Single line text | Fecha de creación en formato RFC 3339 | Si | No |

### Notas

- Airtable genera automáticamente un `Record ID` único que usaremos como identificador interno del usuario (`user_id`).
- No se almacena la contraseña en texto plano, solo el hash Argon2 calculado localmente en Rust.
- El campo `email` debe ser único: al crear la tabla, Airtable no impone unicidad por defecto. Debemos verificarlo desde el código (antes de insertar, consultamos si existe).
- No hay contraseña olvidada ni recuperación en esta version.

---

## Tabla 2: `user_config`

Almacena configuraciones clave-valor por usuario (p.ej. configuración LLM).

### Campos

| Nombre del campo | Tipo Airtable | Descripción | Requerido |
|---|---|---|---|
| `user_id` | Single line text | ID del registro en `accounts` (Record ID de Airtable) | Si |
| `key` | Single line text | Clave de configuración (ej. `llm_provider`, `llm_model`) | Si |
| `value` | Long text | Valor de la configuración | Si |
| `updated_at` | Single line text | Fecha de ultima modificación (RFC 3339) | Si |

### Relaciones

- `user_id` referencia a `accounts.Record ID` (1:N — un usuario tiene muchas configuraciones).
- Par `(user_id, key)` debe ser único (no puede haber dos valores para la misma clave del mismo usuario). Se valida desde el código.

---

## Tabla 3: `user_profiles`

Almacena los perfiles infantiles sincronizados.

### Campos

| Nombre del campo | Tipo Airtable | Descripción | Requerido |
|---|---|---|---|
| `user_id` | Single line text | ID del usuario dueño de este perfil | Si |
| `profile_id` | Single line text | UUID del perfil (mismo que en SQLite local) | Si |
| `display_name` | Single line text | Nombre visible del niño | Si |
| `school_year` | Number (Integer) | Curso escolar (1-6) | Si |
| `age` | Number (Integer) | Edad del niño (6-12, puede estar vacío) | No |
| `level_mode` | Single line text | `automatic` o `manual` | Si |
| `current_level` | Number (Integer) | Nivel actual (1-6) | Si |
| `manual_prompt` | Long text | Contexto pedagógico manual | No |
| `created_at` | Single line text | Fecha de creación (RFC 3339) | Si |
| `updated_at` | Single line text | Fecha de ultima modificación (RFC 3339) | Si |

### Relaciones

- `user_id` referencia a `accounts.Record ID` (1:N — un usuario tiene muchos perfiles).
- `profile_id` es el mismo UUID que en la base SQLite local, usado como identificador para sincronizar.

---

## Tabla 4: `user_sessions`

Almacena las sesiones de juego sincronizadas.

### Campos

| Nombre del campo | Tipo Airtable | Descripción | Requerido |
|---|---|---|---|
| `user_id` | Single line text | ID del usuario dueño de la sesión | Si |
| `session_id` | Single line text | UUID de la sesión (mismo que en SQLite local) | Si |
| `profile_id` | Single line text | UUID del perfil que realizo la sesión | Si |
| `status` | Single line text | Estado: `active`, `completed`, `abandoned` | Si |
| `total_questions` | Number (Integer) | Total de preguntas de la sesión | Si |
| `questions_answered` | Number (Integer) | Preguntas respondidas hasta ahora | Si |
| `correct_count` | Number (Integer) | Respuestas correctas | Si |
| `current_question_index` | Number (Integer) | Indice de la pregunta actual | Si |
| `started_at` | Single line text | Fecha de inicio (RFC 3339) | Si |
| `ended_at` | Single line text | Fecha de fin (RFC 3339, puede estar vacío) | No |

### Relaciones

- `user_id` referencia a `accounts.Record ID` (1:N).
- `profile_id` referencia a `user_profiles.profile_id` (1:N — un perfil tiene muchas sesiones).
- `session_id` es el mismo UUID que en SQLite local.
- La relación con `user_session_questions` se hace mediante `session_id`.

---

## Tabla 5: `user_session_questions`

Almacena cada pregunta individual de las sesiones sincronizadas.

### Campos

| Nombre del campo | Tipo Airtable | Descripción | Requerido |
|---|---|---|---|
| `user_id` | Single line text | ID del usuario dueño | Si |
| `session_id` | Single line text | UUID de la sesión a la que pertenece | Si |
| `question_id` | Single line text | UUID de la pregunta (mismo que en SQLite local) | Si |
| `question_text` | Long text | Enunciado de la pregunta | Si |
| `correct_answer` | Single line text | Respuesta correcta esperada | Si |
| `student_answer` | Single line text | Respuesta dada por el estudiante | No |
| `concept` | Single line text | Concepto trabajado (ej. "sumas y restas") | Si |
| `difficulty` | Single line text | Dificultad: `easy`, `medium`, `hard` | Si |
| `is_correct` | Number (Integer) | 1 si acertó, 0 si falló, vacío si sin responder | No |
| `explanation` | Long text | Explicación generada para esta pregunta | No |
| `question_number` | Number (Integer) | Numero de pregunta dentro de la sesión (1-based) | Si |
| `time_spent_secs` | Number (Integer) | Segundos que tardo en responder | No |
| `created_at` | Single line text | Fecha de creación (RFC 3339) | Si |
| `answered_at` | Single line text | Fecha cuando se respondió (RFC 3339) | No |

### Relaciones

- `user_id` referencia a `accounts.Record ID` (1:N).
- `session_id` referencia a `user_sessions.session_id` (1:N — una sesión tiene muchas preguntas).
- `question_id` es el mismo UUID que en SQLite local.

---

## Diagrama de relaciones

```
accounts (1) ---< (N) user_config
accounts (1) ---< (N) user_profiles
user_profiles (1) ---< (N) user_sessions   (via profile_id)
user_sessions (1) ---< (N) user_session_questions   (via session_id)
```

Cada `---<` significa "uno a muchos".

---

## Resumen de campos compartidos

| Campo | Donde se usa | Propósito |
|---|---|---|
| `Record ID` (Airtable) | Todas las tablas | Identificador único interno de Airtable |
| `user_id` | accounts (propio), user_config, user_profiles, user_sessions, user_session_questions | Vincula todos los datos a un usuario |
| `profile_id` | user_profiles (propio), user_sessions | Vincula sesiones a perfiles |
| `session_id` | user_sessions (propio), user_session_questions | Vincula preguntas a sesiones |
| `updated_at` | user_config, user_profiles | Usado para resolución de conflictos (last-writer-wins) |

---

## Notas para la creación en Airtable

1. Crear una nueva Base (desde cero o desde plantilla vacía).
2. Añadir 5 tablas con los nombres exactos: `accounts`, `user_config`, `user_profiles`, `user_sessions`, `user_session_questions`.
3. En cada tabla, añadir los campos usando el tipo indicado en la columna "Tipo Airtable".
4. **No** crear campos de tipo "Link to another record" como relación formal. Las relaciones se manejan desde el código Rust (join por campo de texto). Esto simplifica la API y evita restricciones de Airtable.
5. No es necesario crear vistas ni filtros. Solo las tablas con sus campos.
6. Airtable genera automáticamente los campos `Record ID` (visible en la API como `id`) y `Created Time`. Ignorar `Created Time`; usamos nuestros propios campos `created_at` y `updated_at`.
