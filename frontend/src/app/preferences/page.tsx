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
      <div className="flex min-h-screen items-center justify-center p-4">
        <div className="text-center">
          <h1 className="text-2xl font-bold text-red-500 mb-2">Missing Join Code</h1>
          <p>Please use the link provided by the party host.</p>
        </div>
      </div>
    )
  }

  return <PreferencesFlow joinCode={joinCode} />
}

export default PreferencesRoute
