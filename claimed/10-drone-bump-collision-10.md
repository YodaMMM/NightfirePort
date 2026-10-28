## Your task: `drone-bump-collision-10`, the guard push branch onto master with Double Cross Part 1 still finishable

Continue `origin/s/drone-bump-collision-9` (merge `origin/master`). party_door_lock and the 07 walks are green there; two items left before it can merge: (1) on the merged tree walkreal_07000001_live `walk_from_the_spawn_to_the_exit_with_the_contact` goes red (the merge's drone.rs state changes shift the shared-RNG stream) - find what the game does at the failing step [M] and fix the port or re-derive the walk's fight with the cite; Double Cross Part 1 must stay finishable; (2) probe_07_route's post-E anchors walk Bond onto a level object's walkable rim at (93.41, -46.32) (y 43.215 over the 43.0 floor) - identify the object [D its placement and footprint] and re-anchor the route around it like a player walks. Never disarm the push. Run the FULL suite and quote its total line.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/drone-bump-collision-10`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox7 at 2026-10-28T21:57Z
