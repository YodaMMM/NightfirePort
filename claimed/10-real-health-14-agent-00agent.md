## Your task: `real-health-14-agent-00agent`, make 07000014's Agent walk real, and finish its 00 Agent walk

walkreal_07000014_live `walk_the_infiltration_to_the_delta_exit_real_damage_agent` is named and documented as real damage, but its body is `with_agent_path_clear(|| walk_the_infiltration_to_the_delta_exit_at(2))` with no `with_real_damage`, so the health top-up stays on and its pass pins nothing about Agent health. Wrap it, as the Operative and 00 Agent walks are wrapped, and make it pass on real health. Then finish `..._real_damage_00agent`, which is #[ignore]d. Read docs/reports/real-health-00agent-12-13-14-15.md (origin/s/real-health-00agent-12-13-14-15 if it is not on master). The measured failure is a wedge, not a death: at tick 26340 the 1.72 landing resupply leg leaves Bond airborne at (33.84, 0.82, 30.95), 0.41 over the scoped floor and 2.70 short of the drop (32.50, 0.83, 33.29), and the leg cap panics. The report's levers: a difficulty-3 re-ground before the landing resupply, a direct `walk_to` the drop's cell, and the stairwell AIM-pair stands, since the first 50.0 health goes in the infiltration grind. Take them after checking why Bond floats there: if the port's grounding differs from the game, fix the port, with the address. Remove the ignore when green; if it must stay, give the measured reason and the next lever. Real input only: no health, ammo or channel writes, and no teleports. Never lower damage or health to make it pass. Fix the port only where it differs from the game, with the address. Shared test helpers (crates/nf-view/tests/common/) change only behind a per-level opt-in. Run walkreal_07000014_live and nf-game, then every walkreal_*_live.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/real-health-14-agent-00agent`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox1 at 2026-11-04T11:43Z
