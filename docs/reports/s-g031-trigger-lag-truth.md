# s/g031-trigger-lag-truth: which tick a touch plate fires on

This branch moves the shared random stream, because it carries s/g031-trigger-lag. That branch moves the guards'
wake tick on 09, which shifts every fight walk on 09.

## Verdict

**The lag is right for every plate measured.** A plate that Bond enters during tick N writes its channel during
tick N+1, never during tick N. This holds for TouchOnce plates, which update in the object walk, on three levels.

Why, from the decompile:
- `Collide_Update` 0x1ea398 clears every object's `+0xD0` contact list and rebuilds it (lines 25-53, then the
  pick loop). It runs once per tick, after the update walk [M 0x1336f8:238].
- `Trigger_Hit` 0x1b1fc8:18 reads the trigger's `+0xD0` list.
- `Trigger_Hit` is reached only through `Trigger_Update` 0x1b1a78. That is called from two places, and both run
  before that tick's Collide:
  - `Trigger_Calc` 0x1b1900 at the walk's head [M 0x1336f8:24], for type 2 (Touch).
  - The walk itself, for type 1 (TouchOnce) and the rest.
- `Collide_Pick` / `Collide_PickObj` [M 0x1eb320, 0x1eae80] add contacts only to the tester's own list. The ray,
  sphere and cylinder queries fill their own result lists, not a trigger's `+0xD0`.

## Live trace [L 2026-11-21, PCSX2-gdb, god mode held, no pad input for the plates]

Method:
- Breakpoints on the tick start 0x1336f8, after `Trigger_Calc` (0x133750), and before and after the
  `Collide_Update` call (0x133b40 / 0x133b48).
- A write watchpoint in break mode on `switch_channels[ch]` (0x26FC90 + ch). Each hit logs the PC and the
  channel value at the next stop.
- Cold-start plates: the level load is pushed with every breakpoint already armed, and ticks are counted from the
  level's first tick.
- Entered plates: Bond is first placed outside the volume, with a relink. Some ticks later, at a tick-start stop,
  he is moved inside within the same room, with no relink.
  - A relinking teleport delays the contact by one more tick, because Bond's own pick for that tick is still the
    old one. The first #757 run showed this: written in tick N+2. So the entered case uses the same-room move.

Every channel store below is `Trigger_Update+0x1b0` (0x1b1c28) called from the walk (ra 0x133838). The TouchOnce
`ch |= hit` stores 0 on the ticks before the hit.

| Level | Plate | Out ch | Bond enters | Channel 0 after | Channel 1 written |
|---|---|---|---|---|---|
| 07000005 | #36 reveal plate (cold start, spawn on it) | 0x45 | tick 1 walk | tick 1 | tick 2 walk |
| 0700000a | #58 spawn plate (cold start) | 0x47 | tick 1 walk | tick 1 | tick 2 walk |
| 0700000a | #757 hatch voice plate (lock ch 0x3A off) | 0xB7 | tick 8 (moved in at the tick start) | tick 8 | tick 9 walk |
| 07000009 | #248 car-park arrival plate (cold start, spawn on it) | 0x51 | tick 1 walk | tick 1 | tick 2 walk |

The port on this branch gives the same ticks. On the old order every one fires a tick early. The tests are in
`g031_object_order_live`: `the_09_arrival_plate_fires_on_tick_2_like_the_game`,
`the_0a_spawn_plate_fires_on_tick_2_like_the_game`, `the_0a_hatch_plate_fires_one_tick_after_the_entry`, and the
existing `the_05_reveal_plate_fires_on_the_tick_after_the_contact`. All four failed with the lag removed and pass
with it.

Not measured: a type-2 Touch plate (it is written inside `Trigger_Calc`). The decompile gives it the same phase:
it reads the same list, from the head of the next tick.

## The two red tests

- **0a hatch walk** (`walk_office_floor_qworm_keyboard_hatch` and its `_real_damage` sibling). Every game claim
  passes on the lag: the beams are untouched, no alert OR fires, the hatch loads 0700000b, and the #757 lock
  holds. Only the committed input recording differed: 14639 ticks against 14038.
  - That recording is the port's own replay pin, not a claim about the game.
  - It was re-recorded with `NF_REC_0A_WRITE=1`.
  - `walkreal_0700000a_live` (13 pass, 2 ignored as before) and `replay_parity_0700000a` pass.
- **09 `the_alarm_switch_fails_objective_1`**. The alarm chain itself is untouched by the lag: the switch press and
  the OR mux #278 are not touch plates.
  - Every 09 guard waits on ch 0x51 (placement param 3), which is plate #248's output. The game writes that
    channel on tick 2 [L above]. The old port wrote it on tick 1, so with the lag every guard wakes one tick later.
    That is the game's tick.
  - The walk's stealth harness then plays out differently. It runs out of darts at nav 15, and the level ends
    before Bond reaches the switch.
  - This is a walk re-tune on a shifted stream. It was left for the stream batch re-pin and not tuned here. The
    other 8 tests in `walkreal_07000009_live` pass.
  - Not done live: the switch press itself. The pad Square presses in the trace did not activate the switch from
    the placed pose, and that run ended on the Mission Failed menu with no cause traced. So the press-to-0x7C-to-0x61
    tick spacing is not measured.

## Tests run

- `g031_object_order_live`: 5 pass. With the lag removed, the 4 plate tests fail.
- `walkreal_0700000a_live`: 13 pass, 2 ignored.
- `replay_parity_0700000a`: pass.
- `walkreal_07000009_live`: 8 pass. `the_alarm_switch_fails_objective_1` fails as described above.
