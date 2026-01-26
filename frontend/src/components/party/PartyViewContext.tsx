'use client'

import { createContext, useContext, useState, ReactNode, useMemo } from 'react'

export type PartyViewType = 'room' | 'picking' | 'voting' | 'watching'

interface PartyViewContextType {
  activeView: PartyViewType
  setActiveView: (view: PartyViewType) => void
  partyState: string
}

const PartyViewContext = createContext<PartyViewContextType | undefined>(undefined)

interface PartyViewProviderProps {
  children: ReactNode
  initialView?: PartyViewType
  partyState?: string
}

export function PartyViewProvider({
  children,
  initialView = 'room',
  partyState = 'created'
}: Readonly<PartyViewProviderProps>) {
  const [activeView, setActiveView] = useState<PartyViewType>(initialView)

  const value = useMemo(() => ({ activeView, setActiveView, partyState }), [activeView, partyState])

  return <PartyViewContext.Provider value={value}>{children}</PartyViewContext.Provider>
}

export function usePartyView() {
  const context = useContext(PartyViewContext)
  if (context === undefined) {
    throw new Error('usePartyView must be used within a PartyViewProvider')
  }
  return context
}
