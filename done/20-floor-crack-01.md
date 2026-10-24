## Your task: `floor-crack-01`, a drone falls through the 07000001 floor at (-4.703, 6.34, -5.968)

On 07000001 a standing guard in STEP_AIM_RIGHT 0x74 loses the floor at x -4.703, z -5.968 and falls forever (`Drone::feet_on_point` misses; gravity runs). `world.floor_below([-4.703, 7.3, -5.968], 40.0)` returns None, while points 0.1 away find the floor at 5.34. Four static objects' bounds meet there (one ends at x -4.735 / z -5.969, one starts at z -5.967, one spans x -4.736..-2.0, z -8.958..-1.065 and covers the point), so the triangles leave a gap at a T-junction. A 1e-6 edge band in `seg_tri_hit` does not close it. fix_guard_weapons_live's `the_3x3_hit_rate_and_damage_matrix_matches_the_game` is ignored until this is fixed (its diff-2 guard walks onto the spot).

1. Dump the triangles of the objects whose bounds contain the point (`objects_with_bound`) and find the gap: a T-junction, a missing triangle, a build/transform error in the port, or the cel scoping (`build_scoped`).
2. Find what the game does at that point: `Drone_FeetOnPoint` 0x140660:98 calls `Collide_RayIntersect` (cel-scoped). Cite why its ray hits there [M], and port that rule — not a widened tolerance unless the decompile shows one.
3. Tests: the feet probe at the point finds the floor; a census over 07000001's walkable floor (a grid of feet probes over the nav paths' nodes) finds no other cracks. Un-ignore the 3x3 test and keep it green.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/floor-crack-01`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox4 at 2026-10-24T04:21Z

---
## Report (done, 2026-10-24T04:28Z)

Closed by the driver: the report is on `s/floor-crack-01` (docs/reports/floor-crack-01.md); the worker didn't run finish.sh.
