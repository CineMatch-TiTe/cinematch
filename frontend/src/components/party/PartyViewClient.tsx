'use client'

import { useState, useEffect } from 'react'

import Image from 'next/image'
import { LogOut, Play, Settings, CheckCircle2, XCircle } from 'lucide-react'

import { Button } from '@/components/ui/button'
import { ActionConfirmationDialog } from '@/components/common/ActionConfirmationDialog'
import { PartyHeader } from '@/components/party/PartyHeader'
import { PartyMemberList } from '@/components/party/PartyMemberList'
import { PreferencesDialog } from '@/components/preferences/PreferencesDialog'
import { PartyResponse } from '@/model/partyResponse'
import { MemberInfo } from '@/model/memberInfo'
import { CurrentUserResponse } from '@/model/currentUserResponse'
import VotingFlow from './voting/VotingFlow'
import PickingFlow from './picking/PickingFlow'
import WatchingFlow from './watching/WatchingFlow'
import { usePartyView } from './PartyViewContext'
import { usePartyViewLogic } from '@/hooks/usePartyViewLogic'
import { setReadyAction } from '@/actions/party-room'
import { toast } from 'sonner'

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
    const { activeView, setActiveView } = usePartyView()

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

    const renderAdvanceButton = () => {
        const text = getAdvanceButtonText()

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
                    <VotingFlow partyId={party.id} phaseEnteredAt={party.phase_entered_at} timeoutSecs={party.voting_timeout_secs} />
                </div>
            )}

            {party.state === 'watching' && party.selected_movie_id && (
                <div style={{ display: isWatchingView ? 'block' : 'none' }}>
                    <WatchingFlow movieId={party.selected_movie_id} phaseEnteredAt={party.phase_entered_at} timeoutSecs={party.watching_timeout_secs} />
                </div>
            )}

            <div
                className="min-h-screen bg-zinc-950 font-sans text-zinc-100 selection:bg-red-500/30 flex flex-col items-center relative pb-32"
                style={{ display: isPartyView ? 'flex' : 'none' }}
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
                        <div className="absolute top-4 right-4 z-50">
                            <PreferencesDialog
                                trigger={
                                    <Button
                                        variant="ghost"
                                        size="icon"
                                        className="text-zinc-500 hover:text-white hover:bg-zinc-800"
                                    >
                                        <Settings className="h-5 w-5" />
                                        <span className="sr-only">Settings</span>
                                    </Button>
                                }
                            />
                        </div>
                        <PartyHeader partyCode={party.code} />
                        <div className="mt-2 text-zinc-500 text-sm uppercase tracking-wider font-medium">
                            {party.state} Phase
                        </div>
                    </header>

                    <main className="flex-1 w-full relative space-y-6">
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
                        {showReadyButton && (
                            <>
                                <div className="text-center text-sm text-zinc-500 font-medium">
                                    {allReady ? (
                                        <span className="text-emerald-400 animate-pulse font-semibold">All Ready! Starting soon...</span>
                                    ) : (
                                        <span>{readyCount}/{members.length} ready</span>
                                    )}
                                </div>
                                <Button
                                    size="lg"
                                    onClick={handleReadyToggle}
                                    className={`w-full font-semibold text-lg py-6 shadow-lg transition-all ${optimisticReady
                                        ? 'bg-red-600 hover:bg-red-700 text-white shadow-red-500/20'
                                        : 'bg-emerald-600 hover:bg-emerald-500 text-white shadow-emerald-500/20'
                                        }`}
                                >
                                    {optimisticReady ? (
                                        <><XCircle className="mr-2 w-5 h-5" /> Unready</>
                                    ) : (
                                        <><CheckCircle2 className="mr-2 w-5 h-5" /> I&apos;m Ready</>
                                    )}
                                </Button>
                            </>
                        )}

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
