'use client'

import {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useMemo,
  useRef,
  useState,
  type ReactNode,
} from 'react'
import { getApiBaseUrl } from './api-config'

interface AuthState {
  token: string | null
  expiresAt: number | null
}

interface AuthContextValue {
  /** Current JWT token (null when unauthenticated) */
  token: string | null
  /** Whether the initial session restoration is still in progress */
  isLoading: boolean
  /** Whether the user is authenticated */
  isAuthenticated: boolean
  /** Store a new JWT and its expiry */
  setAuth: (jwt: string, expiresAt: number) => void
  /** Clear authentication state */
  clearAuth: () => void
  /** Get the current token (convenience for WebSocket connections etc.) */
  getToken: () => string | null
}

const AuthContext = createContext<AuthContextValue | null>(null)

const JWT_COOKIE_NAME = 'jwt'

/** Write a non-httpOnly cookie so server-side code (server actions) can read the JWT */
function setJwtCookie(token: string, expiresAt: number) {
  const expires = new Date(expiresAt * 1000).toUTCString()
  document.cookie = `${JWT_COOKIE_NAME}=${token}; path=/; expires=${expires}; SameSite=Lax`
}

/** Remove the JWT cookie */
function clearJwtCookie() {
  document.cookie = `${JWT_COOKIE_NAME}=; path=/; expires=Thu, 01 Jan 1970 00:00:00 GMT; SameSite=Lax`
}

/** Read the JWT cookie value (client-side) */
function readJwtCookie(): string | null {
  if (typeof document === 'undefined') return null
  const regex = new RegExp(`(?:^|; )${JWT_COOKIE_NAME}=([^;]*)`)
  const match = regex.exec(document.cookie)
  return match ? decodeURIComponent(match[1]) : null
}

export function AuthProvider({ children }: Readonly<{ children: ReactNode }>) {
  const [auth, setAuthState] = useState<AuthState>({ token: null, expiresAt: null })
  const [isLoading, setIsLoading] = useState(true)
  const refreshTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null)

  const clearAuth = useCallback(() => {
    setAuthState({ token: null, expiresAt: null })
    clearJwtCookie()
    if (refreshTimerRef.current) {
      clearTimeout(refreshTimerRef.current)
      refreshTimerRef.current = null
    }
  }, [])

  const scheduleRefresh = useCallback(
    (expiresAt: number, currentToken: string) => {
      if (refreshTimerRef.current) {
        clearTimeout(refreshTimerRef.current)
      }

      const now = Date.now() / 1000
      const ttl = expiresAt - now
      // Refresh at 80% of TTL, minimum 10 seconds before expiry
      const refreshIn = Math.max(ttl * 0.8, ttl - 10) * 1000

      if (refreshIn <= 0) {
        // Token already expired or about to
        clearAuth()
        return
      }

      refreshTimerRef.current = setTimeout(async () => {
        try {
          const baseUrl = getApiBaseUrl()
          const response = await fetch(`${baseUrl}/api/auth/renew`, {
            method: 'POST',
            headers: {
              'Content-Type': 'application/json',
              Authorization: `Bearer ${currentToken}`,
            },
          })

          if (response.ok) {
            const data = await response.json()
            if (data.jwt && data.token_expires_at) {
              setAuthState({ token: data.jwt, expiresAt: data.token_expires_at })
              setJwtCookie(data.jwt, data.token_expires_at)
              scheduleRefresh(data.token_expires_at, data.jwt)
            } else {
              clearAuth()
            }
          } else {
            clearAuth()
          }
        } catch {
          clearAuth()
        }
      }, refreshIn)
    },
    [clearAuth]
  )

  const setAuth = useCallback(
    (jwt: string, expiresAt: number) => {
      setAuthState({ token: jwt, expiresAt })
      setJwtCookie(jwt, expiresAt)
      scheduleRefresh(expiresAt, jwt)
    },
    [scheduleRefresh]
  )

  const getToken = useCallback(() => {
    return auth.token
  }, [auth.token])

  // On mount: try to restore session from existing token in cookie
  useEffect(() => {
    async function restoreSession() {
      try {
        // First check if we have a JWT cookie
        const existingToken = readJwtCookie()
        if (!existingToken) {
          setIsLoading(false)
          return
        }

        // Try to renew using the existing token
        const baseUrl = getApiBaseUrl()
        const response = await fetch(`${baseUrl}/api/auth/renew`, {
          method: 'POST',
          headers: {
            'Content-Type': 'application/json',
            Authorization: `Bearer ${existingToken}`,
          },
        })

        if (response.ok) {
          const data = await response.json()
          if (data.jwt && data.token_expires_at) {
            setAuthState({ token: data.jwt, expiresAt: data.token_expires_at })
            setJwtCookie(data.jwt, data.token_expires_at)
            scheduleRefresh(data.token_expires_at, data.jwt)
          }
        } else {
          clearJwtCookie()
        }
      } catch {
        clearJwtCookie()
      } finally {
        setIsLoading(false)
      }
    }

    restoreSession()
  }, [scheduleRefresh])

  // Cleanup on unmount
  useEffect(() => {
    return () => {
      if (refreshTimerRef.current) {
        clearTimeout(refreshTimerRef.current)
      }
    }
  }, [])

  const contextValue = useMemo<AuthContextValue>(() => ({
    token: auth.token,
    isLoading,
    isAuthenticated: !!auth.token,
    setAuth,
    clearAuth,
    getToken,
  }), [auth.token, isLoading, setAuth, clearAuth, getToken])

  return (
    <AuthContext value={contextValue}>
      {children}
    </AuthContext>
  )
}

export function useAuth(): AuthContextValue {
  const context = useContext(AuthContext)
  if (!context) {
    throw new Error('useAuth must be used within an AuthProvider')
  }
  return context
}

/** Module-level token reference for the Orval client (set by AuthProvider internals) */
let _moduleToken: string | null = null

export function setModuleToken(token: string | null) {
  _moduleToken = token
}

export function getModuleToken(): string | null {
  return _moduleToken
}
