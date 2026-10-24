## Your task: `combat-sight-loss-exit`, the combat router's sight-loss exit to NoOpponent, without stalling the walks

On master the router head in drone.rs (the `from` match on COMBAT / COMBAT_OUT_OF_RANGE / COMBAT_NO_SIGHT / AIM_*_FIRE with `!self.seen && self.unseen_ticks > 15`) is disabled with `if false`, and `(NO_OPPONENT, msg::UPDATE)` returns to Attack on `has_opponent(frame)` alone. With the exit on and the NO_OPPONENT rule `(team == 2 && has_opponent) || (team != 2 && seen)`, these broke: ai_missing_states_live (no_opponent_waits_then_attacks_when_one_exists: stuck in 0x60), fix_searchlights_guards_live, fix_searchlight_alarm_live, abseil_elevator_live, npc_doors_live (06 chase door), bathhouse_voices_live, fix_finale_live, walkreal_07000001_live.

1. Port the router head from [M 0x148338:19-31] with the game's own hold: `HasOpponent` 0x14ad58 reads the +0x238 stamp (30 frames), not the port's 15-tick `unseen_ticks` stand-in. Cite where +0x238 is written.
2. NoOpponent 0x60 case 3 [M 0x1650a0:30-49]: what re-points +0x170 for a team-1 guard. Port that writer rather than a `seen` stand-in.
3. Every test above must pass with the change on, plus the 07000001 hall-guard case the exit was for: a sight-lost guard must not sit in 0x5c forever (walkreal_07000001_live and escort_follow_07000001_live green).

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/combat-sight-loss-exit`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.
