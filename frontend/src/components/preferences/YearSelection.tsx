import React from 'react'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from '@/components/ui/card'
import { Minus, Plus } from 'lucide-react'

const MIN_YEAR = 1950
const MAX_YEAR = Math.floor(new Date().getFullYear() / 5) * 5
const YEAR_STEP = 5
const DEFAULT_YEAR = 2000

interface YearSelectionProps {
  selectedYear: number | null
  onChangeYear: (year: number | null) => void
  onNext: () => void
  onBack: () => void
}

const YearSelection: React.FC<YearSelectionProps> = ({
  selectedYear,
  onChangeYear,
  onNext,
  onBack
}) => {
  const decrement = () => {
    const base = selectedYear ?? DEFAULT_YEAR
    onChangeYear(Math.max(MIN_YEAR, base - YEAR_STEP))
  }

  const increment = () => {
    const base = selectedYear ?? DEFAULT_YEAR
    onChangeYear(Math.min(MAX_YEAR, base + YEAR_STEP))
  }

  return (
    <Card className="w-full mx-auto border-zinc-800 bg-zinc-900/50 backdrop-blur-xl shadow-2xl">
      <CardHeader>
        <CardTitle className="text-zinc-100">Step 2: Choose Era</CardTitle>
        <CardDescription className="text-zinc-400">
          What era of cinema do you prefer?
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-6">
        <div className="flex flex-col items-center gap-4">
          <div className="flex items-center gap-6">
            <button
              onClick={decrement}
              disabled={selectedYear !== null && selectedYear <= MIN_YEAR}
              className="flex h-12 w-12 items-center justify-center rounded-xl border border-zinc-700 bg-zinc-800/50 text-zinc-300 transition-all duration-200 hover:bg-zinc-800 hover:text-white disabled:opacity-40 disabled:cursor-not-allowed"
            >
              <Minus className="h-5 w-5" />
            </button>

            <div className="min-w-[120px] text-center">
              {selectedYear !== null ? (
                <span className="text-4xl font-bold text-zinc-100">{selectedYear}</span>
              ) : (
                <span className="text-2xl font-medium text-zinc-500">Any era</span>
              )}
            </div>

            <button
              onClick={increment}
              disabled={selectedYear !== null && selectedYear >= MAX_YEAR}
              className="flex h-12 w-12 items-center justify-center rounded-xl border border-zinc-700 bg-zinc-800/50 text-zinc-300 transition-all duration-200 hover:bg-zinc-800 hover:text-white disabled:opacity-40 disabled:cursor-not-allowed"
            >
              <Plus className="h-5 w-5" />
            </button>
          </div>

          {selectedYear !== null && (
            <p className="text-sm text-zinc-400">
              {selectedYear - 5} &ndash; {selectedYear + 5}
            </p>
          )}

          {selectedYear !== null && (
            <button
              onClick={() => onChangeYear(null)}
              className="text-sm text-zinc-500 underline underline-offset-2 transition-colors hover:text-zinc-300"
            >
              Clear selection
            </button>
          )}
        </div>

        <div className="flex justify-between pt-4">
          <Button
            variant="outline"
            onClick={onBack}
            className="bg-transparent border-zinc-700 text-zinc-300 hover:bg-zinc-800 hover:text-white"
          >
            Back
          </Button>
          <Button
            onClick={onNext}
            className="bg-red-600 hover:bg-red-700 text-white"
          >
            Next Step
          </Button>
        </div>
      </CardContent>
    </Card>
  )
}

export default YearSelection
