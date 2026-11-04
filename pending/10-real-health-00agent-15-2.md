## Your task: `real-health-00agent-15-2`, finish 07000015's walk on real health at 00 Agent

Branch from origin/s/level-15-agent-doors-2 and merge origin/master in. That branch makes the Agent walk green and pins the fence pops. Read docs/reports/level-15-agent-doors-2.md and docs/reports/real-health-00agent-12-13-14-15.md (origin/s/real-health-00agent-12-13-14-15 if it is not on master). walkreal_07000015_live `walk_spawn_to_the_omega_sector_exit_real_damage_00agent` is #[ignore]d: two measured costs total 132.3 before the plank legs. One is the opening burst, 63.0 in 9 hits, from the difficulty-3 0x50000b2 guard behind the spawn (#15 in the ledger, (72.35, 41.34, -9.44) at his hits). A pre-emptive stand from the gantry cost more, because the line is blocked. The other is the #526 ankle-beam rim-shot blast, about 69.3 at difficulty 3. Take the report's levers: set off #526 from door[32]'s threshold (about 14.9 u down the lane) with the door[32] guard's dropped 0x1a, whose arc clears the rim and whose case-0x4B chain sets #526 [M Explode_Propagate 0x17c518]; the door[38]-north grenadier stands; and read #15's spawn spot from the placements [D] to find a clear line or cover for the opening. Keep the Operative and Agent walks green. Remove the ignore when green; if it must stay, give the measured reason and the next lever. Real input only: no health, ammo or channel writes, and no teleports. Never lower damage or health to make it pass. Fix the port only where it differs from the game, with the address. Shared test helpers (crates/nf-view/tests/common/) change only behind a per-level opt-in. Run walkreal_07000015_live and nf-game, then every walkreal_*_live.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/real-health-00agent-15-2`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

