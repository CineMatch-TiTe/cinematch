'use client'

import { useState } from 'react'
import { Star } from 'lucide-react'

interface StarRatingInputProps {
    currentRating: number | undefined
    onRate: (rating: number) => void
}

export default function StarRatingInput({ currentRating, onRate }: Readonly<StarRatingInputProps>) {
    const [hoverRating, setHoverRating] = useState<number | null>(null)

    const displayRatingValue = hoverRating !== null ? hoverRating : (currentRating || 0)

    return (
        <div className="flex flex-col items-center gap-6 w-full">
            <div className="relative group perspective-1000">
                <div className="flex items-center gap-1.5 p-4 rounded-3xl bg-white/5 backdrop-blur-2xl border border-white/10 shadow-[0_20px_50px_rgba(0,0,0,0.5)] transition-all duration-500 hover:shadow-[0_25px_60px_rgba(0,0,0,0.6)] hover:border-white/20 touch-none">
                    {[...Array(5)].map((_, starIndex) => {
                        const starValue = starIndex + 1

                        // Determine fill percentage for this star (0, 50, or 100)
                        let fillPercent = 0
                        if (displayRatingValue >= starValue * 2) {
                            fillPercent = 100
                        } else if (displayRatingValue >= starValue * 2 - 1) {
                            fillPercent = 50
                        }

                        return (
                            <div
                                key={starIndex}
                                className="relative flex items-center justify-center p-1 group/star transition-transform duration-300 hover:scale-110 active:scale-95"
                                onPointerDown={(e) => {
                                    const rect = e.currentTarget.getBoundingClientRect()
                                    const x = e.clientX - rect.left
                                    const isHalf = x < rect.width / 2
                                    const value = isHalf ? starValue * 2 - 1 : starValue * 2
                                    setHoverRating(value)
                                    if (e.pointerType === 'touch') {
                                        onRate(value)
                                    }
                                }}
                                onPointerMove={(e) => {
                                    if (e.pointerType === 'touch') return
                                    const rect = e.currentTarget.getBoundingClientRect()
                                    const x = e.clientX - rect.left
                                    const isHalf = x < rect.width / 2
                                    setHoverRating(isHalf ? starValue * 2 - 1 : starValue * 2)
                                }}
                                onPointerLeave={() => setHoverRating(null)}
                                onClick={() => {
                                    if (hoverRating !== null) {
                                        onRate(hoverRating)
                                    }
                                }}
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
                {(hoverRating !== null || currentRating) && (
                    <>
                        <p className="text-3xl font-black text-transparent bg-clip-text bg-gradient-to-r from-amber-300 via-amber-400 to-amber-500 animate-in fade-in zoom-in slide-in-from-bottom-2 duration-300 tracking-tight">
                            {hoverRating !== null ? (hoverRating / 2).toFixed(1) : (currentRating && (currentRating / 2).toFixed(1))}
                        </p>
                        <p className="text-sm font-bold uppercase tracking-[0.2em] text-amber-500/60 animate-in fade-in slide-in-from-top-1 duration-500 delay-100">
                            {(() => {
                                const r = hoverRating !== null ? hoverRating : (currentRating || 0)
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
    )
}
