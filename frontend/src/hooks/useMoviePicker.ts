'use client'

import { useCallback, useEffect, useRef, useState } from 'react'
import { toast } from 'sonner'
import { MovieResponse } from '@/model/movieResponse'
import { useMoviePickerContext } from '@/components/providers/MoviePickerProvider'
import { prefetchImages } from '@/lib/utils'

export interface UseMoviePickerOptions {
  key: string
  fetchNext: () => Promise<MovieResponse[]>
  submitAction: (movieId: number, action: boolean | null) => Promise<{ error?: string }>
}

export interface UseMoviePickerReturn {
  currentMovie: MovieResponse | undefined
  loading: boolean
  processing: boolean
  noNewMovies: boolean
  handleLike: () => Promise<void>
  handleDislike: () => Promise<void>
  handleSkip: () => Promise<void>
  hasFinishedAllMovies: boolean
}

export function useMoviePicker({
  key,
  fetchNext,
  submitAction
}: UseMoviePickerOptions): UseMoviePickerReturn {
  const { getState, setState } = useMoviePickerContext()

  const savedState = getState(key)
  const initialized = useRef(false)

  // Initialize state from context if available, otherwise defaults
  const [movies, setMovies] = useState<MovieResponse[]>(savedState?.movies || [])
  const [noNewMovies, setNoNewMovies] = useState(savedState?.noNewMovies || false)

  // If we restored state, we are not loading. If we need to fetch, we are loading.
  const [loading, setLoading] = useState(!savedState)
  const [processing, setProcessing] = useState(false)

  // Sync back to context on every change
  useEffect(() => {
    setState(key, {
      movies,
      noNewMovies
    })
  }, [key, movies, noNewMovies, setState])

  const fetchMore = useCallback(async () => {
    setLoading(true)
    try {
      const newMovies = await fetchNext()

      if (newMovies.length > 0) {
        setMovies((prev) => {
          // Deduplicate against the current queue in-memory
          const currentIds = new Set(prev.map(m => m.movie_id))
          const trulyNew = newMovies.filter(m => !currentIds.has(m.movie_id))

          if (trulyNew.length > 0) {
            // Prefetch posters for all new movies
            prefetchImages(trulyNew.map(m => m.poster_url))
            return [...prev, ...trulyNew]
          }
          return prev
        })
        setNoNewMovies(false)
      } else {
        setNoNewMovies(true)
      }
    } catch (error) {
      console.error('Failed to load movies', error)
      toast.error('Failed to load movies')
    } finally {
      setLoading(false)
    }
  }, [fetchNext])

  // Initial load
  useEffect(() => {
    if (!initialized.current) {
      initialized.current = true
      // If we have no state, fetch.
      // If we have state but somehow empty and not marked as noNewMovies, maybe fetch?
      // Respect savedState first.
      if (!savedState) {
        fetchMore()
      } else {
        // We have saved state.
        // If we were effectively "done" but not marked as noNewMovies?
        // E.g. user refreshed while loading?
        // Let's assume savedState captures the last consistent state.
        // If savedState says not loading, we remain not loading.
      }
    }
  }, [savedState, fetchMore])

  useEffect(() => {
    // Early fetch: trigger loading more movies when we are on the last one (index length - 1)
    // so that hopefully they are ready by the time the user swipes.
    if (initialized.current && !loading && !noNewMovies && movies.length <= 1) {
      fetchMore()
    }
  }, [movies.length, loading, noNewMovies, fetchMore])

  const handleAction = async (action: boolean | null) => {
    if (processing) return
    const currentMovie = movies[0]
    if (!currentMovie) return

    setProcessing(true)
    try {
      const result = await submitAction(currentMovie.movie_id, action)

      if (result.error) {
        toast.error(result.error)
      } else {
        // Remove the movie from the queue (priority queue / FIFO)
        setMovies((prev) => prev.slice(1))
      }
    } catch (error) {
      console.error('Action failed', error)
      toast.error('Something went wrong')
    } finally {
      setProcessing(false)
    }
  }

  const hasFinishedAllMovies = noNewMovies && movies.length === 0

  return {
    currentMovie: movies[0],
    loading: loading && movies.length === 0,
    processing,
    noNewMovies,
    handleLike: () => handleAction(true),
    handleDislike: () => handleAction(false),
    handleSkip: () => handleAction(null),
    hasFinishedAllMovies
  }
}
