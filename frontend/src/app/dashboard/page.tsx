import { redirect } from 'next/navigation'
import { getCurrentUser } from '@/server/user/user'
import { getParty } from '@/server/party/party'
import { DashboardClient } from '@/components/dashboard/DashboardClient'
import { PageBackground } from '@/components/ui/PageBackground'
import { DashboardHeader } from '@/components/dashboard/DashboardHeader'

export default async function DashboardPage() {
  const userRes = await getCurrentUser().catch(() => null)

  if (userRes?.status !== 200 || !userRes.data) {
    redirect('/')
  }

  const partyRes = await getParty({}).catch(() => null)
  if (partyRes?.status === 200 && partyRes.data?.id) {
    redirect(`/party-room?id=${partyRes.data.id}`)
  }

  const user = userRes.data

  return (
    <>
      <PageBackground showImage imageOpacity={10} />
      <DashboardHeader />
      <DashboardClient user={user} />
    </>
  )
}
