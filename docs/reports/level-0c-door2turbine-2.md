# level-0c-door2turbine-2 — the report

Task: walk the walker from the SE walkway to the door2turbine E stance for real on 0700000c.
The prior report left Bond balancing on an unexplained ≈13.05 surface at x 40.45..40.6 whose
owner was not in the static census; find it [D], fix any port collision gap [M/D], then
replace the stand-in with the real-input leg (corner E opens #281, the slot jump onto
(44,−209), the band walk), then blockers 2-5 of level-0c-every-objective in order.

## The 13.05 surface mystery — solved, no port gap

There is no 13.05 surface. The probe (`walkreal_probe0c_door2_live`, committed) sweeps down
rays y 15→11 at x 39.85..41.4 over z −218..−213.6: the only static floor is the walkway strip
at **12.02** (x 39.85..40.41); east of x ~40.45 nothing exists up to y 15. The prior session's
"balance at 13.05" was **Bond's body centre standing on the 12.02 slot floor** (12.02 + body
half-height ≈ 13.05; the same centre-over-floor shape as every other stand-in landing — the
pad stand-in's 12.93 over the 11.84 pad floor). No drone, no dynamic object, no missed mesh
(`sim.drones.list` sampled every 30 ticks around the balance spot: nothing within r 2.5).

What the disc actually has there [D, the probe's tri dump of room_turbine01 (plc 8) and
room_ex10 (plc 116)]:

- The SE "walkway" strip x 39.98..40.41 is a **0.43-wide CEILINGED slot** — floor 12.02, west
  face x 39.99 rising to y 14.24, ceiling 14.22 — built as TWO closed boxes (z −224.98..−219.80
  and −218.69..−213.51) with a 1.11 gap between (z −219.80..−218.69) and a full end wall at
  z −213.51 (y 12.02..14.22). The north box is a dead end: west wall, south wall, north wall,
  ceiling, open east into the shaft void only. The prior stand-in (40, 13, −218) spawned INSIDE
  that dead-end box; no walk route can leave it (the old route's wedge).
- The **west ledge** is the real route: floor 11.61 at x 38.55..39.55, running the WHOLE
  z −222..−213.0 (not just the z −214.5..−213.0 "shelf" the old map saw). room_ex10's east wall
  (x 39.71..39.98, y 11.61..14.57) separates it from the slot only for z −219.26..−213.25 —
  SOUTH of −219.26 the ledge is open to the slot's south box: a 0.41 step down west.
- The ledge's north end drops 0.52 onto the **11.09 corridor** (x 38.05..39.55, z
  −212.75..−212.0), which runs to door2turbine #281's mouth (leaf x 38.49..40.02, z
  −210.56..−210.36).

So: **no port collision gap** — the port's world matches the disc tri for tri here; the old
route was simply the wrong leg of the geometry.

## The real-input leg — done through the slot jump

`walkreal_0700000c_live::walk_the_whole_mission_from_the_start_to_the_0d_handoff`, all
`step_bond` input, on this branch (commit 118246d5):

1. The south-gap stand-in (still a stand-in — see Left) now lands ON the west ledge
   (39.2, 12.85, −217.6) instead of inside the dead-end slot box; the ledge landing is
   asserted (grounded, centre ≈ floor + half-height).
2. REAL: the ledge north (two waypoints, z −215.0 → −213.3), the 0.52 drop onto the corridor,
   the corridor north to the door mouth (38.87, 12.17, −211.49 — standing on 11.09).
3. **The REAL E press opens door2turbine #281**: facing the leaf from the corridor mouth,
   pulsing E every 6 ticks — the door leaves CLOSED (0x2) [M Door_Activate 0x135be0:
   info+0x68 (ch_lock) == 0 skips the channel gate; kind 0 (obj+0xf6) closed (obj+0xf4 &
   0x12 == 0) → Door_SetState(obj, 8)], and when the swing completes the door's ch_open 5
   turns ON [M Door_SetState 0x135618: `switch_channels[info+0x66] = (old obj+0xf4 & 0xc)
   != 0` — the OPEN(4) transition with the old OPENING(8) writes true]. Both asserted
   (state OPEN/CLOSING — the door's auto-close [D audit] may already be running — plus
   `channels.on(5)`). The prior session's "state holds 0x2" blocker is dead: it was the
   route, never the door code.
4. REAL: through the doorway onto the ring SE corner (the port's kind-0xDB doors carry no
   leaf collision — the game picks doors through the ForcedList [M build_LinkToRoom
   0x1c9680:63-66, Collide_Pick 0x1eb320:369-375]).
5. **The slot jump landed on the band**: the corner's east boundary is a wall (x
   40.01..40.37) with a 1.52-wide doorway tunnel (z −208.93..−210.45, floor 11.10, ceiling
   13.96 [D tris 478-485, no x-face tris — open along x]); the walker runs up west of the
   wall, through the tunnel, and jumps east of it (fired grounded at x ≥ 40.42, full run
   speed) — the arc crosses the x 40.8..43.3 void and lands on the band lip [(43.4, 10.89) /
   (44.2, 11.17) at z −209.0 [D map]]; observed landing (44.84, 12.18, −208.80), asserted
   grounded + floor under Bond in 10.2..11.7.
