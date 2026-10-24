## Your task: `rigid-body-remainder`, the rigid-body functions the port never names

Never mentioned in `crates/*/src`: `RB_CalcTorque` 0x1d7658, `RB_ApplyWind` 0x1d7c78, `RB_AddImpulsiveForce` 0x1d7d40, `RB_AddImpulsiveForceAtPoint` 0x1d7d58, `RB_ToWorld` 0x1d7e18, `RB_ToBody` 0x1d7eb8, `RB_apply_omega` 0x1d7f50, `RB_VecSpecial` 0x1d8038. The rigid bodies (placement kind 0xD8, `RB_Create` 0x1d6bf0) are drawn at rest; `docs/models/misc.md` notes their motion (`RB_EulerEvolve`) as unported.

1. Find every caller (explosions, bullet hits, wind, the update) and which on-foot levels place 0xD8 bodies [D + M].
2. Port the rigid-body update and the force calls that reach it on those levels [M], with tests: a body at rest stays put; a blast or a shot moves it along the game's integration. The draw already reads the body's matrix, so a moving body must move on screen (state that in the report; the render check is not yours). A function no on-foot path reaches: prove and cite it.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/rigid-body-remainder`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.
