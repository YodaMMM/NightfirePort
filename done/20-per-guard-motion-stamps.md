## Your task: `per-guard-motion-stamps`, the target-moving stamps are per guard in the game

The port keeps ONE global first-moved / first-stopped stamp for the player's motion; the game keeps one per guard (Drone+0x224, the +0x1DD/+0x1DF latches [M DroneWeap_DoOpponentTargetting 0x179a48:47-73]). Port the per-guard stamps (each guard latches when IT observes the change), with the addresses, and pin: two guards that start tracking at different ticks clear their first-moved terms at different ticks (sister: the global model would clear them together). Run enemy_accuracy_live, ai_live, every walkreal_* real-damage variant and nf-game; quote the totals.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/per-guard-motion-stamps`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox1 at 2026-11-01T18:45Z

---
## Report (done, 2026-11-01T20:41Z)


Result: done — the per-guard first-moved/first-stamped latches (Drone+0x224, +0x1DD/+0x1DE, +0x1DF/+0x1E0 [M 0x179a48:47-73, per-drone via HandleFiring 0x17a180:29/:59]) replace the port single global stamp; two-guards clear-at-different-ticks pin added; enemy_accuracy_live 5p/0f, ai_live 37p/0f, all 12 walkreal real-damage variants green, nf-game 387p/0f, formats+audio 114p/4 slim-ISO disc-data fails, nf-view chunks 1-2 partial 702p/2 no-GPU env fails. Full report: docs/reports/per-guard-motion-stamps.md on branch s/per-guard-motion-stamps.
