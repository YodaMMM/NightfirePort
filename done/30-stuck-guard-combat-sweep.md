## Your task: `stuck-guard-combat-sweep`, extend the stuck-guard sweep to combat movement, and the 07000005 helipad

The stuck-guard sweep covers patrol and alarm travel only. Guards are also seen standing still in combat around the 07000005 helipad and on other levels.

1. Add the combat movers to the sweep's moving set: cover seeking / run to cover, the bunching waits (`DroneMove_NoBunching`), the combat move anims (`DroneAnim_SetCombatMoveAnim` 0x13cfb0), strafe/approach [M].
2. On 07000005, walk Bond into the helipad courtyard with real input (the parked helicopter statics at (-17.0, 43.0..44.8, -37.2..-43.6) [D]), engage the guards, and log every guard that stays in a moving combat state without moving.
3. For each hit, find what the game does differently [M] and fix the class in the shared code. Turn fixed hits into assertions with a sister each.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/stuck-guard-combat-sweep`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox3 at 2026-10-24T10:41Z

---
## Report (done, 2026-10-24T12:13Z)

Closed by the driver: the report is on `s/stuck-guard-combat-sweep` (docs/reports/stuck-guard-combat-sweep.md); the worker didn't run finish.sh.

---
## Report from the worker (done, 2026-10-24T12:17Z)

