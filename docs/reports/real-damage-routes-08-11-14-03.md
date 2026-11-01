# real-damage-routes-08-11-14-03 — report

Task: the real-damage walk variants (walker return fire, `NF_WALK_NO_TOPUP`,
`crates/nf-view/tests/common/damage_log.rs`) survive on master but each ROUTE then breaks:
03 (captors killed before the servant rescue), 08 (wedges on the walkway leg), 11 (a
stair-height check misses by 0.01), 14 (misses the stairwell jump). Fix each route on the
walker side only and add real-damage variant tests that complete each level.

## What's done

**03 — COMPLETE, the real-damage walk passes.** `walkreal_07000003_live`
`walk_the_full_route_and_complete_every_objective_real_damage` runs the whole mission on real
damage: **1 passed, 82.2 s**. The pieces (all walker/test-side, no game code, no assertions
weakened):

- Captor spares (`DamageLog::spare`): the return fire must not eat the rescues — the captors are
  the DTYPE-4 killer guards [M NDrone2_GetDTYPENEW 0x14c938 case 4] and one killed between its
  enable plate and the hostage bind strands the save. The test fills `spare` with the team-1
  mode-4 drones until both rescues have fired, then clears it.
- The shooter re-fight rule: the 240-tick cooldown now only shields a guard who has STOPPED
  shooting; a guard still hitting Bond (he is on the shooter list — the same 240-frame window) is
  re-fought after 120 ticks. Without this a leg pressed past a wounded shooter dies in the open.
- Dry resupply after a fight that emptied the gun, answer-shooters during the weapon cycle, and
  `try_walk_to`/`walk_nav_to` restoring the CALLER's `lenient` (a nav leg inside a lenient route
  clobbered the flag and the next leg panicked at the cap instead of giving up).

