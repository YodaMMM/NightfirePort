# level-0c-east-climb — the report

Branch `s/level-0c-east-climb` (from origin/master 3b61dac5). Task: `walkreal_0700000c_live` still
places Bond at `[95.0, 11.3, −209.5]` for the east climb — find the game's route to the east band
deck (10.34 at (95, −209.5)), walk it on real input and delete the write; or, if the data shows no
route, pin that with a sister assert and name the live check that settles it.

## Verdict: the deck is a collision ISLAND — the data shows no route

Every candidate connection was enumerated and killed on the disc's own data (all [D] via the port's
parsers — the collaudit probe already proved the parse equals the disc's declared collision; this
session's committed probes `walkreal_probe0c_eastclimb5/6/7_live.rs`):

1. **The turb4 stair chain is broken twice.** staircase03 (plc37, the long lower flight 0.16→7.19)
   and staircase02 (plc36, 6.35→8.10) top out at x ≤ 81.7; staircase01 (plc35, the upper flight)
   starts at 9.45 at x 83.8..84.4 and rises to 11.18 at x 85..87.6. The slot between them
   (x 81.5..83.7, z −211..−209.5) is EMPTY from 9.3 down to 6.5 (the fine map: the columns read the
   lower flight's steps 1.75..3.75, nothing between). Crossing needs a ~2.4 m jump that GAINS 1.35 —
   the jump apex is 0.78 [M player.rs, 3.92 m/s up]. Staircase01's own bottom (9.52..9.68 at
   x 84.2..84.5) floats over the under-stair steps' 6.35 — the already-pinned sister 5.
2. **The band strip gaps 10.4 m over the chasm.** The west band (obj_turb2_walkway01, 10.36) ends at
   x 56.08; the next deck-height floor east is obj_turb3_walkway04 (10.37) starting at x ~66.5. The
   [L 2026-10-30 fresh boot] already pinned the walk-off drop at x 63.4.
3. **The north-strip mezzanine breaks.** The room floors at 10.85 flank the band's north edge
   (z −206.0..−205.1) from x 47..53.5 and resume at x ~64.5 east — nothing spans x 54..58.5
   (6.17 ground below). room_turbine02's 10.17 sliver [58.9..63.3, −206.1..−205.1] is itself an
   island (5+ m to the west patch; the flood leaves it unconnected even at step law 0.80).
4. **The deep-south approach dead-ends.** crates4's tops are 7.31 (z −232.6..−230.4) and 8.31
   (z −236.1..−232.6) [D eastclimb6 plc99]; obj_turb5_walkway02's south end (10.37, z −227.26) sits
   3.1 m north with a 2.06 rise — beyond the apex.
5. **The flood-fill verdict.** Over the sim's own collision, 0.25 grid, ~33k columns, directed edges
   (same-level both ways, step-up ≤ law upward only, drops one-way with a knee-height wall check):
   the gate deck + staircase01's deck form ONE component, unreachable from the east ground, the
   y-0 yard, the west band, the corridor (11.09), the deep gallery and the catwalks — with the step
   law at 0.55 AND at an absurd 0.80 (above the 0.78 apex), so the pin does not hinge on the exact
   law. The east ground + yard + west band + the ground under the gate DO form the walked ground
   component — matching the eastroute walk.
6. **No mover serves the island.** No lift kind exists on 0c [D kind census]; the 8 SSs (#138/#179
   ch 18, #180/#184 ch 84, #351/#364/#377 the ch-0 contact-sway winch props with no translation
   keys, #544 Door_secret1) are all west-side [D]; the 2 doors are door2turbine (west) and
   lockeddoor #555 (far off-map) [D]; the 2 ScriptPlayers are the warehouse forklift loop (x −5)
   and the shuttle exhaust — scenery, "no channel events" [D audit §6 the script dump]; the only
   ladders are #164 (9.67, 5.53, −187.34) and #495 (−6.38, 0, −177.55) [D]; all five graple_pulley
   points sit west/north (#308 61.1,−143.5; #346 77.4,−144; #500 −4.6,−189.7; #508 39.4,−221.8;
   #511 54.3,−237.0) [D]. The turb2_pulley tower's crossbars (x 62.2..62.4, 0.2 wide, rising
   9.3→12.85 southward) start 3.1 m above the west ground and end 15+ m south of the band strip —
   unreachable start, useless top.
7. **The game's own nav agrees.** The 0c nav graph patrols the east band (Path_G3, 10.34,
   x 76.9..94.2) and the east ground (Path_G4, 6.17, x 76.7..93.5) as SEPARATE paths with no
   linking legs [D the 0c nav dump, 27 paths].

## Test changes (`crates/nf-view/tests/walkreal_0700000c_live.rs` — the only test touched; no src changes)

- **Sister 5b**: the stair slot is void (no surface 9.3..6.5 at (81.75/82.25/82.75/83.25, −210.0))
  and staircase02's top step reads 8.10 at (81.35, −211.25) — if either changes, the climb reopens.
