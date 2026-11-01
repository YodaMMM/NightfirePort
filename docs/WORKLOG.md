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
