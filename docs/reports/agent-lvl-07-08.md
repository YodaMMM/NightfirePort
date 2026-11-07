# agent-lvl-07-08

Levels 07000007 and 07000008: placements, channels, objectives and level-gated decompile sites checked
against the port.

## Fixed

1. 07000008 zip line (#19, the game's one type-2 wire). The rider faced uphill and accelerated from a
   crawl (about 5 s). The grab faces down the slope [M 0x197130] and the ride is a constant 0.15 per tick
   along the facing, about 2.5 s: `Player_Update` zeroes BL+0x10 on every live tick, because the gate is
   obj+0xF4, the object state [M disasm 0x1937e0, 0x193830], not the substate.
2. 07000007 rope barriers and the doorway blocker. 0x3b objects had no collision. They now stand solid
   while obj+0xF0 lacks bit 0x20: #495 turns solid on ch 21, #496 between ch 9 and ch 0x48
   [M 0x18dbf8, 0x18d3b8, 0x1eae80].
3. Placed glass and breakables. Bond walked through the intact 07000008 control-room panes and fell out of
   the building. Placed 0x20 objects are now solid while alive [M 0x12c230, 0x12b970:60].

Tests: `zipline_07000008_live`, `solid_objects_07_08_live`. All four fail on master.

## Scoped out, open

- Mode-1 wall-hug blockers (07000005, 0700000d) stay non-solid: with them in, the walks there stop.
  Question for the real game: does a walking Bond stop at #324 on 07000005 while ch 113 is off?
- Panes stay non-solid on 07000001 and 07000014: the walks there cross panes they do not break first.
  Question: do those routes meet a whole pane in the real game?
- `Bond::tick_ctx` zeroes the move quad on climb ticks only, reading the obj+0xF4 gate as the substate. The
  game zeroes it every live tick, so a hand-over-hand rider should read as still to guard accuracy. Left as
  is: a 0700000d walk was tuned on it.
- 0xE1 destroyables are not in the collision set either [M 0x12c318 links them like the others]. Not changed.
- `DroneFunc_CheckAlarmRaised`'s ch 0x61 arm runs on every level [M 0x1453a8]; the port runs it on
  07000005/6 only. No 07/08 placement writes ch 0x61, so not changed.

Unproven: nothing here was checked against PCSX2; the fixes rest on the decompile and disc data.
