/**
 * Party state for API responses
 * Mirrors `PartyStateDto` in backend
 */
export type PartyState =
 | 'created'
 | 'picking'
 | 'voting'
 | 'watching'
 | 'review'
 | 'disbanded';

/**
 * Detailed party information
 */
export interface PartyResponse {
  id: string; // Uuid
  leader_id: string; // Uuid
  state: PartyState;
  created_at: string; // DateTime ISO string
  code?: string;
}

/**
 * Response when creating a new party
 */
export interface CreatePartyResponse {
  party_id: string;
  code: string;
  created_at: string;
}

/**
 * Information about a party member
 */
export interface MemberInfo {
  user_id: string;
  username: string;
  is_leader: boolean;
  is_ready: boolean;
  joined_at: string;
}

/**
 * Response with list of party members
 */
export interface PartyMembersResponse {
  members: MemberInfo[];
  count: number;
  ready_count: number;
  all_ready: boolean;
}

/**
 * Generic status response
 */
export interface StatusResponse {
  status: string; // "ok"
}

/**
 * Response after toggling ready state
 */
export interface ReadyStateResponse {
  all_ready: boolean;
}

/**
 * Response after advancing phase
 */
export interface PhaseAdvanceResponse {
  new_state: PartyState;
}

/**
 * Response after starting new round
 */
export interface NewRoundResponse {
  code: string;
}

// Request payloads (if needed, though mostly handled via args)
export interface JoinPartyRequest {
  code: string;
}
