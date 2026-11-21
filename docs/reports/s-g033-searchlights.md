# s/g033-searchlights — G-033 searchlight remainder

**Moves the shared random stream** when a searchlight head is broken (smoke puff count, 6 draws
each) or a guard round crosses a head (a new break). Kept on this branch for the stream batch.
The walkreal runs of the three searchlight levels stayed green unchanged.

## Census
`Level::parse` over `nf_view::dev::LEVELS` (placement_kind_census): 0xDE placements on
07000005 (29), 07000006 (20), 07000041 (5); no other level.

## Checked against `Searchlight_Update` 0x1cb900 (decompile + disassembly)
Matching already: sweep, tracking steer and cone clamps (the game's second clamp re-reads the first
clamp's result; with half >= 0 it can never fire after the first, so the port's else-if is
equivalent), arming at 2.0 from the spot, the 15 s lost limit, the beam, the corona gates, the
alarm write gate, the break walk and its damage gate.

## Fixed
1. **Broken-head smoke** — the port scanned every head on every head's update: a broken head made
   one puff per head on the level per 7th frame (4x on 07000006, 7x on 07000005) and could smoke
   on the break tick. The game smokes only its own head, in the state-2 arm, before the break walk
   [M 0x1cb900:373-385, 406-418]. Now `Searchlights::smoke`, filled by that arm.
2. **Relight tick** — on the channel turn-on the game writes state 0, plays 0xE5 and jumps to the
   contact walk [M disasm 0x1cbd40-0x1cbd8c, `b 0x1cc1d4`]; the port also ran the spot ray, beam and
   corona and force-drew every link that tick. Now the parts come back on the next tick.
3. **Kill tick** — after the off-channel kill the game falls into the state dispatch and runs the
   contact walk [M disasm 0x1cba20-0x1cba5c, 0x1cbd40]; the port returned first, so a bullet on that
   tick did not break the head.
4. **Guard rounds** — the head's walk reads every class-5 contact [M 0x1cb900:411-417]; the port fed
   only Bond's rounds. Drone rounds' per-tick segments now reach the next tick's walk.

## What a player would notice
A shot-out searchlight smokes at the game's rate (it smoked 4-7x too thick); a guard's stray round
can shoot a light out; a light switched on by its channel shows its beam one tick later, as on PS2.

## Tests (fail on the old code, pass now)
- nf-game `searchlight_live`: `channels_and_tracking_behave` (relight tick, re-pinned),
  `a_bullet_on_the_kill_tick_still_breaks_the_head` (old: state 3),
  `the_broken_head_smokes_every_seventh_frame_from_the_tick_after_the_break` (new API).
- nf-view `searchlight_bullet_07000006_live`: `a_broken_head_smokes_once_per_seventh_frame`
  (old: 12 puffs, now 3), `a_guard_round_breaks_a_courtyard_head` (old: never breaks).
- Green: all nine searchlight nf-view binaries, nf-game lib `searchlight`/`gas` filters,
  walkreal_07000005 (16 ok, 1 pre-existing ignore), walkreal_07000006 (9), walkreal_07000041 (15),
  walkreal_07000041_wall (2).

## Unproven / open
- The guard rounds always use the segment pick; the game's slow-tick point pick (raw step < 0.125)
  for a guard round is not traced. The contact tick of both feeds (same tick for Bond's rounds,
  next tick for guard rounds) follows the port's existing pass order, not a live capture.
- The smoke test on 07000006 puts the head in state 2 directly; the break itself is covered by the
  real-path shot test.
- Draw-side, not changed: the corona's sight test starts at Bond's eye; the game uses the camera
  position (`glb_viewer+0xC0`) [M 0x1cb900:338-340]. They differ in third person.
- Last-ulp VU0 rounding of `Mat_Align2Dir` (visual, +-4 ulp).
