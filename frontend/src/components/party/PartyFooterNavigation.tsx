'use client'

import { Users, Clapperboard, ThumbsUp, Film } from 'lucide-react'

import { cn } from '@/lib/utils'
import { usePartyView, PartyViewType } from './PartyViewContext'

export function PartyFooterNavigation() {
  const { activeView, setActiveView, partyState } = usePartyView()

  const navItems: { id: PartyViewType; label: string; icon: React.ElementType }[] = [
    { id: 'room', label: 'Party', icon: Users }
  ]

  if (partyState !== 'voting' && partyState !== 'watching') {
    navItems.push({ id: 'picking', label: 'Picking', icon: Clapperboard })
  }

  if (partyState === 'voting') {
    navItems.push({ id: 'voting', label: 'Voting movies', icon: ThumbsUp })
  }

  if (partyState === 'watching') {
    navItems.push({ id: 'watching', label: 'Current movie', icon: Film })
  }

  return (
    <div className="fixed bottom-6 left-1/2 transform -translate-x-1/2 z-50">
      <div className="flex items-center bg-black/80 backdrop-blur-xl border border-white/10 rounded-full p-1.5 shadow-2xl shadow-black/50">
        {navItems.map((item) => {
          const isActive = activeView === item.id
          return (
            <button
              key={item.id}
              onClick={() => setActiveView(item.id)}
              className={cn(
                'relative flex items-center gap-2 px-6 py-3 rounded-full transition-all duration-300 ease-out',
                isActive
                  ? 'bg-red-600 text-white shadow-lg shadow-red-600/25'
                  : 'text-zinc-400 hover:text-zinc-200 hover:bg-white/5'
              )}
            >
              <item.icon className={cn('w-5 h-5', isActive && 'fill-current')} />
              <span className={cn('font-medium text-sm', isActive ? 'opacity-100' : 'opacity-80')}>
                {item.label}
              </span>
            </button>
          )
        })}
      </div>
    </div>
  )
}
