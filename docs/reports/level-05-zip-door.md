# level-05-zip-door — report

Task: find how the game's player gets from 07000005's cable landing to the zip door #112, port
the missing link, turn `walkreal_07000005_live`'s pinned negative into the positive leg on real
input through the zip line; then 07000006's OPEN LEG (the upper gallery → dressing-room
LoadPoint #746 walked route). Branch `s/level-05-zip-door` (from origin/master ad2f0791).

## What the disc actually carries (the route, decoded)

The old blocker's "the south structures STOP" was a **scan blind spot**: `probe_south05` covered
x −26..50 only, and the wall-hug structures live at x −35..−33. New probes (committed as
examples): `probe_southwest05` (band maps x −44..24 at y0 60/48/34 — the y0 48/34 bands see
UNDER the 39.8/45.5/46.1/51 roofs the old scan mistook for void), `probe_dip05` (the strip
crack at 0.5-unit resolution), `probe_shells05` (the plunge-shell dump).

The full chain from the wire landing to the door [D the band maps + the placement census]:

1. **The landing → the south piece**: the landing's south rail is an L (the x −23.8 lane stops
   at z −2, the x −24.5 lane runs to z −8) [L the wedge runs]. WALKED green.
2. **The strip crack**: z −11.5..−12.5, floor 23.5-28 — a 1.5-unit crack, not a pit
   [D probe_dip05]. HOPPED green (+20 ticks, real jump input).
3. **The approach**: z −16..−19, floors 32.2-36.6, continuing to the east strip at 34.3
   [D the fine maps]. WALKED green.
4. **The wall-hug band** (x −33..−22, z −22..−36, walk ~36.5-37.6 under the 39.8/45.5 roofs —
   ai point #259 (−23.41, 36.51, −31.57) and the guards #65/#68 (feet 37.6) stand on it
   [D placements]): the game's own route plates #311-#314 (out 127/128/129/133) and the alarm
   plate #315 (out 0xA4, at y 37.23) ride it; the crawl prompts #84/#85 (type 6) flank the
   window crawl #35 "Wall Hug Window" ((−34.10, 37.28, −31.65), band z −26.3..−37.0 [D
   placement; M CreepWall_Create 0x196fc8, Player_Activate 0x1a5170:66-97 → Player_CollCreepWall
   0x196758, Player_Creep 0x1969d0]).
5. **The roofed corridor** (x −36, z −38..−44, floor 32.5-36.2 under the 46.0 roof), a second
   crack (z −46..−50), the south gallery (z −52..−60, 31-33.7), the **SW loop** (the lower path
   x −39..−41 z −64..−72, the west walkway x −41..−40 z −70..−76 at 37, the z −78 band x −38..−27,
   the z −76 spine) to the west battlement (x −28..−16, z −72..−74, 36.7-37.5) — plate #269
   "SWITCH_Helicopterstart(SC3)" ((−22.76, 37.15, −72.36) out 0x42) fires en route [D].
6. **THE MOAT INLET** (x −15..−5, floors 20-23 — a 14-unit cut through the wall-walk) is crossed
   by **THE Q-CLAW GRAPPLE**: pulley #281 "graple_pulley" ((−3.40, 42.94, −71.77) with its
   "Grapple_from bottom" CEL #280) stands 17.6 units east of the west battlement's east end —
   inside the claw's bolt range 50.0 [M 0x12e7b0:158 + weapon_data[0x50]+0x1c]; the reel
   [M Player_SSGrapple 0x192860: phase 1 pulls 0.05→0.5/tick, phase 2 hangs within 2.0, the
   fire release detaches] carries Bond over the inlet onto the east battlement (x −4..+4,
   floors 36.3-36.9). The claw is owned at the level start [L inventory.owned[0x50];
   gadget_grapple_live], the route's grapple icons #277/#300/#310 (type 3) mark its legs [D].
7. **The door**: the east battlement east through the trough (x 5..8, 30.2-34.2), the rising
   stairs (x 9..24, 37.0→40.4) to the y-49 door band (x 16..36 at z −68 [D]), door #112 E →
   fallback ch 0x1E → 07000006 [M Trigger_Activate 0x1b2318:91-107]. The door guards #271/#272
   and the ammo #322/#323 (y 49) stand past it [D].

The zip line itself: wire #251 is genuinely hand-over-hand [M Player_CollWire 0x197130 reads
rec[0] = p1 for the type; #251 p1=0 → SS_WIRE — the port's mapping matches], and the ride's
drop is the action press [M Player_Wire 0x197748-0x197834] — the walk pins that drop now.

## What was walked (real input only; 9 passed / 0 failed, 69.5 s)

