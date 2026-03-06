'use client'

import { useActionState, useEffect, useState } from 'react'
import { useRouter } from 'next/navigation'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Alert, AlertDescription } from '@/components/ui/alert'
import { User, Loader2, LogIn } from 'lucide-react'
import { loginAction, type OnboardingActionResult } from '@/actions/onboarding'
import { useAuth } from '@/lib/auth-context'

const initialState: OnboardingActionResult = {
  message: '',
  errors: null
}

export function LoginForm() {
  const [state, formAction, isPending] = useActionState(loginAction, initialState)
  const [isGithubLoading, setIsGithubLoading] = useState(false)
  const { setAuth } = useAuth()
  const router = useRouter()

  useEffect(() => {
    if (state?.auth && state?.redirectTo) {
      setAuth(state.auth.jwt, state.auth.expiresAt)
      router.push(state.redirectTo)
    }
  }, [state, setAuth, router])

  const handleGithubLogin = () => {
    setIsGithubLoading(true)
    const baseUrl = process.env.NEXT_PUBLIC_API_BASE || 'https://api.cinematch.space'
    globalThis.location.href = `${baseUrl}/api/auth/login/github`
  }

  return (
    <form action={formAction} className="space-y-4">
      {state?.message && state.message !== 'Validation failed' && (
        <Alert
          variant="destructive"
          className="bg-red-500/10 border-red-500/50 text-red-200 [&>svg]:text-red-200"
        >
          <AlertDescription className="text-red-200/90">{state.message}</AlertDescription>
        </Alert>
      )}
      <div className="space-y-4">
        <div className="space-y-1">
          <div className="relative">
            <User className="absolute left-3 top-1/2 -translate-y-1/2 h-5 w-5 text-zinc-500" />
            <Input
              name="username"
              placeholder="Your name"
              defaultValue=""
              className={`pl-10 bg-zinc-950/50 border-zinc-700 text-zinc-100 placeholder:text-zinc-500 focus-visible:ring-red-600 focus-visible:border-red-600 ${state?.errors?.username ? 'border-red-500 focus-visible:ring-red-500' : ''
                }`}
            />
          </div>
          {state?.errors?.username && (
            <p className="text-xs text-red-500 pl-1">{state.errors.username[0]}</p>
          )}
        </div>
      </div>

      <Button
        type="submit"
        disabled={isPending}
        className="w-full bg-red-900 hover:bg-red-800 text-white font-semibold py-6 text-lg shadow-[0_0_15px_rgba(153,27,27,0.5)] transition-all hover:shadow-[0_0_25px_rgba(153,27,27,0.6)] disabled:opacity-50 disabled:cursor-not-allowed"
      >
        {isPending ? (
          <>
            <Loader2 className="mr-2 h-5 w-5 animate-spin" />
            Logging in...
          </>
        ) : (
          <>
            <LogIn className="mr-2 h-5 w-5" />
            Login
          </>
        )}
      </Button>

      <div className="relative py-2">
        <div className="absolute inset-0 flex items-center">
          <span className="w-full border-t border-zinc-800" />
        </div>
        <div className="relative flex justify-center text-xs uppercase">
          <span className="bg-zinc-900 px-2 text-zinc-500">Or continue with</span>
        </div>
      </div>

      <Button
        type="button"
        variant="outline"
        className="w-full border-zinc-700 bg-zinc-950 text-zinc-200 hover:bg-zinc-900 hover:text-white py-6 disabled:opacity-50 disabled:cursor-not-allowed"
        onClick={handleGithubLogin}
        disabled={isPending || isGithubLoading}
      >
        {isGithubLoading ? (
          <>
            <Loader2 className="mr-2 h-5 w-5 animate-spin" />
            Logging in...
          </>
        ) : (
          <>
            <svg className="mr-2 h-5 w-5" viewBox="0 0 24 24" fill="currentColor"><path d="M12 0c-6.626 0-12 5.373-12 12 0 5.302 3.438 9.8 8.207 11.387.599.111.793-.261.793-.577v-2.234c-3.338.726-4.033-1.416-4.033-1.416-.546-1.387-1.333-1.756-1.333-1.756-1.089-.745.083-.729.083-.729 1.205.084 1.839 1.237 1.839 1.237 1.07 1.834 2.807 1.304 3.492.997.107-.775.418-1.305.762-1.604-2.665-.305-5.467-1.334-5.467-5.931 0-1.311.469-2.381 1.236-3.221-.124-.303-.535-1.524.117-3.176 0 0 1.008-.322 3.301 1.23.957-.266 1.983-.399 3.003-.404 1.02.005 2.047.138 3.006.404 2.291-1.552 3.297-1.23 3.297-1.23.653 1.653.242 2.874.118 3.176.77.84 1.235 1.911 1.235 3.221 0 4.609-2.807 5.624-5.479 5.921.43.372.823 1.102.823 2.222v3.293c0 .319.192.694.801.576 4.765-1.589 8.199-6.086 8.199-11.386 0-6.627-5.373-12-12-12z"/></svg>
            GitHub
          </>
        )}
      </Button>
    </form>
  )
}
