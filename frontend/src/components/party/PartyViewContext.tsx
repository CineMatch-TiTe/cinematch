'use client'

import { createContext, useContext, useState, ReactNode, useMemo } from 'react'

export type PartyViewType = 'room' | 'picking'

interface PartyViewContextType {
  activeView: PartyViewType
  setActiveView: (view: PartyViewType) => void
}

const PartyViewContext = createContext<PartyViewContextType | undefined>(undefined)

interface PartyViewProviderProps {
  children: ReactNode
  initialView?: PartyViewType
}

export function PartyViewProvider({
  children,
  initialView = 'room'
}: Readonly<PartyViewProviderProps>) {
  const [activeView, setActiveView] = useState<PartyViewType>(initialView)

  const value = useMemo(() => ({ activeView, setActiveView }), [activeView])

  return <PartyViewContext.Provider value={value}>{children}</PartyViewContext.Provider>
}

export function usePartyView() {
  const context = useContext(PartyViewContext)
  if (context === undefined) {
    throw new Error('usePartyView must be used within a PartyViewProvider')
  }
  return context
}
