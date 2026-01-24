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
    <Card className="w-full mx-auto border-zinc-800 bg-zinc-900/50 backdrop-blur-xl shadow-2xl">
      <CardHeader>
        <CardTitle className="text-zinc-100">Step 3: Just one last thing...</CardTitle>
        <CardDescription className="text-zinc-400">Tell us a bit about yourself.</CardDescription>
      </CardHeader>
      <CardContent className="space-y-6">
        <div className="flex flex-col space-y-3">
          <button
            onClick={() => onSelectStatus(true)}
            className={`p-4 rounded-xl border text-left transition-all duration-200
              ${
                isStudying === true
                  ? 'bg-red-900/20 border-red-500 ring-1 ring-red-500'
                  : 'bg-zinc-800/50 border-zinc-700 hover:bg-zinc-800'
              }`}
          >
            <div
              className={`font-semibold text-lg mb-1 ${isStudying === true ? 'text-zinc-100' : 'text-zinc-200'}`}
            >
              Studying software engineering
            </div>
            <p className="text-sm text-zinc-400">I dedicate my life to code.</p>
          </button>

          <button
            onClick={() => onSelectStatus(false)}
            className={`p-4 rounded-xl border text-left transition-all duration-200
              ${
                isStudying === false
                  ? 'bg-red-900/20 border-red-500 ring-1 ring-red-500'
                  : 'bg-zinc-800/50 border-zinc-700 hover:bg-zinc-800'
              }`}
          >
            <div
              className={`font-semibold text-lg mb-1 ${isStudying === false ? 'text-zinc-100' : 'text-zinc-200'}`}
            >
              Studying some nonsense
            </div>
            <p className="text-sm text-zinc-400">I have a life outside of code.</p>
          </button>
        </div>

        <div className="flex justify-between pt-4">
          <Button
            variant="outline"
            onClick={onBack}
            disabled={isSubmitting}
            className="bg-transparent border-zinc-700 text-zinc-300 hover:bg-zinc-800 hover:text-white"
          >
            Back
          </Button>
          <Button
            onClick={onSubmit}
            disabled={isStudying === null || isSubmitting}
            className="bg-red-600 hover:bg-red-700 text-white"
          >
            {isSubmitting ? 'Submitting...' : 'Complete & Join Party'}
          </Button>
        </div>
      </CardContent>
    </Card>
  )
}

export default StudyStatusSelection
