# net-desync-resync: recover from a state-hash mismatch instead of ending the match

2026-10-19: online play needed to survive a desync. Before this, a hash mismatch ended the match for
everyone (`CoreEvent::Desync { tick }` → `net_fail("State desync at tick …")`). This doc is the
resync design with the evidence; every behavioural fact about the PORT cites code, and the whole
feature is port-side robustness (the retail netcode is not a reference here) — policy choices are
**[S]**, mechanics cite the existing code they reuse. Tags: [M] code file:line, [D] disc, [S]
suggested.

## 1. The pieces that already existed (all [M])

- The per-tick state hash: `Sim::net_state_hash` (nf-view/src/scene.rs:10288-10370) — FNV-1a-64
  over frame, per-slot hp/dead/armour/combat, the shared RNG stream, all 16 bodies' poses, the MP
  match/scenario/bot state and `net_world_fold` (pickups, doors, triggers, channels, destroys,
  breakables, SS, hurts). Submitted every `hash_every` (15) ticks (`Session::submit_hash`,
  session.rs:949-958; nf-view config mpnet.rs:315-319) and exchanged as `Packet::Hash` (proto.rs
  tag 5). **View-independent**: every machine folds every slot, so a slot-0 replica and an own-slot
  view produce the same hash.
- The comparison: `Session::check_desync` (session.rs:2432+) — ascending tick order, each tick
  compared once, `Event::Desync { tick, local, remote }` carries both hashes.
- The late-join handover the resync reuses: the host's unbounded input log (`run_log`,
  session.rs:494-500), `begin_late_join` (session.rs:1960+) picking a verifiable snapshot
  (`JOIN_BEGIN` proto.rs tag 11: first_tick, snapshot_tick, snapshot_hash, earlier joins), the
  chunked `JOIN_LOG` (tag 12, `JOIN_LOG_WINDOW` 16 chunks in flight, JOIN_ACK-advanced), the
  joiner's replay (`mp_late_join_catchup_joins`, nf-view/src/lib.rs:408+ — the arena reloads and
  the host's log replays headless through the real tick shape, bit-exact because the replay seeds
  the same per-slot `net_rng` (`net_slot0_setup`, lib.rs:478-481) and runs the host's recorded
  rows), `JOIN_READY` (tag 14) → the host announces `JOIN_SLOT` (tag 15) at head + 90
  (`JOIN_MARGIN_TICKS`), and the replayed scene is adopted wholesale by the app
  (`LateJoinHandover` arm, lib.rs:3990+).
- The wait rules: `ready_inputs` **never guesses** (session.rs:818+ — a missing human input stalls
  the tick, no timeout); a dropped peer's slot rides NEUTRAL (`DropPolicy::Neutral`,
  `host_slot_required`, session.rs:766-780); the only deadlines are `timeout_ms` = 8000
  (session.rs:217-219) for SILENT peers. `Sim::mp_join_spawn` is idempotent —
  `if self.mp_bonds[slot].is_some() { return false }` (scene.rs:10267-10270) — so a re-announced
  spawn for a body that never left is a no-op on every machine.

## 2. The resync

**The host is the authority** [S — it owns the only canonical input log]: a machine whose hash
disagrees with the host's resyncs; everyone else keeps playing.

