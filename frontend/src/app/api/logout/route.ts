import { cookies } from 'next/headers'
import { redirect } from 'next/navigation'
import { logoutUser } from '@/server/auth/auth'

async function handleLogout() {
  await logoutUser()
  const cookieStore = await cookies()
  cookieStore.delete('id')

  redirect('/')
}

export async function GET() {
  return handleLogout()
}

export async function POST() {
  return handleLogout()
}

