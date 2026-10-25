# level-0a-rain-indoors-3 — the last two reds from the rain fix

Branch `s/level-0a-rain-indoors-3` (origin/s/level-0a-rain-indoors-2 merged with origin/master,
clean). Both target tests are green on this branch; the five tests recovered by the rain-2 fix
stay green; the full suite runs on this final merged tree.

## What the shifted random stream actually changed

The rain-2 report's mechanism holds: the port's faster game-true rain consumes ~12 draws/frame,
shifting every later AI/door/script decision. Neither red rested on a port-only [S] random value
in its ASSERTS — both asserts pin game rules ([M]/[D]). What shifted was each walk's
CHOREOGRAPHY: each walk had been riding one lucky stream decision that no longer happens. No
port per-frame draw-ORDER divergence was found beyond the rain-2 report's alignment
(Env_Update's one `Rand_Rand(20000)` → `UpdateDrops`; 3 FRands per `AddDrop`; no wind draws for
rain [M 0x17bc88, 0x18c278, 0x18c580]). The fixes are player-true walks; no assert was weakened.

### 1. `interactables_0700000a_live::armour_pickup_taken_by_walking_over_it` — GREEN

The walk aborted at "node 8: level_exit reached at +1017 ticks". Cause chain, probed with
`crates/nf-view/examples/probe_0a_armour_exit.rs` (a dump, not a test):

- The `level_exit` value is `Some(0x0700000A)` — the CURRENT level. Not the level's own
  LOAD_LEVEL #59 "Load Point" (next `0x0700000B`, far east side [D docs/levels/0700000a.md]) and
  no script event 0x12 fires (an `NF_EXIT_DBG` dump at the script-LoadLevel arm [M
  Script_EventHandler 0x1d94c0:189-199] printed nothing). The writer is the mission's
  `Event::LevelEnd` [M Mission case 4 → `ResetMap_LevelToLoad(LevelToEndTo)` 0x182c60:73 →
  `level_exit = Some(next)`].
- The graph's route to the armour room turns north off the z≈5.5 corridor at its node 8; the
  walker's steering wedges against the passage wall there (+1017 ticks on that leg) and parks
  Bond at (−20.23, 1.03, 9.14) — INSIDE the west office's ch-0x15 TOUCH floor plates (#11 and
  #22: x −24.27..−19.96, z 8.89..14.45, y < 1.4 [D trigger dump]). A ~17-second stand satisfies
  the mission's end condition; the sim requests the level reload mid-route.
- The plates' brief crossings are harmless: their out channel 0x15 feeds the door-unlock muxes
  (OR(ch 0x13/0x15/0x18/0x1a) [D docs/levels/0700000a.md]), and walkreal-0a's
  `west_doorway_repro` (green on this branch) walks the ring straight over them.

Fix (the player's line, no assert touched): the walk now takes the AI ring's west-office legs —
the anchors walkreal-0a's ring walks green on this branch — down the west office block, east
along the z≈5.5 corridor, then north-east onto the armour. Every leg was wall-checked clear at
body height (the probe's leg rays) before being committed. The test still asserts the game
rules: armour 50 on the walk-over [M 0x18e650 case 3; D param 2 = 50] and the sister #731
untouched.

### 2. `escort_follow_07000001_live::escort_at_walking_speed_keeps_the_games_distance_and_the_exit_opens` — GREEN

The contact's stream-shifted park at (3.41, 6.34, 6.56) (ALLY_LEAD_WAIT, path 0.0, from round 6
on) put Bond's close-through-the-nav recovery into the gallery doorway wedge: every nav leg's
straight line from (4.0-5.45, 6.65-6.9, 11.6) hits the wall band at z≈11.05 at t 0.10-0.35, and
the slide door's OPEN panel re-closes every 360 quiet ticks [M the auto-close,
Door_Update 0x13660c-0x136700 — the timer always runs while open; at 360 the proximity scan
closes it unless an actor blocks the live open-leaf sphere], cycling the panel across Bond's
west approach. The contact (whose resume band is route-relative) never un-parked; 12 rounds
exhausted.

Fix (the player's stick moves, no assert touched — the follow-band asserts on
`mission_path_dist` [M 0x15e698/0x15eda8 case 3] and the AND-plate exit condition [M
Trigger_Hit 0x1b1fc8 / Trigger_Update case 10 0x1b1a78] are unchanged):

- the stall recovery's un-wedge: a round that ended where it started jogs Bond laterally both
  ways with a little forward (the analog strafe the harness already uses for wire rides) until
  he actually moves;
- the gallery doorway's wedge signature (bond x 2.5-7.0, z 10.5-12.5, the contact > 2 south):
  walk the frame's OPEN east half — (8.4, 11.6) then (8.4, 10.2) — then re-aim through the nav.

Result: round 7's east-half walk freed Bond; the contact resumed and settled at
(−28.3, 6.3, 14.3); `level_exit 07000002` fired at tick 53311; 9533 distance-band checks, all
inside the game's bands.

### What was tried and rejected

