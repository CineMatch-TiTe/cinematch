import React from 'react'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from '@/components/ui/card'

interface GenreSelectionProps {
  selectedGenres: string[]
  onToggleGenre: (genre: string) => void
  onNext: () => void
  availableGenres: string[]
}

const GenreSelection: React.FC<GenreSelectionProps> = ({
  selectedGenres,
  onToggleGenre,
  onNext,
  availableGenres
}) => {
  const genres = availableGenres

  return (
    <Card className="w-full mx-auto border-zinc-800 bg-zinc-900/50 backdrop-blur-xl shadow-2xl">
      <CardHeader>
        <CardTitle className="text-zinc-100">Step 1: Choose Movie Genres</CardTitle>
        <CardDescription className="text-zinc-400">
          Select the genres you are in the mood for.
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-6">
        <div className="flex flex-wrap gap-3 justify-center">
          {genres.map((genre) => {
            const isSelected = selectedGenres.includes(genre)
            return (
              <button
                key={genre}
                onClick={() => onToggleGenre(genre)}
                className={`px-4 py-2 rounded-full border transition-all duration-200 text-sm font-medium
                  ${
                    isSelected
                      ? 'bg-red-600 text-white border-red-600 shadow-md transform scale-105'
                      : 'bg-zinc-800/50 text-zinc-300 border-zinc-700 hover:bg-zinc-800 hover:text-white hover:border-zinc-600'
                  }`}
              >
                {genre}
              </button>
            )
          })}
        </div>

        <div className="flex justify-end pt-4">
          <Button
            onClick={onNext}
            disabled={selectedGenres.length === 0}
            className="w-full sm:w-auto bg-red-600 hover:bg-red-700 text-white"
          >
            Next Step
          </Button>
        </div>
      </CardContent>
    </Card>
  )
}

export default GenreSelection
