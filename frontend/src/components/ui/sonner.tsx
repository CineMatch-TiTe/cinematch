'use client'

import {
  CircleCheckIcon,
  InfoIcon,
  Loader2Icon,
  OctagonXIcon,
  TriangleAlertIcon
} from 'lucide-react'
import { useTheme } from 'next-themes'
import { Toaster as Sonner, type ToasterProps } from 'sonner'

const Toaster = ({ ...props }: ToasterProps) => {
  return (
    <Sonner
      theme="dark"
      className="toaster group"
      toastOptions={{
        classNames: {
          toast:
            'group toast group-[.toaster]:bg-zinc-950 group-[.toaster]:text-zinc-100 group-[.toaster]:border-zinc-800 group-[.toaster]:shadow-lg',
          description: 'group-[.toast]:text-zinc-400',
          actionButton: 'group-[.toast]:bg-red-600 group-[.toast]:text-white',
          cancelButton: 'group-[.toast]:bg-zinc-800 group-[.toast]:text-zinc-400',
          error: 'group-[.toaster]:text-red-400 group-[.toaster]:border-red-900/50',
          success: 'group-[.toaster]:text-green-400 group-[.toaster]:border-green-900/50',
          warning: 'group-[.toaster]:text-yellow-400 group-[.toaster]:border-yellow-900/50',
          info: 'group-[.toaster]:text-blue-400 group-[.toaster]:border-blue-900/50'
        }
      }}
      icons={{
        success: <CircleCheckIcon className="size-4 text-green-500" />,
        info: <InfoIcon className="size-4 text-blue-500" />,
        warning: <TriangleAlertIcon className="size-4 text-yellow-500" />,
        error: <OctagonXIcon className="size-4 text-red-500" />,
        loading: <Loader2Icon className="size-4 animate-spin text-zinc-500" />
      }}
      {...props}
    />
  )
}

export { Toaster }