- The nav route to the armour via an intermediate mid-room target still routed through the
  wedge (the graph's route is the same down to node 6, and the wedge is on the end leg).
- The lateral jog alone did not free the doorway wedge (the panel + wall band need the east
  half); it stays as the generic un-wedge because rounds 0-5's parks are positional, not
  doorway-specific.

## Verification

- `cargo test --offline --release -p nf-view --test interactables_0700000a_live armour_pickup
  -- --test-threads=1 --nocapture` → **ok. 1 passed** (2.12 s).
- `cargo test --offline --release -p nf-view --test escort_follow_07000001_live escort_at_walking
  -- --test-threads=1 --nocapture` → **ok. 1 passed** (25.59 s).
- The recovered-test binaries (the final tree's build), run directly (`--test-threads=1`):
  sight_frustum_live **2/2**, doublecross_transitions_live **6/6**, escort_hide_findrun_live
  **3/3**, fix_pickups_ammo_live **4/4**, hostage_killer_07000002_live **5/5**,
  fix_0a_rain_indoors_live **2/2** — all 22 recovered tests green. (The first direct pass OOM-
  killed three of them under the suite build's rustc pressure; they passed cleanly once the
  build was done.)
- **Full suite**: started 10:44 UTC on this final merged tree (daemon,
  `NF_ISO=... python3 tools/test/suite.py > suite.out`). See the Suite appendix for the state
  at the cutoff.

## What's left

- The full suite's total line: the sandbox window (to ~12:00 UTC) is shorter than the suite's
  ~13 min build + ~62 min run on this sandbox's hardware (162 binaries/755 s measured on the
  previous session's partial run). The report's Suite appendix records the partial coverage at
  the 11:40 cutoff; the next session should re-read `suite.out` (and `/home/z/suite.log` for
  the per-binary durations) before pushing further.
- **Discovered on this merged tree** (not one of the task's two reds; the rain-2 merge's suite
  never completed, so this is the first time it ran):
  1. `bathhouse_shout_mixer_live::the_bathhouse_floor_guards_shout_reaches_the_mixer_too`
     (07000002) fails — no floor guard's shout voice (ids 111..115) keys within the 4×300-tick
     window after the plate cycles. It PASSES on the urn-break tree's suite run (absent from
     that run's failure list at 529/794 binaries) and fails here reproducibly (direct run
     4.46 s, 1 passed / 1 failed).
  2. `interactables_07000041_live::the_glass_pan_shatters_under_the_real_fire` fails in the
     suite run (same session, same tree).
  Both are the same stream-shift class as the two fixed reds: the pinned expectation rides the
  shared generator's position (a voice pick, a shot window), and the game-true rain moved it.
  My diff cannot have caused either (two test files this task owns + an env-gated debug print).
  Fixing them means re-pinning stream-dependent expectations (or the per-level game-seed
  rebaseline [S]) — next-session work, with the evidence above.
- The environment's OOM kills (exit −9: net_*/mp_* and several big live binaries under memory
  pressure) are sandbox limits, not test failures — same signature as the previous sessions
  (the urn-break tree's 529-binary partial run shows the same OOM set plus the GPU-render
  pylon reds).

## Live checks for the main agent

None new: both fixes are walk choreography through the real input path; nothing visual is
claimed. (If wanted: the armour walk's new ring legs can be rendered from
`--bond` spots along (−20.5, 1.1, 16.5) → (−20.5, 1.1, 5.5) → (−17.5, 1.1, 5.5) →
(−13.46, 0.9, 7.61) to confirm the route reads as a natural player line.)

## Usage

not visible to me (this session's environment shows no token, credit, context-window or cost
figures; nothing to copy).

## Suite appendix (state at the 11:40 UTC cutoff)

- The first daemon (10:44) was killed with the previous session's surviving suite (it had been
  running the urn-break tree since 09:34 and held memory); the final-tree suite relaunched
  10:56 UTC with a proper stdout redirect.
- `build ok: 798 test binaries in 407 s` (warm build).
- At 11:37 UTC: **232 / 798 binaries run**, no total line. Average ~7.3 s/binary projects the
  remaining ~566 binaries at ~69 min — the total line would land ~12:46 UTC, past the sandbox
  expiry (~12:00).
- FAILED so far (3): bathhouse_shout_mixer_live::the_bathhouse_floor_guards_shout_reaches_the_mixer_too,
  interactables_07000041_live::the_glass_pan_shatters_under_the_real_fire (both documented
  above — stream-shift class, not from this task's diff), probe_pylon_render (the GPU-render
  environment red, expected on the no-GPU sandbox).
- CRASH ×7 (exit −9, the sandbox OOM signature, same set class as the previous sessions):
  see `suite.out` for the names (mp_*/net_* and the big walk binaries).
- The two task binaries' results inside the suite run: interactables_0700000a_live and
  escort_follow_07000001_live ran their full test sets before the cutoff — the armour and
  escort tests pass there too (only the 41-level glass red above is from the interactables
  family).
- The daemon keeps running until the sandbox dies; the next session should read `suite.out` /
  `/home/z/suite.log` first — if the sandbox persisted, the total line may already be there.
