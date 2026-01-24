import Link from 'next/link'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { CreatePartyForm } from '@/components/forms/CreatePartyForm'

export default function CreatePartyRoute() {
  return (
    <div className="flex min-h-screen items-center justify-center bg-zinc-950 font-sans text-zinc-100 selection:bg-red-500/30">
      <div className="fixed inset-0 z-0 pointer-events-none">
        <div className="absolute inset-0 bg-[radial-gradient(circle_at_center,var(--tw-gradient-stops))] from-zinc-800/20 via-zinc-950 to-zinc-950" />
        <div className="absolute top-0 left-0 w-full h-full bg-[url('https://images.unsplash.com/photo-1489599849927-2ee91cede3ba?q=80&w=2070&auto=format&fit=crop')] bg-cover bg-center opacity-20 mix-blend-overlay" />
      </div>

      <main className="relative z-10 w-full max-w-md px-6">
        <div className="mb-8 text-center">
          <h1 className="text-4xl font-bold tracking-tighter text-white mb-2">CineMatch</h1>
          <p className="text-zinc-400">Host a party and enjoy movies with friends!</p>
        </div>

        <Card className="border-zinc-800 bg-zinc-900/50 backdrop-blur-xl shadow-2xl">
          <CardHeader>
            <CardTitle className="text-lg font-medium text-zinc-200">Create a party</CardTitle>
          </CardHeader>
          <CardContent className="space-y-4">
            <CreatePartyForm />
            <div className="pt-2 text-center">
              <Link
                href="/"
                className="text-sm text-yellow-500 hover:text-yellow-400 transition-colors hover:underline"
              >
                Join an existing party?
              </Link>
            </div>
          </CardContent>
        </Card>
      </main>
    </div>
  )
}
