## Your task: `guard-capsule-05-down-bump-harness`, the knocked-down guard bump test runs on 07000005

Branch from origin/master. guard_capsule_radii_live::a_down_guard_bumps_at_the_games_03_ball (the 07000005 version) is ignored with two terrain blockers documented in its message; the 07000006 sister runs green and proves the game's 0.3 down ball in play (contact at dxz 0.771). Pick a 05 guard on Bond's own floor (count with Level::parse), walk to him on real input, stun him, and creep in until the body-contact latch fires inside 0.85. If no 05 guard can be reached on the same floor without a teleport, prove it and delete the 05 harness instead. Run guard_capsule_radii_live and walkreal_07000005_live.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/20-guard-capsule-05-down-bump-harness`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

---
claimed-by: sandbox1 at 2026-11-10T12:56Z
