'use client'

import { Loader2, User } from 'lucide-react'
import { CurrentUserResponse } from '@/model'
import { useRouter } from 'next/navigation'
import { useActionState, useEffect, useCallback } from 'react'
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
    </form>
  )
}

export default AccountForm
