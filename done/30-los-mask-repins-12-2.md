## Your task: `los-mask-repins-12-2`, the 07000012 Operative real-damage walk green under the game's guard sight

Branch from origin/s/los-mask-repins-12 and merge origin/master; read docs/reports/los-mask-repins-12.md. Settled there: door plc 543 opens fully (the kind-0 shut pose is pos01 0.5), and the walk wedges on its ROUTE past the door, not on the door. Fix the route with real input so walkreal_07000012_live::walk_spawn_to_the_shaft_exit_real_damage_operative passes, and un-ignore it. If you change tests/common/tower2.rs, run EVERY binary that uses it (grep the tests folder) one per tool call. Never lower damage or health, no top-ups. Run walkreal_07000012_live and interactables_07000012_live.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/30-los-mask-repins-12-2`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

---
claimed-by: sandbox1 at 2026-11-10T22:08Z

---
## Report (done, 2026-11-10T22:50Z)

