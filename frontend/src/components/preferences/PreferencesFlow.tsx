'use client'

import React, { useState } from 'react'
import { useRouter } from 'next/navigation'
import { MovieGenre, UserPreferences, PreferenceStep } from '../../types/types'
import GenreSelection from './GenreSelection'
import DecadeSelection from './DecadeSelection'
import StudyStatusSelection from './StudyStatusSelection'
import { submitPreferences } from '../../app/preferences/api'

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

  const handleToggleGenre = (genre: MovieGenre) => {
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
      await submitPreferences(joinCode, preferences)
      // Redirect to party or success page
      router.push(`/party/${joinCode}`)
    } catch (error) {
      console.error('Failed to submit preferences', error)
      // Handle error (toast, etc.)
    } finally {
      setIsSubmitting(false)
    }
  }

  return (
    <div className="min-h-screen bg-background text-foreground p-4 md:p-8 flex flex-col items-center pt-10 md:pt-20">
      <div className="max-w-4xl mx-auto mb-8 text-center animate-in fade-in slide-in-from-top-4 duration-500">
        <h1 className="text-3xl font-bold bg-clip-text text-red-900 mb-8">
          Setup your taste profile
        </h1>
        <p className="text-muted-foreground">Step {step} of 3</p>
        <div className="w-full max-w-xs mx-auto h-2 bg-secondary rounded-full mt-4 overflow-hidden">
          <div
            className="h-full bg-primary transition-all duration-300 ease-in-out"
            style={{ width: `${(step / 3) * 100}%` }}
          />
        </div>
      </div>

      <div className="animate-in fade-in zoom-in-95 duration-300 w-full">
        {step === 1 && (
          <GenreSelection
            selectedGenres={preferences.genres}
            onToggleGenre={handleToggleGenre}
            onNext={nextStep}
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
