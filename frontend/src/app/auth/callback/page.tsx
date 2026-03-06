'use client'

import { Suspense, useEffect } from 'react'
import { useRouter, useSearchParams } from 'next/navigation'
import { useAuth } from '@/lib/auth-context'
import { Loader2 } from 'lucide-react'

function OAuthCallbackContent() {
  const searchParams = useSearchParams()
  const router = useRouter()
  const { setAuth } = useAuth()

  useEffect(() => {
    const jwt = searchParams?.get('jwt')
    const expiresAt = searchParams?.get('expires_at')

    if (jwt && expiresAt) {
      setAuth(jwt, Number.parseInt(expiresAt, 10))
      router.replace('/dashboard')
    } else {
      // No token provided — redirect to home
      router.replace('/')
    }
  }, [searchParams, setAuth, router])

  return (
    <div className="flex items-center justify-center min-h-screen bg-zinc-950">
      <div className="flex flex-col items-center gap-4">
        <Loader2 className="h-8 w-8 animate-spin text-red-500" />
        <p className="text-zinc-400">Completing login...</p>
      </div>
    </div>
  )
}

/**
 * OAuth callback page.
 * The backend redirects here with JWT details as query parameters after successful OAuth login.
 * URL format: /auth/callback?jwt=<token>&expires_at=<unix_ts>
 */
export default function OAuthCallbackPage() {
  return (
    <Suspense
      fallback={
        <div className="flex items-center justify-center min-h-screen bg-zinc-950">
          <div className="flex flex-col items-center gap-4">
            <Loader2 className="h-8 w-8 animate-spin text-red-500" />
            <p className="text-zinc-400">Loading...</p>
          </div>
        </div>
      }
    >
      <OAuthCallbackContent />
    </Suspense>
  )
}
