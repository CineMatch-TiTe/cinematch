API Usage Guide
===============

Overview of the workflow from guest user creation through movie selection and voting.

Base URL
--------

```
http://localhost:8085/api
```

Authentication
--------------

All endpoints (except `POST /user/login/guest`) require cookie-based authentication. The `id` cookie is set upon successful login and MUST be included in subsequent requests.

Workflow
--------

### 1. User Creation (Guest)

**POST** `/user/login/guest`

Create a temporary guest user account. Optionally provide a username (3-32 characters).

```bash
# With custom username
curl -X POST http://localhost:8080/api/user/login/guest \
  -H "Content-Type: application/json" \
  -d '{"username": "Alice"}' \
  -c cookies.txt

# Auto-generated username
curl -X POST http://localhost:8080/api/user/login/guest \
  -H "Content-Type: application/json" \
  -d '{}' \
  -c cookies.txt
```

**Response:**
```json
{
  "user_id": "550e8400-e29b-41d4-a716-446655440000",
  "username": "Alice"
}
```

The `id` cookie is automatically set and stored in `cookies.txt`. Use `-b cookies.txt` for subsequent requests.

---

### 2. Create or Join a Party

#### Option A: Create a Party

**POST** `/party`

Create a new party. You become the party leader.

```bash
curl -X POST http://localhost:8080/api/party \
  -b cookies.txt \
  -H "Content-Type: application/json"
```

**Response:**
```json
{
  "party_id": "660e8400-e29b-41d4-a716-446655440001",
  "code": "ABCD",
  "created_at": "2026-01-26T12:00:00Z"
}
```

Share the 4-character `code` with others to join.

#### Option B: Join Existing Party

**POST** `/party/join/{code}`

Join a party using the 4-character code.

```bash
curl -X POST http://localhost:8080/api/party/join/ABCD \
  -b cookies.txt \
  -H "Content-Type: application/json"
```

**Response:**
```json
{
  "party_id": "660e8400-e29b-41d4-a716-446655440001",
  "code": "ABCD",
  "created_at": "2026-01-26T12:00:00Z"
}
```

---

### 3. Get Party Details

**GET** `/party/{party_id}`

Get current party state, members, and phase.

```bash
curl -X GET http://localhost:8080/api/party/660e8400-e29b-41d4-a716-446655440001 \
  -b cookies.txt
```

**Response:**
```json
{
  "party_id": "660e8400-e29b-41d4-a716-446655440001",
  "code": "ABCD",
  "state": "Created",
  "party_leader_id": "550e8400-e29b-41d4-a716-446655440000",
  "phase_entered_at": "2026-01-26T12:00:00Z",
  "voting_round": 1,
  "members": [...]
}
```

**Party States:**
- `Created` - Initial state, waiting for members to be ready
- `Picking` - Members selecting movies
- `Voting` - Voting on selected movies
- `Watching` - Final movie selected
- `Review` - After watching, can start new round
- `Disbanded` - Party ended

---

### 4. Connect to WebSocket (Real-time Updates)

**GET** `/ws`

Connect to WebSocket for real-time party updates. Must be authenticated and in a party.

```javascript
// Browser example
const ws = new WebSocket('ws://localhost:8080/api/ws');

ws.onmessage = (event) => {
  const message = JSON.parse(event.data);
  console.log('Update:', message);
};
```

**WebSocket Messages:**
- `PartyMemberJoined` - New member joined
- `PartyMemberLeft` - Member left
- `PartyStateChanged` - Phase changed
- `UpdateReadyState` - Member ready state changed
- `MovieVoteUpdate` - Vote counts updated
- `VotingRoundStarted` - Round 2 started
- `PartyTimeoutUpdate` - Phase timeout info
- `NameChanged` - User renamed
- `PartyLeaderChanged` - Leadership transferred
- `PartyDisbanded` - Party ended

---

### 5. Set User Preferences (Optional)

**PUT** `/user/like/{movie_id}`

Like or unlike a movie to improve recommendations.

```bash
# Like a movie
curl -X PUT http://localhost:8080/api/user/like/12345 \
  -b cookies.txt \
  -H "Content-Type: application/json" \
  -d '{"liked": true}'

# Unlike a movie
curl -X PUT http://localhost:8080/api/user/like/12345 \
  -b cookies.txt \
  -H "Content-Type: application/json" \
  -d '{"liked": false}'
```

**GET** `/movie/recommendations`

Get personalized movie recommendations based on taste.