- **Sister 5c**: the three remaining crossing attempts pinned as voids — (a) band-height floor over
  the chasm's west half (x 57..65 at −207.5, 8..11.9), (b) the north strip's break (x 55..58.5 at
  −205.7, 9.5..11.5), (c) the crates4 air gap (x 93..95 at −229/−228, 9.4..10.2). A hit on any of
  them is a route reopening.
- The stand-in write `[95.0, 11.3, −209.5]` STAYS, its comment now documents the island pin (the
  stair slot, the chasm gap, the mezzanine break, the crates gap, the no-mover census, the separate
  G3/G4 nav patrols) — the live checks below decide whether the real game has a dynamic the parse
  cannot see.
- The header comment updated to the pinned state.
- New diagnostic probes committed as the evidence trail: `walkreal_probe0c_eastclimb5_live.rs` (the
  placement + up-facing-level census of the east zone), `..._eastclimb6_live.rs` (the fine
  down-ray maps + the room-floor tri dump), `..._eastclimb7_live.rs` (the flood-fill).

## How it was verified

- `walkreal_0700000c_live`: **2 passed, 0 failed** (18.84 s, release, single thread, NF_ISO set) —
  the whole mission walk including the new sisters. **The task's asked-for total: 2 passed / 0
  failed.**
- `interactables_0700000c_live`: **5 passed, 0 failed** (11.10 s). `walkreal_0700000c_eastroute_live`:
  **1 passed, 0 failed** (2.18 s).
- The walkreal_0700\* mission-family sweep (25 binaries, release, `--test-threads=1`), totals
  quoted: walkreal_07000001_live 5 passed / 0 failed; 07000002 1/0; 07000003 4/0; 07000004 7/0;
  07000005 10/0; 07000006 3/0; 07000007 2/0; 07000008 3/0; 07000009 4/0; 0700000a 7 passed / 0
  failed / 2 ignored; 0700000b 3/0; 0700000d 1/0; 07000011 1/0; 07000011_route 1/0; 07000012 1/0;
  07000013 1/0; 07000014 6/0; 07000015 1/0; 07000016 8 passed / 0 failed / 1 ignored; 0700001b
  9 passed / 0 failed / 1 ignored; 07000041 5/0; 0700004a 1/0; 0700004a_satchel 2/0 — **all green**.
- The full suite (tools/test/suite.py): **no total line this session.** The background fast runner
  was started three times and died at tool-call boundaries each time (this sandbox kills background
  jobs between calls; the first two died on the nohup PATH/alsa env before the build finished, the
  third during compilation). The foreground window went to the mission sweep above. No src changed
  this session (only the 0c test file + new probe files), so the sweep covers every binary the
  changed file shares code with; the next session should run the full suite on the merged tree and
  quote the total.

## What's left

- The full-suite total line on this tree (merged with origin/master).
- The live checks below — they are the deciders for the stand-in.

## Live checks for the main agent

1. **The east climb (the decider).** Fresh boot (no save state), level 0700000c, put Bond on the
   east ground (80.63, ~7.1, −207.7 — the G4 node) or at the gate's ground spot (95, 7.3, −209.5),
   then: (a) fieldprobe screenshots from `--bond 84.0,7.3,-208.4,0.0,3.1` (staircase01's west face
   and whatever stands above it), `--bond 86.0,7.3,-209.0,0.0,3.14` (the stair bottom and the
   band's south rail gap), `--bond 94.0,7.3,-209.5,1.57,0.0` (the tunnel under walkway02 looking
   south) — if the game shows a stair/lift the port's parse misses, gdbwatch the contact cel
   (HITDATA+0x54) while pressing into it to name the dynamic; (b) try every direction from the
   gate ground spot and record whether the body EVER reaches deck height 10.34. If the real game
   cannot climb either, the island pin stands and the stand-in stays forever, documented; if the
   game escapes, the escape names the missing dynamic and the stand-in can die.
2. **The band patrol spawn (a cheap cross-check).** The G3 drones (plc58 (94.16, 11.42, −223.20),
   plc59 (94.68, 11.42, −210.25)) spawn ON the island at boot. A fresh-boot watch of one drone's
   patrol (fieldprobe on its position) shows whether the game's AI ever paths it OFF the band —
   if a G3 drone walks down to the ground (or a G4 drone walks up), the connection exists somewhere
   the static parse misses, and check 1's gdbwatch should hunt that edge.
3. **The gate's own trigger height.** The gate quad is y 11.25..13.98 at x 96.92 [D plc556] and the
   room05 floor east of the deck reads 10.85/11.10 [D eastclimb6 plc11]. If check 1 finds any way
   onto the deck, confirm the gate fires from the deck walk (the port already pins this from the
   stand-in) — no extra work expected.

## Usage

not visible to me (no token, context-window or cost figures are shown in this environment).
