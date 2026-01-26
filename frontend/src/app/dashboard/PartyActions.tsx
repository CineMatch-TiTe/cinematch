'use client'

import { useTransition } from 'react'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Users, Plus, Loader2 } from 'lucide-react'
import { JoinPartyDialog } from './JoinPartyDialog'
import { createPartyInstantAction } from '@/actions/dashboard'
import { toast } from 'sonner'

export function PartyActions() {
  const [isCreating, startTransition] = useTransition()

  const handleCreateParty = () => {
    startTransition(async () => {
      const result = await createPartyInstantAction()
      if (result?.error) {
        toast.error(result.error)
      }
    })
  }

  return (
    <div className="grid md:grid-cols-2 gap-6 animate-in fade-in zoom-in duration-500">
      <Card className="bg-zinc-900/50 border-zinc-800 hover:border-red-900/50 transition-all hover:bg-zinc-900/80 group">
        <CardHeader>
          <CardTitle className="flex items-center gap-2 text-zinc-100">
            <Plus className="h-5 w-5 text-red-500 group-hover:text-red-400 transition-colors" />
            Host a Party
          </CardTitle>
          <CardDescription className="text-zinc-400">
            Start a new party and invite your friends to swipe together.
          </CardDescription>
        </CardHeader>
        <CardContent>
          <Button
            onClick={handleCreateParty}
            disabled={isCreating}
            className="w-full bg-red-900 hover:bg-red-800 text-white shadow-lg shadow-red-900/20"
          >
            {isCreating ? (
              <>
                <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                Creating...
              </>
            ) : (
              'Create New Party'
            )}
          </Button>
        </CardContent>
      </Card>

      <Card className="bg-zinc-900/50 border-zinc-800 hover:border-zinc-700 transition-all hover:bg-zinc-900/80 group">
        <CardHeader>
          <CardTitle className="flex items-center gap-2 text-zinc-100">
            <Users className="h-5 w-5 text-zinc-500 group-hover:text-zinc-400 transition-colors" />
            Join a Party
          </CardTitle>
          <CardDescription className="text-zinc-400">
            Have a code? Join an existing party and start matching.
          </CardDescription>
        </CardHeader>
        <CardContent>
          <JoinPartyDialog
            trigger={
              <Button
                variant="secondary"
                className="w-full bg-zinc-800 hover:bg-zinc-700 text-zinc-100 border border-zinc-700"
              >
                Join Existing Party
              </Button>
            }
          />
        </CardContent>
      </Card>
    </div>
  )
}
