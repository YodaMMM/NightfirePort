## Your task: `real-health-14-00agent-9`, 07000014 can be walked to the exit on 00 Agent with health to spare

Branch from origin/s/real-health-14-00agent-8, merge current origin/master, and read that branch's report. The walk clears the exit at 3.8 health, one hit from death, so it is green only by luck. Take its margin lever: the first #56 engagement still costs 20.0 (67.5 to 47.5 at tick 5157) because the fight starts on clip 0 with 5 spare and stalls on a reload under the hunter's burst. Restock before the drain's engagement, or scope the head window into the drain's first #56 round, and re-pin the ledger with a real margin. Then the d2 wrap if the drop-west pocket crossing stops re-rolling. New flags are opt-in and default off. Real input only, no top-ups, never lower damage or health. Run the whole walkreal_07000014_live binary and interactables_07000014_live, and if a shared helper changes, every walkreal_*_live one level per tool call.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/real-health-14-00agent-9`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox3 at 2026-11-11T03:18Z

---
## Report (done, 2026-11-11T04:14Z)

