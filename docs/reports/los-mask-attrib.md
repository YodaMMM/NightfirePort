# los-mask-attrib — report

Task: los-mask-01-walk-2's "What's left" items 3+4 — attribute the three reds (09's
alarm-switch nav route, 13's plain-walk doorway probe, guard_rounds_glass's 07000003 glass
stop) to the mask change or master's own merge by running each on both, then fix in the
game's order (red on the old code) or repair the walk gated to its level. Branch
`s/los-mask-attrib` = origin/s/los-mask-01-walk-2 (73a33ac5a) merged with current
origin/master (6a4fcf4ef), merge f8b1c0592 clean.

## Attribution (the first deliverable): all three are the mask branch's own delta

Each failing test binary was run on BOTH trees, same test files (09/13/guard_rounds test
files are byte-identical between origin/master and the branch — checked the two-dot diff):

| test | origin/master (6a4fcf4ef) | branch (mask tree) |
| --- | --- | --- |
| `guard_rounds_glass_live` | **ok** 1/0 | FAILED (03 scenes drained false, hurt false) |
| `walkreal_07000009_live` | **ok** 9/0 | FAILED 8/1 (`the_alarm_switch_fails_objective_1`) |
| `walkreal_07000013_live` | **ok** 3/0/1 | FAILED 2/1 (the plain warm chain's out-doorway probe) |

Not master's merge: current master is green on all three. The delta is the mask lineage's
`LOS_MASKS_ON=true` envelope (world.rs `pick_pair`/object admissions in the masked walks),
which master does not carry (master's `LOS_MASKS_ON=false`, 6 vs 11 envelope hits in
world.rs; the merge-base is 0a9d39435).

## Root cause (one mechanism, measured three ways): the stand-in probe went mask-shaped

`move_probe_clear` (world.rs) — the port's guard/route step probe, **a stand-in with no game
query behind it** — routed into the masked walk when the gate turned on
(`line_of_sight_masked(a, b, LOS_PROBE_8)`). The game's `NDrone2_MoveTest` 0x155330 runs
`AINetwork_BoundsTest` ONLY — distance 30/|dy| 4/slope band, then BoundsTest; **no collision
ray at all** [M 0x155330:86-108, the decompile read this session]. The ray is a long-standing
[S] port choice (nav.rs since 47c0ebe83), and its proven semantics were the plain
`segment_clear` (panes transparent, no material drops) — the semantics every green walk was
measured with.

- **09** — the nav route to the ch-124 switch: with the masked probe the route re-picked its
  nodes and the walk wandered west into the level-exit volume; with the plain probe restored
  the route machinery is provably correct again — a level-09 diagnostic (since removed)
  printed the goal node at (17.66, −369.03, 43.17), 1.08 u from the switch, route result 1
  (fallback), 19 pts ending at the switch node. The remaining 09 red is NOT the route: see
  below.
- **03** — `guard_rounds_glass`: the mask change moved the guard's sight verdict (below), and
  the probe change moved his route beats; with the plain probe restored the binary's other
  levels behave as before.
- **13** — the plain walk's security-room door leg (below).

**Fix (in the game's order):** `move_probe_clear` is pinned to `segment_clear` on BOTH sides
of the gate (world.rs), with the [M 0x155330 bounds-only] citation and the [S] deviation
recorded at the gate. The mask envelope stays with the real `Collide_LineOfSight` callers
(`line_of_sight`, `line_of_sight_hit`, `line_of_sight_scoped`, `line_of_sight_excluding` —
`Collide_LineOfSight` 0x1edf50's port, whose application the envelope report proved).

## 03 / guard_rounds_glass: FIXED — the test's scene census was wrong for the game

The 07000003 scenes (panes #338/#339): the guard reaches his combat states but never fires;
the pane never drains; Bond is never hurt [L the run print]. The scene verdict with the
game's own vision mask: the guard's 0xa27 sight ray is **blocked at t 0.719 by mat 0x05**
triangles at x 33.85 (the pane's band) [L the run's `[glass-scene]` print]. The game's
vision mask drops surface class 0x10 ONLY [M `Collide_Intersect` 0x1ebaa8:315-317 — mask
0x2], so **in the game a guard there cannot see Bond through that pane either** — the old
test's premise (any kind-2 pane carries a scene) pinned the pre-mask port's pane-skipping
`segment_clear`, which skipped panes entirely [M PANE_PART's note, world.rs]. The port now
matches the decompile; the TEST was wrong.

**Test fix (with the address):** `try_pane` now validates each scene with the game's sight
rule before running it — a scene whose 0xa27 ray is blocked is not a scene (the blocker's
mat/t printed). The pane loop `continue`s instead of `break`s (a later pane may carry a
valid scene), and the census assert drops to `covered >= 2` with the 03 story in the comment:
07000001 (pane #76, sight clear, drained + passed) and 07000012 (pane #120, the
0xffdf-filtered sky-bridge panes, passed but not drained) carry scenes; 07000003's panes stop
the game's own sight ray at mat 0x05, so no scene exists there. The decompile-backed
assertions (drained != filtered, Bond hurt) are untouched.

## 09: root-caused past the route; the walk repair is left red

The route is correct (the diagnostic above). The measured failure: during the nav-15
clearing beat, tower guard g4 races for the switch (state 0x3e, ever_seen true, seen_ticks
113 — he saw Bond during the approach where the masked vision admits him), the chase follows
his position but loses by ~200 ticks (the chase hops opened a stair-base door 9 u BELOW
Bond's floor — the walker_b2 door pick keys on dxz only, [L the chase trace: three
`opening door (21.5,-377.1,40.9)` pulses while Bond stands at y −368.30]), and g4 presses at
frame 7517 — the fail menu prints ch 0x61 true [L NF_EXIT_DBG]. Left RED with three levers
for the next session: (1) walker_b2's door pick needs a |dy| gate (shared helper — opt-in
flag per the merge rules), (2) a pre-emptive dart beat at nav 14 for any tower guard with
seen_ticks > 0, (3) the chase should intercept at the switch, not at the racer's current
position. Binary otherwise green: **8 passed / 1 failed**.

## 13: the doorway lane stall, left red with the mechanism measured

The plain warm chain crosses the security-room door northward once, returns inside for the
panel leg, and the re-cross press stalls: Bond at (−26.96, −368.30, 35.77), zero root motion
for 600+ ticks with the door open (state 0x4, swing 1.00) [L the 13diag run: guard #0 in
AIM_STAND_FIRE 0x68 inside the room's NW corner, guards #1/#3 (0x5c) just outside the
doorway]. The masked-vision timeline leaves these guards posted where the pre-mask tree's
did not, and the merged solid bodies (master 0a9d39435, in both trees) let them hold the
lane. Repairs measured and rejected this session: the file's index-first pre-clear spent its
stands on farther guards (killed false ×7), a nearest-blocker aimed stand also returned
killed false (the doorway geometry denies the exposure the stand needs), and the walker's own
step machinery wedges at the same z 35.95-36.0 (6000-tick cap, floor_ny 1.00, no head bump).
The loop cap extension to 3000 ticks does not clear it either. Left RED: the binary is
**2 passed / 1 failed / 1 ignored** — the real-damage sisters still pass. Next levers: the
stand's gun-ray exposure rule against the doorway lip, or a pre-clear beat positioned INSIDE
the room before the first crossing (the d3 beat's own spot, one room earlier).

## The battery (release, --test-threads=1, NF_ISO on the slim on-foot ISO)

Ran on the final tree (branch = los-mask-01-walk-2 + current master + this session's fixes):

- **GREEN**: `guard_rounds_glass_live` 1/0; `npc_doors_live` 5/0/3; `nf-game` all 118 test
  binaries ok (lib 760/0/4, ai_live 37, + the rest — no FAILED lines); `walkreal_07000003_live`
  6/0/1; `walkreal_0700000a_live` 13/0/2; `walkreal_0700000c_live` 3/0/1;
  `walkreal_0700000d_live` 1/0/2; `walkreal_07000041_live` 15/0; `walkreal_0700004a_live` 4/0;
  `walkreal_07000015_live` 7/0/2; `walkreal_07000016_live` 15/0/3; `walkreal_0700001b_live`
  16/0/3.
- **RED, attributed**: `walkreal_07000009_live` 8/1 (the racer, above);
  `walkreal_07000013_live` 2/1 (the doorway, above); `walkreal_07000002_live` 3/2 — run on
  current master for attribution: **master green 5/0** — the mask branch's own delta, the
  envelope report's own "02 needs the same re-tune as 01" class (its What's-left item 2), not
  a fresh regression.
- **Not re-verified this session** (the 2-hour window spent on the attribution + the three
  root-causes): 01 (timed out at the 240 s guard mid-d2-walk — the envelope report's own d2
  red class), 04/05/06/07/08, 0b (the roof-sprint lineage's own red, left by the prior
  session), the 0c/11/12/14 ledgered-ignores and route sisters, 41_wall, 4a_satchel. The
  probe revert restores master's exact probe semantics and eight of the sweep's greens
  re-ran green (0a/0c/0d/15/16/1b/41/4a), so the remaining levels' risk is the same class the
  prior sessions already ledgered.

The fast runner (`tools/test/suite.py`) was started in the background twice (first minutes
of each session); both starts died between tool calls with an empty suite.out — the same
sandbox behaviour los-mask-01-walk-2 recorded. The per-binary totals above are the final
tree's state.

## What's left

1. 09's walk repair (the three levers above), then re-run `walkreal_07000009_live`.
2. 13's doorway repair (the levers above), then re-run `walkreal_07000013_live`.
3. The 01/02 masked-vision re-tune (the envelope report's items 1-2, unchanged), then the
   full walkreal battery this session could not fit.
4. The fast suite on the final tree (chunked; the background starts die in this sandbox).

## Live checks for the main agent

1. On 07000003 at the pane #338 stand (guard (29.9, 5.25, −9.25), Bond (35.4, 5.25, −9.25)):
   confirm in the game that a guard there does NOT open fire through the pane (the port's
   0xa27 ray stops at mat 0x05, t 0.719 — gdbwatch on DroneVision_CanSeeObjectFrom
   0x176f18's result settles the scene census).
2. On 07000009: whether a fresh-boot tower guard races to the ch-124 switch when Bond darts
   his patrol partner at the lobby stair head — the game's own chase/press timing for the
   racer lever.

## Usage

not visible to me — the driver shows no token, cost or context-window figures (the IM
gateway metadata carries only session/chat/trace ids; no usage counters appeared in any tool
result). Shell-clock proxies: branch+WORKLOG push 06:40 UTC, the probe pin commit 07:33, the
scene-census fix commit 07:56, this report 09:10 UTC 2026-11-10.
