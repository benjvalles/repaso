# Mates

Aplicación multiplataforma (escritorio + Android) para que niños de primaria repasen matemáticas mediante sesiones cortas, explicaciones motivadoras y seguimiento de progreso. Pensada para uso doméstico y profesional. Construida con Tauri 2, Svelte 5, TypeScript, Rust y SQLite.

## Características

- Sesiones de preguntas adaptadas al curso (1o a 6o de primaria)
- Evaluación automática de respuestas numéricas
- Explicaciones y reformulaciones generadas por IA
- Tres proveedores LLM intercambiables: Ollama (local/offline), Gemini (cloud) y OpenAI-compatible (LM Studio, OpenRouter, etc.)
- Zona adulta protegida por PIN con gestión de perfiles, dashboard, estadísticas y exportación de datos
- Capa profesional con estudiantes, tutores, grupos, tareas y reportes
- Sincronización cloud opcional mediante Baserow (bidireccional, last-writer-wins)
- Privacidad: nunca se envían datos personales del niño al LLM, solo contexto pedagógico
- Soporte multilingüe: español, catalán, euskera, gallego e inglés (según locale del dispositivo)
- Nivel automático (nunca retrocede) o manual con contexto pedagógico personalizado
- Interfaz en español (detecta el locale del dispositivo)

## Stack tecnológico

| Capa | Tecnología |
|------|------------|
| Frontend | Svelte 5 + TypeScript + Vite |
| Backend | Rust + SQLite (rusqlite) |
| Desktop | Tauri 2 (Linux, Windows, macOS) |
| Móvil | Tauri 2 Android (APK aarch64) |
| IA | Ollama / Gemini / OpenAI-compatible |
| Cloud | Baserow (REST API) + Brevo (email) |

## Requisitos previos

