import { cookies } from 'next/headers'
import { redirect } from 'next/navigation'
import { logoutUser } from '@/server/user/user'

export async function GET() {
  await logoutUser()
  const cookieStore = await cookies()
  cookieStore.delete('id')

  redirect('/')
}
