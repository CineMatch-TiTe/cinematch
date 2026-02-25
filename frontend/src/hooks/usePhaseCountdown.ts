'use client'

import { useState, useEffect } from 'react'

/**
 * Countdown hook driven by the REST PartyResponse.
 *
 * @param phaseEnteredAt  ISO-8601 timestamp from `party.phase_entered_at`
 * @param timeoutSecs     Active timeout duration (e.g. `voting_timeout_secs` or `watching_timeout_secs`)
 * @param deadlineAt      Optional absolute deadline. If provided, this is used instead of calculating from phaseEnteredAt.
 * @returns `secondsLeft` — integer ≥ 0 that ticks every second. 0 means expired.
 */
export function usePhaseCountdown(
    phaseEnteredAt: string,
    timeoutSecs: number,
    deadlineAt?: string | null
): number {
    const computeRemaining = () => {
        if (deadlineAt) {
            const end = new Date(deadlineAt).getTime()
            return Math.max(0, Math.ceil((end - Date.now()) / 1000))
        }
        if (timeoutSecs === 0) return 0
        const deadlineMs = new Date(phaseEnteredAt).getTime() + timeoutSecs * 1000
        return Math.max(0, Math.ceil((deadlineMs - Date.now()) / 1000))
    }

    const [secondsLeft, setSecondsLeft] = useState(computeRemaining)

    useEffect(() => {
        setSecondsLeft(computeRemaining())

        const id = setInterval(() => {
            const remaining = computeRemaining()
            setSecondsLeft(remaining)
            if (remaining <= 0) clearInterval(id)
        }, 1000)

        return () => clearInterval(id)
        // eslint-disable-next-line react-hooks/exhaustive-deps
    }, [phaseEnteredAt, timeoutSecs, deadlineAt])

    return secondsLeft
}
