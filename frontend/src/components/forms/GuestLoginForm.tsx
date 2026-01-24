'use client'

import { useActionState } from 'react'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Alert, AlertDescription } from '@/components/ui/alert'
import { User, Lock, Loader2 } from 'lucide-react'
import { guestLoginAction } from '@/app/actions'

const initialState = {
  message: '',
  errors: undefined
}

export function GuestLoginForm({ initialJoinCode }: Readonly<{ initialJoinCode?: string }>) {
  const [state, formAction, isPending] = useActionState(guestLoginAction, initialState)

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
              className={`pl-10 bg-zinc-950/50 border-zinc-700 text-zinc-100 placeholder:text-zinc-500 focus-visible:ring-red-600 focus-visible:border-red-600 ${
                state?.errors?.username ? 'border-red-500 focus-visible:ring-red-500' : ''
              }`}
            />
          </div>
          {state?.errors?.username && (
            <p className="text-xs text-red-500 pl-1">{state.errors.username[0]}</p>
          )}
        </div>

        <div className="space-y-1">
          <div className="relative">
            <Lock className="absolute left-3 top-1/2 -translate-y-1/2 h-5 w-5 text-zinc-500" />
            <Input
              name="joinCode"
              placeholder="Join code"
              defaultValue={initialJoinCode || ''}
              disabled={!!initialJoinCode}
              className={`pl-10 bg-zinc-950/50 border-zinc-700 text-zinc-100 placeholder:text-zinc-500 focus-visible:ring-red-600 focus-visible:border-red-600 ${
                state?.errors?.joinCode ? 'border-red-500 focus-visible:ring-red-500' : ''
              } ${initialJoinCode ? 'opacity-50 cursor-not-allowed' : ''}`}
            />
            {initialJoinCode && <input type="hidden" name="joinCode" value={initialJoinCode} />}
          </div>
          {state?.errors?.joinCode && (
            <p className="text-xs text-red-500 pl-1">{state.errors.joinCode[0]}</p>
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
            Joining...
          </>
        ) : (
          'Join as guest'
        )}
      </Button>
    </form>
  )
}
