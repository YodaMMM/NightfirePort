## level-0c-interactables-2 (2026-10-31, s/level-0c-interactables-2)

Branch from origin/master (81da6c47, batch-1031l). Task: 0c — every player-touchable kind
behaves like the game. `interactables_0700000c_live.rs` has 5 tests; using
`interactables_0700000b_live.rs` as the model (one live test per KIND of player-touchable
placement), run the census of 0700000c [D every placement kind], and for every kind a player
can touch, add the live test that drives it through the real path (walk/E/shoot) and asserts
the game's result [M the class's Update/Activate; D its params], with a sister that fails on
the wrong result. Fix any port gap the tests expose, with evidence. Run
interactables_0700000c_live, walkreal_0700000c_live and nf-game; quote the totals.

## Progress (2026-10-31, ~21:10 UTC)

- Branch created from origin/master (81da6c47). Existing 0c coverage on master noted:
  interactables_0700000c_live.rs (5 tests), interact_0700000c_live.rs, walk_0700000c_live.rs,
  walkreal_0700000c_live.rs (+ barrelrow/eastroute), census_0700000c.rs,
  docs/levels/0700000c.md + -audit.md. Census starts next.

## Progress (2026-10-31, ~22:15 UTC) — the census, the gap, the totals

- Census of 0700000c [D the placements dump + the triggers dump]: 575 placements, 34 trigger
  rows. Player-touchable kinds and their real-path drives: Door E (interact_0700000c_live
  `the_use_key_opens_door2turbine_and_the_sister_keeps_it_shut`), TouchOnce plate + SoundTrigger
  + the global-voice rule (interactables…c `the_hint_plate_…`), mission-trigger walk (interact…c
  `walking_the_mission_trigger_fires_channel_43_once`), armour pickup (interact…c), weapon pickup
  + medal-gated bonuses (interactables…c), bullet-filter destroyables (interactables…c +
  interact…c), the secret-door E+body AND (interactables…c), zipline + Q-Claw grapple (interact…c),
  music trigger event (interact…c), param-less fuseboxes inert (interact…c), ladders
  (walkreal_0700000c_live + ladder495), the hand-over-hand wire crossing (walkreal), the
  LoadLevel gate (walk_0700000c), the vent hinge/AND/SS chain (fix_level_0c_live +
  fix_0c_hatch_live). Absent from the disc: buttons/switches 0x29, lifts, safes, keypads,
  monitors, vehicles, shoji [D docs/levels/0700000c.md §7.1; the kind histogram]. The 0xDC
  skylight touchplates #275-277 are inert by disc data (no volume, tris 0) [D triggers dump].
- The census gap: the AMMO-kind pickup take had no 0c drive (other levels' files drive it; no
  0c test touches #477/#520/#521). Added `the_sniper_ammo_pickup_credits_the_pool_and_the_tower_
  pair_sits` to interactables_0700000c_live.rs: the walk-over take of #477 (PICKUP_300AP (Sniper),
  p0=1/p1=31/p2=10 [D]) pins the one logged `Ammo { weapon: 31, rounds: 10 }`, the refused tick's
  clip fill (clip[31] = 5 [M the fill arm 0x1a8d18; the 0-return gate at 0x18e650
  joined_r0x0018ebf0 leaves the pickup for the next tick]) and the pool's 15 (the leaked 5 + the
  re-take's 10) — the game's own two-tick dance, replayed bit-faithfully by the port; owned[31]
  stays false (case 1 has no equip arm [M]); sisters: #477 and the tower pair #520/#521 sit
  before, #520 sits after.
- No port code changed: the take path already matches the decompile.
- Totals: interactables_0700000c_live 6/0, walkreal_0700000c_live 2/0, nf-game 953/0 (9 ignored).
  Full suite chunked on this tree: 244/996 binaries recorded, 877 passed, 2 failed, 67 ignored —
  the two are the sandbox's (fix_0c_pylon_live needs a GPU adapter; mp_online_failures' networked
  binary is killed/hung here), not code failures. The state file /home/z/suite_0c_state.json
  resumes the remaining 752 binaries.
