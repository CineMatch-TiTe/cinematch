import React from 'react'

interface PageBackgroundProps {
  showImage?: boolean
  imageOpacity?: 10 | 20
}

export function PageBackground({ showImage = false, imageOpacity = 20 }: Readonly<PageBackgroundProps>) {
  return (
    <div className="fixed inset-0 z-0 pointer-events-none">
      <div className="absolute inset-0 bg-[radial-gradient(circle_at_center,var(--tw-gradient-stops))] from-zinc-800/20 via-zinc-950 to-zinc-950" />
      {showImage && (
        <div 
          className={`absolute top-0 left-0 w-full h-full bg-[url('https://images.unsplash.com/photo-1489599849927-2ee91cede3ba?q=80&w=2070&auto=format&fit=crop')] bg-cover bg-center mix-blend-overlay ${
            imageOpacity === 10 ? 'opacity-10' : 'opacity-20'
          }`} 
        />
      )}
    </div>
  )
}
