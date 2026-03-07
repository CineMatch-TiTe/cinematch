'use client'

import { Star } from 'lucide-react'
import { Avatar, AvatarFallback, AvatarImage } from '@/components/ui/avatar'
import PhaseCountdown from '../PhaseCountdown'
import StarRatingInput from './StarRatingInput'
import { useReviewFlow } from '@/hooks/useReviewFlow'

interface ReviewFlowProps {
    movieId: number
}

export default function ReviewFlow({ movieId }: Readonly<ReviewFlowProps>) {
    const { movie, party, members, currentUser, reviewAverage, reviewRatings, myRating, handleRate } =
        useReviewFlow(movieId)

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

            <StarRatingInput currentRating={myRating} onRate={handleRate} />

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
