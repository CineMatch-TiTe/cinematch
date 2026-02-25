import React from 'react'
import Image from 'next/image'
import { LogOut, Settings } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { PreferencesDialog } from '@/components/preferences/PreferencesDialog'

export function DashboardHeader() {
  return (
    <header className="relative z-10 border-b border-zinc-800 bg-zinc-900/50 backdrop-blur-md">
      <div className="container mx-auto px-4 h-16 flex items-center justify-between">
        <div className="flex items-center gap-2">
          <Image src="/Logo.png" alt="CineMatch" width={32} height={32} className="w-8 h-8" />
          <span className="font-bold text-xl tracking-tight">CineMatch</span>
        </div>
        <div className="flex items-center gap-2">
          <PreferencesDialog
            trigger={
              <Button
                variant="ghost"
                size="icon"
                className="text-zinc-400 hover:text-white hover:bg-zinc-800"
              >
                <Settings className="h-5 w-5" />
                <span className="sr-only">Settings</span>
              </Button>
            }
          />
          <form action="/api/logout" method="POST">
            <Button
              variant="ghost"
              size="icon"
              className="text-zinc-400 hover:text-red-400 hover:bg-zinc-800"
            >
              <LogOut className="h-5 w-5" />
              <span className="sr-only">Logout</span>
            </Button>
          </form>
        </div>
      </div>
    </header>
  )
}
