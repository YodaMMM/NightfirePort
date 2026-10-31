## Your task: `onfoot-moving-accuracy`, the "Bond is moving" accuracy term on foot

Guard accuracy uses a "target is moving" term (x0.75) [M DoBulletAccuracy 0x179740, the moving test at 0x179a48:34-41]. The game reads Bond's MOVE VECTOR for it; the port reads a change in position on foot (the zero-g case was fixed on origin/s/1b-real-health to read the thruster pair — read its drone.rs/scene.rs change). Port the on-foot case the game's way (the walk/strafe input vector the player code stores [M Player_SSWalk 0x192310 and its move-vector store]), so a player pushing into a wall or sliding reads as the game reads it. Pin with tests (moving / standing / pushing into a wall; sister). Continue from origin/s/1b-real-health (merge origin/master first). Run nf-game, every *drone*/*combat*/*accuracy* binary and the fighting walkreal_* binaries. Commit and push your branch after each step so a reset loses nothing. Run the named binaries and quote every total.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/onfoot-moving-accuracy`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

