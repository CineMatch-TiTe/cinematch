'use server'

import { cookies } from 'next/headers'
import { redirect } from 'next/navigation'
import { z } from 'zod'
import { loginGuest } from '@/server/user/user'
import { joinParty, createParty } from '@/server/party/party'

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
  cookieName?: string
  cookieValue?: string
  status?: number
}

async function performGuestLogin(username: string): Promise<LoginResult> {
  const response = await loginGuest({ username })

  if (response.status === 201) {
    const setCookieHeader = response.headers.get('set-cookie')
    if (setCookieHeader) {
      const [cookiePart] = setCookieHeader.split(';')
      const [name, value] = cookiePart.split('=')
      if (name && value) {
        return { success: true, cookieName: name.trim(), cookieValue: value.trim() }
      }
    }
    console.error('[GuestLogin] No Set-Cookie header or invalid format')
    return { success: false, status: response.status }
  }

  return { success: false, status: response.status }
}

async function setSessionCookie(name: string, value: string) {
  const cookieStore = await cookies()
  cookieStore.set(name, value, {
    httpOnly: true,
    sameSite: 'lax',
    path: '/'
  })
}

export async function guestLoginAction(prevState: unknown, formData: FormData) {
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

    if (!loginResult.success || !loginResult.cookieName || !loginResult.cookieValue) {
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

    const { cookieName, cookieValue } = loginResult
    const cookieString = `${cookieName}=${cookieValue}`

    const joinResponse = await joinParty(result.data.joinCode, {
      headers: { Cookie: cookieString }
    })

    if (joinResponse.status === 200) {
      await setSessionCookie(cookieName, cookieValue)
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

  redirect(`/preferences?joinCode=${encodeURIComponent(result.data.joinCode)}`)
}

export async function createPartyAction(prevState: unknown, formData: FormData) {
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

  let joinCode = ''

  try {
    const loginResult = await performGuestLogin(result.data.username)

    if (!loginResult.success || !loginResult.cookieName || !loginResult.cookieValue) {
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

    const { cookieName, cookieValue } = loginResult
    const cookieString = `${cookieName}=${cookieValue}`

    const createResponse = await createParty({
      headers: { Cookie: cookieString }
    })

    if (createResponse.status === 201) {
      await setSessionCookie(cookieName, cookieValue)
      joinCode = createResponse.data.code
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

  if (joinCode) {
    redirect(`/preferences?joinCode=${encodeURIComponent(joinCode)}`)
  } else {
    return {
      message: 'Failed to create party (Code missing)',
      errors: null
    }
  }
}

export async function loginAction(prevState: unknown, formData: FormData) {
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

    if (!loginResult.success || !loginResult.cookieName || !loginResult.cookieValue) {
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

    const { cookieName, cookieValue } = loginResult
    await setSessionCookie(cookieName, cookieValue)
  } catch (error) {
    console.error('[Login] Error', error)
    return {
      message: 'An unexpected error occurred',
      errors: null
    }
  }

  redirect('/preferences')
}
