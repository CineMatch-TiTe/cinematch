'use client'

import { X } from 'lucide-react'
import { Button } from '@/components/ui/button'
import MovieCard from '../MovieCard'
import { PickingLoadingState, PickingEmptyState } from '.'
import { useMoviePicker } from '@/hooks/useMoviePicker'

interface PickingFlowProps {
  partyId: string
  onClose: () => void
}

export default function PickingFlow({ partyId, onClose }: Readonly<PickingFlowProps>) {
  const {
    currentMovie,
    loading,
    refetching,
    processing,
    handleLike,
    handleSkip,
    hasFinishedAllMovies
  } = useMoviePicker({ partyId })

  if (loading || refetching) {
    return <PickingLoadingState isRefetching={refetching} />
  }

  if (hasFinishedAllMovies) {
    return <PickingEmptyState onClose={onClose} />
  }

  if (!currentMovie) {
    return null
  }

  return (
    <div className="relative z-50">
      <Button
        size="icon"
        variant="secondary"
        className="fixed top-4 right-4 z-60 rounded-full bg-black/50 hover:bg-black/70 text-white backdrop-blur-md border border-white/10"
        onClick={onClose}
      >
        <X className="w-5 h-5" />
      </Button>

      <MovieCard
        movie={currentMovie}
        onLike={handleLike}
        onSkip={handleSkip}
        disabled={processing}
      />
    </div>
  )
}
