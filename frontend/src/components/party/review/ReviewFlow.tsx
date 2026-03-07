'use client'

import { useEffect, useState, useRef } from 'react'
import { usePartyView } from '../PartyViewContext'
import { rateMovie, movieGetInfo } from '@/server/movie/movie'
import { MovieResponse } from '@/model/movieResponse'
import { Star } from 'lucide-react'
import { Avatar, AvatarFallback, AvatarImage } from '@/components/ui/avatar'
import JSConfetti from 'js-confetti'
import PhaseCountdown from '../PhaseCountdown'

interface ReviewFlowProps {
    movieId: number
}

export default function ReviewFlow({ movieId }: ReviewFlowProps) {
    const { party, members, currentUser, reviewAverage } = usePartyView()
    const [movie, setMovie] = useState<MovieResponse | null>(null)
    const confettiRef = useRef<JSConfetti | null>(null)
    const hasCelebrated = useRef(false)
    const containerRef = useRef<HTMLDivElement>(null)

    const [hoverRating, setHoverRating] = useState<number | null>(null)
    const [isDragging, setIsDragging] = useState(false)

    // Load movie
    useEffect(() => {
        movieGetInfo({ movie_id: movieId })
            // eslint-disable-next-line @typescript-eslint/no-explicit-any
            .then((res: any) => setMovie(res.data || res))
            .catch(console.error)
    }, [movieId])

    // Confetti setup
    useEffect(() => {
        confettiRef.current = new JSConfetti()
        return () => {
            confettiRef.current = null
        }
    }, [])

    const reviewRatings = party.review_ratings || {}
    const allRated = members.length > 0 && members.every(m => reviewRatings[m.user_id] !== undefined)

    useEffect(() => {
        if (allRated && !hasCelebrated.current) {
            hasCelebrated.current = true
            const average = reviewAverage ?? 0;

            if (average >= 7.5) {
                // High rating: Golden/Star theme with lots of confetti
                confettiRef.current?.addConfetti({
                    confettiColors: ['#fbbf24', '#f59e0b', '#d97706', '#b45309', '#fffbeb', '#fcd34d'],
                    confettiNumber: 250,
                })
                setTimeout(() => {
                    confettiRef.current?.addConfetti({
                        emojis: ['⭐', '🌟', '✨', '🏆', '🍿'],
                        emojiSize: 40,
                        confettiNumber: 50,
                    })
                }, 400)
            } else if (average < 4.0) {
                // Low rating: Rain emojis, maybe garbage
                confettiRef.current?.addConfetti({
                    emojis: ['🌧️', '🗑️', '🍅', '🥱', '👎'],
                    emojiSize: 40,
                    confettiNumber: 50,
                })
            } else {
                // Average rating: Standard colorful confetti
                confettiRef.current?.addConfetti({
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

    return (
        <div className="flex flex-col items-center justify-center p-6 gap-8 w-full max-w-2xl mx-auto pt-20 relative z-10">
            <div className="text-center space-y-2">
                <h2 className="text-3xl font-bold text-zinc-100">Review</h2>
                <div className="flex flex-col items-center gap-2">
                    <p className="text-zinc-400">Rate the movie from 0.5 to 5 stars</p>
                    {party.ready_deadline_at && (
                        <div className="flex flex-col items-center gap-2 animate-in fade-in slide-in-from-top-4 duration-500">
                            <p className="text-amber-400 font-medium text-sm">Everyone has rated! Returning to lobby...</p>
                            <PhaseCountdown
                                phaseEnteredAt={party.phase_entered_at || new Date().toISOString()}
                                timeoutSecs={0}
                                deadlineAt={party.ready_deadline_at}
                            />
                        </div>
                    )}
                </div>
            </div>

            {movie && (
                <div className="flex flex-col items-center gap-4">
                    {movie.poster_url ? (
                        // eslint-disable-next-line @next/next/no-img-element
                        <img src={movie.poster_url} alt={movie.title} className="w-48 h-72 object-cover rounded-xl shadow-2xl shadow-black/50" />
                    ) : (
                        <div className="w-48 h-72 bg-zinc-800 rounded-xl" />
                    )}
                    <h3 className="text-2xl font-bold text-zinc-100 text-center">{movie.title}</h3>
                </div>
            )}

            {/* 5 Star Rating Input */}
            <div
                ref={containerRef}
                className="flex items-center gap-2 bg-white/5 backdrop-blur-xl p-6 rounded-[2rem] border border-white/10 touch-none cursor-pointer group active:bg-white/10 transition-all duration-300 shadow-2xl shadow-black/40"
                onPointerDown={(e) => {
                    setIsDragging(true)
                    e.currentTarget.setPointerCapture(e.pointerId)
                    const calculateRating = (clientX: number) => {
                        if (!containerRef.current) return 1
                        const rect = containerRef.current.getBoundingClientRect()
                        const padding = 24 // p-6 is 24px
                        const x = clientX - (rect.left + padding)
                        const width = rect.width - (padding * 2)
                        const percentage = Math.max(0, Math.min(1, x / width))
                        return Math.max(1, Math.ceil(percentage * 10))
                    }
                    const newRating = calculateRating(e.clientX)
                    setHoverRating(newRating)
                    if ('vibrate' in navigator) navigator.vibrate(5)
                }}
                onPointerMove={(e) => {
                    if (!containerRef.current) return
                    const calculateRating = (clientX: number) => {
                        const rect = containerRef.current!.getBoundingClientRect()
                        const padding = 24
                        const x = clientX - (rect.left + padding)
                        const width = rect.width - (padding * 2)
                        const percentage = Math.max(0, Math.min(1, x / width))
                        return Math.max(1, Math.ceil(percentage * 10))
                    }
                    const newRating = calculateRating(e.clientX)
                    if (isDragging || hoverRating !== null) {
                        if (newRating !== hoverRating) {
                            setHoverRating(newRating)
                            if (isDragging && 'vibrate' in navigator) navigator.vibrate(5)
                        }
                    }
                }}
                onPointerUp={(e) => {
                    setIsDragging(false)
                    e.currentTarget.releasePointerCapture(e.pointerId)
                    if (!containerRef.current) return
                    const calculateRating = (clientX: number) => {
                        const rect = containerRef.current!.getBoundingClientRect()
                        const padding = 24
                        const x = clientX - (rect.left + padding)
                        const width = rect.width - (padding * 2)
                        const percentage = Math.max(0, Math.min(1, x / width))
                        return Math.max(1, Math.ceil(percentage * 10))
                    }
                    const newRating = calculateRating(e.clientX)
                    handleRate(newRating)
                    setHoverRating(null)
                    if ('vibrate' in navigator) navigator.vibrate(10)
                }}
                onPointerLeave={() => {
                    if (!isDragging) setHoverRating(null)
                }}
                onPointerEnter={(e) => {
                    if (!isDragging && containerRef.current) {
                        const rect = containerRef.current.getBoundingClientRect()
                        const padding = 24
                        const x = e.clientX - (rect.left + padding)
                        const width = rect.width - (padding * 2)
                        const percentage = Math.max(0, Math.min(1, x / width))
                        setHoverRating(Math.max(1, Math.ceil(percentage * 10)))
                    }
                }}
            >
                {[0, 1, 2, 3, 4].map((index) => {
                    const value = (index + 1) * 2
                    const displayRating = hoverRating !== null ? hoverRating : (myRating || 0)

                    const isFull = displayRating >= value
                    const isHalf = displayRating === value - 1

                    // Progressive scaling for the active star
                    const isActive = hoverRating !== null && (displayRating === value || displayRating === value - 1)
                    const scale = isDragging
                        ? (isActive ? 'scale(1.25)' : 'scale(1.1)')
                        : (isActive ? 'scale(1.2)' : 'scale(1)')

                    return (
                        <div
                            key={index}
                            className="relative w-12 h-12 flex-shrink-0 pointer-events-none transition-all duration-200 cubic-bezier(0.34, 1.56, 0.64, 1)"
                            style={{ transform: scale }}
                        >
                            <Star className="absolute inset-0 w-12 h-12 text-zinc-800 fill-zinc-900/50" />
                            <div
                                className="absolute inset-0 h-12 overflow-hidden transition-all duration-300"
                                style={{ width: isFull ? '100%' : isHalf ? '50%' : '0%' }}
                            >
                                <Star className="w-12 h-12 text-amber-400 fill-amber-400 max-w-none drop-shadow-[0_0_15px_rgba(251,191,36,0.5)]" />
                            </div>
                        </div>
                    )
                })}
            </div>

            {/* Display active selection text */}
            <div className="h-6">
                {(hoverRating !== null || myRating) && (
                    <p className="text-amber-400/80 font-medium animate-in fade-in zoom-in duration-200">
                        {hoverRating !== null ? (hoverRating / 2).toFixed(1) : (myRating && (myRating / 2).toFixed(1))} Stars
                    </p>
                )}
            </div>

            {/* Party Members Ratings */}
            <div className="w-full bg-zinc-900/50 border border-zinc-800 rounded-2xl p-6 space-y-4">
                <div className="flex items-center justify-between pb-4 border-b border-zinc-800">
                    <h4 className="font-semibold text-zinc-200">Party Ratings</h4>
                    {reviewAverage !== null && reviewAverage > 0 && (
                        <div className="flex items-center gap-1 text-amber-400 font-bold bg-amber-400/10 px-3 py-1 rounded-full">
                            <Star className="w-4 h-4 fill-amber-400" />
                            <span>{(reviewAverage / 2).toFixed(1)} Avg</span>
                        </div>
                    )}
                </div>

                <div className="space-y-3">
                    {members.map(member => {
                        const rating = reviewRatings[member.user_id]
                        return (
                            <div key={member.user_id} className="flex items-center justify-between">
                                <div className="flex items-center gap-3">
                                    <Avatar className="w-10 h-10 border-2 border-zinc-800 shadow-sm">
                                        <AvatarImage src={`https://api.dicebear.com/7.x/avataaars/svg?seed=${member.username}`} />
                                        <AvatarFallback className="bg-zinc-800 text-zinc-400 text-xs">
                                            {member.username.substring(0, 2).toUpperCase()}
                                        </AvatarFallback>
                                    </Avatar>
                                    <span className="text-zinc-300 font-medium">
                                        {member.user_id === currentUser.user_id ? 'You' : member.username}
                                    </span>
                                </div>
                                <div>
                                    {rating ? (
                                        <div className="flex items-center gap-1 text-amber-400 font-semibold bg-amber-400/10 px-2.5 py-0.5 rounded-full text-sm">
                                            <Star className="w-3.5 h-3.5 fill-amber-400" />
                                            {rating / 2}
                                        </div>
                                    ) : (
                                        <span className="text-zinc-500 text-sm italic">Waiting...</span>
                                    )}
                                </div>
                            </div>
                        )
                    })}
                </div>
            </div>
        </div>
    )
}
