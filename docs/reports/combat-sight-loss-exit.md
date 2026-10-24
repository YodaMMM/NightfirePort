# combat-sight-loss-exit — the combat router's sight-loss exit to NoOpponent, without stalling the walks

Task: on master the router head in drone.rs (the `from` match on COMBAT / COMBAT_OUT_OF_RANGE /
COMBAT_NO_SIGHT / AIM_*_FIRE with `!self.seen && self.unseen_ticks > 15`) is disabled with `if false`,
and `(NO_OPPONENT, msg::UPDATE)` returns to Attack on `has_opponent(frame)` alone. With the exit on and
the NO_OPPONENT rule `(team == 2 && has_opponent) || (team != 2 && seen)`, eight tests broke. Work in
progress — updated as steps land.

## Plan

1. Port the router head from [M 0x148338:19-31] with the game's own hold: `HasOpponent` 0x14ad58 reads
   the +0x238 stamp (30 frames), not the port's 15-tick `unseen_ticks` stand-in. Cite where +0x238 is
   written.
2. NoOpponent 0x60 case 3 [M 0x1650a0:30-49]: what re-points +0x170 for a team-1 guard. Port that writer
   rather than a `seen` stand-in.
3. Every broken test green with the change on, plus the 07000001 hall-guard case (walkreal_07000001_live
   and escort_follow_07000001_live).

## Status

- (in progress)
