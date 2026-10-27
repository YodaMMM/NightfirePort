## Your task: `level-4a-ride-cast`, where the 0700004a ride cut-scene's characters stand

The 0700004a lift ride's landing cut-scene (pack 0x060006cf, SP #289 base (0, -53.53, -3.38)) plays its cast with CAR-LOCAL roots (sequence 0x040005fd root (2.35, 0.85, -0.64) [D]), so the port draws them ~53 units above the car. The real game draws them in the car: PCSX2 ride frame 700 shows the red-beret guard holding the satchel up in the car's hatch beside the officer, frame 900 shows Bond under the car [L 2026-10-26]. Composing the SP's matrix onto every script actor is NOT the rule: it threw 07000014's control-room cast (world roots near (26..34, 1.2, 48), SP base (52.16, 2.64, 46.62)) ~50 units away and was reverted. `Script_AnimStart` 0x1d8f40 creates the actor at the origin and `Script_Interp` 0x1d9a08 case 2 composes no base [M]. Find the mechanism that places this pack's actors in the car (an anim parented to an entity stream, the AnimObject's own transform, a root channel flag, the pack's type-0xC loadable record) [M + D, with a PCSX2 read of the guard's anim object obj+0x30/obj+0x90 during the ride if needed], port exactly that, and keep every other cut-scene's cast where it is (cutscene_models_live, the 07000014 scene).

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-4a-ride-cast`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

