# level-11-stairwell-bullet-wall-4 — the remaining invisible bullet walls in the stairwell are the exterior backdrop planes the game's cel-scoped pick never sees

Branch `s/level-11-stairwell-bullet-wall-4` (from origin/s/level-11-stairwell-bullet-wall-3
cd9ef225, its tip, merged with origin/master aa94ce02). Window 18:56–20:55 UTC; sandbox was reset,
so setup was redone from scratch (clone → branch push at 18:58, work-slim, the ISO, rustup, the
ALSA user prefix, cargo fetch). The fast runner (`tools/test/suite.py`) started its build at 19:14
on the merged tree.

## What's done — the root cause, evidenced

**The census (assertion-free, committed): `world.rs stairwell_walls4_census_07000011`.** From the
reported spot — Bond (22.40, 30.09, 1.42, yaw −1.505, pitch −0.035), aim dir ≈ (−0.996, −0.055,
0.066) via the port's `Bond::look` lanes [M crates/nf-game/src/player.rs] — a 45-ray fan across the
well plus 5 near-centre rays with per-hit identity. Every near-centre ray meets, in order:

1. t ≈ 0.009: map43#126 `stairwell_blocking_ploys … fudge` — glist 0x10, mat 0x40, port_pass=true,
   drawn=false → already passed by wall-3's rule [M Collide_Update 0x1ea398:216-229 + Collide_Intersect
   0x1ebaa8 disasm 0x1ebf78-0x1ebf80]. Not the blocker.
2. **t ≈ 0.048–0.10: map43#175 `exterior_cell5_back_right` — glist 0x0, mat 0x0a, drawn=true (the
   object has gfx leaves), port_pass=false — THE BLOCKER.** Two of its triangles: the diagonal
   plane [(16.5,33.6,0.1)(21.9,33.6,5.5)(16.5,24.5,0.1)] under the stair flight and the upright
   plane across the middle of the well — exactly the two mid-air decal planes the report names.
   The port's flat `segment_bullet` stops the round there (centre ray t = 0.0616).
3. t ≈ 0.13: map43#127 `ROOM_StairWell_2` mat 0x0a — the real far wall the game's decals land on
   [D, and the wall-1 report's t 0.36–0.39 from the lower spots].

The game's round passes `exterior_cell5_back_right` **because the flying bullet's pick is
cel-scoped and that object's cel is never in the pick from this spot**:

- [M Collide_Pick 0x1eb320:125] the candidate nodes come ONLY from `Collide_StraddleCels` — the
  BFS from the query's cel (HITTEST+0x80) through the portals the segment crosses
  (`Intersect_Portal` 0x1ea7b8) — plus the ForcedList (dynamic objects; that walk runs for the
  bullet because HITTEST+0x98 = 0 [M Bullet_update disasm 0x12f040 `sh $zero, 0xe8($s4)`] clears
  the bit-0x10 gate, and the bullet's obj-level filters pass: obj+0xf0 = glist 0x0 [M Collide_PickObj
  0x1eae80:72-82 builds the skip mask 0x20 from +0x98 = 0]).
- [M Collide_StraddleCels 0x1eac98:19-78] the BFS crosses ONLY `Intersect_Portal` acceptors; cels
  with no crossed portal are never picked.
- [M Bullet_update 0x12e7b0 disasm 0x12f0cc-0x12f0d8] the flying bullet re-feeds its pick's start
  cel every tick (`sw $v0, 0xd0($s4)` ← `lw $v0, 0x20($s5)`), and arms +0x94 = 0x101/0x201,
  +0x9a = 0x20 for guns without weapon_data+0x6c & 0x800 [M 0x12f048-58, 0x12f134/0x12f154].
