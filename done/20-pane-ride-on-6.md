## Your task: `pane-ride-on-6`, intact glass really stops Bond on every level

Branch from origin/s/pane-ride-on-5, merge current origin/master, and read its docs/reports/pane-ride-on-5.md. The door fan is beaten (57.6 to 7.2); the d3 walk's last red in the 01 binary is the ambush pocket, mapped as the total ray asymmetry. Take its pocket root fix first. Then settle the 03 red: walkreal_07000003_live's real-damage Agent walk died at tick 106206 at (22.93, 6.37, 12.51) and has no baseline, so run it on current master in the same sandbox to see whether pane riding causes it, and repair it gated to 07000003 if it does. Then the walkreal battery from 04 to 0b and on, one level per tool call. Remove the dead_pane_live ignores once every walk is green. Keep the kind-0/1 props out of the ride set and carry the live checks forward, including the new check 6. Real input only, no top-ups, never lower damage or health, and level helpers stay gated to their level.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/pane-ride-on-6`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox3 at 2026-11-08T22:51Z

---
## Report (done, 2026-11-09T00:55Z)

