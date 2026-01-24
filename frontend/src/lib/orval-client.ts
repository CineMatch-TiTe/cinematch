export const customInstance = async <T>({
  url,
  method,
  headers,
  params,
  data
}: {
  url: string
  method: 'GET' | 'POST' | 'PUT' | 'DELETE' | 'PATCH'
  headers?: HeadersInit
  params?: Record<string, string>
  data?: unknown
}): Promise<{ data: T; status: number; headers: Headers }> => {
  const baseUrl = process.env.NEXT_PUBLIC_API_BASE || 'http://localhost:8085'

  // Resolve absolute URL
  const absoluteUrl = url.startsWith('http') ? url : `${baseUrl}${url}`

  // Construct search params
  const searchParams = new URLSearchParams(params)
  const finalUrl = `${absoluteUrl}${searchParams.toString() ? `?${searchParams.toString()}` : ''}`

  const config: RequestInit = {
    method,
    headers: {
      'Content-Type': 'application/json',
      ...(headers as Record<string, string>)
    },
    ...(data ? { body: JSON.stringify(data) } : {})
  }

  const response = await fetch(finalUrl, config)

  // Helper to handle body parsing safely
  let responseData: T
  const contentType = response.headers.get('content-type')
  if (contentType && contentType.indexOf('application/json') !== -1) {
    try {
      responseData = await response.json()
    } catch {
      responseData = {} as T
    }
  } else {
    // Attempt to handle text response if JSON fails or not indicated
    try {
      const text = await response.text()
      // If generic T is expected to be object, this might fail typing at runtime if T is strictly object, but for Orval it usually maps well.
      responseData = text as unknown as T
    } catch {
      responseData = {} as T
    }
  }

  // Handle errors generally if needed, but Orval usually expects { data, status, headers } to throw if error status?
  // Actually Orval generated code usually handles the status check (Is 2xx or not) inside the generated function *if* we just return the raw response,
  // BUT with a custom instance, we return the object and Orval types expect `Promise<T>` where T is the response type.
  // Wait, looking at current generated code:
  /*
    const res = await fetch(...)
    const body = ...
    return { data, status: res.status, headers: res.headers }
  */
  // So our custom instance should return exactly that structure.

  return { data: responseData, status: response.status, headers: response.headers }
}
export default customInstance
