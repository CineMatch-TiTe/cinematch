import { useTransition, useState } from 'react'
import Link from 'next/link'

import { z } from 'zod'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { User, Lock, Loader2 } from 'lucide-react'
import { guestLoginAction } from './actions'

const usernameSchema = z
  .string()
  .trim()
  .min(3, { message: 'Username must be at least 3 characters' })
  .max(32, { message: 'Username must be at most 32 characters' })
  .regex(/^[a-zA-Z0-9_ -]+$/, {
    message: 'Username can only contain letters, numbers, spaces, hyphens, and underscores'
  })

const joinCodeSchema = z
  .string()
  .trim()
  .min(4, { message: 'Join code must be at least 4 characters' })
  .max(12, { message: 'Join code too long' })

const guestLoginFormSchema = z.object({
  username: usernameSchema,
  joinCode: joinCodeSchema
})

export default function HomeRoute() {
  const [username, setUsername] = useState('')
  const [joinCode, setJoinCode] = useState('')
  const [errors, setErrors] = useState<{ username?: string; joinCode?: string; form?: string }>({})
  const [isPending, startTransition] = useTransition()

  const handleGuestJoin = async () => {
    // Reset errors
    setErrors({})

    // Client-side Validation
    const result = guestLoginFormSchema.safeParse({ username, joinCode })

    if (!result.success) {
      const fieldErrors = result.error.flatten().fieldErrors
      setErrors({
        username: fieldErrors.username?.[0],
        joinCode: fieldErrors.joinCode?.[0]
      })
      return
    }

    startTransition(async () => {
      const formData = new FormData()
      formData.append('username', username)
      formData.append('joinCode', joinCode)

      const response = await guestLoginAction(null, formData)

      if (response?.errors) {
        setErrors({
          username: response.errors.username?.[0],
          joinCode: response.errors.joinCode?.[0]
        })
      } else if (response?.message) {
        setErrors((prev) => ({ ...prev, form: response.message }))
      }
    })
  }

  return (
    <div className="flex min-h-screen items-center justify-center bg-zinc-950 font-sans text-zinc-100 selection:bg-red-500/30">
      {/* Background ambience - mimics the cinematic feel */}
      <div className="fixed inset-0 z-0 pointer-events-none">
        <div className="absolute inset-0 bg-[radial-gradient(circle_at_center,_var(--tw-gradient-stops))] from-zinc-800/20 via-zinc-950 to-zinc-950" />
        <div className="absolute top-0 left-0 w-full h-full bg-[url('https://images.unsplash.com/photo-1489599849927-2ee91cede3ba?q=80&w=2070&auto=format&fit=crop')] bg-cover bg-center opacity-20 mix-blend-overlay" />
      </div>

      <main className="relative z-10 w-full max-w-md px-6">
        <div className="mb-8 text-center">
          <h1 className="text-4xl font-bold tracking-tighter text-white mb-2">CineMatch</h1>
          <p className="text-zinc-400">Join to party and guess what you want to watch!</p>
        </div>

        <Card className="border-zinc-800 bg-zinc-900/50 backdrop-blur-xl shadow-2xl">
          <CardHeader>
            <CardTitle className="text-lg font-medium text-zinc-200">Join to party</CardTitle>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="space-y-4">
              <div className="space-y-1">
                <div className="relative">
                  <User className="absolute left-3 top-1/2 -translate-y-1/2 h-5 w-5 text-zinc-500" />
                  <Input
                    placeholder="Your name"
                    value={username}
                    onChange={(e) => setUsername(e.target.value)}
                    className={`pl-10 bg-zinc-950/50 border-zinc-700 text-zinc-100 placeholder:text-zinc-500 focus-visible:ring-red-600 focus-visible:border-red-600 ${errors.username ? 'border-red-500 focus-visible:ring-red-500' : ''}`}
                  />
                </div>
                {errors.username && <p className="text-xs text-red-500 pl-1">{errors.username}</p>}
              </div>

              <div className="space-y-1">
                <div className="relative">
                  <Lock className="absolute left-3 top-1/2 -translate-y-1/2 h-5 w-5 text-zinc-500" />
                  <Input
                    placeholder="Join code"
                    value={joinCode}
                    onChange={(e) => setJoinCode(e.target.value)}
                    className={`pl-10 bg-zinc-950/50 border-zinc-700 text-zinc-100 placeholder:text-zinc-500 focus-visible:ring-red-600 focus-visible:border-red-600 ${errors.joinCode ? 'border-red-500 focus-visible:ring-red-500' : ''}`}
                  />
                </div>
                {errors.joinCode && <p className="text-xs text-red-500 pl-1">{errors.joinCode}</p>}
              </div>
            </div>

            {errors.form && <p className="text-sm text-red-500 text-center">{errors.form}</p>}

            <Button
              onClick={handleGuestJoin}
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

            <div className="pt-2 text-center">
              <Link
                href="/create-party"
                className="text-sm text-yellow-500 hover:text-yellow-400 transition-colors hover:underline"
              >
                Want to host a new party?
              </Link>
            </div>
          </CardContent>
        </Card>
      </main>
    </div>
  )
}
