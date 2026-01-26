'use client'

import { useEffect, useTransition, useState } from 'react'
import { useRouter } from 'next/navigation'
import { toast } from 'sonner'
import { LogOut, Play } from 'lucide-react'

import { Button } from '@/components/ui/button'
import { ActionConfirmationDialog } from '@/components/common/ActionConfirmationDialog'
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

  const [leaveDialogOpen, setLeaveDialogOpen] = useState(false)
  const [voteDialogOpen, setVoteDialogOpen] = useState(false)

  const isLeader = party.leader_id === currentUser.user_id

  const handleLeaveClick = () => setLeaveDialogOpen(true)
  const handleVoteClick = () => setVoteDialogOpen(true)

  const confirmLeave = async () => {
    await leavePartyAction(party.id)
    setLeaveDialogOpen(false)
  }

  const confirmVote = async () => {
    startManualTransition(async () => {
      const result = await startVotingAction(party.id)
      if (result.error) toast.error(result.error)
      else toast.success('Voting started!')
      setVoteDialogOpen(false)
    })
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

  return (
    <div className="min-h-screen bg-zinc-950 font-sans text-zinc-100 selection:bg-red-500/30 flex flex-col items-center relative">
      <div className="fixed inset-0 z-0 pointer-events-none">
        <div className="absolute inset-0 bg-[radial-gradient(circle_at_center,var(--tw-gradient-stops))] from-zinc-800/20 via-zinc-950 to-zinc-950" />
      </div>

      <div className="w-full max-w-md p-4 flex-1 flex flex-col z-10 relative">
        <header className="flex flex-col items-center mb-6">
          <h1 className="text-2xl font-bold tracking-tight mb-2 text-white">Party Room</h1>
          <PartyHeader partyCode={party.code} />
        </header>

        <main className="flex-1 w-full relative">
          <h3 className="text-sm font-semibold text-zinc-500 mb-3 px-1 uppercase tracking-wider">
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

        <footer className="fixed bottom-0 left-0 right-0 p-4 bg-zinc-950/80 backdrop-blur-md border-t border-zinc-900 flex justify-center z-20">
          <div className="w-full max-w-md flex flex-col gap-3">
            {isLeader && (
              <Button
                size="lg"
                disabled={isManualPending}
                className="w-full font-semibold text-lg py-6 shadow-lg shadow-red-500/20 bg-red-600 hover:bg-red-700 text-white animate-in fade-in slide-in-from-bottom-4"
                onClick={handleVoteClick}
              >
                <Play className="mr-2 w-5 h-5 fill-current" /> Start Voting
              </Button>
            )}

            <Button
              variant="ghost"
              size="lg"
              disabled={isManualPending}
              className="w-full text-zinc-400 hover:text-red-500 hover:bg-red-500/10"
              onClick={handleLeaveClick}
            >
              <LogOut className="mr-2 w-4 h-4" /> Leave Party
            </Button>
          </div>
        </footer>
        <div className="h-32" />

        <ActionConfirmationDialog
          open={leaveDialogOpen}
          onOpenChange={setLeaveDialogOpen}
          title="Leave Party?"
          description="Are you sure you want to leave this party? You will need to rejoin if you want to come back."
          confirmText="Leave"
          variant="destructive"
          onConfirm={confirmLeave}
        />

        <ActionConfirmationDialog
          open={voteDialogOpen}
          onOpenChange={setVoteDialogOpen}
          title="Start Voting?"
          description="Are you sure you want to start the voting phase? Ensure all members have joined."
          confirmText="Start Voting"
          onConfirm={confirmVote}
        />
        <div className="h-32" />
      </div>
    </div>
  )
}
