import { Button } from '@/components/ui/button'
import { Film, Loader2 } from 'lucide-react'
import { UserPreferencesResponse } from '@/model/userPreferencesResponse'
import React, { useState } from 'react'
import useSWR from 'swr'
import { getGenresAction } from '@/actions/movie-actions'
import { updateUserPreferencesAction } from '@/actions/user'
import { toast } from 'sonner'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'

const fetchGenres = async () => {
  const genres = await getGenresAction()
  return genres.length > 0 ? genres : ['Action', 'Comedy', 'Drama', 'Horror', 'Sci-Fi', 'Romance', 'Thriller', 'Documentary', 'Animation']
}

const PreferencesTabFlow = ({ initialPrefs, onSuccess }: { initialPrefs: UserPreferencesResponse, onSuccess: () => void }) => {
  const [activeSubTab, setActiveSubTab] = useState('genres')
  const [loading, setLoading] = useState(false)
  const { data: availableGenres, isLoading: isGenresLoading } = useSWR('/api/movie/genres', fetchGenres)

  const [selectedGenres, setSelectedGenres] = useState<string[]>(initialPrefs.include_genres || [])

  const hasChanges = () => {
    return JSON.stringify([...selectedGenres].sort()) !== JSON.stringify([...(initialPrefs.include_genres || [])].sort())
  }

  const toggleGenre = (genre: string) => {
    setSelectedGenres(prev => prev.includes(genre) ? prev.filter(g => g !== genre) : [...prev, genre])
  }
  const handleSave = async () => {
    setLoading(true)
    try {
      const res = await updateUserPreferencesAction({
        include_genres: selectedGenres,
        exclude_genres: initialPrefs.exclude_genres || [],
      })

      if (res.error) {
        toast.error(res.error)
      } else {
        toast.success('Preferences saved')
        onSuccess()
      }
    } finally {
      setLoading(false)
    }
  }

  return (
    <div className="space-y-6">
      <Tabs value={activeSubTab} onValueChange={setActiveSubTab} className="w-full">
        <TabsList className="grid w-full grid-cols-3 bg-zinc-900 border border-red-900/30 p-1">
          <TabsTrigger value="genres" className="py-2 text-xs gap-2 text-white data-[state=active]:bg-red-700 data-[state=active]:text-white">
            <Film className="h-3 w-3" /> Genres
          </TabsTrigger>
        </TabsList>

        {/* Step 1: Genres */}
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
                  onClick={() => toggleGenre(genre)}
                  className={`px-3 py-1.5 rounded-full border text-xs transition-all ${
                    selectedGenres.includes(genre)
                      ? 'bg-red-600 border-red-600 text-white'
                      : 'bg-zinc-800 border-red-900/30 text-white hover:border-red-700 hover:bg-zinc-700'
                  }`}
                >
                  {genre}
                </button>
              ))}
            </div>
          )}
          {selectedGenres.length > 0 && (
            <p className="text-xs text-zinc-500 text-center">
              Selected: {selectedGenres.join(', ')}
            </p>
          )}
        </TabsContent>

      </Tabs>

      {/* Save Button */}
      <Button
        onClick={handleSave}
        disabled={loading || !hasChanges()}
        className={`w-full ${hasChanges() ? 'bg-red-700 hover:bg-red-600' : 'bg-zinc-700'} text-white font-medium`}
      >
        {loading && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
        {hasChanges() ? 'Save Preferences' : 'No Changes'}
      </Button>
    </div>
  )
}

export default PreferencesTabFlow