**11 — the stair-height break is FIXED; the walk now dies later, in the 07000012 extension.**
The break: after "the west flight C down" the band asserted y < 32.5 and the real-damage run
read **y 32.513348** — 0.013 over. Cause [D the 35.2→31.1 / 8-z flight scan]: the leg targeted
the flight's bottom (14.8, 31.4, −2.6) with r(1.0); the arrival rule is horizontal-only
(`d = dxz(p, wp.at)`, walker.rs:1077), so the run arrived at (14.84, 32.59, −1.61) — dxz 0.99,
still 2.8 z up the flight (the flight falls 0.51 y per z). The top-up run's approach heading
arrived nearer the bottom; the real-damage run's return-fire fights shifted the heading. Fix
(walker side, the route's own legs): target flight C and flight B PAST their bottoms, on the
north strip (z −3.4) — a radius-1.0 arrival then stops no higher than z −2.4 (y ≈ 31.2). After
the fix the real-damage run passes every band (flight C y 32.27, flight B y 24.09), walks the
whole stairwell chain (+3617 ticks), reaches the Load Point, E-loads 07000012, and dies at tick
3122 in the east wing — see "What's left".

**08 — the walkway wedge is GONE; the walk now dies at the Hind's opening blasts.** The
real-damage run walks the whole approach (station hall, walkway legs, stairwell, the fights),
boards the gondola (the floor plate fires ch 4), rides to the bottom (ch 83 at the script's tick
1410 [D stream 46]) and dies at tick 33381. Evidence from the traced run (`NF_DMG_TRACE=1`):
- en route: drone #14 −2.4 (tick 762, d 4.7), the #20 fight −10.8 (3 hits, d 10.0, weapon 0x15),
  the #19 fight −12.6 (7 hits, d 8.9) — Bond arrives at the bottom with **74.2 health**;
- the Hind's blasts: three kind-44 explosions (radius 8.0, damage 100.0, owner −1) at
  (110.4, 14.5, −179.3) d 1.3, (107.0, 16.0, −173.4) d 7.9, (112.0, 15.0, −174.5) d 5.5 cost
  **−52.5, −7.5, −14.2 = 74.2** — exactly his arrival health. The blasts are the Hind's own
  attack during the fight (Bond fires his 4 AT-420 rockets [D the ride's pickups]; the Hind
  closes 51→18 u and its rounds detonate at Bond's feet — a standing player eats the same
  proximity blasts; the fight legs were written for the top-up walk).

The 08 break the claim described (wedges on the walkway leg) no longer reproduces on this tree:
the walker-side checkpoint work (the shooter re-fight rule, dry resupply, the lenient restore)
carried the route through the walkway legs.

**14 — unchanged (my experiments reverted).** The real-damage run still fails at the stairwell:
the west pit jump lands Bond on the stairwell frame (y 0.96 → the climb walks it → the foot
assert reads y 0.213 > 0.0) instead of the −0.91 landing. What I established, all [D] via the
port's collision (`cargo run --example probe14_corridor`, the committed grid probe):
- the landing is TWO strips (x 44.75..46.25) at z 51.5..52.25 and z 54.0..54.5, with the pit
  (−2.40) between them at z 52.5..53.5 — a mid-run cadence jump from the lane leg's z lands in
  the gap or on the adjacent 0.60 segment (z 48.5..51.0);
- an edge-anchored jump (the east jump's proven pattern: try_walk_to the band's east edge, hop
  at i==0) fails differently: the anchor leg itself WEDGES 0.39 u short of (47.9, 51.75) — the
  stuck dump shows a clear 0.9998 segment, `floor_below −0.5998`, no drone or door within 2.5 u
  — a phantom lip the walker cannot cross; face-west-then-jump kills the run-up and the ~1.3 u
  arc dies in the band; approaching from the east with momentum changed nothing (deterministic);
- a tap-interval experiment (`fast_taps`, automatics rip at 3-tick gaps) made 03 FAIL — reverted;
  the lesson stands: the walkers' fight timelines are load-bearing for the other real-damage
  routes.

`walkreal_07000014_live.rs` is back at its committed state (the real-damage wrapper included);
nothing of 14 is changed on the branch.

## Runs quoted

- `walkreal_07000003_live` `..._real_damage`: **ok. 1 passed** (82.2 s, and 82.7 s on the
  pre-revert build) — nf-view.
- `walkreal_07000008_live` `..._real_damage`: FAILED — died at tick 33381 (traced rerun
  included; 23.2 s + 35 s).
- `walkreal_07000011_live` `..._real_damage`: FAILED — died at tick 3122 (4.8 s; the stair
  bands all pass now).
- `walkreal_07000014_live` `..._real_damage`: FAILED — the stairwell foot assert (2.0 s).
- `cargo test --offline --release -j2 -p nf-game -- --test-threads=1`: **1014 passed,
  0 failed, 9 ignored** (every binary ok).
- nf-view compiles clean (`cargo test --no-run -p nf-view` green after every change).

## What's left (the next message continues from this branch)

1. **08**: arrive at the Hind fight with > 74.2 + margin. The levers found: the #19/#20 shaft
   fights cost 23.4 of the 25.8 (weapon 0x15 guards at d 9-10 — `prefer_head`'s 6 u cap and
   `keep_off`/`back_off` 4.0 do not engage there); the fights used gun 0x19 (semi-auto — the
   tap gap is not the limiter). Options: a scripted pre-emptive fight from the stairwell mouth,
   or `switch_to_armed` preferring the harder-hitting owned weapon (0x4a, 100-round MG [D
   weapon_handling rows]) — walker-side only.
2. **11**: the 07000012 extension's east wing — the return-fire fights cost 21-24 health each
   (gun 0x4e/0x19 at d 9-12, guards partly in cover, "killed false" leaves the fight to the
   cooldown while fresh shotgunners volley). Needs the same fight-economics work as 08; the
   shared `tower2::walk_12` route must not be re-timed for the top-up walks.
3. **14**: the west pit jump. Needs either a jump solver in the walker (the arc is ~1.3-1.5 u
   from a standing start) or a live check of how the game's own player makes the crossing.
4. The fast runner on the final tree: my branch is 57 commits behind origin/master; the merge +
   full `tools/test/suite.py` rerun did not fit this window (the four walkreal binaries + nf-game
   above were run directly). Merge origin/master, resolve, and run the suite once before the
   next push.

## Live checks for the main agent

1. **08 Hind blasts**: during the bottom-of-mountain fight on a FRESH boot, watch whether the
   Hind's rockets damage the player at full health (PCSX2: hp before/after the first proximity
   blast; expect ~−50 if the port is faithful). Address: the Hind's blast spawn — the port logs
   kind-44 explosions at (110.4, 14.5, −179.3) radius 8.0 damage 100.0; the game's equivalent is
   the copter's weapon fire at Bond's ride-end position (111.05, 13.53, −179.70).
2. **14 west pit jump** (corridor x 48.8, y −0.05, z 52.8): how does the game's own player cross
   to the −0.91 landing (x 44.75..46.25, z 51.5..52.25 [D probe grid])? Record the player's
   position/yaw/jump tick making the crossing on a fresh boot. If the game crosses elsewhere,
   the port's route leg is wrong, not the jump.
3. **11 flight C bottom** (the stairwell's west flight, 35.2→31.1): whether the game's player
   standing at the flight's bottom reads y ≈ 31.1+0.0 (feet) — confirming the band window
   (30.4, 32.5) is the flight's true bottom band.

## Usage

not visible to me — this environment exposes no token, context-window or cost figures for the
session (no numbers to copy; nothing estimated).

## ntfy failed

All three fallbacks failed (the sandbox cannot reach ntfy.sh):

1. `curl -sS -m 20 -H "Title: NightfirePort" -d "real-damage-routes-08-11-14-03: ..." https://ntfy.sh/nightfire-yoda-8k3x`
   → `curl: (28) Connection timed out after 20000 milliseconds`
2. `python3 -c "... urllib.request ..."`
   → `urllib.error.URLError: <urlopen error [Errno 101] Network is unreachable>`
3. `wget -q -O- --post-data "real-damage-routes-08-11-14-03: ..." https://ntfy.sh/nightfire-yoda-8k3x`
   → timed out after 30 s, killed (no output).

