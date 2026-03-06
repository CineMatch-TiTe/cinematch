'use client'

import { Button } from '@/components/ui/button'
import { CheckCircle2, XCircle } from 'lucide-react'
import { useVoting } from '@/hooks/useVoting'
import VotingCard from './VotingCard'
import PhaseCountdown from '../PhaseCountdown'
import { useEffect, useState } from 'react'
import { usePartyView } from '@/components/party/PartyViewContext'

interface VotingFlowProps {
  partyId: string
  phaseEnteredAt: string
  timeoutSecs: number
  deadlineAt?: string | null
}

export default function VotingFlow({ partyId, phaseEnteredAt, timeoutSecs, deadlineAt }: Readonly<VotingFlowProps>) {
  const { movies, votingRound, voteTotals, loading, countdown, showContent, transitionData, handleVote, handleReady } =
    useVoting(partyId, phaseEnteredAt)
  const { members, currentUser } = usePartyView()
  const serverReady = members.find((m) => m.user_id === currentUser.user_id)?.is_ready ?? false
  const [isVotingReady, setIsVotingReady] = useState(serverReady)

  useEffect(() => { setIsVotingReady(serverReady) }, [serverReady])

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
        {transitionData.restart ? (
          <>
            <div className="text-5xl font-black mb-4 text-transparent bg-clip-text bg-linear-to-r from-amber-400 to-orange-500 animate-pulse">
              No Decisive Winner!
            </div>
            <div className="text-xl text-zinc-400">Restarting voting with new movies...</div>
          </>
        ) : (
          <>
            <div className="text-6xl font-black mb-4 text-transparent bg-clip-text bg-linear-to-r from-red-500 to-orange-500 animate-pulse">
              Round {transitionData.round}
            </div>
            <div className="text-xl text-zinc-400">Preparing next set of movies...</div>
          </>
        )}
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
          <PhaseCountdown
            phaseEnteredAt={phaseEnteredAt}
            timeoutSecs={deadlineAt ? timeoutSecs : 0}
            deadlineAt={deadlineAt}
          />
          <div className="text-sm font-medium text-red-500 uppercase tracking-widest border border-red-500/30 px-3 py-1 rounded-full bg-red-500/10">
            Round {votingRound}
          </div>
          <h2 className="text-3xl font-bold text-center bg-clip-text bg-linear-to-r text-white">
            Vote for multiple movies!
          </h2>
          <div className="text-zinc-400 text-sm font-medium">
            (Pick your Top {votingRound === 1 ? '5' : '3'})
          </div>
          {votingRound === 1 && (
            <Button
              onClick={() => {
                const next = !isVotingReady
                setIsVotingReady(next)
                handleReady(next)
              }}
              className={`mt-4 font-semibold rounded-full px-6 py-2 transition-all flex items-center gap-2 ${isVotingReady
                ? 'bg-red-600 hover:bg-red-700 text-white shadow-[0_0_15px_rgba(239,68,68,0.3)] hover:shadow-[0_0_25px_rgba(239,68,68,0.5)]'
                : 'bg-emerald-600 hover:bg-emerald-500 text-white shadow-[0_0_15px_rgba(16,185,129,0.3)] hover:shadow-[0_0_25px_rgba(16,185,129,0.5)]'
                }`}
            >
              {isVotingReady ? (
                <><XCircle className="w-5 h-5" /> Undo Ready</>
              ) : (
                <><CheckCircle2 className="w-5 h-5" /> I&apos;m Finished Voting</>
              )}
            </Button>
          )}
        </div>

        {movies.map((movie) => {
          return <VotingCard key={movie.movie_id} movie={movie} onVote={handleVote} votes={voteTotals[movie.movie_id]} />
        })}
      </div>
    </div>
  )
}
