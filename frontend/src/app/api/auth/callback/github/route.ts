import { cookies } from 'next/headers'
import { redirect } from 'next/navigation'
import type { NextRequest } from 'next/server'

/**
 * GitHub OAuth callback route handler.
 *
 * GitHub redirects here with `code` and `state` query params.
 * This handler forwards them to the backend, extracts the JWT from the
 * response, sets the JWT cookie, and redirects to the dashboard.
 *
 * This keeps the backend response hidden from the browser.
 */
export async function GET(request: NextRequest) {
  const searchParams = request.nextUrl.searchParams
  const code = searchParams.get('code')
  const state = searchParams.get('state')

  if (!code) {
    redirect('/?error=missing_code')
  }

  const baseUrl = process.env.NEXT_PUBLIC_API_BASE || 'http://localhost:8085'

  // Build the backend callback URL with the same query params
  const backendUrl = new URL(`${baseUrl}/api/auth/callback/github`)
  backendUrl.searchParams.set('code', code)
  if (state) {
    backendUrl.searchParams.set('state', state)
  }

  try {
    const response = await fetch(backendUrl.toString(), {
      method: 'GET',
      headers: {
        'Content-Type': 'application/json',
      },
      // Forward cookies in case the backend uses them for account linking
      credentials: 'include',
    })

    if (!response.ok) {
      console.error('[OAuth Callback] Backend returned', response.status)
      redirect('/?error=oauth_failed')
    }

    const data = await response.json()

    if (!data.jwt || !data.expires_at) {
      console.error('[OAuth Callback] Missing JWT in backend response')
      redirect('/?error=oauth_failed')
    }

    // Set the JWT cookie
    const cookieStore = await cookies()
    const expiresDate = new Date(data.expires_at * 1000)
    cookieStore.set('jwt', data.jwt, {
      path: '/',
      expires: expiresDate,
      sameSite: 'lax',
    })
  } catch (error) {
    console.error('[OAuth Callback] Error', error)
    redirect('/?error=oauth_failed')
  }

  redirect('/dashboard')
}
