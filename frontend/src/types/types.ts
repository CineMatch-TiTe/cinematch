export enum MovieGenre {
  Action = 'Action',
  Adventure = 'Adventure',
  Animation = 'Animation',
  Comedy = 'Comedy',
  Crime = 'Crime',
  Documentary = 'Documentary',
  Drama = 'Drama',
  Family = 'Family',
  Fantasy = 'Fantasy',
  History = 'History',
  Horror = 'Horror',
  Music = 'Music',
  Mystery = 'Mystery',
  Romance = 'Romance',
  ScienceFiction = 'Science Fiction',
  TVMovie = 'TV Movie',
  Thriller = 'Thriller',
  War = 'War',
  Western = 'Western'
}

export type PreferenceStep = 1 | 2 | 3

export interface UserPreferences {
  genres: MovieGenre[]
  decades: string[]
  isStudying: boolean | null // true for "Studying software engineering", false for "Studying some nonsense", null for unselected
}

export interface PreferencesFlowProps {
  joinCode: string
}
