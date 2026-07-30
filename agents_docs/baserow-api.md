# Baserow API — guía para LLMs

Conexión con la base de datos Mates via API REST de Baserow.

## Conexión

- **Base URL**: `https://api.baserow.io/api/database/`
- **Auth**: header `Authorization: Token <token>`
- **Database ID** y **Access Token**: definidos en `.env` como `BASEROW_DATABASE_ID` y `BASEROW_API_TOKEN`
- Formato: JSON. Content-Type: `application/json`

## Tablas

| Nombre | table_id | primary_field |
|--------|----------|---------------|
| accounts | 1071739 | email (field_9480686) |
| user_config | 1071740 | user_id (field_9480689) |
| user_profiles | 1071741 | profile_id (field_9480692) |
| user_sessions | 1071742 | session_id (field_9480695) |
| user_session_questions | 1071743 | question_id (field_9480698) |

## Campos por tabla

### accounts (1071739)
- field_9480686: email (string, PK)
- field_9480701: name (string)
- field_9480702: password_hash (string, multilinea)
- field_9480703: created_at (string)

### user_config (1071740)
- field_9480689: user_id (string, PK)
- field_9480705: key (string)
- field_9480706: value (string, multilinea)
- field_9480707: updated_at (string)

### user_profiles (1071741)
- field_9480692: profile_id (string, PK)
- field_9480708: user_id (string)
- field_9480709: display_name (string)
- field_9480710: school_year (number)
- field_9480711: age (number)
- field_9480712: level_mode (string)
- field_9480713: current_level (number)
- field_9480714: manual_prompt (string)
- field_9480715: created_at (string)
- field_9480716: updated_at (string)
- field_9679183: deleted_at (string, null si activo)

### user_sessions (1071742)
- field_9480695: session_id (string, PK)
- field_9480717: user_id (string)
- field_9480718: profile_id (string)
- field_9480719: status (string)
- field_9480720: total_questions (number)
- field_9480721: questions_answered (number)
- field_9480722: correct_count (number)
- field_9480723: current_question_index (number)
- field_9480724: started_at (string)
- field_9480725: ended_at (string)
- field_9679263: updated_at (string)
- field_9679269: deleted_at (string, null si activo)

### user_session_questions (1071743)
- field_9480698: question_id (string, PK)
- field_9480726: user_id (string)
- field_9480727: session_id (string)
- field_9480728: question_text (string, multilinea)
- field_9480729: correct_answer (string)
- field_9480730: student_answer (string)
- field_9480731: concept (string)
- field_9480732: difficulty (string)
- field_9480733: is_correct (number, 0/1)
- field_9480734: explanation (string, multilinea)
- field_9480735: question_number (number)
- field_9480736: time_spent_secs (number)
- field_9480737: created_at (string)
- field_9480738: answered_at (string)
- field_9679275: updated_at (string)
- field_9679279: deleted_at (string, null si activo)

## Endpoints

`GET /fields/table/{table_id}/` — listar campos
`GET /rows/table/{table_id}/` — listar filas (paginado)
`GET /rows/table/{table_id}/{row_id}/` — obtener una fila
`POST /rows/table/{table_id}/` — crear fila
`PATCH /rows/table/{table_id}/{row_id}/` — actualizar fila
`PATCH /rows/table/{table_id}/{row_id}/move/` — mover fila
`DELETE /rows/table/{table_id}/{row_id}/` — eliminar fila

### Upload de archivos

`POST https://api.baserow.io/api/user-files/upload-file/` — multipart con campo `file`
`POST https://api.baserow.io/api/user-files/upload-via-url/` — body `{"url": "..."}`

### Listar tablas del database

`GET /tables/database/{database_id}/`

## Query params (list rows)

- `page` (int, default 1)
- `size` (int, default 100)
- `search` (string)
- `order_by` (string, ej: `field_1` o `-field_1` para descendente)
- `user_field_names` (bool, si se pasa los campos se devuelven por nombre en vez de field_XXXX)
- `filters` (JSON, árbol de filtros)
- `filter__{field}__{filter}` (string, ej: `filter__Name__equal=test`)
- `filter_type` (AND/OR, default AND)
- `include` (string, campos separados por coma a incluir)
- `exclude` (string, campos separados por coma a excluir)
- `view_id` (int, aplica filtros/orden de una vista)
- `{link_field}__join` (string, incluye campos de tabla vinculada)

## Filtros por tipo de campo

**Text**: equal, not_equal, contains, contains_not, contains_word, doesnt_contain_word, starts_with, length_is_lower_than, empty, not_empty
**Number**: higher_than, higher_than_or_equal, lower_than, lower_than_or_equal, is_even_and_whole, empty, not_empty, equal, not_equal, contains, contains_not
**Date**: date_is, date_is_not, date_is_before, date_is_after, date_is_on_or_before, date_is_on_or_after, date_is_within
**Select**: single_select_equal, single_select_not_equal, single_select_is_any_of, single_select_is_none_of, multiple_select_has, multiple_select_has_not
**Link row**: link_row_has, link_row_has_not, link_row_contains, link_row_not_contains
**Boolean**: boolean (true/false)
**File**: filename_contains, has_file_type (image/document), files_lower_than

## Notas

- Para escribir en campos singleSelect/multipleSelects usar el **nombre** de la opción (string), no el objeto
- El parámetro `user_field_names` cambia cómo se interpretan `order_by`, `include`, `exclude` y `filters` (esperan nombres reales en vez de field_XXXX)
- Usar `before` en POST para insertar en una posición específica
- `send_webhook_events` (opcional, default true) controla si se disparan webhooks

> Documentación completa en `docs/doc-API-Baserow.md`
