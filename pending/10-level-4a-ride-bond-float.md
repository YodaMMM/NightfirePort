## Your task: `level-4a-ride-bond-float`, Bond stands on the lift in the ride cut-scene

In Phoenix Fire Part 3's (0700004a) lift-ride landing cut-scene (pack 0x060006cf, SP #289 base (0, -53.53, -3.38)) Bond's body floats above where he should stand. `s/level-4a-ride-cast-b` (on master) decoded the anim-object placement (the root track seeds obj+0x30 [M Script_AnimStart 0x1d8f40:69,94; AnimFrameResolve 0x1c35f8]; no base compose in the per-tick anim stream [M Script_Interp 0x1d9a08 case 2]) and pinned the car-local roots [D 0x040005fd root (2.35, 0.85, -0.64)], but the compose that puts the cast in the car is still open. Find the Bond actor's stream, its sequence roots [D] and the game's rule that places him [M], and port it scoped to what the game does so Bond stands on the deck; the 07000014 scenes stay unchanged (a global SP-base compose broke them once).

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-4a-ride-bond-float`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

