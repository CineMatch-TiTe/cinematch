'use client'

import { useEffect, useTransition, useState } from 'react'
import { useRouter } from 'next/navigation'
import { toast } from 'sonner'
import Image from 'next/image'
import { LogOut, Play } from 'lucide-react'

import { Button } from '@/components/ui/button'
import { ActionConfirmationDialog } from '@/components/common/ActionConfirmationDialog'
import { PartyHeader } from '@/components/party/PartyHeader'
import { PartyMemberList } from '@/components/party/PartyMemberList'
import {
  kickMemberAction,
  leavePartyAction,
  promoteMemberAction,
  advancePhaseAction
} from '@/actions/party-room'
import { PartyResponse } from '@/model/partyResponse'
import { MemberInfo } from '@/model/memberInfo'
import { CurrentUserResponse } from '@/model/currentUserResponse'
import PickingFlow from './picking/PickingFlow'
import { usePartyView } from './PartyViewContext'

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
  const [advanceDialogOpen, setAdvanceDialogOpen] = useState(false)
  const { activeView, setActiveView } = usePartyView()

  const isLeader = party.leader_id === currentUser.user_id

  const handleLeaveClick = () => setLeaveDialogOpen(true)
  const handleAdvanceClick = () => setAdvanceDialogOpen(true)

  const confirmLeave = async () => {
    await leavePartyAction(party.id)
    setLeaveDialogOpen(false)
  }

  const confirmAdvance = async () => {
    startManualTransition(async () => {
      const result = await advancePhaseAction(party.id)
      if (result.error) toast.error(result.error)
      else toast.success('Phase advanced!')
      setAdvanceDialogOpen(false)
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

  // We render PickingFlow always to preserve state, but hide it when not active.
  // Same for the main party view content.
  const isPickingView = activeView === 'picking'

  const renderAdvanceButton = () => {
    if (!isLeader) return null

    let text = ''
    if (party.state === 'created') text = 'Start Picking'
    if (party.state === 'picking') text = 'Start Voting'

    if (!text) return null

    return (
      <Button
        size="lg"
        disabled={isManualPending}
        className="w-full font-semibold text-lg py-6 shadow-lg shadow-red-500/20 bg-red-600 hover:bg-red-700 text-white animate-in fade-in slide-in-from-bottom-4"
        onClick={handleAdvanceClick}
      >
        <Play className="mr-2 w-5 h-5 fill-current" /> {text}
      </Button>
    )
  }

  return (
    <>
      <div style={{ display: isPickingView ? 'block' : 'none' }}>
        <PickingFlow partyId={party.id} />
      </div>

      <div
        className="min-h-screen bg-zinc-950 font-sans text-zinc-100 selection:bg-red-500/30 flex flex-col items-center relative pb-32"
        style={{ display: isPickingView ? 'none' : 'flex' }}
      >
        <div className="fixed inset-0 z-0 pointer-events-none">
          <div className="absolute inset-0 bg-[radial-gradient(circle_at_center,var(--tw-gradient-stops))] from-zinc-800/20 via-zinc-950 to-zinc-950" />
        </div>

        <div className="w-full max-w-md p-4 flex-1 flex flex-col z-10 relative">
          <header className="flex flex-col items-center mb-6">
            <div className="flex flex-row items-center mb-2 gap-2">
              <Image src="/Logo.png" alt="Logo" width={32} height={32} />
              <h1 className="text-2xl font-bold tracking-tight text-white">Party Room</h1>
            </div>
            <PartyHeader partyCode={party.code} />
            <div className="mt-2 text-zinc-500 text-sm uppercase tracking-wider font-medium">
              {party.state} Phase
            </div>
          </header>

          <main className="flex-1 w-full relative space-y-6">
            {party.state === 'picking' && (
              <Button
                size="lg"
                className="w-full py-8 text-xl font-bold bg-linear-to-r from-indigo-500 to-purple-600 hover:from-indigo-600 hover:to-purple-700 shadow-xl shadow-indigo-500/20 border border-white/10"
                onClick={() => setActiveView('picking')}
              >
                <div className="flex flex-col items-center">
                  <span className="flex items-center gap-2">Match your movie taste</span>
                  <span className="text-xs font-normal opacity-80 mt-1 text-white/80">
                    Swipe movies to help us recommend better
                  </span>
                </div>
              </Button>
            )}

            <div>
              <h3 className="text-sm font-semibold text-zinc-500 mb-3 px-1 uppercase tracking-wider">
                Members ({members.length})
              </h3>
              <div
                className={isManualPending ? 'opacity-70 transition-opacity' : 'transition-opacity'}
              >
                <PartyMemberList
                  members={members}
                  loading={false}
                  currentUserId={currentUser.user_id}
                  isCurrentUserLeader={isLeader}
                  onKick={handleKick}
                  onPromote={handlePromote}
                />
              </div>
            </div>
          </main>

          <div className="w-full max-w-md p-4 mt-8 flex flex-col gap-3 z-20">
            {renderAdvanceButton()}

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

          <ActionConfirmationDialog
            open={leaveDialogOpen}
            onOpenChange={setLeaveDialogOpen}
            title="Leave Party?"
            description="Are you sure you want to leave this party? You will need to rejoin if you want to come back."
            confirmText="Leave"
            onConfirm={confirmLeave}
          />

          <ActionConfirmationDialog
            open={advanceDialogOpen}
            onOpenChange={setAdvanceDialogOpen}
            title="Advance Party Phase?"
            description={`Are you sure you want to advance the party state? This will move everyone to the next phase.`}
            confirmText="Advance"
            onConfirm={confirmAdvance}
          />
          <div className="h-32" />
        </div>
      </div>
    </>
  )
}
