'use server'

import { revalidatePath } from 'next/cache'
import { redirect } from 'next/navigation'
import { cookies } from 'next/headers'
import {
  advancePhase,
  kickMember,
  leaveParty,
  transferLeadership,
  getMyParty,
  pickMovie,
  voteMovie,
  getVote
} from '@/server/party/party'
import { getRecommendations } from '@/server/movie/movie'
import { logoutUser } from '@/server/user/user'
import { SearchFilter } from '@/model/searchFilter'

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

export async function advancePhaseAction(partyId: string) {
  try {
    const response = await advancePhase(partyId)
    if (response.status !== 200) {
      return { error: 'Failed to advance phase' }
    }
    revalidatePath(`/party-room/${partyId}`)
    return { success: true }
  } catch (error) {
    console.error('Advance Phase Error', error)
    return { error: 'Failed to advance phase' }
  }
}

export async function getRecommendedMoviesAction() {
  try {
    const response = await getRecommendations()
    if (response.status === 200) {
      return { data: response.data.recommended_movies }
    }
    return { error: 'Failed to fetch recommendations' }
  } catch (error) {
    console.error('Get Recommendations Error', error)
    return { error: 'Failed to fetch recommendations' }
  }
}

export async function pickMovieAction(partyId: string, movieId: number) {
  try {
    const response = await pickMovie(partyId, movieId)
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
    const response = await voteMovie(partyId, movieId, { like })

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
    const response = await getVote(partyId, { cache: 'no-store' })
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
    const promises = movieIds.map((id) => movieGetInfo(id))
    const responses = await Promise.all(promises)
    const movies = responses.filter((r) => r.status === 200).map((r) => r.data)

    return { data: movies }
  } catch (error) {
    console.error('Get Movies By IDs Error', error)
    return { error: 'Failed to fetch movie details' }
  }
}
