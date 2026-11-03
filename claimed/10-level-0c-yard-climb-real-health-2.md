## Your task: `level-0c-yard-climb-real-health-2`, finish 0700000c's yard climb on real health with aimed shots

walkreal_0700000c_live's real-health sister (~line 409) is #[ignore]d. Read blocked/10-level-0c-yard-climb-real-health.md on the queue branch: the deck sentries' upward rounds never landed from the strip. s/player-upward-shot (report docs/reports/player-upward-shot.md) found the cause. The harness's stands fired from the hip, and weapon 0x1e's hip cone (spread base 100.0 → 4.9 u of scatter at 35 u [L weapon_data rec 0x1e]) misses the chest box the way the game's does. The aimed act lands [M Bullet_init 0x12c5a0:139-170], and that branch adds it opt-in (`DamageLog::aimed_fights`). Branch from origin/master and merge origin/s/player-upward-shot in if it is not on master yet. Opt the 0c real-health walk into the aimed fights, kill the deck sentries aimed from the strip, then climb and mantle. Un-ignore the sister on real input only; never lower damage or health. Run walkreal_0700000c_live, upward_shot_trace_live, the climb/mantle group (fix_ladder*_live, ladder_02*_live) and nf-game.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-0c-yard-climb-real-health-2`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox6 at 2026-11-03T09:51Z
