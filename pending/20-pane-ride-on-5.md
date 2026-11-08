## Your task: `pane-ride-on-5`, intact glass really stops Bond on every level

Branch from origin/s/pane-ride-on-4, merge current origin/master, and read its docs/reports/pane-ride-on-4.md. The d3 walk still dies at 7433 in the ambush pocket with #3 unwounded; the landed 8.4 razor is the baseline to beat, and the door fan is the next lever. Then run the walkreal battery beyond 01 one level per tool call, because the 10-minute tool cap cut the 02-0b batch and the background runner is killed between calls. Record each level's result in the report. Remove the dead_pane_live ignores once every walk is green. Keep the kind-0/1 props out of the ride set and carry the live checks forward. Real input only, no top-ups, never lower damage or health, and level helpers stay gated to their level.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/pane-ride-on-5`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

