'use client'

import { useCallback, useEffect, useRef, useState } from 'react'
import { toast } from 'sonner'
import { MovieResponse } from '@/model/movieResponse'
import { useMoviePickerContext } from '@/context/MoviePickerContext'

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
  const [seenMovieIds, setSeenMovieIds] = useState<Set<number>>(savedState?.seenMovieIds || new Set())
  const [currentIndex, setCurrentIndex] = useState(savedState?.currentIndex || 0)
  const [noNewMovies, setNoNewMovies] = useState(savedState?.noNewMovies || false)

  // If we restored state, we are not loading. If we need to fetch, we are loading.
  const [loading, setLoading] = useState(!savedState)
  const [processing, setProcessing] = useState(false)

  // Sync back to context on every change
  useEffect(() => {
    setState(key, {
      movies,
      seenMovieIds,
      currentIndex,
      noNewMovies
    })
  }, [key, movies, seenMovieIds, currentIndex, noNewMovies, setState])

  const fetchMore = useCallback(async () => {
    setLoading(true)
    try {
      const newMovies = await fetchNext()

      // Filter out movies we've already seen
      // We use the functional update of setMovies to ensure access to latest 'movies' if needed,
      // but here we primarily need 'seenMovieIds'.
      // However, 'seenMovieIds' in the closure might be stale if we don't include it in deps.
      // But we are in useCallback with dependency... wait.
      // seenMovieIds could change if user swipes while fetching?
      // "loading" blocks interaction so user can't swipe. Safe.

      const filtered = newMovies.filter((m) => !seenMovieIds.has(m.movie_id))

      if (filtered.length > 0) {
        setMovies((prev) => [...prev, ...filtered])
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
  }, [fetchNext, seenMovieIds])

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
    if (initialized.current && !loading && !noNewMovies && currentIndex >= movies.length) {
      fetchMore()
    }
  }, [currentIndex, movies.length, loading, noNewMovies, fetchMore])

  const handleAction = async (action: boolean | null) => {
    if (processing) return
    const currentMovie = movies[currentIndex]
    if (!currentMovie) return

    setProcessing(true)
    try {
      const result = await submitAction(currentMovie.movie_id, action)

      if (result.error) {
        toast.error(result.error)
      } else {
        // Mark as seen
        setSeenMovieIds((prev) => {
          const next = new Set(prev)
          next.add(currentMovie.movie_id)
          return next
        })
        setCurrentIndex((prev) => prev + 1)
      }
    } catch (error) {
      console.error('Action failed', error)
      toast.error('Something went wrong')
    } finally {
      setProcessing(false)
    }
  }

  const hasFinishedAllMovies = noNewMovies && currentIndex >= movies.length

  return {
    currentMovie: movies[currentIndex],
    loading,
    processing,
    noNewMovies,
    handleLike: () => handleAction(true),
    handleDislike: () => handleAction(false),
    handleSkip: () => handleAction(null),
    hasFinishedAllMovies
  }
}
