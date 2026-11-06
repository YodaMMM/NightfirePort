# guard-repin-master-2 — 03 real_damage_agent green; 13 lever landed (measured), 0d promise retired with the PCSX2 budget, 08 00agent measured ignore

Branch `s/guard-repin-master-2` (from `origin/s/guard-repin-master`, already up to date with
`origin/master`). Walk levers only; no [L] pin loosened beyond item 3's promise change, no game
code changed.

## 1. 03 real_damage_agent — FIXED (green)

The report's far-shooter theory (d 30-40 through glass) was corpse-distance misleading. The
NF_DMG_TRACE ledger of the red run shows the real kill vector in two parts:
- the courtyard cluster's #11 (skin 0x5000038, hp 24, weapon 0xe) opened at tick 7523 from
  (14.5, −29.0) d 9.1, soaked three plain fight windows (7931/9419/9952) and CHARGED to d 2.9
  on the west-lawn legs, dealing 56.0 — the walk entered the servant legs at 34.6;
- at the servant-0 door (22.3, 10.9) the east-lawn posts #5 (shotgun 0x1c, 12 hp, d 3.9) and #3
  (0xe, hp 24) unloaded exactly the remaining 34.6 in 222 ticks (51583-51805) — Bond died at
  51805 while the door-opening loop held him (the fights run inside nav legs, not use_door).

FIX LANDED (all file-local to `walkreal_07000003_live.rs`, d-gated `difficulty >= 2`, the plain
d1 walk untouched):
1. the 06 `stand_shooters_down` law copied level-local (`stand_shooters_down_03` + `aimed_stand_03`,
   the eye-ray exposure set this walk's own fights fight with — the strict gun-ray set returned
   "exposed false" for every candidate from the leg-end anchors), placed at the servant leg ends
   (each rescue loop's end) and the exit lawn before the E;
2. THE MEASURED LEVER: a walk-back leg to (11.0, 6.5, −21.5) — "the mid-courtyard angle (the
   shooters' fight line)" — before the 0x21 plate legs. From the leg-end anchor (13, −18) neither
   #10 nor #11 has a reachable box (the cand dump measured "hitting true exposed false" for
   both); the fights fought them from the nav's mid-leg stops (10.4..11.5, −20.8). The angle leg
   re-takes that line and its own return-fire fights take the cluster there: the walk reached
   the servant legs at 76.6 instead of 34.6, the door ambush's #5/#3 were killed (39760/40080),
   and the run passed the door —
