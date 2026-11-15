# 10-level-0d-operative-walk-10

0700000d's Operative real-damage walk (`walkreal_0700000d_live::walk_arrival_to_the_c5_door_real_damage`) is still
ignored. Master 36db1e785 carries rounds 7-9 (the walkers' crossing sweep hoisted before the roof retarget): the
death is still in the ladder-top phase, t4161 at (213.07, 17.37, -234.51), 10 hits for 100.0 - the trio's first
64.0 lands before the sweep opens (~3800: #29 32.0 at 3042/3222, #27 at 3398/3578). Read the ignore message's
round-9 ledger and take its next lever (answer #29/#27 before the climb, from a line the scans already found).
Real input only, no top-ups; the plain replay and the d2 twin stay byte-identical.
Tests: walkreal_0700000d_live, interactables_0700000d_live. Branch: s/level-0d-operative-walk-10.

---
claimed-by: sandbox4 at 2026-11-14T18:26Z

Main 11-14 (master 06db4a156, NF_DMG_TRACE): the ladder-top bill is the 0x1e SNIPERS - #28 14.0 mid-climb (t2425), #29 16.0 x2 (3042/3222), #27 16.0 x2-3 from the west roof (151.4,22.7,-245..-256) at 62-65 u, #16 16.0 from the deck, #10 16.0+6.0 from the bay; the climb itself takes 864 ticks. A west-roof sniper pass FIRST at the ladder top (kill_guard_at_present (151.2,-248.0) +-(6,12), 900) kills one (+224) and the death moves t4161 -> t5052 (#27 still alive, 6.0 health idling); three passes die inside them (t3664). Next: all three west snipers before the deck/bay acts, or from the ladder base before the climb.

Main 11-14: the west-roof snipers from the LADDER BASE (kill_guard_at_present (151.2,-248.0) +-(6,12), 600) - no line (pass false at 100.0); the walk then dies at t3947 (one fewer hit absorbed... same phase). The base has no sight of the west roof.

Main 11-14 (667be11e3, crouched ladder top merged, death t5532): a dedicated pass on the bay sniper #10 (214.7,-289.5) +-(3,3) before the bay duo kills him (+487) but the time lets #14 (south roof (166.3,21.2,-271.4), 16.0 x2) and #27 (west roof, x3) finish Bond at t4399. The ladder top faces ~5 0x1e snipers at 55-65 u, each kill costs 200-500 ticks while the others land 16 every ~200: the stand order alone cannot win; it needs cover between kills (crouch helped one hit) or the snipers thinned before the climb. Second west pass before the bay: t4025.

Main 11-14 (667be11e3, crouched): a second west-roof pass before the bay duo does not land (pass false, 38.0) and the walk dies sooner, t5245 vs t5532. Not shipped.

Main 11-14: a ground scan before the climb finds the snipers in clear line from most of the yard (186..216, -240..-222): #27 63 spots, #28 71, #29 66, #16 59, #10 49, #18 30 (#17 none). West-roof passes from the ground at (210,-237)/(204,-237) kill one then die on the open ground (t3261/t3198) - the whole yard is in the snipers lines; the crouched ladder top (t5532) stays the best stand.

Main 11-14 (merged 272ec32c8): the yard armour (kind 3 at (205.4, 0.1, -205.3)) from the blind strip before the climb (Operative real damage only) - 50 taken, 34 back at the strip; death t5532 -> t10148 on the deck at (202.0, 16.0, -255.1): #14 (0x5000087, 0x1e) from (166.3, 21.2, -271.4) 16.0 x4 at 39.6 u, then the bay pair #18/#10 from (206-208, 12.1, -288.5) at 34 u. Other live armour: (192.8, 10.8, -272.0) 42 u, (147.0, 21.8, -252.8) on the west roof, (107.7, 6.8, -275.2).

Main 11-14 (merged afec3f768): the south-roof pair act at 2 x 900 (Operative) lands #14 at +889; Bond reaches the gantry ride alive at 29.8 and dies ON it at t18516 at (144.1, 14.6, -296.1) - two sniper hits (16.0 at t18156, 12.6 at t18516). The ride loop steps the sim directly, so its death was silent before (the walk then failed as a stuck slot hop with health 0); it now asserts by name. Next: the ride-window sniper (the pre-ride #19 box act is d2-only; #19 skin 0x5000087 at (172.0, 19.6, -292.5)).

Main 11-14 (on afec3f768, not shipped): the gantry-ride killer is #15 (skin 0x5000087, 0x1e) on the GROUND at (176.2-178.0, 1.0, -280.5..-281.1): 16.0 at t18156 + 12.6 at t18516. A box act on him from the boarding spot before the ride: the ray dies 0.102 along (the gantry floor), and the rifle is EMPTY there (0+0 - the 2 x 900 south-roof acts spent it); death t18317. The ladder-base rifle ammo (221.5/221.6, 0.5, -232/-233) is refused at the climb (spare already 30) and the detour costs the climb (t7376). Next: rounds for after the roof acts, or a ride-time answer to #15 with another gun.

Main 11-14 (merged d1491c571): the rifle (switched back from the Q-Claw) aims and taps at #15 on clear lines during the gantry ride - #15 dead in 30 taps, ride ends at 26.2, the slot drop lands, death t19289 in the yard at (137.7, 4.6, -296.1) to the SMG trickle (0x500006d #2/#3/#6/#8, 0x500007e #1, 0.6-1.2 a hit, 44 hits for 83.6 total + 16.4 other).

Main 11-14 (on d1491c571, not shipped): the yard landing arrives at ~10 (26.2 on the ride minus the 16.4 slot fall) and dies in ~200 ticks to the SMG trickle. The armour at (192.8, 10.8, -272.0) is NOT reachable from bay enter (194, 10.6, -282): a wall at z -276.4 stops the nav 4.4 u short (12k ticks lost, death on the ride t31750). Health for the landing must come from the climb hits (#17 32.0 at 71 u, #28 16.0, #10 14.0).

Main 11-14 (on d1491c571): the ladder-top band sweep at 3 x 600 / 3 x 900 (was 200) never lands on #17 at (163.3, 1.0, -214.0), 53.7 u, and dies earlier (t15093 in the bay) - the 200 cap stays.

Main 11-14 (on d1491c571): the climb-side bill is #10 14.0 (t10459, deck), #17 16.0 x2 (t12022/12388, catwalk, from (157.6, 1.0, -226.0) at 48-55 u) and #28 16.0 (t13479, roof east b). Crouching for the catwalk keeps both #17 hits and drops #28s, but the crouched legs end elsewhere and the annex push-off fall pin (timer 76) fails even after standing again. Not shipped.

Main 11-14 (on d1491c571): a rifle act on #17 from the walk-off strip before the catwalk (kill_guard_at_present (157.6, -226.0) +/- (5, 6), 600): #17 down at +500 but Bond is at 38.0 after it (the act window draws the same fire) and dies earlier in the yard, t18367 vs t19289. Not shipped.

Main 11-14 (on d1491c571): sniping the yard from the gantry park before the hop - the yard guards (#2/#3/#5, 0x500006d/0x500007e) are already AWAKE (states 0x68-0x96) at the apex, the box acts land nothing (rifle 5+13 unfired), and the gantry carries Bond back east while he waits (x 140 -> 179); death on the gantry t20441. Dead lever: the hop must happen at the apex.

Main 11-14 (on d1491c571): extending the ride fire to the nearest yard guard with a clear bullet line after #15 dies: no extra taps (30, no clear yard lines from the moving gantry), t19321 - noise. Not shipped.

---
## Report (done, 2026-11-15T02:10Z)

# level-0d-operative-walk-10 — the ground-answer lever, measured dead; the ledger + the flight-trace next lever

Branch `s/level-0d-operative-walk-10`, base origin/master 820819cb3 (36db1e785, which carries
rounds 7-9, is an ancestor). The driver's lever: answer #29/#27 before the climb, from a line
the scans already found. The walk
`walkreal_0700000d_live::walk_arrival_to_the_c5_door_real_damage` stays **ignored/red**; the
round-10 ledger and the next lever are in the ignore message.

## What's done

1. **Fresh recon on this tree** (two probe runs, assertion-free, `--ignored --nocapture`):
   - `probe_0d_ray_stands` (the existing matrix probe): every ground stand's eye→chest ray to
     the trio's LOAD placements (145.6-145.8, 22.70) dies 0.83-0.90 — the slab edge seen from
     below. The ground CLEAR lines the round-9 ledger names exist only to the SETTLED spots.
   - `probe_0d_ground_wake` (NEW, committed): wake at the drop point + a 700-tick settle, then
     (a) the 14 route-stand matrix, (b) `scan27`, the full 186..216 × −240..−222 grid at 2 u.
     Findings: the settled roof snipers stand at **HEALTH 10 each** (one aimed hit); the grid
     holds **56 CLEAR spots, ALL to g29** (x 204-216, z −234..−240: trench a (210,−240) d 59
     and trench f (214,−245) d 63, stable across two rounds 400 ticks apart); **g27 has ZERO
     ground spots** (her south spawn's clear-ray law is the ladder-top wake dump only); g28 is
     absent from the parked timeline (the run's strip-mouth #10 kill shot is what wakes her —
     she was present at the trench moment in both real runs).
2. **The lever wired and judged on its own real runs** (the legs were `real_damage && difficulty == 1`
   gated; the plain replay and the d2 twin untouched):
   - Run 1 (the trench-a act, cap 450): the leg walked +79 ticks, the census caught the whole
     trio (g27 (151.35,−255.80) 0x97, g28 (151.43,−241.25) 0x94, g29 (151.39,−245.26) 0x93,
     health 10 each), the g29 ray full CLEAR / static clear / bullet_scoped CLEAR at d 59.6,
     **two aimed shots fired — ALL MISSED** ("still alive after 450 ticks (health 10, dist
     60.0)"). #29 died anyway to the ladder-top FIRST pass's crossing catch (+222, the walk-9
     law; her hp 0 at frame 3854). Death **t5132** (the crouched baseline t5532 on the
     re-rolled dice; #27 3 × 16.0 from her south spawn the top killer).
   - Run 2 (+ the trench-f #28 attempt, cap 300, the same measured-blind stand): the trench-a
     act missed again (identical deterministic shots), the FIRST pass caught its guard at
     +222, #16's deck fire landed 32.0, the phase's later acts never ran. Death **t4336,
     WORSE** — the walk-6 act-fire dice law held: every added pre-climb act re-rolls the
     stream lethal-earlier (5532 → 5132 → 4336). 10 hits for 100.0 both runs, 0 return-fire
     fights.
3. **The legs are REMOVED** (the #18-leg precedent: the net is NEGATIVE) with the measured-dead
   law in the route comment; the route is the walk-9 shape again.
4. **The audit run** (the reverted baseline + `NF_0D_AUDIT=1`, read-only): death **t5532
   EXACTLY** — the walk-9 baseline restored deterministically (the revert verified). The audit
   confirmed the ladder-top torso bullets LAND on the trio (the g29 crossing kill: SPAWN part
   Some(4,0.836) world CLEAR, DEAD trv 52.2, the world stop BEYOND her). So the far-target
   miss is **specific to the GROUND up-hill lines** (eye ~1.8 → y 23.0 at d 59-63, a ~21° ray).

## How it was verified

- `walkreal_0700000d_live` whole (final tree): **2 passed** (`walk_arrival_to_the_c5_door`
  — the plain replay — and `walk_arrival_to_the_c5_door_real_damage_agent` — the d2 real-damage
  twin, both byte-identical), **1 ignored** (the real-damage walk with the refreshed ledger).
- `interactables_0700000d_live` whole: **11 passed, 0 failed**.
- The probe binaries: `probe_0d_ray_stands`, `probe_0d_ground_wake` (14-stand matrix + scan27,
  two rounds each), plus the NF_0D_AUDIT walk run.
- **The full four-crate suite did not fit the window.** The sandbox was reset before this
  session (cargo, 7z and the ISO were gone), so the setup + the release build consumed most of
  it, and background jobs die between tool calls here (the suite runner cannot survive in the
  background). This branch touches ONLY two nf-view test files
  (`crates/nf-view/tests/walkreal_0700000d_live.rs` + the new
  `crates/nf-view/tests/probe_0d_ground_wake.rs`) — no `crates/*/src/` changes, no
  `crates/nf-view/tests/common/` (shared-helper) changes — so the other levels' walks and the
  engine crates are untouched by construction. The two binaries the task names were run whole.

## What's left (the ignore message's next levers, in order)

1. **The trench-a bullet audit at d 60** (`NF_0D_AUDIT=1`, read-only, no draws — the walk-8
   tool): re-add the trench act under the audit env in a PROBE-STYLE run (never a shipped leg)
   and trace the two shots' flight. This session's audit already proved the ladder-top torso
   bullets land, so the suspect list is narrow: the flight's per-tick START-CEL re-feed
   [M Bullet_update 0x12f0cc-0x12f0d8] crossing multiple cels on the steep up-hill ray, vs the
   posed boxes the aim gate sees past. If the stop is the port's boxes-only refine missing the
   origin-centred two-sphere capsule [M NDrone2_Collision 0x140e90:91-121], the fix
   un-act-proofs the WHOLE far-target family (the trio from the top, #10 in the bay, #17's
   band).
2. The port-side capsule fix itself [M NDrone2_Collision 0x140e90:91-121] (its own task).
3. The yard levers (the fast first tap, the girder sweep, walk-5's wired set).

## Live checks for the main agent

- Watch one trench-a shot's flight in PCSX2 (fresh boot, the same stand (210.6, 1.1, −238.7),
  aim at g29 (151.4, 23.0, −245.6)): does the real game's bullet reach the box? If the real
  game lands the d-60 up-hill shot where the port's dies, the port's flight path is the
  deviation and the trace will show the cel where it stops.

## Usage

Token, context-window and cost figures for this session: **not visible to me** — this
environment exposes no token counter, context gauge or cost readout. Nothing estimated.
