'use client'

import { useEffect, useState } from 'react'
import Image from 'next/image'
import { Popcorn } from 'lucide-react'
import { toast } from 'sonner'
import JSConfetti from 'js-confetti'

import { MovieResponse } from '@/model/movieResponse'
import { getMoviesByIdsAction } from '@/actions/party-room'
import { Card } from '@/components/ui/card'

interface WatchingFlowProps {
  movieId: number
}

export default function WatchingFlow({ movieId }: Readonly<WatchingFlowProps>) {
  const [movie, setMovie] = useState<MovieResponse | null>(null)
  const [loading, setLoading] = useState(true)

  useEffect(() => {
    const fetchMovie = async () => {
      const result = await getMoviesByIdsAction([movieId])
      if (result.error || !result.data || result.data.length === 0) {
        toast.error('Failed to load movie details')
      } else {
        setMovie(result.data[0])
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
    <div className="min-h-screen bg-zinc-950 text-zinc-100 pt-8 px-4 flex flex-col items-center">
      <div className="w-full max-w-2xl space-y-6">
        <Card className="bg-zinc-900 border-zinc-800 overflow-hidden shadow-2xl shadow-red-900/20">
          <div className="relative aspect-video w-full bg-zinc-800">
            {movie.poster_url ? (
              <>
                <Image
                  src={`https://image.tmdb.org/t/p/original${movie.poster_url}`} // Use backdrop if available, but movieResponse only has poster_url usually. Let's check model.
                  // MovieResponse has poster_url. It doesn't seem to have valid backdrop_path exposed in types clearly seen before, let's Stick to poster but maybe formatted differently or check if we can get backdrop.
                  // Actually, let's just use poster with a blur background or fit it nicely.
                  alt={movie.title}
                  fill
                  className="object-cover opacity-30 blur-sm"
                />
                <div className="absolute inset-0 flex items-center justify-center p-8">
                  <div className="relative w-48 h-72 shadow-2xl rounded-lg overflow-hidden border-2 border-white/10">
                    <Image
                      src={`https://image.tmdb.org/t/p/w500${movie.poster_url}`}
                      alt={movie.title}
                      fill
                      className="object-cover"
                    />
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
