# los-mask-repins-12-2 — report

Branch `s/los-mask-repins-12-2` = origin/s/los-mask-repins-12 `0e33a261a` (origin/master
`0e1371451` was already merged there) + work commits. Task: the 07000012 Operative real-damage
walk green under the game's guard sight, un-ignored.

## What's done

**`walkreal_07000012_live::walk_spawn_to_the_shaft_exit_real_damage_operative` is green and
un-ignored.** The whole binary runs 4 passed / 0 failed / 0 ignored.

**The wedge was the test walker's own nudge left, not the route's geometry and not drone #2.**

- The predecessor's ledger reproduced exactly (deterministic): death at t7167, 57 hits for
  100.0, every hit from drone #2 (skin 0x5000008, weapon 0x19) at (−9.1, −4.1, −21.0) d 20.8
  hp 12 — never damaged itself. `NF_DMG_TRACE` showed drone #2's first hit at tick 3649 and
  Bond's position (9.3, 1.1, −12.8) — the door-543 west stand — for EVERY one of the remaining
  3667 ticks: the "through door plc 543" crossing leg never walked. Drone #2 was a drip on a
  Bond who could not move, not a fight the route was losing.
- `NF_WALKTRACE` (a new env-gated per-tick leg dump in `walker_b2.rs`, diagnostic only) caught
  the mechanism: the crossing leg's tick 0 carries `sidestep −753` and it counts DOWN from
  there. The stuck ladder's "nudge left" sets `sidestep = −40` and the decay ran
  `self.sidestep -= 1` unconditionally — a LEFT nudge grows forever, and the walker strafes
  left at forward 0.4 for the rest of the walk. `walker.rs` documented this exact bug and
  carries the opt-in fix (`sidestep_decays`, "a left one (−60 / −40) grows forever, so Bond
  strafes left through every later leg"); `walker_b2.rs` never got the mirror. The predecessor's
  d1 run only reached the stand because the eternal strafe slid it south into the door-hole band
  by accident — and every later leg was strafe-crippled.
- The route's second wedge sat behind the first: with the nudge bounded, the approach leg to the
  west stand pins Bond at (10.62, −12.85) — the corridor's west wall is SOLID north of the
  doorway hole (hole z −13.86..−15.15 [D the static collision tris]) — and the stuck ladder's
  door answer no-ops on the already-open leaf (a no-op `use_door` still resets the stuck
  counter), so the ladder never escalates to a sidestep that would help; the leg ran out its
  6000-tick cap [L the run]. The route now walks the geometry the wall admits: south along the
  corridor to the hole band (10.55, −14.40) FIRST, then through to the west stand.

**The fixes (all opt-in or d1-gated; no damage, health, armour or ammo writes anywhere; no
top-ups; no teleports after the load):**

1. `crates/nf-view/tests/common/walker_b2.rs` — `pub sidestep_decays: bool` (walker.rs's flag
   mirrored; the decay line becomes `self.sidestep -= if self.sidestep_decays {
   self.sidestep.signum() } else { 1 }`). Default OFF: every walk tuned on the old ladder keeps
   the old behaviour byte-identical; the 07000012 d1 walk is the only setter. Plus the
   `NF_WALKTRACE` env-gated per-tick leg diagnostic in the same file's trace style.
2. `crates/nf-view/tests/common/tower2.rs` — inside the existing `DOOR_543_WEST_STAND` gate
   (d1 only, set only by this walk): the two approach legs before the west stand.
3. `crates/nf-view/tests/walkreal_07000012_live.rs` — the Operative test un-ignored with the
   mechanism in its doc comment; `w.sidestep_decays = true` set at the difficulty-1 gate.

**The green run's ledger (the fights the level's own machinery won):** #7/#6/#4/#8/#3 as
before; then `through door plc 543: returned fire on drone #1 (killed true) at tick 3595` and
`through door plc 543: returned fire on drone #2 (killed true) at tick 3754` — both DURING the
crossing leg: with the strafe carry gone the walk actually moves, Bond enters the south hall's
sight band, and the armed head-hunting + unseen-cover response drops the lower-floor guard the
ledger worried about. Walked out in 7017 ticks, stuck 0, falls 0, health 96.5 (3.5 taken all
run); the real 0700004A exit load lands health 88.1 armour 50 and completes both objectives.

