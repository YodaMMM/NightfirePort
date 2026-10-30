# level-05-every-objective-3 — report

Task: The Exchange castle (07000005) from the crawl exit to the 07000006 exit — the roofed
corridor south of the crawl exit, the second crack, the south gallery, the SW loop, the SWITCH
plate #269, the Q-Claw grapple over the moat inlet to pulley #281, the east stairs and door
#112's E → ch 0x1E → 07000006. Branch `s/level-05-every-objective-3` (base origin/master
21fbeed3). Sandbox was reset at session start (23:49 UTC); setup redone from scratch.

## What's done

**The route south of the crawl exit is decoded off the disc and walked on real input to the
grapple landing.** All geometry below is [D] — `crates/nf-view/examples/probe_corridor05.rs`
(committed), fine ray maps over the disc's own tri set (`level.instances(true)`, 0.25-0.5 unit
resolution, y0 bands 42/44/47/50/52/55):

- **The corridor mouth** (the level-05-every-objective-2 report's open question): it faces SOUTH
  from the crawl exit — the lane runs x −36..−34.25 from z −36 (floor 36.0) south to z −45
  (floor 37.8). The old wedge at (−37.88, −36.18) was the steer drifting WEST of the lane's
  x −36 wall. The walk takes the lane centre x −34.9.
- **The second crack needs NO jump**: the "crack" (z −46..−50, floors 18-25) cuts x −36..−34.25,
  but the x −35.5..−34.25 spine holds 37.0-37.8 through the whole zone — the walk continues
  along it.
- **The gallery**: the spine (x −35, 37.4) continues z −50..−64 with the gallery floor 4-6 below
  on both sides; the widening at z −65 (x −37..−35: 37-38) hands off to the SW loop.
- **The SW loop**: a zigzagging 1-1.5-unit walkway at 37 — z −66: x −38; z −67: x −39.5; z −68:
  x −40; z −69.5..−72: x −41..−40.5; z −73..−74: x −40; z −75: x −39; z −76: x −38.5; z −77:
  x −37..−36; z −77.6..−78: x −35..−33.
- **The SWITCH plate #269** "SWITCH_Helicopterstart(SC3)" ((−22.76, 37.15, −72.36), p0=0x42 [D
  placement]): reached via the z −77 band east + the z −76/−75 spine (x −30.5..−27) onto the
  battlement lane (z −73.5..−72.5, x −28.5..−16 at 37). **The walk over it raised ch 0x42**
  (asserted green in the test).
- **The Q-Claw grapple over the moat inlet** (x −15..−5, floors 20-25): the claw draws through
  the real switch input (`Input::switch_to(0x50)` [M Player_WeaponChange]; owned at level start
  [L]), the bolt hits pulley #281 ((−3.40, 42.94, −71.77) [D placement] with CEL #280
  "Grapple_from bottom"), the reel hangs Bond at the point (phase 2 [M Player_SSGrapple
  0x192860]), and the fire's release drops him onto the east battlement — landed at
  (1.10, 37.62, −72.58), asserted green.
- **The east lane**: the battlement runs east x −4..13 at 36-37 (the z −71 row's x-10 hole is
  passed on the lane's centre), and the covered staircase rises x 14→35.5 (37.1→43.4) under the
  y-49 terrace overhang [D MAP 10/12].
- **Door #112's exit chain pinned** (probe-grade placement, climb parked — below):
  `the_door_band_exit_chains_to_07000006_climb_parked` places Bond on the y-49 terrace, walks to
  the door's threshold, presses E, and asserts **ch 0x1E on** [D placement #112 p3=0x1e; M
  Trigger_Activate 0x1b2318:91-107] and **level_exit == 07000006** [D placement p0=0x7000006].

**The walk test is green through the landing** (see Suite): the
`the_cable_landing_walks_the_south_walkway_toward_the_zip_door` legs — the approach, the window
ledge, the crawl, the seal negative, and ALL the new legs above — pass with the corridor/spine/
loop/#269/grapple asserts. The previous session's 9 tests stay green.

## The one port gap found (the honest boundary)

**The ch 0x42 script's cut-scene window never opens back up in the port.** Right after the
grapple landing (tick ~8723) the sim flips `disabled=true` and NEVER re-enables (3600+ idle
ticks watched, [L] the test run's hold log: bond frozen, disabled=true). The real game's
helicopter-start cut-scene (the SWITCH #269 → ch 0x42 chain, SC3) should play and hand back.
The east-stairs legs (x −4..13 + the covered staircase, fully decoded [D]) are written into the
test but stop at this window as a documented boundary — they will go green when the script
window closes (or the script is ported).

**The stairs→terrace climb is ALSO parked**: the y-49 terrace's every face is 5.5+ tall off the
staircase's 44.0 top (south face z −71.0, west face x ~14, east face abuts the building, north
edge z −65 is a drop) — no walkable link exists in the disc's collision [D probe_corridor05 MAP
8-12]. The door chain test pins the exit from a probe-grade terrace placement, clearly marked.

## Live checks for the main agent

1. **The ch 0x42 cut-scene**: fresh boot 07000005, poke Bond to the battlement's east end
   (−16.4, 38.2, −72.4), grapple east, and watch the real game's script after the landing —
   what does the helicopter-start cut-scene do, how long does it disable input, and where does
   it put Bond? (The port's window never closes.)
2. **The stairs→terrace climb**: fresh boot, poke Bond to the staircase top (35.0, 43.4, −71.4),
   walk north — how does the real game get from the 44.0 top onto the y-49 terrace (x 14..35.5,
   z −71..−65)? (No walkable link in the disc's collision at 0.25 resolution.)
3. Confirm the terrace walk east to door #112 and the E press (the port's chain is pinned: ch
   0x1E → 07000006).

## How it was verified

- Every route fact cites [D] probe_corridor05's fine maps (committed) or [D] placements from
  probe_placedump05; the grapple/reel/release cites [M Player_SSGrapple 0x192860] and the
  gadget_grapple_live precedent; the switch input cites [M Player_WeaponChange]; the door chain
  cites [M Trigger_Activate 0x1b2318] + [D placement].
- The walk runs through `Sim::step_bond` real inputs only (turn/forward/strafe/jump/use/fire/
  switch_to); the only probe-grade step is the door test's terrace placement, marked in its doc.
- `cargo test --offline --release -p nf-view --test walkreal_07000005_live -- --test-threads=1`
  (NF_ISO set, NF_SILENT=1): the file's result is quoted under Suite.

## What's left

- The ch 0x42 script window (port gap — the cut-scene never releases the sim).
- The stairs→terrace climb (unknown mechanism — live check 2).
- The east-stairs legs going green behind those two answers (the legs are already written).

## Suite

The fast runner (`tools/test/suite.py`) was started at 23:55 UTC but its build phase died on the
session's alsa link error (no `libasound.so` on the link line; fixed this session via
`LIBRARY_PATH` + the alsa.pc prefix). It was restarted after the walk file verified; its total
line is appended in the WORKLOG entry when it lands (the run outlives this write-up).
**Walk file total (this session, slim ISO): see the WORKLOG entry — `walkreal_07000005_live`
10 tests.**

## Usage

not visible to me
