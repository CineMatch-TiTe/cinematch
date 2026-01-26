'use client'

import { Loader2 } from 'lucide-react'

interface PickingLoadingStateProps {
  isRefetching?: boolean
}

export default function PickingLoadingState({
  isRefetching = false
}: Readonly<PickingLoadingStateProps>) {
  return (
    <div className="fixed inset-0 z-50 flex flex-col items-center justify-center bg-zinc-950/90 backdrop-blur-md">
      <Loader2 className="w-10 h-10 text-white animate-spin mb-4" />
      <p className="text-zinc-400 animate-pulse">
        {isRefetching ? 'Finding more movies...' : 'Finding movies for you...'}
      </p>
    </div>
  )
}
