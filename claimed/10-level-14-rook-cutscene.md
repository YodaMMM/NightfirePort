## Your task: `level-14-rook-cutscene`, 14: Rook appears in his cut-scene

On 07000014, after the generator hall blows (the console E -> the cascade), the cut-scene at the Delta Sector door plays (camera at about (-34.62, 1.07, 26.19), around frame 7735 of a normal run) but Rook is not in it — the port shows a masked guard in front of the door instead. Rook is the drone with skin 0x05000066 [D docs/levels/07000014.md: (27.81, 0.20, 45.78), wake ch 0x0D, death ch 0x54]. Find the cut-scene's script and its actor list [D the NIS streams: Entity hashes, the skin/model each entity resolves to; M Script_CreateEntity 0x1d8968 + the cut-scene actor binding], what the game binds there (Rook's own drone posed by the script, or a script entity with Rook's skin), and why the port draws the guard / omits Rook (a hash that resolves to the wrong object, a skin not loaded on 14, the drone hidden by its spawner/wake state). Fix the port with evidence and pin it with a test that names the entity and the skin the cut-scene draws at its start (and a sister that fails on the guard skin). Run nf-game, every *cutscene*/*nis*/*script* binary and walkreal_07000014_live; quote the totals.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-14-rook-cutscene`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox5 at 2026-10-31T05:12Z
