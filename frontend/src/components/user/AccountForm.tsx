import { Loader2, User } from 'lucide-react'
import { CurrentUserResponse } from '@/model'
import { useRouter } from 'next/navigation'
import { useState } from 'react'
import { renameUserAction } from '@/actions/user'
import { toast } from 'sonner'
import { Label } from '@/components/ui/label'
import { Input } from '@/components/ui/input'
import { Button } from '@/components/ui/button'

const AccountForm = ({ initialUser, onSuccess }: { initialUser: CurrentUserResponse, onSuccess: () => void }) => {
  const router = useRouter()
  const [username, setUsername] = useState(initialUser.username || '')
  const [loading, setLoading] = useState(false)

  const handleRename = async (e: React.FormEvent) => {
    e.preventDefault()
    setLoading(true)
    try {
      const res = await renameUserAction(initialUser.user_id, { new_username: username })
      if (res.error) toast.error(res.error)
      else {
        toast.success('Username updated')
        onSuccess()
        router.refresh()
      }
    } finally {
      setLoading(false)
    }
  }

  return (
    <form onSubmit={handleRename} className="space-y-4 py-2">
      <div className="space-y-2">
        <Label htmlFor="username" className="text-white font-medium">Username</Label>
        <div className="relative">
          <User className="absolute left-2.5 top-2.5 h-4 w-4 text-red-400" />
          <Input
            id="username"
            value={username}
            onChange={(e) => setUsername(e.target.value)}
            className="pl-9 bg-zinc-900 border-red-800/50 text-white placeholder:text-zinc-500 focus:border-red-600 focus:ring-red-600/20"
            disabled={loading}
          />
        </div>
      </div>
      <Button type="submit" className="w-full bg-red-700 hover:bg-red-600 text-white font-medium" disabled={loading}>
        {loading && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
        Update Profile
      </Button>
    </form>
  )
}

export default AccountForm