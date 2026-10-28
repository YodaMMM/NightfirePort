## Your task: `drone-bump-collision-9`, the 07 party hall walks past the east door

Continue `origin/s/drone-bump-collision-8` (merge `origin/master`). The guard wall is down per the live measurements; cutscene_timer_pause and walkreal_07000007 are green there. Left: (1) probe_07_route's post-E hover wedge - Bond at (93.40, -46.32) hangs 0.23 over the floor, grounded=false, for a 6000-tick leg: find which floor/portal/door-leaf face the port's scoped floor ray reads there vs the game [M Player_FeetOnPoint 0x194188, Player_Collision 0x194df0; the open leaf's dynamic tris] and fix it; (2) party_door_lock_07000007_live's sister `#244 still answers the use key`: derive the door's real use zone from its 0x48 trigger bounds [D] and the use path [M Player_Activate 0x1a5170], re-derive the stand point with the cite. Never disarm the push. Run the FULL suite and quote its total line.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/drone-bump-collision-9`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox3 at 2026-10-28T19:04Z
