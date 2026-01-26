'use server'

import { revalidatePath } from 'next/cache'
import {
  getCurrentUser,
  getUserPreferences,
  renameUser,
  editUserPreferences
} from '@/server/user/user'
import { RenameUserRequest } from '@/model/renameUserRequest'
import { UpdateUserPreferencesRequest } from '@/model/updateUserPreferencesRequest'

export async function getCurrentUserAction() {
  try {
    const response = await getCurrentUser()
    if (response.status === 200) {
      return { data: response.data }
    }
    return { error: 'Failed to fetch user' }
  } catch (error) {
    console.error('Get User Error', error)
    return { error: 'Failed to fetch user' }
  }
}

export async function getUserPreferencesAction() {
  try {
    const response = await getUserPreferences()
    if (response.status === 200) {
      return { data: response.data }
    }
    return { error: 'Failed to fetch preferences' }
  } catch (error) {
    console.error('Get Preferences Error', error)
    return { error: 'Failed to fetch preferences' }
  }
}

export async function renameUserAction(userId: string, data: RenameUserRequest) {
  try {
    const response = await renameUser(userId, data)
    if (response.status === 200) {
      revalidatePath('/dashboard')
      revalidatePath(`/party-room/[id]`, 'page')
      // Revalidating party room might be broad but safest to ensure name updates everywhere?
      // Actually simpler to just return success and let client update for now, or revalidate path where user is.
      return { success: true }
    }
    return { error: 'Failed to rename user' }
  } catch (error) {
    console.error('Rename User Error', error)
    return { error: 'Failed to rename user' }
  }
}

export async function updateUserPreferencesAction(data: UpdateUserPreferencesRequest) {
  try {
    const response = await editUserPreferences(data)
    if (response.status === 200) {
      return { success: true, data: response.data }
    }
    return { error: 'Failed to update preferences' }
  } catch (error) {
    console.error('Update Preferences Error', error)
    return { error: 'Failed to update preferences' }
  }
}
