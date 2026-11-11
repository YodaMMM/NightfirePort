## Your task: `level-14-skybridge-retune`, 07000014's Operative and Agent real-damage walks and the tripbomb route are green

Branch from current origin/master and read the ignore messages in crates/nf-view/tests/walkreal_07000014_live.rs. The 00 Agent walk belongs to real-health-14-00agent-9; leave it alone. Repair three walks, one per tool call. First walk_the_infiltration_to_the_delta_exit_real_damage, then its _agent sister: after the claw wall fix, the route's SkyBridge crossing is a stand-in (the claw bolt dies on the duct wall) and the shifted clock leaves Bond short on the pit band. Find the crossing the game's own claw path allows; if the claw-on-duct-wall behaviour itself looks wrong, check it against the decompile before re-routing. Then walk_the_tripbomb_route_pick_up_place_and_shoot_it, which wedges on the sill row with no motion. Find why there is no motion before re-routing. Helpers stay gated to 07000014 and the difficulty. Real input only, no top-ups, never lower damage or health. Remove each ignore when green; if a walk is still red, push the ledger and next lever in its ignore message. Run the whole walkreal_07000014_live binary and interactables_07000014_live.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-14-skybridge-retune`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

