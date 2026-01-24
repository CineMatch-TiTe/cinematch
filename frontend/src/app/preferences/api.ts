// import { customInstance } from '@/lib/orval-client'
import { UserPreferences } from '../../types/types'

export const submitPreferences = (joinCode: string, preferences: UserPreferences) => {
  // Placeholder for API call
  console.log('Submitting preferences for join code:', joinCode, preferences)

  // Example of how it would look with SWR/Orval custom instance
  // return customInstance<{ success: boolean }>(`/api/party/${joinCode}/preferences`, {
  //   method: 'POST',
  //   body: JSON.stringify(preferences),
  // });

  return new Promise<{ success: boolean }>((resolve) => {
    setTimeout(() => {
      resolve({ success: true })
    }, 1000)
  })
}
