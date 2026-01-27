import { useState, useTransition, useEffect, useRef, useCallback } from 'react'
import { toast } from 'sonner'
import { getMoviesByIdsAction, getPartyVotesAction, voteMovieAction } from '@/actions/party-room'
import { MovieResponse } from '@/model'

export function useVoting(partyId: string) {
  const [movies, setMovies] = useState<MovieResponse[]>([])
  const [votingRound, setVotingRound] = useState<number | null>(null)
  const [loading, setLoading] = useState(true)
  const [countdown, setCountdown] = useState(5)
  const [showContent, setShowContent] = useState(false)
  const [transitionData, setTransitionData] = useState<{ round: number } | null>(null)
  const [, startTransition] = useTransition()

  // Track displayed movie IDs to detect changes without dependencies
  const displayedMovieIds = useRef<number[]>([])

  // Countdown effect
  useEffect(() => {
    if (countdown > 0) {
      const timer = setTimeout(() => setCountdown((prev) => prev - 1), 1000)
      return () => clearTimeout(timer)
    }

    if (countdown === 0) {
      const t = setTimeout(() => setShowContent(true), 0)
      return () => clearTimeout(t)
    }
  }, [countdown])

  // Helper to handle transition
  const handleBallotChange = useCallback(
    async (newMovieIds: number[], nextRound: number) => {
      if (transitionData) return
      setTransitionData({ round: nextRound })
      displayedMovieIds.current = newMovieIds

      const moviesPromise =
        newMovieIds.length > 0 ? getMoviesByIdsAction(newMovieIds) : Promise.resolve({ data: [] })

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
    },
    [transitionData]
  )

  // Initial fetch
  useEffect(() => {
    const init = async () => {
      const voteResult = await getPartyVotesAction(partyId)
      if (voteResult.error || !voteResult.data) {
        toast.error('Failed to load voting session')
        setLoading(false)
        return
      }

      const ballotIds = voteResult.data.movie_ids || []
      setVotingRound(voteResult.data.voting_round ?? 1)
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

  // Poll for votes
  const fetchVotes = useCallback(async () => {
    if (transitionData) return

    const result = await getPartyVotesAction(partyId)

    startTransition(() => {
      if (result.data) {
        const newMovieIds = result.data.movie_ids || []
        const currentIds = displayedMovieIds.current

        const hasChanged =
          newMovieIds.length !== currentIds.length ||
          !newMovieIds.every((id) => currentIds.includes(id))

        if (hasChanged) {
          const newRound = result.data.voting_round ?? votingRound ?? 1
          handleBallotChange(newMovieIds, newRound)
        } else if (result.data.voting_round && result.data.voting_round !== votingRound) {
          setVotingRound(result.data.voting_round)
        }
      }
    })
  }, [partyId, transitionData, votingRound, handleBallotChange])

  useEffect(() => {
    const interval = setInterval(fetchVotes, 5000)
    return () => clearInterval(interval)
  }, [fetchVotes])

  const handleVote = async (movieId: number, like: boolean) => {
    const result = await voteMovieAction(partyId, movieId, like)
    if (result.error) {
      toast.error(result.error)
    } else if (result.data) {
      const votesResult = await getPartyVotesAction(partyId)
      if (votesResult.data) {
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

  return {
    movies,
    votingRound,
    loading,
    countdown,
    showContent,
    transitionData,
    handleVote
  }
}
