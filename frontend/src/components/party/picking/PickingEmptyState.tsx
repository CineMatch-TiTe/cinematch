'use client'

import { Button } from '@/components/ui/button'

import { usePartyView } from '@/components/party/PartyViewContext'

export default function PickingEmptyState() {
  const { setActiveView } = usePartyView()
  return (
    <div className="fixed inset-0 z-50 flex flex-col items-center justify-center bg-zinc-950/90 backdrop-blur-md p-6 text-center">
      <h2 className="text-2xl font-bold text-white mb-2">That&apos;s all for now!</h2>
      <p className="text-zinc-400 mb-8 max-w-xs">
        We&apos;ve run out of movies based on your preferences. Check back later or wait for others!
      </p>
      <div className="flex gap-3">
        <Button
          onClick={() => setActiveView('room')}
          size="lg"
          className="bg-red-600 text-white hover:bg-red-700 shadow-lg shadow-red-500/20"
        >
          Return to Party
        </Button>
      </div>
    </div>
  )
}
