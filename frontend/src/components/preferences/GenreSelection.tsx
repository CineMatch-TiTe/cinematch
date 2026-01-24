import React from 'react'
import { MovieGenre } from '../../types/types'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from '@/components/ui/card'

interface GenreSelectionProps {
  selectedGenres: MovieGenre[]
  onToggleGenre: (genre: MovieGenre) => void
  onNext: () => void
}

const GenreSelection: React.FC<GenreSelectionProps> = ({
  selectedGenres,
  onToggleGenre,
  onNext
}) => {
  const genres = Object.values(MovieGenre)

  return (
    <Card className="w-full mx-auto">
      <CardHeader>
        <CardTitle>Step 1: Choose Movie Genres</CardTitle>
        <CardDescription>Select the genres you are in the mood for.</CardDescription>
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
                      ? 'bg-primary text-primary-foreground border-primary shadow-md transform scale-105'
                      : 'bg-background text-foreground border-input hover:bg-accent hover:text-accent-foreground'
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
            className="w-full sm:w-auto"
          >
            Next Step
          </Button>
        </div>
      </CardContent>
    </Card>
  )
}

export default GenreSelection
