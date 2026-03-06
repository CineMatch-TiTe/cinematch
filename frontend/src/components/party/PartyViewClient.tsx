'use client'

import { useState, useEffect } from 'react'

import { PartyResponse } from '@/model/partyResponse'
import { MemberInfo } from '@/model/memberInfo'
import { CurrentUserResponse } from '@/model/currentUserResponse'

import VotingFlow from './voting/VotingFlow'
import PickingFlow from './picking/PickingFlow'
import WatchingFlow from './watching/WatchingFlow'
import { PartyRoom } from './PartyRoom'

import { usePartyView } from './PartyViewContext'
import { usePartyViewLogic } from '@/hooks/usePartyViewLogic'
import { useDeadlineCountdown } from '@/hooks/useDeadlineCountdown'
import { setReadyAction } from '@/actions/party-room'
import { toast } from 'sonner'
import { usePartySocket } from '@/hooks/usePartySocket'

interface PartyViewClientProps {
    party: PartyResponse
    members: MemberInfo[]
    currentUser: CurrentUserResponse
}

export default function PartyViewClient({
    currentUser
}: Readonly<PartyViewClientProps>) {
    const { activeView, setActiveView, party, members, handleWsMessage } = usePartyView()
    const [mounted, setMounted] = useState(false)
    useEffect(() => { setMounted(true) }, [])

    usePartySocket({
        partyId: party.id,
        onMessage: handleWsMessage
    })

    const {
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
    } = usePartyViewLogic({ party, currentUser, setActiveView })

    const currentMember = members.find(m => m.user_id === currentUser.user_id)
    const serverReady = currentMember?.is_ready ?? false
    const [optimisticReady, setOptimisticReady] = useState(serverReady)
    const showReadyButton = party.state === 'created' || party.state === 'picking'

    // Sync optimistic state from server when it changes
    useEffect(() => { setOptimisticReady(serverReady) }, [serverReady])

    const readyCount = members.filter(m => m.user_id === currentUser.user_id ? optimisticReady : m.is_ready).length
    const allReady = members.length > 0 && readyCount === members.length

    const handleReadyToggle = async () => {
        const next = !optimisticReady
        setOptimisticReady(next) // instant flip
        const result = await setReadyAction(party.id, next)
        if (result.error) {
            setOptimisticReady(!next) // revert on error
            toast.error(result.error)
        }
    }

    const isPickingView = activeView === 'picking'
    const isVotingView = activeView === 'voting'
    const isWatchingView = activeView === 'watching'
    const isPartyView = activeView === 'room'

    // Phase transition countdown for "All Ready"
    const transitionSecondsLeft = useDeadlineCountdown(party.ready_deadline_at)
    const showAllReadyCountdown = !!(party.ready_deadline_at && transitionSecondsLeft > 0)

    if (!mounted) {
        return <div className="min-h-screen bg-zinc-950" />
    }

    return (
        <>
            <div
                style={{
                    display:
                        isPickingView && party.state !== 'voting' && party.state !== 'watching'
                            ? 'block'
                            : 'none'
                }}
            >
                <PickingFlow partyId={party.id} />
            </div>

            {party.state === 'voting' && (
                <div style={{ display: isVotingView ? 'block' : 'none' }}>
                    <VotingFlow
                        partyId={party.id}
                        phaseEnteredAt={party.phase_entered_at}
                        timeoutSecs={party.voting_timeout_secs}
                        deadlineAt={party.ready_deadline_at}
                    />
                </div>
            )}

            {party.state === 'watching' && party.selected_movie_id && (
                <div style={{ display: isWatchingView ? 'block' : 'none' }}>
                    <WatchingFlow
                        movieId={party.selected_movie_id}
                        phaseEnteredAt={party.phase_entered_at}
                        timeoutSecs={party.watching_timeout_secs}
                        deadlineAt={party.ready_deadline_at}
                    />
                </div>
            )}

            <div
                className="min-h-screen bg-zinc-950 font-sans text-zinc-100 selection:bg-red-500/30 flex flex-col items-center relative pb-32"
                style={{ display: isPartyView ? 'flex' : 'none' }}
            >
                <div className="fixed inset-0 z-0 pointer-events-none">
                    <div className="absolute inset-0 bg-[radial-gradient(circle_at_center,var(--tw-gradient-stops))] from-zinc-800/20 via-zinc-950 to-zinc-950" />
                </div>

                <PartyRoom
                    party={party}
                    members={members}
                    currentUser={currentUser}
                    isLeader={isLeader}
                    isManualPending={isManualPending}
                    advanceButtonText={getAdvanceButtonText()}
                    onAdvanceClick={handleAdvanceClick}
                    onKick={handleKick}
                    onPromote={handlePromote}
                    showReadyButton={showReadyButton}
                    showAllReadyCountdown={showAllReadyCountdown}
                    allReady={allReady}
                    readyCount={readyCount}
                    transitionSecondsLeft={transitionSecondsLeft}
                    optimisticReady={optimisticReady}
                    onReadyToggle={handleReadyToggle}
                    onLeaveClick={handleLeaveClick}
                    leaveDialogOpen={leaveDialogOpen}
                    setLeaveDialogOpen={setLeaveDialogOpen}
                    confirmLeave={confirmLeave}
                    advanceDialogOpen={advanceDialogOpen}
                    setAdvanceDialogOpen={setAdvanceDialogOpen}
                    confirmAdvance={confirmAdvance}
                />
            </div>
        </>
    )
}
