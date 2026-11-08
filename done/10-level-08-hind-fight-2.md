## Your task: `level-08-hind-fight-2`, the 07000008 Hind fight can be won on real input on every difficulty

Branch from origin/s/level-08-hind-fight, merge current origin/master, and read its docs/reports/level-08-hind-fight.md. Take "What's left" in order. First debug the missile-dodge scan against a live missile's real Bullet fields (one `eprintln!` in the scan loop settles it), so the -55 close blast becomes an edge hit and the reloaded clip's four rockets finish the fight (202 + 4 x 101 = 606 > 598). Keep the station shifts, and consider timing them to moments with no missile inbound. Then lift the same counter-tactic onto the d1 and d2 sisters under their own gates, and un-gate the spot assert from the ignored test once d3 is green. Helpers stay gated to 07000008 and the difficulty, never lower damage or health, and keep the rocket count the walk really carries. Remove each ignore when green; if a walk is still red, push the ledger and next lever in its ignore message. Run the whole walkreal_07000008_live binary, the copter and homing binaries and nf-game.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-08-hind-fight-2`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox5 at 2026-11-08T11:31Z

---
## Report (done, 2026-11-08T12:19Z)

Closed by the driver: the report is on `s/level-08-hind-fight-2` (docs/reports/level-08-hind-fight-2.md); the worker didn't run finish.sh.
