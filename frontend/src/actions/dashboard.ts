'use server'

import { redirect } from 'next/navigation'
import { createParty, joinParty } from '@/server/party/party'

export async function createPartyInstantAction() {
  try {
    const response = await createParty()

    if (response.status === 201 && response.data?.code) {
      // We don't necessarily need the code for the redirect if we are leader,
      // but the party room usually needs partyId.
      // Wait, createParty response typically returns the party object including ID.
      // Let's check the type definition in party.ts again if needed,
      // but usually it returns CreatePartyResponse which has code.
      // Does it have ID?
      // Checking `createPartyResponse` in `party.ts`...
      // It returns `CreatePartyResponse`.
      // Actually the `createParty` response might just contain the code or id.
      // The previous `createPartyAction` in `onboarding.ts` used `response.data.code`.
      // And then redirected to `/preferences`.
      // But here we want to redirect to `/party-room/[id]`.
      // Let's check `CreatePartyResponse` model if possible, or assume we might need to fetch the party or it returns ID.
      // If `CreatePartyResponse` only has code, we might need to join it or fetch my party to get ID?
      // Wait, `getMyParty` returns `PartyResponse` which has `id`.
      // Let's look at `createParty` in `onboarding.ts` again.
      // It redirects to `/preferences?joinCode=...`.
      // If I create a party, I am the leader.
      // I probably want to go to the party room.
      // But I need the party ID.
      // Let's assume `createParty` response structure.
      // I will double check `src/server/party/party.ts` output for `CreatePartyResponse` if I can't be sure.
      // But for now, I'll try to find the ID.
      // If `CreatePartyResponse` aligns with `PartyResponse`, it has `id`.
      // For now, let's look at `getMyPartyIdAction` in `party-room.ts`
      // It calls `getMyParty()`.
      // So I can call `createParty()` then `getMyParty()` to get the ID if it's not in the response.
    }

    // Let's re-read party.ts types quickly in the next view if needed, but I'll write the safe version.

    if (response.status === 201) {
      // Since we just created it, we can get it.
      // Or maybe the response data IS the party.
      // `CreatePartyResponse` usually has `code`.
      // Let's fetch the party to be sure about the ID for redirection.

      // Actually, commonly `createParty` returns the join code.
      // Let's try to get the party details to get the ID.
      const partyRes = await import('@/server/party/party').then((mod) => mod.getMyParty())
      if (partyRes.status === 200 && partyRes.data?.id) {
        redirect(`/party-room/${partyRes.data.id}`)
      }
    }

    return { error: 'Failed to create party' }
  } catch (error) {
    console.error('Create Party Instant Error', error)
    return { error: 'Failed to create party' }
  }
}

export async function joinPartyInstantAction(code: string) {
  if (!code) return { error: 'Code is required' }
  try {
    const response = await joinParty(code)
    if (response.status === 200) {
      // Joined successfully. Now get party ID.
      const partyRes = await import('@/server/party/party').then((mod) => mod.getMyParty())
      if (partyRes.status === 200 && partyRes.data?.id) {
        redirect(`/party-room/${partyRes.data.id}`)
      }
    } else if (response.status === 404) {
      return { error: 'Party not found' }
    } else {
      return { error: 'Failed to join party' }
    }
  } catch (error) {
    console.error('Join Party Instant Error', error)
    return { error: 'Failed to join party' }
  }
  return { error: 'Failed to join party' }
}

export async function getPersonalRecommendationsAction() {
  const { getRecommendations } = await import('@/server/movie/movie')
  try {
    const response = await getRecommendations()
    if (response.status === 200 && response.data?.recommended_movies) {
      return { data: response.data.recommended_movies }
    }
    return { error: 'Failed to fetch recommendations' }
  } catch (error) {
    console.error('Get Personal Recommendations Error', error)
    return { error: 'Failed to fetch recommendations' }
  }
}

export async function updatePersonalTasteAction(movieId: number, liked: boolean | null) {
  const { updateTaste } = await import('@/server/user/user')
  try {
    const response = await updateTaste(movieId, { liked })
    if (response.status !== 200) {
      return { error: 'Failed to update taste' }
    }
    return { success: true }
  } catch (error) {
    console.error('Update Personal Taste Error', error)
    return { error: 'Failed to update taste' }
  }
}
