import { redirect } from 'next/navigation'
import { getCurrentUser } from '@/server/user/user'
import { getMyParty } from '@/server/party/party'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { LogOut, Settings, Users, Plus } from 'lucide-react'
import Link from 'next/link'
import Image from 'next/image'
import { PreferencesDialog } from '@/components/preferences/PreferencesDialog'

export default async function DashboardPage() {
  const userRes = await getCurrentUser().catch(() => null)

  if (userRes?.status !== 200 || !userRes.data) {
    redirect('/')
  }

  const partyRes = await getMyParty().catch(() => null)
  if (partyRes?.status === 200 && partyRes.data?.id) {
    redirect(`/party-room/${partyRes.data.id}`)
  }

  const user = userRes.data

  return (
    <div className="min-h-screen bg-zinc-950 text-zinc-100 selection:bg-red-500/30">
      <div className="fixed inset-0 z-0 pointer-events-none">
        <div className="absolute inset-0 bg-[radial-gradient(circle_at_center,var(--tw-gradient-stops))] from-zinc-800/20 via-zinc-950 to-zinc-950" />
        <div className="absolute top-0 left-0 w-full h-full bg-[url('https://images.unsplash.com/photo-1489599849927-2ee91cede3ba?q=80&w=2070&auto=format&fit=crop')] bg-cover bg-center opacity-10 mix-blend-overlay" />
      </div>

      <header className="relative z-10 border-b border-zinc-800 bg-zinc-900/50 backdrop-blur-md">
        <div className="container mx-auto px-4 h-16 flex items-center justify-between">
          <div className="flex items-center gap-2">
            <Image src="/Logo.png" alt="CineMatch" width={32} height={32} className="w-8 h-8" />
            <span className="font-bold text-xl tracking-tight">CineMatch</span>
          </div>
          <div className="flex items-center gap-2">
            <PreferencesDialog
              trigger={
                <Button
                  variant="ghost"
                  size="icon"
                  className="text-zinc-400 hover:text-white hover:bg-zinc-800"
                >
                  <Settings className="h-5 w-5" />
                  <span className="sr-only">Settings</span>
                </Button>
              }
            />
            <form action="/api/logout" method="POST">
              <Button
                variant="ghost"
                size="icon"
                className="text-zinc-400 hover:text-red-400 hover:bg-zinc-800"
              >
                <LogOut className="h-5 w-5" />
                <span className="sr-only">Logout</span>
              </Button>
            </form>
          </div>
        </div>
      </header>

      <main className="relative z-10 container mx-auto px-4 py-8 max-w-4xl">
        <div className="mb-8">
          <h1 className="text-3xl font-bold mb-2">Welcome back, {user.username}!</h1>
          <p className="text-zinc-400">Ready to find something to watch?</p>
        </div>

        <div className="grid md:grid-cols-2 gap-6">
          <Card className="bg-zinc-900/50 border-zinc-800 hover:border-red-900/50 transition-colors group">
            <CardHeader>
              <CardTitle className="flex items-center gap-2 text-zinc-100">
                <Plus className="h-5 w-5 text-red-500 group-hover:text-red-400 transition-colors" />
                Host a Party
              </CardTitle>
              <CardDescription className="text-zinc-400">
                Start a new party and invite your friends to swipe together.
              </CardDescription>
            </CardHeader>
            <CardContent>
              <Button
                asChild
                className="w-full bg-red-900 hover:bg-red-800 text-white shadow-lg shadow-red-900/20"
              >
                <Link href="/create-party">Create New Party</Link>
              </Button>
            </CardContent>
          </Card>

          <Card className="bg-zinc-900/50 border-zinc-800 hover:border-zinc-700 transition-colors group">
            <CardHeader>
              <CardTitle className="flex items-center gap-2 text-zinc-100">
                <Users className="h-5 w-5 text-zinc-500 group-hover:text-zinc-400 transition-colors" />
                Join a Party
              </CardTitle>
              <CardDescription className="text-zinc-400">
                Have a code? Join an existing party and start matching.
              </CardDescription>
            </CardHeader>
            <CardContent>
              <Button
                asChild
                variant="secondary"
                className="w-full bg-zinc-800 hover:bg-zinc-700 text-zinc-100 border border-zinc-700"
              >
                <Link href="/#join">Join Existing Party</Link>
              </Button>
            </CardContent>
          </Card>
        </div>
      </main>
    </div>
  )
}
