# level-1b-drake-3 — 0700001b: the zero-G float's wall, the shaft mouths, the route to Drake

**Status: the two questions the round was asked are DECIDED with [M]/[D] evidence and pinned as green tests; the walk and interactables tests stay `#[ignore]`d at the (now proven game-side) sealed boundary, with the scripted-carry live checks named.** Branch `s/level-1b-drake-3` = origin/master 8b101249d + s/level-1b-drake-2 cf56f98b9 (master's parallel drake-fight merge taken for the conflicted shared files; drake-2's probe1b_float superset kept).

## 1. The wall at x 4.43 — decided: the game's own boundary, body-solid

**[D probe1b_wall]** (new assertion-free dump of the level archive) the face is **plc #128 OBJECT_Boundary_1** (kind 0x8019, cel 0x8000, glist 0x0): a two-triangle pair at **x 4.433 spanning y −16.56..+16.74, z −20.40..+20.40**, both stored normals facing EAST (+x), **surface byte 0x40** (surface index 0 + the 0x40 flag). It spans the whole map cross-section — the slit band, the mouths' height, everything below the y ≈ 16.74 slab.

**[M] the game's rules stop the player body on it:**
- The surface byte's 0x40 sets `HITDATA+0x51` bit 1 on every hit (`Collide_Intersect` 0x1ebaa8 disasm 0x1ebf78-0x1ebf80), and that bit drops hits **only for the HITTEST+0x98 bit-0x8 bullet queries** (0x1ebff0-0x1ec010). The player body rides `Collide_CylinderIntersect` 0x1ecc00:61's kind 0x800, which carries no bit 0x8.
- `Intersect_CylGeom` 0x109da0's per-triangle loop rejects no face by side — its head-on cull (:302-322) only skips faces within ~2.6° of horizontal; a vertical boundary is tested from both sides.
- `Player_ZeroG` 0x192ab0 composes **only thrust/rotation** (no collision calls at all) — the shared movement applies the same body collision. The float gets no bypass.

So the port's capsule stop at x ≈ 4.98 (4.433 + the 0.55 body radius) **is the game's behaviour, not a port artefact**. Pinned green: `walkreal_0700001b_live::the_boundary_wall_stops_the_float_and_passes_bullets` — the body ray hits the 4.433 plane with mat 0x40 across the band and at the mouths' height; the sisters pin the bullet pass (the 0x40 drop) and the west→east ray pass (the east-facing normals, [M 0x10a8c8:168]); a real-thrust float from the slit mouth min-x lands in 4.85..5.15 and never past.

## 2. The z ±8.82 "mouths" — decided: iris-door booths, not a route

