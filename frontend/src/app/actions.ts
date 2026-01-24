'use server'

import { cookies } from 'next/headers'
import { redirect } from 'next/navigation'
import { z } from 'zod'
import { loginGuest } from '@/server/user/user'

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
      // Forward 'set-cookie' if present in headers, though usually the `fetch` in Server Component environment
      // doesn't automatically forward cookies to the client unless we explicitly set them.
      // Orval returns `headers`. We need to extract the cookie.

      // Note: Orval's generated client with our custom instance returns `headers` as a Headers object.
      const setCookieHeader = response.headers.get('set-cookie')

      if (setCookieHeader) {
        // Simple parsing to forward the cookie.
        // set-cookie can contain multiple cookies but usually we just want the session token.
        // Next.js `cookies().set` is the way to set it.
        // However, parsing `Set-Cookie` string to `cookies().set(...)` arguments can be non-trivial if complex.
        // For simple session, we can try to extract name and value.

        // Allow basic forwarding.
        // A more robust app might split by ';' and extract key=value.
        const [cookiePart] = setCookieHeader.split(';')
        const [name, value] = cookiePart.split('=')
        if (name && value) {
          const cookieStore = await cookies()
          cookieStore.set(name.trim(), value.trim(), {
            httpOnly: true, // Should match what backend sent, but let's be safe
            sameSite: 'lax',
            path: '/'
          })
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
