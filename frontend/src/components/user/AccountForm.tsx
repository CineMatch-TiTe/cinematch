'use client'

import { Loader2, User } from 'lucide-react'
import { CurrentUserResponse } from '@/model'
import { useRouter } from 'next/navigation'
import { useActionState, useEffect, useCallback, useState } from 'react'
import { renameUserAction, ActionState } from '@/actions/user'
import { toast } from 'sonner'
import { Label } from '@/components/ui/label'
import { Input } from '@/components/ui/input'
import { Button } from '@/components/ui/button'

const AccountForm = ({
  initialUser,
  onSuccess
}: {
  initialUser: CurrentUserResponse
  onSuccess: () => void
}) => {
  const router = useRouter()
  const [isGithubLoading, setIsGithubLoading] = useState(false)

  const handleLinkGithub = () => {
    setIsGithubLoading(true)
    const baseUrl = process.env.NEXT_PUBLIC_API_BASE || 'https://api.cinematch.space'
    globalThis.location.href = `${baseUrl}/api/auth/login/github`
  }
  // Bind userId to the action
  const renameUserWithId = useCallback(
    (prevState: ActionState | null, formData: FormData) =>
      renameUserAction(initialUser.user_id, prevState, formData),
    [initialUser.user_id]
  )
  const [state, formAction, isPending] = useActionState(renameUserWithId, null)

  useEffect(() => {
    if (state?.error) {
      toast.error(state.error)
    } else if (state?.success) {
      toast.success('Username updated')
      onSuccess()
      router.refresh()
    }
  }, [state, onSuccess, router])

  return (
    <form action={formAction} className="space-y-4 py-2">
      <div className="space-y-2">
        <Label htmlFor="username" className="text-white font-medium">
          Username
        </Label>
        <div className="relative">
          <User className="absolute left-2.5 top-2.5 h-4 w-4 text-red-400" />
          <Input
            id="username"
            name="username"
            defaultValue={initialUser.username || ''}
            className="pl-9 bg-zinc-900 border-red-800/50 text-white placeholder:text-zinc-500 focus:border-red-600 focus:ring-red-600/20"
            disabled={isPending}
          />
        </div>
      </div>
      <Button
        type="submit"
        className="w-full bg-red-700 hover:bg-red-600 text-white font-medium"
        disabled={isPending}
      >
        {isPending && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
        Update Profile
      </Button>

      {initialUser.is_guest && (
        <div className="pt-4 border-t border-zinc-800 space-y-4">
          <div className="relative">
            <div className="absolute inset-0 flex items-center">
              <span className="w-full border-t border-zinc-800" />
            </div>
            <div className="relative flex justify-center text-xs uppercase">
              <span className="bg-zinc-950 px-2 text-zinc-500">Secure your account</span>
            </div>
          </div>
          <p className="text-xs text-zinc-400 text-center">
            Link your account to GitHub to persist your profile and taste data.
          </p>
          <Button
            type="button"
            variant="outline"
            className="w-full border-zinc-800 bg-zinc-900 text-white hover:bg-zinc-800 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
            onClick={handleLinkGithub}
            disabled={isPending || isGithubLoading}
          >
            {isGithubLoading ? (
              <>
                <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                Connecting...
              </>
            ) : (
              <>
                <svg className="mr-2 h-4 w-4" viewBox="0 0 24 24" fill="currentColor"><path d="M12 0c-6.626 0-12 5.373-12 12 0 5.302 3.438 9.8 8.207 11.387.599.111.793-.261.793-.577v-2.234c-3.338.726-4.033-1.416-4.033-1.416-.546-1.387-1.333-1.756-1.333-1.756-1.089-.745.083-.729.083-.729 1.205.084 1.839 1.237 1.839 1.237 1.07 1.834 2.807 1.304 3.492.997.107-.775.418-1.305.762-1.604-2.665-.305-5.467-1.334-5.467-5.931 0-1.311.469-2.381 1.236-3.221-.124-.303-.535-1.524.117-3.176 0 0 1.008-.322 3.301 1.23.957-.266 1.983-.399 3.003-.404 1.02.005 2.047.138 3.006.404 2.291-1.552 3.297-1.23 3.297-1.23.653 1.653.242 2.874.118 3.176.77.84 1.235 1.911 1.235 3.221 0 4.609-2.807 5.624-5.479 5.921.43.372.823 1.102.823 2.222v3.293c0 .319.192.694.801.576 4.765-1.589 8.199-6.086 8.199-11.386 0-6.627-5.373-12-12-12z"/></svg>
                Link GitHub Account
              </>
            )}
          </Button>
        </div>
      )}
    </form>
  )
}

export default AccountForm
