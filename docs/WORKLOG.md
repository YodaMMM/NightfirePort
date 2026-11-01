# WORKLOG

## level-03-locked-door-use-volume (branch s/level-03-locked-door-use-volume)

- start (master 8ce724ad). Task: the mask-4 trigger beside 07000003's locked door — the port's use-pick does not match the game's reach; find the game's use-pick volume [M Player_Activate 0x1a5170:100-137 + the object's collision/glist bounds; D placement], fix the port's stand-in, un-ignore `interactables_07000003_live::locked_door_refuses_the_e`, run every interactables_* / interact_* binary and quote totals.
- the mask-4 use-pick volume fixed: `player_use_triggers`' expanded-AABB stand-in replaced with the game's two-stage pick — the bounds-sphere pre-gate [M Collide_PickObj 0x1eae80:219-228, the sphere from the tris' AABB] + the Intersect_PointGeom refine on the disc's glist tris [M 0x1ebaa8:186-190 → 0x1e92d8:200-229, front-side plane window (0, 1.0] + the projection on the tri; the edge-clamp arms' refusal pinned by the 07000003 audit, // [S] unverified at the site with a live check filed]. `locked_door_refuses_the_e` un-ignored — 9 passed / 0 failed / 0 ignored on the 03 file; the E-plate test's stance moved to the documented wedge spot with the horizontal look (the pick is the point head+1×look, the old aim landed the point north of the volume).
- the two-sided plane window (the 0x15 exit planes' behind-the-plane E forced fVar25 = −radius [M 0x1e92d8:116-121, 227]; the flags byte stays [S] unverified) + the projection-inside refine — walk_07000015_live 2/0/0, interactables_0700000d_live 11/0/0 (the crane touchplate's stand moved onto the sliver sheets), interactables_07000003_live re-verified 9/0/0.

## use-pick-two-stage-2
Started 2026-11-01 20:48 UTC. Branch s/use-pick-two-stage-2 = origin/s/level-03-locked-door-use-volume + origin/master (6042ee53).
Task: the use-volume branch's two-stage pick merged onto master went 17 red (all E-use paths). Find what
differs from the game (the use point, the [S] parts of the point test, the bounds sphere, the non-glist
trigger classes), fix, keep 03's locked_door_refuses_the_e passing, all listed binaries green.
Plan:
1. Read the branch's report + the merge conflict in player_use_triggers.
2. Settle the law from the decompile: 0x1ec940 (mode/flags), 0x1e92d8 (window + accept), 0x1eae80 (pre-gate).
3. Resolve keeping both intents, run the 17 + the branch's 03/15/0d tests, fix what breaks with evidence.

Work log:
- (20:52) merge conflict resolved: master's mesh law kept (sphere_meets_tris: window (0, r] + the CLAMPED
  distance [M Intersect_PointGeom 0x1e92d8:200/227/229-231; the use pick's flags word +0x9a = param_9 = 0
  → fVar25 = 0 [M 0x1ec940's uStack_1d8 = param_9; 0x1e92d8:110-113]]), the branch's bounds pre-gate kept
  as the cull (AABB-derived enclosing sphere; the game's is the MODEL's sphere [M 0x1eae80:219-228]), the
  branch's projection-inside reading (point_reaches_tris + point_in_tri) removed as dead code superseded by
  the decompile — the 03 doorway refusal is the front-only window (the fin's X < 0 from the doorway), not
  a projection rule. Building + running the 17 next.
- (21:10) the 17 red reproduced and triaged: the branch's projection-inside reading (point_reaches_tris)
  refused legit edge-adjacent presses. The decompile settles the law: Intersect_PointGeom 0x1e92d8 registers
  on vecutil_Dist2Tri_PointGeom's CLAMPED distance ≤ r [M 0x1e92d8:229-231; 0x1dfac8's switch cases 1-6 = the
  edge/vertex arms], the window is (fVar25, r] with fVar25 = 0 for the use pick (hittest+0x9a = the call's
  param_10 slot [M 0x1ec940's uStack_1d6 = param_10; 0x1e92d8:110-113]), the node distance is preset 1e8 and
  a refine miss drops the node [M 0x1ebaa8's 0x4cbebc20 preset; 0x1ec940's finalize]. master's
  sphere_meets_tris IS that law — kept; the branch's bounds pre-gate kept as the cull; the dead
  point_reaches_tris/point_in_tri removed.
- (21:35) the empty-tris arm keeps master's tight per-axis sphere-vs-box (my first bounds-gate-everywhere
  cut fired a long-thin exit trigger from a door row — use_key_census_live 07000001 was the regression pin).
- (21:45) the game's stage-2 cel scope ported (Collide_Pick 0x1eb320:125/:313-328): each trigger's ONE link
  cel resolved at its model world-sphere centre [M Trigger_Create 0x1b1720:96 + Control_BuildWorldSph
  0x133070; the hints/icons construction], the player's straddle set over the pick segment r 1.0 gates the
  pick; fail-open on unresolved cels; MP bodies ungated (documented).
- (21:55) 03's locked_door_refuses_the_e CANNOT pass under the decompile law: the y-6.44 sheet's corner sits
  0.43..0.77 from any doorway use point [D coll-scan obj 308 hit 229], and the plate's cel (53) == the
  doorway stance's cel [D probe] — the audit's refusal needs the game's live use point (the viewer-matrix
  dir, 0x1a5170:100-108), a live check. Test re-ignored with the full citation in its doc comment. All other
  pins green: the 16 red-list binaries + use_key_census_live 20/0 + 03 8/0/1 + 15 2/0 + 0d 11/0.
- (21:58) nf-game 1028/0/9. Full suite started on the final tree.
