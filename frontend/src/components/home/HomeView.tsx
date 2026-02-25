import Link from 'next/link'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { GuestLoginForm } from '@/components/forms/GuestLoginForm'
import { PageBackground } from '@/components/ui/PageBackground'
import { AuthLayout } from '@/components/common/AuthLayout'

export function HomeView({ initialJoinCode }: Readonly<{ initialJoinCode?: string }>) {
  return (
    <div className="flex flex-col min-h-screen items-center justify-center">
      <PageBackground showImage imageOpacity={20} />

      <AuthLayout 
        title="CineMatch" 
        subtitle="Join to party and guess what you want to watch!"
      >
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
                Want to login and host a party?
              </Link>
            </div>
          </CardContent>
        </Card>
      </AuthLayout>
    </div>
  )
}
