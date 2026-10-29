## Your task: `level-11-stairwell-bullet-holes-6`, the stairwell's last mid-air bullet hole

Read the level-11-stairwell-bullet-holes-5 report (on master; its measurement test `stairwell_impacts_render_07000011_live` is there, the pin ignored-RED). (1) Land the scoped flush re-query it describes (`segment_surface_flags_scoped(cell, from, to)` mirroring `segment_bullet_scoped`, the flush using `find_cel(cur)`) - [M] `Bullet_CollisionHandler` 0x12ce80's HITDATA walk - and re-measure. (2) If the one mid-air decal at ~(23.20, 28.22, 5.77) remains, compare the flush's per-hit processing line by line against 0x12ce80 (post-ricochet scoped pick; the glist-0x10 node rule in `Collide_Intersect` 0x1ebaa8) and fix the remaining difference, or prove it game-true [M/D] and pin that. Un-ignore the pin when it is green. The RNG stream moves with any face change: run the FULL suite and quote its total line.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-11-stairwell-bullet-holes-6`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox3 at 2026-10-29T15:19Z

---
## Report (done, 2026-10-29T16:35Z)

Closed by the driver: the report is on `s/level-11-stairwell-bullet-holes-6` (docs/reports/level-11-stairwell-bullet-holes-6.md); the worker didn't run finish.sh.
