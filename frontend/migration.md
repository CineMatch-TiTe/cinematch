# Orval Frontend Migration Guide

When regenerating the frontend `orval` client, the backend OpenAPI spec changed several conventions. The following migration steps are necessary to update existing frontend code to consume the newly generated client correctly.

## 1. Import Paths (Tags Split)
Orval was configured to use `mode: 'tags-split'` which organizes generated endpoints by their backend OpenAPI tags into sub-modules under `src/server/`. Replace old, generic imports (often from `user/user` or `party/party`) with their new tagged locations:

### Auth
- **Old**: `user/user`
- **New**: `auth/auth`
- **Functions Moved**: `loginGuest`, `logoutUser`

### Party & Member Ops
- **Functions kept in `party/party`**: `getParty`, `createParty`
- **Moved to `member-ops/member-ops`**: `joinParty`, `leaveParty`, `getPartyMembers`
- **Moved to `leader-tools/leader-tools`**: `advancePhase`, `kickMember`, `transferLeadership`

### Ratings & Recommendations
- **Moved to `recommendation/recommendation`**: `getRecommendations`
- **Moved to `picking/picking`**: `pickMovie`
- **Moved to `voting/voting`**: `voteMovie`, `getVote`

## 2. Deprecated / Renamed Endpoints
- **`getMyParty`**: Replaced entirely by `getParty()`. Usage such as `getMyParty()` or `getParty(id)` now requires passing an empty object `{}` or `{ id: string }`.
- **`getPartyRecommendations`**: Replaced by generic `getRecommendations({ party_id: partyId })`.

## 3. Function Signatures (Parameter Structs)
Almost all endpoint bindings changed their function signature styles from **positional arguments** to **object wrappers**.
If an endpoint took an `id`, `partyId`, or boolean, it now typically expects an explicit payload or parameters object:

### Examples

**Party Fetching:**
```typescript
// Old
getParty(partyId)
getMyParty()

// New
getParty({ id: partyId })
getParty({})
```

**Actions with Targets / IDs (Leader Tools & Ops):**
```typescript
// Old
joinParty(code)
leaveParty(partyId)
kickMember(partyId, { target_user_id: memberId })
transferLeadership(partyId, { new_leader_id: memberId })
advancePhase(partyId)

// New
joinParty({ code })
leaveParty({ id: partyId })
kickMember({ id: partyId, target_user_id: memberId })
transferLeadership({ id: partyId, new_leader_id: memberId })
advancePhase({ id: partyId })
```

**Voting and Picking:**
```typescript
// Old
pickMovie(partyId, movieId, { liked })
voteMovie(partyId, movieId, { like })
getVote(partyId, { cache: 'no-store' })

// New
pickMovie({ id: partyId, movie_id: movieId, liked: liked !== null ? liked : undefined })
voteMovie({ id: partyId, movie_id: movieId, like })
getVote({ id: partyId }, { cache: 'no-store' })
```

**User Tools:**
```typescript
// Old
renameUser(userId, { new_username: rawUsername })
updateTaste(movieId, { liked })
movieGetInfo(id)

// New
renameUser({ name: rawUsername })    // userId is injected mostly from session/payload
updateTaste({ movie_id: movieId, liked: liked !== null ? liked : undefined })
movieGetInfo({ movie_id: id })
```
