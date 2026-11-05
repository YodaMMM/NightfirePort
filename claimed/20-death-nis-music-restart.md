## Your task: `death-nis-music-restart`, the death music also plays when a death cut-scene starts from the restart arm

`SP_Update` case 1 reaches `Script_Play` as well [M 0x1dbda8 case 1, 0x1dc174], so a death NIS started there runs the same camera tail: `Music_Event(6, 1)`, then the flush. Merge origin/s/subtitle-flush-camera-scripts first if it is not on master; it added the flush on that arm of crates/nf-game/src/nis.rs and left this gap in a comment. Push the same kind-0x41 request there, so both arms share one helper for the camera tail. List which placements can start a death NIS through this arm [D], and pin one through the real path if any exists; if none does, say so in the report and keep a unit pin. Remove the comment once the gap is closed. Run the nis tests, the music binaries and nf-game.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/death-nis-music-restart`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox4 at 2026-11-05T17:40Z
