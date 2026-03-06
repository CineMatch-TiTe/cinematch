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

  // Parallel fetch of user prefs, party status, members, and currentUser
  const { getUserPreferences, getCurrentUser } = await import('@/server/user/user')
  const { getParty } = await import('@/server/party/party')
  const { getPartyMembers } = await import('@/server/member-ops/member-ops')

  const [prefsRes, partyRes, membersRes, userRes] = await Promise.all([
    getUserPreferences(), 
    getParty({ party_id: partyId }),
    getPartyMembers({ party_id: partyId }),
    getCurrentUser()
  ])

  let initialView: PartyViewType = 'room'
  const party = partyRes.status === 200 ? partyRes.data : null
  const partyState = party ? party.state : 'created'
  const members = membersRes.status === 200 ? membersRes.data.members : []
  const currentUser = userRes.status === 200 ? userRes.data : null

  if (!party || !currentUser) {
      return <>{children}</> // let page.tsx handle redirecting
  }

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
    <PartyViewProvider 
      initialView={initialView} 
      initialParty={party}
      initialMembers={members}
      currentUser={currentUser}
    >
      <div className="relative">
        {children}
        <PartyFooterNavigation />
      </div>
    </PartyViewProvider>
  )
}
