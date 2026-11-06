## Your task: `guard-radii-01-d3`, the 01 00 Agent walk under the game's guard radii

Branch from `origin/s/guard-radii-finish`, merging `origin/master` first, and push to `s/guard-radii-01-d3`. Read item 5 of docs/reports/guard-radii-finish.md on that branch. walkreal_07000001_live `..._real_damage_00agent` is the one red in that binary: guard 11's threshold fight runs the pistol dry, the fall-through shoot flanks the door dry, and lip guard #5 (0x5000038, 24 hp) takes the last 24. A mid-fight resupply was tried and measured worse. Take the two levers it names: resupply BEFORE the hall phase at difficulty 3, so the threshold fight starts with ammo, and run the lip block's scan again once the escort crossing begins, since #5 surfaces late. The radii are the game's: never put 2.0 back, never lower damage or health. Test-side only, helpers opt-in. Run walkreal_07000001_live whole and quote it, then nf-game.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/guard-radii-01-d3`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox7 at 2026-11-06T03:23Z
