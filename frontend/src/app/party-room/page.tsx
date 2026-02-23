import { redirect } from 'next/navigation'
import { getParty } from '@/server/party/party'
import { getPartyMembers } from '@/server/member-ops/member-ops'
import { getCurrentUser } from '@/server/user/user'
import { getMyPartyIdAction } from '@/actions/party-room'
import PartyViewClient from '@/components/party/PartyViewClient'

export default async function PartyRoom({ searchParams }: Readonly<{ searchParams: Promise<{ id?: string }> }>) {
  const resolvedParams = await searchParams
  let partyId = resolvedParams.id

  if (!partyId) {
    const res = await getMyPartyIdAction()
    if ('id' in res && res.id) {
      redirect(`/party-room?id=${res.id}`)
    } else {
      redirect('/dashboard')
    }
  }

  // Parallel data fetching
  const [userRes, partyRes, membersRes] = await Promise.all([
    getCurrentUser(),
    getParty({ party_id: partyId }),
    getPartyMembers({ party_id: partyId })
  ])

  // Type safety checks (Orval returns { status, data })
  const currentUser = userRes.status === 200 ? userRes.data : null
  const party = partyRes.status === 200 ? partyRes.data : null
  const members = membersRes.status === 200 ? membersRes.data.members : []

  if (!currentUser || !party) {
    if (currentUser) {
      redirect('/dashboard')
    }

    redirect('/')
  }

  return <PartyViewClient party={party} members={members} currentUser={currentUser} />
}
