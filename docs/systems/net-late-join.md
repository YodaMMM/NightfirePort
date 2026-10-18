# net-late-join — joining an ongoing online match

Status: implemented (session protocol + app surface + tests). Evidence tags: [M] code (function +
address), [D] disc, [L] EE snapshot, [S] suggested / port-only design. **Online play is a port
feature — the PS2 game has no late join** (a HELLO after GO was refused before this task), so every
wire choice here is [S]; the game facts it leans on are cited.

## 1. What this is

The user (2026-10-18): "a way to join ongoing matches". The online match is deterministic lockstep
(`docs/systems/net-lockstep.md`, `net-match-host.md`): same seed + settings on every machine, only
per-tick inputs exchanged, a per-tick state hash detects desyncs. Late join exploits exactly that:

1. The **joiner** HELLOs a host whose match has GO'd.
2. The **host** assigns a free slot and serves the recorded **run log** — the per-tick inputs every
   tick actually ran with, since tick 0 — plus the newest **verifiable snapshot**: a tick the app
   reported a state hash for (in practice the live head).
3. The joiner loads the arena from the handover settings + seed and **re-simulates the log headless
   (fast-forward)**, then compares its hash at the snapshot tick with the host's. Mismatch → the
   joiner refuses ITSELF: it leaves with `LeaveReason::JoinVerifyFailed` and the host reports
   `Event::ClientLeft { slot, reason: JoinVerifyFailed }` — the clear reason on the wire.
4. On a verified replay the joiner confirms (`confirm_late_join`); the host picks the **join tick**
   = its live head + 90 ticks (1.5 s), and announces `JOIN_SLOT { slot, join_tick }` to every peer.
5. At the join tick every machine **spawns the joiner's agent** (§3) and the slot flips human: the
   joiner's inputs start there, before it the slot rides the game's own disconnected-pad NEUTRAL
   (`psiInput_PollDevices` 0x10ca78 writes buttons 0 / analog 0x7f for a pad not in state 99 [M]) —
   never a guess, the lockstep stall still covers a late input.

Max players: the session's 4 slots ARE the game's MP limit — humans occupy slots 0..4 and
`MP_Start` 0x183a50 inits humans into slots 0..player_count-1, bots into 4..4+bot_count [M];
a HELLO with no free slot is refused `Full`. Teams: the joiner takes the slot the settings assigned
it (`MpSettings.players[slot].team`, the 700v2 team layout) — the spawn pick is team-scoped
(`MP_GetSpawnPoint` scans its team's range [M 0x185098:85-99]).

## 2. The wire (5 new packets, `crates/nf-net/src/proto.rs`)

| type | name | direction | payload |
|------|------|-----------|---------|
| 11 | JOIN_BEGIN | host → joiner | first_tick u32, snapshot_tick u32, snapshot_hash u64 |
| 12 | JOIN_LOG | host → joiner | first_tick u32, tick_count u8 (≤ 12), chunk_index u16, chunk_count u16, tick_count × 4 × 24 B inputs |
| 13 | JOIN_ACK | joiner → host | chunk_index u16 — chunks ≤ index held |
| 14 | JOIN_READY | joiner → host | — (the replay verified; retransmitted until JOIN_SLOT answers) |
| 15 | JOIN_SLOT | host → all | slot u8, join_tick u32 |

The transfer is windowed (16 chunks in flight) and re-sent on the `resend_ms` cadence until the
acks advance, so loss cannot lose a tick; the log chunk is 12 ticks × 96 B = 1152 B — inside
`MAX_PACKET`. The host suppresses the live INPUT relay to a joiner while its transfer is in
flight; on handover the joiner's ack base is seeded at snapshot+1 (the host seeds its own view of
the joiner at the snapshot too — the joiner produces no inputs before its join tick, so its acks
need the carrier: its own slot rides truthful NEUTRAL, the agent does not exist yet).

The **run log** is recorded by the app: `Session::record_run_tick(tick, [Input; 4])` right after
`ready_inputs` returned `Some` (host-only; ~96 B/tick ≈ 6.9 MB for a 20-minute match). It is the
exact transcript bit-exact by construction — what ran is what replays.

## 3. The spawn (the game's own MP create shape)

At the announced tick, EVERY machine runs `Sim::mp_join_spawn(slot)` (`crates/nf-view/src/scene.rs`):

