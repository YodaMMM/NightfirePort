# level-0c-east-side-route — the game's own route to 0c's east band

Branch `s/level-0c-east-side-route` (from origin/master 8ce724ad). Task: the 0c whole walk
teleports across the gate band (54.5, 11.0, -207.6) -> (68.0, 11.4, -207.9); the fresh-boot live
check settled that the straight crossing drops into the pit [L 2026-10-30]. Find the game's
walkable route [D], walk it on real input, replace the teleport, keep the walk green.

## What was found

**The game's own nav graph routes its ground patrols AROUND the band, not across it.** The 0c
nav dump (27 paths, this session's `probe_0c_nav` run — data via `nf_formats::aipath`, block
0x05 [D]) shows:
- the WEST ground path: nodes (42.97, 6.17, -207.22), (62.18, 6.17, -207.35) with a south leg to
  (43.08, 6.17, -237.81), (54.80, 6.17, -237.96) — links 3<->0, 2<->1, 1<->0;
- the EAST ground path: (80.63, 6.17, -207.68), (94.34, 6.17, -207.78), (94.27, 6.17, -224.21) —
  a SEPARATE path (no link crosses the chasm);
- a y-0 ring path (kind 0x202) at (76.33..85.04, 0.00..0.01, -215.21..-230.69) — the game
  declares the south y-0 yard walkable;
- the band patrols (Patrol Path_G3 x 76.9..94.2 at 10.34; Patrol Path_G4 x 76.7..93.5 at 6.17)
  and the mid-void band rail (path n00 (63.06, 10.34, -207.54) <-> n01 (49.69, 10.34, -207.29))
  — the AI rails, not player floor (the move test sweeps horizontally [M NDrone2_MoveTest
  0x155330]).

**The ground route is real and walkable — this session WALKED it on real input** (the new test
`crates/nf-view/tests/walkreal_0700000c_eastroute_live.rs`, 1 passed, 1163 ticks, 0 stuck,
0 falls):
1. the band west end (10.34) → turb2's staircase flights down (`obj_turb2_staircase01/02/03`, y
   2.97..11.17, x 49.29..58.73, z -208.96..-211.67 [D band9 plc20-22]);
2. the crate field / the y-0 yard east, passing UNDER the chasm (the yard floor ~0.0 spans x
   65..87.5, z -206.5..-216 — the scans' earlier "bottomless chasm" readings were rays that
   stopped at y 2, above the yard);
3. a running JUMP onto the east ramp's first step (`obj_turb4_staircase02/03`'s lower flight —
   steps 1.75→7.19 at x 80.75..87.25, z -209.25..-211.25 [D the 0.25-grid step scan]) — the walk
   test's own jump idiom (the slot jump, leg 9);
4. up the ramp (0.2 steps) to the east ground (`obj_turb4_walkway03`, y 6.16..7.19, x
   73.22..87.58 [D plc42]) — the east side reached at ground level at x ~77, past the chasm
   x 64.05..72.82.
Sister pinned in the test: the band-height void over the chasm stands (no floor 8..12 at
(59, -207.5)).

**The deep-south gallery is real too** ([D] the z -240.29..-235.51 walkway family —
turb1_walkway06 .. turb5_walkway04, all y 6.16..7.19, x 40..96 [D band9 plc15-18, 27, 31, 40,
44, 46]; the nav's south legs land on it): a diagnostic walk took Bond from the west ground
(43, 6.17, -207.5) south to z -237.8 and east to x 88.8 on real input with zero stuck events
(probe walk, not committed as a test — the east end of the gallery dead-ends at the crates4
stack, x 92.65..95.30, tops 8.31, z -236..-230.4 [D plc99]).

## What is still open (honest)

**The final east climb — ground 6.17 to the east band 10.34 — is NOT pinned.** Every scanned
face dead-ends: staircase01's west face (9.0..9.34) stands over the under-stair void (the y-0
yard); `obj_turb5_walkway02`'s south end (10.34, z -227.26) faces the ground with no steps; the
crates4 tops (8.31) are 2.03 below the band south edge — past the 0.78 jump apex [M player.rs's
3.92 m/s up]; the band's south rail (11.17 at z -209.0) walls the deck. The gate #556 does NOT
fire from the ground (probe: Bond at (95.0, 7.3, -209.5) facing it, 120 use pulses — no
level_exit), so the walk's east legs still need band level. **The walk test's stand-in 3 of 3
(the teleport) stays**, unchanged and green.

## How it was verified

- `walkreal_0700000c_eastroute_live` (the new test): **1 passed, 0 failed** (2.18 s, release,
  NF_ISO set).
- `walkreal_0700000c_live` (the whole walk, untouched): **2 passed, 0 failed** (15.70 s) — the
  walk stays green with the stand-in.
- The nav dump + the step scans were diagnostic probes (deleted before commit; the numbers quoted
  above are [D] disc data via the port's parsers, which the collaudit probe already proved equal
  to the disc's declared collision, 0 gaps in all 46 maps).
- Full suite: this sandbox kills background processes between tool calls (canary-verified last
  session; setsid/nohup/disown do not save it), and the foreground full pass does not fit the
  window. The touched binaries are quoted above; the next session should run the full suite on
  the merged tree.

## Live checks for the main agent

1. **The east climb (the decider).** Fresh boot, level 0700000c, put Bond on the east ground
   (80.63, ~7.1, -207.7 — the G4 node) and hunt the ascent to the east band (10.34) with
   fieldprobe screenshots from: `--bond 84.0,7.3,-208.4,0.0,3.1` (should show staircase01's west
   face and whatever the game puts above it), `--bond 86.0,7.3,-209.0,0.0,3.14` (the ramp top and
   the band's south rail), `--bond 93.8,7.3,-228.0,0.0,0.0` (walkway02's south end and the
   crates4). If the game shows a staircase/ladder the port's parse misses, gdbwatch the contact
   cel (HITDATA+0x54) to name it.
2. **The catwalk access.** The catwalk (15.53..15.96) spans x 40.7..79.6 [D catwalk probe] with
   no found access. Screenshots from the turbine room's upper lane and the display-ground east
   return would settle whether the game reaches the east band via the catwalk instead of the
   ground.

## What's left

- The final east climb (above) — then the walk test's teleport (54.5, 11.0, -207.6) ->
  (68.0, 11.4, -207.9) can be replaced by: the walked ground crossing (this session's test) +
  the pinned climb + the band west-walk back to x 68.
- A full suite pass on a background-capable sandbox.

## Usage

not visible to me (this environment shows no token, credit, context-window or cost figures for
the session; nothing to copy). The research subagents of the PREVIOUS session (level-06) reported
`Tokens -> prompt:137627 completion:7520 total:145147` and `Tokens -> prompt:116493 completion:
7048 total:123541` in their result metadata.
