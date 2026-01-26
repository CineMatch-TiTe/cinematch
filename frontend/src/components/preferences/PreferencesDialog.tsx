'use client'

import React from 'react'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
  DialogTrigger
} from '@/components/ui/dialog'


import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { Loader2 } from 'lucide-react'
import useSWR from 'swr'
import {
  getCurrentUserAction,
  getUserPreferencesAction
} from '@/actions/user'


import AccountForm from '@/components/user/AccountForm'
import PreferencesTabFlow from '@/components/preferences/PreferencesTabFlow'


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

export function PreferencesDialog({ trigger, open, onOpenChangeAction }: {
  trigger?: React.ReactNode
  open?: boolean
  onOpenChangeAction?: (open: boolean) => void
}) {
  const { data: userData, mutate: mutateUser, isLoading: isUserLoading } = useSWR('/api/user', fetchUser)
  const { data: prefData, mutate: mutatePref, isLoading: isPrefLoading } = useSWR('/api/user/pref', fetchPrefs)

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

        <Tabs defaultValue="account" className="w-full">
          <TabsList className="grid w-full grid-cols-2 bg-zinc-900 border border-red-900/30 mb-4">
            <TabsTrigger value="account" className="text-white data-[state=active]:bg-red-700 data-[state=active]:text-white">Account</TabsTrigger>
            <TabsTrigger value="preferences" className="text-white data-[state=active]:bg-red-700 data-[state=active]:text-white">Movie Filters</TabsTrigger>
          </TabsList>

          <TabsContent value="account">
            {isUserLoading ? (
              <div className="flex justify-center p-8"><Loader2 className="animate-spin text-red-500" /></div>
            ) : userData ? (
              <AccountForm initialUser={userData} onSuccess={() => mutateUser()} />
            ) : (
              <div className="text-red-500 text-sm">Failed to load user data</div>
            )}
          </TabsContent>

          <TabsContent value="preferences">
            {isPrefLoading ? (
              <div className="flex justify-center p-8"><Loader2 className="animate-spin text-red-500" /></div>
            ) : prefData ? (
              <PreferencesTabFlow initialPrefs={prefData} onSuccess={() => mutatePref()} />
            ) : (
              <div className="text-red-500 text-sm">Failed to load preferences</div>
            )}
          </TabsContent>
        </Tabs>
      </DialogContent>
    </Dialog>
  )
}