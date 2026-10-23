# WORKLOG

## level-15-ladder-2 (branch s/level-15-ladder-2, started 2026-10-23 19:15 UTC)

Task: 07000015 ladder grab (round 2). `crates/nf-view/tests/interactables_07000015_live.rs::the_ladder_takes_the_climb` is `#[ignore]`: the grab cannot fire from any reachable standing spot (east room's south wall at z -1.95). Goal: find how the game's ladder grab reaches it (use sphere / grab volume / allowed approach) from the decompile [M], port it, un-ignore the test with a sister that can fail. Real input only via `step_bond`.

Work Log:
- 19:15 UTC: sandbox alive (repo, work/, ISO, ../nf-queue all present). Branch s/level-15-ladder-2 created from origin/master a67f1a27e.

### 2026-10-23 20:28 UTC — the ladder admission traced to the game's own dead link

- Traced the full grab admission chain in the decompile [M]: Ladder_Create 0x1966c0 → build_LinkToRoom 0x1c9680 → build_FindCel 0x1c8cb0 (room cel entity-params bounds boxes; parsemap_block_entity_params 0x1d1508 fills slot+0x20 min / slot+0x30 max — the same records the port's MapObject.min/max parses); the ladder's world-sphere centre (84.529, 31.733, −2.288) is inside NO room box [D — east room y ≥ 39.28, shaft cels x ≤ 81.87] → unlinked (obj+0x20 = 0); Controls_StraddleTest 0x134778 (disasm: BFS start = obj+0x20, delay slot sw $a2,0x80($sp)) counts 1 < 2 → not on Collide_Pick 0x1eb320's ForcedList (LAB_001eba38) → never a pick candidate → Player_CollLadder 0x196118 never fires. All THREE ladders miss FindCel the same way. The game cannot grab them from any spot; the port already matches (lad.cell = None → never admitted).
- The ignored test's premise (the walk-in grabs) contradicted the decompile — the TEST was wrong. Rewrote it as the_dead_ladder_never_grabs_at_point_blank (real-input walk across the ladder plane's x on the south walkway, climb never engages) + the sister sister_the_dead_ladder_mesh_reaches_the_point_blank_stander (the pick mesh WOULD hit at point blank — the blocker is the admission, not the geometry; can fail on any pick/geometry/box change). Both GREEN (10 passed / 0 failed / 1 pre-existing ignore).
- Added CollisionWorld::debug_scope_cells (debug introspection, the debug_* family).
- Census §7 ladder row corrected (the old "one climbed here" was wrong).
- Live checks for the main agent offered in the report (Ladder_Create's $v0 → obj+0x20 expect 0, obj+0xf0 & 0x10008000 expect 0).
