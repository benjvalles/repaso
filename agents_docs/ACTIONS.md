# GitHub Actions — Build Desktop & Android

## Workflow

**Archivo**: `.github/workflows/build-desktop.yml`

### Triggers

| Trigger | Descripción |
|---------|-------------|
| `workflow_dispatch` | Ejecución manual desde la web de GitHub o CLI (`gh workflow run build-desktop.yml`) |
| `push tags v*` | Push de un tag que empiece por `v` (ej: `v0.2.1`). Crea release automáticamente. |

### Concurrency

Cancela ejecuciones previas del mismo grupo para evitar builds duplicados.

---

## Jobs

### 1. `build` — Desktop (Linux, Windows, macOS)

**Plataformas** (matrix):

| Platform | OS | Target Rust |
|----------|----|-------------|
| ubuntu-22.04 | ubuntu-22.04 | x86_64-unknown-linux-gnu |
| windows | windows-latest | x86_64-pc-windows-msvc |
| macos | macos-latest | x86_64-apple-darwin |

**Pasos**:
1. Checkout
2. Crear `src-tauri/.env` con 3 variables desde GitHub Secrets
3. Instalar dependencias Linux (solo ubuntu)
4. Instalar pnpm (lee versión de `package.json`)
5. Instalar Node.js 22
6. Instalar Rust stable + target
7. `pnpm install --frozen-lockfile`
8. `pnpm check` (type-check)
9. `pnpm tauri build`
10. Upload artifact: `src-tauri/target/release/bundle/`

### 2. `build-android` — Android APK

**Runner**: ubuntu-22.04

**Pasos**:
1. Checkout
2. Crear `src-tauri/.env` con 3 variables desde GitHub Secrets
3. Decodificar keystore desde `ANDROID_KEYSTORE_BASE64` → `/tmp/keystore.jks`
4. Instalar pnpm, Node.js 22, Rust stable
5. Instalar JDK 17 (temurin) — **obligatorio** para sdkmanager
6. Instalar Android SDK + NDK 27
7. `pnpm install --frozen-lockfile`
8. `pnpm check`
9. `pnpm tauri android build --target aarch64 --apk` con variables de firma
10. Upload APK

**Importante**: No usar `pnpm android:build:signed` en CI porque intenta leer `android-signing.env` (gitignored).

### 3. `create-release` — GitHub Release

**Condición**: Solo se ejecuta en tags `v*`.
**Dependencias**: `build` + `build-android`

**Pasos**:
1. Download todos los artifacts (`mates-*`)
2. Crear release con `softprops/action-gh-release`

---

## Secrets de GitHub

### Para build de la app (desktop + Android)

| Secret | Descripción |
|--------|-------------|
| `SHARED_SECRETS` | Shared secret para autenticar contra el proxy |
| `PROXY_BASEROW_URL` | URL del proxy para Baserow |
| `PROXY_BREVO_URL` | URL del proxy para Brevo |

### Solo para Android

| Secret | Descripción | Generar con |
|--------|-------------|-------------|
| `ANDROID_KEYSTORE_BASE64` | Keystore de firma en Base64 | `base64 -w 0 keystore.jks` |
| `ANDROID_KEYSTORE_PASSWORD` | Password del keystore | — |
| `ANDROID_KEY_ALIAS` | Alias de la clave | — |
| `ANDROID_KEY_PASSWORD` | Password de la clave | — |

### No necesarios en GitHub Actions

| Secret | Por que no |
|--------|-----------|
| `BASEROW_DATABASE_ID` | No se usa en `env!()` ni `build.rs` |
| `BASEROW_API_TOKEN` | Lo gestiona el proxy (Cloudflare Worker) |
| `BREVO_API_KEY` | Lo gestiona el proxy (Cloudflare Worker) |

---

## Variables de entorno y el archivo `.env`

### Flujo de inyección

```
.github/workflows/build-desktop.yml
  ↓ (creates src-tauri/.env)
src-tauri/build.rs
  ↓ (lee .env, exporta 3 vars como cargo:rustc-env)
src-tauri/src/cloud/baserow.rs  → env!("PROXY_BASEROW_URL"), env!("SHARED_SECRETS")
src-tauri/src/email.rs          → env!("PROXY_BREVO_URL")
  ↓ (tauri_build::build() lee bundle.resources)
src-tauri/tauri.conf.json       → "resources": [".env"] (embebe .env en el binario)
```

### Variables exportadas por build.rs

Solo 3 variables se compilan en el binario:

```rust
let vars = ["SHARED_SECRETS", "PROXY_BASEROW_URL", "PROXY_BREVO_URL"];
```

### Por que `shell: bash` en "Create .env file"

El heredoc `<< EOF` no funciona en PowerShell (Windows). Se fuerza `shell: bash` para que funcione en las 3 plataformas.

---

## Errores conocidos y soluciones

### 1. `Multiple versions of pnpm specified`

**Causa**: `version: 9` en `pnpm/action-setup` + `"packageManager": "pnpm@9.15.4"` en `package.json`.

**Solucion**: No especificar `version` en `pnpm/action-setup`. Lee la version de `package.json`.

### 2. `resource path .env doesn't exist`

**Causa**: `tauri.conf.json` tiene `"resources": [".env"]` pero el archivo no existe.

**Solucion**: Anadir paso "Create .env file" antes del build.

### 3. `This tool requires JDK 17 or later`

**Causa**: `sdkmanager` necesita JDK 17+. El runner tiene JDK 11.

**Solucion**: Anadir `actions/setup-java@v4` con `java-version: 17` antes de `android-actions/setup-android`.

### 4. `Missing file specification after redirection operator` (Windows)

**Causa**: PowerShell no soporta heredocs `<< EOF`.

**Solucion**: Anadir `shell: bash` al paso.

### 5. `android-signing.env: No such file or directory`

**Causa**: `pnpm android:build:signed` ejecuta `source ./android-signing.env` (gitignored).

**Solución**: Usar `pnpm tauri android build --target aarch64 --apk` directamente. Las variables de entorno ya están configuradas en el `env:` del workflow.

---

## Como ejecutar el workflow

### Desde la web de GitHub
1. Ir a `Actions` → `Build Desktop`
2. Click `Run workflow`
3. Seleccionar rama
4. Click `Run workflow`

### Desde la CLI
```bash
gh workflow run build-desktop.yml
gh workflow run build-desktop.yml --ref main
```

### Push de tag (genera release)
```bash
git tag v0.2.1
git push origin v0.2.1
```

### Ver progreso
```bash
gh run list --workflow=build-desktop.yml
gh run watch
```

---

## Artefactos generados

| Artifact | Contenido |
|----------|-----------|
| `mates-ubuntu-22.04` | `.deb`, `.AppImage` |
| `mates-windows` | `.msi`, `.exe` (NSIS) |
| `mates-macos` | `.dmg`, `.app` |
| `mates-android` | `.apk` (aarch64) |
