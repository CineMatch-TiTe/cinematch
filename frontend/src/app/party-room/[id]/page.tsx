import { redirect } from 'next/navigation'
import { getParty, getPartyMembers } from '@/server/party/party'
import { getCurrentUser } from '@/server/user/user'
import PartyViewClient from '@/components/party/PartyViewClient'

export default async function PartyRoom({ params }: Readonly<{ params: Promise<{ id: string }> }>) {
  const { id: partyId } = await params

  // Parallel data fetching
  const [userRes, partyRes, membersRes] = await Promise.all([
    getCurrentUser(),
    getParty(partyId),
    getPartyMembers(partyId)
  ])

  // Type safety checks (Orval returns { status, data })
  const currentUser = userRes.status === 200 ? userRes.data : null
  const party = partyRes.status === 200 ? partyRes.data : null
  const members = membersRes.status === 200 ? membersRes.data.members : []

  if (!currentUser || !party) {
    // Should behave like 404 or redirect if critical data missing
    redirect('/')
  }

  return <PartyViewClient party={party} members={members} currentUser={currentUser} />
}
