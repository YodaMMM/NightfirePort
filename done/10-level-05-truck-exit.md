# 07000005: the truck exit and the courtyard guard

The level exit by truck now waits while a guard aware of Bond is within 30 u (ch 0x98, the game's EndLevel event 0x11 [D]). Walk the real route to the truck with real input on each difficulty and check whether the courtyard guard notices Bond there; if he always does, find what the game's route or the guard's rules (alert decay, sight, the 30 u test) do differently so a player can leave, and fix the port or document the player's way.

Reproduce on the real level first through real input, not a teleport. Compare with the decompile, fix, and test red on the old code. Run the walkreal_* binary of every level the change touches, one level per tool call. Branch from origin/master, push to `s/level-05-truck-exit`, report in docs/reports/level-05-truck-exit.md. No draw changes: name what is wrong if it is a draw issue.

---
claimed-by: sandbox1 at 2026-11-09T16:30Z

---
## Report (done, 2026-11-09T17:36Z)


---
## Report from the worker (done, 2026-11-09T17:36Z)

# level-05-truck-exit — the truck exit and the courtyard guard

Task 07000005: the level exit by truck waits while a guard aware of Bond is within 30 u (ch 0x98, the
game's EndLevel event 0x11 [D]). Walk the real route to the truck with real input on each difficulty,
check whether the courtyard guard notices Bond there; if he always does, find what the game's route or
the guard's rules (alert decay, sight, the 30 u test) do differently so a player can leave.

## Answer: the guard does not block the exit — on any difficulty. Nothing to fix.

## The mechanism (all previously evidenced, re-verified here)

- The EndLevel script `0x06000A23` (SP #320, gate ch 0x65) fires event 0x11 `[0x98, 0x98, 0x1D, 1]`
  at its tick 300; `Script_EventHandler` 0x1d94c0 case 0x11 writes ch 0x1D (the LOAD_LEVEL gate) only
  while ch 0x98 is clear, else re-writes ch 0x98 [M 0x1d94c0 case 0x11; D the script's params].
- ch 0x98 is `Drone_InitComms` 0x139520's chatter gate, written EVERY frame: 1 while any living
  ENABLED drone at alert ≥ 1.0 stands within 30 u of its opponent, not team 2; ch 0x97 is the
  one-shot latch of the same condition [M 0x139520:161-201, :198-201 — decompile re-read this
  session: `switch_channels[0x97] = 1` when `bVar3`, then `switch_channels[0x98] = bVar3`].
- The decay law: `NDrone2_ControlSTANDARD` 0x1490f0:65-79 clamps alert to 1.0 first and decays only
  while alert < 1.0, at 1/(FRAME_RATE·50) per tick, floored at +0x500 [M decompile re-read]. A guard
  clamped at 1.0 never decays — the wait holds until he is disabled, dead, moved, or never was aware.
  The port's decay (drone.rs `if dr.alert < 1.0 && dr.alert > dr.alert_floor { dr.alert -= 1.0 / (60.0 * 50.0) }`
  after the same clamp) matches [M].
- The gate-guard spawner #268 (mode 2, group 2, start ch 0x22, kill ch 0x43, max 2) is gated on
  switch_channels[0x22] (the gatehouse plates #21/#22/#23 OR mux #25, raised mid-ride by the truck),
  refuses a spawn whose spot is visible, and its guards wait in WaitSwitch for ch 0x22 [M
  DroneSpawner_Control 0x137e68, NDrone2_DSTATE_Global 0x15b830 → WaitSwitch 0x15b960; D
  docs/systems/triggers.md:113]. The kill channel ch 0x43 is the wall-hug plate #301 (x 23.9-25.9,
  z 70.8-72.2 [D triggers.md:98]) — off the truck route (the ride passes x ≈ 29.4 at that z), so the
  guards are not removed on this route; they don't need to be.

## The real-input walk, every difficulty (the new test)

`crates/nf-view/tests/truck_exit_guard_live.rs` —
`the_truck_exit_works_on_every_difficulty_the_courtyard_guard_never_blocks_it`: the walkreal route
from the level start with REAL INPUT ONLY (no teleport after the load; the same route the committed
`walkreal_07000005_live::walk_to_the_gatehouse_and_ride_the_truck` drives): cold start #26 → the
gatehouse porch (98.6, 26.0, 106.5) → the ch-0x5C porch plate → the hand-off cut-scene → the crouched
ride on the truck bed → the park. Run once per difficulty through the front-end's launch path
`load_with_carry_difficulty(..., 1|2|3|4)`.

Results — identical on difficulties 1, 2, 3 and 4 (exit tick 5687 each, deterministic):

- The guard chain completes: ch 0x22 rises mid-ride, the post touch #318 raises ch 0x72, the park
  raises ch 0x70, mux #316 raises ch 0x65, the EndLevel script runs, and `level_exit = 0x07000006`.
- **ch 0x98 is 0 on every one of the ~5487 ticks** (asserted), and ch 0x97 never latches: no guard is
  ever aware within 30 u on the real route. The exit is never blocked.
- At the park, four gate guards stand near the truck, none aware:
  - the two spawner-spawned guards at their post (28.340/32.064, 37.0, 44.861) [D the post], skins
    0x05000030/0x0500002e, state 0x04 (unaware), ENABLED, **alert 0.000 (floor 0.000)**, 7.1/7.3 u
    from crouched Bond on the bed — within 30 u, but never alert ≥ 1.0, so the 30 u test has no
    aware input;
  - the two pre-placed WaitSwitch guards sitting DISABLED at (27.578/32.435, 37.0, 49.657/49.859),
    state 0x01, alert 0.000.

## The game's own ground truth agrees [L ee_snapshot.bin]

The ride-time EE snapshot (file offset = EE address; parsed this session):

- switch_channels = {0x45, 0x73, 0x97}; **ch 0x98 = 0**, ch 0x22 = 0, ch 0x1D = 0; the 0x97/0x98
  time cells both hold 0x6919.
- 12 drones in the list at 0x298e14. The two post-area guards sit at (27.57765, 37.0, 49.6566) and
  (32.43511, 37.0, 49.8593) — **the port's disabled pair, to two decimal places** — with flags bit
  0x100 CLEAR (disabled — a disabled living team-1 guard cannot raise the chatter [M :167-170]) and
  alert 0.000.
- The four enabled guards at (61.3, 27.2, 99.3)/(64.2, 26.8, 93.9)/(150.2, 16.9, 100.8)/(62.1, 34.6,
  104.1) hold Bond (the cold start, per their +0x1a0 distances 42.7-46.6) as opponent — all past the
  30 u test; alert 0.000 each. Seven interior guards are disabled, four of them parked at alert
  1.000 = their floor, all > 90 u away.
- So in the real game the chatter gate is 0 at the snapshot and the post pair is disabled/unaware —
  the same picture the port produces on the walked route.
- The game's own truck-run trace ends with ch 0x1D [D tests/traces/05-truck.trace chain order: 0x72
  (−317) → 0x70+0x65 (−300) → 0x73 (−273) → 0x1D (0)]: the real game ends the level with the guard
  chain live, exactly as the port does.

The one [L] datum the port cannot reproduce mechanically: that run's ch 0x97 = 1 — the chatter latched
at some earlier moment of that session (the snapshot's run cannot be replayed here; the nearest
enabled guard at the snapshot sits 42 u out, so the latch predates it). The port's walk latches 0x97
only if an aware guard is within 30 u — per the decompile's law [M :176-178]. Nothing in the port's
route produces the condition; the game's snapshot does not show one either at its moment.

## Verdict

- The task's conditional ("if he always does [notice Bond]") is **false**: on the real route with
  real input, on difficulties 1-4, the courtyard guard never becomes aware of Bond (alert 0.000
  throughout, ch 0x98 never set) — the exit works everywhere, and the game's own snapshot/trace
  agree. **No fix needed; there is no player's way to document because no way is needed.**
- The wait mechanism itself is already correct and pinned red/green in
  `crates/nf-view/tests/level_end_live.rs::route_a_truck_exit_does_not_end_the_level_with_a_guard_on_bond`
  (ch 0x98 on → event 0x11 takes the 0x98 arm → no load), with the alert-decay law matching
  [M 0x1490f0:65-79]. No draw-side involvement anywhere on this path.

## Verification

- New: `cargo test -p nf-view --test truck_exit_guard_live` — 1 passed (the four-difficulty walk,
  ~21 s; prints the per-difficulty guard story).
- Re-run committed: `walkreal_07000005_live::walk_to_the_gatehouse_and_ride_the_truck` — 1 passed
  (the walk ends the level into 07000006 at tick 5687, unchanged).
- Suite: `cargo test --offline --release -j2 -p nf-game -p nf-formats -p nf-audio -- --test-threads=1`
  — all green except the 3 `disc_golden` tests (bigf golden manifests: `bigf_header_fields_behave_as_documented`,
  `bigf_tables_match_the_golden_manifest`, `directory_tree_matches_the_golden_manifest`), the documented
  slim-ISO allowance.
- The mechanism pins, whole binaries: `level_end_live` 7/7, `gate_guard_truck_live` 2/2,
  `walkreal_07000005_live` **16 passed / 0 failed / 1 ignored** (72.7 s).
- The full nf-view suite (all ~60 binaries, `--test-threads=1`) did not fit this window — the
  `cargo test -p nf-view` link of every binary outran the sandbox time budget (a background attempt
  was killed between tool calls, per the sandbox's rule; the foreground re-run covered the binaries
  above). No `crates/*/src/` file changed: `git diff origin/master --stat` = the new test file +
  docs only, so no other level's walkreal is touched by code; the 05 walkreal binary ran (above).

## Live checks for the main agent

None required. One optional confirmation, if a debugger window is ever open on a real ride:
- `dump 0x26fc90+0x97 1` and `+0x98` in the last second before the truck exit's ch 0x1D — expect
  0x97 to have latched only if some guard saw Bond earlier in that run; 0x98 must read 0 at the exit
  tick (else the run would not exit).
- The spawned post guard's alert byte: `drone+0x50C` (f32) while Bond crouches on the bed 7 u away —
  expect < 1.0 (the port pins 0.000; any game value ≥ 1.0 for ≥ 300 ticks would contradict the trace's
  exit and would mean the game's sight never feeds that guard — worth one look only if the exit ever
  hangs in play).

## What's left

Nothing for this task's question. If the driver wants the 0x97 mystery chased (which guard latched it
in the snapshot's session), that needs a fresh live trace of a full real ride with the per-frame
channel log (`tools/trace/` pads) — not reproducible headlessly.

## Usage

Not visible to me: this worker session's environment shows no token, cost or context-window figures
(anywhere I can read). The sandbox clock (`date -u`) is the only meter I have; session start read
16:38:28 UTC, this report written at ~17:35 UTC.