- `MP_GetSpawnPoint(team, NULL)` before the create — the shared-stream draw is part of the
  deterministic tick, identical on every peer [M 0x183a50:120-123];
- the create is `Player_Init(slot, pos, facing, …)`'s shape — `Player_Init` 0x1910f8 refuses ONLY
  an occupied slot (`glb_players[slot] != 0`) and otherwise CREATES the agent, so a free slot
  mid-match is the game's own path; the port's create is `Bond::stand_at` (the load's spawn loop,
  lib.rs) + `set_slot` [M];
- `MP_ReSpawn` 0x185368 is NOT the path for a new joiner: all three of its callers
  (`Player_HandleDeath` 0x190bb0, `BOT_respawn` 0x124e38, `MP_RestartScenario` 0x183fc0) pass an
  EXISTING obj — it re-positions, never creates.

A slot that already holds a body (a dropped peer's agent — `DropPolicy::Neutral` keeps it alive on
NEUTRAL) is a **RECONNECT**: no spawn, the real input simply resumes at the join tick — the same
shape as the pad state machine re-initialising on replug (`ps2_handle_pad` 0x10c738 [M]).

## 4. Session gating

`Session` grows `join_from: [u32; 4]` — the tick from which a slot is human. `ready_inputs`,
`missing_slots` and the host's broadcast materialisation (`slot_input_for_send`) all treat a slot
as required only when `slot_human[s] && tick >= join_from[s]`; otherwise NEUTRAL. The joiner's OWN
slot rides the same rule (it produces no inputs before the announced tick — its acks ride the
truthful NEUTRAL carrier). Nothing else changes for pre-match joins: `join_from` is 0 for every
match-start human.

## 5. The app surface (`crates/nf-view`)

- `NetCore::record_run_tick` (host), `confirm_late_join`, `late_join_verify_failed`.
- `CoreEvent::LateJoinStarted { slot }` (host: serving), `LateJoinHandover { slot, settings,
  snapshot_tick, snapshot_hash, log }` (joiner: replay now), `LateJoinSlot { slot, join_tick }`
  (all: arm the slot).
- `nf_view::mp_late_join_catchup(iso, settings, log, replay_ticks) -> (Scene, hash)` — the
  headless fast-forward: the same `load_mp_carry` load, then the net tick's driver shape
  (`step_net_remotes` + the local `Bond::tick_ctx` + `set_slot`) per logged tick, returning the
  state hash. Wall-clock measurement belongs to the caller (the determinism rule keeps timers out
  of the Sim).
- The app hooks (lib.rs): the host records every run tick; the joiner replays on `LateJoinHandover`
  and confirms or refuses; on `LateJoinSlot` every machine arms `net_join_pending` and the net tick
  spawns at the announced tick (the joiner's view adopts the slot via `swap_local_slot`).

## 6. Tests

`crates/nf-net/tests/late_join.rs` (shim, lossy: 5‰ drop + 30 ms latency + 20 ms jitter, logical
ms): the transfer is bit-exact (the replay hash chain reproduces the snapshot hash), the join tick
is announced identically to all three and the post-join hashes agree with no desync; a full match
(4 agents [M MP_Start 0x183a50]) refuses a late HELLO with `Full`; a failed verification leaves
`JoinVerifyFailed` — the host reports the clear reason and the slot stays free for a retry.

`crates/nf-view/tests/net_join_live.rs` (real 127.0.0.1 UDP, three full arena sims, serial): host +
client play 300 honest ticks (the host records the transcript), a third dials, its
`mp_late_join_catchup` reproduces the snapshot hash exactly, and after the join all three sims'
per-tick hashes agree to the end with the joined agent's body at the same spawn on every machine;
the sister test corrupts one log input — the replay hash diverges, the joiner refuses itself and
the host reports `ClientLeft { JoinVerifyFailed }` with the roster untouched.

Measured catch-up (the sandbox, release build): 301 ticks in 1194 ms ≈ **251 ticks/s** headless —
a 20-minute match limit (72 000 ticks) back-fills in ~286 s of wall time, during which the match
keeps running (the joiner catches up live through the relay after the snapshot).

## 7. Live checks for the main agent

None required — the PS2 has no late join to observe and the tests are headless. The visible work
(the joining page, the notice on the host's lobby, the new agent popping in) is the draw code's;
`CoreEvent::LateJoinStarted` / `LateJoinHandover` / `LateJoinSlot` are its cues.
