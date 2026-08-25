interface Env {
  BASEROW_API_TOKEN: string
  BREVO_API_KEY: string
  SHARED_SECRETS: string
}

interface Target {
  base: string
  header: string
  value: (env: Env) => string
}

const TARGETS: Record<string, Target> = {
  baserow: {
    base: "https://api.baserow.io/api",
    header: "Authorization",
    value: (env) => `Token ${env.BASEROW_API_TOKEN}`,
  },
  brevo: {
    base: "https://api.brevo.com/v3",
    header: "api-key",
    value: (env) => env.BREVO_API_KEY,
  },
}

/// ID de la tabla de cuentas en Baserow.
const TABLE_ACCOUNTS = 1071739

/// Verifica que el user_id existe en la tabla de cuentas.
async function validateUser(userId: string, token: string): Promise<boolean> {
  const url = `https://api.baserow.io/api/database/rows/table/${TABLE_ACCOUNTS}/${userId}/`
  const resp = await fetch(url, {
    headers: {
      Authorization: `Token ${token}`,
      "accept-encoding": "identity",
    },
  })
  return resp.ok
}

export default {
  async fetch(request: Request, env: Env): Promise<Response> {
    const url = new URL(request.url)
    const prefix = url.pathname.split("/")[1]
    const target = TARGETS[prefix]
    if (!target || !url.pathname.startsWith(`/${prefix}/`)) {
      return new Response("Not Found", { status: 404 })
    }

    // Capa 1: validar X-Proxy-Key contra la lista de secretos válidos
    const proxyKey = request.headers.get("X-Proxy-Key")
    const validKeys = env.SHARED_SECRETS.split(",").map((k) => k.trim())
    if (!proxyKey || !validKeys.includes(proxyKey)) {
      return new Response("Unauthorized: invalid proxy key", { status: 401 })
    }

    // Capa 2: validar X-User-Id (solo para peticiones a Baserow)
    if (prefix === "baserow") {
      const userId = request.headers.get("X-User-Id")
      const isGet = request.method === "GET" || request.method === "HEAD"
      const isAccountsTable = url.pathname.includes(`/table/${TABLE_ACCOUNTS}/`)

      if (isGet && isAccountsTable) {
        // Permitir lecturas de la tabla de cuentas sin user_id (login/registro)
      } else if (!userId) {
        return new Response("Unauthorized: missing X-User-Id", { status: 401 })
      } else {
        const valid = await validateUser(userId, env.BASEROW_API_TOKEN)
        if (!valid) {
          return new Response("Unauthorized: invalid user", { status: 401 })
        }
      }
    }

    // Capa 3: inyectar token y proxyar
    const path = url.pathname.slice(prefix.length + 1)
    const headers = new Headers(request.headers)
    headers.delete("host")
    headers.delete("x-proxy-key")
    headers.delete("x-user-id")
    headers.set("accept-encoding", "identity")
    headers.set(target.header, target.value(env))

    const body =
      request.method === "GET" || request.method === "HEAD" ? undefined : request.body

    const upstream = await fetch(`${target.base}${path}${url.search}`, {
      method: request.method,
      headers,
      body,
    })

    return new Response(upstream.body, upstream)
  },
} satisfies ExportedHandler<Env>
