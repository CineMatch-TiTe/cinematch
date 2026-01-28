import { redirect } from 'next/navigation'
import { getCurrentUser } from '@/server/user/user'
import { getMyParty } from '@/server/party/party'
import { Button } from '@/components/ui/button'
import { LogOut, Settings } from 'lucide-react'
import Image from 'next/image'
import { PreferencesDialog } from '@/components/preferences/PreferencesDialog'
import { DashboardClient } from '@/components/dashboard/DashboardClient'

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

      <DashboardClient user={user} />
    </div>
  )
}
