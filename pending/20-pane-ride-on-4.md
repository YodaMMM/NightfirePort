## Your task: `pane-ride-on-4`, intact glass really stops Bond on every level

Branch from origin/s/pane-ride-on-3, merge current origin/master, and read its docs/reports/pane-ride-on-3.md. Take "What's left" in order. First the d3 walk's last 4.6 hp: its bleed map shows the door volley lands inside the 3.0 band, and a deeper hold line (8.0) made it worse (death 5761), so the lever is the spacing of the crossing fight, not the hold depth. Then remove the two dead_pane_live ignores and run the full walkreal battery in the foreground (the background runner is killed between tool calls in this sandbox), the ai binaries and nf-game. Keep the kind-0/1 props out of the ride set and carry its live checks forward, including the d3 crossing volley's pellet doubling at the aperture. Real input only, no top-ups, never lower damage or health, and level helpers stay gated to their level.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/pane-ride-on-4`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

