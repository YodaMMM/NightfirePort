# level-0d-crane-and-to-the-end — the dock→deck crane crossing is real input; the deck→yard remainder is mapped and blocked only on the jib rim gap

## What was asked

`walkreal_0700000d_live::walk_arrival_to_the_c5_door` walked to the dock's south edge and
TELEPORTED Bond to the yard (140.0, 5.5, −276.0) as the documented crane stand-in. Make the
crossing real ([D] the crane placements/keys, the script player and its channels; [M] the
classes), fix any port gap, replace the teleport with real input, then extend the walk past the
C5 door to the level's end, every objective by the game's own writers, every test in the file
green, quote the totals.

## What this window found (the route, every leg traced)

The master test already carried the post-C5 extension (the photograph, the C5 kill, the corridor,
the ch-100 mission end to 0x730000d, all four objectives asserted state 4 — commit e8caec06), so
the open work was the crane crossing itself. The window's probes
(`probe_crane2..16`, all committed, print-only, `--ignored`):

**The warehouse side ([D] placements + walked):** the spawn platform (settled y 12.19) drops
~11 m off its south edge to the `room_ShippingWarehouse*` ground ([D] the CEL rooms, census;
the fall is survivable — the walker's recovery hop, no damage stop); the ground descends
1.04 → −0.58 → −3.36 south to a wall at z ≈ −268, and rises east to y ≈ 1.0 at the ladder
#435 `ladder2crane1` (212.22, 8.14, −234.51 [D]; sphere r 8.21 → the pick spans y ≈ 0..16.35).
The real walk spawned → the platform edge → the drop → six ground waypoints → the ladder base
is **all real input** and ends at (213.68, 1.13, −235.35).

**The ladder + the wire ([M] + [D]):** `Act::Ladder` climbs #435 to y 17.37 (the top lunge
dismount [M Player_Climb]). The ladder top carries the wire third-icon #297 (213.26, 17.58,
−234.52, p0 = 3 [D]) — `icon_ctx` reads 3 there [probe7] — and with the icon in context a jump
grabs the NEAREST wire within 1000 [M Player_HandleJump 0x1916b0:118-147]: wire #271
`wire_crane2crane` (centre (205.61, 17.04, −248.52), r 14.02, yaw −1.065, **type 0 =
hand-over-hand** [D; the grab snap measured (211.72, 15.94, −237.64) [probe7]]). **Strafe +1 =
mode 7** — the facing target `wire.yaw + 3π/2` [M Player_Wire 0x1977c0-0x1979d0] with the b4/b5
root motion (+z local, spans +0.7637/+0.7105 per cycle [D; the tracks ARE on 0700000d,
probe9]) — carries toward the A end (199.78, 17.04, −259.04): the ~24 m ride takes ~1900 ticks,
the tail inside the 1800-tick auto-drop [M 0x197748-0x197754], and the drop lands on the rotate
crane's deck at **(199.25, 14.16, −260.16)** [probe8]. (Strafe −1 deadlocks at the grab end:
mode 8 faces B-ward and the end snap holds — the first ride attempt [probe8 pre-fix].)

**The rotate crane ([D] keys + [M] SP):** the deck walk reaches the control plate #450
`crane_touchplate` (199.53, 13.31, −263.81 [D]); the E writes ch 0xa [D #450 p0; M
Trigger_Activate 0x1b2318:23-33]. SP #492 `ScriptPlayer_CraneRotate` (mode 0x40, mask 1, out
0xa [D]) plays script 0x06000a08 — **750 ticks, stream 0 = the crane glist 02000a9c keyed
61° → −11° (quat y 0.508 → −0.096, t 0..750 [D dump_0700000d_crane_keys])**; stream 1 (glist
02000abb) is parented to stream 0 and inherits the rotation. The port plays it correctly
(clock +1/tick measured, probe15) and the rider on the deck is CARRIED by the rotation
(ground_motion [M Player_FeetOnPoint 0x194188 → Control_InheritVelocity 0x133e88]; a rider at
r ≈ 3 from the anchor drifted (−1.11, +1.17) over 280 ticks [probe15]). One port-side trap
documented on the way: the walker's `Act::Use` presses `step_bond`'s FOURTH argument
(use_key), not the second (fire) — a manual E on the wrong slot silently never fires the plate
[probe11/15].

## The change

