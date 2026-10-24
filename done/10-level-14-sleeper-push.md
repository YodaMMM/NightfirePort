## Your task: `level-14-sleeper-push`, 07000014: past the sleeper drop, the east corridor, then the infiltration walk

Master has the rewritten east-duct route (`walkreal_07000014_live`; read `docs/reports/level-14-east-duct.md` on `origin/s/level-14-east-duct`, merged — the report itself is not on master). The real-input traverse now stops at the drop where Bond lands on a sleeper drone: the port's actor-actor push recentres Bond on top of the carrier instead of moving him off it.

1. Find the game's drone-vs-player push [M — `Player_Collision` 0x194df0 / `Player_CollisionHandler` 0x1949a8's object contacts and the drone's collision response; which way the push resolves when the player lands on a drone's capsule: horizontal or vertical] and port it, with a test: Bond dropped onto a standing drone ends beside it, not on top.
2. Then the corridor east (x 41 -> 49 at z ~62.9), the x ~49 south run to the mouths at z 42.16 and the mezzanine. The grapple points #626/#657 at (45.30 / 40.78, 4.79, 42.86) sit at the vent mouths' level — check whether they, not the stairwell slot, are the player's route to the mezzanine [D + M].
3. Un-ignore `the_east_duct_route_reaches_the_stairwell_slot` and `walk_the_infiltration_to_the_delta_exit` as they go green.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-14-sleeper-push`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.



---
claimed-by: sandbox4 at 2026-10-24T18:25Z

---
## Report (done, 2026-10-24T19:26Z)

