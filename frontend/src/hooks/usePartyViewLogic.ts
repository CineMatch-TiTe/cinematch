import { useState, useTransition } from 'react'

import { toast } from 'sonner'
import {
  kickMemberAction,
  leavePartyAction,
  promoteMemberAction,
  advancePhaseAction
} from '@/actions/party-room'
import { PartyResponse } from '@/model/partyResponse'
import { CurrentUserResponse } from '@/model/currentUserResponse'

interface UsePartyViewLogicProps {
  party: PartyResponse
  currentUser: CurrentUserResponse
}

export function usePartyViewLogic({ party, currentUser }: UsePartyViewLogicProps) {
  const [isManualPending, startManualTransition] = useTransition()
  const [leaveDialogOpen, setLeaveDialogOpen] = useState(false)
  const [advanceDialogOpen, setAdvanceDialogOpen] = useState(false)

  // View switching is handled directly in PartyViewContext.handleWsMessage
  // for immediate reactivity when WebSocket state changes arrive.

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

  const getAdvanceButtonText = () => {
    if (!isLeader) return null
    if (party.state === 'created') return 'Start Picking'
    if (party.state === 'picking') return 'Start Voting'
    if (party.state === 'voting') return 'Skip Phase'
    if (party.state === 'watching') return 'Skip to Review'
    if (party.state === 'review') return 'New Round'
    return null
  }

  return {
    isManualPending,
    leaveDialogOpen,
    setLeaveDialogOpen,
    advanceDialogOpen,
    setAdvanceDialogOpen,
    isLeader,
    handleLeaveClick,
    handleAdvanceClick,
    confirmLeave,
    confirmAdvance,
    handleKick,
    handlePromote,
    getAdvanceButtonText
  }
}
