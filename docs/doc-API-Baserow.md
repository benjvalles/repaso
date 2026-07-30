# Documentación de la API de la base de datos de Mates

Base de datos abierta. La base de datos Mates proporciona una manera fácil de integrar los datos con cualquier sistema externo. La API sigue la semántica REST, utiliza JSON para codificar objetos y se basa en códigos HTTP estándar, errores de lectura humana y mecánica para señalar los resultados de las operaciones.

- **ID de esta base de datos**: 490809
- **Cliente API de ejemplo JavaScript**: axios
- **Cliente API de ejemplo Python**: requests

## Autenticación

Baserow utiliza una autenticación simple basada en tokens. Es necesario generar al menos un token de base de datos en los ajustes para utilizar los endpoints descritos a continuación. Es posible dar permisos de creación, lectura, actualización y eliminación hasta el nivel de tabla por token.

Puede autenticarse en la API proporcionando su token en la cabecera HTTP `Authorization: Bearer <token>`. Todas las solicitudes deben ser autenticadas y realizadas a través de HTTPS.

```bash
curl \
  -H "Authorization: Token YOUR_TOKEN" \
  "https://api.baserow.io/api/database/"
```

## Tablas

- [accounts](#accounts-tabla-id-1071739)
- [user_config](#user_config-tabla-id-1071740)
- [user_profiles](#user_profiles-tabla-id-1071741)
- [user_sessions](#user_sessions-tabla-id-1071742)
- [user_session_questions](#user_session_questions-tabla-id-1071743)

---

## accounts tabla (id: 1071739)

### Campos

Cada fila de la tabla `accounts` contiene los siguientes campos:

| ID | Nombre | Tipo | Descripción | Filtros compatibles |
|----|--------|------|-------------|---------------------|
| field_9480686 | email | string | Acepta texto de una sola línea. | equal, not_equal, contains, contains_not, contains_word, doesnt_contain_word, starts_with, length_is_lower_than, empty, not_empty |
| field_9480701 | name | string | Acepta texto de una sola línea. | equal, not_equal, contains, contains_not, contains_word, doesnt_contain_word, starts_with, length_is_lower_than, empty, not_empty |
| field_9480702 | password_hash | string | Acepta texto multilínea. Si el formato de texto enriquecido está activado, puede utilizar Markdown para dar formato al texto. | equal, not_equal, contains, contains_not, contains_word, doesnt_contain_word, starts_with, length_is_lower_than, empty, not_empty |
| field_9480703 | created_at | string | Acepta texto de una sola línea. | equal, not_equal, contains, contains_not, contains_word, doesnt_contain_word, starts_with, length_is_lower_than, empty, not_empty |

### Campos de lista

Para enumerar los campos de la tabla `accounts`, se debe realizar una solicitud GET al extremo de los campos. Solo es posible enumerar los campos si el token de la base de datos tiene permisos de lectura, creación o actualización.

```
GET https://api.baserow.io/api/database/fields/table/1071739/
```

```bash
curl -X GET \
  -H "Authorization: Token YOUR_TOKEN" \
  "https://api.baserow.io/api/database/fields/table/1071739/"
```

**Propiedades del campo de resultado**:

| Propiedad | Tipo | Descripción |
|-----------|------|-------------|
| id | integer | Clave principal del campo. Se puede usar para generar el nombre de la columna de la base de datos agregando el prefijo `field_`. |
| name | string | Nombre del campo. |
| table_id | integer | ID de tabla relacionada. |
| order | integer | Orden de campos en la tabla. 0 para el primer campo. |
| type | string | Tipo definido para este campo. |
| primary | boolean | Indica si el campo es un campo principal. Si es verdadero, el campo no se puede eliminar y el valor debe representar la fila completa. |
| read_only | boolean | Indica si el campo es de sólo lectura. Si es cierto, no es posible actualizar el valor de la celda. |
| description | string | Campo descripción. |

**Muestra de respuesta**:

```json
[
  {
    "id": 9480686,
    "table_id": 1071739,
    "name": "email",
    "order": 0,
    "type": "text",
    "primary": true,
    "read_only": false,
    "description": "..."
  },
  {
    "id": 9480701,
    "table_id": 1071739,
    "name": "name",
    "order": 1,
    "type": "text",
    "primary": false,
    "read_only": false,
    "description": "..."
  },
  {
    "id": 9480702,
    "table_id": 1071739,
    "name": "password_hash",
    "order": 2,
    "type": "long_text",
    "primary": false,
    "read_only": false,
    "description": "..."
  }
]
```

### Filas de lista

Para enumerar las filas en la tabla `accounts`, se debe realizar una solicitud GET al extremo. La respuesta está paginada.

```
GET https://api.baserow.io/api/database/rows/table/1071739/
```

```bash
curl -X GET \
  -H "Authorization: Token YOUR_TOKEN" \
  "https://api.baserow.io/api/database/rows/table/1071739/"
```

**Parámetros de consulta**:

| Parámetro | Requerido | Tipo | Descripción |
|-----------|-----------|------|-------------|
| page | opcional | integer | Default: 1. Define qué página de filas debe devolverse. |
| size | opcional | integer | Default: 100. Define cuántas filas se deben devolver por página. |
| user_field_names | opcional | any | Cuando se proporciona, los nombres de campo devueltos serán los nombres reales de los campos. |
| search | opcional | string | Default: ''. Si se proporcionan, solo se devolverán las filas con datos que coincidan con la consulta de búsqueda. |
| order_by | opcional | string | Default: 'id'. Opcionalmente se pueden ordenar las filas por campos separados por coma. |
| filters | opcional | JSON | Las filas se pueden filtrar opcionalmente utilizando un árbol de filtros serializado JSON. |
| filter__{field}__{filter} | opcional | string | Filtrar por campo y tipo de filtro. |
| filter_type | opcional | string | Default: 'AND'. AND/OR para combinar filtros. |
| include | opcional | string | Lista separada por comas de campos a incluir. |
| exclude | opcional | string | Lista separada por comas de campos a excluir. |
| view_id | opcional | integer | Aplica los filtros y ordenaciones de una vista. |
| {link_row_field}__join | opcional | string | Permite solicitar valores de campo de una tabla de destino a través de campos de fila de vínculo. |

**Muestra de respuesta**:

```json
{
  "count": 1024,
  "next": "https://api.baserow.io/api/database/rows/table/1071739/?page=2",
  "previous": null,
  "results": [
    {
      "id": 0,
      "order": "1.00000",
      "email": "string",
      "name": "string",
      "password_hash": "string",
      "created_at": "string"
    }
  ]
}
```

### Obtener fila

Obtiene una sola fila de `accounts`.

```
GET https://api.baserow.io/api/database/rows/table/1071739/{row_id}/
```

**Parámetros de ruta**:

| Parámetro | Tipo | Descripción |
|-----------|------|-------------|
| row_id | integer | El identificador único de la fila que se solicita. |

**Parámetros de consulta**:

| Parámetro | Requerido | Tipo | Descripción |
|-----------|-----------|------|-------------|
| user_field_names | opcional | any | Cuando se proporciona, los nombres de campo devueltos serán los nombres reales de los campos. |

```json
{
  "id": 0,
  "order": "1.00000",
  "email": "string",
  "name": "string",
  "password_hash": "string",
  "created_at": "string"
}
```

### Crear fila

Crea una nueva fila en `accounts`.

```
POST https://api.baserow.io/api/database/rows/table/1071739/
```

```bash
curl -X POST \
  -H "Authorization: Token YOUR_TOKEN" \
  -H "Content-Type: application/json" \
  "https://api.baserow.io/api/database/rows/table/1071739/" \
  --data '{
    "email": "string",
    "name": "string",
    "password_hash": "string",
    "created_at": "string"
  }'
```

**Parámetros de consulta**:

| Parámetro | Requerido | Tipo | Descripción |
|-----------|-----------|------|-------------|
| user_field_names | opcional | any | Cuando se proporciona, los nombres de campo devueltos serán los nombres reales de los campos. |
| before | opcional | integer | Si se proporciona, la fila recién creada se colocará antes de la fila con la identificación proporcionada. |
| send_webhook_events | opcional | any | Activa los webhooks después de la operación. |

**Solicitar esquema de cuerpo**:

| Campo | ID | Tipo | Requerido | Descripción |
|-------|-----|------|-----------|-------------|
| email | field_9480686 | string | opcional | Acepta texto de una sola línea. |
| name | field_9480701 | string | opcional | Acepta texto de una sola línea. |
| password_hash | field_9480702 | string | opcional | Acepta texto multilínea. |
| created_at | field_9480703 | string | opcional | Acepta texto de una sola línea. |

```json
{
  "id": 0,
  "order": "1.00000",
  "email": "string",
  "name": "string",
  "password_hash": "string",
  "created_at": "string"
}
```

### Actualizar fila

Actualiza una fila existente de `accounts`.

```
PATCH https://api.baserow.io/api/database/rows/table/1071739/{row_id}/
```

**Parámetros de ruta**:

| Parámetro | Tipo | Descripción |
|-----------|------|-------------|
| row_id | integer | El identificador único de la fila que debe actualizarse. |

**Parámetros de consulta**: user_field_names (opcional), send_webhook_events (opcional)

**Solicitar esquema de cuerpo**: Mismos campos que en Crear fila.

### Mover fila

Mueve una fila existente antes de otra fila.

```
PATCH https://api.baserow.io/api/database/rows/table/1071739/{row_id}/move/
```

**Parámetros de ruta**:

| Parámetro | Tipo | Descripción |
|-----------|------|-------------|
| row_id | integer | Mueve la fila relacionada con el valor. (opcional) |

**Parámetros de consulta**:

| Parámetro | Tipo | Descripción |
|-----------|------|-------------|
| before_id | integer | Mueve la fila relacionada con el `row_id` dado antes de la fila relacionada con el valor proporcionado. Si no se proporciona, la fila se moverá al final. |
| send_webhook_events | any | Activa los webhooks después de la operación. |
| user_field_names | any | Cuando se proporciona, los nombres de campo devueltos serán los nombres reales de los campos. |

### Borrar fila

Elimina una fila existente de `accounts`.

```
DELETE https://api.baserow.io/api/database/rows/table/1071739/{row_id}/
```

```bash
curl -X DELETE \
  -H "Authorization: Token YOUR_TOKEN" \
  "https://api.baserow.io/api/database/rows/table/1071739/{row_id}/"
```

**Parámetros de ruta**:

| Parámetro | Tipo | Descripción |
|-----------|------|-------------|
| row_id | integer | El identificador único de la fila que debe eliminarse. |

**Parámetros de consulta**: send_webhook_events (opcional)

---

## user_config tabla (id: 1071740)

### Campos

| ID | Nombre | Tipo | Descripción | Filtros compatibles |
|----|--------|------|-------------|---------------------|
| field_9480689 | user_id | string | Acepta texto de una sola línea. | equal, not_equal, contains, contains_not, contains_word, doesnt_contain_word, starts_with, length_is_lower_than, empty, not_empty |
| field_9480705 | key | string | Acepta texto de una sola línea. | equal, not_equal, contains, contains_not, contains_word, doesnt_contain_word, starts_with, length_is_lower_than, empty, not_empty |
| field_9480706 | value | string | Acepta texto multilínea. | equal, not_equal, contains, contains_not, contains_word, doesnt_contain_word, starts_with, length_is_lower_than, empty, not_empty |
| field_9480707 | updated_at | string | Acepta texto de una sola línea. | equal, not_equal, contains, contains_not, contains_word, doesnt_contain_word, starts_with, length_is_lower_than, empty, not_empty |

### Endpoints

- `GET /api/database/fields/table/1071740/` — Listar campos
- `GET /api/database/rows/table/1071740/` — Listar filas
- `GET /api/database/rows/table/1071740/{row_id}/` — Obtener fila
- `POST /api/database/rows/table/1071740/` — Crear fila
- `PATCH /api/database/rows/table/1071740/{row_id}/` — Actualizar fila
- `PATCH /api/database/rows/table/1071740/{row_id}/move/` — Mover fila
- `DELETE /api/database/rows/table/1071740/{row_id}/` — Borrar fila

---

## user_profiles tabla (id: 1071741)

### Campos

| ID | Nombre | Tipo | Descripción | Filtros compatibles |
|----|--------|------|-------------|---------------------|
| field_9480692 | profile_id | string | Acepta texto de una sola línea. | equal, not_equal, contains, contains_not, contains_word, doesnt_contain_word, starts_with, length_is_lower_than, empty, not_empty |
| field_9480708 | user_id | string | Acepta texto de una sola línea. | equal, not_equal, contains, contains_not, contains_word, doesnt_contain_word, starts_with, length_is_lower_than, empty, not_empty |
| field_9480709 | display_name | string | Acepta texto de una sola línea. | equal, not_equal, contains, contains_not, contains_word, doesnt_contain_word, starts_with, length_is_lower_than, empty, not_empty |
| field_9480710 | school_year | number | Acepta un número positivo. | contains, contains_not, starts_with, higher_than, higher_than_or_equal, lower_than, lower_than_or_equal, is_even_and_whole, empty, not_empty |
| field_9480711 | age | number | Acepta un número positivo. | higher_than, higher_than_or_equal, lower_than, lower_than_or_equal, is_even_and_whole, empty, not_empty |
| field_9480712 | level_mode | string | Acepta texto de una sola línea. | equal, not_equal, contains, contains_not, contains_word, doesnt_contain_word, starts_with, length_is_lower_than, empty, not_empty |
| field_9480713 | current_level | number | Acepta un número positivo. | higher_than, higher_than_or_equal, lower_than, lower_than_or_equal, is_even_and_whole, empty, not_empty |
| field_9480714 | manual_prompt | string | Acepta texto de una sola línea. | equal, not_equal, contains, contains_not, contains_word, doesnt_contain_word, starts_with, length_is_lower_than, empty, not_empty |
| field_9480715 | created_at | string | Acepta texto de una sola línea. | equal, not_equal, contains, contains_not, contains_word, doesnt_contain_word, starts_with, length_is_lower_than, empty, not_empty |
| field_9480716 | updated_at | string | Acepta texto de una sola línea. | equal, not_equal, contains, contains_not, contains_word, doesnt_contain_word, starts_with, length_is_lower_than, empty, not_empty |
| field_9679183 | deleted_at | string | Marca temporal ISO 8601 de borrado lógico. `null` si activo. | empty, not_empty |

### Endpoints

- `GET /api/database/fields/table/1071741/` — Listar campos
- `GET /api/database/rows/table/1071741/` — Listar filas
- `GET /api/database/rows/table/1071741/{row_id}/` — Obtener fila
- `POST /api/database/rows/table/1071741/` — Crear fila
- `PATCH /api/database/rows/table/1071741/{row_id}/` — Actualizar fila
- `PATCH /api/database/rows/table/1071741/{row_id}/move/` — Mover fila
- `DELETE /api/database/rows/table/1071741/{row_id}/` — Borrar fila

---

## user_sessions tabla (id: 1071742)

### Campos

| ID | Nombre | Tipo | Descripción | Filtros compatibles |
|----|--------|------|-------------|---------------------|
| field_9480695 | session_id | string | Acepta texto de una sola línea. | equal, not_equal, contains, contains_not, contains_word, doesnt_contain_word, starts_with, length_is_lower_than, empty, not_empty |
| field_9480717 | user_id | string | Acepta texto de una sola línea. | equal, not_equal, contains, contains_not, contains_word, doesnt_contain_word, starts_with, length_is_lower_than, empty, not_empty |
| field_9480718 | profile_id | string | Acepta texto de una sola línea. | equal, not_equal, contains, contains_not, contains_word, doesnt_contain_word, starts_with, length_is_lower_than, empty, not_empty |
| field_9480719 | status | string | Acepta texto de una sola línea. | equal, not_equal, contains, contains_not, contains_word, doesnt_contain_word, starts_with, length_is_lower_than, empty, not_empty |
| field_9480720 | total_questions | number | Acepta un número positivo. | contains, contains_not, starts_with, higher_than, higher_than_or_equal, lower_than, lower_than_or_equal, is_even_and_whole, empty, not_empty |
| field_9480721 | questions_answered | number | Acepta un número positivo. | contains, contains_not, starts_with, higher_than, higher_than_or_equal, lower_than, lower_than_or_equal, is_even_and_whole, empty, not_empty |
| field_9480722 | correct_count | number | Acepta un número positivo. | contains, contains_not, starts_with, higher_than, higher_than_or_equal, lower_than, lower_than_or_equal, is_even_and_whole, empty, not_empty |
| field_9480723 | current_question_index | number | Acepta un número positivo. | contains, contains_not, starts_with, higher_than, higher_than_or_equal, lower_than, lower_than_or_equal, is_even_and_whole, empty, not_empty |
| field_9480724 | started_at | string | Acepta texto de una sola línea. | equal, not_equal, contains, contains_not, contains_word, doesnt_contain_word, starts_with, length_is_lower_than, empty, not_empty |
| field_9480725 | ended_at | string | Acepta texto de una sola línea. | equal, not_equal, contains, contains_not, contains_word, doesnt_contain_word, starts_with, length_is_lower_than, empty, not_empty |
| field_9679263 | updated_at | string | Marca temporal ISO 8601 de última modificación. | empty, not_empty |
| field_9679269 | deleted_at | string | Marca temporal ISO 8601 de borrado lógico. `null` si activo. | empty, not_empty |

### Endpoints

- `GET /api/database/fields/table/1071742/` — Listar campos
- `GET /api/database/rows/table/1071742/` — Listar filas
- `GET /api/database/rows/table/1071742/{row_id}/` — Obtener fila
- `POST /api/database/rows/table/1071742/` — Crear fila
- `PATCH /api/database/rows/table/1071742/{row_id}/` — Actualizar fila
- `PATCH /api/database/rows/table/1071742/{row_id}/move/` — Mover fila
- `DELETE /api/database/rows/table/1071742/{row_id}/` — Borrar fila

---

## user_session_questions tabla (id: 1071743)

### Campos

| ID | Nombre | Tipo | Descripción | Filtros compatibles |
|----|--------|------|-------------|---------------------|
| field_9480698 | question_id | string | Acepta texto de una sola línea. | equal, not_equal, contains, contains_not, contains_word, doesnt_contain_word, starts_with, length_is_lower_than, empty, not_empty |
| field_9480726 | user_id | string | Acepta texto de una sola línea. | equal, not_equal, contains, contains_not, contains_word, doesnt_contain_word, starts_with, length_is_lower_than, empty, not_empty |
| field_9480727 | session_id | string | Acepta texto de una sola línea. | equal, not_equal, contains, contains_not, contains_word, doesnt_contain_word, starts_with, length_is_lower_than, empty, not_empty |
| field_9480728 | question_text | string | Acepta texto multilínea. Puede utilizar Markdown. | equal, not_equal, contains, contains_not, contains_word, doesnt_contain_word, starts_with, length_is_lower_than, empty, not_empty |
| field_9480729 | correct_answer | string | Acepta texto de una sola línea. | equal, not_equal, contains, contains_not, contains_word, doesnt_contain_word, starts_with, length_is_lower_than, empty, not_empty |
| field_9480730 | student_answer | string | Acepta texto de una sola línea. | equal, not_equal, contains, contains_not, contains_word, doesnt_contain_word, starts_with, length_is_lower_than, empty, not_empty |
| field_9480731 | concept | string | Acepta texto de una sola línea. | equal, not_equal, contains, contains_not, contains_word, doesnt_contain_word, starts_with, length_is_lower_than, empty, not_empty |
| field_9480732 | difficulty | string | Acepta texto de una sola línea. | equal, not_equal, contains, contains_not, contains_word, doesnt_contain_word, starts_with, length_is_lower_than, empty, not_empty |
| field_9480733 | is_correct | number | Acepta un número positivo. | higher_than, higher_than_or_equal, lower_than, lower_than_or_equal, is_even_and_whole, empty, not_empty |
| field_9480734 | explanation | string | Acepta texto multilínea. Puede utilizar Markdown. | equal, not_equal, contains, contains_not, contains_word, doesnt_contain_word, starts_with, length_is_lower_than, empty, not_empty |
| field_9480735 | question_number | number | Acepta un número positivo. | higher_than, higher_than_or_equal, lower_than, lower_than_or_equal, is_even_and_whole, empty, not_empty |
| field_9480736 | time_spent_secs | number | Acepta un número positivo. | higher_than, higher_than_or_equal, lower_than, lower_than_or_equal, is_even_and_whole, empty, not_empty |
| field_9480737 | created_at | string | Acepta texto de una sola línea. | equal, not_equal, contains, contains_not, contains_word, doesnt_contain_word, starts_with, length_is_lower_than, empty, not_empty |
| field_9480738 | answered_at | string | Acepta texto de una sola línea. | equal, not_equal, contains, contains_not, contains_word, doesnt_contain_word, starts_with, length_is_lower_than, empty, not_empty |
| field_9679275 | updated_at | string | Marca temporal ISO 8601 de última modificación. | empty, not_empty |
| field_9679279 | deleted_at | string | Marca temporal ISO 8601 de borrado lógico. `null` si activo. | empty, not_empty |

### Endpoints

- `GET /api/database/fields/table/1071743/` — Listar campos
- `GET /api/database/rows/table/1071743/` — Listar filas
- `GET /api/database/rows/table/1071743/{row_id}/` — Obtener fila
- `POST /api/database/rows/table/1071743/` — Crear fila
- `PATCH /api/database/rows/table/1071743/{row_id}/` — Actualizar fila
- `PATCH /api/database/rows/table/1071743/{row_id}/move/` — Mover fila
- `DELETE /api/database/rows/table/1071743/{row_id}/` — Borrar fila

---

## Carga de archivos

### Subir archivo directamente

Sube un archivo a Baserow cargando el contenido del archivo directamente.

```
POST https://api.baserow.io/api/user-files/upload-file/
```

```bash
curl -X POST \
  -H "Authorization: Token YOUR_TOKEN" \
  -F file=@photo.png \
  "https://api.baserow.io/api/user-files/upload-file/"
```

**Cuerpo de la solicitud**: multipart con el campo `file`.

```json
{
  "url": "https://files.baserow.io/...",
  "thumbnails": {
    "tiny": { "url": "...", "width": 21, "height": 21 },
    "small": { "url": "...", "width": 48, "height": 48 }
  },
  "name": "VXotniBOVm8tbstZkKsMK...",
  "size": 229940,
  "mime_type": "image/png",
  "is_image": true,
  "image_width": 1280,
  "image_height": 585,
  "uploaded_at": "2020-11-17T12:..."
}
```

### Subir archivo a través de URL

Sube un archivo a Baserow descargándolo de la URL proporcionada.

```
POST https://api.baserow.io/api/user-files/upload-via-url/
```

```bash
curl -X POST \
  -H "Authorization: Token YOUR_TOKEN" \
  -H "Content-Type: application/json" \
  "https://api.baserow.io/api/user-files/upload-via-url/" \
  --data '{
    "url": "https://baserow.io/assets/..."
  }'
```

**Cuerpo de la solicitud**:

| Campo | Tipo | Descripción |
|-------|------|-------------|
| url | string | Sube un archivo a Baserow descargándolo de la URL proporcionada. |

---

## Listar todas las tablas

Este endpoint solo funciona con autenticación mediante token. Enumera todas las tablas a las que el token tiene acceso.

```
GET https://api.baserow.io/api/database/tables/database/490809/
```

```bash
curl -X GET \
  -H "Authorization: Token YOUR_TOKEN" \
  "https://api.baserow.io/api/database/tables/database/490809/"
```

```json
[
  {
    "id": 0,
    "name": "string",
    "order": 2147483647,
    "database_id": 0
  }
]
```

---

## Filtros

### Filtros de texto

| Filtro | Valor ejemplo | Descripción |
|--------|---------------|-------------|
| equal | string | campo es 'string' |
| not_equal | string | campo no es 'string' |
| contains | string | campo contiene 'string' |
| contains_not | string | campo no contenga 'string' |
| contains_word | string | campo contiene palabras 'string' |
| doesnt_contain_word | string | campo no contiene palabras 'string' |
| starts_with | string | campo starts with 'string' |
| length_is_lower_than | 5 | campo la longitud es menor que '5' |
| empty | — | campo está vacío |
| not_empty | — | campo no está vacío |

### Filtros de fecha

| Filtro | Descripción |
|--------|-------------|
| date_is | campo es 'fecha' |
| date_is_not | campo no es 'fecha' |
| date_is_before | campo es antes de 'fecha' |
| date_is_on_or_before | campo es en o antes de 'fecha' |
| date_is_after | campo es después de 'fecha' |
| date_is_on_or_after | campo es en o después de 'fecha' |
| date_is_within | campo está dentro de 'fecha' |
| date_equal | (obsoleto) |
| date_not_equal | (obsoleto) |
| date_before_today | (obsoleto) |
| date_after_today | (obsoleto) |
| date_within_days | (obsoleto) |
| date_within_weeks | (obsoleto) |
| date_within_months | (obsoleto) |

### Filtros numéricos

| Filtro | Valor ejemplo | Descripción |
|--------|---------------|-------------|
| higher_than | 100 | campo Más alto que '100' |
| higher_than_or_equal | 100 | campo mayor o igual que '100' |
| lower_than | 100 | campo Más bajo que '100' |
| lower_than_or_equal | 100 | campo menos o igual que '100' |
| is_even_and_whole | true | campo es par y entero 'true' |

### Filtros de selección

| Filtro | Descripción |
|--------|-------------|
| single_select_equal | campo es 'id' |
| single_select_not_equal | campo no es 'id' |
| single_select_is_any_of | campo es cualquiera de 'ids' |
| single_select_is_none_of | campo es ninguno de 'ids' |
| multiple_select_has | campo tiene alguno de 'id' |
| multiple_select_has_not | campo no tiene ninguno de 'id' |

### Filtros de enlace (link row)

| Filtro | Descripción |
|--------|-------------|
| link_row_has | campo tiene 'id' |
| link_row_has_not | campo no tenga 'id' |
| link_row_contains | campo contiene 'string' |
| link_row_not_contains | campo no contenga 'string' |

### Filtros booleanos

| Filtro | Descripción |
|--------|-------------|
| boolean | campo es 'true' |

### Filtros de archivo

| Filtro | Valor ejemplo | Descripción |
|--------|---------------|-------------|
| filename_contains | string | campo el nombre del archivo contiene 'string' |
| has_file_type | image / document | campo tiene tipo de archivo |
| files_lower_than | 2 | campo archivos inferiores a '2' |

### Filtros de colaboradores

| Filtro | Descripción |
|--------|-------------|
| user_is | campo es 'id' |
| user_is_not | campo no es 'id' |
| multiple_collaborators_has | campo tiene 'id' |
| multiple_collaborators_has_not | campo no tenga 'id' |

### Filtros de valor (para campos link)

| Filtro | Descripción |
|--------|-------------|
| has_value_equal | campo tiene un valor igual a 'string' |
| has_not_value_equal | campo no tiene el mismo valor |
| has_value_contains | campo contiene un valor 'string' |
| has_not_value_contains | campo no contiene un valor 'string' |
| has_value_contains_word | campo el valor contiene una palabra 'string' |
| has_not_value_contains_word | campo el valor no contiene una palabra 'string' |
| has_value_length_is_lower_than | campo tiene un valor inferior a 'string' |
| has_value_higher | campo tiene un valor mayor que 'string' |
| has_not_value_higher | campo no tiene un valor mayor que 'string' |
| has_value_higher_or_equal | campo tiene un valor mayor o igual que 'string' |
| has_not_value_higher_or_equal | campo no tiene un valor mayor o igual que 'string' |
| has_value_lower | campo tiene un valor menor que 'string' |
| has_not_value_lower | campo no tiene un valor menor que 'string' |
| has_value_lower_or_equal | campo tiene un valor menor o igual que 'string' |
| has_not_value_lower_or_equal | campo no tiene un valor menor que o igual que 'string' |
| has_all_values_equal | campo tiene todos los valores iguales 'string' |
| has_any_select_option_equal | campo tiene alguna opción de selección igual 'string' |
| has_none_select_option_equal | campo no tiene ninguna opción de selección igual 'string' |

### Filtros de fecha para valores link

| Filtro | Descripción |
|--------|-------------|
| has_date_equal | campo tiene fecha igual |
| has_not_date_equal | campo no tiene fecha igual |
| has_date_before | campo tiene fecha anterior |
| has_not_date_before | campo no tiene fecha anterior |
| has_date_on_or_before | campo tiene fecha en o antes de |
| has_not_date_on_or_before | campo no tiene fecha en o antes de |
| has_date_after | campo tiene fecha posterior |
| has_not_date_after | campo no tiene fecha posterior |
| has_date_on_or_after | campo tiene fecha en o después de |
| has_not_date_on_or_after | campo no tiene fecha en o después de |
| has_date_within | campo tiene fecha dentro |
| has_not_date_within | campo no tiene fecha dentro |

---

## Errores HTTP

| Código | Nombre | Descripción |
|--------|--------|-------------|
| 200 | Ok | Solicitud completada con éxito. |
| 400 | Bad request | La solicitud contiene valores no válidos o no se pudo analizar el JSON. |
| 401 | Unauthorized | Cuando intenta acceder a un punto final sin un token de la base de datos válido. |
| 404 | Not found | No se encuentra la fila o la tabla. |
| 413 | Request Entity Too Large | La solicitud superó el tamaño de carga útil máximo permitido. |
| 500 | Internal Server Error | El servidor encontró una condición inesperada. |
| 502 | Bad gateway | Baserow se está reiniciando o hay una interrupción inesperada en curso. |
| 503 | Service unavailable | El servidor no pudo procesar su solicitud a tiempo. |

```json
{
  "error": "ERROR_NO_PERMISSION...",
  "description": "The token does not..."
}
```
