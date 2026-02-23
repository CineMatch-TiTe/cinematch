import { ReactNode } from 'react'
import { PartyViewProvider, PartyViewType } from '@/components/party/PartyViewContext' // Make sure import is correct
import { PartyFooterNavigation } from '@/components/party/PartyFooterNavigation'
import { getMyPartyIdAction } from '@/actions/party-room'

export default async function PartyRoomLayout({
  children
}: Readonly<{
  children: ReactNode
}>) {
  // We cannot read searchParams in layouts reliably. So we ask the backend what party we are in.
  const res = await getMyPartyIdAction()
  const partyId = 'id' in res ? res.id : undefined

  if (!partyId) {
    return <>{children}</> // Should be caught by page redirect
  }

  // Parallel fetch of user prefs and party status
  const { getUserPreferences } = await import('@/server/user/user')
  const { getParty } = await import('@/server/party/party')

  const [prefsRes, partyRes] = await Promise.all([getUserPreferences(), getParty({ party_id: partyId })])

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
