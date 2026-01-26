export const customInstance = async <T>(
  url: string,
  options: RequestInit & { params?: Record<string, string> } = {}
): Promise<T> => {
  const baseUrl = process.env.NEXT_PUBLIC_API_BASE || 'http://localhost:8085'

  // Resolve absolute URL
  const absoluteUrl = url.startsWith('http') ? url : `${baseUrl}${url}`

  const { params, headers, ...rest } = options

  // Construct search params
  const searchParams = new URLSearchParams(params)
  const finalUrl = `${absoluteUrl}${searchParams.toString() ? `?${searchParams.toString()}` : ''}`

  const reqHeaders: HeadersInit = {
    'Content-Type': 'application/json',
    ...(headers as Record<string, string>)
  }

  // Server-side cookie forwarding
  if (typeof window === 'undefined') {
    const { cookies } = await import('next/headers')
    const cookieStore = await cookies()
    const allCookies = cookieStore
      .getAll()
      .map((c) => `${c.name}=${c.value}`)
      .join('; ')

    if (allCookies) {
      const existingCookie = reqHeaders['Cookie'] as string | undefined
      if (existingCookie) {
        reqHeaders['Cookie'] = `${existingCookie}; ${allCookies}`
      } else {
        reqHeaders['Cookie'] = allCookies
      }
    }
  }

  const config: RequestInit = {
    ...rest,
    headers: reqHeaders,
    credentials: 'include'
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
      // If generic T is expected to be object, this might fail typing at runtime if T is strictly object, but for Orval it usually maps well.
      data = text
    } catch {
      data = {}
    }
  }

  return { data, status: response.status, headers: response.headers } as T
}

export default customInstance