`walkreal_0700000d_live::walk_arrival_to_the_c5_door`: the teleport stand-in is REPLACED by the
real-input chain — the platform drop, six ground legs, the ladder climb, the wire jump-grab,
the strafe-+1 hand-over-hand to the deck, the deck walk, the E #450 (ch 0xa asserted [D]) and
the 750-tick swing (SP #492 state 5 asserted [D]) — with asserts on every leg's measured
endpoint. **The stand-in remains for the deck→yard remainder only** (see below), moved after
the swing. The header block rewritten. The set_channel(0x1b/0x1c) line stays: nothing on the
crane route writes 0x1b/0x1c [D — the level's scripts carry no channel writes at all, census
scan] and the warm record-11 carry is the game's own [M 0x182270:92-95].

**The file: 1 passed / 0 failed (35.66 s) on the final tree.**

## What is left (the deck→yard remainder, precisely)

1. **The jib walk's rim gap.** The ray-down along the jib's rest direction (anchor (200.48,
   −262.37) toward the rest tip (188.65, −268.92) [probe16]) shows: the cab deck 13.04 at
   r ≤ 3.5, a **surface GAP at r ≈ 5** (nothing down to y 0), a 12.13 surface at r 6.5-8, and
   the 18.58 roof slab over r ≥ 9.5 with the dock's north steps under its edge at r 13.5
   (12.67 → 11.92 → 11.13 → 10.35 → 9.56). Walking from the deck toward r 8 drops Bond through
   the gap to the ground (probe16's ride leg) — the gap needs a jump (the walker's Act::Jump
   mid-route) or a different approach line.
2. **The tip→dock delivery.** The swing's tip sweeps counterclockwise 72° — from over the
   trench at (188.65, −268.92) across the dock's north edge (the tip at bearing ~250° passes
   (195.9, −275.1) at mid-rotation [D keys + probe16's map]) to the east ground. A rider on the
   jib mid-arc should step off onto the dock (9.5) around rotation −5..−30° (t ~500-610).
3. **The dock E #460** (`Overhead Crane_touchplate`, 189.02, 9.72, −288.92 [D]) — proven
   separately by `interactables_0700000d_live::crane_e_plate_arms_the_handle_ss_and_the_shipping_sp`
   (ch 0x2b, the beam shuttle [D]); the walkreal needs it in-route.
4. **The shipping-beam boarding.** The beam (glist 02000a80, origin (137.54, 11.33, −287.11),
   local ±4.17/±2.98/±17.01 [D]) shuttles x 137.54 → 177.22 → 137.54 over 3000 ticks [D]; the
   ride carry works (the merged `probe_the_ride_carry`) but the boarding is unproven: the rail
   tops (13.47/14.32) sit ~4 m above the dock (9.5) and ~4.7 m below the catwalk (18.98 — nav
   path4 [D], spanning x 147..193 over the beam's z −287.11 line). The step-off-and-fall from
   the catwalk onto the passing beam is the remaining candidate ([S] unverified).
5. **The west ride + the yard drop** at the beam's west extreme (origin x 137.54, over the
   yard), then the existing legs 2-5 run unchanged.

## Verification

- `walkreal_0700000d_live` **1 passed / 0 failed** (35.66 s) on the final tree — the full
  mission including the real crane chain, the C5 kill, the corridor and the 0x730000d hand-off.
- `interactables_0700000d_live` and `walk_0700000d_live` untouched this window; the suite
  covers them (below).
- Every new claim above cites its probe output (the probes are committed and re-runnable with
  `--ignored --nocapture`).

## Suite

`tools/test/suite.py` (NF_ISO on the slim ISO) was started in the window's first minutes and
ran the branch tree as of 20:11 UTC (before the final walkreal edit; the edit touches ONE test
file, re-run individually above). At the time of writing it is still in the nf-view binaries;
failures so far: the 3 `disc_golden` tests + `identity_disc` (the slim-ISO family — the
expected ones) and CRASH (exit −9) on `net_match_end_clients_live`, `mp_scenarios_live`,
`mp_online_failures`, `net_soak_faults_live`, `mp_remote_fx_live`, `mp_16_slots_live` — the
exit −9 is the sandbox's memory pressure while the window's probe builds ran concurrently; the
previous window's suite.log shows `net_soak_faults_live` passing in 5 s in isolation. These six
need a clean re-run (no concurrent builds) before merge; no source change this window touches
them. The final total line will be appended to the WORKLOG entry when the run completes.

## Live checks for the main agent

1. Fresh boot 0700000d: from the ladder #435 top (213, 17.4, −234.5), does the game's jump grab
   the wire and does the hand-over-hand ride travel A-ward on strafe RIGHT (the port's mode 7
   reading) — i.e., which strafe direction moves Bond toward (199.78, −259.04)?
2. On the rotate crane's deck after E #450: does the game carry the rider through the full 72°
   swing, and can the player walk from the deck onto the jib across the r ≈ 5 gap (jump?), or
   is the rim gap a real hole the route must jump?
3. Where does a jib rider disembark onto the dock — and from the dock, what does a real player
   do to board the shipping beam (the catwalk step-off? the trolley slot?)? Watch Bond's
   position through the beam's 3000-tick shuttle.

## Usage

not visible to me