**[D]** the mouths are two booths at (5.23, 9.45, ±8.82): a glass pane (mat 0x40, bullet-pass) at x 5.91 in `OBJECT_AddOnStructure`, an iris door (plc #30 `OBJECT_IrisDoor1` / plc #32 `OBJECT_IrisDoor3`, kind 0x2031) at x 5.23, and **SP#141/#142 "ScriptPlayer_IrisDoorOpen"** (kind 0xd9) carrying the door meshes (44 tris each).

**The doors are load-time loopers, not missile-triggered pieces** — the earlier "SP#141/#142's looping zero-G scripts open on the missile-done channels" reading is REFUTED:
- The create runs `SP_LoadScript` for mode ∉ {0,1} [M `SP_CreateScriptPlayer` 0x1db9a8 tail], and its mode-8 arm starts the script **LOOPING** (`Script_Play(script, 1)`) in state 6 [M `SP_LoadScript` 0x1dbc08:46-48]. The doors animate from the level load.
- The script 0x06000818 per loop [D decode]: the 8 iris-panel entities (0x02000834..3b) open at keys 0..120, the script HOLDS at t208 (stream 9 `ev0x03[0]` — the −1 delay), the panels close at 480..600 with the door sounds (0x000004c4 / 0x000004c3), and the missile-done channels **0x18/0x1c are CLEARED at t550** (streams 12/13) — each ~10 s loop clears the missile bookkeeping, it does not consume it as a trigger.
- The channel-rise restart does not apply: [M `SP_Update` 0x1dbda8 case 6] reloads only through the OTHER script slot, and both SPs carry a single script (hash[1] = 0 [D live players]) — a rise on ch 0x18/0x1c starts nothing.
- Either way the doors are NOT the route west: their meshes sit at **x 5.23, east of the boundary** at 4.433, and the boundary spans the doors' whole y band — open doors dead-end at it.

Pinned green: `walkreal_0700001b_live::the_iris_doors_are_load_time_loopers` (state 6 + script loaded from load [M 0x1db9a8 + 0x1dbc08]; the 8 panel entities exist [D]; the ch 0x18/0x1c rise leaves both SPs in state 6 — the sister).

## 3. The route to Drake — where it stands

The game's crossing past x 4.43 **must be a scripted carry** (no real-input path exists past the boundary). The candidate chain is unchanged: SP#211 "ShuttleFlyBy2" (gate 0x1e, trigger 0x2e) plays 0x060007fd → 0x060008f0, whose streams pulse ch 75 (the ported zero-G entry [M 0x216388]) and whose tail pulses **ch 74 (0x600096e, t392-399 [D])** — still the unmeasured candidate for the carry. The port's parsed 0x060008f0 fires no event 6 (the census is drake-2's, unchanged). Both 1b tests stay `#[ignore]`d at exactly this point, with the live checks in the walkreal ignore message.

**Live checks for the main agent** (also in walkreal_0700001b_live's ignore message):
1. **Fresh boot, during the slit crossing**: does the game's Bond pass x 4.43 westward in substate 9, or is he stopped like the port? (The port's stop is now [M]-proven correct, so if the game passes, the pass is the scripted carry, not free movement.)
2. **Through 0x060008f0's t392-399**: watch `switch_channels[74]` and the shuttle entity's matrix — does the player object's position get written from an entity matrix (the carry), and at which address?

## What's left (for the next round)

- The scripted carry: identify the mechanism (the ch-74 pulse is the candidate; a live read of the carry write is the decider), port it, then the landing → the fight gate → the defeat → the un-ignore of `walk_arrival_to_drakes_defeat` and `drake_defeat_by_real_fire_ends_the_mission` with sisters.
- The interactables kill-channel blocker (master's newer narrowing): the walker-driven kill leaves ch 0x0e off while the raw step_bond kill raises it — possibly the same walker-shoot regression as 07000002's computer (below).

## Verification (what ran)

- Touched crates, on the final tree: `walkreal_0700001b_live` **8 passed / 1 ignored** (2 new green pins + the re-pinned chain test), `interactables_0700001b_live` **6 passed / 1 ignored**, `probe1b_wall` (assertion-free dump, 1 passed).
- `nf-view` lib rebuilt clean; `nf-game`/`nf-formats`/`nf-audio` untouched by this round's diff (below).
- **Full suite (`tools/test/suite.py`)**: 263 binaries completed with **2 failures** — `fix_blast_scripts_live::the_destroyable_smash_plays_its_type_script` and `walkreal_07000002_live::walk_the_castle_all_objectives_to_the_exit` ("the computer's destruction raises ch 0x48"). The sandbox kills background processes at every tool-call boundary (setsid included — the previous round documented the same), so the suite ran in the foreground and hit the 10-minute tool cap at binary 263; the remaining binaries (including `walkreal_0700001b_live` and `interactables_0700001b_live`) were run directly instead — both green (above). **Both failures are on this tree's inherited code**: `git diff origin/master..HEAD` touches only 0700001b test files and docs (6 files, no src) — neither failing test's path is in the diff, and the previous round's tree (older master base) had the blast tests green. They are origin/master regressions for the next rounds, not this branch's.
- Sandbox note: `nf_view::load` + the walker are timing-stable; the 2 failures reproduce deterministically (run twice).

## Usage

not visible to me (no token, cost or context-window figures are exposed in this environment).
