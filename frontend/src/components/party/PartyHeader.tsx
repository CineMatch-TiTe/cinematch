'use client'

import { Button } from '@/components/ui/button'
import { Copy, Share2 } from 'lucide-react'
import { toast } from 'sonner'

interface PartyHeaderProps {
  partyCode: string | null | undefined
}

export function PartyHeader({ partyCode }: PartyHeaderProps) {
  const handleShare = async () => {
    if (!partyCode) return

    const url = `${globalThis.location.origin}/?joinCode=${partyCode}`

    try {
      await navigator.clipboard.writeText(url)
      toast.success('Link copied to clipboard!')
    } catch (err) {
      console.error('Failed to copy:', err)
      toast.error('Failed to copy link')
    }
  }

  const handleCopyCode = async () => {
    if (!partyCode) return
    try {
      await navigator.clipboard.writeText(partyCode)
      toast.success('Code copied to clipboard!')
    } catch (err) {
      console.error('Failed to copy:', err)
      toast.error('Failed to copy code')
    }
  }

  if (!partyCode) return null

  return (
    <div className="flex flex-col gap-4 items-center justify-center py-6 w-full">
      <div className="flex flex-col items-center gap-2">
        <h2 className="text-sm font-medium text-muted-foreground uppercase tracking-wider">
          Party Code
        </h2>
        <button
          onClick={handleCopyCode}
          type="button"
          className="flex items-center gap-2 text-4xl font-black tracking-widest cursor-pointer hover:opacity-80 transition-opacity bg-transparent border-none p-0"
        >
          {partyCode}
          <Copy className="w-5 h-5 text-muted-foreground" />
        </button>
      </div>

      <Button variant="outline" className="gap-2 rounded-full" onClick={handleShare}>
        <Share2 className="w-4 h-4" />
        Share Invite Link
      </Button>
    </div>
  )
}