- [M Collide_Update 0x1ea398:216-229] the walk's pass arms stand as wall-3 pinned them; the
  disasm also re-verified the surface-byte folds in Collide_Intersect: 0x40 sets the pass bit
  ungated [M 0x1ebf78-0x1ebf80]; 0x80 sets it only under HITTEST+0x98 bit 1 [M 0x1ebf8c-0x1ebf94]
  — which the flying bullet's +0x98 = 0 clears, so an 0x80 surface's only pass is the walk's 50%
  roll [M Collide_Update:217-220], and a surface byte of exactly 0x10 passes whenever the bullet's
  +0x9a carries 0x20 [M Collide_Update:222-225]. Both arms are still unmodelled in the port's
  flight (the glass one deterministic; the roll needs a shared-RNG draw placed exactly where the
  game draws one — per walked 0x80 contact, nearest-first).

**The scope probe (assertion-free, committed):
`nf-view/tests/stairwell_walls4_scope_07000011_live.rs`.** It rebuilds the sim's parse path
(level → instances → `parse_cel_graph` → `scope_graph` → `build_scoped`), maps each instance to its
graph cell through `own_cells_main`, and runs the game's BFS from the spot's cells:

- The spot's bbox-containment cells: graphcell128 = **ROOM_StairWell_2's own cel** (inst121,
  placement 0xc047 — a room-class cel), plus cell129 yellowstripe (glist 0x221), cell132/133
  (shadows/floornumbers, glist 0x221), cell151 = `exterior_cell5_back_right`'s own cel (inst144,
  0xc047), and the fudge lives in graphcell127 (placement 0x8019 — a generic).
- **The BFS along every reported ray collects ONLY graphcell128** — no portal is crossed into
  cell151 (backdrop) or cell127 (fudge). So the game's pick from this spot sees the stairwell
  room's cel nodes (+ the generics chained to it) and NEVER the exterior backdrop: the round flies
  across the well to the ROOM's far wall. The mid-air decal planes cannot happen in the game.
- The port's flight, by contrast, traces the FLAT static world (`segment_bullet`,
  drone.rs:13384) — every triangle of every pushed instance — so the backdrop planes wall the
  well. That is the remaining invisible wall.

**Supporting [D] notes from the census**: the mat-0x06 ROOM_StairWell_2 hits (7 fan hits,
t 0.009–0.104) are the room's own geometry (drawn); `exec_office_window5` (glist 0x201, mat 0x0f)
appears once at t 0.85 on the fan's edge. None of them carries 0x40/0x80/0x10.

## The fix

1. **The flight query — LANDED (not yet wired into the flight): `segment_bullet_scoped(cell,
   from, to)` in world.rs** — `scoped_runs(cell, ray_portal_hit)` (the straddle BFS the scoped
   actor queries already run), then the same one-sided nearest-first walk as `segment_bullet` over
   those runs + `always` + the missile cage + the dynamic set, keeping the 0x40 drop
   [M 0x1ebff0-0x1ec010]. Out-of-range/no-scope cells fall back to the flat answer (the caller's
   safety net). Pinned by the new lib test `the_bullets_scoped_pick_skies_the_portal_unreachable_walls`:
   a wall in a portal-unreachable cel stops the flat trace and not the scoped one; a straddled
   portal brings it back; the 0x40 fudge plane passes with the solid card behind stopping; the
   fallbacks hold. **nf-game lib on this tree: 549 passed / 0 failed / 5 ignored** (the new test
   included; the pre-existing scoped/world/bullet tests all still green).
2. **The bullet's per-tick cell** [S — the wiring choice; the game re-feeds the pick's start cel
   per tick (0x12f0d8) but the port does not track a per-bullet cel]: bbox containment over the
   scope cells, preferring room-class cells (the game's `build_FindCel` 0x1c8cb0 collects room
   cels), smallest volume as the tie-break — or porting nf-view's `find_cell_raycast` into the
   world so the collision-ray tie-break matches the game. The probe shows the stairwell spot has
   5 bbox candidates, so the tie-break matters and needs the pinned test to settle it.