## How it was verified

Per-binary, `--test-threads=1`, whole binaries (the full-suite runner could not run — see
below). Every number below is from this branch's tree:

| binary | result |
|---|---|
| walkreal_07000012_live (whole) | **4 passed / 0 failed / 0 ignored** (the Operative green) |
| interactables_07000012_live | 12 / 0 |
| walkreal_07000013_live (the tower2 co-user) | 3 / 0 / 1 ignored (its own pre-existing ledger) |
| walkreal_07000011_live | 3 / 0 / 1 ignored |
| walkreal_07000011_route_live | 1 / 0 |
| walkreal_07000014_live | 7 / 0 / 3 ignored |
| walkreal_0700000d_live | 1 / 0 / 2 ignored |
| walkreal_0700000c_live | 1 / 0 / 3 ignored (the ledgered 0c repins, untouched here) |
| walkreal_0700000c_barrelrow_live | 1 / 0 |
| walkreal_0700000b_live | 9 / 0 |
| walkreal_0700000a_live | 13 / 0 / 2 ignored |
| walkreal_07000009_live | 9 / 0 |
| corridor_guards_07000012_live | 1 / 0 |
| enemies_07000012_live | 3 / 0 |
| nf-game --lib | 782 / 0 / 4 ignored |
| nf-formats + nf-audio | 27+1+4+1 and 92+1, all 0 failed |

The shared-helper duty: `walker_b2.rs` and `tower2.rs` are shared, so every `walkreal_*_live`
binary and the whole interactables family that includes them ran whole (the table above; the
interactables set 03/06/07/0a/0b/0c/0d/13/14: 9, 5, 4, 14, 14, 7, 11, 10, 10 passed, 0 failed —
and interactables_07000012 is 12/0). No regression anywhere; every ignore that remains is a
pre-existing ledger from an earlier task.

## What's left

- Nothing on this task's item. The next 07000012 lever, if wanted: the north-lobby fight (#5,
  killed false at tick 5741) ended alive both this run and the predecessor's d3 runs — the
  north-lobby pre-clear lever (`PRE_CLEAR_NORTH_LOBBY`) exists and is gated to the 07000013
  chain; arming it for d1 would be a separate pinned-behaviour decision, not needed for green.
- `tools/test/suite.py` did not run: background processes die between tool calls in this
  sandbox (started it, `suite.out` stayed 0 bytes, no process survived the tool-call gap — the
  same constraint `los-mask-repins-12` recorded). The per-binary coverage above is the
  substitute; the next sandbox that keeps background alive should run the suite on this branch
  before more code moves.

## Live checks for the main agent

None needed — this task's fix is in the test walker and the test route, and the game-facing
behaviours it rests on were already live-checked by los-mask-repins-12 (the 543 crossing line
(6.21, 1.13, −25.40) vs the port's (6.25, 1.13, −25.39)). If a render pass is cheap: the
Operative run's Bond at the door-543 west stand (≈ 9.3, 1.1, −12.8, facing south) should show
the leaf swung WEST (away from the corridor), and at the south hall (≈ 6.5, 1.1, −19.5) the
lower-floor guard's post at (−9.1, −4.1, −21.0) should be visible through the hall's opening —
both are port-side facts the walk asserts indirectly.

## Usage

not visible to me — this environment exposes no token, credit, cost or context-window figures
for the session, and I will not estimate them.

## ntfy failed

(filled in after the finish steps — see the section below if the fallbacks fail again)
