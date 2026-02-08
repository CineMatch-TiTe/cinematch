'use client'

import { useState } from 'react'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
  DialogTrigger
} from '@/components/ui/dialog'
import { Loader2 } from 'lucide-react'
import useSWR from 'swr'
import { getCurrentUserAction, getUserPreferencesAction } from '@/actions/user'
import AccountForm from '@/components/user/AccountForm'
import Preferences from '@/components/preferences/Preferences'

const fetchUser = async () => {
  const res = await getCurrentUserAction()
  if (res.data) return res.data
  throw new Error(res.error || 'Failed to fetch user')
}

const fetchPrefs = async () => {
  const res = await getUserPreferencesAction()
  if (res.data) return res.data
  throw new Error(res.error || 'Failed to fetch prefs')
}

export function PreferencesDialog({
  trigger,
  open,
  onOpenChangeAction
}: Readonly<{
  trigger?: React.ReactNode
  open?: boolean
  onOpenChangeAction?: (open: boolean) => void
}>) {
  const {
    data: userData,
    mutate: mutateUser,
    isLoading: isUserLoading
  } = useSWR('/api/user', fetchUser)
  const { data: prefData, mutate: mutatePref } = useSWR('/api/user/pref', fetchPrefs)
  const [showPrefsFlow, setShowPrefsFlow] = useState(false)

  const renderSettingsContent = () => {
    if (isUserLoading) {
      return (
        <div className="flex justify-center p-8">
          <Loader2 className="animate-spin text-red-500" />
        </div>
      )
    }

    if (userData) {
      return (
        <>
          <AccountForm initialUser={userData} onSuccess={() => mutateUser()} />

          <div className="mt-8 pt-6 border-t border-zinc-800">
            <h3 className="text-lg font-medium text-white mb-2">Taste Profile</h3>
            <p className="text-sm text-zinc-400 mb-4">
              Update your movie preferences, favorite genres, and decades.
            </p>
            <button
              onClick={() => setShowPrefsFlow(true)}
              className="w-full bg-zinc-800 hover:bg-zinc-700 text-white font-medium py-2 px-4 rounded-md transition-colors border border-zinc-700"
            >
              Redo Taste Profile Setup
            </button>
          </div>
        </>
      )
    }

    return <div className="text-red-500 text-sm">Failed to load user data</div>
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChangeAction}>
      {trigger && <DialogTrigger asChild>{trigger}</DialogTrigger>}
      <DialogContent className="sm:max-w-112.5 bg-zinc-950 border-red-900/20 text-white max-h-[90vh] overflow-y-auto">
        <DialogHeader>
          <DialogTitle className="text-white">Settings</DialogTitle>
          <DialogDescription className="text-zinc-400">
            Customize your experience and movie matching filters.
          </DialogDescription>
        </DialogHeader>

        {showPrefsFlow ? (
          <Preferences
            mode="wizard"
            initialPrefs={prefData}
            onComplete={() => {
              mutatePref()
              setShowPrefsFlow(false)
            }}
          />
        ) : (
          <div className="space-y-6">{renderSettingsContent()}</div>
        )}
      </DialogContent>
    </Dialog>
  )
}
