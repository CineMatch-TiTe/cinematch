'use client'

import { useCallback } from 'react'
import MovieCard from '../MovieCard'
import { PickingLoadingState, PickingEmptyState } from '.'
import { useMoviePicker } from '@/hooks/useMoviePicker'
import { getRecommendedMoviesAction, pickMovieAction } from '@/actions/party-room'

export default function PickingFlow({ partyId }: Readonly<{ partyId: string }>) {
  const fetchNext = useCallback(async () => {
    try {
      const result = await getRecommendedMoviesAction(partyId)
      return result.data ?? []
    } catch (error) {
      console.error('Failed to fetch recommendations', error)
      return []
    }
  }, [partyId])

  const submitAction = useCallback(
    async (movieId: number, action: boolean | null) => {
      return await pickMovieAction(partyId, movieId, action)
    },
    [partyId]
  )

  const {
    currentMovie,
    loading,
    refetching,
    processing,
    handleLike,
    handleDislike,
    handleSkip,
    hasFinishedAllMovies
  } = useMoviePicker({ fetchNext, submitAction })

  if (loading || refetching) {
    return <PickingLoadingState isRefetching={refetching} />
  }

  if (hasFinishedAllMovies) {
    return <PickingEmptyState />
  }

  if (!currentMovie) {
    return null
  }

  return (
    <div className="relative z-50 pt-20 pb-24 h-screen flex flex-col justify-center">
      <MovieCard
        movie={currentMovie}
        onLike={handleLike}
        onDislike={handleDislike}
        onSkip={handleSkip}
        disabled={processing}
      />
    </div>
  )
}
