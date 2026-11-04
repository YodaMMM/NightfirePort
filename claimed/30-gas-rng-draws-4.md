## Your task: `gas-rng-draws-4`, finish the gas draws on the shared RNG stream

Continue origin/s/gas-rng-draws-3: branch from it, merge origin/master in, and read docs/reports/gas-rng-draws-3.md and docs/reports/gas-rng-draws-2.md on that branch. The gas-init draws ride the shared stream [M 0x1aacc8:72-80], so every later roll shifts and some real-path walks need new routes. Take the report's list in order. Start enemy_accuracy's per-setup re-pin first, since that run takes 8 minutes. 1) The walk re-routes: 01 (contact route and the friendly-death stretch), 02 (to_the_exit), 03 (east-lane wedge and friendly death), 14 (the corridor-door chain at (30.3, 35.4)), 16 (punch pin and defeat walk), 1b (Drake's defeat). Adapt each the way a player would, never by forcing state. 2) fix_blast_scripts' type-0 leg, which the report found geometry-dead: its teleport spot is a void and destroy[10] is a zero-extent box. 3) fix_finale's 16 shelter pulse. No assertion may be weakened unless a decompile address proves it wrong. Real input only: no health, ammo or channel writes, and no teleports. Never lower damage or health to make it pass. Fix the port only where it differs from the game, with the address. Shared test helpers (crates/nf-view/tests/common/) change only behind a per-level opt-in. Run every walkreal_*_live whole, then the full suite with tools/test/chunk_suite.py.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/gas-rng-draws-4`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox3 at 2026-11-04T16:26Z
