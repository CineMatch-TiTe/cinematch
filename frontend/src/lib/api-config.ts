/**
 * Resolves the appropriate API base URL based on the environment.
 * In Next.js App Router:
 * - Server Components, Server Actions, and API Routes run on the server.
 *   They can access the internal docker network via NEXT_PRIVATE_API_BASE.
 * - Client Components run in the browser and must use the public URL via NEXT_PUBLIC_API_BASE.
 */
export function getApiBaseUrl(): string {
  if (globalThis.window === undefined) {
    // Server-side
    return process.env.NEXT_PRIVATE_API_BASE || process.env.NEXT_PUBLIC_API_BASE || 'http://localhost:8085';
  }
  // Client-side
  return process.env.NEXT_PUBLIC_API_BASE || 'http://localhost:8085';
}
