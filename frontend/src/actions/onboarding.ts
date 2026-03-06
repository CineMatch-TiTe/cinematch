'use server'

import { z } from 'zod'
import { loginGuest } from '@/server/auth/auth'
import { createParty } from '@/server/party/party'
import { joinParty } from '@/server/member-ops/member-ops'

const usernameSchema = z
  .string()
  .trim()
  .min(3, { message: 'Username must be at least 3 characters' })
  .max(32, { message: 'Username must be at most 32 characters' })
  .regex(/^[a-zA-Z0-9_ -]+$/, {
    message: 'Username can only contain letters, numbers, spaces, hyphens, and underscores'
  })

const joinCodeSchema = z
  .string()
  .trim()
  .min(4, { message: 'Join code must be at least 4 characters' })
  .max(12, { message: 'Join code too long' })

const guestLoginFormSchema = z.object({
  username: usernameSchema,
  joinCode: joinCodeSchema
})

const createPartyFormSchema = z.object({
  username: usernameSchema
})

const loginFormSchema = z.object({
  username: usernameSchema
})

type LoginResult = {
  success: boolean
  jwt?: string
  expiresAt?: number
  status?: number
}

async function performGuestLogin(username: string): Promise<LoginResult> {
  const response = await loginGuest({ username })

  if (response.status === 201) {
    const data = response.data
    if (data.jwt && data.token_expires_at) {
      return {
        success: true,
        jwt: data.jwt,
        expiresAt: data.token_expires_at,
      }
    }
    console.error('[GuestLogin] No JWT in response')
    return { success: false, status: response.status }
  }

  return { success: false, status: response.status }
}

export type OnboardingActionResult = {
  errors?: Record<string, string[] | undefined> | null
  message?: string
  auth?: {
    jwt: string
    expiresAt: number
  }
  redirectTo?: string
}

export async function guestLoginAction(prevState: unknown, formData: FormData): Promise<OnboardingActionResult> {
  const username = formData.get('username') as string
  const joinCode = formData.get('joinCode') as string

  const result = guestLoginFormSchema.safeParse({ username, joinCode })

  if (!result.success) {
    const formatted = z.treeifyError(result.error)
    const fieldErrors = {
      username: formatted.properties?.username?.errors,
      joinCode: formatted.properties?.joinCode?.errors
    }
    return {
      errors: fieldErrors,
      message: 'Validation failed'
    }
  }

  try {
    const loginResult = await performGuestLogin(result.data.username)

    if (!loginResult.success || !loginResult.jwt || !loginResult.expiresAt) {
      console.error('[GuestLogin] Login failed', loginResult.status)
      if (loginResult.status === 409) {
        return {
          message: 'Validation failed',
          errors: {
            username: ['Username is not available or you are already logged in.'],
            joinCode: undefined
          }
        }
      }
      return {
        message: 'Login failed. Please try again.',
        errors: null
      }
    }

    const { jwt, expiresAt } = loginResult

    // Use the JWT to join the party
    const joinResponse = await joinParty(
      { code: result.data.joinCode },
      { headers: { Authorization: `Bearer ${jwt}` } }
    )

    if (joinResponse.status === 200) {
      return {
        auth: { jwt, expiresAt },
        redirectTo: `/preferences?joinCode=${encodeURIComponent(result.data.joinCode)}`,
      }
    } else {
      console.error('[GuestLogin] Join failed', joinResponse.status)
      return {
        message: 'Login successful, but failed to join party. Please check the code.',
        errors: null
      }
    }
  } catch (error) {
    console.error('[GuestLogin] Error', error)
    return {
      message: 'An unexpected error occurred',
      errors: null
    }
  }
}

export async function createPartyAction(prevState: unknown, formData: FormData): Promise<OnboardingActionResult> {
  const username = formData.get('username') as string

  const result = createPartyFormSchema.safeParse({ username })

  if (!result.success) {
    const formatted = z.treeifyError(result.error)
    return {
      errors: {
        username: formatted.properties?.username?.errors
      },
      message: 'Validation failed'
    }
  }

  try {
    const loginResult = await performGuestLogin(result.data.username)

    if (!loginResult.success || !loginResult.jwt || !loginResult.expiresAt) {
      if (loginResult.status === 409) {
        return {
          message: 'Validation failed',
          errors: {
            username: ['Username is not available or you are already logged in.']
          }
        }
      }
      return {
        message: 'Login failed. Please try again.',
        errors: null
      }
    }

    const { jwt, expiresAt } = loginResult

    const createResponse = await createParty({
      headers: { Authorization: `Bearer ${jwt}` }
    })

    if (createResponse.status === 201) {
      const joinCode = createResponse.data.code
      return {
        auth: { jwt, expiresAt },
        redirectTo: `/preferences?joinCode=${encodeURIComponent(joinCode)}`,
      }
    } else {
      return {
        message: 'Login successful, but failed to create party.',
        errors: null
      }
    }
  } catch (error) {
    console.error('[CreateParty] Error', error)
    return {
      message: 'An unexpected error occurred',
      errors: null
    }
  }
}

export async function loginAction(prevState: unknown, formData: FormData): Promise<OnboardingActionResult> {
  const username = formData.get('username') as string

  const result = loginFormSchema.safeParse({ username })

  if (!result.success) {
    const formatted = z.treeifyError(result.error)
    return {
      errors: {
        username: formatted.properties?.username?.errors
      },
      message: 'Validation failed'
    }
  }

  try {
    const loginResult = await performGuestLogin(result.data.username)

    if (!loginResult.success || !loginResult.jwt || !loginResult.expiresAt) {
      if (loginResult.status === 409) {
        return {
          message: 'Validation failed',
          errors: {
            username: ['Username is not available or you are already logged in.']
          }
        }
      }
      return {
        message: 'Login failed. Please try again.',
        errors: null
      }
    }

    return {
      auth: { jwt: loginResult.jwt, expiresAt: loginResult.expiresAt },
      redirectTo: '/preferences',
    }
  } catch (error) {
    console.error('[Login] Error', error)
    return {
      message: 'An unexpected error occurred',
      errors: null
    }
  }
}
