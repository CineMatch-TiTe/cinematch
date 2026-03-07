'use client'

import { useRef, useEffect } from 'react'
import useSWR from 'swr'
import { usePartyView } from '@/components/party/PartyViewContext'
import { rateMovie, movieGetInfo } from '@/server/movie/movie'
import JSConfetti from 'js-confetti'

export function useReviewFlow(movieId: number) {
    const { party, members, currentUser, reviewAverage } = usePartyView()
    const confettiRef = useRef<JSConfetti | null>(null)
    const hasCelebrated = useRef(false)

    const { data: movie } = useSWR(
        `/api/movie/${movieId}`,
        () =>
            movieGetInfo({ movie_id: movieId }).then((res) => {
                if ('data' in res && 'title' in res.data) return res.data
                throw new Error('Movie not found')
            }),
        { revalidateOnFocus: false },
    )

    function getConfetti() {
        confettiRef.current ??= new JSConfetti()
        return confettiRef.current
    }

    const reviewRatings = party.review_ratings || {}
    const allRated = members.length > 0 && members.every((m) => reviewRatings[m.user_id] !== undefined)

    useEffect(() => {
        if (allRated && !hasCelebrated.current) {
            hasCelebrated.current = true
            const average = reviewAverage ?? 0

            if (average >= 7.5) {
                getConfetti().addConfetti({
                    confettiColors: ['#fbbf24', '#f59e0b', '#d97706', '#b45309', '#fffbeb', '#fcd34d'],
                    confettiNumber: 250,
                })
                setTimeout(() => {
                    getConfetti().addConfetti({
                        emojis: ['⭐', '🌟', '✨', '🏆', '🍿'],
                        emojiSize: 40,
                        confettiNumber: 50,
                    })
                }, 400)
            } else if (average < 4) {
                getConfetti().addConfetti({
                    emojis: ['🌧️', '🗑️', '🍅', '🥱', '👎'],
                    emojiSize: 40,
                    confettiNumber: 50,
                })
            } else {
                getConfetti().addConfetti({
                    confettiNumber: 150,
                })
            }
        }
    }, [allRated, reviewAverage])

    const myRating = reviewRatings[currentUser.user_id]

    const handleRate = async (rating: number) => {
        try {
            await rateMovie({ movie_id: movieId, rating })
        } catch (e) {
            console.error(e)
        }
    }

    return { movie, party, members, currentUser, reviewAverage, reviewRatings, myRating, handleRate }
}
