import { Button } from '@/components/ui/button'
import { MovieResponse } from '@/model/movieResponse'
import { ThumbsDown, ThumbsUp } from 'lucide-react'
import Image from 'next/image'

interface MovieCardProps {
  movie: MovieResponse
  onLike: () => void
  onSkip: () => void
  disabled?: boolean
}

export default function MovieCard({ movie, onLike, onSkip, disabled }: MovieCardProps) {
  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/80 backdrop-blur-sm animate-in fade-in duration-200">
      <div className="w-full max-w-md h-[80vh] flex flex-col bg-zinc-900 rounded-3xl overflow-hidden shadow-2xl ring-1 ring-white/10 relative">
        {/* Movie Poster Background */}
        <div className="absolute inset-0 z-0">
          <Image
            src={movie.poster_url || '/placeholder-movie.jpg'}
            alt={movie.title}
            fill
            className="object-cover opacity-60"
            priority
          />
          <div className="absolute inset-0 bg-linear-to-t from-zinc-950 via-zinc-950/60 to-transparent" />
        </div>

        {/* Content */}
        <div className="relative z-10 flex-1 flex flex-col justify-end p-6 pb-24 space-y-4">
          <div>
            <h2 className="text-3xl font-bold text-white shadow-black drop-shadow-md leading-tight">
              {movie.title}
            </h2>
            <div className="flex flex-wrap gap-2 mt-2">
              {movie.release_date && (
                <span className="text-sm font-medium text-zinc-300 bg-black/40 px-2 py-1 rounded backdrop-blur-md">
                  {new Date(movie.release_date).getFullYear()}
                </span>
              )}
              {movie.rating && (
                <span className="text-sm font-medium text-yellow-400 bg-black/40 px-2 py-1 rounded backdrop-blur-md">
                  ★ {movie.rating}
                </span>
              )}
              {movie.runtime && (
                <span className="text-sm font-medium text-zinc-300 bg-black/40 px-2 py-1 rounded backdrop-blur-md">
                  {movie.runtime} min
                </span>
              )}
            </div>
          </div>

          <p className="text-zinc-200 line-clamp-4 text-sm leading-relaxed drop-shadow-md">
            {movie.overview}
          </p>

          <div className="flex flex-wrap gap-2 pt-2">
            {movie.genres.slice(0, 3).map((genre) => (
              <span
                key={genre}
                className="text-xs font-medium text-zinc-400 border border-zinc-700/50 px-2 py-1 rounded-full bg-black/40 backdrop-blur-md"
              >
                {genre}
              </span>
            ))}
          </div>
        </div>

        {/* Action Buttons */}
        <div className="absolute bottom-6 left-0 right-0 px-8 flex justify-between items-center z-20">
          <Button
            size="icon"
            variant="ghost"
            className="w-16 h-16 rounded-full bg-zinc-800/80 hover:bg-red-500/20 hover:text-red-500 text-zinc-400 backdrop-blur-md border border-white/10 transition-all active:scale-95"
            onClick={onSkip}
            disabled={disabled}
          >
            <ThumbsDown className="w-8 h-8" />
          </Button>

          <Button
            size="icon"
            variant="ghost"
            className="w-16 h-16 rounded-full bg-green-500/20 hover:bg-green-500/30 text-green-500 backdrop-blur-md border border-green-500/30 transition-all active:scale-95 shadow-lg shadow-green-500/10"
            onClick={onLike}
            disabled={disabled}
          >
            <ThumbsUp className="w-8 h-8 fill-current" />
          </Button>
        </div>
      </div>
    </div>
  )
}
