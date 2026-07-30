# Mates — instrucciones

## Stack

- **Desktop**: Tauri 2
- **Frontend**: Svelte 5 plano + TypeScript + Vite
- **Backend**: Rust con SQLite (rusqlite bundled), argon2, reqwest
- **Idioma**: español (locale `es-ES`)

## Comandos

| Comando | Descripción |
|---------|-------------|
| `pnpm dev` | Vite dev server (puerto 1420) |
| `pnpm build` | Vite build |
| `pnpm check` | Type-check con svelte-check |
| `pnpm tauri` | Tauri CLI |

No hay ESLint, Prettier ni rustfmt configurados.
**CI**: `.github/workflows/build-desktop.yml` — compila en Linux, Windows y macOS (manual o push tag `v*`). Usa pnpm.

## Frontend

- **Entrypoint**: `index.html` → `src/main.ts`
- **App principal**: `src/App.svelte` (único componente, ~1100 líneas, sin router)
- **Type-check**: `tsconfig.json` con `strict: true`, `moduleDetection: "force"`
- **Proto extensiones**: `src/prototypes.ts` — añade `toInt()`, `toFloat()` a `Number` y `String`. **Debe importarse** (`import "./prototypes"`) en `main.ts` para que el runtime funcione. Las declaraciones globales usan `declare global` + `export {}`.
- `src/routes/` existe pero está vacío. No usar SvelteKit routing.

### Convenciones
- Usamos `pnpm`
- Mantén compatibilidad con node v22.23.1
- Documenta todos los métodos y funciones nuevas con JSDoc, siguiendo el patron existente en el proyecto.
- En el final de linea, no poner `;` a no ser que la siguiente línea comience por `(`.

### Patrón UI: elementos eliminados

Para mostrar elementos en soft-delete y permitir su recuperación, seguir el patrón de `ProfilesTab.svelte`:

```svelte
{#if array.length > 0}
  <details class="deleted-profiles">
    <summary>X eliminados ({array.length})</summary>
    {#each array as item (item.id)}
      <div class="profile-row muted">
        <div>...información del elemento...</div>
        <button class="secondary" type="button" onclick={() => appState.recover(item.id)}>Recuperar</button>
      </div>
    {/each}
  </details>
{/if}
```

- El `<details>` usa `class="deleted-profiles"` (reutilizada, aunque diga "profiles").
- Cada fila usa `class="profile-row muted"`.
- Botón "Recuperar" con `class="secondary"`.
- El estado reactivo se almacena en `app-state.svelte.ts` con un array `deletedXxx` y un método `loadDeletedXxx`.
- La llamada a recuperar incluye sync con la nube si hay sesión activa.

## Backend (Rust)

- **Directorio**: `src-tauri/`, lib name `mates_lib`
- **Entrypoint**: `src-tauri/src/lib.rs` (~1850 líneas, toda la lógica Tauri + SQLite)
- **LLM providers**: `src-tauri/src/llm/` — Ollama, Gemini, OpenAI-compatible
- **Conexiones**: `reqwest` + `tokio` (multi-thread)
- **Build**: `cargo build` desde `src-tauri/` (o vía `npm run tauri dev`)

### Convenciones
- Documenta todos las funciones nuevas
- Usa 4 espacios como tabulación

## Sincronización cloud (Baserow)

La sincronización es **siempre bidireccional** con last-writer-wins.

### Regla de oro
Cada `sync_all_data` ejecuta **upload + download** secuencialmente para cada tabla:

```
sync_profiles_table:
  1. upload_local_profiles  → sube locals más recientes a Baserow
  2. write_remote_profiles  → descarga remotes más recientes a SQLite
```

No existe el concepto de "solo push" o "solo pull". Ambas fases se ejecutan siempre.

### Cómo se determina el ganador
- Cada fila tiene `updated_at` (RFC3339). Se compara como string (orden lexicográfico válido).
- Gana la fila con `updated_at` más reciente, sin importar si es local o remota.
- Esto aplica también a **eliminaciones y recuperaciones** (soft-delete con `deleted_at`).
- Para sesiones/preguntas activas (no borradas), si existen en ambos lados gana la versión local (cada dispositivo es origen de sus sesiones).

### Frontend: refrescar la UI tras sync
Siempre que se llame a `sync_all_data` hay que llamar también a `refreshStatus()` después para que la UI recoja los cambios descargados. Sin esto, los datos en SQLite se actualizan pero la UI muestra datos obsoletos.

**Puntos donde se hace sync + refreshStatus:**
- `saveProfile` / `deleteProfile` / `recoverProfile`
- `syncNow` (botón manual)
- Arranque con auto-login en `App.svelte:onMount`

## Proyecto

- No es monorepo; un solo app con frontend (`src/`) y backend (`src-tauri/`)
- `CONTEXT.md` contiene la especificación completa del proyecto (privacidad, flujo pedagógico, fases implementadas). Consultar antes de cambios arquitectónicos.
- No hay tests ni scripts de testing.

## Variables de entorno

El proyecto usa un archivo `.env` (gitignored) con estas variables, que NUNCA serán leídas:

```
BASEROW_DATABASE_ID=xxx
BASEROW_API_TOKEN=your_token
BASEROW_API_URL=https://api.baserow.io/api
```

| Variable | Descripción |
|----------|-------------|
| `BASEROW_DATABASE_ID` | ID de la base de datos "Mates" en Baserow |
| `BASEROW_API_TOKEN` | Token de acceso de la base de datos (generar en https://baserow.io/fr/profile/account) |
| `BASEROW_API_URL` | URL base de la API de Baserow |

También existe un `.env.example` como plantilla para otros desarrolladores.


## Salida
- Devuelve el código primero. Explicación posterior, sólo si no es obvia.
- Sin prosa en línea. Utilice los comentarios con moderación, sólo cuando la lógica no esté clara.
- No hay código repetitivo a menos que se solicite explícitamente.
- Siempre en Español

## Reglas del código
- La solución de trabajo más sencilla. Sin ingeniería excesiva.
- No hay abstracciones para operaciones de un solo uso.
- Sin características especulativas ni "quizás también quieras..."
- Lea el archivo antes de modificarlo. Nunca edites a ciegas.
- No se modifican cadenas de documentos ni anotaciones de tipo en el código.
- No hay manejo de errores para escenarios que no pueden ocurrir.
- Tres líneas similares es mejor que una abstracción prematura.
- **No leer ni acceder al archivo `.env` bajo ningún concepto.** Contiene claves y secretos que no deben exponerse.

## Reglas de revisión
- Indique el error. Muestra la solución. Detener.
- No hay sugerencias más allá del alcance de la revisión.
- No hay elogios sobre el código antes o después de la revisión.

## Reglas de depuración
- Nunca especule sobre un error sin leer primero el código relevante.
- Indica qué encontraste, dónde y la solución. Una pasada.
- Si la causa no está clara: dígalo. No adivines.

## Formato simple
- Sin guiones, comillas inteligentes ni símbolos Unicode decorativos.
- Solo guiones simples y comillas rectas.
- Los caracteres en lenguaje natural (CJK, etc.) están bien cuando el contenido los requiere.
- La salida del código debe ser segura para copiar y pegar.
