# level-0d-yard-sniper-los — the 0d yard sniper LOS

Branch `s/level-0d-yard-sniper-los` from origin/master (e79e7bc9). Task: un-ignore
`walkreal_0700000d_live::walk_arrival_to_the_c5_door_real_damage` — drone #31 (placed sniper,
weapon 0x1e) was said to have "an unbroken line of sight to every cell of the yard", so Bond
died at tick 6549.

## Status: not finished — the ground leg is fixed and verified, the climb exposure is not

The real-damage walk is still `#[ignore]`d. It now dies much later (tick 2988, on the wire walk
past the ladder top) instead of tick 6549 in the yard, and the ground traverse itself is
zero-hit. The non-damage twin passes the full mission on the new route (20984 ticks, 0 stuck,
0 falls, exit 0x730000d). The full suite was NOT run (deadline).

## Do 1 — the occluder verdict: no port bug in the LOS ray, mask or cel gate

Every [M]/[D] check the task asked for came back "the LOS really is open":

- **[D] the corridor has no occluders.** Every placement in x 150..225, z −222..−272
  (probe_0d_los_grid part A) is plants (`sunnyplant-*`, no collision), AI cover nodes, the
  wire/ladder/grapple hardware and the crane scripts. The only collision-bearing object near
  the walk is the forklift #371 at (219.88, 0, −230.42) — the east edge. The shipping
  containers are `obj_ship1/2_volumetric` #6/#7 — kind 0x3B, **no collision in the parse**, at
  the origin; their beam/castor instances (0x19, with collision) stand elsewhere. The concrete
  blocks #89/#90/#91 (0xD8, not ported) are in the ST-3 yard at (133–146, 11.4, −304), not in
  this corridor. No container, crate, crane or wall stands between the perch and the walk.
- **[M] the port's ray is a superset of the game's.** The drone sight call
  (`Drone::seek_opponent_los` → `world.line_of_sight(.., LOS_GUARD_VISION)`) runs
  `segment_clear` (LOS_MASKS_ON=false, world.rs:2472) over stat + missile + dynamic geometry
  with NO drops. The game's `Collide_LineOfSight` with mask 0xa27 [M 0x176f18:69/95] DROPS
  more: materials 0xd/0xe (bit 0x1), 0x10 (bit 0x2), the 0x40-material bytes (bit 0x800), the
  node-flagged hits (bit 0x8) [M Collide_Intersect 0x1ebaa8:300-319]. A ray that is clear in
  the port's stricter walk is clear in the game's.
- **[M] the cel-graph pre-pass passes here too.** `AINetwork_TestRayCels` 0x15ab78 returns 0
  (blind) when either cel is 0, else BFSes `Collide_StraddleCels` 0x1eac98 over the portals the
  ray straddles (`Intersect_Portal` 0x201 path) and requires the target's cel. The port swaps
  this for the world ray ([S], the comment at drone.rs:17729 documents the deviation). Measured
  with the new `world.debug_ray_cels` diagnostic (the same straddle BFS over the port's
  disc-derived scope graph): the perch at (161.11, 7.45, −235.15) and **every** ground cell
  x 180..216, z −224..−268 sit in the same cel 6 — lo (116.9, −6.7, −268.7), hi (370.2, 100.8,
  −191.9) — so the BFS returns the start cel and the gate passes for all of them
  (probe part C: `target_cel Some(6) bfs 1 reached true`, 60/60 cells).
- **[D] 0d has no AI volumes at all** — zero 0xF5/0xF8 placements (`aivolume::build` → 0
  volumes), so the shadow-volume visibility factor is 1.0 everywhere; nothing there either.
- **[D] drone #31 = oDrone placement #465** @(161.11, 6.93, −235.15): skin 0x5000087, weapon
  0x1e, standing guard with NO patrol points (params 3:0x0 4:0x0 — one of a trio with #360 and
  #472), sight_range 1000, half_fov π/2, idle yaw ≈ 3.02.

Conclusion: the port's LOS equals the game's here — open. That is Do 2's case.

## The real killer was not the LOS — it was the route (Do 2, done)

The reproduction (NF_DMG_TRACE on the ignored walk, master route): Bond was **pinned 1814
ticks** at (186.32, −3.36, −265.94) against the deep pit's east wall and shot 6 times while
stuck. The pit map (probe_0d_trench_pit_map): the deep pit (floor −4.39, x 180–186,
z −268..−258) ends in a 4.4 m west-facing wall at x ≈ 186.9 (every ray y −3.4..−0.9 hits it at
fraction 0.136) — the leg's own waypoint (190, −3, −266) is on the far side and unreachable;
the walker's mitigation ground against it for 30 s. All 9 "hits" were drone #31 (3 misses on
the moving Bond, 6 into the pinned one); Bond took ZERO damage while actually walking.