- **Node.js** v22.23.1 o superior
- **pnpm** 9.15.4 (gestor de paquetes)
- **Rust** (via [rustup](https://rustup.rs))
- Dependencias de sistema para Tauri (consulta [tauri.app/start/prerequisites](https://v2.tauri.app/start/prerequisites/))

## Instalación y arranque

```bash
git clone <repo-url>
cd mates
pnpm install
pnpm tauri dev
```

Esto arranca el servidor de desarrollo de Vite (puerto 1420) y la ventana de Tauri.

## Compilación

Para compilar un binario de producción:

```bash
pnpm tauri build
```

Para Android (APK):

```bash
pnpm android:init          # solo la primera vez
pnpm android:build         # build APK aarch64
pnpm android:build:all     # build para todas las targets
```

> **Nota:** El proyecto está en fase de desarrollo. No hay instalador empaquetado; los builds producen binarios funcionales para desarrollo y pruebas.

## Configuración de API keys

La app necesita varias claves según las funcionalidades que quieras usar. Ninguna es obligatoria para el funcionamiento básico con Ollama local.

### Proveedores LLM

Se configuran desde la **zona adulta → pestaña "IA / LLM"** y se guardan en SQLite. No usan el archivo `.env`.

#### Ollama (local/offline, sin API key)

Recomendado para empezar. No necesita internet ni claves.

1. Descarga e instala Ollama desde [ollama.com](https://ollama.com)
2. Instala un modelo: `ollama pull llama3` (o `llama3.2`, `phi4`, etc.)
3. En la app, configura:
   - **URL**: `http://localhost:11434`
   - **Modelo**: `llama3` (o el que hayas instalado)

#### Gemini (cloud)

Calidad alta, necesita conexión a internet.

1. Obtén una API key gratuita en [Google AI Studio](https://aistudio.google.com/apikey)
2. En la app, configura:
   - **Proveedor**: Gemini
   - **API key**: la clave de AI Studio
   - **Modelo**: `gemini-1.5-flash` (recomendado) o `gemini-2.0-flash`

#### OpenAI-compatible

Funciona con cualquier endpoint compatible con la API de OpenAI: LM Studio, OpenRouter, vLLM, llama.cpp server, etc.

- Si usas un servidor local (LM Studio, llama.cpp), no necesitas API key
- Si usas OpenRouter u otro servicio cloud, necesitas su API key

En la app, configura:
- **URL**: la de tu servidor (ej. `http://localhost:1234` para LM Studio)
- **API key**: la del servicio (opcional para local)
- **Modelo**: el nombre del modelo

### Sincronización cloud (Baserow)

La sincronización entre dispositivos usa [Baserow](https://baserow.io), una base de datos online de código abierto. Es **opcional**; sin ella la app funciona completamente local.

1. Crea una cuenta gratuita en [baserow.io](https://baserow.io)
2. Crea una base de datos (llámala "Mates")
3. Copia el **ID de la base de datos** de la URL (ej. `490809`)
4. Genera un **token de API** en [baserow.io/fr/profile/account](https://baserow.io/fr/profile/account)
5. Crea un archivo `.env` en la raíz del proyecto (o edita el existente):

```env
BASEROW_DATABASE_ID=490809
BASEROW_API_TOKEN=tu_token_aqui
BASEROW_API_URL=https://api.baserow.io/api
```

La sincronización es siempre bidireccional (last-writer-wins): cada operación sube primero los datos locales más recientes y luego descarga los remotos, comparando `updated_at` en formato RFC3339. Para sesiones activas (no borradas), gana siempre la versión local.

### Email transaccional (Brevo)

Brevo se usa para verificación de email y recuperación de contraseña en la capa de sincronización cloud. Es **opcional**; si no se configura, las funciones de email se desactivan silenciosamente.

1. Crea una cuenta en [brevo.com](https://brevo.com)
2. Genera una API key en **Settings → API Keys**
3. Añádela al `.env`:

```env
BREVO_API_KEY=tu_api_key_de_brevo
```

### Nota sobre el `.env`

Las variables del `.env` se cargan automáticamente al iniciar la app. Primero busca el archivo en el directorio de trabajo actual, y luego en los directorios del ejecutable, recursos o datos de la app. No es necesario hacer nada más.

## Uso general

### Zona infantil

1. Al arrancar, el niño selecciona su perfil
2. Inicia una sesión de 10 preguntas
3. Responde cada pregunta; la app evalúa la respuesta y da feedback inmediato
4. Si falla, recibe una explicación motivadora y una reformulación del concepto
5. Al finalizar, ve un resumen con aciertos, precisión y tiempo

### Zona adulta

Acceso protegido por PIN de 4 a 6 dígitos (se define en el primer arranque y se almacena con hash Argon2). Para resetear el PIN se borran todos los datos locales escribiendo `RESET` como confirmación. Contiene:

- **Perfiles**: crear, editar, eliminar y recuperar perfiles infantiles. Cada perfil tiene nombre, curso (1o-6o), edad opcional (6-12 años), nivel automático (nunca retrocede) o manual con contexto pedagógico personalizado (máx. 1000 caracteres)
- **IA / LLM**: configurar proveedor, modelo, URL y API key; probar la conexión
- **Historial**: sesiones realizadas por cada perfil
- **Dashboard**: estadísticas globales, análisis de conceptos (dominados/en progreso/necesitan práctica), evolución por sesión (gráfico de barras), exportación CSV y JSON
- **Profesional**: gestión de estudiantes, tutores, grupos, tareas y reportes
- **Nube**: registro, inicio de sesión, sincronización manual, auto-login al iniciar

## Arquitectura

```
mates/
├── src/                     # Frontend Svelte + TypeScript
│   ├── main.ts              # Entrypoint
│   ├── App.svelte           # Componente principal (~1100 líneas, sin router)
│   ├── app-state.svelte.ts  # Estado reactivo global
│   └── prototypes.ts        # Extensiones de prototype (toInt, toFloat)
├── src-tauri/               # Backend Rust
│   └── src/
│       ├── lib.rs           # Entrypoint Tauri (~1850 líneas)
│       ├── llm/             # Proveedores de IA
│       │   ├── ollama.rs    # OllamaProvider
│       │   ├── gemini.rs    # GeminiProvider
│       │   ├── openai_compatible.rs  # OpenAICompatibleProvider
│       │   └── common.rs    # Prompts y parsing compartido
│       ├── cloud/           # Sincronización Baserow
│       ├── email.rs         # Email transaccional (Brevo)
│       └── models.rs        # Modelos de datos SQLite
└── specs/                   # Especificaciones detalladas
```

## Comandos

| Comando | Descripción |
|---------|-------------|
| `pnpm dev` | Servidor de desarrollo Vite (puerto 1420) |
| `pnpm build` | Build del frontend (Vite) |
| `pnpm preview` | Preview del build de Vite |
| `pnpm check` | Type-check con svelte-check |
| `pnpm check:watch` | Type-check en modo watch |
| `pnpm tauri` | CLI de Tauri |
| `pnpm tauri:build` | Build de producción (Tauri) |
| `pnpm android:init` | Inicializar proyecto Android |
| `pnpm android:dev` | Servidor de desarrollo Android |
| `pnpm android:build` | Build APK (aarch64) |
| `pnpm android:build:signed` | Build APK firmado |
| `pnpm android:build:all` | Build Android para todas las targets |
| `pnpm version:bump:patch` | Incrementa versión patch (0.1.0 → 0.1.1) |
| `pnpm version:bump:minor` | Incrementa versión minor (0.1.0 → 0.2.0) |

## Licencia

**GPL-3.0**. Puedes usar, modificar y distribuir esta aplicación libremente, pero cualquier modificación derivada debe mantenerse bajo la misma licencia (copyleft).

## IDE recomendado

[VS Code](https://code.visualstudio.com/) + [Svelte](https://marketplace.visualstudio.com/items?itemName=svelte.svelte-vscode) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer).
