import React from 'react'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from '@/components/ui/card'

interface DecadeSelectionProps {
  selectedDecades: string[]
  onToggleDecade: (decade: string) => void
  onNext: () => void
  onBack: () => void
}

const DecadeSelection: React.FC<DecadeSelectionProps> = ({
  selectedDecades,
  onToggleDecade,
  onNext,
  onBack
}) => {
  const decades = ['1960s', '1970s', '1980s', '1990s', '2000s', '2010s', '2020s']

  return (
    <Card className="w-full mx-auto border-zinc-800 bg-zinc-900/50 backdrop-blur-xl shadow-2xl">
      <CardHeader>
        <CardTitle className="text-zinc-100">Step 2: Choose Decades</CardTitle>
        <CardDescription className="text-zinc-400">
          Which eras of cinema do you prefer?
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-6">
        <div className="grid grid-cols-2 md:grid-cols-3 gap-3">
          {decades.map((decade) => {
            const isSelected = selectedDecades.includes(decade)
            return (
              <button
                key={decade}
                onClick={() => onToggleDecade(decade)}
                className={`p-4 rounded-xl border text-center transition-all duration-200 font-medium
                  ${
                    isSelected
                      ? 'bg-red-600 text-white border-red-600 shadow-lg ring-2 ring-red-500/20'
                      : 'bg-zinc-800/50 text-zinc-300 border-zinc-700 hover:bg-zinc-800 hover:text-white'
                  }`}
              >
                {decade}
              </button>
            )
          })}
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
            disabled={selectedDecades.length === 0}
            className="bg-red-600 hover:bg-red-700 text-white"
          >
            Next Step
          </Button>
        </div>
      </CardContent>
    </Card>
  )
}

export default DecadeSelection
