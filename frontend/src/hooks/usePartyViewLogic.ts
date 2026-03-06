import { useState, useEffect, useTransition, useRef } from 'react'

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
  setActiveView: (view: 'picking' | 'voting' | 'watching' | 'room') => void
}

export function usePartyViewLogic({ party, currentUser, setActiveView }: UsePartyViewLogicProps) {
  const [isManualPending, startManualTransition] = useTransition()
  const [leaveDialogOpen, setLeaveDialogOpen] = useState(false)
  const [advanceDialogOpen, setAdvanceDialogOpen] = useState(false)
  const prevPartyState = useRef(party.state)


  // Effect to handle view switching based on party state
  useEffect(() => {
    if (prevPartyState.current !== party.state) {
      if (party.state === 'picking') {
        setActiveView('picking')
      } else if (party.state === 'voting') {
        setActiveView('voting')
      } else if (party.state === 'watching') {
        setActiveView('watching')
      }
      prevPartyState.current = party.state
    }
  }, [party.state, setActiveView])

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
    if (party.state === 'review' || party.state === 'watching') return 'New Round'
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
