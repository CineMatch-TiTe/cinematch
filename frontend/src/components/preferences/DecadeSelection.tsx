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
    <Card className="w-full mx-auto">
      <CardHeader>
        <CardTitle>Step 2: Choose Decades</CardTitle>
        <CardDescription>Which eras of cinema do you prefer?</CardDescription>
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
                      ? 'bg-primary text-primary-foreground border-primary shadow-lg ring-2 ring-primary/20'
                      : 'bg-card text-card-foreground border-input hover:border-primary/50 hover:bg-accent'
                  }`}
              >
                {decade}
              </button>
            )
          })}
        </div>

        <div className="flex justify-between pt-4">
          <Button variant="outline" onClick={onBack}>
            Back
          </Button>
          <Button onClick={onNext} disabled={selectedDecades.length === 0}>
            Next Step
          </Button>
        </div>
      </CardContent>
    </Card>
  )
}

export default DecadeSelection
