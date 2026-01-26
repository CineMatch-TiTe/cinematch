'use client'

import { useCallback, useEffect, useRef, useState } from 'react'
import { toast } from 'sonner'
import { Loader2, X, RefreshCw } from 'lucide-react'
import { Button } from '@/components/ui/button'
import MovieCard from './MovieCard'
import {
  getRecommendedMoviesAction,
  getUserPreferencesAction,
  pickMovieAction,
  searchMoviesAction
} from '@/actions/party-room'
import { MovieResponse } from '@/model/movieResponse'

interface PickingFlowProps {
  partyId: string
  onClose: () => void
}

export default function PickingFlow({ partyId, onClose }: PickingFlowProps) {
  const [movies, setMovies] = useState<MovieResponse[]>([])
  const [seenMovieIds, setSeenMovieIds] = useState<Set<number>>(new Set())
  const [loading, setLoading] = useState(true)
  const [refetching, setRefetching] = useState(false)
  const [processing, setProcessing] = useState(false)
  const [currentIndex, setCurrentIndex] = useState(0)
  const [noNewMovies, setNoNewMovies] = useState(false)
  const [usingSearch, setUsingSearch] = useState(false)
  const [searchPage, setSearchPage] = useState(1)
  const searchGenresRef = useRef<string[]>([])

  const fetchMovies = useCallback(async () => {
    try {
      const result = await getRecommendedMoviesAction()
      if (result.error) {
        toast.error(result.error)
        return null
      } else if (result.data) {
        return result.data
      }
    } catch (error) {
      console.error('Failed to fetch movies', error)
      toast.error('Failed to load movies')
    }
    return null
  }, [])

  const filterNewMovies = useCallback((fetchedMovies: MovieResponse[], seenIds: Set<number>) => {
    return fetchedMovies.filter((movie) => !seenIds.has(movie.movie_id))
  }, [])

  useEffect(() => {
    const initialFetch = async () => {
      const fetchedMovies = await fetchMovies()
      if (fetchedMovies) {
        setMovies(fetchedMovies)
      }
      setLoading(false)
    }

    initialFetch()
  }, [fetchMovies])

  // Track current movie as seen before moving to next
  const markCurrentAsSeen = useCallback(() => {
    const currentMovie = movies[currentIndex]
    if (currentMovie) {
      setSeenMovieIds((prev) => new Set(prev).add(currentMovie.movie_id))
    }
  }, [movies, currentIndex])

  // Search movies using user preferences as fallback
  const searchMoviesFromPreferences = useCallback(async (page: number = 1) => {
    try {
      // Get user preferences if we don't have genres yet
      if (searchGenresRef.current.length === 0) {
        const prefsResult = await getUserPreferencesAction()
        if (prefsResult.data?.include_genres) {
          searchGenresRef.current = prefsResult.data.include_genres
        }
      }

      // Use genres as search queries, or fallback to generic terms
      const genres = searchGenresRef.current
      const searchTerms =
        genres.length > 0 ? genres : ['popular', 'classic', 'new release', 'trending']

      // Search using a random genre/term from user preferences
      const randomTerm = searchTerms[Math.floor(Math.random() * searchTerms.length)]
      const searchResult = await searchMoviesAction(randomTerm, page)

      if (searchResult.data) {
        return searchResult.data
      }
    } catch (error) {
      console.error('Search fallback failed', error)
    }
    return null
  }, [])

  // Helper to set new movies and show success toast
  const setNewMoviesWithToast = useCallback((newMovies: MovieResponse[]) => {
    setMovies(newMovies)
    setCurrentIndex(0)
  }, [])

  // Try to fetch movies from recommendations
  const tryRecommendations = useCallback(async (): Promise<boolean> => {
    const fetchedMovies = await fetchMovies()
    if (!fetchedMovies) return false

    const newMovies = filterNewMovies(fetchedMovies, seenMovieIds)
    if (newMovies.length > 0) {
      setNewMoviesWithToast(newMovies)
      return true
    }
    return false
  }, [fetchMovies, filterNewMovies, seenMovieIds, setNewMoviesWithToast])

  // Try to fetch movies from search with pagination
  const trySearchWithPages = useCallback(async (): Promise<boolean> => {
    const searchedMovies = await searchMoviesFromPreferences(searchPage)
    if (!searchedMovies) return false

    const newMovies = filterNewMovies(searchedMovies, seenMovieIds)
    if (newMovies.length > 0) {
      setNewMoviesWithToast(newMovies)
      setSearchPage((prev) => prev + 1)
      return true
    }

    // Try next page if current page has no new movies
    setSearchPage((prev) => prev + 1)
    const nextPageMovies = await searchMoviesFromPreferences(searchPage + 1)
    if (!nextPageMovies) return false

    const nextNewMovies = filterNewMovies(nextPageMovies, seenMovieIds)
    if (nextNewMovies.length > 0) {
      setNewMoviesWithToast(nextNewMovies)
      setSearchPage((prev) => prev + 1)
      return true
    }
    return false
  }, [
    searchMoviesFromPreferences,
    searchPage,
    filterNewMovies,
    seenMovieIds,
    setNewMoviesWithToast
  ])

  // Refetch recommendations and load new unseen movies, with search fallback
  const handleRefetch = useCallback(async () => {
    setRefetching(true)
    setNoNewMovies(false)

    try {
      // First try recommendations if we haven't switched to search yet
      if (!usingSearch) {
        const foundNew = await tryRecommendations()
        if (foundNew) return

        // Switch to search mode
        setUsingSearch(true)
      }

      // Use search as fallback
      const foundFromSearch = await trySearchWithPages()
      if (!foundFromSearch) {
        setNoNewMovies(true)
      }
    } finally {
      setRefetching(false)
    }
  }, [usingSearch, tryRecommendations, trySearchWithPages])

  // Auto-refetch when movies run out
  useEffect(() => {
    const hasFinishedMovies = movies.length > 0 && currentIndex >= movies.length
    if (hasFinishedMovies && !refetching && !noNewMovies) {
      handleRefetch()
    }
  }, [currentIndex, movies.length, refetching, noNewMovies, handleRefetch])

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

  const handleSkip = () => {
    if (processing) return
    markCurrentAsSeen()
    setCurrentIndex((prev) => prev + 1)
  }

  if (loading || refetching) {
    return (
      <div className="fixed inset-0 z-50 flex flex-col items-center justify-center bg-zinc-950/90 backdrop-blur-md">
        <Loader2 className="w-10 h-10 text-white animate-spin mb-4" />
        <p className="text-zinc-400 animate-pulse">
          {refetching ? 'Finding more movies...' : 'Finding movies for you...'}
        </p>
      </div>
    )
  }

  // If no movies available or truly no new movies
  if (movies.length === 0 || (currentIndex >= movies.length && noNewMovies)) {
    return (
      <div className="fixed inset-0 z-50 flex flex-col items-center justify-center bg-zinc-950/90 backdrop-blur-md p-6 text-center">
        <h2 className="text-2xl font-bold text-white mb-2">That&apos;s all for now!</h2>
        <p className="text-zinc-400 mb-8 max-w-xs">
          We&apos;ve run out of recommendations based on current preferences. Check back later or
          wait for others!
        </p>
        <div className="flex gap-3">
          <Button
            onClick={handleRefetch}
            size="lg"
            variant="outline"
            className="border-white/20 text-white hover:bg-white/10"
          >
            <RefreshCw className="w-4 h-4 mr-2" />
            Try Again
          </Button>
          <Button onClick={onClose} size="lg" className="bg-white text-black hover:bg-zinc-200">
            Return to Party
          </Button>
        </div>
      </div>
    )
  }

  // Still processing refetch, show nothing here (loading state handles it)
  if (currentIndex >= movies.length) {
    return null
  }

  const currentMovie = movies[currentIndex]

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
