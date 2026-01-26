'use server'

import { revalidateTag, revalidatePath } from 'next/cache'
import { redirect } from 'next/navigation'
import { advancePhase, kickMember, leaveParty, transferLeadership } from '@/server/party/party'

export async function leavePartyAction(partyId: string) {
  try {
    const response = await leaveParty(partyId)
    // We don't necessarily need to check response status if the error throws,
    // but Orval generated client might not throw on non-200.
    // Based on orval-client implementation, it catches errors and returns { data, status }.

    if (response.status !== 200) {
      throw new Error('Failed to leave party')
    }
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
