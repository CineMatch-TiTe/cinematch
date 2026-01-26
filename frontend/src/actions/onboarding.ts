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

export async function guestLoginAction(prevState: unknown, formData: FormData) {
  const username = formData.get('username') as string
  const joinCode = formData.get('joinCode') as string

  // Validate inputs
  const result = guestLoginFormSchema.safeParse({ username, joinCode })

  if (!result.success) {
    return {
      errors: result.error.flatten().fieldErrors,
      message: 'Validation failed'
    }
  }

  try {
    const response = await loginGuest({
      username: result.data.username
    })

    if (response.status === 201) {
      const setCookieHeader = response.headers.get('set-cookie')

      if (setCookieHeader) {
        const [cookiePart] = setCookieHeader.split(';')
        const [name, value] = cookiePart.split('=')
        if (name && value) {
          // Attempt to join the party using the new session cookie
          const cookieString = `${name.trim()}=${value.trim()}`
          const joinResponse = await joinParty(result.data.joinCode, {
            headers: {
              Cookie: cookieString
            }
          })

          if (joinResponse.status === 200) {
            const cookieStore = await cookies()
            cookieStore.set(name.trim(), value.trim(), {
              httpOnly: true, // Should match what backend sent, but let's be safe
              sameSite: 'lax',
              path: '/'
            })
          } else {
            return {
              message: 'Login successful, but failed to join party. Please check the code.'
            }
          }
        }
      }
    } else {
      return {
        message: 'Login failed. Please try again.'
      }
    }
  } catch (error) {
    console.error('Login Error', error)
    return {
      message: 'An unexpected error occurred'
    }
  }

  // Redirect on success (outside try-catch to avoid catching the redirect() error which is intended behavior in Next.js)
  redirect(`/preferences?joinCode=${encodeURIComponent(result.data.joinCode)}`)
}

const createPartyFormSchema = z.object({
  username: usernameSchema
})

export async function createPartyAction(prevState: unknown, formData: FormData) {
  const username = formData.get('username') as string

  // Validate inputs
  const result = createPartyFormSchema.safeParse({ username })

  if (!result.success) {
    return {
      errors: result.error.flatten().fieldErrors,
      message: 'Validation failed'
    }
  }

  let fullPartyId = ''
  let joinCode = ''

  try {
    const loginResponse = await loginGuest({
      username: result.data.username
    })

    if (loginResponse.status === 201) {
      const setCookieHeader = loginResponse.headers.get('set-cookie')

      if (setCookieHeader) {
        const [cookiePart] = setCookieHeader.split(';')
        const [name, value] = cookiePart.split('=')

        if (name && value) {
          const cookieString = `${name.trim()}=${value.trim()}`

          // Create the party
          const createResponse = await createParty({
            headers: {
              Cookie: cookieString
            }
          })

          if (createResponse.status === 201) {
            const cookieStore = await cookies()
            cookieStore.set(name.trim(), value.trim(), {
              httpOnly: true,
              sameSite: 'lax',
              path: '/'
            })

            fullPartyId = createResponse.data.party_id
            joinCode = createResponse.data.code
          } else {
            return {
              message: 'Login successful, but failed to create party.'
            }
          }
        }
      }
    } else {
      return {
        message: 'Login failed. Please try again.'
      }
    }
  } catch (error) {
    console.error('Create Party Error', error)
    return {
      message: 'An unexpected error occurred'
    }
  }

  if (joinCode) {
    redirect(`/preferences?joinCode=${encodeURIComponent(joinCode)}`)
  }
}
