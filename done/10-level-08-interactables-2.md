## Your task: `level-08-interactables-2`, the four red 08 interactables

Continue `origin/s/level-08-interactables` (merge `origin/master` first; read its report — it has the census and the per-kind tests). Four of its tests are red, each with a lead in the report: (1) `music_plates_mux_into_the_map_sound` — the OR-mux #102 (out 0x68 = 90|91|92 [D]) never rises: check the port's mux OR arm against [M the 0x1b2c68 mux loops]; (2) `touchonce_plates_fire_on_the_walk_in` (#80's walk-in from outside its AABB); (3) `shooting_a_destroyable_kills_it_and_spawns_the_blast` (monitor #211: LOS and the bullet->destroy contact); (4) `pickups_take_on_the_walk_over` (the armour pick, parked translations). For each: decide port gap vs test setup with evidence [M/D], fix the port where it is wrong, fix the test where it is wrong. Every test in the file green (ignores only with a named blocker). Run the touched binaries and quote their totals.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-08-interactables-2`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox3 at 2026-10-30T17:55Z

---
## Report (done, 2026-10-30T19:07Z)

Closed by the driver: the report is on `s/level-08-interactables-2` (docs/reports/level-08-interactables-2.md); the worker didn't run finish.sh.
