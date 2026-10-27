## Your task: `level-14-capture-placement`, Bond lands in the cell after the Countdown capture scene

On Countdown Part 1 (07000014), after the capture cut-scene that sets the objective "Sabotage base security and escape from room", the port leaves Bond at about (-107.07, -89.41, 73.28): outside the level, in the void. Find where the game puts the player when that scene ends [M the scene's script ops that move/teleport the player (Script_* player placement, the SP/entity base the scene composes, the level's start-point or marker it uses), the cell's placement data [D]] and port exactly that, so Bond stands in the cell and the escape objective is playable. Drive it on the real path in a test (reach the scene, let it play, assert Bond's position/room after it ends against the game's rule) [M].

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-14-capture-placement`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox1 at 2026-10-27T07:19Z

---
## Report (done, 2026-10-27T08:43Z)

