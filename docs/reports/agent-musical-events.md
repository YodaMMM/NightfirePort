# agent-musical-events: `UpdateMusicalEvents`' own attack-gate writes

## What the dispatcher writes [M 0x114e70]

`MusicVars` 0x2707a0 (u16, attackers needed), `DAT_002707a2` (u16, floor), `DAT_002707a4`
(window, seconds) and `DAT_002707a8` (sight distance) are the attack gate `NDrone2_SeenAndAttacking`
0x145150 reads. Besides the per-level machines, the dispatcher writes them itself:

| Lines | When | Write |
|---|---|---|
| :33-82 | the level restart | the level's `MapDroneData` row |
| :125-127 | e[10] set (the channel-10 music zones) | `MusicVars = 1` |
| :102-123 | e[10] low and the bonus count < 0 | the level's row |
| :130-152 | e[2] == 2 (not during a cut-scene) | the -99 row |
| :166-188 | the alert count (`MusicUnderAttackCount2`) negative | the level's row |

Every scan with no matching row writes count 2, floor 0, window 8, distance 25 (:79-82, :119-122,
:148-151, :185-188; MayhewB 0x112270:72-75 the same). The port's `GateWrite::Restore` fallback in
drone.rs wrote floor 8 and left the window: now count 2, floor 0, window 8, distance 25.

`MusicUnderAttackCount2` is not reset when it runs out, so while e[2] stays non-0/non-2 the
level's row goes back every tick, after the e[10] write: the last write of the tick wins [M order].

Disc rows [D MapDroneData 0x24f418]: 05 (2,0,2,35), 01 (2,0,8,10), 02 (2,0,8,30), 03 (2,0,8,10),
12 (1,0,8,20), 0b (1,0,8,60), 0c (2,0,12,65), 0d (1,0,100,100), 14/15/16 (2,0,10,40), -99 (1,0,10,50).
06 and 07 have no row and take the default.

## The port

`crates/nf-game/src/music.rs`: `update` pushes the writes above, in the game's order, onto
`gate_writes`; `AttackGate::apply` applies them (the -99 row is `GateWrite::Alarm`, its row seeded
by `wire_music` as `alarm_row`). On 06 and 07 inside a channel-10 zone, and on 01 too, one attacking
guard now starts the combat music; 12's row already reads 1. After an e[2] == 2 alarm (01, 03 music
triggers) the gate takes count 1, a 10-second window and distance 50 until the 600-tick bonus runs out.

## Tests

`crates/nf-view/tests/musical_events_gate.rs` on the real levels, guards held in a combat state
without sight of Bond:
- 06 in the channel-10 zone: one guard posts e[2] = 5 and a combat section is picked; the default
  row afterwards is (2, 0, 8, 25). Fails on the old code (no e[2] = 5), and fails with the old
  floor-8 fallback.
- 06 outside the zone: one guard does nothing, two start the combat music (passes on both).
- 03: e[2] = 2 seats (1, 0, 10, 50); 600 ticks on, 03's row (2, 0, 8, 10) is back. Fails on the
  old code. The alarm is written to slot 2 directly, not through a trigger placement.

## Unproven / notes

- `prev_event2` (the e[2] the drones post) feeds `SP_Update`'s start check for script 0x6000994,
  so more e[2] = 5 posts could in principle change when that script starts and move the stream.
  Not observed on the binaries run.
- 01's machine (MayhewA) is not on this branch; the gate write is level-independent.
