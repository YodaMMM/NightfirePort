## Your task: `guard-castle-chat-gate-4`, the castle chat pair lands with every walk green

Branch from origin/s/guard-castle-chat-gate-3 and merge origin/master; read docs/reports/guard-castle-chat-gate-3.md there. The per-tick look_for_opponent gate (bit 4 = held sight past the reaction time [M 0x177ed0:26-28, 0x175ef0:41-44, 0x176a28:200-202]) stays - it is the game. Reds 1-2 (guard_parity_live, guard_ai_other4_live) are corrected on that branch: re-run both on the merged tree and confirm. Red 3 (rook06_realpath_live, the switch-16 guard d#3 going stationary 0x19 via the FirstSightState prop-0x1b coin): the d3.mode == 0 assert pins the result of a shared-RNG draw, not a disc or decompile fact - the walk must hold for EITHER outcome of that coin: re-derive the guardhouse-court leg on real input so it works whether d#3 takes a combat move or the stationary sniper tail [M 0x148010:41-45 vs :100-131], and replace the mode pin with an [S] log line. Red 4 (walkreal_07000005_live::the_door_band_exit_chains_to_07000006_walked_real_damage: #12/#13 now engage at 5-7 u): re-derive that leg on real input for the held-sight rule (cover, approach line, take them from range before they close). Never lower damage or health, no top-ups. Run guard_parity_live, guard_ai_other4_live, rook06_realpath_live, walkreal_07000005_live, walkreal_07000006_live, fix_npcs_live and nf-game --lib (vision_bit_census).


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/20-guard-castle-chat-gate-4`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.