- **Detection → decision (nf-net `check_desync`)**: on a mismatch the CLIENT (inside its 60 s
  window? → fatal) sets `resync_pending`, freezes (`tick_input` returns None for its own slot —
  the never-guess stall; its app shows "Resyncing…" and stops ticking) and sends the new
  `Packet::Resync { tick }` (proto.rs tag 19), retransmitting every `resend_ms`; the HOST arms a
  `resync_watch` for the offending slot and keeps playing. The mismatch (tick + both hashes) is
  written to the log on every machine that sees it (`[net] state desync at tick {tick}: local
  {local:016x} remote {remote:016x}` in the app's Desync arm, lib.rs:3974+).
- **The serve (nf-net, host `Packet::Resync` handler)**: the newest snapshot tick the peer's hash
  AGREES with (walking back from the head through both sides' hash windows; failing that the
  newest both-hashed tick), the slot marked non-contributing from now
  (`join_from[slot] = u32::MAX` → `host_slot_required` false → NEUTRAL — the exact shape a dropped
  peer rides under `DropPolicy::Neutral`), then `JOIN_BEGIN` + the run log chunks **on the live
  connection** — the same packets a dialing late joiner gets, minus the WELCOME/SLOTMAP/slot
  grant (the peer keeps its slot and its body).
- **The replay (nf-view, the existing `LateJoinHandover` arm, reused verbatim)**:
  `mp_late_join_catchup_joins` reloads the arena and replays the host's log; the replay's hash
  must equal `snapshot_hash`; `confirm_late_join` → the host's `JOIN_READY` handler re-arms the
  slot at head + 90 (`join_from[slot] = join_tick`, the announce rides `JOIN_SLOT` to every
  machine) **without a `join_history` entry** — the resyncing body never left, so a future late
  joiner's replay spawns it once, at its original join tick; the gap ticks ran NEUTRAL, which the
  log's rows carry. The machine catches up through the gap off the normal relay (its own slot
  NEUTRAL pre-announce, like a late joiner's), and the re-announced `mp_join_spawn` is a no-op
  everywhere (the body exists — the idempotency above). On the announced tick's hash the machine
  takes its own slot back (`swap_local_slot` + the per-slot `net_rng` reseed, lib.rs:4447+) and
  the "Resyncing…" note comes down (`App::net_resyncing`, cleared in the slot swap).
- **The message**: `App::net_resyncing()` + `App::NET_RESYNC_TEXT` ("Resyncing…") — the data the
  draw code renders; NOT in the Sim (the app/UI layer; the session's timers use its logical
  clock, mpnet's `epoch` precedent — the determinism rule holds).

## 3. The fallbacks (all fatal → today's lobby message, `DesyncFatal` → `net_fail`)

1. A second mismatch within 60 s of the last resync request (the client's `last_resync_ms`;
   `RESYNC_RETRY_WINDOW_MS` = 60_000, session.rs).
2. The replay's hash ≠ the host's snapshot hash (the existing `late_join_verify_failed` path →
   `net_fail("The join could not be verified…")`).
3. A resync request never served inside `timeout_ms` (8 s) — the client's deadline.
4. The host's watch unanswered inside `timeout_ms` — the peer never asked (the pre-resync
   behaviour for it); the watch also dies with a departed peer (the Neutral match runs on).
5. A second slot desyncing while one resync is in flight (the host's single watch).

## 4. Tests

- `crates/nf-view/tests/net_resync_live.rs` — the target shape: a 3-machine live loopback (host +
  2 clients, real UDP, `net_rng` via `net_slot0_setup`, every machine a full slot-0 replica), a
  mid-match `bond.pos[0] += 0.01` on one machine: it resyncs (desync on the diverged machine and
  the host only, the handover replays through the app's real catch-up, the re-announce lands, the
  re-spawns are no-ops), all three agree on every hash exchange after, no fatal, the match runs
  on. Sisters: no perturbation → nothing resyncs; a second perturbation within 60 s → fatal.
  **Needs a large-memory machine**: a 4 GB machine OOM-kills a 3-full-scene process at the
  third arena load (rc 137).
- `crates/nf-net/tests/resync.rs` — the same flow at session level on the shim (3 machines, fake
  hash chains, persistent one-bit hash corruption from the desync tick on): the handshake, the
  serve, the replay verify, the NEUTRAL gap on both sides, the re-announce, no `join_history`
  growth, and the two fallbacks. **State: `without_a_divergence_nothing_resyncs` green; the two
  resync runs complete the handshake/replay/re-announce and converge, then STALL together ~9 ticks
  before the target (host 353 = c1 = charlie of 362) — the post-rejoin input-window seam (the
  same class the late-join path's three-window comments cover, session.rs's `their_ack`/`our_ack`
  re-seeding at JOIN_READY) — the next session continues from the branch.**
- `cargo test -p nf-net` (the whole crate): 39 unit + the integration files green with the
  changes (the late-join/reconnect/lockstep/loopback suites all pass — the widened
  `Event::Desync` and the resync arming do not disturb them).

## 5. Live checks (open)

1. With `NF_NET_RATE=1`, run a real online match, corrupt one client's state (a debugger poke),
   and watch: the desync log line (tick + both hashes), the "Resyncing…" note on that machine
   only, the others never pausing more than a stall beat, and the match continuing.
2. The `net_resyncing` draw: the note should render while the diverged machine replays (seconds on
   a real link) and clear at the rejoin.
