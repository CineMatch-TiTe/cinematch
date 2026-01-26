'use client'

export default function PersonalPickingEmptyState() {
  return (
    <div className="fixed inset-0 z-50 flex flex-col items-center justify-center bg-zinc-950/90 backdrop-blur-md p-6 text-center">
      <h2 className="text-2xl font-bold text-white mb-2">That&apos;s all for now!</h2>
      <p className="text-zinc-400 mb-8 max-w-xs">
        We&apos;ve run out of recommendations for you. Check back later!
      </p>
      <div className="flex gap-3">{/* User can switch tabs using the bottom pill switcher */}</div>
    </div>
  )
}
