import { useState } from 'react'
import { Button } from '@/components/ui/button'
import { Skeleton } from '@/components/ui/skeleton'
import { Badge } from '@/components/ui/badge'
import { Card, CardContent } from '@/components/ui/card'
import { MovieResponse } from '@/model/movieResponse'
import { ThumbsDown, ThumbsUp, SkipForward, Star, Calendar, Clock } from 'lucide-react'
import Image from 'next/image'

interface MovieCardProps {
  movie: MovieResponse
  onLike: () => void
  onDislike: () => void
  onSkip: () => void
  disabled?: boolean
}

export default function MovieCard({
  movie,
  onLike,
  onDislike,
  onSkip,
  disabled
}: Readonly<MovieCardProps>) {
  const [isImageLoading, setIsImageLoading] = useState(true)
  const [prevPosterUrl, setPrevPosterUrl] = useState(movie.poster_url)

  if (movie.poster_url !== prevPosterUrl) {
    setPrevPosterUrl(movie.poster_url)
    setIsImageLoading(true)
  }

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/80 backdrop-blur-sm animate-in fade-in duration-200">
      <Card className="w-full max-w-md h-[80vh] flex flex-col bg-zinc-900 border-zinc-800 overflow-hidden shadow-2xl relative">
        {/* Movie Poster Background */}
        <div className="absolute inset-0 z-0">
          <Image
            src={movie.poster_url || '/placeholder-movie.jpg'}
            alt={movie.title}
            fill
            sizes="(max-width: 768px) 100vw, 500px"
            className={`object-cover transition-opacity duration-500 ${
              isImageLoading ? 'opacity-0' : 'opacity-60'
            }`}
            priority
            onLoad={() => setIsImageLoading(false)}
          />
          {isImageLoading && <Skeleton className="absolute inset-0 bg-zinc-800" />}
          <div className="absolute inset-0 bg-gradient-to-t from-zinc-950 via-zinc-950/60 to-transparent" />
        </div>

        {/* Content */}
        <CardContent className="relative z-10 flex-1 flex flex-col justify-end p-6 pb-24 space-y-4">
          <div>
            <h2 className="text-3xl font-bold text-white shadow-black drop-shadow-md leading-tight">
              {movie.title}
            </h2>
            <div className="flex flex-wrap gap-2 mt-2">
              {movie.release_date && (
                <Badge variant="secondary" className="bg-black/40 text-zinc-300 hover:bg-black/60 backdrop-blur-md border-0">
                  <Calendar className="w-3 h-3 mr-1" />
                  {new Date(movie.release_date).getFullYear()}
                </Badge>
              )}
              {movie.rating && (
                <Badge variant="secondary" className="bg-black/40 text-yellow-400 hover:bg-black/60 backdrop-blur-md border-0">
                  <Star className="w-3 h-3 mr-1 fill-yellow-400" />
                  {movie.rating}
                </Badge>
              )}
              {movie.runtime && (
                <Badge variant="secondary" className="bg-black/40 text-zinc-300 hover:bg-black/60 backdrop-blur-md border-0">
                  <Clock className="w-3 h-3 mr-1" />
                  {movie.runtime} min
                </Badge>
              )}
            </div>
          </div>

          <p className="text-zinc-200 line-clamp-4 text-sm leading-relaxed drop-shadow-md">
            {movie.overview}
          </p>

          <div className="flex flex-wrap gap-2 pt-2 pb-6">
            {movie.genres.slice(0, 3).map((genre) => (
              <Badge
                key={genre}
                variant="outline"
                className="text-xs font-medium text-zinc-400 border-zinc-700/50 bg-black/40 backdrop-blur-md"
              >
                {genre}
              </Badge>
            ))}
          </div>
        </CardContent>

        {/* Action Buttons */}
        <div className="absolute bottom-6 left-0 right-0 px-8 flex justify-between items-center z-20 gap-4">
          <Button
            size="icon"
            variant="ghost"
            className="w-16 h-16 rounded-full text-red-400 bg-red-400/10 hover:bg-red-500/20 hover:text-red-500 backdrop-blur-md border border-white/10 transition-all active:scale-95"
            onClick={onDislike}
            disabled={disabled}
          >
            <ThumbsDown className="w-8 h-8" />
          </Button>

          <Button
            size="icon"
            variant="ghost"
            className="w-16 h-16 rounded-full text-zinc-400 bg-zinc-800/50 hover:bg-zinc-700/50 hover:text-white backdrop-blur-md border border-white/10 transition-all active:scale-95"
            onClick={onSkip}
            disabled={disabled}
          >
            <SkipForward className="w-6 h-6" />
          </Button>

          <Button
            size="icon"
            variant="ghost"
            className="w-16 h-16 rounded-full bg-green-500/20 hover:bg-green-500/30 text-green-500 backdrop-blur-md border border-green-500/30 transition-all active:scale-95 shadow-lg shadow-green-500/10 hover:text-green-400"
            onClick={onLike}
            disabled={disabled}
          >
            <ThumbsUp className="w-8 h-8 fill-current" />
          </Button>
        </div>
      </Card>
    </div>
  )
}
