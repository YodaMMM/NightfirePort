## Your task: `guard-stuck-0c-14`, two guards stall on their way to a goal (Chain Reaction, Countdown)

Branch from origin/master. guard_stuck_sweep_live (run with --include-ignored --nocapture) flags two guards that hold GO_TO_GOAL (state 0x63) without moving: 0700000c drone#19 (skin 0x05000090) at (47.36, 9.71, -179.46), goal (60.61, 9.71, -179.71), 13.2 u away, route not blocked; 07000014 drone#0 (skin 0x05000090, mode 0x07) at (50.34, 1.00, 53.29), goal (56.06, 0.01, 42.91), 11.9 u away. Find why each stops (the route, a door, a wall push, the arrival radius, the goal itself) and compare with the game [M the GO_TO_GOAL handler and route follow]. If the game's guard also stops there, prove it [M] and say so; otherwise fix it with a test red on the old code. Run guard_stuck_sweep_live, walkreal_0700000c_live, walkreal_07000014_live and the matching interactables binaries.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/20-guard-stuck-0c-14`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

---
claimed-by: sandbox5 at 2026-11-10T14:55Z

---
## Report (done, 2026-11-10T15:42Z)

Closed by the driver: the report is on `s/guard-stuck-0c-14` (docs/reports/guard-stuck-0c-14.md); the worker didn't run finish.sh.
