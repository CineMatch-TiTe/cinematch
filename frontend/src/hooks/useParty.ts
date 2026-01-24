import useSWR, { SWRConfiguration } from 'swr';
import {
  CreatePartyResponse,
  NewRoundResponse,
  PartyMembersResponse,
  PartyResponse,
  PhaseAdvanceResponse,
  ReadyStateResponse,
  StatusResponse,
} from '../types/party';

const API_BASE = `${process.env.NEXT_PUBLIC_API_BASE}/api/party`;

// Generic fetcher for SWR
async function fetcher<T>(url: string): Promise<T> {
  const res = await fetch(url);
  if (!res.ok) {
    const error = await res.json().catch(() => ({}));
    throw new Error(error.error || 'An error occurred while fetching the data.');
  }
  return res.json();
}

// Generic helper for POST requests
async function postRequest<T>(url: string, body?: unknown): Promise<T> {
  const res = await fetch(url, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    body: body ? JSON.stringify(body) : undefined,
  });

  if (!res.ok) {
    const error = await res.json().catch(() => ({}));
    throw new Error(error.error || 'API request failed');
  }

  return res.json();
}

/**
 * Hook to fetch party details
 */
export function useParty(partyId: string | null, config?: SWRConfiguration) {
  const { data, error, isLoading, mutate } = useSWR<PartyResponse>(
    partyId ? `${API_BASE}/${partyId}` : null,
    fetcher,
    config
  );

  return {
    party: data,
    isLoading,
    isError: error,
    mutate,
  };
}

/**
 * Hook to fetch party members
 */
export function usePartyMembers(
  partyId: string | null,
  config?: SWRConfiguration
) {
  const { data, error, isLoading, mutate } = useSWR<PartyMembersResponse>(
    partyId ? `${API_BASE}/${partyId}/members` : null,
    fetcher,
    config
  );

  return {
    members: data?.members || [],
    count: data?.count || 0,
    readyCount: data?.ready_count || 0,
    allReady: data?.all_ready || false,
    isLoading,
    isError: error,
    mutate,
  };
}

// ============================================================================
// Party Actions (Mutations)
// ============================================================================

export const partyApi = {
  create: () => postRequest<CreatePartyResponse>(API_BASE),

  join: (code: string) => postRequest<PartyResponse>(`${API_BASE}/join/${code}`),

  leave: (partyId: string) =>
    postRequest<StatusResponse>(`${API_BASE}/${partyId}/leave`),

  kickMember: (partyId: string, targetUserId: string) =>
    postRequest<StatusResponse>(`${API_BASE}/${partyId}/kick`, {
      target_user_id: targetUserId,
    }),

  transferLeadership: (partyId: string, newLeaderId: string) =>
    postRequest<StatusResponse>(`${API_BASE}/${partyId}/transfer-leadership`, {
      new_leader_id: newLeaderId,
    }),

  toggleReady: (partyId: string) =>
    postRequest<ReadyStateResponse>(`${API_BASE}/${partyId}/ready`),

  advancePhase: (partyId: string) =>
    postRequest<PhaseAdvanceResponse>(`${API_BASE}/${partyId}/advance`),

  startNewRound: (partyId: string) =>
    postRequest<NewRoundResponse>(`${API_BASE}/${partyId}/new-round`),

  disband: (partyId: string) =>
    postRequest<StatusResponse>(`${API_BASE}/${partyId}/disband`),
};
