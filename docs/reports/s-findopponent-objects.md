# s/findopponent-objects: FindOpponent's helicopter arm, the ally close sight probe, the scan rule

Both commits move the shared random stream. Commit 1 (the two arms) moves it on 07000008 once
the Hind flies; commit 2 (the scan rule) moves it wherever a team-2 drone fights (01, 08, 16).

## What a player sees

On 07000008 (The Exchange, the Hind at the bottom of the mountain), Zoe now turns on the
helicopter once it flies and fires at it, as the game does. Before, the port gave her no
target once the guards were gone. Her rounds do the Hind no damage yet (open item). From inside
the cabin her rounds can strike Bond on the way out; her skin's voice set makes every such hit
0 damage [M Drone_ModBulletDamage 0x139f84-0x139f88], so it costs no health, but the hit still
runs the pain path.

## The game [M]

- Class 0x3B is the helicopter alone: `Copter_Create` 0x17d340:62 is the only `obj+0xFF = 0x3b`
  store in the ELF; its state short `obj+0xF4` starts 0 [:63] and is 1 from the first
  post-gate tick [M 0x17dbb8:115-137].
- `NDrone2_FindOpponent` 0x1424e0 for a team-2 drone first walks the control objects
  [:123-147]: class 0x3B, `0 < +0xF4 < 4`, and the `Vec_DirToRot` pitch of (object - drone) in
  [-0.5235988, 0.17453294]; the first hit is the opponent (no range, no sight test, no budget,
  early return).
- A held class-0x3B opponent re-validates with `NDrone2_CanSeeObject` (object-position arm,
  0x176f18:81-96) and drops once `+0xF4 > 3` [:176-177].
- `DroneWeap_Ready2Fire` 0x178c48:60-96 has no health gate for a class-0x3B target, so the
  ally's fire is allowed.
- `DroneVision_OpponentVisibility` 0x175af0:24-30: under 2.0 to the opponent with a sight
  range, `NDrone2_CanSeeObject(drone, +0x170, 5, 0)` runs first; a clear ray is 1.0. For team 2
  the opponent is the drone's pick, so the port now probes that drone (or the copter) instead
  of skipping the probe.
- Commit 2, the scan (same function, team-2 drone list): the counter snapshot is taken at ENTRY
  [disasm 0x142554 `lhu $s7, 0x282($a1)`] and compared at EVERY visited node
  [0x143144-0x14314c]; the range is the engage range only when a held opponent survived
  re-validation, and only that survivor's node is skipped [:150-211]; the walk stops at the
  list's end and parks the cursor at the head [:245-247]. The port had the snapshot after the
  held probe, the check only after a probe, a wrapping walk, and the pre-validation opponent
  for range and skip.

## Census [D]

`nf_view::load` over `dev::LEVELS`: team-2 drones on 07000001, 07000002, 07000008, 07000016;
copter objects on 07000005 (2), 07000008 (1), 0700000B (1), 07000012 (1). Only 07000008 has
both. Turrets, decoys and allies are not class 0x3B; nothing else is.

## Tests

Commit 1:
- `crates/nf-view/tests/findopponent_copter_live.rs` (new, 4 tests, all fail with the arms
  switched off): the tick-level copter pick on 08; the walk's state and pitch gates and zero
  budget; the held copter's probe and state > 3 drop; the team-2 close probe giving 1.0 with the
  cone formula at 0.
- `crates/nf-game/tests/find_opponent_live.rs`: the tick test clears 07000005's ungated copter,
  which the borrowed ally would now take (correct behaviour, outside what that test pins).

Commit 2:
- `find_opponent_live.rs`: two new tests (the entry snapshot stops the scan at the first visited
  node; the walk does not wrap), both fail on the commit-1 scan.

Walks (master green on every one listed):
- enemy_accuracy_live: green on both commits.
- walkreal_07000002_live: green on both commits.
- walkreal_07000008_live: real_damage and real_damage_agent RED on commit 1 and 2 (Bond dies in
  the cabin fight, t25657-27022; the stream moved by Zoe's shots). The other 08 tests green,
  `the_ally_never_shoots_bond_on_the_walk` included.
- walkreal_07000016_live: green on commit 1; real_damage_agent RED on commit 2 (t18073).
- walkreal_07000001_live: green on commit 1 for `walk_from_the_spawn_to_the_exit_with_the_contact`;
  commit 2 reds it (the contact dies, t22156) and `following_mayhew_down_completes_the_level`
  (6000-tick cap onto the meet plate).

## Open

- A drone round hitting the copter's parts is not ported: the copter hit chains take Bond's
  rounds only (`update_player_bullets`), so Zoe's fire does the Hind no damage.
- The copter walk follows the port's copter order, not the control list's; no level has two
  copters beside a team-2 drone, so the order is never read.
- The walk pick's sight flag (the attack entry reads it) uses the object-position ray off the
  budget: the port's `ProcessDroneSight` ray runs at Bond only [S].
- Whether Zoe fires at the Hind from the cabin in a real run is not checked against a live
  capture.
- The red walks need a re-tune on the joined stream; no port bug was found behind them.
