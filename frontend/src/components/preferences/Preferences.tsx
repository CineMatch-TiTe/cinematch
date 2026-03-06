'use client'

import { Button } from '@/components/ui/button'
import { Film, Loader2 } from 'lucide-react'
import { UserPreferencesResponse } from '@/model/userPreferencesResponse'
import React from 'react'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import GenreSelection from './GenreSelection'
import YearSelection from './YearSelection'
import StudyStatusSelection from './StudyStatusSelection'
import { usePreferences } from '@/hooks/usePreferences'

interface PreferencesProps {
  mode: 'wizard' | 'settings'
  initialPrefs?: UserPreferencesResponse
  onComplete?: () => void
  joinCode?: string
}

const Preferences: React.FC<PreferencesProps> = ({ mode, initialPrefs, onComplete, joinCode }) => {
  const {
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
  } = usePreferences({ initialPrefs, onComplete, joinCode })

  // Wizard Mode
  if (mode === 'wizard') {
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
              availableGenres={availableGenres || []}
            />
          )}

          {step === 2 && (
            <YearSelection
              selectedYear={selectedYear}
              onChangeYear={handleChangeYear}
              onNext={nextStep}
              onBack={prevStep}
            />
          )}

          {step === 3 && (
            <StudyStatusSelection
              isStudying={preferences.isStudying}
              onSelectStatus={handleSelectStatus}
              onSubmit={submitPreferences}
              onBack={prevStep}
              isSubmitting={isSubmitting}
            />
          )}
        </div>
      </div>
    )
  }

  // Settings Mode
  return (
    <div className="space-y-6">
      <Tabs defaultValue="genres" className="w-full">
        <TabsList className="grid w-full grid-cols-3 bg-zinc-900 border border-red-900/30 p-1">
          <TabsTrigger value="genres" className="py-2 text-xs gap-2 text-white data-[state=active]:bg-red-700 data-[state=active]:text-white">
            <Film className="h-3 w-3" /> Genres
          </TabsTrigger>
        </TabsList>

        <TabsContent value="genres" className="space-y-4 mt-4">
          <p className="text-xs text-zinc-400 text-center">Select your favorite movie genres</p>
          {isGenresLoading ? (
            <div className="flex justify-center p-4"><Loader2 className="animate-spin text-red-500" /></div>
          ) : (
            <div className="flex flex-wrap gap-2 justify-center">
              {(availableGenres || []).map((genre) => (
                <button
                  type="button"
                  key={genre}
                  onClick={() => handleToggleGenre(genre)}
                  className={`px-3 py-1.5 rounded-full border text-xs transition-all ${
                    preferences.genres.includes(genre)
                      ? 'bg-red-600 border-red-600 text-white'
                      : 'bg-zinc-800 border-red-900/30 text-white hover:border-red-700 hover:bg-zinc-700'
                  }`}
                >
                  {genre}
                </button>
              ))}
            </div>
          )}
          {preferences.genres.length > 0 && (
            <p className="text-xs text-zinc-500 text-center">
              Selected: {preferences.genres.join(', ')}
            </p>
          )}
        </TabsContent>
      </Tabs>

      <Button
        onClick={updatePreferences}
        disabled={isSubmitting || !hasChanges()}
        className={`w-full ${hasChanges() ? 'bg-red-700 hover:bg-red-600' : 'bg-zinc-700'} text-white font-medium`}
      >
        {isSubmitting && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
        {hasChanges() ? 'Save Preferences' : 'No Changes'}
      </Button>
    </div>
  )
}

export default Preferences
