'use client'

import { useState, useEffect, useCallback } from 'react'

/**
 * Single-purpose hook to count down to an absolute ISO deadline.
 * @param deadline ISO-8601 timestamp string
 * @returns seconds left until deadline
 */
export function useDeadlineCountdown(deadline?: string | null): number {
    const compute = useCallback(() => {
        if (!deadline) return 0
        const end = new Date(deadline).getTime()
        return Math.max(0, Math.ceil((end - Date.now()) / 1000))
    }, [deadline])

    const [left, setLeft] = useState(compute)

    useEffect(() => {
        setLeft(compute())
        if (!deadline) return

        const id = setInterval(() => {
            const next = compute()
            setLeft(next)
            if (next <= 0) clearInterval(id)
        }, 1000)

        return () => clearInterval(id)
    }, [deadline, compute])

    return left
}
