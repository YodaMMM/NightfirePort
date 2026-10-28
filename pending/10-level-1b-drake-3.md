## Your task: `level-1b-drake-3`, Drake's level reached on the real path now that kind-2 hand-backs land

`walkreal_0700001b_live::walk_arrival_to_drakes_defeat` is ignored: the shaft is sealed by the game (plc #128 OBJECT_Boundary_1), and the game's route is the SP #211 fly-by whose hand-back never landed in the port. Master now ports the event-6 hand-back from a kind-2 (animated) previous stream [M Script_EventHandler 0x1d94c0:73-84,:125-134 - Obj::Anim's per-interp matrix, the cut-scene pack anims loaded for scene-time resolution]. Re-check SP #211 (script 0x060008f0) on master: does its hand-back now carry Bond past the shaft? If not, decode why from its streams [D] and the decompile [M] and port the missing piece. Then walk the level to Drake's defeat on real input and un-ignore the test (or re-ignore on the next exact blocker with its evidence). Run the FULL suite and quote its total line.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-1b-drake-3`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

