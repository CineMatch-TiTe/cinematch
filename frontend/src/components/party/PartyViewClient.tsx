'use client'

import { useEffect, useTransition } from 'react'
import { useRouter } from 'next/navigation'
import { toast } from 'sonner'
import { LogOut, Play } from 'lucide-react'

import { Button } from '@/components/ui/button'
import { PartyHeader } from '@/components/party/PartyHeader'
import { PartyMemberList } from '@/components/party/PartyMemberList'
import {
  kickMemberAction,
  leavePartyAction,
  promoteMemberAction,
  startVotingAction
} from '@/actions/party-room'
import { PartyResponse } from '@/model/partyResponse'
import { MemberInfo } from '@/model/memberInfo'
import { CurrentUserResponse } from '@/model/currentUserResponse'

interface PartyViewClientProps {
  party: PartyResponse
  members: MemberInfo[]
  currentUser: CurrentUserResponse
}

export default function PartyViewClient({
  party,
  members,
  currentUser
}: Readonly<PartyViewClientProps>) {
  const router = useRouter()
  const [isManualPending, startManualTransition] = useTransition()
  const [, startPollingTransition] = useTransition()
  useEffect(() => {
    const interval = setInterval(() => {
      startPollingTransition(() => {
        router.refresh()
      })
    }, 5000)

    return () => clearInterval(interval)
  }, [router])

  const isLeader = party.leader_id === currentUser.user_id

  const handleLeave = async () => {
    await leavePartyAction(party.id)
  }

  const handleKick = async (memberId: string) => {
    startManualTransition(async () => {
      const result = await kickMemberAction(party.id, memberId)
      if (result.error) toast.error(result.error)
      else toast.success('Member kicked')
    })
  }

  const handlePromote = async (memberId: string) => {
    startManualTransition(async () => {
      const result = await promoteMemberAction(party.id, memberId)
      if (result.error) toast.error(result.error)
      else toast.success('Leadership transferred')
    })
  }

  const handleStartVoting = async () => {
    startManualTransition(async () => {
      const result = await startVotingAction(party.id)
      if (result.error) toast.error(result.error)
      else toast.success('Voting started!')
    })
  }

  return (
    <div className="min-h-screen bg-background flex flex-col items-center">
      <div className="w-full max-w-md p-4 flex-1 flex flex-col">
        <header className="flex flex-col items-center mb-6">
          <h1 className="text-2xl font-bold tracking-tight mb-2">Party Room</h1>
          <PartyHeader partyCode={party.code} />
        </header>

        <main className="flex-1 w-full relative">
          <h3 className="text-sm font-semibold text-muted-foreground mb-3 px-1 uppercase tracking-wider">
            Members ({members.length})
          </h3>
          <div className={isManualPending ? 'opacity-70 transition-opacity' : 'transition-opacity'}>
            <PartyMemberList
              members={members}
              loading={false}
              currentUserId={currentUser.user_id}
              isCurrentUserLeader={isLeader}
              onKick={handleKick}
              onPromote={handlePromote}
            />
          </div>
        </main>

        <footer className="fixed bottom-0 left-0 right-0 p-4 bg-background/80 backdrop-blur-sm border-t flex justify-center">
          <div className="w-full max-w-md flex flex-col gap-3">
            {isLeader && (
              <Button
                size="lg"
                disabled={isManualPending}
                className="w-full font-semibold text-lg py-6 shadow-lg animate-in fade-in slide-in-from-bottom-4"
                onClick={handleStartVoting}
              >
                <Play className="mr-2 w-5 h-5 fill-current" /> Start Voting
              </Button>
            )}

            <Button
              variant="ghost"
              size="lg"
              disabled={isManualPending}
              className="w-full text-muted-foreground hover:text-destructive hover:bg-destructive/10"
              onClick={handleLeave}
            >
              <LogOut className="mr-2 w-4 h-4" /> Leave Party
            </Button>
          </div>
        </footer>
        <div className="h-32" />
      </div>
    </div>
  )
}
