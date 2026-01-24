import React from 'react'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from '@/components/ui/card'

interface StudyStatusSelectionProps {
  isStudying: boolean | null
  onSelectStatus: (status: boolean) => void
  onSubmit: () => void
  onBack: () => void
  isSubmitting: boolean
}

const StudyStatusSelection: React.FC<StudyStatusSelectionProps> = ({
  isStudying,
  onSelectStatus,
  onSubmit,
  onBack,
  isSubmitting
}) => {
  return (
    <Card className="w-full mx-auto">
      <CardHeader>
        <CardTitle>Step 3: Just one last thing...</CardTitle>
        <CardDescription>Tell us a bit about yourself.</CardDescription>
      </CardHeader>
      <CardContent className="space-y-6">
        <div className="flex flex-col space-y-3">
          <button
            onClick={() => onSelectStatus(true)}
            className={`p-4 rounded-xl border text-left transition-all duration-200
              ${
                isStudying === true
                  ? 'bg-primary/10 border-primary ring-1 ring-primary'
                  : 'bg-card border-input hover:bg-accent'
              }`}
          >
            <div className="font-semibold text-lg mb-1">Studying software engineering</div>
            <p className="text-sm text-muted-foreground">I dedicate my life to code.</p>
          </button>

          <button
            onClick={() => onSelectStatus(false)}
            className={`p-4 rounded-xl border text-left transition-all duration-200
              ${
                isStudying === false
                  ? 'bg-primary/10 border-primary ring-1 ring-primary'
                  : 'bg-card border-input hover:bg-accent'
              }`}
          >
            <div className="font-semibold text-lg mb-1">Studying some nonsense</div>
            <p className="text-sm text-muted-foreground">I have a life outside of code.</p>
          </button>
        </div>

        <div className="flex justify-between pt-4">
          <Button variant="outline" onClick={onBack} disabled={isSubmitting}>
            Back
          </Button>
          <Button onClick={onSubmit} disabled={isStudying === null || isSubmitting}>
            {isSubmitting ? 'Submitting...' : 'Complete & Join Party'}
          </Button>
        </div>
      </CardContent>
    </Card>
  )
}

export default StudyStatusSelection
