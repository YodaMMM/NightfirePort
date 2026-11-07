# aim-route-finish — the 14 x3 route reds latch; the 1b pacing measured (the gate lever reverted, the 7→1 climb-out kept, the next lever identified); 08/0c/16 untouched

Branch `s/aim-route-finish` (from `origin/s/1b-zero-g-floor`, merged `origin/master`, clean).
No `crates/*/src/` file changed on this branch — every change is a test file or a test-doc fix,
so no game behaviour moved except where a walk's own route changed. The [L] 08 pins and the
0x30000000 fix are untouched (inherited from the aim-console-phase lineage).

## 1. The 14 x3 doorway leg — fixed by the west lane, not the doorway

The three reds (`the_e_use_switch_fires_the_sabotage_cascade_and_opens_the_west_door`,
`walk_the_infiltration_to_the_delta_exit`,
`walk_the_tripbomb_route_pick_up_place_and_shoot_it`) all stuck at "the corridor south to the
leg-2 spot": the straight line clips the door wall's west jamb (the face bars the direct south
steer at x ≥ 30.3 — the segment hit 0.17 along from (30.33, 0.55, 35.44) [measured, the
1b-zero-g-floor probe]).

**The report's doorway waypoint (≈ x 31.8, z 36.3) was tried first and measured WEDGED.** The
walker targets the doorway, the kind-2 door at (32.0, 2.1, 35.8) starts moving, and the walk
never crosses: the stuck dump reads `bond=(30.70,0.54,35.21) target=(31.80,0.65,36.30) d=1.55
segment_to_target=Some(0.48)` with the door `state=0x8 pos01=0.77` after 6000 ticks. The
mechanism: the walker's E pulses toggle the door ([M Door_Activate is a TOGGLE]) while
`walker_b2::use_door` early-returns on the moving state (0x8 — "already open/opening, just walk
through") and resets the stuck counter, so the mitigation never escalates; and the door's
proximity scan follows the LEAF's live sphere ([M Door_Update 0x1363c8:52-68 — the moved pass
rebuilds obj+0x80 after Door_Interp]), which slides away from Bond as the leaf opens — the
open/close cycle never settles. The report's own alternative lever was used instead:
**re-anchor the leg's approach on the door line — the west lane the probe walked**
(`w.walk_to(Wp::new(29.7, 0.8, 36.2).r(0.7), "the west lane past the jamb")` before the leg-2
spot, at both sites: the shared `walk_the_west_route_to_the_switch_band` and the tripbomb
walk's copied block). From the lane the leg-2 line clears the jamb by ~0.5 u.

Measured: the whole `walkreal_07000014_live` binary **9 passed / 0 failed / 1 ignored**
(single-threaded, 146.8 s) — was 6/3/1. The delta-exit Agent/00-Agent variants, which share
the edited fn, pass unchanged.

## 2. The 1b missile deadline — measured; the gate lever regressed and was reverted; the climb-out kept; the real lever identified

Three runs of `walk_arrival_to_drakes_defeat` (the plain drakes walk, ~295 s each), latch
frames (console, absolute sim frame):

| run | levers | latched | MissileDeploy |
| --- | --- | --- | --- |
| the 1b-zero-g-floor report | gate 48 | 3@4017, 4@4425, 5@4629, 6@5043, 8@5511, 7@5943, 1@6406, 2@6537 | [3,3,2,2,3,2,2,3] |
| run 1 | gate 36 + climb-out | 3@4017, 4@4377, 5@4755, 6@5097, 8@5859, 7@6021, 1@6484, 2@6615 | [3,3,2,2,3,2,2,3] |
| run 3 (landed) | gate 48 + climb-out + anticipation arm | 3@4017, 4@4425, 5@4629, 6@5043, 8@5511, 7@5943, **1@6370, 2@6513** | [3,3,2,2,3,2,2,3] |

- **The burst gate stayed at 48.** Lowering it to 36 (the report's lever — "the three holds'
  exact cost") moved console 4 −48 but regressed 5 by +126 and 8 by +348: the earlier bursts
  restart the recharge churn inside each hold chain (the tap pattern in the [snap] trace turns
  into 5-tap bursts with 60-130-tick dry gaps). Reverted with the measured numbers in the
  comment.
- **The 7→1 departure-side two-phase crossing kept** (`climbing_out` latch in `shoot_consoles`:
  while Bond is below y −7 and the goal is ≥ 2 above, fly to goal.y + 5 first — the straight
  line from (46,−7) to (20,−2) runs y −5.6 at x 38.7, under the bank's face top −5.21). It
  pulled console 1 **36 ticks early** (6406→6370) and 2 by 24 (6537→6513) — the 7→1 flight is
  inside 1's hold-wait window, so a cleaner flight lands directly.
- **The anticipation arm added but measured dead** (runs 2 and 3 differ by nothing — run 2's
  latches are bit-identical to run 1's). The arm floats Bond to the NEXT console's station
  whenever no hold is up. It never fires because **the console scripts spend their lives
  WAITING at holds**: `held_hit_targets` is non-empty for the current script almost every tick,
  so the hold-gated planner block keeps the stale station and the next console's flight runs
  inside its own hold window (the measured 6→8 chain: 8's first hold appeared at loop-tick
  ~2351, the crossing left at ~2351, the first tap 337 ticks later — the whole flight eats
  8's chain, and 8's missile read at S+2399 catches all of it).
- **The next window's lever (the recipe, not yet built): count answered holds.** Each console
  script carries three holds ([D the probe1b_scripts decode]); a hold resumes on the tap, and
  the script then waits at the next hold. When the walk has answered the LAST hold of the
  current console (it knows its own hit count per console), the current console's channel is
  imminent — start flying to the next station THEN (while `best` is still Some), which pulls
  the ~300-tick flight out of the next console's chain. That is the only remaining ~300-tick
  block; with it, 8's latch ≈ hold₁(8) + chain(~200) ≈ S+2550 (still ~150 over S+2399 — the
  exact p0 start frame S, the 1b report's live-check item, turns this into arithmetic).

The task's stop condition stands: `MissileDeploy [3,3,2,2,3,2,2,3]` — consoles 5/8/1/2's
missiles still arm. The disarm asserts after it are unreached this window.

## 3. 08, 0c, 16 — not started; the classification carried

The 1b-zero-g-floor probe's classification stands unchanged (all route, none snap): **08** the
station legs' hurt volume (death at d 84.8-92.3 chip exposure + a 78.0 world hit — the
station-legs re-pin its drone.rs comment predicted); **0c** the far-sniper drain (the
finishers at d 61-90 left at full hp — the plain 20 u shooters window); **16** the point-blank
melee swarm (three swimmers in the 0.8-1.3 always-hit band). The route levers named there (the
03 walk's hurting_first arm, the 06 sweeps, the station-legs re-pin) are the next window's
work; nothing on this branch touches those binaries' inputs (test-file changes only, listed
above), and their red counts are exactly the pre-existing 5/1, 2/1, 13/1/2.

## What this branch does to the stream batch

This branch moves where the 1b console rounds land (the snap-aware planner + the two-phase
crossing from the 1b lineage) AND their timing (the 7→1 climb-out shifted 1/2 earlier; the
gate experiment's numbers are in the branch history but its net state is the original 48). The
1b drakes walks are still red at the MissileDeploy gate, so per the 1b report's own advice
("joining s/stream-batch makes sense once the 1b drakes walks are green end to end"):
**this branch should NOT join the stream batch yet** — join after the hold-counting
anticipation lands and the drakes walks go green, so the batch picks up one stable timeline
instead of re-pinning twice.

## Verification

- `walkreal_07000014_live` whole: **9 passed / 0 failed / 1 ignored** (was 6/3/1).
- `walkreal_0700001b_live` `walk_arrival_to_drakes_defeat`: 3 measured runs (the table above);
  the binary's other 15 tests do not call `shoot_consoles` (the only fn edited) and were green
  pre-window; `walk_arrival_to_drakes_defeat_on_real_health` NOT re-run this window (each run
  is ~295 s; it shares `play_the_mission`/`shoot_consoles` with the plain walk, so its red is
  the same MissileDeploy gate).
- `cargo test --offline --release --no-fail-fast -p nf-game -p nf-formats -p nf-audio
  -- --test-threads=1`: 162 test binaries ran, 160 ok (nf-game's 684-test lib suite, the door
  live tests, npc/fix suites included); `nf-formats disc_golden` 11/3 — **the 3 allowed slim-ISO
  failures**; `nf-formats identity_disc` 1 failed — the onfoot-ISO artifact (the disc identity's
  volume NIGHTFIRE / SLUS-20579 / NTSC / 1.00 all match; the assert is the FULL disc's
  1,191,072 sectors, the onfoot image has 518,435 — the same class as the allowed 3, no code
  regression).
- The rest of `nf-view` (the other walkreal binaries) did not run this window: this branch
  changed only the 14 and 1b test files (no `crates/*/src/`, nothing under
  `crates/nf-view/tests/common/`), so no other walkreal binary's inputs moved.
- The door-test compile fixes were needed because the aim-object-flags lineage changed
  `Doors::create`'s object closure to carry the true glist word (the 4th field, `o.flags` =
  celglist+0x44 [M 0x1d1508]) and missed the five nf-game door test files — this tree's first
  full nf-game run found them.

## What's left (next window)

1. 1b: the hold-counting anticipation (the recipe in §2) — the last ~300-tick flight out of
   8/1/2's chains; then the real-health drakes walks and the exact p0 start frame S (the live
   check below) to close the deadline arithmetic.
2. 08: the station-legs re-pin; 0c: the far-shooter lever; 16: the melee-swarm spacing (the
   1b report's §3 levers).
3. The full `nf-view` suite on a window with room for the walkreal set.

## Live checks for the main agent

1. **The 14 kind-2 door at (32.0, 2.1, 35.8)** — in PCSX2, stand Bond just west of the jamb
   (30.7, 0.55, 35.2) and watch the door: does the GAME's door cycle open/closed the way the
   port's does (the proximity test reading the leaf's MOVED sphere, [M 0x1363c8:52-68])? If the
   game's door stays open there, the port's moved-sphere rebuild or the auto-close arm
   diverges — that decides whether the doorway waypoint can replace the west lane.
2. **The missile p0 scripts' start frame S** (carried from the 1b report): watch ch 19's rise
   and the first missile script's state-1 tick counter — the exact S turns the §2 budget into
   arithmetic (bounded 2644 < S ≤ 2830 by the measured disarm/arm split).

## Usage

not visible to me — this session exposes no token, credit, context-window or cost figures to copy.
