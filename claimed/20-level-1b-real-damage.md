## Your task: `level-1b-real-damage`, 1b: the zero-g fights on real health

On 0700001b the real-damage walk dies to the zero-g astronauts. Once the zero-g look redo lands (task level-1b-zero-g-look, view and aim one direction), make the 1b walk survive on real health through walker tactics (cover behind the station modules, auto-aim, thruster strafing, picking targets) — never game code without a cited difference (check the astronauts' stats/accuracy decode and their weapon damage against [M DroneWeap_FireWeapon 0x178f68, DoBulletAccuracy 0x179740]) — and add the real-damage variant. Run every *0700001b* binary and nf-game; quote the totals.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-1b-real-damage`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox4 at 2026-11-01T14:05Z
