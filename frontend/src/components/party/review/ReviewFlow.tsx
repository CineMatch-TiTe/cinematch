'use client'

import { useRef, useState, useEffect } from 'react'
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

    const [hoverRating, setHoverRating] = useState<number | null>(null)

    // Load movie
    useEffect(() => {
        movieGetInfo({ movie_id: movieId })
            .then((res) => {
                if ('data' in res && 'title' in res.data) {
                    setMovie(res.data)
                }
            })
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

            {/* 5 Star Rating Input - Premium Redesign */}
            <div className="flex flex-col items-center gap-6 w-full">
                <div className="relative group perspective-1000">
                    <div className="flex items-center gap-1.5 p-4 rounded-3xl bg-white/5 backdrop-blur-2xl border border-white/10 shadow-[0_20px_50px_rgba(0,0,0,0.5)] transition-all duration-500 hover:shadow-[0_25px_60px_rgba(0,0,0,0.6)] hover:border-white/20">
                        {[...Array(5)].map((_, starIndex) => {
                            const starValue = starIndex + 1
                            const displayRating = hoverRating !== null ? hoverRating : (myRating || 0)

                            // Determine fill percentage for this star (0, 50, or 100)
                            let fillPercent = 0
                            if (displayRating >= starValue * 2) {
                                fillPercent = 100
                            } else if (displayRating >= starValue * 2 - 1) {
                                fillPercent = 50
                            }

                            return (
                                <div
                                    key={starIndex}
                                    className="relative flex items-center justify-center p-1 group/star transition-transform duration-300 hover:scale-110 active:scale-95"
                                    onPointerMove={(e) => {
                                        const rect = e.currentTarget.getBoundingClientRect()
                                        const x = e.clientX - rect.left
                                        const isHalf = x < rect.width / 2
                                        setHoverRating(isHalf ? starValue * 2 - 1 : starValue * 2)
                                    }}
                                    onPointerLeave={() => setHoverRating(null)}
                                    onClick={() => handleRate(hoverRating || displayRating)}
                                >
                                    {/* Background Star (Gray) */}
                                    <Star className="w-10 h-10 text-white/10 fill-white/5 transition-colors duration-300 group-hover/star:text-white/20" />

                                    {/* Foreground Star (Gold) - Clipped */}
                                    <div
                                        className="absolute inset-0 flex items-center justify-center pointer-events-none transition-all duration-300 ease-out"
                                        style={{
                                            clipPath: `inset(0 ${100 - fillPercent}% 0 0)`,
                                            filter: fillPercent > 0 ? 'drop-shadow(0 0 12px rgba(251,191,36,0.6))' : 'none'
                                        }}
                                    >
                                        <Star className="w-10 h-10 text-amber-400 fill-amber-400" />
                                    </div>

                                    {/* Hover Highlight Glow */}
                                    {hoverRating !== null && (fillPercent > 0) && (
                                        <div className="absolute inset-0 bg-amber-400/20 rounded-full blur-xl opacity-0 group-hover/star:opacity-100 transition-opacity duration-300" />
                                    )}
                                </div>
                            )
                        })}
                    </div>
                </div>

                {/* Dynamic Status Text */}
                <div className="flex flex-col items-center gap-1 min-h-[3rem]">
                    {(hoverRating !== null || myRating) && (
                        <>
                            <p className="text-3xl font-black text-transparent bg-clip-text bg-gradient-to-r from-amber-300 via-amber-400 to-amber-500 animate-in fade-in zoom-in slide-in-from-bottom-2 duration-300 tracking-tight">
                                {hoverRating !== null ? (hoverRating / 2).toFixed(1) : (myRating && (myRating / 2).toFixed(1))}
                            </p>
                            <p className="text-sm font-bold uppercase tracking-[0.2em] text-amber-500/60 animate-in fade-in slide-in-from-top-1 duration-500 delay-100">
                                {(() => {
                                    const r = hoverRating !== null ? hoverRating : (myRating || 0)
                                    if (r === 0) return ""
                                    if (r <= 2) return "Garbage"
                                    if (r <= 3) return "Terrible"
                                    if (r <= 4) return "Poor"
                                    if (r <= 5) return "Mediocre"
                                    if (r <= 6) return "Average"
                                    if (r <= 7) return "Good"
                                    if (r <= 8) return "Great"
                                    if (r <= 9) return "Amazing"
                                    return "Masterpiece"
                                })()}
                            </p>
                        </>
                    )}
                </div>
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
