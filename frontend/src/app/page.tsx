import { redirect } from 'next/navigation'
import { getCurrentUser } from '@/server/user/user'
import { getParty } from '@/server/party/party'
import { HomeView } from '@/components/home/HomeView'

export default async function HomeRoute({
  searchParams
}: Readonly<{
  searchParams: Promise<{ [key: string]: string | string[] | undefined }>
}>) {
  const { joinCode } = await searchParams
  const initialJoinCode = Array.isArray(joinCode) ? joinCode[0] : joinCode

  // Check if user is already logged in and has an active party
  const userRes = await getCurrentUser().catch(() => null)
  if (userRes?.status === 200) {
    const partyRes = await getParty({}).catch(() => null)
    if (partyRes?.status === 200 && partyRes.data?.id) {
      redirect(`/party-room?id=${partyRes.data.id}`)
    } else {
      redirect('/dashboard')
    }
  }

  return <HomeView initialJoinCode={initialJoinCode} />
}