The new route (all real input, no health writes):

1. **Leg 0 — the level's own armour.** `PICKUP_BodyArmourHeavy` #525 (180.64, 11.18, −208.82
   [D placement]) stands 3.4 m from the start, on the platform the walk already crosses. Taken
   on the walk-over (+50 armour [M Pickup_Handler 0x18e650: type 3 refused while armour ≥ 50,
   else hard-set 50]). Verified: probe_0d_armour_timeline prints armour 50.0 through the drop
   and the whole strip.
2. **Leg 1 — the blind strip.** The yard's real cover is the perch sniper's own FOV wedge: the
   strip z −224..−230 across x 178..218 reads visibility 0.0–0.67 < 0.7, so the game never
   even probes LOS there [M DroneVision_ProcessDroneSight 0x176a28: the LOS round-robin only
   reaches drones with visibility ≥ 0.7; DroneVision_OpponentVisibility 0x175af0's cone
   (half_fov + the 0.3927 pad) × range]. probe_0d_vis_field_east_strip maps it cell by cell.
   The walk now goes drop (181, −227.5) → (188/196/204/212, z −228..−230) → east of the crane
   leg (215, −233.2) → the ladder base (213.2, −234.8) grabbed from the leg's east side (the
   climb 862 ticks vs 1674 from the west approach). The ground traverse: **zero hits**
   (was: 89.2 damage).

## What is left

- **The climb + wire-walk exposure.** After the strip, five snipers see the climbing Bond:
  #27/#28/#29 (the west roof, y ≈ 22.7, x ≈ 151), #16/#33 (the dock, y ≈ 13.7), #31 (the
  perch). ~7–10 shots ≈ 100 damage over the climb and the first wire steps vs 100 health +
  50 armour — the real-damage walk dies at tick 2988 at (210.1, 16.2, −240.1). The crane leg
  does not shield a climber from the HIGH west-roof rays. Next message's levers: the port's
  `bond_view_bones` during SS_CLIMB (does the game's climb pose present smaller/lower bones —
  or hide behind the leg — where the port's chest point hangs exposed?); whether the game's
  aim-stand cadence really lets five guards all open fire on one climber; whether the
  Zipline #484 (187.97, 9.85, −219.12 [D]) offers a covered alternate. Only then: un-ignore,
  run walkreal_0700000d_live, the drone LOS/sight tests, every real-damage walk, the suite.
- The full suite has not run on this branch at all (deadline). The non-damage twin and the
  probe builds are green; nothing outside the 0d walk + the probes + the one diagnostic fn
  changed, but the suite must confirm the shared-file rules before merge.
- `world.debug_ray_cels` is a new pub diagnostic on CollisionWorld (one-line wrapper over the
  private straddle BFS); the probe file is assertion-free and `--ignored` only.

## Live checks for the main agent

1. **The blind strip (the core claim).** PCSX2, 0700000d, warp Bond to (196, 1.0, −228.5) and
   walk x 188→212 along z ≈ −228.5 with `fieldprobe`/watch on drone #31 (placement #465,
   (161.11, 6.93, −235.15)): expected — its +0x228 sight bits stay clear (visibility < 0.7, no
   LOS call), no shots while Bond is on the strip. If the game's drone DOES acquire on the
   strip, the port's cone math (half_fov π/2 + the 0.3927 pad) is wrong and the strip routing
   needs a second look.
2. **The ladder climb under fire.** Manual climb of ladder #435 (212.47, 8.14, −234.51) from
   the east side; gdbwatch the +0x228 bits of the west-roof snipers (#27/#28/#29 at
   (151.0–151.4, 22.7, −237..−256)) during the climb. Expected if the port is faithful: they
   acquire and fire (the port's climber eats 3 hits from #28). If the game's climber is NOT
   fired on from the west roof, the port's climb-pose view bones are the bug.
3. **Renders (--bond).** (a) `--bond 196,1.0,-228.5,1.57,0` — the blind strip mid-point: should
   show the dock's south face overhead/east and the open yard south (confirming no geometry
   covers the strip). (b) `--bond 161.1,7.5,-235.1,1.4,0` — the perch drone's eye view: should
   show the whole yard floor unobstructed to the east (the open-LOS verdict, visible).

## Usage

not visible to me
