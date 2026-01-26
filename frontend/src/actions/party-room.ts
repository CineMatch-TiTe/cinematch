'use server'

import { revalidatePath } from 'next/cache'
import { redirect } from 'next/navigation'
import { cookies } from 'next/headers'
import {
  advancePhase,
  kickMember,
  leaveParty,
  transferLeadership,
  getMyParty
} from '@/server/party/party'
import { logoutUser } from '@/server/user/user'

export async function leavePartyAction(partyId: string) {
  try {
    await leaveParty(partyId)
    await logoutUser()
    const cookieStore = await cookies()
    cookieStore.delete('id')
  } catch (error) {
    console.error('Leave Party Error', error)
    return { error: 'Failed to leave party' }
  }

  redirect('/')
}

export async function kickMemberAction(partyId: string, memberId: string) {
  try {
    const response = await kickMember(partyId, { target_user_id: memberId })
    if (response.status !== 200) {
      return { error: 'Failed to kick member' }
    }
    revalidatePath(`/party-room/${partyId}`)
    return { success: true }
  } catch (error) {
    console.error('Kick Member Error', error)
    return { error: 'Failed to kick member' }
  }
}

export async function promoteMemberAction(partyId: string, memberId: string) {
  try {
    const response = await transferLeadership(partyId, { new_leader_id: memberId })
    if (response.status !== 200) {
      return { error: 'Failed to transfer leadership' }
    }
    revalidatePath(`/party-room/${partyId}`)
    return { success: true }
  } catch (error) {
    console.error('Promote Member Error', error)
    return { error: 'Failed to transfer leadership' }
  }
}

export async function startVotingAction(partyId: string) {
  try {
    const response = await advancePhase(partyId)
    if (response.status !== 200) {
      return { error: 'Failed to start voting' }
    }
    revalidatePath(`/party-room/${partyId}`)
    return { success: true }
  } catch (error) {
    console.error('Start Voting Error', error)
    return { error: 'Failed to start voting' }
  }
}

export async function getMyPartyIdAction() {
  try {
    const response = await getMyParty()
    if (response.status === 200) {
      return { id: response.data.id }
    }
    return { error: 'Failed to fetch party' }
  } catch (error) {
    console.error('Get My Party Error', error)
    return { error: 'Failed to fetch party' }
  }
}
