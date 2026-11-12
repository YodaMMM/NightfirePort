## Your task: `real-health-00agent-15-6`, 07000015 can be walked on 00 Agent with real health past the grenadier

Branch from origin/s/real-health-00agent-15-5, merge current origin/master, and read its docs/reports/real-health-00agent-15-5.md, "What's left", plus the ignore message. Take its levers in order: answer grenadier #8 before his first throw (widen the pre_throw_stands idiom to difficulty 3, or a d3 stand at the plank's cover edges); re-take the #9 give-up at node 2; then the meet's pool chain; then the coded levers that never ran. Judge each over several real runs. Helpers stay gated to 07000015 and difficulty 3, so other walks stay byte-identical. Real input only, no top-ups, never lower damage or health. Remove the ignore when green; if it is still red, push the ledger and next lever in its ignore message. Run the whole walkreal_07000015_live binary and interactables_07000015_live.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/real-health-00agent-15-6`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: main agent at 2026-11-12T12:05:16Z
