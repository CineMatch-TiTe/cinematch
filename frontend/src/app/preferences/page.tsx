import React from 'react'
import PreferencesFlow from '../../components/preferences/PreferencesFlow'

interface PageProps {
  searchParams: Promise<{ [key: string]: string | string[] | undefined }>
}

const PreferencesRoute = async ({ searchParams }: PageProps) => {
  const resolvedParams = await searchParams
  const joinCode = typeof resolvedParams.joinCode === 'string' ? resolvedParams.joinCode : ''

  if (!joinCode) {
    return (
      <div className="flex flex-row min-h-screen items-start justify-center pt-20 bg-zinc-950 font-sans text-zinc-100 selection:bg-red-500/30 overflow-y-scroll">
        <div className="fixed inset-0 z-0 pointer-events-none">
          <div className="absolute inset-0 bg-[radial-gradient(circle_at_center,var(--tw-gradient-stops))] from-zinc-800/20 via-zinc-950 to-zinc-950" />
        </div>
        <main className="relative z-10 w-full max-w-md px-6">
          <div className="flex flex-col items-center justify-center mb-8 gap-4 text-center">
            <div className="bg-red-500/10 p-6 rounded-lg border border-red-500/20 backdrop-blur-sm">
              <h1 className="text-2xl font-bold text-red-500 mb-2">Missing Join Code</h1>
              <p className="text-zinc-400">Please use the link provided by the party host.</p>
            </div>
          </div>
        </main>
      </div>
    )
  }

  return (
    <div className="flex flex-row min-h-screen items-start justify-center pt-20 bg-zinc-950 font-sans text-zinc-100 selection:bg-red-500/30 overflow-y-scroll">
      <div className="fixed inset-0 z-0 pointer-events-none">
        <div className="absolute inset-0 bg-[radial-gradient(circle_at_center,var(--tw-gradient-stops))] from-zinc-800/20 via-zinc-950 to-zinc-950" />
      </div>
      <main className="relative z-10 w-full max-w-4xl px-6">
        <PreferencesFlow joinCode={joinCode} />
      </main>
    </div>
  )
}

export default PreferencesRoute
