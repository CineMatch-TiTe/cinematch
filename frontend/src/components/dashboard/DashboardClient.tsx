'use client'

import { useState } from 'react'
import { PartyActions } from './PartyActions'
import PersonalPickingFlow from '@/components/party/picking/PersonalPickingFlow'
import { Button } from '@/components/ui/button'
import { CurrentUserResponse } from '@/model/currentUserResponse'
interface DashboardClientProps {
  user: CurrentUserResponse
}

type DashboardView = 'actions' | 'picking'

export function DashboardClient({ user }: Readonly<DashboardClientProps>) {
  const [view, setView] = useState<DashboardView>('actions')

  return (
    <main className="relative z-10 container mx-auto px-4 py-8 max-w-4xl min-h-[calc(100vh-80px)] flex flex-col">
      <div className="mb-4">
        <h1 className="text-3xl font-bold mb-2">Welcome back, {user.username}!</h1>
        <p className="text-zinc-400">Ready to find something to watch?</p>
      </div>

      <div className="flex-1 flex flex-col justify-center">
        {view === 'actions' ? (
          <PartyActions />
        ) : (
          <div className="w-full max-w-md mx-auto">
            <PersonalPickingFlow />
          </div>
        )}
      </div>

      {/* Pill Switcher */}
      <div className="fixed bottom-8 left-1/2 -translate-x-1/2 z-50">
        <div className="bg-zinc-900/90 backdrop-blur-xl border border-zinc-800 p-1 rounded-full shadow-2xl shadow-black/50 flex gap-1">
          <Button
            variant={view === 'actions' ? 'secondary' : 'ghost'}
            size="sm"
            onClick={() => setView('actions')}
            className={`rounded-full px-6 transition-all ${
              view === 'actions'
                ? 'bg-zinc-100 text-zinc-950 hover:bg-white'
                : 'text-zinc-400 hover:text-zinc-100 hover:bg-zinc-800'
            }`}
          >
            Home
          </Button>
          <Button
            variant={view === 'picking' ? 'secondary' : 'ghost'}
            size="sm"
            onClick={() => setView('picking')}
            className={`rounded-full px-6 transition-all ${
              view === 'picking'
                ? 'bg-red-600 text-white hover:bg-red-500'
                : 'text-zinc-400 hover:text-zinc-100 hover:bg-zinc-800'
            }`}
          >
            Picking
          </Button>
        </div>
      </div>
    </main>
  )
}
