import { LogOut } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { ActionConfirmationDialog } from '@/components/common/ActionConfirmationDialog'
import { PartyMemberList } from './PartyMemberList'
import { PartyRoomHeader } from './PartyRoomHeader'
import { PartyReadyControls } from './PartyReadyControls'
import { PartyResponse } from '@/model/partyResponse'
import { MemberInfo } from '@/model/memberInfo'
import { CurrentUserResponse } from '@/model/currentUserResponse'

export interface PartyRoomProps {
    party: PartyResponse
    members: MemberInfo[]
    currentUser: CurrentUserResponse
    isLeader: boolean
    isManualPending: boolean
    advanceButtonText?: string | null
    onAdvanceClick: () => void
    onKick: (userId: string) => void
    onPromote: (userId: string) => void
    
    showReadyButton: boolean
    showAllReadyCountdown?: boolean | null
    allReady: boolean
    readyCount: number
    transitionSecondsLeft: number
    optimisticReady: boolean
    onReadyToggle: () => void

    onLeaveClick: () => void
    leaveDialogOpen: boolean
    setLeaveDialogOpen: (open: boolean) => void
    confirmLeave: () => void

    advanceDialogOpen: boolean
    setAdvanceDialogOpen: (open: boolean) => void
    confirmAdvance: () => void
}

export function PartyRoom({
    party, members, currentUser, isLeader, isManualPending, 
    advanceButtonText, onAdvanceClick, onKick, onPromote,
    showReadyButton, showAllReadyCountdown, allReady, readyCount,
    transitionSecondsLeft, optimisticReady, onReadyToggle,
    onLeaveClick, leaveDialogOpen, setLeaveDialogOpen, confirmLeave,
    advanceDialogOpen, setAdvanceDialogOpen, confirmAdvance
}: Readonly<PartyRoomProps>) {
    return (
        <div className="w-full max-w-md p-4 flex-1 flex flex-col z-10 relative">
            <PartyRoomHeader
                party={party}
                isLeader={isLeader}
                isManualPending={isManualPending}
                advanceButtonText={advanceButtonText}
                onAdvanceClick={onAdvanceClick}
            />

            <main className="flex-1 w-full relative space-y-6">
                <div>
                    <h3 className="text-sm font-semibold text-zinc-500 mb-3 px-1 uppercase tracking-wider">
                        Members ({members.length})
                    </h3>
                    <div className={isManualPending ? 'opacity-70 transition-opacity' : 'transition-opacity'}>
                        <PartyMemberList
                            members={members}
                            loading={false}
                            currentUserId={currentUser.user_id}
                            isCurrentUserLeader={isLeader}
                            onKick={onKick}
                            onPromote={onPromote}
                        />
                    </div>
                </div>
            </main>

            <div className="w-full max-w-md p-4 mt-8 flex flex-col gap-3 z-20">
                {showReadyButton && (
                    <PartyReadyControls
                        showAllReadyCountdown={showAllReadyCountdown}
                        allReady={allReady}
                        readyCount={readyCount}
                        memberCount={members.length}
                        transitionSecondsLeft={transitionSecondsLeft}
                        optimisticReady={optimisticReady}
                        onReadyToggle={onReadyToggle}
                    />
                )}

                <Button
                    variant="ghost"
                    size="lg"
                    disabled={isManualPending}
                    className="w-full text-zinc-400 hover:text-red-500 hover:bg-red-500/10"
                    onClick={onLeaveClick}
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
    )
}
