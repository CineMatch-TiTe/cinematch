'use client'

import { Timer } from 'lucide-react'
import { usePhaseCountdown } from '@/hooks/usePhaseCountdown'

interface PhaseCountdownProps {
    phaseEnteredAt: string
    timeoutSecs: number
}

/** Floating countdown pill — shows mm:ss remaining in the current phase. */
export default function PhaseCountdown({ phaseEnteredAt, timeoutSecs }: Readonly<PhaseCountdownProps>) {
    const secondsLeft = usePhaseCountdown(phaseEnteredAt, timeoutSecs)

    const minutes = Math.floor(secondsLeft / 60)
    const secs = secondsLeft % 60
    const display = `${String(minutes).padStart(2, '0')}:${String(secs).padStart(2, '0')}`

    const isUrgent = secondsLeft <= 10

    return (
        <div
            className={`inline-flex items-center gap-1.5 px-3 py-1 rounded-full text-sm font-mono font-semibold border transition-colors ${isUrgent
                    ? 'border-red-500/50 bg-red-500/10 text-red-400 animate-pulse'
                    : 'border-zinc-700 bg-zinc-800/80 text-zinc-300'
                }`}
        >
            <Timer className="w-3.5 h-3.5" />
            {display}
        </div>
    )
}
