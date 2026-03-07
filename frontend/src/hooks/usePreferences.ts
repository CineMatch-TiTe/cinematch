import { useState, useCallback } from 'react'
import useSWR from 'swr'
import { useRouter } from 'next/navigation'
import { getGenresAction } from '@/actions/movie-actions'
import { submitUserPreferencesAction } from '@/actions/user-actions'
import { updateUserPreferencesAction } from '@/actions/user'
import { getMyPartyIdAction } from '@/actions/party-room'
import { UserPreferencesResponse } from '@/model/userPreferencesResponse'
import { UserPreferences, PreferenceStep } from '@/types/types'
import { toast } from 'sonner'

interface UsePreferencesProps {
  initialPrefs?: UserPreferencesResponse
  onComplete?: () => void
  joinCode?: string
}

const fetchGenres = async () => {
  const genres = await getGenresAction()
  return genres.length > 0 ? genres : ['Action', 'Comedy', 'Drama', 'Horror', 'Sci-Fi', 'Romance', 'Thriller', 'Documentary', 'Animation']
}

export const usePreferences = ({ initialPrefs, onComplete }: UsePreferencesProps = {}) => {
  const router = useRouter()
  const [step, setStep] = useState<PreferenceStep>(1)
  const [isSubmitting, setIsSubmitting] = useState(false)

  const { data: availableGenres, isLoading: isGenresLoading } = useSWR('/api/movie/genres', fetchGenres)

  const [preferences, setPreferences] = useState<UserPreferences>(() => {
    if (initialPrefs) {
      return {
        genres: initialPrefs.include_genres || [],
        isStudying: initialPrefs.target_release_year ? true : null
      }
    }
    return {
      genres: [],
      isStudying: null
    }
  })

  const [selectedYear, setSelectedYear] = useState<number | null>(
    initialPrefs?.target_release_year ?? null
  )


  const handleToggleGenre = useCallback((genre: string) => {
    setPreferences((prev) => {
      const current = prev.genres
      const newGenres = current.includes(genre)
        ? current.filter((g) => g !== genre)
        : [...current, genre]
      return { ...prev, genres: newGenres }
    })
  }, [])

  const handleChangeYear = useCallback((year: number | null) => {
    setSelectedYear(year)
  }, [])

  const handleSelectStatus = useCallback((isStudying: boolean) => {
    setPreferences((prev) => ({ ...prev, isStudying }))
  }, [])

  const nextStep = useCallback(() => {
    setStep((prev) => (prev < 3 ? ((prev + 1) as PreferenceStep) : prev))
  }, [])

  const prevStep = useCallback(() => {
    setStep((prev) => (prev > 1 ? ((prev - 1) as PreferenceStep) : prev))
  }, [])

  const hasChanges = useCallback(() => {
    if (!initialPrefs) return true
    const currentGenres = [...preferences.genres].sort((a, b) => a.localeCompare(b))
    const initialGenres = [...(initialPrefs.include_genres || [])].sort((a, b) => a.localeCompare(b))
    return JSON.stringify(currentGenres) !== JSON.stringify(initialGenres)
  }, [preferences.genres, initialPrefs])

  const submitPreferences = async () => {
    // For wizard flow, we need isStudying to be set
    if (preferences.isStudying === null) return

    setIsSubmitting(true)
    try {
      let target_release_year: number | null = null
      let release_year_flex: number | null = 9

      if (selectedYear !== null) {
        target_release_year = selectedYear
        release_year_flex = 5
      }

      await submitUserPreferencesAction({
        include_genres: preferences.genres,
        is_tite: preferences.isStudying,
        target_release_year,
        release_year_flex,
        exclude_genres: []
      })

      if (onComplete) {
        onComplete()
        return
      }

      const partyResult = await getMyPartyIdAction()

      // We always redirect to the party UUID, even if they used a join code.
      if (partyResult.error || !partyResult.id) {
        // Not in a party, which is completely normal for new users
        router.push('/dashboard')
        return
      }

      router.push(`/party-room?id=${partyResult.id}`)
    } catch (error) {
      console.error('Failed to submit preferences', error)
      toast.error('Failed to submit preferences')
    } finally {
      setIsSubmitting(false)
    }
  }

  const updatePreferences = async () => {
    setIsSubmitting(true)
    try {
      const res = await updateUserPreferencesAction({
        include_genres: preferences.genres,
        exclude_genres: initialPrefs?.exclude_genres || [],
      })

      if (res.error) {
        toast.error(res.error)
      } else {
        toast.success('Preferences saved')
        if (onComplete) onComplete()
      }
    } catch (error) {
      console.error(error)
      toast.error('Failed to update preferences')
    } finally {
      setIsSubmitting(false)
    }
  }

  return {
    step,
    preferences,
    selectedYear,
    isSubmitting,
    availableGenres,
    isGenresLoading,
    hasChanges,
    handleToggleGenre,
    handleChangeYear,
    handleSelectStatus,
    nextStep,
    prevStep,
    submitPreferences,
    updatePreferences
  }
}
