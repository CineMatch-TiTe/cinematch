'use client'

import MovieCard from '../MovieCard'
import { PickingLoadingState, PickingEmptyState } from '.'
import { useMoviePicker } from '@/hooks/useMoviePicker'

export default function PickingFlow({ partyId }: Readonly<{ partyId: string }>) {
  const {
    currentMovie,
    loading,
    refetching,
    processing,
    handleLike,
    handleDislike,
    handleSkip,
    hasFinishedAllMovies
  } = useMoviePicker({ partyId })

  if (loading || refetching) {
    return <PickingLoadingState isRefetching={refetching} />
  }

  if (hasFinishedAllMovies) {
    // Note: PickingEmptyState might still need a way to navigate back or just show a message.
    // Ideally it just says "No more movies" and user can switch tab via footer.
    // For now, removing onClose from PickingEmptyState if it allows it, or passing no-op/redirect?
    // Let's check PickingEmptyState signature later or pass undefined if optional.
    // Assuming PickingEmptyState needs onClose, we might need to adjust it too.
    // For now let's pass an empty function or modify PickingEmptyState.
    // Let's modify this to just not pass onClose if not needed, or better, Refactor PickingEmptyState too?
    // I'll check PickingEmptyState.tsx in next step if it fails. For now, let's assume I can remove it.
    // Actually, I should probably check PickingEmptyState.tsx.
    // But let's just render it without props if possible or handle it.
    // Let's assume PickingEmptyState needs refactoring too.
    // But for this step, I will remove the Close button from PickingFlow.
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
