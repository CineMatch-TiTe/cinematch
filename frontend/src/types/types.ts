export type PreferenceStep = 1 | 2 | 3

export interface UserPreferences {
  genres: string[]
  isStudying: boolean | null // true for "Studying software engineering", false for "Studying some nonsense", null for unselected
}

