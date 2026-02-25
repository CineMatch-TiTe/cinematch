import { clsx, type ClassValue } from 'clsx'
import { twMerge } from 'tailwind-merge'

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs))
}

/**
 * Prefetches the images at the given URLs by creating Image objects in the background.
 * This helps the browser cache the images before they are needed by the UI.
 */
export function prefetchImages(urls: (string | null | undefined)[]) {
  if (typeof window === 'undefined') return

  urls.forEach((url) => {
    if (url) {
      const img = new Image()
      img.src = url
    }
  })
}
