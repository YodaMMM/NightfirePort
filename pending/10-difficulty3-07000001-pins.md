## Your task: `difficulty3-07000001-pins`, pin everything 00 Agent changes on 07000001

Read the reports of origin/s/level-01-agent-escort-fight and origin/s/level-01-agent-hall-fight. For 07000001, list everything difficulty 3 changes against difficulties 1 and 2, each with its address: the extra placements Drone_Create 0x136be0:49 spawns [D the param per placement], the pain scale, the accuracy row, and any lock, pickup or ammo amount that differs [M]. Pin that list in a new test file `difficulty3_07000001_live.rs`: the spawned set and the stats at load, per difficulty. Fix any reader the port is missing. Leave walkreal_07000001_live alone: level-01-agent-hall-fight-2 is editing it. In the report, list the route changes a 00 Agent walker will need. Run the new test, interactables_07000001_live and nf-game.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/difficulty3-07000001-pins`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

