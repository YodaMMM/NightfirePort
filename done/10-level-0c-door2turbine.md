## Your task: `level-0c-door2turbine`, Chain Reaction Part 1's blockers on the real walk

`walkreal_0700000c_live::walk_the_whole_mission_from_the_start_to_the_0d_handoff` is green with stand-ins pinned by their blockers (read the level-0c-every-objective report). Fix them in order, each from the game's behaviour [M/D], then replace each stand-in with the real-input leg: (1) door `door2turbine` #281 at (40.0, 12.6, -210.4) does not open on the real E path (state holds 0x2; kind 0xDB, ch_open word 5, lock_flag 0) [M Player_Activate 0x1a5170 case 0x1C, the door's use/lock path] - a real port gap; (2) the plate #274 volume sits inside the north display case's glass box - whether the case glass body-blocks that cell in the game or the plate fires another way; (3) the display ground -> warehouse/yard descent (the ch-84 warehouse vent #180/#184?); (4) the ring's west/south gaps - what the game crosses them with; (5) the doorway-ledge -> yard descent. Real input only. Run the FULL suite and quote its total line.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-0c-door2turbine`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox1 at 2026-10-28T23:53Z

---
## Report (done, 2026-10-29T01:28Z)

Closed by the driver: the report is on `s/level-0c-door2turbine` (docs/reports/level-0c-door2turbine.md); the worker didn't run finish.sh.