3. `w.dmg.hurting_first = true` (the 08 walks' opt-in damage-weighted pick): the new leak was
   the east-lawn post #4's 26-chip volley (36.7, ticks 40215-40329) answered only after the
   nearer #3/#5; the damage-weighted pick answers the biggest current damager first.

Measured: `walk_the_full_route_and_complete_every_objective_real_damage_agent` PASSES
(15.8 s, single-test run; the whole binary did not fit the window — see the battery).

## 2. 13 warm chain real_damage — lever landed and measured, NOT green

Two levers landed this window, both file-local d3 (the d2 Agent sister stayed green through
both):
- the lobby north-west sweep (`clear_the_visible` at the (−14.0, 40.0) leg end, radius 24, the
  report's own "move the west-mid sweep's spot west/north"): #9 stood down (killed false, his
  boxes never open from there), #10 killed. The death moved 3006 → 3472 — same room crossing,
  #8's point-blank 31.5 at d 2.9.
- the east-lane-leg-end sweep: MEASURED AND DROPPED — that leg end is a killing floor; the sweep
  stood Bond in the lobby mouth and the crossfire killed the walk at tick 1384 (34 hits for 79.0
  mid-sweep). The run's ledger also shows #5's 28.0 in 8 hits land at frames 1914-2101 — BEFORE
  the west-mid sweep kills him — so the budget drain starts on the east-lane leg, but the stand
  spot to answer it from does not exist on the route today.

What's left: the budget is spent to 31.5 by the room door (79.0 taken: #5 28.0, #11 10.5, #6 6.0,
#0 3.0, plus the chain's carried cost) and #8's single 31.5 finishes exactly that. The lever that
stands is an answer DURING the doorway crossing (a doorway aim law that looks DOWN the room and
pre-fires #8's charge — his rays never open from any leg end, only once he is IN the doorway), or
an earlier, cheaper #5 answer (a spot east of the furniture maze that sees his post). Both need
their own measured session.

## 3. 0d c5 door — the promise retired with the measurement; the walk NOT green

The test's "zero-hit while moving in every run" promise is retired in
`walkreal_0700000d_live.rs` (the transit comment block): the PCSX2 measurement the task cites is
now the lane's documented law — Bond held at (208.62, 1.04, −229.64) for 45 s took 14 hits, with
single rounds crossing class 0x1d objects at (163.9, 7.2), (173.9, 6.2), (189.5, 4.6), (200.1,
3.6) and flying on along one line [L the PCSX2 hold + the round-line dump] — the strip is NOT a
zero-hit lane, the old promise was the pre-radii branch's glass-shield property. The lane's
budget line in the test stays the strip-kills assert (≤ 48 of the pool).

The 600-tick zero, explained in the test comment and repeated here: the strip-mouth #10 kill at
cap 600 scored zero landing hits while probe_0d_ray_stands reads the eye→chest ray CLEAR at
d 63.6 and the walker's aim converged (the aim-bit zero-spread act [M Bullet_init 0x12c5a0]). The
walker's convergence targets the guard's CURRENT hit-box centres, so a full-cap zero means the
rounds' world trace never reaches the boxes they aim at — either the guard's boxes sit elsewhere
than his world position reports (the radii work may settle him off his placement (214.21, 12.09,
−289.44) [D]) or the intervening geometry stops the rounds somewhere the vision ray passes. That
is a game-code/box question, not a walk problem — the live check below pins it. The 180 dwell
stays (measured: the 600 attempt was pure exposure, 38.0 taken, kill never landed).

Binary state: 1 passed / 1 failed / 1 ignored (the drop-phase death at 9300, "the drop must land
on the annex", 94.0 taken — the transit itself survived; the agent sister rides its pre-existing
ignore). What's left: the drop-phase budget (the climb legs' volleys answered) — its own session.

## 4. 08 whole mission real_damage_00agent — measured ignore (task-allowed)

The station-legs re-pin cannot be done in this window: the stands' GLASS_SIGHT fights run
(14 fights) and the d 59-97 exchange still loses — #20 (d 59.2) 36.0 in 3, #23 (d 71.0) 9.0 in 3,
#4 (d 81.4) 16.0 in 2, #14 (d 97.5) 4.0 in 1, death tick 27596 to a hurt volume after the sniper
drain [L the guard-repin-master ledger]; the branch's own drone.rs comment predicted exactly this
red when the glass pass was armed ("the real-health route cost is re-pinned with the 08 station
walks"). The task allows an ignore with a measured reason for the 00 Agent sister ONLY:
`walk_the_whole_mission_on_real_input_real_damage_00agent` now carries that ignore (the full
ledger + the re-pin task in the ignore text). The d2 Agent sister stays green and unignored.
Measured: the 08 binary is 5 passed / 0 failed / 1 ignored (117.3 s).

## The battery (whole binaries, --test-threads=1, slim ISO, no --include-ignored)

- walkreal_07000008_live: **5 passed / 0 failed / 1 ignored** (117.3 s) — the ignore is this
  task's own measured 00agent one.
- walkreal_07000006_live: **6 passed / 0 failed / 1 ignored** (20.4 s) — the 1 is the file's own
  pre-existing 00agent ignore.
- walkreal_07000013_live: **2 passed / 1 failed** (34.9 s) — the d3 real_damage red above.
- walkreal_0700000d_live: **1 passed / 1 failed / 1 ignored** (26.9 s) — the drop-phase red above.
- 03: the target test itself passed (single-test run above); the whole 750 s binary, walkreal
  01, 16, 1b and enemy_accuracy_live did NOT fit the sandbox window this time — no counts from
  this session; the next message runs them first.

## What's left (next window)

1. 03: the whole-binary battery + 01/16/1b/accuracy (rule 10's list).
2. 13: the doorway-crossing answer for #8 (pre-fire down the room) or a cheaper #5 answer —
   the budget lever that stands is not a leg-end sweep.
3. 0d: the drop-phase budget (the climb legs' volleys), after the bay-guard box question.
4. 08: the station-legs re-pin (the 00agent ignore's text names the two candidate levers).

## Live checks for the main agent

1. The bay guard #10 (0700000d placement (214.21, 12.09, −289.44)) — while the player stands at
   the strip mouth (196, 0.5, −228.5): read Drone+0x8EC (goal_arrival), his state, and the
   hit_boxes WORLD positions vs the placement; then fire one aimed round at a box centre and
   record where the world trace stops it. The port's 600 converged ray-clear shots at d 63.6
   scored zero; if the boxes or the settled position disagree with the placement, that is a
   game-code bug (access: read; expected: the boxes' world centre within ~0.5 of the placement
   chest).
2. The 03 east-lawn post #4 (0x5000036 at (23.8, 6.2, 13.9)): confirm which enable channel arms
   him and at what tick he first opens fire after Bond's servant-0 route passes the east lawn
   (access: watch +0xF4 enable and the first bullet create; expected: armed by the 0x21/0x24
   plate family, first fire only when Bond re-enters his sight band — his 26-chip drain was
   answered only by the damage-weighted pick).

## Usage

- Session token/context/cost figures: not visible to me (no token or cost display is exposed
  in this environment).