`cargo test --offline --release -p nf-view --test walkreal_07000005_live -- --test-threads=1`:
**9 passed; 0 failed; 0 ignored**. The rewritten leg
`the_cable_landing_walks_the_south_walkway_toward_the_zip_door` walks from the fresh cold start:
the whole cable route (the slab band, the gap, the notch grab, the strafe ride), the wire's
action-press drop [M 0x197748-0x197834], leg 1 past the L-rail, the crack hop, the approach to
z −19.6 — then pins the band's north seal (below). The sister negative
`the_notch_holds_the_walk_and_the_grab_needs_its_zone` stays green; the old stall-pinning
negative is superseded (its "best z never crosses −10" held only because the walk hugged
x −23.5 — the L-rail's west lane crosses z −8 cleanly).

## The one remaining blocker (pinned, with evidence)

**The band's north seal at z −20.5.** Every lane from the approach onto the wall-hug band is
closed in the port's collision [L the wedge runs]:
- x −23.7: walls at z −19.5 under an overhang lip (the jump bounces at 34.4 under a ceiling
  ~35.3);
- x −31..−32 (the west slope): walls at z −20.45, floor_ny 0.95;
- x −26: tops out on the face's rim at y ~38 with **1.6 headroom** — Bond's 1.86 body cannot
  stand there;
- x −24.3: walls at z −19.45.
The test pins `best_z > −20.0` for the 420-tick seal walk (a [D]-backed negative: if the
collision grows a route the lanes could not take, the assert fires). The band/ledge/crawl/
corridor/grapple/door legs are fully specified in the test's doc comment with their cites and
walk once the seal's game-side answer lands. Two candidate answers, both checkable live:
(a) the game's step/contact rule carries the player over a face the port's reads as a wall
(the port's step arm vs [M Player_StandAtNewPosition 0x190d88]); (b) the entry lane is
elsewhere (the y0-34 band hits at x −35..−34, z −20..−27 — a lower shelf — are covered by the
plunge shell #291's sheet at those cells, so they are death, not route).

**Second finding (a port gap candidate):** the plunge shell #291's sheet passes through y ≈ 30.7
at (−25.2, −10.4) [D probe_shells05's tri dump: tri #6's plane y = 26.85 − 0.1286·(z − 14.86)];
a 0.4-unit lip-dip there fires ch 0xBE → the death NIS (the walk's first version died exactly
there). The game's wall-hug route runs the same ledge safely [D the route's own plates] — the
live check below settles whose contact rule is coarse.

## The 06 open leg (the gallery → dressing room)

Not walked this session (the 05 route consumed the window; the probe for it timed out under the
suite's build contention). `probe_gallery06.rs` is committed (the placements z < −28 + the fine
band map x 0..44, z −80..−28) — it is the first thing to run next session. The test header's
own candidates stand: a stair volume the port's floors miss, a scripted carry, or the ai-point
patrol (ai point 5 at (2.33, 54.67, −52.50) patrols a 9-node path above the tunnel [D ai dump]).

## Live checks for the main agent

1. **The band's north seal**: fresh boot of 07000005, poke Bond to (−24.4, 34.9, −19.4)
   (fieldprobe.py), walk south by input, screenshot at z −19.0/−20.0/−20.6, and watch
   obj+0x30: does the real player step onto the 37.6 band (the game's step rule carries him
   over the face), or is the entry elsewhere? If he crosses, gdbwatch the face's tri contact —
   the port's step arm needs the exact rule [M Player_StandAtNewPosition 0x190d88].
2. **The plunge shell contact rule**: fresh boot, stand Bond at (−25.0, 31.0, −9.5), walk
   south over the crack at x −23.5 and dip 0.5 into the lip at (−25.2, −10.4): does ch 0xBE
   fire in the real game? (The port's shell #291 sheet says it should; the walked route says
   the game does not kill there [D the route's plates].) Watch the toucher build (pos − 0.6)
   and the shell's plane distance.
3. **The grapple inlet crossing**: fresh boot, poke Bond to the west battlement's east end
   (−18, 37.1, −72.3), draw the claw (0x50), aim at the pulley (−3.40, 42.94, −71.77), hold
   fire: the reel should carry him over the inlet (17.6 units) — screenshot at the hang and
   after the release. Expected: phase 2 within 2.0 of the pulley, the release drops him on the
   east battlement (36.6-36.9).

## What's left

1. The seal's answer (live check 1) → then the un-walked legs: the band west leg, the crawl
   #35 (E at the prompt #85), the corridor + the second crack hop, the SW loop, the grapple
   inlet leg (live check 3 pre-verifies it), the east stairs, door #112's E → the exit assert.
   The legs' waypoint chains are already drafted in this session's WORKLOG (the coordinates,
   the guards' heights, the shell margins).
2. The 06 gallery → dressing-room route: run `probe_gallery06.rs`, then the same walk treatment.
3. The fast suite on the final tree: restarted 21:09 UTC on this tree (the first 20:14 launch
   raced a mid-patch build and its BUILD FAILED is that race, not the tree); the touched test
   binary is green (9/0/0 above); the suite's total line lands on the branch's WORKLOG entry
   when the run completes.

## Usage

not visible to me
