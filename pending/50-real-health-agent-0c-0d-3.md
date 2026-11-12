## Your task: `real-health-agent-0c-0d-3`, 0700000c and 0700000d can be walked on Agent with real health

Branch from origin/s/real-health-agent-0c-0d-2, merge current origin/master, and read that branch's docs/reports/real-health-agent-0c-0d-2.md plus the ignore messages on `walk_the_whole_mission_from_the_start_to_the_0d_handoff_real_damage_agent` and `walk_arrival_to_the_c5_door_real_damage_agent`. Take its levers in order: the pitch-convergence fix in the aimed stand for 0d (one shared-helper change, opt-in, default off), then the early band sweep again, then 0c's #30 stand east of the hall edge or #31's fire-window pre-emption. Do not repeat its measured dead ends. Real input only, no top-ups, never lower damage or health. Remove each ignore when green. If a walk is still red, push the ledger and next lever in its ignore message. Run the whole walkreal_0700000c_live and walkreal_0700000d_live binaries, and if a shared helper changes, every walkreal_*_live.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/real-health-agent-0c-0d-3`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
Main agent, fresh deaths on master 729f9694e (each ignored walk of the level, --ignored --test-threads=1; test names not captured):
walkreal_0700000c_live: Bond died at tick 6148 (frame 6150) at (54.02,6.01,-157.98) armour 0.0
walkreal_0700000d_live: Bond died at tick 3749 (frame 3749) at (213.07,17.37,-234.51) armour 0.0
walkreal_0700000d_live: Bond died at tick 5020 (frame 5020) at (213.07,17.37,-234.51) armour 0.0