3. **The walk's other arms**: the glass pass (surface byte == 0x10 under +0x9a & 0x20) is a
   deterministic drop in the same walk; the 0x80 roll draws `Rand_Rand(0x3f) & 1` per walked 0x80
   contact in nearest-first order [M Collide_Update:217-220] — a shared-RNG draw, to be added
   only with the draw-order pinned and the suite re-run.

NOT wired in this window: the change alters every level's bullet flight, and the required full
suite could not be re-run after a behaviour change inside the remaining window. Nothing was
weakened or pinned unverified.

## How it was verified

- The failing state reproduced in the census BEFORE any fix: the flat `segment_bullet` stops at
  t 0.0616 on `exterior_cell5_back_right` on the centre ray, on all 5 near-centre rays and 45/45
  fan rays (t 0.048–0.099 across the fan).
- The game's pass is pinned by the [M] chain above (Pick → StraddleCels → per-tick start cel) plus
  the scope probe's BFS output; the probe's instance↔cell mapping is `own_cells_main`-based
  [M parse_cel_graph] and self-consistent (cell_owner[cell128] = the ROOM instance, etc.).
- A probe artifact was caught and corrected in the WORKLOG: the RAW `build_scoped` keying leaves
  graphcell128's bucket empty (pushed-ordinal ≠ cell index once sub-maps add cels first); the
  sim's loader re-keys the buckets (`attribute_scope` ×2, scene.rs 8482/8504 — room-class cells
  keep their own runs, generics join their chain owners). So the sim's cell128 DOES hold the
  stairwell room; the defect is the flight query, not the buckets.
- Crates touched: nf-game (the two probes + the `scoped_run_owners` diagnostic) and nf-view (the
  scope probe). Both compile; the census runs green (`1 passed`); the scope probe runs green
  (`1 passed`); both are `#[ignore]`d (assertion-free) so they stay out of pass/fail.
- The fast runner started at 19:14 on the merged tree (before any of this task's edits) and is
  still running at report time; its failures so far are on the merged tree, none touched by this
  task (see below). **The full-suite total line: see the WORKLOG's final entry — the run did not
  finish inside this window, so there is no total line to report.**

## What's left

- Wire `segment_bullet_scoped` + the per-tick bullet cell (the [S] tie-break above), then the
  real-input pins: from the reported spot, rounds across the well reach the far wall / the rope
  guards with no mid-air contact; a sister into a real wall still stops. Headless renders of the
  spot before/after (the decals) for the main agent.
- The glass 0x10 arm and the 0x80 roll (the roll's RNG draw order), each with its own pin.
- The full-suite total line on the final tree.

## Live checks for the main agent

1. **The stairwell spot, fresh boot of 07000011** (a save-state resume skips the boot path): stand
   Bond at (22.40, 30.09, 1.42), yaw −1.505, pitch −0.035, fire a burst across the well. Expected:
   the decals land on the far stairwell wall (map43#127) and on the rope-guard-side geometry — NO
   decals hanging mid-air on the diagonal plane under the stair flight or on the upright plane
   across the well's middle. A screenshot pair (the well from Bond's spot) pins it.
2. **The bullet's cel in play**: gdbwatch on the flying bullet's HITTEST+0x80 (obj+0xd0) through a
   flight from that spot — the value should change as the round crosses cel boundaries
   (`Collide_StraddleCels` starts from it each tick [M 0x12f0d8]); note one value while the round
   is inside the shaft vs after it leaves through the top, if it does.
3. **The 0x80 roll, if any**: whether any stairwell surface the rounds cross carries the surface
   byte 0x80 [D the collision blocks] — if yes, a slow-mo burst at the same spot should show ~50%
   of the rounds flying past a 0x80 face; if no, the roll arm stays untestable here.

## Usage

not visible to me — this session's environment shows no token, credit or context-window figures
(no usage display was available at report time; nothing estimated).
