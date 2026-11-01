## Your task: `level-11-start-corridor-guard`, 11: a guard walks into the start corridor and shoots Bond

On a fresh 07000011 load with Bond standing at the start, the port has a red-suited guard walk into the corridor beyond the unlocked door from about tick 200, hit Bond by tick 509 and kill him by tick 3172. On the PS2 the same corridor stays empty over the same frames [L PCSX2 capture: the start-position lighting sweep, frames ~f200-f3172]. Find which drone this is (placement, group, wake channel, route), why it wakes or walks in the port [M the wake/ENTER gates, Drone_Create 0x136be0's difficulty gate, the patrol/route class], fix the gap with the address cited, and pin it: an idle Bond at the start is not approached in the first 3200 ticks (sister: the guard IS present in drones.list). Run every *07000011* binary and nf-game; quote the totals.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-11-start-corridor-guard`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox5 at 2026-11-01T02:21Z
