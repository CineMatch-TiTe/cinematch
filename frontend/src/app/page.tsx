import Link from 'next/link'
import { redirect } from 'next/navigation'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { GuestLoginForm } from '@/components/forms/GuestLoginForm'
import Image from 'next/image'
import { getCurrentUser } from '@/server/user/user'
import { getMyParty } from '@/server/party/party'

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
    const partyRes = await getMyParty().catch(() => null)
    if (partyRes?.status === 200 && partyRes.data?.id) {
      redirect(`/party-room/${partyRes.data.id}`)
    }
  }

  return (
    <div className="flex min-h-screen items-center justify-center bg-zinc-950 font-sans text-zinc-100 selection:bg-red-500/30">
      <div className="fixed inset-0 z-0 pointer-events-none">
        <div className="absolute inset-0 bg-[radial-gradient(circle_at_center,var(--tw-gradient-stops))] from-zinc-800/20 via-zinc-950 to-zinc-950" />
        <div className="absolute top-0 left-0 w-full h-full bg-[url('https://images.unsplash.com/photo-1489599849927-2ee91cede3ba?q=80&w=2070&auto=format&fit=crop')] bg-cover bg-center opacity-20 mix-blend-overlay" />
      </div>

      <main className="relative z-10 w-full max-w-md px-6">
        <div className="flex flex-col items-center justify-center mb-8 gap-4 text-center">
          <Image
            src="/Logo.png"
            className="w-36 h-auto"
            alt="CineMatch"
            width={320}
            height={320}
            loading="eager"
          />
          <h1 className="text-4xl font-bold tracking-tighter text-white mb-2">CineMatch</h1>
          <p className="text-zinc-400">Join to party and guess what you want to watch!</p>
        </div>

        <Card className="border-zinc-800 bg-zinc-900/50 backdrop-blur-xl shadow-2xl">
          <CardHeader>
            <CardTitle className="text-lg font-medium text-zinc-200">Join to party</CardTitle>
          </CardHeader>
          <CardContent className="space-y-4">
            <GuestLoginForm initialJoinCode={initialJoinCode} />
            <div className="pt-2 text-center">
              <Link
                href="/create-party"
                className="text-sm text-yellow-500 hover:text-yellow-400 transition-colors hover:underline"
              >
                Want to host a new party?
              </Link>
            </div>
          </CardContent>
        </Card>
      </main>
    </div>
  )
}
