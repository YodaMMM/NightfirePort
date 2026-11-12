## Your task: `real-health-agent-16-1b-4`, 07000016 and 1b can be walked on Agent with real health

Branch from origin/s/real-health-agent-16-1b-3, merge current origin/master, and read that branch's report plus the two Agent ignore messages. Restore the report files onto the branch if the merge drops them, as the last round did. 1b is the closer one, so start there: extend Drake's r < 8 aim band across the first 500 ticks of his fight, because his measured hits landed at 9.3 u outside it, or re-score cover against the ring's live positions (cannot be seen from where the ring is, not from where it spawns). Then 16's lever from that report. Do not repeat the measured dead ends. Real input only, no top-ups, never lower damage or health. Remove each ignore when green. If a walk is still red, push the ledger and next lever in its ignore message. Run the whole walkreal_07000016_live and walkreal_0700001b_live binaries.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/real-health-agent-16-1b-4`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
Main agent, fresh deaths on master 729f9694e (each ignored walk of the level, --ignored --test-threads=1; test names not captured):
walkreal_07000016_live: Bond died at tick 1020 (frame 1020) at (12.23,0.94,-89.00) armour 0.0
walkreal_07000016_live: Bond died at tick 14833 (frame 14833) at (-7.56,-22.28,-172.78) armour 0.0
walkreal_07000016_live: Bond died at tick 2176 (frame 2176) at (-0.19,7.96,-79.44) armour 0.0
walkreal_0700001b_live: Bond died at tick 11385 (frame 11385) at (46.01,-14.08,-3.00) armour 0.0
walkreal_0700001b_live: Bond died at tick 5619 (frame 5619) at (31.92,-1.68,-0.59) armour 0.0
- Main agent, named: the t1020 death is walk_the_chasers_defeat_completes_the_mission_on_real_damage_00agent - the upper-ring guards (y 6.7: #5/#2 skin 0x50000b2 and #3 0x500008c, weapon 0x1a) land 7.0 a hit from 22-28 u onto the floor-0 approach (the level_fights field doc's ring ambush); t14833 is the Operative real-damage walk (#14 at 1.6 u, weapon 0x14, the pit); t2176 the Agent walk.
