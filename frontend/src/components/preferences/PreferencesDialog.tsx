'use client'

import React, { useState } from 'react'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
  DialogTrigger
} from '@/components/ui/dialog'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { User, Loader2 } from 'lucide-react'
import { toast } from 'sonner'
import { useRouter } from 'next/navigation'
import useSWR from 'swr'
import { UpdateUserPreferencesRequest } from '@/model/updateUserPreferencesRequest'
import {
  getCurrentUserAction,
  getUserPreferencesAction,
  renameUserAction,
  updateUserPreferencesAction
} from '@/actions/user'

import { CurrentUserResponse } from '@/model/currentUserResponse'
import { UserPreferencesResponse } from '@/model/userPreferencesResponse'

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

// Inner Form Component for Account
const AccountForm = ({
  initialUser,
  onSuccess
}: {
  initialUser: CurrentUserResponse
  onSuccess: () => void
}) => {
  const router = useRouter()
  // Orval generated types might be slightly different
  const [username, setUsername] = useState(initialUser.username || '')
  const [loading, setLoading] = useState(false)

  const handleRename = async (e: React.FormEvent) => {
    e.preventDefault()
    setLoading(true)

    try {
      const res = await renameUserAction(initialUser.user_id, { new_username: username })
      if (res.error) {
        toast.error(res.error)
      } else {
        toast.success('Username updated')
        onSuccess()
        router.refresh()
      }
    } finally {
      setLoading(false)
    }
  }

  return (
    <form onSubmit={handleRename} className="space-y-4">
      <div className="space-y-2">
        <Label htmlFor="username">Username</Label>
        <div className="flex gap-2">
          <div className="relative flex-1">
            <User className="absolute left-2.5 top-2.5 h-4 w-4 text-zinc-500" />
            <Input
              id="username"
              value={username}
              onChange={(e) => setUsername(e.target.value)}
              className="pl-9 bg-zinc-900 border-zinc-700 focus-visible:ring-red-500"
              disabled={loading}
            />
          </div>
        </div>
        <p className="text-xs text-zinc-500">This is how you will appear to other party members.</p>
      </div>
      <Button
        type="submit"
        className="w-full bg-red-900 hover:bg-red-800 text-white"
        disabled={loading}
      >
        {loading && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
        Update Profile
      </Button>
    </form>
  )
}

// Inner Form Component for Preferences
const PreferencesForm = ({
  initialPrefs,
  onSuccess
}: {
  initialPrefs: UserPreferencesResponse
  onSuccess: () => void
}) => {
  const [loading, setLoading] = useState(false)
  const [includeGenres, setIncludeGenres] = useState<string[]>(initialPrefs.include_genres || [])
  const [excludeGenres, setExcludeGenres] = useState<string[]>(initialPrefs.exclude_genres || [])
  const [targetYear, setTargetYear] = useState<number | ''>(initialPrefs.target_release_year ?? '')
  const [yearFlex, setYearFlex] = useState<number>(initialPrefs.release_year_flex ?? 0)

  const handleSavePrefs = async (e: React.FormEvent) => {
    e.preventDefault()
    setLoading(true)

    const payload: UpdateUserPreferencesRequest = {
      include_genres: includeGenres,
      exclude_genres: excludeGenres,
      target_release_year: targetYear === '' ? null : Number(targetYear),
      release_year_flex: Number(yearFlex)
    }

    try {
      const res = await updateUserPreferencesAction(payload)
      if (res.error) {
        toast.error(res.error)
      } else {
        toast.success('Preferences saved')
        onSuccess()
      }
    } finally {
      setLoading(false)
    }
  }

  return (
    <form onSubmit={handleSavePrefs} className="space-y-4">
      <div className="space-y-4 border border-zinc-800 rounded-md p-4 bg-zinc-900/30">
        <div className="space-y-2">
          <Label>Target Release Year</Label>
          <Input
            type="number"
            placeholder="e.g. 1990"
            value={targetYear}
            onChange={(e) => setTargetYear(e.target.value === '' ? '' : Number(e.target.value))}
            className="bg-zinc-900 border-zinc-700"
            disabled={loading}
          />
          <p className="text-xs text-zinc-500">Optional. Leave empty for any year.</p>
        </div>

        <div className="space-y-2">
          <Label>Year Flexibility (+/- years)</Label>
          <Input
            type="number"
            value={yearFlex}
            onChange={(e) => setYearFlex(Number(e.target.value))}
            className="bg-zinc-900 border-zinc-700"
            disabled={loading}
          />
        </div>
      </div>

      <div className="space-y-2">
        <Label>Liked Genres (Comma separated)</Label>
        <Input
          placeholder="Action, Comedy..."
          value={includeGenres.join(', ')}
          onChange={(e) =>
            setIncludeGenres(
              e.target.value
                .split(',')
                .map((s) => s.trim())
                .filter(Boolean)
            )
          }
          className="bg-zinc-900 border-zinc-700"
          disabled={loading}
        />
      </div>

      <div className="space-y-2">
        <Label>Disliked Genres (Comma separated)</Label>
        <Input
          placeholder="Horror, Romance..."
          value={excludeGenres.join(', ')}
          onChange={(e) =>
            setExcludeGenres(
              e.target.value
                .split(',')
                .map((s) => s.trim())
                .filter(Boolean)
            )
          }
          className="bg-zinc-900 border-zinc-700"
          disabled={loading}
        />
      </div>

      <Button
        type="submit"
        className="w-full bg-zinc-100 text-zinc-950 hover:bg-zinc-200"
        disabled={loading}
      >
        {loading && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
        Save Preferences
      </Button>
    </form>
  )
}

export function PreferencesDialog({
  trigger,
  open,
  onOpenChange
}: {
  trigger?: React.ReactNode
  open?: boolean
  onOpenChange?: (open: boolean) => void
}) {
  const {
    data: userData,
    mutate: mutateUser,
    isLoading: isUserLoading
  } = useSWR('/api/user', fetchUser)
  const {
    data: prefData,
    mutate: mutatePref,
    isLoading: isPrefLoading
  } = useSWR('/api/user/pref', fetchPrefs)

  const renderAccountContent = () => {
    if (isUserLoading) {
      return (
        <div className="flex justify-center p-8">
          <Loader2 className="h-8 w-8 animate-spin text-zinc-500" />
        </div>
      )
    }
    if (userData) {
      return <AccountForm initialUser={userData} onSuccess={() => mutateUser()} />
    }
    return <div className="text-red-500">Failed to load user data</div>
  }

  const renderPreferencesContent = () => {
    if (isPrefLoading) {
      return (
        <div className="flex justify-center p-8">
          <Loader2 className="h-8 w-8 animate-spin text-zinc-500" />
        </div>
      )
    }
    if (prefData) {
      return <PreferencesForm initialPrefs={prefData} onSuccess={() => mutatePref()} />
    }
    return <div className="text-red-500">Failed to load preferences</div>
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      {trigger && <DialogTrigger asChild>{trigger}</DialogTrigger>}
      <DialogContent className="sm:max-w-[500px] bg-zinc-950 border-zinc-800 text-zinc-100">
        <DialogHeader>
          <DialogTitle>Settings</DialogTitle>
          <DialogDescription className="text-zinc-400">
            Manage your profile and movie preferences.
          </DialogDescription>
        </DialogHeader>
        <Tabs defaultValue="account" className="w-full">
          <TabsList className="grid w-full grid-cols-2 bg-zinc-900 border-zinc-800">
            <TabsTrigger value="account" className="text-white">
              Account
            </TabsTrigger>
            <TabsTrigger value="preferences" className="text-white">
              Preferences
            </TabsTrigger>
          </TabsList>

          <TabsContent value="account" className="space-y-4 pt-4">
            {renderAccountContent()}
          </TabsContent>

          <TabsContent value="preferences" className="space-y-4 pt-4">
            {renderPreferencesContent()}
          </TabsContent>
        </Tabs>
      </DialogContent>
    </Dialog>
  )
}
