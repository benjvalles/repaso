# baserow-proxy

Proxy de Cloudflare Worker que inyecta las credenciales de Baserow y Brevo en las peticiones salientes de la app Mates. La app nunca conoce los tokens.

## Capas de seguridad

1. **X-Proxy-Key**: header con shared secret. El proxy valida contra `SHARED_SECRETS` (lista separada por comas).
2. **X-User-Id**: header con el ID de usuario. El proxy verifica que existe en la tabla de cuentas (1071739). Excepcion: lecturas de la tabla de cuentas sin user_id (login/registro).
3. **Token inyectado**: el proxy reemplaza el header de auth con el token real de Baserow/Brevo.

## Rutas

| Ruta        | Destino                      | Header inyectado              |
|-------------|------------------------------|-------------------------------|
| `/baserow/*` | `https://api.baserow.io/api` | `Authorization: Token ...`    |
| `/brevo/*`   | `https://api.brevo.com/v3`   | `api-key: ...`                |

Cualquier otra ruta responde 404. Metodo HTTP, body y query string se reenvian sin cambios.

## Desarrollo local

```sh
pnpm install
cp .dev.vars.example .dev.vars   # rellenar con tus tokens + SHARED_SECRETS
pnpm dev                         # escucha en http://localhost:8787
```

`.dev.vars` esta en `.gitignore`: no subir nunca los tokens.

## Despliegue

```sh
npx wrangler login
npx wrangler secret put BASEROW_API_TOKEN
npx wrangler secret put BREVO_API_KEY
npx wrangler secret put SHARED_SECRETS
pnpm wrangler:deploy
```

Tras desplegar, actualizar `PROXY_BASEROW_URL` y `PROXY_BREVO_URL` en `.env` y recompilar la app.

- **URL**: `https://baserow-proxy.baserow-proxy.workers.dev`
- **Version ID**: `ca000b52-5c82-4934-8b5a-e95b96c0e9f8`
