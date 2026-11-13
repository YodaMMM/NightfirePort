## Your task: `real-health-00agent-1b-6`, 0700001b's 00 Agent real-health walk defeats Drake

Branch from origin/master. walkreal_0700001b_live::walk_arrival_to_drakes_defeat_on_real_health_00agent is ignored; the last round pinned the real branch-read cadence (probe1b_cadence) and fixed the walk's deploy/clock sampling during defense fights; the console phase is still red - the ledger and next levers are in the ignore message. Run walkreal_0700001b_live, probe1b_cadence and interactables_0700001b_live. Real input only, no top-ups, never lower damage or health; every lever gated to the level, its difficulty and !w.tops_up(). Remove the ignore when green, otherwise push the ledger and the next lever into it.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/50-real-health-agent-06`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

**Branch:** `s/real-health-00agent-1b-6`.

---
claimed-by: sandbox7 at 2026-11-13T12:14Z
