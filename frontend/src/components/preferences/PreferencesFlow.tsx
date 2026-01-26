'use client'

import React, { useState } from 'react'
import { useRouter } from 'next/navigation'
import { UserPreferences, PreferenceStep } from '../../types/types'
import { submitUserPreferencesAction } from '../../actions/user-actions'
import { getGenresAction } from '../../actions/movie-actions'
import { getMyPartyIdAction } from '../../actions/party-room'
import GenreSelection from './GenreSelection'
import DecadeSelection from './DecadeSelection'
import StudyStatusSelection from './StudyStatusSelection'

interface PreferencesFlowProps {
  joinCode: string
}

const PreferencesFlow: React.FC<PreferencesFlowProps> = ({ joinCode }) => {
  const router = useRouter()
  const [step, setStep] = useState<PreferenceStep>(1)
  const [preferences, setPreferences] = useState<UserPreferences>({
    genres: [],
    decades: [],
    isStudying: null
  })
  const [isSubmitting, setIsSubmitting] = useState(false)
  const [availableGenres, setAvailableGenres] = useState<string[]>([])

  React.useEffect(() => {
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

  const handleToggleDecade = (decade: string) => {
    setPreferences((prev) => {
      const current = prev.decades
      const newDecades = current.includes(decade)
        ? current.filter((d) => d !== decade)
        : [...current, decade]
      return { ...prev, decades: newDecades }
    })
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
      const decadeYears = preferences.decades.map((d) => Number.parseInt(d))
      let target_release_year: number | null = null
      let release_year_flex: number | null = null

      if (decadeYears.length > 0) {
        const minYear = Math.min(...decadeYears)
        const maxYear = Math.max(...decadeYears) + 9
        target_release_year = Math.round((minYear + maxYear) / 2)
        release_year_flex = Math.ceil((maxYear - minYear) / 2)
      }

      await submitUserPreferencesAction({
        include_genres: preferences.genres,
        is_tite: preferences.isStudying,
        target_release_year,
        release_year_flex,
        exclude_genres: []
      })

      // Get correct party ID for redirection
      const partyResult = await getMyPartyIdAction()

      if (partyResult.error || !partyResult.id) {
        console.error('Failed to get party ID for redirection')
        // Fallback to join code if fetching ID fails (though likely to fail 404)
        // or just error out. For now, try joinCode as last resort or show error?
        // Let's stick to the plan: if we can't get the ID, something is wrong.
        // But to be safe, maybe just log and try the joinCode or reload.
        router.push(`/party-room/${joinCode}`)
        return
      }

      // Redirect to party with correct UUID
      router.push(`/party-room/${partyResult.id}`)
    } catch (error) {
      console.error('Failed to submit preferences', error)
      // Handle error (toast, etc.)
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
            selectedDecades={preferences.decades}
            onToggleDecade={handleToggleDecade}
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
