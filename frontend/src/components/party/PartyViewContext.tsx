'use client'

import { createContext, useContext, useState, useRef, ReactNode, useMemo, useCallback } from 'react'
import { useRouter } from 'next/navigation'
import { toast } from 'sonner'
import { PartyResponse } from '@/model/partyResponse'
import { MemberInfo } from '@/model/memberInfo'
import { CurrentUserResponse } from '@/model/currentUserResponse'
import { PartyResponseReviewRatings } from '@/model/partyResponseReviewRatings'
import { ServerMessage } from '@/lib/ws-types'

export type PartyViewType = 'room' | 'picking' | 'voting' | 'watching' | 'review'

interface PartyViewContextType {
  activeView: PartyViewType
  setActiveView: (view: PartyViewType) => void
  partyState: string
  party: PartyResponse
  members: MemberInfo[]
  currentUser: CurrentUserResponse
  handleWsMessage: (msg: ServerMessage) => void
  lastMessage: ServerMessage | null
  consumeLivePhaseTransition: () => string | null
  reviewAverage: number | null
}

const PartyViewContext = createContext<PartyViewContextType | undefined>(undefined)

interface PartyViewProviderProps {
  children: ReactNode
  initialView?: PartyViewType
  initialParty: PartyResponse
  initialMembers: MemberInfo[]
  currentUser: CurrentUserResponse
}

export function PartyViewProvider({
  children,
  initialView = 'room',
  initialParty,
  initialMembers,
  currentUser
}: Readonly<PartyViewProviderProps>) {
  const router = useRouter()
  const [activeView, setActiveView] = useState<PartyViewType>(initialView)
  const [party, setParty] = useState<PartyResponse>(initialParty)
  const [members, setMembers] = useState<MemberInfo[]>(initialMembers)
  const [lastMessage, setLastMessage] = useState<ServerMessage | null>(null)
  const [reviewAverage, setReviewAverage] = useState<number | null>(null)
  const livePhaseTransitionRef = useRef<string | null>(null)

  const consumeLivePhaseTransition = useCallback(() => {
    const value = livePhaseTransitionRef.current
    livePhaseTransitionRef.current = null
    return value
  }, [])

  const handleWsMessage = useCallback((msg: ServerMessage) => {
    setLastMessage(msg)

    if (typeof msg === 'string') {
      if (msg === 'PartyDisbanded') {
        toast.info('The party has been disbanded')
        router.push('/dashboard')
      } else if (msg === 'ResetReadiness') {
        setMembers((prev) => prev.map((m) => ({ ...m, is_ready: false })))
      }
      return
    }

    if ('PartyStateChanged' in msg) {
      const payload = msg.PartyStateChanged
      livePhaseTransitionRef.current = payload.state
      setParty((prev) => ({
        ...prev,
        state: payload.state,
        ready_deadline_at: payload.deadline_at ?? null,
        // Reset phase_entered_at so countdown components don't flash
        // the previous phase's time before PartyTimeoutUpdate arrives.
        phase_entered_at: new Date().toISOString(),
        ...(payload.selected_movie_id !== undefined && {
          selected_movie_id: payload.selected_movie_id ?? null,
        }),
        ...(payload.review_ratings !== undefined && {
          review_ratings: (payload.review_ratings as PartyResponseReviewRatings) ?? null,
        }),
        // batch voting_round if present
        ...(payload.voting_round !== undefined && {
          voting_round: payload.voting_round,
        }),
        // invalidate party code if not in created phase
        ...(payload.state !== 'created' && {
          code: null,
        }),
      }))
      // Immediately switch the active view to match the new phase
      if (payload.state === 'picking') setActiveView('picking')
      else if (payload.state === 'voting') setActiveView('voting')
      else if (payload.state === 'watching') setActiveView('watching')
      else if (payload.state === 'review') setActiveView('review')
      else if (payload.state === 'created') setActiveView('room')
    } else if ('PartyMemberJoined' in msg) {
      const payload = msg.PartyMemberJoined
      setMembers((prev) => {
        if (prev.some((m) => m.user_id === payload.user_id)) return prev
        return [
          ...prev,
          {
            user_id: payload.user_id,
            username: payload.username,
            is_leader: false,
            is_ready: false,
            joined_at: new Date().toISOString(),
          },
        ]
      })
    } else if ('PartyMemberLeft' in msg) {
      const userId = msg.PartyMemberLeft
      setMembers((prev) => prev.filter((m) => m.user_id !== userId))
    } else if ('PartyLeaderChanged' in msg) {
      const newLeaderId = msg.PartyLeaderChanged
      setParty((prev) => ({ ...prev, leader_id: newLeaderId }))
      setMembers((prev) =>
        prev.map((m) => ({ ...m, is_leader: m.user_id === newLeaderId }))
      )
    } else if ('UpdateReadyState' in msg) {
      const payload = msg.UpdateReadyState
      setMembers((prev) =>
        prev.map((m) =>
          m.user_id === payload.user_id ? { ...m, is_ready: payload.ready } : m
        )
      )
    } else if ('PartyTimeoutUpdate' in msg) {
      const payload = msg.PartyTimeoutUpdate
      setParty((prev) => {
        // Map timeout_secs to the correct phase-specific field
        const timeoutUpdates: Partial<PartyResponse> = {}
        if (payload.timeout_secs != null) {
          if (prev.state === 'voting') {
            timeoutUpdates.voting_timeout_secs = payload.timeout_secs
          } else if (prev.state === 'watching') {
            timeoutUpdates.watching_timeout_secs = payload.timeout_secs
          }
        }

        return {
          ...prev,
          ...timeoutUpdates,
          phase_entered_at: payload.phase_entered_at ?? prev.phase_entered_at,
          // If deadline_at is present in the payload, use it.
          // If absent but phase_entered_at is present, this is a phase-info update — keep existing deadline.
          // If both absent (empty cancel signal), clear the deadline.
          ready_deadline_at:
            payload.deadline_at !== undefined
              ? payload.deadline_at
              : payload.phase_entered_at !== undefined
                ? prev.ready_deadline_at
                : null,
        }
      })
    } else if ('PartyCodeChanged' in msg) {
      const newCode = msg.PartyCodeChanged
      setParty((prev) => ({ ...prev, code: newCode }))
    } else if ('NameChanged' in msg) {
      const payload = msg.NameChanged
      setMembers((prev) =>
        prev.map((m) =>
          m.user_id === payload.user_id ? { ...m, username: payload.new_name } : m
        )
      )
    } else if ('PartyMemberRated' in msg) {
      const payload = msg.PartyMemberRated
      setParty((prev) => ({
        ...prev,
        review_ratings: {
          ...(prev.review_ratings || {}),
          [payload.user_id]: payload.rating,
        },
      }))
      setReviewAverage(payload.party_average)
    }
  }, [router])

  const value = useMemo(
    () => ({
      activeView,
      setActiveView,
      partyState: party.state,
      party,
      members,
      currentUser,
      handleWsMessage,
      lastMessage,
      consumeLivePhaseTransition,
      reviewAverage,
    }),
    [activeView, party, members, currentUser, handleWsMessage, lastMessage, consumeLivePhaseTransition, reviewAverage]
  )

  return <PartyViewContext.Provider value={value}>{children}</PartyViewContext.Provider>
}

export function usePartyView() {
  const context = useContext(PartyViewContext)
  if (context === undefined) {
    throw new Error('usePartyView must be used within a PartyViewProvider')
  }
  return context
}
