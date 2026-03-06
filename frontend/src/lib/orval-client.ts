const JWT_COOKIE_NAME = 'jwt'

/**
 * Resolve auth headers for the request:
 * - Server-side: forward all cookies so the backend can read the `jwt` cookie directly.
 * - Client-side: read the JWT from document.cookie and send as Authorization header
 *   (browsers forbid setting the Cookie header on fetch).
 */
async function resolveAuthHeaders(): Promise<Record<string, string>> {
  // Client-side: send JWT as Bearer token
  if (globalThis.window !== undefined) {
    const match = document.cookie.match(new RegExp(`(?:^|; )${JWT_COOKIE_NAME}=([^;]*)`))
    const token = match ? decodeURIComponent(match[1]) : null
    return token ? { Authorization: `Bearer ${token}` } : {}
  }

  // Server-side: forward all cookies from the incoming request
  try {
    // Use require to hide the import from Orval's static AST parser
    // eslint-disable-next-line @typescript-eslint/ban-ts-comment
    // @ts-ignore
    // eslint-disable-next-line @typescript-eslint/no-require-imports
    const { cookies } = require('next/headers')
    const cookieStore = await cookies()
    const allCookies = cookieStore.getAll() as { name: string; value: string }[]
    if (allCookies.length > 0) {
      const cookieHeader = allCookies.map((c) => `${c.name}=${c.value}`).join('; ')
      return { Cookie: cookieHeader }
    }
    return {}
  } catch {
    return {}
  }
}

export const customInstance = async <T>(
  url: string,
  options: RequestInit & { params?: Record<string, string> } = {}
): Promise<T> => {
  const baseUrl = process.env.NEXT_PUBLIC_API_BASE || 'https://api.cinematch.space'

  // Resolve absolute URL
  const absoluteUrl = url.startsWith('http') ? url : `${baseUrl}${url}`

  const { params, headers, ...rest } = options

  // Construct search params
  const searchParams = new URLSearchParams(params)
  const queryString = searchParams.toString()
  const finalUrl = absoluteUrl + (queryString ? '?' + queryString : '')

  // Auth headers come before caller headers so explicit Authorization (e.g. onboarding) wins
  const authHeaders = await resolveAuthHeaders()
  const reqHeaders: HeadersInit = {
    'Content-Type': 'application/json',
    ...authHeaders,
    ...(headers as Record<string, string>),
  }

  const config: RequestInit = {
    ...rest,
    headers: reqHeaders,
  }

  const response = await fetch(finalUrl, config)

  // Helper to handle body parsing safely
  let data: unknown
  const contentType = response.headers.get('content-type')
  if (contentType?.includes('application/json')) {
    try {
      data = await response.json()
    } catch {
      data = {}
    }
  } else {
    // Attempt to handle text response if JSON fails or not indicated
    try {
      const text = await response.text()
      data = text
    } catch {
      data = {}
    }
  }

  return { data, status: response.status, headers: response.headers } as T
}

export default customInstance
