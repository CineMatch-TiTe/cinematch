import { ReactNode } from 'react'
import { PartyViewProvider, PartyViewType } from '@/components/party/PartyViewContext' // Make sure import is correct
import { PartyFooterNavigation } from '@/components/party/PartyFooterNavigation'

export default async function PartyRoomLayout({
  children,
  params
}: Readonly<{
  children: ReactNode
  params: Promise<{ id: string }>
}>) {
  // Ensure params are awaited
  const { id: partyId } = await params

  // Parallel fetch of user prefs and party status
  const { getUserPreferences } = await import('@/server/user/user')
  const { getParty } = await import('@/server/party/party')

  const [prefsRes, partyRes] = await Promise.all([getUserPreferences(), getParty(partyId)])

  let initialView: PartyViewType = 'room'
  const partyState = partyRes.status === 200 ? partyRes.data.state : 'created'

  if (partyState === 'voting') {
    initialView = 'voting'
  } else if (partyState === 'watching') {
    initialView = 'watching'
  } else if (prefsRes.status === 200) {
    const prefs = prefsRes.data
    // If user has set include_genres, we assume they have a taste profile and can start picking
    if (prefs.include_genres && prefs.include_genres.length > 0) {
      initialView = 'picking'
    }
  }

  return (
    <PartyViewProvider initialView={initialView} partyState={partyState}>
      <div className="relative min-h-screen bg-zinc-950">
        {children}
        <PartyFooterNavigation />
      </div>
    </PartyViewProvider>
  )
}
