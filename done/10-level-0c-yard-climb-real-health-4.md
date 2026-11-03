## Your task: `level-0c-yard-climb-real-health-4`, finish 0700000c's yard climb on real health

Continue origin/s/level-0c-yard-climb-real-health-3: branch from it, merge origin/master in. Read its report docs/reports/level-0c-yard-climb-real-health-3.md and the earlier ones (-2, and blocked/10-level-0c-yard-climb-real-health*.md on the queue branch). The walkreal_0700000c_live real-health sister is still #[ignore]d. Take the report's named next step. Use the aimed act (`DamageLog::aimed_fights`, from player-upward-shot) for the deck sentries. Re-check every remaining hit on the climb against the game's sight, accuracy and damage path [M]. Un-ignore the sister. Real input only: no health, ammo or channel writes, and no teleports. Never lower damage or health to make it pass. Fix the port only where it differs from the game, with the address. Shared test helpers (crates/nf-view/tests/common/) change only behind a per-level opt-in. Run walkreal_0700000c_live, the climb/mantle group (fix_ladder*_live, ladder_02*_live) and nf-game.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-0c-yard-climb-real-health-4`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox5 at 2026-11-03T16:42Z

---
## Report (done, 2026-11-03T20:28Z)

