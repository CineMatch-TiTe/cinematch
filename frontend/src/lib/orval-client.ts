const JWT_COOKIE_NAME = 'jwt'

/**
 * Read the JWT from:
 * 1. Module-level token (set by AuthProvider on the client)
 * 2. Cookie (for server-side rendering / server actions)
 */
async function resolveToken(): Promise<string | null> {
  // Client-side: read from cookie directly
  if (globalThis.window !== undefined) {
    const match = document.cookie.match(new RegExp(`(?:^|; )${JWT_COOKIE_NAME}=([^;]*)`))
    return match ? decodeURIComponent(match[1]) : null
  }

  // Server-side: read from next/headers cookies
  try {
    // Use require to hide the import from Orval's static AST parser
    // eslint-disable-next-line @typescript-eslint/ban-ts-comment
    // @ts-ignore
    // eslint-disable-next-line @typescript-eslint/no-require-imports
    const { cookies } = require('next/headers')
    const cookieStore = await cookies()
    const jwtCookie = cookieStore.get(JWT_COOKIE_NAME)
    return jwtCookie?.value ?? null
  } catch {
    return null
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

  const reqHeaders: HeadersInit = {
    'Content-Type': 'application/json',
    ...(headers as Record<string, string>)
  }

  // Add JWT Bearer token
  const token = await resolveToken()
  if (token) {
    ;(reqHeaders)['Authorization'] = `Bearer ${token}`
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
