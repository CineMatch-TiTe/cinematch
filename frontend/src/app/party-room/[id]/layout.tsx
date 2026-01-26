import { ReactNode } from 'react'
import { PartyViewProvider, PartyViewType } from '@/components/party/PartyViewContext' // Make sure import is correct
import { PartyFooterNavigation } from '@/components/party/PartyFooterNavigation'
import { getUserPreferences } from '@/server/user/user'

export default async function PartyRoomLayout({
  children,
  params
}: Readonly<{
  children: ReactNode
  params: Promise<{ id: string }>
}>) {
  // We don't strictly need partyId here for the layout logic, but standard signature
  await params // Ensure params are awaited if needed, though we don't use id here yet.

  // Fetch user preferences to determine initial view
  const prefsRes = await getUserPreferences()

  let initialView: PartyViewType = 'room'

  if (prefsRes.status === 200) {
    const prefs = prefsRes.data
    // If user has set include_genres, we assume they have a taste profile and can start picking
    if (prefs.include_genres && prefs.include_genres.length > 0) {
      initialView = 'picking'
    }
  }

  return (
    <PartyViewProvider initialView={initialView}>
      <div className="relative min-h-screen bg-zinc-950">
        {children}
        <PartyFooterNavigation />
      </div>
    </PartyViewProvider>
  )
}
