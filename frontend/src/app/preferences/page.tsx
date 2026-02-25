import React from 'react'
import Preferences from '@/components/preferences/Preferences'
import { PageBackground } from '@/components/ui/PageBackground'

interface PageProps {
  searchParams: Promise<{ [key: string]: string | string[] | undefined }>
}

const PreferencesRoute = async ({ searchParams }: PageProps) => {
  const resolvedParams = await searchParams
  const joinCode = typeof resolvedParams.joinCode === 'string' ? resolvedParams.joinCode : ''

  return (
    <>
      <PageBackground />
      <div className="flex flex-row min-h-screen items-start justify-center pt-6 overflow-y-auto w-full">
        <main className="relative z-10 w-full max-w-4xl px-6">
          <Preferences mode="wizard" joinCode={joinCode} />
        </main>
      </div>
    </>
  )
}

export default PreferencesRoute
