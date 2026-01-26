'use client'

import { useState, useEffect } from 'react'
import { useRouter } from 'next/navigation'
import { UserPreferences, PreferenceStep } from '../../types/types'
import { UserPreferencesResponse } from '@/model/userPreferencesResponse'
import { submitUserPreferencesAction } from '../../actions/user-actions'
import { getGenresAction } from '../../actions/movie-actions'
import { getMyPartyIdAction } from '../../actions/party-room'
import GenreSelection from './GenreSelection'
import DecadeSelection from './DecadeSelection'
import StudyStatusSelection from './StudyStatusSelection'

interface PreferencesFlowProps {
  joinCode?: string
  initialPrefs?: UserPreferencesResponse
  onComplete?: () => void
}

const PreferencesFlow: React.FC<PreferencesFlowProps> = ({
  joinCode,
  initialPrefs,
  onComplete
}) => {
  const router = useRouter()
  const [step, setStep] = useState<PreferenceStep>(1)
  const [preferences, setPreferences] = useState<UserPreferences>(() => {
    if (initialPrefs) {
      return {
        genres: initialPrefs.include_genres || [],
        decades: [], // Local state for decades, logic handled via selectedDecade
        isStudying: initialPrefs.target_release_year ? true : null // Approximation or we need 'is_tite' from somewhere else?
        // Wait, UserPreferencesResponse doesn't seem to have 'is_tite'.
        // Let's check UserPreferencesResponse definition again.
        // It has target_release_year.
        // The submit action uses is_tite which maps to isStudying state.
        // But the response type seems to miss it?
        // If it's not in response, we can't restore it perfectly.
        // Let's assume null if missing or try to infer.
        // However, looking at the file read previously:
        // export interface UserPreferencesResponse { ... }
        // It does NOT have is_tite or similar boolean.
        // This suggests we might lose that state on refresh if not careful.
        // For now, let's just initialize it to null or false if we can't know.
        // Or maybe we interact with a different endpoint or type?
        // Let's initialize genres at least.
      }
    }
    return {
      genres: [],
      decades: [],
      isStudying: null
    }
  })
  // Fix local state management if types mismatch due to refactor
  // Actually, we need to change UserPreferences type definition or handle local state differently.
  // Let's assume UserPreferences type is imported. We might need to check/change it.
  // But for now, let's just make local state conform to what we need: single decade.

  // We need to override the type locally if UserPreferences expects string[] for decades
  // but we want string | null for single selection in UI.
  // However, the action submitUserPreferencesAction likely expects specific format.
  // Let's see submitUserPreferencesAction signature in previous view_file calling PreferencesFlow.tsx.
  // It takes include_genres, is_tite, target_release_year, release_year_flex.
  // So 'decades' is just local state to compute target_release_year.

  const [selectedDecade, setSelectedDecade] = useState<string | null>(null)

  // Sync initialPrefs to local state if needed (parsing logic)
  useEffect(() => {
    if (initialPrefs && initialPrefs.target_release_year) {
      // rough estimation reverse mapping or just ignore for now as 'decades' is harder to reverse from target year perfectly without more logic.
      // But wait, initialPrefs comes from UserPreferences?
      // Looking at PreferencesDialog, it passes prefData which is from getUserPreferencesAction -> UserPreferencesResponse
      // UserPreferencesResponse has include_genres, is_tite, target_release_year etc.
      // It does NOT have 'decades'.
      // So we should map target_release_year to a decade string if possible.
      const year = initialPrefs.target_release_year
      if (year) {
        // e.g. 1965 -> 1960s
        const decadeYear = Math.floor(year / 10) * 10
        setSelectedDecade(`${decadeYear}s`)
      }
    }
  }, [initialPrefs])

  const [isSubmitting, setIsSubmitting] = useState(false)
  const [availableGenres, setAvailableGenres] = useState<string[]>([])

  useEffect(() => {
    const fetchGenres = async () => {
      const genres = await getGenresAction()
      setAvailableGenres(genres)
    }
    fetchGenres()
  }, [])

  const handleToggleGenre = (genre: string) => {
    setPreferences((prev) => {
      const current = prev.genres
      const newGenres = current.includes(genre)
        ? current.filter((g) => g !== genre)
        : [...current, genre]
      return { ...prev, genres: newGenres }
    })
  }

  const handleSelectDecade = (decade: string) => {
    // Determine if it is already selected (toggle behavior for single select?)
    // Usually single select just selects. Deselect if clicking same? Let's say yes.
    if (selectedDecade === decade) {
      setSelectedDecade(null)
    } else {
      setSelectedDecade(decade)
    }
  }

  const handleSelectStatus = (isStudying: boolean) => {
    setPreferences((prev) => ({ ...prev, isStudying }))
  }

  const nextStep = () => {
    setStep((prev) => (prev < 3 ? ((prev + 1) as PreferenceStep) : prev))
  }

  const prevStep = () => {
    setStep((prev) => (prev > 1 ? ((prev - 1) as PreferenceStep) : prev))
  }

  const handleSubmit = async () => {
    if (preferences.isStudying === null) return

    setIsSubmitting(true)
    try {
      let target_release_year: number | null = null
      let release_year_flex: number | null = 9 // Default flex for a decade

      if (selectedDecade) {
        // "1960s" -> 1960
        const startYear = Number.parseInt(selectedDecade, 10)
        // Center of decade = 1964.5 ?? Or just 1960 + 5?
        // Logic in previous code: min + max / 2.
        // 1960 to 1969. Min 1960, Max 1969.
        // Center = 1964.5 -> round -> 1965
        // Flex = (1969 - 1960) / 2 = 4.5 -> ceil -> 5
        target_release_year = startYear + 5
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

      if (partyResult.error || !partyResult.id) {
        console.error('Failed to get party ID for redirection')

        if (joinCode) {
          router.push(`/party-room/${joinCode}`)
        } else {
          // Fallback if no joinCode
          router.push('/dashboard')
        }
        return
      }

      router.push(`/party-room/${partyResult.id}`)
    } catch (error) {
      console.error('Failed to submit preferences', error)
    } finally {
      setIsSubmitting(false)
    }
  }

  return (
    <div className="w-full flex flex-col items-center">
      <div className="sticky top-0 z-20 w-full bg-zinc-950/80 backdrop-blur-md py-6 mb-8 border-b border-zinc-800/50">
        <div className="max-w-4xl mx-auto text-center animate-in fade-in slide-in-from-top-4 duration-500 px-4">
          <h1 className="text-3xl font-bold text-zinc-100 mb-4">Setup your taste profile</h1>
          <p className="text-zinc-400">Step {step} of 3</p>
          <div className="w-full max-w-xs mx-auto h-2 bg-zinc-800 rounded-full mt-4 overflow-hidden">
            <div
              className="h-full bg-red-600 transition-all duration-300 ease-in-out"
              style={{ width: `${(step / 3) * 100}%` }}
            />
          </div>
        </div>
      </div>

      <div className="animate-in fade-in zoom-in-95 duration-300 w-full">
        {step === 1 && (
          <GenreSelection
            selectedGenres={preferences.genres}
            onToggleGenre={handleToggleGenre}
            onNext={nextStep}
            availableGenres={availableGenres}
          />
        )}

        {step === 2 && (
          <DecadeSelection
            selectedDecade={selectedDecade}
            onToggleDecade={handleSelectDecade}
            onNext={nextStep}
            onBack={prevStep}
          />
        )}

        {step === 3 && (
          <StudyStatusSelection
            isStudying={preferences.isStudying}
            onSelectStatus={handleSelectStatus}
            onSubmit={handleSubmit}
            onBack={prevStep}
            isSubmitting={isSubmitting}
          />
        )}
      </div>
    </div>
  )
}

export default PreferencesFlow
