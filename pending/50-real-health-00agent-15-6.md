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

---
## Main agent (2026-11-12, released back)
On s/real-health-00agent-15-5 + master 729f9694e the d3 walk dies at t5527 (13 hits for 100, 10 fights). Lever 1 measured: pre_throw_stands ALONE at d3 (no switch_dry) dies at t1048 at the aperture (81.53, 41.41, -0.49), 11 hits - identical to the earlier both-flags run, so the stand itself is the cost at 00, not switch_dry. Rule it out; go to the plank cover-edge stand or the fight pick reaching #8 before the arc lands.
