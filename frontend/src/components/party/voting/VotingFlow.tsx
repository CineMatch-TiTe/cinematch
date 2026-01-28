'use client'

import { useVoting } from '@/hooks/useVoting'
import VotingCard from './VotingCard'

interface VotingFlowProps {
  partyId: string
}

export default function VotingFlow({ partyId }: Readonly<VotingFlowProps>) {
  const { movies, votingRound, loading, countdown, showContent, transitionData, handleVote } =
    useVoting(partyId)

  if (loading || !showContent) {
    return (
      <div className="fixed inset-0 z-50 flex flex-col items-center justify-center bg-zinc-950 text-white">
        <div className="text-9xl font-black mb-8 animate-pulse text-red-600">
          {countdown > 0 ? countdown : 'GO!'}
        </div>
        <div className="text-2xl font-light text-zinc-400">Getting movies ready...</div>
      </div>
    )
  }

  if (transitionData) {
    return (
      <div className="fixed inset-0 z-50 flex flex-col items-center justify-center bg-zinc-950 text-white animate-in fade-in duration-500">
        <div className="text-6xl font-black mb-4 text-transparent bg-clip-text bg-linear-to-r from-red-500 to-orange-500 animate-pulse">
          Round {transitionData.round}
        </div>
        <div className="text-xl text-zinc-400">Preparing next set of movies...</div>
      </div>
    )
  }

  if (movies.length === 0) {
    return (
      <div className="fixed inset-0 z-50 flex flex-col items-center justify-center bg-zinc-950 text-white p-4 text-center">
        <h2 className="text-2xl font-bold mb-2">No movies recommended?</h2>
        <p className="text-zinc-400">
          It seems we couldn&apos;t find any common ground. Try picking more movies next time!
        </p>
      </div>
    )
  }

  return (
    <div className="min-h-screen bg-zinc-950 text-zinc-100 pb-32 pt-8 px-4 flex flex-col items-center">
      <div className="w-full max-w-2xl space-y-6">
        <div className="flex flex-col items-center justify-center mb-8 gap-2">
          <div className="text-sm font-medium text-red-500 uppercase tracking-widest border border-red-500/30 px-3 py-1 rounded-full bg-red-500/10">
            Round {votingRound}
          </div>
          <h2 className="text-3xl font-bold text-center bg-clip-text bg-linear-to-r text-white">
            Vote for your Top {votingRound === 1 ? '5' : '3'}
          </h2>
        </div>

        {movies.map((movie) => {
          return <VotingCard key={movie.movie_id} movie={movie} onVote={handleVote} />
        })}
      </div>
    </div>
  )
}
