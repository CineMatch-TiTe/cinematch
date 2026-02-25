'use client'

import { useEffect, useState } from 'react'
import Image from 'next/image'
import { Popcorn } from 'lucide-react'
import { toast } from 'sonner'
import JSConfetti from 'js-confetti'

import { MovieResponse } from '@/model/movieResponse'
import { getMoviesByIdsAction } from '@/actions/party-room'
import { Card } from '@/components/ui/card'
import { Skeleton } from '@/components/ui/skeleton'
import PhaseCountdown from '../PhaseCountdown'
import { prefetchImages } from '@/lib/utils'

interface WatchingFlowProps {
  movieId: number
  phaseEnteredAt: string
  timeoutSecs: number
  deadlineAt?: string | null
}

export default function WatchingFlow({ movieId, phaseEnteredAt, timeoutSecs, deadlineAt }: Readonly<WatchingFlowProps>) {
  const [movie, setMovie] = useState<MovieResponse | null>(null)
  const [loading, setLoading] = useState(true)
  const [isImageLoading, setIsImageLoading] = useState(true)

  useEffect(() => {
    const fetchMovie = async () => {
      const result = await getMoviesByIdsAction([movieId])
      if (result.error || !result.data || result.data.length === 0) {
        toast.error('Failed to load movie details')
      } else {
        setMovie(result.data[0])
        prefetchImages([result.data[0].poster_url])
        const jsConfetti = new JSConfetti()
        jsConfetti.addConfetti({
          emojis: ['🍿', '🎬', '✨']
        })
      }
      setLoading(false)
    }

    fetchMovie()
  }, [movieId])

  if (loading) {
    return (
      <div className="fixed inset-0 z-50 flex flex-col items-center justify-center bg-zinc-950 text-white">
        <div className="animate-pulse flex flex-col items-center gap-4">
          <Popcorn className="w-16 h-16 text-yellow-500" />
          <div className="text-xl font-light text-zinc-400">Preparing the theater...</div>
        </div>
      </div>
    )
  }

  if (!movie) {
    return (
      <div className="fixed inset-0 z-50 flex flex-col items-center justify-center bg-zinc-950 text-white">
        <p>Movie not found.</p>
      </div>
    )
  }

  return (
    <div className="min-h-screen bg-zinc-950 text-zinc-100 pt-8 pb-32 px-4 flex flex-col items-center">
      <div className="w-full max-w-2xl space-y-6">
        <div className="flex justify-center mb-4">
          <PhaseCountdown
            phaseEnteredAt={phaseEnteredAt}
            timeoutSecs={timeoutSecs}
            deadlineAt={deadlineAt}
          />
        </div>
        <Card className="bg-zinc-900 border-zinc-800 overflow-hidden shadow-2xl shadow-red-900/20">
          <div className="relative aspect-video w-full bg-zinc-800">
            {movie.poster_url ? (
              <>
                <Image
                  src={movie.poster_url}
                  alt={movie.title}
                  fill
                  unoptimized
                  sizes="100vw"
                  className="object-cover opacity-30 blur-sm"
                />
                <div className="absolute inset-0 flex items-center justify-center p-8">
                  <div className="relative w-48 h-72 shadow-2xl rounded-lg overflow-hidden border-2 border-white/10">
                    <Image
                      src={movie.poster_url}
                      alt={movie.title}
                      fill
                      unoptimized
                      sizes="(max-width: 768px) 192px, 192px"
                      className={`object-cover transition-opacity duration-300 ${isImageLoading ? 'opacity-0' : 'opacity-100'
                        }`}
                      onLoad={() => setIsImageLoading(false)}
                      priority
                    />
                    {isImageLoading && <Skeleton className="absolute inset-0 bg-zinc-800" />}
                  </div>
                </div>
              </>
            ) : (
              <div className="w-full h-full flex items-center justify-center text-zinc-500">
                No Image
              </div>
            )}
          </div>
          <div className="p-8">
            <h1 className="text-3xl font-bold mb-2 text-white">{movie.title}</h1>
            <div className="flex items-center gap-4 text-sm text-zinc-400 mb-6">
              {movie.release_date && <span>{movie.release_date.split('-')[0]}</span>}
              {movie.runtime && (
                <span>
                  {Math.floor(movie.runtime / 60)}h {movie.runtime % 60}m
                </span>
              )}
            </div>
            <p className="text-zinc-300 leading-relaxed text-lg">{movie.overview}</p>

            {movie.director && (
              <div className="mt-6 pt-6 border-t border-white/5">
                <p className="text-sm text-zinc-500 uppercase tracking-widest font-semibold mb-1">
                  Director
                </p>
                <p className="text-zinc-200">{movie.director}</p>
              </div>
            )}
          </div>
        </Card>
      </div>
    </div>
  )
}
