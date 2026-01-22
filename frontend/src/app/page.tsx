import Link from 'next/link'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { User, Lock } from 'lucide-react'

export default function Home() {
  return (
    <div className="flex min-h-screen items-center justify-center bg-zinc-950 font-sans text-zinc-100 selection:bg-red-500/30">
      {/* Background ambience - mimics the cinematic feel */}
      <div className="fixed inset-0 z-0 pointer-events-none">
        <div className="absolute inset-0 bg-[radial-gradient(circle_at_center,_var(--tw-gradient-stops))] from-zinc-800/20 via-zinc-950 to-zinc-950" />
        <div className="absolute top-0 left-0 w-full h-full bg-[url('https://images.unsplash.com/photo-1489599849927-2ee91cede3ba?q=80&w=2070&auto=format&fit=crop')] bg-cover bg-center opacity-20 mix-blend-overlay" />
      </div>

      <main className="relative z-10 w-full max-w-md px-6">
        <div className="mb-8 text-center">
          <h1 className="text-4xl font-bold tracking-tighter text-white mb-2">CineMatch</h1>
          <p className="text-zinc-400">Liity mukaan partyyn ja arvaa mitä haluat nähdä!</p>
        </div>

        <Card className="border-zinc-800 bg-zinc-900/50 backdrop-blur-xl shadow-2xl">
          <CardHeader>
            <CardTitle className="text-lg font-medium text-zinc-200">Liity partyyn</CardTitle>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="space-y-4">
              <div className="relative">
                <User className="absolute left-3 top-1/2 -translate-y-1/2 h-5 w-5 text-zinc-500" />
                <Input
                  placeholder="Nimesi"
                  className="pl-10 bg-zinc-950/50 border-zinc-700 text-zinc-100 placeholder:text-zinc-500 focus-visible:ring-red-600 focus-visible:border-red-600"
                />
              </div>

              <div className="relative">
                <Lock className="absolute left-3 top-1/2 -translate-y-1/2 h-5 w-5 text-zinc-500" />
                <Input
                  placeholder="Liittymiskoodi"
                  className="pl-10 bg-zinc-950/50 border-zinc-700 text-zinc-100 placeholder:text-zinc-500 focus-visible:ring-red-600 focus-visible:border-red-600"
                />
              </div>
            </div>

            <Button className="w-full bg-red-900 hover:bg-red-800 text-white font-semibold py-6 text-lg shadow-[0_0_15px_rgba(153,27,27,0.5)] transition-all hover:shadow-[0_0_25px_rgba(153,27,27,0.6)]">
              Kirjaudu vierailijana
            </Button>

            <div className="pt-2 text-center">
              <Link
                href="/create-party"
                className="text-sm text-yellow-500 hover:text-yellow-400 transition-colors hover:underline"
              >
                Haluatko hostata uuden partyn?
              </Link>
            </div>
          </CardContent>
        </Card>
      </main>
    </div>
  )
}
