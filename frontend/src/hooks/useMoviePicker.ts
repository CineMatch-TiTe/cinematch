'use client'

import { useCallback, useEffect, useState } from 'react'
import { toast } from 'sonner'
import { getRecommendedMoviesAction, pickMovieAction } from '@/actions/party-room'
import { MovieResponse } from '@/model/movieResponse'
// Number of movies remaining before triggering prefetch
const PREFETCH_THRESHOLD = 3

interface UseMoviePickerOptions {
  partyId: string
}

interface UseMoviePickerReturn {
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

export function useMoviePicker({ partyId }: UseMoviePickerOptions): UseMoviePickerReturn {
  const [movies, setMovies] = useState<MovieResponse[]>([])
  const [seenMovieIds, setSeenMovieIds] = useState<Set<number>>(new Set())
  const [loading, setLoading] = useState(true)
  const [refetching, setRefetching] = useState(false)
  const [processing, setProcessing] = useState(false)
  const [currentIndex, setCurrentIndex] = useState(0)
  const [noNewMovies, setNoNewMovies] = useState(false)

  // Prefetch state for smooth transitions
  const [prefetchedMovies, setPrefetchedMovies] = useState<MovieResponse[]>([])
  const [isPrefetching, setIsPrefetching] = useState(false)

  const filterNewMovies = useCallback((fetchedMovies: MovieResponse[], seenIds: Set<number>) => {
    return fetchedMovies.filter((movie) => !seenIds.has(movie.movie_id))
  }, [])

  // Search movies using user preferences
  // Fetch party recommendations
  const fetchRecommendations = useCallback(async () => {
    try {
      const result = await getRecommendedMoviesAction(partyId)

      if (result.data) {
        return result.data
      }
      return []
    } catch (error) {
      console.error('Fetch recommendations failed', error)
      toast.error('Failed to fetch movies')
      return []
    }
  }, [partyId])

  // Helper to set new movies
  const setNewMovies = useCallback((newMovies: MovieResponse[]) => {
    setMovies(newMovies)
    setCurrentIndex(0)
  }, [])

  // Initial fetch
  useEffect(() => {
    const initialFetch = async () => {
      setLoading(true)
      const newMovies = await fetchRecommendations()

      // Filter out seen ones just in case backend includes them
      // (Backend normally excludes picked movies, but client-side checking is good practice)
      // Actually backend recommendations should be fresh.
      // But we can filter against seenMovieIds just in case of race conditions during session.
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
  }, []) // Run once on mount

  // Track current movie as seen before moving to next
  const markCurrentAsSeen = useCallback(() => {
    const currentMovie = movies[currentIndex]
    if (currentMovie) {
      setSeenMovieIds((prev) => new Set(prev).add(currentMovie.movie_id))
    }
  }, [movies, currentIndex])

  // Prefetch next batch of movies in the background
  const prefetchNextMovies = useCallback(async () => {
    if (isPrefetching || prefetchedMovies.length > 0 || noNewMovies) return

    setIsPrefetching(true)
    try {
      // Fetch more recommendations
      const movies = await fetchRecommendations()

      const newMovies = filterNewMovies(movies, seenMovieIds)

      if (newMovies.length > 0) {
        setPrefetchedMovies(newMovies)
      } else {
        // If no new movies returned, and we tried fetching, it means we are likely done.
        // To avoid infinite loop, we must stop prefetching.
        // However, we only mark noNewMovies if we are sure.
        // But if we don't, the effect will trigger again immediately.
        // So we MUST set it to true or handle it.
        // Setting it to true will stop prefetching.
        // When the user finishes current movies, it will try one last refetch (in the other useEffect)
        // or just accept noNewMovies.
        // Actually the other useEffect checks !noNewMovies.
        // So setting it here is correct.
        setNoNewMovies(true)
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

  // Trigger prefetch when approaching the end of current movies
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

  // Seamlessly merge prefetched movies when current batch is exhausted
  useEffect(() => {
    const hasFinishedMovies = movies.length > 0 && currentIndex >= movies.length

    if (hasFinishedMovies) {
      if (prefetchedMovies.length > 0) {
        // Seamlessly transition to prefetched movies, but filter again just in case
        const filtered = filterNewMovies(prefetchedMovies, seenMovieIds)
        if (filtered.length > 0) {
          setMovies(filtered)
          setPrefetchedMovies([])
          setCurrentIndex(0)
        } else {
          // All prefetched were seen, try fetching more
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
        // Fallback: no prefetched movies ready, show loading and fetch
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
        // No more movies available
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

  const handleLike = async () => {
    if (processing) return
    const currentMovie = movies[currentIndex]
    if (!currentMovie) return

    setProcessing(true)
    try {
      const result = await pickMovieAction(partyId, currentMovie.movie_id, true)
      if (result.error) {
        toast.error(result.error)
      } else {
        markCurrentAsSeen()
        setCurrentIndex((prev) => prev + 1)
      }
    } catch (error) {
      console.error('Pick error', error)
      toast.error('Something went wrong')
    } finally {
      setProcessing(false)
    }
  }

  const handleDislike = async () => {
    if (processing) return
    const currentMovie = movies[currentIndex]
    if (!currentMovie) return

    setProcessing(true)
    try {
      const result = await pickMovieAction(partyId, currentMovie.movie_id, false)
      if (result.error) {
        toast.error(result.error)
      } else {
        markCurrentAsSeen()
        setCurrentIndex((prev) => prev + 1)
      }
    } catch (error) {
      console.error('Dislike error', error)
      toast.error('Something went wrong')
    } finally {
      setProcessing(false)
    }
  }

  const handleSkip = async () => {
    if (processing) return
    const currentMovie = movies[currentIndex]
    if (!currentMovie) return

    setProcessing(true)
    try {
      const result = await pickMovieAction(partyId, currentMovie.movie_id, null)
      if (result.error) {
        toast.error(result.error)
      } else {
        markCurrentAsSeen()
        setCurrentIndex((prev) => prev + 1)
      }
    } catch (error) {
      console.error('Skip error', error)
      toast.error('Something went wrong')
    } finally {
      setProcessing(false)
    }
  }

  const currentMovie = movies[currentIndex]
  const hasFinishedAllMovies = movies.length === 0 || (currentIndex >= movies.length && noNewMovies)

  return {
    currentMovie,
    loading,
    refetching,
    processing,
    noNewMovies,
    handleLike,
    handleDislike,
    handleSkip,
    hasFinishedAllMovies
  }
}
