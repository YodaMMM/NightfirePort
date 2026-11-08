## Your task: `drone-bump-latch-turn`, a bumped guard turns its walk the way the game's does

Branch from origin/s/drone-bump-msg-22, merge current origin/master, and read its docs/reports/drone-bump-msg-22.md. Msg 0x22 now reaches the guard and the latch and direction are stored, but nothing reads them: `DroneAnim_PreTransCallback` 0x13b490:98-118 turns the anim's root motion by the +0x90 direction while the latch is set. Port that reader in the game's order, and check docs/port-map/guard-ai.md's "partial" note on PreTransCallback for any other arm still missing there. Then the second level pin the task asked for: find a mission walker that enters state 0x16 later in a walkreal route and bump it there through real input, or list exactly what a live PCSX2 read would need under the report's live checks. Rand draws must keep the game's count. Run the ai binaries, guard_ai_*_live, walkreal_07000001_live, walkreal_07000009_live and nf-game.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/drone-bump-latch-turn`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox3 at 2026-11-08T12:18Z
