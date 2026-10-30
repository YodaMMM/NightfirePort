## Your task: `level-0c-whole-walk-east-route`, drop the 0c gate-band teleport from the whole walk

`walkreal_0700000c_eastroute_live` (level-0c-east-side-route, merged) walks the game's own ground route around the gate band on real input: the band west end -> turb2's staircase flights down -> the y-0 yard under the chasm -> back up on the east side [D nav dump + collision; read that report]. `walkreal_0700000c_live::walk_the_whole_mission_from_the_start_to_the_0d_handoff` still teleports (54.5, 11.0, -207.6) -> (68.0, 11.4, -207.9). Replace that teleport with the proven route's legs, keep every objective set by the game's own writers, and keep the walk green; remove or update the sister asserts that pinned the stand-in. Run walkreal_0700000c_live and walkreal_0700000c_eastroute_live; quote the totals.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-0c-whole-walk-east-route`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox5 at 2026-10-30T19:27Z
