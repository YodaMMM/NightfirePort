## Your task: `level-objects-uncited`, level-object functions the port never names

Never mentioned in `crates/*/src`: `Copter_FireMissile` 0x17d9f0, `SpaceLaser_Register` 0x1aa9e8, `Sunreflection_Update` 0x18db40, `Bubble_Update` 0x1aca48, `camera_fixed` 0x1ba4e8, `LS_LoadBonus` 0x1ce488, and the sub/creature family `Sub_Laser` 0x1acda0, `Sub_DeployTorpedoes` 0x1acf68, `Sub_Update` 0x1ad200, `Sub_ApplyDammage` 0x1ad570, `Sub_Collide` 0x1ad868, `Sub_Delete` 0x1ad870, `MiniSub_FireTorpedo` 0x181888, `MiniSubMove_Forward` 0x1819d8, `MiniSubState_Attack` 0x181b00, `MiniSub_Update` 0x181f40, `MiniSub_Collide` 0x182128, `MiniSub_Delete` 0x182268, `Creature_Delete` 0x135230, `Creature_Update` 0x135268, `Creature_Hurt` 0x1352d0, `Creature_Move` 0x135340.

For each: name its creator/caller and the placement kind or script event that reaches it, and which levels (on-foot vs driving — the driving missions are not ported yet) [D + M]. On-foot reach: port the simulation [M] with a test (draw-side parts — the sun reflection sprite, bubbles' sprites, missile meshes — are not yours: list them in the report as draw hand-offs with the hash, matrix and timing). Driving-only or unreachable: cite the proof in a comment next to the nearest related port code.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-objects-uncited`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

---
claimed-by: sandbox1 at 2026-10-26T13:26Z

---
## Report (done, 2026-10-26T14:41Z)

