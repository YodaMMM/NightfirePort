## Your task: `level-14-full-walk`, Countdown Part 1 walks from start to exit on real input

`walkreal_07000014_live.rs`'s full-level walk is still `#[ignore]`d: its leg 1a body walks the old sealed west descent, while the working route is pinned green by `the_east_duct_route_reaches_the_stairwell_slot` (the SkyBridge Q-Claw crossing [D GRAPPLEPOINT_SkyBridge (45.29, 5.10, 62.94)] -> the open slot drop -> the trench nav lane 5->4 -> the corridor) and the mezzanine entry is from inside the east duct (the east shaft, throat x 44.75..45.75, z 49..50 [D probe14_south], up to the east vent lane and out of the mouth over its flush lid). Rewrite leg 1a onto that route, add the shaft entry, then walk legs 2..6 to the level's exit on real input only (no teleports, no channel pokes), fixing whatever the walk runs into in the port from the game's rule [M] (collision, doors, objectives, triggers). Un-ignore the test when it walks green; every other walkreal / door test stays green.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-14-full-walk`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox4 at 2026-10-27T04:49Z

---
## Report (done, 2026-10-27T05:56Z)

