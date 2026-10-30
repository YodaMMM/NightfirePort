# level-0c-gate-band-crossing — report

**Branch:** `s/level-0c-gate-band-crossing` (from origin/master 0c90eba8, merged origin/master 8878f6b6 at session start).

## What the task asked

The 0c whole walk crosses the gate band x 55.5..67 on a stand-in: the port's disc collision is
void there, yet the game's nav path17 links (49.69, 10.74, -207.29) <-> (63.06, 10.74, -207.54)
across it. Find what carries the player across (a mover, a wire, a collision the port mis-reads),
fix the port side, replace the stand-in with a real-input leg, keep the walk green.

## What was done

Session 1 (this branch, commits a8ff5e90/f16f49be/9065964c) probed the band: the void is static
(no SS/wire/mover), the catwalk 15.5 is continuous x 40.7..79.6 with no found access, the
grapple hypothesis was eliminated (none of the 5 grapple points is over the void), the strip
route is a cul-de-sac (7.98-unit gap vs the engine's 5.2-unit carry), the forklift drives the
north yard at y 0.

Session 2 (this window) closed the carrier census and corrected the one over-reading:

1. **[D] The full band-box placement census** (new probe `walkreal_probe0c_band8_live`): every
   placement of every map in x 40..85, z -220..-190. Over the void there is NO carrier entity —
   only the guard spawn plc60 (62.99, 11.42, -207.43), a PlayerStart plc104 on the band west
   (48.56, 10.34, -207.53), and oil barrels at band height east (x 72-74). The sim's grapple
   points (5, all class 0x3a) and wire store confirm: no grapple point and no wire touches the
   band gap. All five 0x3a points are named `graple_pulley` (#308/#346/#500/#508/#511 [D
   docs/levels/0700000c.md:98]); the near one gp4 (54.27, 15.35, -237.02) is the south yard, 30
   units south of the band.
2. **[D] The placed collision extents of the whole turbine walkway/staircase/beam family** (new
   probe `walkreal_probe0c_band9_live`): the pit floor is obj_turb2_walkway04 plc28 (x
   49.29..63.66, y 6.16..7.19, z -209.06..-205.12), continuing east as the sliver
   obj_turb3_walkway03 plc33 (x 63.66..64.05, y 6.17..7.19), then the floor-less chasm x
   64.05..66.27 down past y 2, with the resumed band 10.16+ sitting **+3.2 above the pit floor**
   — the pit route dead-ends, no jump crosses up. The staircases are symmetric flanks, not a
   crossing: turb2's rise 2.97->11.17 at x 49.29..58.73 and turb4's 0.16->11.20 at x 78.14..87.59
   (both z -208.96..-211.67, south of the band); nothing spans x 59..78. The supportbeams are
   0.5-0.8-wide columns, no deck.
3. **[M] path17's link is not floor evidence.** The walk test's evidence block over-read it as
   "the game's own data insists the band is continuous". Corrected in this session's commit: the
   game's node-to-node move test sweeps the body from node to node (NDrone2_MoveTest 0x155330,
   the port's `nav::move_test` mirror; gates |dy| <= 4 and the room boundary [M 0x155330 via
   drone.rs]) — a horizontal sweep across a HOLE hits no wall and no floor, so a designer-linked
   pair over a pit tests clean. The mid-void guard plc60 pins a placement, not a walkable
   surface. AI path assignment is per-drone (NDrone2_AssignAIPath [M 00154190]; AINetwork_CalcRoute
   [M 00156ba8]).

**Conclusion:** nothing the disc declares carries the player across x 56.08..66.27 at band
height — mover, wire, grapple, script mover and mis-read collision are each positively excluded
by the placed data, and the port's static world equals the disc's declared collision exactly
(session 1's collaudit probe, 0 gaps in all 46 maps), so there is no port-side parse to fix.
The crossing mechanism remains unpinned by everything the port can parse; the one instrument
that can decide it is a fresh-boot PCSX2 watch (below).

**The walk:** stand-in 3 of 3 kept (teleport (54.5, 11.0, -207.6) -> (68.0, 11.4, -207.9)), now
pinned by FOUR sister asserts — the band-height void, the strip cul-de-sac wall, and two new
ones from this session: the pit floor's presence and height at (59, -207.5) (6.16..7.19 [D
plc28]) and the east sliver at (63.9, -207.5) (6.17..7.19 [D plc33]). A real-input leg through
the gap is not buildable from the parsed data: every candidate route (strip jump, pit climb,
grapple, wire, mover) is excluded by the extents above, and rerouting the walk onto the catwalk
requires an access leg the data does not show.

## How it was verified

- `walkreal_0700000c_live` (the touched binary): **2 passed, 0 failed** (14.48 s, release,
  NF_ISO set) — the whole 0c walk + the 0d arrival, with the new sisters live.
- The two new probes: `walkreal_probe0c_band8_live` **1 passed**, `walkreal_probe0c_band9_live`
  **1 passed** (their dumps are the [D] evidence quoted above).
- Suite (foreground chunks; see the runner note): nf-formats **108 passed, 3 failed** — the 3
  failures are exactly the permitted `disc_golden` tests (the slim ISO lacks /DRIVING/MIS01.MUS:
  `bigf_header_fields_behave_as_documented`, `bigf_tables_match_the_golden_manifest`,
  `directory_tree_matches_the_golden_manifest`); nf-audio **28 passed, 0 failed**.

## The suite runner in this sandbox

`nohup python3 tools/test/suite.py` was started twice in the first minutes; **this sandbox kills
background processes between tool calls** (verified with a canary writer: it died within one
tool-call boundary, setsid/nohup/disown did not save it), so the runner could not stay up. The
fallback was foreground per-crate chunks under the standard command; the window allowed
nf-formats + nf-audio plus the touched nf-view binaries. The full four-crate pass needs a
sandbox that keeps background processes alive.

## What's left

- The fresh-boot live checks below.
- If the live check shows the game's player genuinely crossing at band height, the next session
  builds the real-input leg (the mechanism will then be nameable from the debugger's HITDATA);
  if it shows the fall, the walk's stand-in graduates to a documented "the game has no crossing
  here" note and the route question (catwalk access) opens.
- A full suite pass on a background-capable sandbox.

## Live checks for the main agent

1. **The player at the band gap (the decider).** Fresh boot (not save-state), level 0700000c,
   Bond on the band west end (54.5, 10.9, -207.6) facing east; hold forward. Watch the player's
   y over x 56..67 at z -207.5 with gdbwatch.py on the player's position triple. Expected per
   the port: the body falls to the pit floor (y lands 6.16..7.19). If the game instead crosses
   at y ~10.3: record the HITDATA cel (HITDATA+0x54) and the surface the pick reports at x ~60 —
   that names the carrier the port's parse misses.
2. **The mid-void guard plc60.** Same fresh boot: watch the drone's y for 10 s from spawn
   (62.99, 11.42, -207.43). Expected if AI is collision-bound: it falls to the pit floor or
   slides west; if it stands/walks at 11.4 the AI y is path-driven and path17 is an AI rail.
3. **The catwalk access (the alternate route).** fieldprobe screenshots from
   `--bond 54.5,11.0,-207.6,0.0,0.0` (should show the catwalk ~5.2 above, spanning the gap) and
   from the turbine hall's upper east lane looking south — to see whether the game's visible
   route to the gate climbs to the catwalk instead of crossing the band.

## Usage

not visible to me (this environment shows no token, credit, context-window or cost figures for
the session; nothing to copy).
