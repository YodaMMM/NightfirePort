## Your task: `los-mask-repins-0c-2`, Chain Reaction's three walks green under the game's guard sight

Branch from origin/s/los-mask-repins-0c (its ignore messages carry this round's ledgers) and merge origin/master. FIRST settle the landing-shot question in the sim: with the walk at the landing and strip guard #35 (0x5000087) standing at (-7.07, 12.24, -218.60) in AimStandFire, cast Bond's real fire ray at the guard and report what it hits (the lip, the guard, nothing) with the surface's id; compare with the game's bullet test [M Bullet_CollisionHandler / Collide_Intersect] and fix the port if the round should land. Then take the ledgers' next lever (the lip-top path west of the guard's capsule, then a held slide east past his AimStandFire cycle) and un-ignore walk_the_whole_mission_from_the_start_to_the_0d_handoff, its real_damage sister and walkreal_0700000c_eastroute_live::the_ground_route_crosses_to_the_east_side. Real input only, never lower damage or health. Run walkreal_0700000c_live, walkreal_0700000c_eastroute_live, walkreal_0700000c_barrelrow_live and interact_0700000c_live.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/30-los-mask-repins-0c-2`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

---
claimed-by: sandbox4 at 2026-11-10T21:12Z

---
## Report (done, 2026-11-10T22:20Z)

Closed by the driver: the report is on `s/los-mask-repins-0c-2` (docs/reports/los-mask-repins-0c-2.md); the worker didn't run finish.sh.

---
## Report from the worker (done, 2026-11-10T22:49Z)


---
## Report from the worker (done, 2026-11-10T22:49Z)

