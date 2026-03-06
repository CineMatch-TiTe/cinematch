import { CheckCircle2, XCircle } from 'lucide-react'
import { Button } from '@/components/ui/button'
import PhaseCountdown from './PhaseCountdown'

export interface PartyReadyControlsProps {
    showAllReadyCountdown?: boolean | null
    allReady: boolean
    readyCount: number
    memberCount: number
    transitionSecondsLeft: number
    optimisticReady: boolean
    onReadyToggle: () => void
    readyLabel?: string
    unreadyLabel?: string
}

export function PartyReadyControls({
    showAllReadyCountdown,
    allReady,
    readyCount,
    memberCount,
    transitionSecondsLeft,
    optimisticReady,
    onReadyToggle,
    readyLabel = "I'm Ready",
    unreadyLabel = 'Unready'
}: Readonly<PartyReadyControlsProps>) {
    return (
        <>
            <div className="text-center text-sm text-zinc-500 font-medium">
                {showAllReadyCountdown && (
                    <div className="flex flex-col items-center gap-1">
                        <span className="text-emerald-400 font-semibold">Everyone Ready!</span>
                        <div className="flex items-center gap-2 text-zinc-400">
                            <span>Starting in</span>
                            <div className="scale-75 origin-center">
                                <PhaseCountdown
                                    phaseEnteredAt={new Date().toISOString()}
                                    timeoutSecs={transitionSecondsLeft}
                                />
                            </div>
                        </div>
                    </div>
                )}
                {!showAllReadyCountdown && allReady && (
                    <span className="text-emerald-400 animate-pulse font-semibold">All Ready! Starting soon...</span>
                )}
                {!showAllReadyCountdown && !allReady && (
                    <span>{readyCount}/{memberCount} ready</span>
                )}
            </div>
            <Button
                size="lg"
                onClick={onReadyToggle}
                className={`w-full font-semibold text-lg py-6 shadow-lg transition-all ${
                    optimisticReady
                        ? 'bg-red-600 hover:bg-red-700 text-white shadow-red-500/20'
                        : 'bg-emerald-600 hover:bg-emerald-500 text-white shadow-emerald-500/20'
                }`}
            >
                {optimisticReady ? (
                    <><XCircle className="mr-2 w-5 h-5" /> {unreadyLabel}</>
                ) : (
                    <><CheckCircle2 className="mr-2 w-5 h-5" /> {readyLabel}</>
                )}
            </Button>
        </>
    )
}
