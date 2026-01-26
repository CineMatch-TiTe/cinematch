'use client'

import React from 'react'
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle
} from '@/components/ui/alert-dialog'

interface ActionConfirmationDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  title: string
  description?: string
  confirmText?: string
  cancelText?: string
  onConfirm: () => void
}

export function ActionConfirmationDialog({
  open,
  onOpenChange,
  title,
  description,
  confirmText = 'Continue',
  cancelText = 'Cancel',
  onConfirm
}: Readonly<ActionConfirmationDialogProps>) {
  return (
    <AlertDialog open={open} onOpenChange={onOpenChange}>
      <AlertDialogContent className="bg-zinc-900 border-zinc-800 text-white">
        <AlertDialogHeader>
          <AlertDialogTitle className="text-white">{title}</AlertDialogTitle>
          {description && (
            <AlertDialogDescription className="text-zinc-400">{description}</AlertDialogDescription>
          )}
        </AlertDialogHeader>
        <AlertDialogFooter>
          <AlertDialogCancel className="bg-zinc-800 border-zinc-700 text-zinc-300 hover:bg-zinc-700 hover:text-white">
            {cancelText}
          </AlertDialogCancel>
          <AlertDialogAction
            onClick={onConfirm}
            className={'bg-red-600 text-white hover:bg-red-700'}
          >
            {confirmText}
          </AlertDialogAction>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  )
}
