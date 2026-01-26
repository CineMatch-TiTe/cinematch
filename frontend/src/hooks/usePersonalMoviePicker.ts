'use client'

import { useCallback, useEffect, useState } from 'react'
import { toast } from 'sonner'
import { MovieResponse } from '@/model/movieResponse'
import { getPersonalRecommendationsAction, updatePersonalTasteAction } from '@/actions/dashboard'

const PREFETCH_THRESHOLD = 3

interface UsePersonalMoviePickerReturn {
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

export function usePersonalMoviePicker(): UsePersonalMoviePickerReturn {
  const [movies, setMovies] = useState<MovieResponse[]>([])
  const [seenMovieIds, setSeenMovieIds] = useState<Set<number>>(new Set())
  const [loading, setLoading] = useState(true)
  const [refetching, setRefetching] = useState(false)
  const [processing, setProcessing] = useState(false)
  const [currentIndex, setCurrentIndex] = useState(0)
  const [noNewMovies, setNoNewMovies] = useState(false)

  // Prefetch state
  const [prefetchedMovies, setPrefetchedMovies] = useState<MovieResponse[]>([])
  const [isPrefetching, setIsPrefetching] = useState(false)

  const filterNewMovies = useCallback((fetchedMovies: MovieResponse[], seenIds: Set<number>) => {
    return fetchedMovies.filter((movie) => !seenIds.has(movie.movie_id))
  }, [])

  const fetchRecommendations = useCallback(async () => {
    try {
      const result = await getPersonalRecommendationsAction()

      if (result.data) {
        return result.data
      }
      return []
    } catch (error) {
      console.error('Fetch recommendations failed', error)
      toast.error('Failed to fetch movies')
      return []
    }
  }, [])

  const setNewMovies = useCallback((newMovies: MovieResponse[]) => {
    setMovies(newMovies)
    setCurrentIndex(0)
  }, [])

  // Initial fetch
  useEffect(() => {
    const initialFetch = async () => {
      setLoading(true)
      const newMovies = await fetchRecommendations()
      const filtered = filterNewMovies(newMovies, seenMovieIds)

      if (filtered.length > 0) {
        setNewMovies(filtered)
        setNoNewMovies(false)
      } else {
        setNoNewMovies(true)
      }
      setLoading(false)
    }

    initialFetch()
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [])

  const markCurrentAsSeen = useCallback(() => {
    const currentMovie = movies[currentIndex]
    if (currentMovie) {
      setSeenMovieIds((prev) => new Set(prev).add(currentMovie.movie_id))
    }
  }, [movies, currentIndex])

  const prefetchNextMovies = useCallback(async () => {
    if (isPrefetching || prefetchedMovies.length > 0 || noNewMovies) return

    setIsPrefetching(true)
    try {
      const movies = await fetchRecommendations()
      const newMovies = filterNewMovies(movies, seenMovieIds)

      if (newMovies.length > 0) {
        setPrefetchedMovies(newMovies)
      }
    } finally {
      setIsPrefetching(false)
    }
  }, [
    isPrefetching,
    prefetchedMovies.length,
    noNewMovies,
    fetchRecommendations,
    filterNewMovies,
    seenMovieIds
  ])

  useEffect(() => {
    const moviesRemaining = movies.length - currentIndex
    const shouldPrefetch =
      movies.length > 0 &&
      moviesRemaining <= PREFETCH_THRESHOLD &&
      !isPrefetching &&
      prefetchedMovies.length === 0 &&
      !noNewMovies

    if (shouldPrefetch) {
      prefetchNextMovies()
    }
  }, [
    currentIndex,
    movies.length,
    isPrefetching,
    prefetchedMovies.length,
    noNewMovies,
    prefetchNextMovies
  ])

  useEffect(() => {
    const hasFinishedMovies = movies.length > 0 && currentIndex >= movies.length

    if (hasFinishedMovies) {
      if (prefetchedMovies.length > 0) {
        const filtered = filterNewMovies(prefetchedMovies, seenMovieIds)
        if (filtered.length > 0) {
          setMovies(filtered)
          setPrefetchedMovies([])
          setCurrentIndex(0)
        } else {
          setPrefetchedMovies([])
          setRefetching(true)
          fetchRecommendations().then((movies) => {
            const newFiltered = filterNewMovies(movies, seenMovieIds)
            if (newFiltered.length > 0) {
              setMovies(newFiltered)
              setCurrentIndex(0)
              setNoNewMovies(false)
            } else {
              setNoNewMovies(true)
            }
            setRefetching(false)
          })
        }
      } else if (!isPrefetching && !noNewMovies) {
        setRefetching(true)
        fetchRecommendations().then((movies) => {
          const filtered = filterNewMovies(movies, seenMovieIds)
          if (filtered.length > 0) {
            setMovies(filtered)
            setCurrentIndex(0)
            setNoNewMovies(false)
          } else {
            setNoNewMovies(true)
          }
          setRefetching(false)
        })
      } else if (!isPrefetching) {
        setNoNewMovies(true)
      }
    }
  }, [
    currentIndex,
    movies.length,
    prefetchedMovies,
    isPrefetching,
    noNewMovies,
    fetchRecommendations,
    filterNewMovies,
    seenMovieIds
  ])

  const handleAction = async (liked: boolean | null) => {
    if (processing) return
    const currentMovie = movies[currentIndex]
    if (!currentMovie) return

    setProcessing(true)
    try {
      const response = await updatePersonalTasteAction(currentMovie.movie_id, liked)

      if (response.error) {
        toast.error(response.error)
      } else {
        markCurrentAsSeen()
        setCurrentIndex((prev) => prev + 1)
      }
    } catch (error) {
      console.error('Action error', error)
      toast.error('Something went wrong')
    } finally {
      setProcessing(false)
    }
  }

  return {
    currentMovie: movies[currentIndex],
    loading,
    refetching,
    processing,
    noNewMovies,
    handleLike: () => handleAction(true),
    handleDislike: () => handleAction(false),
    handleSkip: () => handleAction(null),
    hasFinishedAllMovies: movies.length === 0 || (currentIndex >= movies.length && noNewMovies)
  }
}
