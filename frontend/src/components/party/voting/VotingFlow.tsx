'use client'

import { useEffect, useState, useTransition, useRef } from 'react'
import { toast } from 'sonner'

import { GetVoteResponseVoteTotals } from '@/model/getVoteResponseVoteTotals'
import { voteMovieAction, getPartyVotesAction, getMoviesByIdsAction } from '@/actions/party-room'
import { MovieResponse } from '@/model'
import VotingCard from './VotingCard'

interface VotingFlowProps {
  partyId: string
}

export default function VotingFlow({ partyId }: Readonly<VotingFlowProps>) {
  const [movies, setMovies] = useState<MovieResponse[]>([])
  const [voteTotals, setVoteTotals] = useState<GetVoteResponseVoteTotals>({})
  const [votingRound, setVotingRound] = useState<number | null>(null)
  const [loading, setLoading] = useState(true)
  const [countdown, setCountdown] = useState(5) // 5 seconds countdown
  const [showContent, setShowContent] = useState(false)
  const [, startTransition] = useTransition()

  // Countdown effect
  useEffect(() => {
    if (countdown > 0) {
      const timer = setTimeout(() => setCountdown((prev) => prev - 1), 1000)
      return () => clearTimeout(timer)
    }

    // When countdown hits 0, show content
    if (countdown === 0) {
      // Use a timeout to break the synchronous chain if strict mode complains, or just set it
      const t = setTimeout(() => setShowContent(true), 0)
      return () => clearTimeout(t)
    }
  }, [countdown])

  const [transitionData, setTransitionData] = useState<{ round: number } | null>(null)

  // Track displayed movie IDs to detect changes without dependencies
  const displayedMovieIds = useRef<number[]>([])

  // Helper to handle transition
  const handleBallotChange = async (newMovieIds: number[], nextRound: number) => {
    // If we're already transitioning or viewed this ballot, ignore (logic check should happen before calling this usually but good safety)
    if (transitionData) return
    setTransitionData({ round: nextRound })
    displayedMovieIds.current = newMovieIds // Update ref immediately to prevent double triggers

    // Fetch next movies during delay
    const moviesPromise =
      newMovieIds.length > 0 ? getMoviesByIdsAction(newMovieIds) : Promise.resolve({ data: [] })

    // Minimum delay of 3 seconds
    const [moviesResult] = await Promise.all([
      moviesPromise,
      new Promise((resolve) => setTimeout(resolve, 3000))
    ])

    if (moviesResult.data) {
      setMovies(moviesResult.data)
    } else {
      setMovies([])
    }
    setVotingRound(nextRound)
    setTransitionData(null)
  }

  // Initial fetch of votes and movies
  useEffect(() => {
    const init = async () => {
      const voteResult = await getPartyVotesAction(partyId)
      if (voteResult.error || !voteResult.data) {
        toast.error('Failed to load voting session')
        setLoading(false)
        return
      }

      const ballotIds = voteResult.data.movie_ids || []
      setVoteTotals(voteResult.data.vote_totals || {})
      setVotingRound(voteResult.data.voting_round ?? 1)

      // Update ref
      displayedMovieIds.current = ballotIds

      if (ballotIds.length > 0) {
        const moviesResult = await getMoviesByIdsAction(ballotIds)
        if (moviesResult.data) {
          setMovies(moviesResult.data)
        }
      }
      setLoading(false)
    }
    init()
  }, [partyId])

  // Poll for votes and round changes
  useEffect(() => {
    const fetchVotes = async () => {
      // Skip polling logic if transitioning
      if (transitionData) return

      startTransition(async () => {
        const result = await getPartyVotesAction(partyId)
        if (result.data) {
          // Update vote totals always
          if (result.data.vote_totals) {
            setVoteTotals(result.data.vote_totals)
          }

          // Check if ballot changed
          const newMovieIds = result.data.movie_ids || []
          const currentIds = displayedMovieIds.current
          const newRound = result.data.voting_round ?? votingRound ?? 1

          const hasChanged =
            newMovieIds.length !== currentIds.length ||
            !newMovieIds.every((id) => currentIds.includes(id))

          if (hasChanged) {
            // Trigger transition
            handleBallotChange(newMovieIds, newRound)
          } else {
            // Even if ballot didn't change, round number might (unlikely without ballot change but safely update it)
            if (result.data.voting_round && result.data.voting_round !== votingRound) {
              setVotingRound(result.data.voting_round)
            }
          }
        }
      })
    }

    const interval = setInterval(fetchVotes, 5000)
    return () => clearInterval(interval)
  }, [handleBallotChange, partyId, transitionData, votingRound])

  const handleVote = async (movieId: number, like: boolean) => {
    // Optimistic update could go here if needed, but for now we rely on polling/server response
    const result = await voteMovieAction(partyId, movieId, like)
    if (result.error) {
      toast.error(result.error)
    } else if (result.data) {
      // Update specific movie totals immediately
      const newTotals = result.data
      setVoteTotals((prev) => ({
        ...prev,
        [movieId.toString()]: newTotals
      }))

      // Also refresh full list just in case needed (e.g. if round ended)
      const votesResult = await getPartyVotesAction(partyId)
      if (votesResult.data) {
        if (votesResult.data.vote_totals) {
          setVoteTotals(votesResult.data.vote_totals)
        }

        // Check for round/ballot change immediately
        const newMovieIds = votesResult.data.movie_ids || []
        const currentIds = displayedMovieIds.current
        const newRound = votesResult.data.voting_round ?? votingRound ?? 1

        const hasChanged =
          newMovieIds.length !== currentIds.length ||
          !newMovieIds.every((id) => currentIds.includes(id))

        if (hasChanged) {
          handleBallotChange(newMovieIds, newRound)
        }
      }
    }
  }

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
