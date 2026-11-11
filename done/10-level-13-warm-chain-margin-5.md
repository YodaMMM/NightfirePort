## Your task: `level-13-warm-chain-margin-5`, 07000013 can be walked to the end on 00 Agent past the lobby-east posts

Branch from origin/s/level-13-warm-chain-margin-4, merge current origin/master, and read its docs/reports/level-13-warm-chain-margin-4.md, "The new blocker behind the wedge" and "What is left". The north-lobby wedge is fixed; the d3 walk now dies to the lobby-east posts #6 (rifle) and #7. Take its first lever: reach the (2.4, 13.9) flank alive, the route the d2 fight#6 trace proved, for example a south-side z 13.5-14 leg under the posts' sight lines, and take #7 first, then #6 from the flank. Its rejected levers are measured dead ends; do not repeat them: the pre-clear hook at 14-16 u, bridge_and_retry, and in-place creep or longer stands. Then the 13-leg walk with the drains. Judge each over several real runs. Helpers stay gated to 07000013 and the difficulty, so other walks stay byte-identical. Real input only, no top-ups, never lower damage or health. Remove the ignore when green; if it is still red, push the ledger and next lever in its ignore message. Run the whole walkreal_07000013_live binary and interactables_07000013_live.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-13-warm-chain-margin-5`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox1 at 2026-11-10T23:45Z

---
## Report (done, 2026-11-11T00:22Z)

Closed by the driver: the report is on `s/level-13-warm-chain-margin-5` (docs/reports/level-13-warm-chain-margin-5.md); the worker didn't run finish.sh.
