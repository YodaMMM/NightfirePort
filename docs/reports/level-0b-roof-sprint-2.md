# level-0b-roof-sprint-2 — report

Branch `s/level-0b-roof-sprint-2` = `origin/s/los-mask-join` (5e36d06ba) + this task's commits.
**Status: NOT finished — the Agent and 00 Agent real-damage roof walks are still red.** The window's
stop line (19:31 UTC) arrived mid route-tuning; the next window continues from this branch.

## What's done

1. **Next-step 1 of blocked/10-level-0b-roof-sprint.md is ANSWERED — the crossing exists.** The
   predecessor's probe had the wrong yaw; fixed (yaw 0 = north, the walk's `atan2(dx, dz)`
   convention) and re-run at Bond's standing height (capsule centre 58.07 on the 57.03 deck
   [L the death run (2.26,58.11,40.99)]): **the body crosses the z ≈ 41.0 face line and the take
   fires at every x 0.6–3.4**, from a standstill push, no jump — TAKEN at z 40.68–41.04 in 6–144
   ticks; only x 3.8 slides past east (ends (5.78, 58.07, 49.59), no take). The take fires from the
   DECK south of the face — no wall contact, no sill step. **The port's collision is not the
   blocker; no live check is needed for this question.** (crates/nf-view/tests/probe_0b_take.rs,
   the v2 sweep.)
2. **The real failure is a margin failure, not a wall or a law failure.** With `NF_PICKUPLOG=1` the
   real walk logs exactly one parachute touch test — at Bond (2.074, 58.110, 40.666): dist 1.768
   vs r 1.791, **los true** — and then `[take] tick 15938 weapon 0x47 kind 4 → Some(Label)`: the
   take DID fire, ch 0x3F raised, ~100 ticks after Bond first pressed the face, and the finisher
   (−5.2 from #4) landed 7 ticks later. The touch law works as ported [M Pickup_Update
   0x18e148:202-220 + Pickup_Handler 0x18e650:330-333]. Bond arrives at the face with **5.2 hp**
   because (a) the dome-floor GT pair #174/#175 costs 19.2 before the roof and (b) #4 parks ON the
   sprint line at (2.6, 58.0, 32.7) — the last 8 u cost 36.4 (3 × 11.2 + 2.8 + 2.8) [L the
   0b-stall trace added to the deck sprint: hp per 30 ticks from the re-place to the face].
3. **The plate E-hold now fires from any approach — a test-side game-law fix.** The E pick is the
   head + view-dir sphere [M Player_Activate 0x1a5170:100-108] and fires on the **press edge only**
   [M 0x1a5460:396-403]; the old stance loop held use across all six stances (one edge, stance 1
   only) and the east-dash arrival yaw missed it. The stances now face the plate first and give
   each stance its own fresh press (walkreal_0700000b_live.rs, the E-hold block). Verified: the
   walks pass the plate assert with the west-crossing approach that previously failed it.
4. **The dome-floor turret pair: measured lanes.** Head-on lane (the proven route): 6 × 3.2 on the
   approach (frames 12761-12804), re-place at 80.8. West crossing at x −7.0: approach only 6.4
   (2 hits) — but the return then runs under barrels pre-trained east: re-place 49.2 (head-on
   return) / 36.4 (west return). **The head-on pair is the best measured combination and stays.**
   The measurements are in the comments at the approach/return blocks.
5. **The deck leg: the reroutes measured, the WIP line kept.** The west band hug (target (−11.5,
   30), then (−10.0, 24)) held 87.2 hp through the #6/#7/#8 band (they all park at x ≥ 8.5
   [D the diag]) — but the deck's NW structure (x −11..−14, z 24-28) slides every shallow NE
   heading west along its face into the corner, and the re-steer crawls under fire (three stuck
   runs: (-13.73, 27.80), (-12.99, 27.05), died (-8.15, 22.13) crawling) [L the 0b-stall traces].
   A dxz-based two-phase switch also let Bond overshoot the waypoint and walk BACK into #4's cone
   (yaw 1.26 → 5.99, hp 34.0 → 14.4 standing). The straight WIP line to the slot is the best
   measured completion and is what ships on the branch; the deck leg is red again in the final
   build because the plate-stance fix (item 3) reshuffled the shared RNG stream upstream of the
   deck (the walk now dies at tick 15741 at (−0.28, 58.11, 30.64), 26 hits — no parachute take
   this build).
6. No source (crates/) changes were made or needed — every fix this window is in the level's own
   test file + the probe. No draw changes. No damage/health lowered, no pins loosened, no ignores
   added.

## How it was verified

- Every number above is from a run of the real level through the real input path
  (`walkreal_0700000b_live walk_fresh_spawn_to_the_mission_end_real_damage_agent --nocapture`),
  the `NF_PICKUPLOG=1` take-test/take lines ([M] citations inline in the port), the `[0b-stall]`
  per-30-ticks trace in the deck sprint, or the fresh-load probe sweeps (labelled as such). The
  walk binary was rebuilt after each change; each run is deterministic on the fixed 60 Hz sim.
- The full suite did NOT run this window (the stop line arrived first). The only changed crates
  file is none; the changed test files are `walkreal_0700000b_live.rs` and `probe_0b_take.rs`
  (this level only).

## What's left (the next window's plan)

1. The deck leg needs a lane that both (a) keeps 20+ u on #6/#7/#8 through their z 20-28 band
   (the west hug did this — 87.2 hp at z 18) and (b) rounds the NW structure (x −11..−14,
   z 24-28) without a shallow NE heading. Probe the structure's east face with the fresh-load
   sweep pattern (it answers in one run), then chain: west band hug → a steep east cut south of
   the structure → the take zone x 1.8-2.2. The margin target: the face arrival must be ≥ 45 hp
   (the #4 pass costs ~16.8 wide, the wall+finisher ~38.8 [L]).
2. The re-steer pathology (the body sliding off the facing after a heading change under pain) is
   the thing to watch: switch phases only with dxz comfortably large and never steer backward.
3. Then the 00 Agent sister (pain ×~1.25 — the margin work must land both walks), the d1 walk,
   and the whole `walkreal_0700000b_live` binary; then the full suite before any merge.
4. The dome lane: if the deck margin still falls short, re-visit the turret pair (the west
   crossing saves 12.8 on the approach but the return needs its own answer — e.g. route the
   return to re-cross the pair's plane at the FAR west point after the barrels settle north).

## Live checks for the main agent

None required this window — the one question the route work turned on (a body crossing of the
housing's south face line z ≈ 41.0) is answered from the port's own collision with the fixed
probe: the crossing exists at every x 0.6-3.4 and the take fires from the deck. If the next
window wants game confirmation anyway: a PCSX2 run walking Bond from the deck at (2.2, ~40.0)
due north (no fire, `god` if needed) — expected: the body crosses z 41.0 and the parachute take
message fires around z 40.7, matching the probe.

## Usage

not visible to me — this session's environment shows no token, credit or context-window figures
(no usage display was available anywhere in the interface I worked from).