6. REAL: the band east — waypoints (50, −207.6) ✓ and (58, −207.6) ✓ arrive; **the walk then
   wedges between x 56.5 and 66** — see Left.

`cargo test --offline --release -p nf-view --test walkreal_0700000c_live -- --test-threads=1`:
the leg runs green through "the band east 2"; the single remaining failure is the band x
58..66 wedge (the harness's 6000-tick cap after Bond drops off the band's unprobed stretch).

## The band gap — the one route item left, with its evidence

The probe's band west map [D, down rays y 12→8]: the band floor 10.34 runs x 44..53 (z
−209..−206.5) and resumes x 67..69+ (the old test's x 70..96 stretch was always real); **x
55..65 is void at every probed z (−209.5..−206.0)**. A 10.85 ledge strip runs at z
−206.0..−205.2 (x 47.5..53 AND x 61..69 — a second gap x 55..59), and north of it the ring's
east walkway descends (8.98 at z −203.8, 8.52 at z −202.0, x 61..69) toward the north-east.
How the game crosses x 53..67 is NOT pinned: either a mechanism the port's static world lacks
or a connection further north/east than this session's sweeps (which stopped at z −198.8,
x 69). The old header's "floor 10.3 at x 46..96" was an over-generalisation from sparse
probes — corrected here.

## Blockers 2-5 of level-0c-every-objective — not started

This session spent its window on the door district (the driver's primary). Blockers 2 (plate
#274's display-case cell), 3 (display → warehouse descent), 4 (the ring west/south gaps — the
south one is now known to be the #283 wire ride onto the west ledge), 5 (the doorway-ledge →
yard descent) are untouched; the probe and route evidence above feeds 4 directly.

## How it was verified

- The probe binary `walkreal_probe0c_door2_live` (committed) reproduces every [D] number above
  from the ISO in one run (`--nocapture`): the parapet/slot top profile, the west-face height
  profile, the placement attribution (y window widened past the old scan's 13.0 cap), the
  shelf/corridor fine map, the dynamic-object check, the slot map, the south-west channel map,
  the east-wall profile, the room_turbine01/room_ex10 tri dump, and the band west map.
- `Door_Activate` 0x135be0 and `Door_SetState` 0x135618 re-read from
  `work/decomp/ACTION.ELF/` this session; the channel-write timing (old state's 0xc bits
  before the store) is quoted from 0x135618 and matches the port's `door.rs set_state`.
- The walkreal leg was iterated on the rebuilt binary until every step landed (the transcript
  quotes above are from the final run: door2turbine "state OPEN, ch 5 on", the slot jump
  landing at (44.84, 12.18, −208.80)).

## The full suite

The fast runner (`tools/test/suite.py`, NF_ISO set, NF_SILENT=1) was started in the background
at 05:42 UTC on this branch (origin/master f1791382); its build phase recompiled the merged-in
changes (**895 test binaries in 1107 s**) and the run is serial with a 600 s per-binary limit.
At the last read (06:28 UTC) it had completed **325 of 895 binaries with 1838 passed, 6
failed, 63 ignored**. The failures so far, with evidence:

- `disc_golden` ×3 — the documented slim-ISO allowance.
- `identity_disc::the_real_disc_passes_the_identity_check` — disc-image identity (the slim
  ISO again, not this task's code).
- `fix_0c_pylon_live` ×2 — wgpu `adapter: NotFound` (this headless sandbox has no GPU
  adapter for a render surface; drawing-only, another task's test).
- `CRASH net_soak_faults_live (exit -9)` — the sandbox's 4 GB RAM OOM-killed it mid-run.

The total line had not landed by this report (the run needs longer than the sandbox window);
`walkreal_0700000c_live` itself had not been reached by the serial order at the last read.
The task's own binary was verified directly as quoted above.

## Left (next session)

1. The band x 53..67 crossing (the probe's band west map above is the starting evidence; the
   descending east staircase at x 61..69, z −204..−202 is the likely continuation).
2. Blockers 2-5 of level-0c-every-objective.
3. The full-suite FINAL total line (the run may outlive this sandbox).
4. Optionally: the west/south gap stand-ins become wire rides if the main agent's live checks
   pin the #282/#283 grabs.

## Live checks for the main agent

1. The band gap: in the real game, walk the band east from the turbine door — record how the
   game crosses x 53..67 at z −206..−209 (a jump from the 10.85 strip, a stair the port's
   static world lacks, or a scripted move). Watch x 55..59 at z −205.2..−206.0.
2. The SE walkway slot: confirm the real player NEVER walks the slot north of the hole (the
   disc's two ceilinged boxes + gap say the ledge is the route).
3. The #283 wire ride (blocker 4): from the south-west arc, jump-grab `wire_silo2turbine` and
   record where the ride drops Bond (the B end (38.03, −217.64) hangs at the west ledge's
   south edge — does the game land Bond on the ledge?).

## Usage

not visible to me
