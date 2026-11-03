## Your task: `level-12-alarm-guard-walls`, 07000012 guards wedge against nothing after the alarm

After the alarm on 07000012, guards running toward Bond stop and press against empty space near (22.97, 1.06, 36.21) (seen at frame 1391 of a real play). Nothing is drawn there. level-12-door-gap (the room cel offset) and level-12-corridor-guards-stuck are already on master, so start from current master. Reproduce on real input: walk the walkreal_07000012 route, trip the alarm the way a player does, and record every drone whose position stops changing while its state is chasing or attacking. For each wedge, find what blocks it: the static collision [D coll-scan], a door or blocker volume, a nav link, or the port's bot push. Compare that with the game's drone movement and collision path [M NDrone2_DoCollision 0x149ec8, Collide_Jointy 0x1ec180, the nav/obstacle routines]. If the port blocks where the game does not, fix the port. Pin a live test: after the alarm, every chasing guard reaches Bond's area within a bounded number of ticks, and none sits more than N ticks against a spot with no [D] geometry. Run walkreal_07000012_live, interactables_07000012_live and nf-game.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-12-alarm-guard-walls`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox1 at 2026-11-03T00:12Z
