'use client'

import { useCallback, useEffect, useRef, useState } from 'react'
import { toast } from 'sonner'
import { getUserPreferencesAction, pickMovieAction, searchMoviesAction } from '@/actions/party-room'
import { MovieResponse } from '@/model/movieResponse'
import { SearchFilter } from '@/model/searchFilter'

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
  handleSkip: () => void
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
  const [searchPage, setSearchPage] = useState(1)
  const searchGenresRef = useRef<string[]>([])
  const searchYearRef = useRef<{ min?: number; max?: number } | undefined>(undefined)

  // Prefetch state for smooth transitions
  const [prefetchedMovies, setPrefetchedMovies] = useState<MovieResponse[]>([])
  const [isPrefetching, setIsPrefetching] = useState(false)

  const filterNewMovies = useCallback((fetchedMovies: MovieResponse[], seenIds: Set<number>) => {
    return fetchedMovies.filter((movie) => !seenIds.has(movie.movie_id))
  }, [])

  // Search movies using user preferences
  const searchMoviesFromPreferences = useCallback(async (page: number = 1) => {
    try {
      // Get user preferences if we don't have genres yet
      if (searchGenresRef.current.length === 0) {
        const prefsResult = await getUserPreferencesAction()
        if (prefsResult.data?.include_genres) {
          searchGenresRef.current = prefsResult.data.include_genres
        }
      }

      // Use genres for filtering
      const genres = searchGenresRef.current

      // Calculate year range if not already set
      if (!searchYearRef.current) {
        const prefsResult = await getUserPreferencesAction()
        if (prefsResult.data) {
          const { target_release_year, release_year_flex } = prefsResult.data
          if (target_release_year) {
            searchYearRef.current = {
              min: target_release_year - release_year_flex,
              max: target_release_year + release_year_flex
            }
          } else {
            searchYearRef.current = {} // Mark as checked but empty
          }
        }
      }

      const yearFilter = searchYearRef.current

      const filter: SearchFilter = {
        include_genres: genres,
        exclude_genres: [],
        min_year: yearFilter?.min,
        max_year: yearFilter?.max
      }

      const searchResult = await searchMoviesAction(filter, page)

      if (searchResult.data) {
        return searchResult.data
      }
    } catch (error) {
      console.error('Search failed', error)
      toast.error('Failed to search movies')
    }
    return null
  }, [])

  // Helper to set new movies
  const setNewMovies = useCallback((newMovies: MovieResponse[]) => {
    setMovies(newMovies)
    setCurrentIndex(0)
  }, [])

  // Fetch movies from search with pagination
  const fetchSearchMovies = useCallback(async (): Promise<boolean> => {
    const searchedMovies = await searchMoviesFromPreferences(searchPage)
    if (!searchedMovies) return false

    const newMovies = filterNewMovies(searchedMovies, seenMovieIds)
    if (newMovies.length > 0) {
      setNewMovies(newMovies)
      setSearchPage((prev) => prev + 1)
      return true
    }

    // Try next page if current page has no new movies
    setSearchPage((prev) => prev + 1)
    const nextPageMovies = await searchMoviesFromPreferences(searchPage + 1)
    if (!nextPageMovies) return false

    const nextNewMovies = filterNewMovies(nextPageMovies, seenMovieIds)
    if (nextNewMovies.length > 0) {
      setNewMovies(nextNewMovies)
      setSearchPage((prev) => prev + 1)
      return true
    }
    return false
  }, [searchMoviesFromPreferences, searchPage, filterNewMovies, seenMovieIds, setNewMovies])

  // Initial fetch using search
  useEffect(() => {
    const initialFetch = async () => {
      const success = await fetchSearchMovies()
      if (!success) {
        setNoNewMovies(true)
      }
      setLoading(false)
    }

    initialFetch()
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [])

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
      const searchedMovies = await searchMoviesFromPreferences(searchPage)
      if (!searchedMovies) {
        setIsPrefetching(false)
        return
      }

      const newMovies = filterNewMovies(searchedMovies, seenMovieIds)
      if (newMovies.length > 0) {
        setPrefetchedMovies(newMovies)
        setSearchPage((prev) => prev + 1)
      } else {
        // Try next page if current page has no new movies
        const nextSearchPage = searchPage + 1
        setSearchPage(nextSearchPage)
        const nextPageMovies = await searchMoviesFromPreferences(nextSearchPage)
        if (nextPageMovies) {
          const nextNewMovies = filterNewMovies(nextPageMovies, seenMovieIds)
          if (nextNewMovies.length > 0) {
            setPrefetchedMovies(nextNewMovies)
            setSearchPage((prev) => prev + 1)
          }
        }
      }
    } finally {
      setIsPrefetching(false)
    }
  }, [
    isPrefetching,
    prefetchedMovies.length,
    noNewMovies,
    searchMoviesFromPreferences,
    searchPage,
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
        // Seamlessly transition to prefetched movies
        setMovies(prefetchedMovies)
        setPrefetchedMovies([])
        setCurrentIndex(0)
      } else if (!isPrefetching && !noNewMovies) {
        // Fallback: no prefetched movies ready, show loading and fetch
        setRefetching(true)
        fetchSearchMovies().then((found) => {
          if (!found) {
            setNoNewMovies(true)
          }
          setRefetching(false)
        })
      } else if (!isPrefetching) {
        // No more movies available
        setNoNewMovies(true)
      }
    }
  }, [currentIndex, movies.length, prefetchedMovies, isPrefetching, noNewMovies, fetchSearchMovies])

  const handleLike = async () => {
    if (processing) return
    const currentMovie = movies[currentIndex]
    if (!currentMovie) return

    setProcessing(true)
    try {
      const result = await pickMovieAction(partyId, currentMovie.movie_id)
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

  const handleSkip = () => {
    if (processing) return
    markCurrentAsSeen()
    setCurrentIndex((prev) => prev + 1)
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
    handleSkip,
    hasFinishedAllMovies
  }
}
