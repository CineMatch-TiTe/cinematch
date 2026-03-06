import { redirect } from 'next/navigation'
import { logoutUser } from '@/server/auth/auth'
import { cookies } from 'next/headers'

async function handleLogout() {
  // Clear the JWT cookie
  const cookieStore = await cookies()
  cookieStore.delete('jwt')

  // Also tell the backend to logout (clears the backend's id cookie)
  try {
    await logoutUser()
  } catch {
    // Ignore errors — we're clearing the session regardless
  }

  redirect('/')
}

export async function GET() {
  return handleLogout()
}

export async function POST() {
  return handleLogout()
}
