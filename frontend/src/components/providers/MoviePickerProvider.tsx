'use client'

import React, { createContext, useContext, useRef } from 'react'
import { MovieResponse } from '@/model/movieResponse'

interface PickerState {
    movies: MovieResponse[]
    noNewMovies: boolean
}

interface MoviePickerContextType {
    getState: (key: string) => PickerState | undefined
    setState: (key: string, state: PickerState) => void
    clearState: (key: string) => void
}

const MoviePickerContext = createContext<MoviePickerContextType | undefined>(undefined)

export function MoviePickerProvider({ children }: Readonly<{ children: React.ReactNode }>) {
    const states = useRef<Map<string, PickerState>>(new Map())

    const getState = (key: string) => {
        return states.current.get(key)
    }

    const setState = (key: string, state: PickerState) => {
        states.current.set(key, state)
    }

    const clearState = (key: string) => {
        states.current.delete(key)
    }

    const contextValue = React.useMemo(() => ({ getState, setState, clearState }), [])

    return (
        <MoviePickerContext.Provider value={contextValue}>
            {children}
        </MoviePickerContext.Provider>
    )
}

export function useMoviePickerContext() {
    const context = useContext(MoviePickerContext)
    if (context === undefined) {
        throw new Error('useMoviePickerContext must be used within a MoviePickerProvider')
    }
    return context
}
