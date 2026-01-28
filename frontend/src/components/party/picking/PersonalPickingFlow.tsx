'use client'

import { useCallback } from 'react'
import MovieCard from '../MovieCard'
import { PickingLoadingState } from '.'
import PersonalPickingEmptyState from './PersonalPickingEmptyState'
import { useMoviePicker } from '@/hooks/useMoviePicker'
import { getPersonalRecommendationsAction, updatePersonalTasteAction } from '@/actions/dashboard'

export default function PersonalPickingFlow() {
  const fetchNext = useCallback(async () => {
    try {
      const result = await getPersonalRecommendationsAction()
      return result.data ?? []
    } catch (error) {
      console.error('Failed to fetch personal recommendations', error)
      return []
    }
  }, [])

  const submitAction = useCallback(async (movieId: number, action: boolean | null) => {
    return await updatePersonalTasteAction(movieId, action)
  }, [])

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
    return <PersonalPickingEmptyState />
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