```bash
curl -X GET http://localhost:8080/api/movie/recommendations \
  -b cookies.txt
```

---

### 6. Picking Phase

Once all members are ready, the party automatically advances to `Picking` phase.

#### Search for Movies

**GET** `/movie/search?query={query}`

Search for movies by title.

```bash
curl -X GET "http://localhost:8080/api/movie/search?query=inception" \
  -b cookies.txt
```

#### Pick a Movie

**POST** `/party/{party_id}/pick/{movie_id}`

Add a movie to the party's selection. Each member can pick multiple movies.

```bash
curl -X POST http://localhost:8080/api/party/660e8400-e29b-41d4-a716-446655440001/pick/12345 \
  -b cookies.txt \
  -H "Content-Type: application/json"
```

#### Get All Picks

**GET** `/party/{party_id}/picks`

View all movies picked by party members.

```bash
curl -X GET http://localhost:8080/api/party/660e8400-e29b-41d4-a716-446655440001/picks \
  -b cookies.txt
```

#### Delete a Pick

**DELETE** `/party/{party_id}/pick/{movie_id}`

Remove one of your picks.

```bash
curl -X DELETE http://localhost:8080/api/party/660e8400-e29b-41d4-a716-446655440001/pick/12345 \
  -b cookies.txt
```

#### Set Ready

**POST** `/party/{party_id}/ready`

Mark yourself as ready. When all members are ready, party advances to `Voting` phase.

```bash
curl -X POST http://localhost:8080/api/party/660e8400-e29b-41d4-a716-446655440001/ready \
  -b cookies.txt \
  -H "Content-Type: application/json" \
  -d '{"is_ready": true}'
```

**Response:**
```json
{
  "all_ready": true  // true when all members are ready
}
```

---

### 7. Voting Phase

When all members are ready, party automatically advances to `Voting` phase.

#### Get Voting Ballot

**GET** `/party/{party_id}/vote`

Get the list of movies to vote on and current vote totals.

```bash
curl -X GET http://localhost:8080/api/party/660e8400-e29b-41d4-a716-446655440001/vote \
  -b cookies.txt
```

**Response:**
```json
{
  "movie_ids": [12345, 67890, 11111],
  "voting_round": 1,
  "can_vote": true,
  "vote_totals": {
    "12345": {"likes": 2, "dislikes": 1},
    "67890": {"likes": 1, "dislikes": 0},
    "11111": {"likes": 0, "dislikes": 2}
  }
}
```

#### Vote on a Movie

**POST** `/party/{party_id}/vote/{movie_id}`

Vote like (true) or dislike (false) on a movie.

```bash
# Like a movie
curl -X POST http://localhost:8080/api/party/660e8400-e29b-41d4-a716-446655440001/vote/12345 \
  -b cookies.txt \
  -H "Content-Type: application/json" \
  -d '{"vote": true}'

# Dislike a movie
curl -X POST http://localhost:8080/api/party/660e8400-e29b-41d4-a716-446655440001/vote/12345 \
  -b cookies.txt \
  -H "Content-Type: application/json" \
  -d '{"vote": false}'
```

**Response:**
```json
{
  "likes": 3,
  "dislikes": 1
}
```

#### Voting Rounds

- **Round 1:** Vote on all picked movies
- **Round 2:** Automatically starts when voting ends. Top 3 movies by net score (likes - dislikes) advance to round 2.
- After round 2, party advances to `Watching` phase with the winning movie.

---

### 8. Leader Actions

#### Advance Phase (Leader Only)

**POST** `/party/{party_id}/advance`

Force advance to next phase (skip waiting for all ready/votes).

```bash
curl -X POST http://localhost:8080/api/party/660e8400-e29b-41d4-a716-446655440001/advance \
  -b cookies.txt \
  -H "Content-Type: application/json"
```

#### Kick Member (Leader Only)

**POST** `/party/{party_id}/kick`

Remove a member from the party.

```bash
curl -X POST http://localhost:8080/api/party/660e8400-e29b-41d4-a716-446655440001/kick \
  -b cookies.txt \
  -H "Content-Type: application/json" \
  -d '{"target_user_id": "770e8400-e29b-41d4-a716-446655440002"}'
```

#### Transfer Leadership (Leader Only)

**POST** `/party/{party_id}/transfer-leadership`

Transfer party leadership to another member.

