'use server'

import { editUserPreferences } from '@/server/user/user'
import { UpdateUserPreferencesRequest } from '@/model'

export async function submitUserPreferencesAction(data: UpdateUserPreferencesRequest) {
  try {
    const response = await editUserPreferences(data)

    // Check if the response status indicates an error (not 2xx)
    // Orval response usually follows structure { data, status }
    if (response.status >= 400) {
      console.error('API Error Details:', JSON.stringify(response.data, null, 2))
      throw new Error(`API Error: ${response.status} - ${JSON.stringify(response.data)}`)
    }

    return { success: true, data: response.data }
  } catch (error) {
    console.error('Server Action Error:', error)
    throw error // Re-throw to be caught by the component
  }
}
