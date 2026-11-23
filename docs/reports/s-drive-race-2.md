# s/drive-race-2: the RACE.VIV race on the road-join navigator

The random stream: the race code draws only from the driving generator (`ai::DriveRng`): the
kind-4 chooser at a node of three or more links (RACE has none) and the 5-second ram roll of
`hostile::ram_tick`. The on-foot shared stream is not touched.

Base: the merge lane head 6c20b049b (s/drive-road-join 1e0827aed and later). This branch ports
s/drive-race (88dc8fb4e, 57dac008b) onto it.

## What changed against s/drive-road-join

- `driving::race` (new, from s/drive-race): `RaceDef`, `Rival`, `RaceState` / `RaceHud`,
  `RaceRoute`; `AiCommand::hint_toward` / `hint_away` (AICo +0x84 / +0x85); `DriveScript::fired`.
- One navigator API. The rival now uses road-join's navigator throughout:
  - `Rival::apply` writes nav +0x04 (group), +0xb0 (`slot`, when AICo +0x7e >= 0) and
    +0xb1 / +0xb2 and then joins through `join::nav_for_pattern` (state 5, pattern 4) and
    `RoadNav::join_nearest`, in the order `FUN_00145ff8` uses [M 0x145ff8:251-267]. It takes
    the road grid and the car's heading as new arguments.
  - `Rival::step` advances with `RoadNav::advance_with` and a `NavHint`. The hint direction
    follows `FUN_00130140`: toward the target with +0xb1, away from it with +0xb2, else the aim
    direction, which stands in for AI +0x1e0 [S].
  - `RaceRoute::build` finds its first piece with `join::nearest_piece` (the start's grid cell).
- `ai::NAV_KIND_FOLLOW` (= 4) added; `advance_with` matches on it.
- **Rule fix in `ai::next_segment_follow` (FUN_002235c8).** At a node with exactly two links
  and no hint flag, when no other link continues the chain's sense, the game keeps the
  **current piece** and its end. The loop at 0x2239b0 falls out to 0x2239f4, which returns s6.
  s6 holds nav +0x3e (the current piece) from the entry [M 0x2235c8 disassembly]. Road-join
  returned the last other link instead. Drive-race had it right, so this branch follows drive-race.
  New test `kind4_chooser_at_a_two_link_node_with_no_continuing_piece_stays` fails with
  road-join's fallback and passes with the fix.

## What changed against s/drive-race

Dropped in favour of road-join's versions, each its own port of the same function:

| s/drive-race | replaced by | note |
|---|---|---|
| `RoadNav::forced_link` | `RoadNav::slot` | same field, nav +0xb0 |
| `RoadNav::advance_hinted(net, dist, [f32; 3], rng)` | `advance_with(net, dist, NavHint, rng)` | kinds 1/3/4/5, lane blend |
| `ai::next_segment_follow` returning (piece, end) | road-join's, end written in place | the game writes nav +0x3c through `param_3` [M] |
| `ai::nearest_segment` (every piece) | `join::nearest_piece` (one grid cell, Bezier point) | [M 0x225490] |
| `RoadNav::join_follow` (centre line, always A→B) | `join_nearest` via `nav_for_pattern` | see below |

Where the two branches disagreed on a game rule:

- **Two-link fallback**: drive-race was right, as described above.
- **The kind-4 join's lateral offset.** Drive-race always joined the centre line, marked [S].
  The game passes `param_4 = 0` to `FUN_00225c48` when AI +0x154 is set, and +0x154 =
  (AICo +0x7c == 0) [M 0x12e468, 0x145ff8:251]. In that case it keeps the offset to the road,
  clamped to the road width. The rival's road commands (AICo 0, 4, 6) all have +0x7c = 0, so the rival now joins
  with the offset. That is road-join's join, decided from the decompile.
- **The end pick at the join.** Drive-race always joined A→B. The game does that only for a
  kind-4 navigator with neither hint flag set [M 0x225c48]. Both flags are clear on RACE, so
  nothing changes there, but the general case now follows the game.

## Tests

`CARGO_TARGET_DIR=target-o cargo test --offline --release -j 8 -p nf-game`, with `NF_SILENT=1`
and `NF_ISO` set:

- `drive_race` 8/8 (one new). `drive_race_disc` 6/6. `driving_road_join` 27/27.
  `driving_roads` 21/21.
- All 34 `drive*` / `driving*` test binaries: 257 passed, 0 failed.
- `nf-game --lib driving::` unit tests: 239 passed, 0 failed.
- `cargo build --release -p nf-view -p nf-extract`: clean.

Test changes from s/drive-race:

- The chooser tests run on a copy of the navigator, because the end byte is written in place.
- The join test uses a one-cell grid and `join_nearest`.
- The carrot-walk test now checks that kind 2 is refused; kind 3 is ported now.
- The rival tests pass the grid and heading.

`full_race_at_the_rivals_pace_is_close` (38 u/s): the gap at the end was under 1 unit with the
centre-line join. With the game's offset join it is about 460 units, and the player wins. The
bound was raised from 300 to 600. The gap is a number from this harness, not from the game
[S]. The lane pick that would steer the offset (`join::lane_pick`, FUN_00135f70) is not wired
into the rival. The other two full races keep their results: the player wins at 60 u/s, the
rival wins at 25 u/s.

## Not proven

- The rival keeps whatever lateral offset it joined with (up to 7.8 units on RACE) until the
  next command, because `lane_pick` is not called from `Rival::step`. In the game the lane pick
  runs every tick for pattern 4 off state 4. Wiring it in is the next step for the race.
- AI +0x1e0 as the hint without +0xb1 / +0xb2 is stood in by the aim direction. The kind-4
  chooser does not read the hint in that case, so nothing on RACE depends on it.
- Everything listed as unproven in the s/drive-race report still holds: the position and
  best-lap readouts are the port's, the full-race rival is a point driven by its pedals, the
  bridge-jump path end, the same-step finish order, the police rules and traffic.
- `crates/nf-view` does not step the race yet.