```bash
curl -X POST http://localhost:8080/api/party/660e8400-e29b-41d4-a716-446655440001/transfer-leadership \
  -b cookies.txt \
  -H "Content-Type: application/json" \
  -d '{"new_leader_id": "770e8400-e29b-41d4-a716-446655440002"}'
```

#### Disband Party (Leader Only)

**POST** `/party/{party_id}/disband`

End the party permanently.

```bash
curl -X POST http://localhost:8080/api/party/660e8400-e29b-41d4-a716-446655440001/disband \
  -b cookies.txt \
  -H "Content-Type: application/json"
```

---

### 9. User Management

#### Get Current User

**GET** `/user`

Get your user profile.

```bash
curl -X GET http://localhost:8080/api/user \
  -b cookies.txt
```

#### Rename User

**PATCH** `/user/rename/{user_id}`

Change your username (3-32 characters).

```bash
curl -X PATCH http://localhost:8080/api/user/rename/550e8400-e29b-41d4-a716-446655440000 \
  -b cookies.txt \
  -H "Content-Type: application/json" \
  -d '{"new_username": "AliceUpdated"}'
```

#### Leave Party

**POST** `/party/{party_id}/leave`

Leave the current party.

```bash
curl -X POST http://localhost:8080/api/party/660e8400-e29b-41d4-a716-446655440001/leave \
  -b cookies.txt \
  -H "Content-Type: application/json"
```

#### Logout

**POST** `/user/logout`

Clear authentication cookie.

```bash
curl -X POST http://localhost:8080/api/user/logout \
  -b cookies.txt
```

---

## Complete Example Flow

```bash
# 1. Create guest user
curl -X POST http://localhost:8080/api/user/login/guest \
  -H "Content-Type: application/json" \
  -d '{"username": "Alice"}' \
  -c cookies.txt

# 2. Create party
curl -X POST http://localhost:8080/api/party \
  -b cookies.txt \
  -H "Content-Type: application/json"

# 3. Get party details
curl -X GET http://localhost:8080/api/party/{party_id} \
  -b cookies.txt

# 4. Set ready (advances to Picking)
curl -X POST http://localhost:8080/api/party/{party_id}/ready \
  -b cookies.txt \
  -H "Content-Type: application/json" \
  -d '{"is_ready": true}'

# 5. Search and pick movies
curl -X GET "http://localhost:8080/api/movie/search?query=matrix" \
  -b cookies.txt

curl -X POST http://localhost:8080/api/party/{party_id}/pick/603 \
  -b cookies.txt

# 6. Set ready again (advances to Voting)
curl -X POST http://localhost:8080/api/party/{party_id}/ready \
  -b cookies.txt \
  -H "Content-Type: application/json" \
  -d '{"is_ready": true}'

# 7. Get voting ballot
curl -X GET http://localhost:8080/api/party/{party_id}/vote \
  -b cookies.txt

# 8. Vote on movies available in the ballot
curl -X POST http://localhost:8080/api/party/{party_id}/vote/603 \
  -b cookies.txt \
  -H "Content-Type: application/json" \
  -d '{"vote": true}'

# 8. After voting completes, party advances to Watching phase
```

---

## WebSocket Real-time Updates

Connect to `/api/ws` to receive real-time updates:

```javascript
const ws = new WebSocket('ws://localhost:8080/api/ws');

ws.onmessage = (event) => {
  const msg = JSON.parse(event.data);
  
  switch (msg[0]) {  // ServerMessage is an enum
    case 'PartyMemberJoined':
      console.log('New member:', msg[1]);
      break;
    case 'PartyStateChanged':
      console.log('New state:', msg[1]);
      break;
    case 'MovieVoteUpdate':
      console.log('Votes updated:', msg[1]);
      break;
    // ... handle other message types
  }
};
```

---

## Error Responses

All errors follow this format:

```json
{
  "error": "Error message description"
}
```

**Common Status Codes:**
- `200` - Success
- `201` - Created
- `400` - Bad Request (invalid input)
- `401` - Unauthorized (not logged in)
- `403` - Forbidden (not authorized for action)
- `404` - Not Found
- `406` - Not Acceptable (e.g., not in a party for WebSocket)
- `409` - Conflict (e.g., already logged in)
- `500` - Internal Server Error

---

## API Documentation

Interactive API documentation available at:
- Swagger UI: `http://localhost:8085/swagger-ui/`
- ReDoc: `http://localhost:8085/redoc`
- Scalar: `http://localhost:8085/scalar`
- RapiDoc: `http://localhost:8085/rapidoc`
