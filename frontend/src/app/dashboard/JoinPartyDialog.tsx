'use client'

import { useState, useTransition } from 'react'
import { Button } from '@/components/ui/button'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger
} from '@/components/ui/dialog'
import { Input } from '@/components/ui/input'
import { joinPartyInstantAction } from '@/actions/dashboard'
import { Loader2 } from 'lucide-react'
import { toast } from 'sonner'

interface JoinPartyDialogProps {
  trigger: React.ReactNode
}

export function JoinPartyDialog({ trigger }: JoinPartyDialogProps) {
  const [code, setCode] = useState('')
  const [isOpen, setIsOpen] = useState(false)
  const [isPending, startTransition] = useTransition()

  const handleJoin = () => {
    if (!code) return

    startTransition(async () => {
      const result = await joinPartyInstantAction(code)
      if (result?.error) {
        toast.error(result.error)
      } else {
        // Redirect handles the rest, but we can close dialog
        setIsOpen(false)
      }
    })
  }

  return (
    <Dialog open={isOpen} onOpenChange={setIsOpen}>
      <DialogTrigger asChild>{trigger}</DialogTrigger>
      <DialogContent className="bg-zinc-900 border-zinc-800 text-zinc-100 sm:max-w-md">
        <DialogHeader>
          <DialogTitle>Join a Party</DialogTitle>
          <DialogDescription className="text-zinc-400">
            Enter the party code to join your friends.
          </DialogDescription>
        </DialogHeader>
        <div className="flex items-center space-x-2 py-4">
          <Input
            id="code"
            placeholder="Party Code"
            value={code}
            onChange={(e) => setCode(e.target.value.toUpperCase())}
            className="flex-1 bg-zinc-950/50 border-zinc-700 text-zinc-100 placeholder:text-zinc-500 focus-visible:ring-red-600 focus-visible:border-red-600"
            onKeyDown={(e) => {
              if (e.key === 'Enter') handleJoin()
            }}
          />
        </div>
        <DialogFooter className="sm:justify-end">
          <Button
            type="button"
            variant="secondary"
            onClick={() => setIsOpen(false)}
            className="bg-zinc-800 hover:bg-zinc-700 text-zinc-100"
          >
            Cancel
          </Button>
          <Button
            type="button"
            onClick={handleJoin}
            disabled={isPending || !code}
            className="bg-red-900 hover:bg-red-800 text-white"
          >
            {isPending ? (
              <>
                <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                Joining...
              </>
            ) : (
              'Join'
            )}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
