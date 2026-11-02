## Your task: `consider-explosive-gate`, guards flee explosives from the game's states

`DroneFunc_ConsiderExplosive` in the game refuses to flee only from states 0x28, 0x2d, 0x4a-0x4b, 0x53-0x55, 0x63 and 0x86-0x88 [M 0x1431c0:40-67]; the port also refuses 0x2e-0x49, 0x52, 0x56 and 0x62-0x85, so a port guard in AimStandFire 0x68 never flees a grenade (GAPS G-055). Port the game's gate with the address, pin a guard in 0x68 fleeing a live grenade (sister: one in 0x63 does not), and fix any walk that becomes fragile on the walker side. Run ai_live, the grenade/explosive binaries, every walkreal_* and nf-game; quote the totals.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/consider-explosive-gate`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox5 at 2026-11-02T02:43Z
