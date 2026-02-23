'use server'

import { redirect } from 'next/navigation'
import { createParty } from '@/server/party/party'
import { joinParty } from '@/server/member-ops/member-ops'

function isRedirectError(error: unknown) {
  return (
    typeof error === 'object' &&
    error !== null &&
    'digest' in error &&
    typeof (error as { digest: unknown }).digest === 'string' &&
    (error as { digest: string }).digest.startsWith('NEXT_REDIRECT')
  )
}

export async function createPartyInstantAction() {
  try {
    const userPrefsRes = await import('@/server/user/user').then((mod) => mod.getUserPreferences())
    const hasPreferences =
      userPrefsRes.status === 200 &&
      userPrefsRes.data?.include_genres &&
      userPrefsRes.data.include_genres.length > 0

    if (!hasPreferences) {
      return { requirePreferences: true }
    }

    const response = await createParty()

    if (response.status === 201) {
      // Since we just created it, we can get it.
      // Or maybe the response data IS the party.
      // `CreatePartyResponse` usually has `code`.
      // Let's fetch the party to be sure about the ID for redirection.

      // Actually, commonly `createParty` returns the join code.
      // Let's try to get the party details to get the ID.
      const partyRes = await import('@/server/party/party').then((mod) => mod.getParty({}))
      if (partyRes.status === 200 && partyRes.data?.id) {
        redirect(`/party-room?id=${partyRes.data.id}`)
      }
    }

    return { error: 'Failed to create party' }
  } catch (error) {
    if (isRedirectError(error)) {
      throw error
    }
    console.error('Create Party Instant Error', error)
    return { error: 'Failed to create party' }
  }
}

export async function joinPartyInstantAction(code: string) {
  if (!code) return { error: 'Code is required' }
  try {
    const response = await joinParty({ code })
    if (response.status === 200) {
      // Joined successfully. Now get party ID.
      const partyRes = await import('@/server/party/party').then((mod) => mod.getParty({}))
      if (partyRes.status === 200 && partyRes.data?.id) {
        redirect(`/party-room?id=${partyRes.data.id}`)
      }
    } else if (response.status === 404) {
      return { error: 'Party not found' }
    } else {
      return { error: 'Failed to join party' }
    }
  } catch (error) {
    if (isRedirectError(error)) {
      throw error
    }
    console.error('Join Party Instant Error', error)
    return { error: 'Failed to join party' }
  }
  return { error: 'Failed to join party' }
}

export async function getPersonalRecommendationsAction() {
  const { getRecommendations } = await import('@/server/recommendation/recommendation')
  try {
    const response = await getRecommendations({})
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
    const response = await updateTaste({ movie_id: movieId, liked: liked !== null ? liked : undefined })
    if (response.status !== 200) {
      return { error: 'Failed to update taste' }
    }
    return { success: true }
  } catch (error) {
    console.error('Update Personal Taste Error', error)
    return { error: 'Failed to update taste' }
  }
}
