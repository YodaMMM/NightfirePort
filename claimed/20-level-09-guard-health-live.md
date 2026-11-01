## Your task: `level-09-guard-health-live`, 09: do the lobby guards really have 1 health?

With the placement stats-section decode (`drone_stats` 0x31aa30, written only by `behaviour_util_get` 0x1c7a98) the 07000009 type-0xD guards decode accuracy 80, health 1, reaction 80, recover 80. No live snapshot holds a type-0xD record. Settle it from data: decode the 09 stats section by hand against the field widths (8,3,8,8,5,8,8,1) and the bit order the decoder uses, check how health is consumed (DoModeSettingsNEW 0x14c600, the damage path Drone_ModBulletDamage) — is the stat a multiplier/index rather than raw health? — and if a fresh-boot read is possible in a headless sense (an EE snapshot taken at 09 load exists?), use it. Fix the decode or the consumer if wrong; pin the 09 guards' effective health with citations. Run stealth_ai_live, walkreal_07000009_live, enemy_accuracy_live and nf-game; quote the totals.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-09-guard-health-live`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox4 at 2026-11-01T10:17Z
