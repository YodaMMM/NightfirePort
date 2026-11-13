## Your task: `real-health-00agent-01-hall-4`, 07000001's 00 Agent real-damage walk gets past the hall door phase

Branch from origin/s/real-health-00agent-01-hall-3 (its ledger is not on master yet) and read docs/reports/real-health-00agent-01-hall-3.md there, "What's left" and the next lever it names: the advance dance with the hard exchange budget. Land that lever, measure it on its own real run, then carry the walk through the AND gate and the rest of its route (the post-AND route is untouched and was green through the AND on the older streams). The report's live notes say the door-phase crowd reacted to a standing Bond through the aperture but not to a crouched one; use that only as a lead and confirm it in the port's own runs. Keep the ledger in the test's ignore message. Helpers stay gated to 07000001 and difficulty 3, so other walks stay byte-identical. Real input only, no top-ups, never lower damage or health. Remove the ignore when green; if still red, push the ledger and the next lever. Run the whole walkreal_07000001_live binary and interactables_07000001_live.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/real-health-00agent-01-hall-4`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox1 at 2026-11-13T07:20Z
