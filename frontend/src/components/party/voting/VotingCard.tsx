import { useState } from 'react'
import { MovieResponse } from '@/model/movieResponse'
import { Button } from '@/components/ui/button'
import { Card } from '@/components/ui/card'
import Image from 'next/image'
import { ThumbsUp } from 'lucide-react'

function VotingCard({
  movie,
  onVote
}: Readonly<{
  movie: MovieResponse
  onVote: (id: number, like: boolean) => void
}>) {
  const [hasVoted, setHasVoted] = useState(false)

  return (
    <Card
      className={`bg-zinc-900/50 border-zinc-800 overflow-hidden transition-colors ${
        hasVoted ? 'border-red-600' : 'hover:border-zinc-700'
      }`}
    >
      <div className="flex flex-row h-40 sm:h-48 px-4">
        <div className="relative w-28 sm:w-36 shrink-0 bg-zinc-800">
          {movie.poster_url ? (
            <Image
              src={`https://image.tmdb.org/t/p/w200${movie.poster_url}`}
              alt={movie.title}
              fill
              className="object-cover"
            />
          ) : (
            <div className="w-full h-full flex items-center justify-center text-zinc-600">
              No Poster
            </div>
          )}
        </div>
        <div className="flex-1 p-4 flex flex-col justify-between">
          <div>
            <h3 className="text-l text-white font-semibold line-clamp-1">{movie.title}</h3>
            <div className="text-sm text-zinc-400 mt-1 flex items-center gap-2">
              <span>{movie.release_date?.split('-')[0]}</span>
            </div>
            <p className="text-sm text-zinc-500 mt-2 line-clamp-2">{movie.overview}</p>
          </div>

          <div className="flex items-center justify-between mt-3">
            <div className="flex gap-2">
              <Button
                size="sm"
                className="bg-red-600 hover:bg-red-700 text-white"
                onClick={() => {
                  onVote(movie.movie_id, true)
                  setHasVoted(true)
                }}
              >
                <ThumbsUp className="w-4 h-4 mr-2" /> Like
              </Button>
            </div>
          </div>
        </div>
      </div>
    </Card>
  )
}

export default VotingCard
