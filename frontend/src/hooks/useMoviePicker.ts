'use client'

import { useCallback, useEffect, useRef, useState } from 'react'
import { toast } from 'sonner'
import { MovieResponse } from '@/model/movieResponse'

const PREFETCH_THRESHOLD = 3

export interface UseMoviePickerOptions {
  fetchNext: () => Promise<MovieResponse[]>
  submitAction: (movieId: number, action: boolean | null) => Promise<{ error?: string }>
}

export interface UseMoviePickerReturn {
  currentMovie: MovieResponse | undefined
  loading: boolean
  refetching: boolean
  processing: boolean
  noNewMovies: boolean
  handleLike: () => Promise<void>
  handleDislike: () => Promise<void>
  handleSkip: () => Promise<void>
  hasFinishedAllMovies: boolean
}

export function useMoviePicker({
  fetchNext,
  submitAction
}: UseMoviePickerOptions): UseMoviePickerReturn {
  const [movies, setMovies] = useState<MovieResponse[]>([])
  const [seenMovieIds, setSeenMovieIds] = useState<Set<number>>(new Set())
  const [loading, setLoading] = useState(true)
  const [processing, setProcessing] = useState(false)
  const [currentIndex, setCurrentIndex] = useState(0)
  const [noNewMovies, setNoNewMovies] = useState(false)
  const [isPrefetching, setIsPrefetching] = useState(false)

  const initialized = useRef(false)

  const filterNewMovies = useCallback((fetchedMovies: MovieResponse[], seenIds: Set<number>) => {
    return fetchedMovies.filter((movie) => !seenIds.has(movie.movie_id))
  }, [])

  const loadMoreMovies = useCallback(async () => {
    if (isPrefetching || noNewMovies) return

    setIsPrefetching(true)
    try {
      const newMovies = await fetchNext()
      setMovies((prevMovies) => {
        return prevMovies
      })

      const filtered = filterNewMovies(newMovies, seenMovieIds)

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
      setIsPrefetching(false)
      setLoading(false)
    }
  }, [fetchNext, filterNewMovies, isPrefetching, noNewMovies, seenMovieIds])

  useEffect(() => {
    if (!initialized.current) {
      initialized.current = true
      loadMoreMovies()
    }
  }, [loadMoreMovies])

  useEffect(() => {
    const remaining = movies.length - currentIndex
    if (!loading && !noNewMovies && remaining <= PREFETCH_THRESHOLD) {
      loadMoreMovies()
    }
  }, [currentIndex, movies.length, loading, noNewMovies, loadMoreMovies])

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

  const hasFinishedAllMovies = movies.length > 0 && currentIndex >= movies.length && noNewMovies

  return {
    currentMovie: movies[currentIndex],
    loading,
    processing, // separate loading and processing
    refetching: isPrefetching, // map isPrefetching to refetching for compatibility
    noNewMovies,
    handleLike: () => handleAction(true),
    handleDislike: () => handleAction(false),
    handleSkip: () => handleAction(null),
    hasFinishedAllMovies
  }
}
