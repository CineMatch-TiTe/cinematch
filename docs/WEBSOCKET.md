# WebSocket Protocol

## Connection

**Endpoint:** `GET /api/ws`

Requires authentication (JWT cookie). The user must be a member of a party for messages to be relevant.

```javascript
const ws = new WebSocket('ws://localhost:8085/api/ws');

ws.onmessage = (event) => {
  const message = JSON.parse(event.data);
  console.log(message.type, message);
};
```

## Architecture

The WebSocket system uses Actix actors:

1. **`WsSession`** — one per connection. Handles ping/pong keepalive and forwards incoming/outgoing messages.
2. **`WsRegistry`** — singleton actor maintaining a `HashMap<UserId, Vec<WsSession>>`. Provides `send_to_users()` for targeted broadcast.
3. **`broadcast_to_party()`** — helper that fetches party member IDs from the database, then calls `WsRegistry::send_to_users()`.

## Server Messages

All messages are JSON with a `type` field for discrimination.

| Type | When | Payload |
|------|------|---------|
| `PartyMemberJoined` | A user joins the party | `{ user_id, username }` |
| `PartyMemberLeft` | A user leaves (or is kicked) | `{ user_id }` |
| `PartyStateChanged` | Phase transitions | `{ new_state, phase_entered_at }` |
| `UpdateReadyState` | A member toggles ready | `{ user_id, is_ready }` |
| `MovieVoteUpdate` | Vote totals change | `{ movie_id, likes, dislikes }` |
| `VotingRoundStarted` | Round 2 begins | `{ voting_round }` |
| `PartyTimeoutUpdate` | Timer info for current phase | `{ timeout_type, execute_at }` |
| `NameChanged` | A user renames themselves | `{ user_id, new_name }` |
| `PartyLeaderChanged` | Leadership transferred | `{ new_leader_id }` |
| `PartyDisbanded` | Party permanently ended | `{}` |

## Broadcasting

When a state change occurs (e.g., a vote is cast), the handler calls:

```rust
broadcast_to_party(&ctx, party_id, &message, Some(exclude_user)).await;
```

- `exclude_user` (optional) prevents echoing the event back to the user who triggered it.
- Members not connected via WebSocket simply miss the message — the REST API remains the source of truth. The frontend polls/refetches as a fallback.

## Connection Lifecycle

1. Client connects → `WsSession` actor starts, registers in `WsRegistry`
2. Ping/pong keepalive runs on a timer
3. On disconnect → `WsSession` actor stops, unregisters from `WsRegistry`
4. Reconnection is handled by the frontend — there is no server-side replay of missed messages
