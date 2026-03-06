import { PartyStateDto } from '@/model/partyStateDto'

// Reason for the timeout deadline
export type TimeoutReason = 'PhaseTimeout' | 'AllReady'

// Party state change with optional timeout info
export interface PartyStateChangedPayload {
  state: PartyStateDto
  deadline_at?: string | null
  timeout_reason?: TimeoutReason | null
}

// Timeout info for the current phase
export interface PartyTimeoutUpdatePayload {
  phase_entered_at?: string | null
  timeout_secs?: number | null
  deadline_at?: string | null
  reason?: TimeoutReason | null
}

export interface VotingRoundStartedPayload {
  round: number
}

export interface MovieVotesPayload {
  movie_id: number
  likes: number
  dislikes: number
}

export interface NameChangedPayload {
  user_id: string
  new_name: string
}

export interface MemberJoinedPayload {
  user_id: string
  username: string
}

export interface ReadyStateUpdatePayload {
  user_id: string
  ready: boolean
}

// MovieData based on backend MovieData struct
export interface MovieDataPayload {
  id: number
  title: string
  overview: string
  poster_path: string | null
  backdrop_path: string | null
  release_date: string | null
  vote_average: number
  vote_count: number
  genre_ids: number[]
}

// The discriminated union of all possible messages from the server
// Depending on how serde serializes enums (usually externally tagged by default),
// it looks like: { "PartyStateChanged": { "state": "voting", ... } }
export type ServerMessage =
  | { RecommendMovie: MovieDataPayload }
  | { NameChanged: NameChangedPayload }
  | { PartyLeaderChanged: string } // uudi string
  | { PartyMemberJoined: MemberJoinedPayload }
  | { PartyMemberLeft: string } // uuid string
  | { PartyStateChanged: PartyStateChangedPayload }
  | { UpdateReadyState: ReadyStateUpdatePayload }
  | 'PartyDisbanded' // This might be serialized as a string "PartyDisbanded" if it has no payload
  | { MovieVoteUpdate: MovieVotesPayload }
  | { VotingRoundStarted: VotingRoundStartedPayload }
  | { PartyTimeoutUpdate: PartyTimeoutUpdatePayload }

// Types for messages sent to the server
export interface VoteMovieClientPayload {
  movie_id: number
  vote: boolean
}

export type ClientMessage =
  | { VoteMovie: VoteMovieClientPayload }
  | { ChangeName: string }
  | { SetReadyState: boolean }
