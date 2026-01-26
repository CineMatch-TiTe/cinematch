'use server'

import { getGenres } from '@/server/movie/movie'

export async function getGenresAction(): Promise<string[]> {
  try {
    const response = await getGenres()

    if (response.status !== 200) {
      console.error('Failed to fetch genres:', response.status, response.data)
      return []
    }

    return response.data.genres || []
  } catch (error) {
    console.error('Server Action Error fetching genres:', error)
    return []
  }
}
