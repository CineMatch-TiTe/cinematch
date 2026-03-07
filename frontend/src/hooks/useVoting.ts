import { useState, useTransition, useEffect, useRef, useCallback } from 'react'
import { toast } from 'sonner'
import { getMoviesByIdsAction, getPartyVotesAction, voteMovieAction, setReadyAction } from '@/actions/party-room'
import { MovieResponse, GetVoteResponseVoteTotals } from '@/model'
import { prefetchImages } from '@/lib/utils'
import { usePartyView } from '@/components/party/PartyViewContext'

export function useVoting(partyId: string) {
  const { lastMessage, consumeLivePhaseTransition } = usePartyView()

  // Show countdown only when voting phase was entered via a live WS message
  const [showCountdown] = useState(() => consumeLivePhaseTransition() === 'voting')

  const [movies, setMovies] = useState<MovieResponse[]>([])
  const [votingRound, setVotingRound] = useState<number | null>(null)
  const [voteTotals, setVoteTotals] = useState<GetVoteResponseVoteTotals>({})
  const [loading, setLoading] = useState(true)
  const [countdown, setCountdown] = useState(() => (showCountdown ? 5 : 0))
  const [showContent, setShowContent] = useState(() => !showCountdown)
  const [transitionData, setTransitionData] = useState<{ round: number; restart?: boolean } | null>(null)
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
      // Detection: 
      // 1. Moving from round 2 to 1 (fallback)
      // 2. Staying in round 1 but we already HAD movies (manual or auto-restart on empty)
      const isRestart = (votingRound === 2 && nextRound === 1) ||
        (votingRound === 1 && nextRound === 1 && movies.length > 0)

      setTransitionData({ round: nextRound, restart: isRestart })
      displayedMovieIds.current = newMovieIds

      const moviesPromise =
        newMovieIds.length > 0 ? getMoviesByIdsAction(newMovieIds) : Promise.resolve({ data: [] })

      const [moviesResult] = await Promise.all([
        moviesPromise,
        new Promise((resolve) => setTimeout(resolve, 3000))
      ])

      if (moviesResult.data) {
        setMovies(moviesResult.data)
        prefetchImages(moviesResult.data.map(m => m.poster_url))
      } else {
        setMovies([])
      }
      setVotingRound(nextRound)
      setVoteTotals({}) // Clear local totals on transition
      setTransitionData(null)
    },
    [transitionData, votingRound, movies]
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
          prefetchImages(moviesResult.data.map(m => m.poster_url))
        }
      }

      if (voteResult.data.vote_totals) {
        setVoteTotals(voteResult.data.vote_totals)
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
          !newMovieIds.every((id: number) => currentIds.includes(id))

        if (hasChanged) {
          const newRound = result.data.voting_round ?? votingRound ?? 1
          handleBallotChange(newMovieIds, newRound)
        } else if (result.data.voting_round && result.data.voting_round !== votingRound) {
          setVotingRound(result.data.voting_round)
        }

        if (result.data.vote_totals) {
          setVoteTotals(result.data.vote_totals)
        }
      }
    })
  }, [partyId, transitionData, votingRound, handleBallotChange])

  useEffect(() => {
    if (lastMessage && typeof lastMessage === 'object') {
      if ('VotingRoundStarted' in lastMessage) {
        startTransition(() => {
          setVotingRound(lastMessage.VotingRoundStarted.round)
        })
        fetchVotes()
      } else if ('MovieVoteUpdate' in lastMessage) {
        const { movie_id, likes, dislikes } = lastMessage.MovieVoteUpdate
        startTransition(() => {
          setVoteTotals((prev) => ({ ...prev, [movie_id]: { likes, dislikes } }))
        })
      } else if ('PartyStateChanged' in lastMessage && lastMessage.PartyStateChanged.state === 'voting') {
        const newRound = lastMessage.PartyStateChanged.voting_round
        if (newRound && newRound !== votingRound) {
          startTransition(() => {
            setVotingRound(newRound)
          })
          fetchVotes()
        }
      }
    }
  }, [lastMessage, fetchVotes, votingRound])

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
          !newMovieIds.every((id: number) => currentIds.includes(id))

        if (hasChanged) {
          handleBallotChange(newMovieIds, newRound)
        }
      }
    }
  }

  const handleReady = async (isReady: boolean) => {
    const result = await setReadyAction(partyId, isReady)
    if (result.error) {
      toast.error(result.error)
    }
  }

  return {
    movies,
    votingRound,
    voteTotals,
    loading,
    countdown,
    showContent,
    transitionData,
    handleVote,
    handleReady
  }
}
