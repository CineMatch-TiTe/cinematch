'use client'

import MovieCard from '../MovieCard'
import { PickingLoadingState } from '.'
import PersonalPickingEmptyState from './PersonalPickingEmptyState'
import { usePersonalMoviePicker } from '@/hooks/usePersonalMoviePicker'

export default function PersonalPickingFlow() {
  const {
    currentMovie,
    loading,
    refetching,
    processing,
    handleLike,
    handleDislike,
    handleSkip,
    hasFinishedAllMovies
  } = usePersonalMoviePicker()

  if (loading || refetching) {
    return <PickingLoadingState isRefetching={refetching} />
  }

  if (hasFinishedAllMovies) {
    // Reuse existing empty state for now
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
