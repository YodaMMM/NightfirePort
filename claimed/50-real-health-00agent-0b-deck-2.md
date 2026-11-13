## Your task: `real-health-00agent-0b-deck-2`, 0700000b's 00 Agent real-damage walk crosses the deck

Branch from origin/master. walkreal_0700000b_live::walk_fresh_spawn_to_the_mission_end_real_damage_00agent is ignored; its ignore message holds the current ledger (death t17415 on the deck crossing, 22 hits; #4's 0x18 rounds from his column (1.62, 58.03, 32.11) cost 35.0 in 5; the jam close and the d3 deck fight call are in). Next lever, from that ledger: the fight pick finds no VISIBLE candidate from the crossing although world segment_bullet rays to #6/#8/#9 read clear - census both ray sets from the crossing spots (NF_RF_TRACE=1 + the [0b-ray] map), then either widen the d3 fight's reach to the world-ray set (opt-in, d3 only) or move the P2 aim so a fight opens from a visible spot; the dart sleep on the dartable #4 (state 0x68 dartable true at t17199) saves the 35.0. Helpers gated to 0700000b and difficulty 3, so the Operative replay stays byte-identical (replay_parity_0700000b green). Real input only, no top-ups, never lower damage or health. Remove the ignore when green, otherwise push the ledger. Run walkreal_0700000b_live, replay_parity_0700000b, interactables_0700000b_live.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/50-real-health-agent-06`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

**Branch:** `s/real-health-00agent-0b-deck-2`.

---
claimed-by: sandbox3 at 2026-11-13T03:36Z
