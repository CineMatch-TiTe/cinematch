import Link from 'next/link'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { LoginForm } from '@/components/forms/LoginForm'
import { PageBackground } from '@/components/ui/PageBackground'
import { AuthLayout } from '@/components/common/AuthLayout'

export function CreatePartyView() {
  return (
    <div className="flex flex-col min-h-screen items-center justify-center">
      <PageBackground showImage imageOpacity={20} />

      <AuthLayout 
        title="CineMatch" 
        subtitle="Login to start finding movies!"
      >
        <Card className="border-zinc-800 bg-zinc-900/50 backdrop-blur-xl shadow-2xl">
          <CardHeader>
            <CardTitle className="text-lg font-medium text-zinc-200">Login</CardTitle>
          </CardHeader>
          <CardContent className="space-y-4">
            <LoginForm />
            <div className="pt-2 text-center">
              <Link
                href="/"
                className="text-sm text-yellow-500 hover:text-yellow-400 transition-colors hover:underline"
              >
                Have a join code?
              </Link>
            </div>
          </CardContent>
        </Card>
      </AuthLayout>
    </div>
  )
}
