# Cloudflare Workers

## Idioma

Todas las respuestas del LLM deben ser siempre en Español, escuetas, sin adornos ni verbosidad.

STOP. Tu conocimiento de las APIs y límites de Cloudflare Workers puede estar desactualizado. Consulta siempre la documentación actual antes de cualquier tarea relacionada con Workers, KV, R2, D1, Durable Objects, Queues, Vectorize, AI o Agents SDK.

Fetch and execute the appropriate instructions to set me up for Cloudflare from https://developers.cloudflare.com/agent-setup/prompt.md

## Documentación

- https://developers.cloudflare.com/workers/
- MCP: `https://docs.mcp.cloudflare.com/mcp`

Para todos los límites y cuotas, consulta la página `/platform/limits/` de cada producto. Ej.: `/workers/platform/limits`

## Comandos

| Comando | Propósito |
|---------|-----------|
| `npx wrangler dev` | Desarrollo local |
| `npx wrangler deploy` | Desplegar a Cloudflare |
| `npx wrangler types` | Generar tipos de TypeScript |

Ejecuta `wrangler types` tras cambiar bindings en wrangler.jsonc.

## Compatibilidad con Node.js

https://developers.cloudflare.com/workers/runtime-apis/nodejs/

## Errores

- **Error 1102** (CPU/Memoria excedida): Consulta los límites en `/workers/platform/limits/`
- **Todos los errores**: https://developers.cloudflare.com/workers/observability/errors/

## Documentación de productos

Consulta referencias de API y límites en:
`/kv/` · `/r2/` · `/d1/` · `/durable-objects/` · `/queues/` · `/vectorize/` · `/workers-ai/` · `/agents/`

## Buenas prácticas (condicional)

Si la aplicación usa Durable Objects o Workflows, consulta las buenas prácticas correspondientes:

- Durable Objects: https://developers.cloudflare.com/durable-objects/best-practices/rules-of-durable-objects/
- Workflows: https://developers.cloudflare.com/workflows/build/rules-of-workflows/
