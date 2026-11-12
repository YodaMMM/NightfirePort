## Your task: `level-0a-civilian-slide-cels-4`, land the portal drawn gate (guards walk with their gait, no glide)

Start from origin/s/level-0a-civilian-slide-cels-3 merged with origin/master. The gate is the game's rule [M View_AddObjects 0x1e62d8 + NDrone2_DoAnimation 0x149f88]; what is left is the walks the gait shift moved, per the -3 report's ledger. Order: (1) walkreal_07000001_live's exit family (one walk, 4 tests: route around the RESTART LoadPointActivator at (-24.06, 6.46, 7.99), the hall doorway stands #10/#14/#15, keep the contact alive); (2) look hard at two that may be more than a re-tune: 08 agent walks into a live-wire hurt volume the old path avoided (-1.40 hits at (112.98, 13.47, -177.50)) - why does the route change, a guard pushing Bond?; 11's base real-damage walk wedges AIRBORNE at (21.51, 48.03, 15.89) near door (20.0, 46.8, 16.6) after falling off the south band - check whether a walking guard / door contact pushes Bond (guard capsule / door collision are game rules: name the arm); (3) 05 d3, 13, 15 agent, 11 sisters. Real input only, never lower damage. A walk that cannot be re-tuned in the window is #[ignore]d naming a task, but no more than three. Then the full suite with the fast runner, run to completion (start it early, check it, re-start if the sandbox kills it).

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-0a-civilian-slide-cels-4`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox5 at 2026-11-12T06:51Z

---
## Report (done, 2026-11-12T08:47Z)

