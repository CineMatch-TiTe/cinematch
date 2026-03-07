'use server'

import { revalidatePath } from 'next/cache'
import { redirect } from 'next/navigation'
import { advancePhase, kickMember, transferLeadership } from '@/server/leader-tools/leader-tools'
import { leaveParty, setReady } from '@/server/member-ops/member-ops'
import { getParty } from '@/server/party/party'
import { getRecommendations } from '@/server/recommendation/recommendation'
import { pickMovie } from '@/server/picking/picking'
import { voteMovie, getVote } from '@/server/voting/voting'

import { SearchFilter } from '@/model/searchFilter'

export async function leavePartyAction(partyId: string) {
  try {
    await leaveParty({ party_id: partyId })
  } catch (error) {
    console.error('Leave Party Error', error)
    return { error: 'Failed to leave party' }
  }

  redirect('/dashboard')
}

export async function kickMemberAction(partyId: string, memberId: string) {
  try {
    const response = await kickMember({ party_id: partyId, target_user_id: memberId })
    if (response.status !== 200) {
      return { error: 'Failed to kick member' }
    }
    revalidatePath(`/party-room?id=${partyId}`)
    return { success: true }
  } catch (error) {
    console.error('Kick Member Error', error)
    return { error: 'Failed to kick member' }
  }
}

export async function promoteMemberAction(partyId: string, memberId: string) {
  try {
    const response = await transferLeadership({ party_id: partyId, new_leader_id: memberId })
    if (response.status !== 200) {
      return { error: 'Failed to transfer leadership' }
    }
    revalidatePath(`/party-room?id=${partyId}`)
    return { success: true }
  } catch (error) {
    console.error('Promote Member Error', error)
    return { error: 'Failed to transfer leadership' }
  }
}

export async function advancePhaseAction(partyId: string) {
  try {
    const response = await advancePhase({ party_id: partyId })
    if (response.status !== 200) {
      return { error: 'Failed to advance phase' }
    }
    revalidatePath(`/party-room?id=${partyId}`)
    return { success: true }
  } catch (error) {
    console.error('Advance Phase Error', error)
    return { error: 'Failed to advance phase' }
  }
}

export async function getRecommendedMoviesAction(partyId: string) {
  try {
    const response = await getRecommendations({ party_id: partyId })
    if (response.status === 200) {
      return { data: response.data.recommended_movies }
    }
    return { error: 'Failed to fetch recommendations' }
  } catch (error) {
    console.error('Get Recommendations Error', error)
    return { error: 'Failed to fetch recommendations' }
  }
}

export async function pickMovieAction(partyId: string, movieId: number, liked?: boolean | null) {
  try {
    const response = await pickMovie({ party_id: partyId, movie_id: movieId, liked: liked !== null ? liked : undefined })
    if (response.status !== 200) {
      return { error: 'Failed to pick movie' }
    }
    return { success: true }
  } catch (error) {
    console.error('Pick Movie Error', error)
    return { error: 'Failed to pick movie' }
  }
}

export async function getMyPartyIdAction() {
  try {
    const response = await getParty({ party_id: undefined })
    if (response.status === 200) {
      return { id: response.data.id }
    }
    return { error: 'Failed to fetch party' }
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
  } catch (error: any) {
    // 404 means the user is legitimately not in a party
    if (error?.status === 404) {
      return { error: 'not_in_party' }
    }
    console.error('Get My Party Error', error)
    return { error: 'Failed to fetch party' }
  }
}

export async function searchMoviesAction(filter: SearchFilter, page: number = 1) {
  const { searchMovies } = await import('@/server/movie/movie')
  try {
    const response = await searchMovies(filter, { title: '', page })
    if (response.status === 200) {
      return { data: response.data.movies }
    }
    return { error: 'Failed to search movies' }
  } catch (error) {
    console.error('Search Movies Error', error)
    return { error: 'Failed to search movies' }
  }
}

export async function getUserPreferencesAction() {
  const { getUserPreferences } = await import('@/server/user/user')
  try {
    const response = await getUserPreferences()
    if (response.status === 200) {
      return { data: response.data }
    }
    return { error: 'Failed to fetch preferences' }
  } catch (error) {
    console.error('Get User Preferences Error', error)
    return { error: 'Failed to fetch preferences' }
  }
}

export async function voteMovieAction(partyId: string, movieId: number, like: boolean) {
  try {
    const response = await voteMovie({ party_id: partyId, movie_id: movieId, like })

    if (response.status !== 200) {
      console.error('Vote failed with status:', response.status, response.data)
      return { error: `Failed to vote: ${response.status}` }
    }
    return { success: true, data: response.data }
  } catch (error) {
    console.error('Vote Movie Error Details:', error)
    return { error: 'Failed to vote (Exception)' }
  }
}

export async function getPartyVotesAction(partyId: string) {
  try {
    const response = await getVote({ party_id: partyId }, { cache: 'no-store' })
    if (response.status === 200) {
      return { data: response.data }
    }
    return { error: 'Failed to fetch votes' }
  } catch (error) {
    console.error('Get Votes Error', error)
    return { error: 'Failed to fetch votes' }
  }
}

export async function getMoviesByIdsAction(movieIds: number[]) {
  const { movieGetInfo } = await import('@/server/movie/movie')
  try {
    const promises = movieIds.map((id) => movieGetInfo({ movie_id: id }))
    const responses = await Promise.all(promises)
    const movies = responses.filter((r) => r.status === 200).map((r) => r.data)

    return { data: movies }
  } catch (error) {
    console.error('Get Movies By IDs Error', error)
    return { error: 'Failed to fetch movie details' }
  }
}

export async function setReadyAction(partyId: string, isReady: boolean) {
  try {
    const response = await setReady({ party_id: partyId, is_ready: isReady })
    if (response.status === 200) {
      return { success: true }
    }
    return { error: 'Failed to set ready state' }
  } catch (error) {
    console.error('Set Ready Error', error)
    return { error: 'Failed to set ready state' }
  }
}
