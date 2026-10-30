# level-15-interactables — every interactable kind on 07000015

Branch `s/level-15-interactables` (master b3c4e4a1 + the task's files; origin/master did not
move during the session). The task: one live test per KIND of player-touchable placement on
07000015, driven the way a player drives it, each with a sister; census every player-touchable
placement [D placements + docs/levels/07000015.md §7]; add the missing kinds; fix every port
gap the tests expose.

## What's done

**The census** was already in `docs/levels/07000015.md` §7 (the earlier level-15 audit/walk
tasks), backed by the dump tool `crates/nf-view/tests/census_07000015.rs` (opt-in `--ignored`,
re-run this session over the slim ISO, 968 placements across the 55 maps, 38 kinds [D]). I
re-ran the dump and compared every player-touchable kind against the existing tests. The
kind → test map after this task (`interactables_07000015_live.rs`, plus the level's walk tests
where the kind's operation is a mission walk):

| Kind | Placements [D] | The test |
|---|---|---|
| TouchOnce 0xEA | ×13 | `the_spawn_touch_reveals_the_objective` (the reveal plate → ch 0x4b → the objective announces), `the_east_room_touch_wakes_its_squad` |
| Pickups 0xF0 | ×14 | `the_pickups_take_on_the_walk` (both Pickup 007s consume on the walk-over, the walked route) |
| Doors 0xdb | ×54 | `plain_door_opens_and_the_dead_keypad_door_refuses` (the E opens; the 0xc8 lock with no writer refuses); the keypad trio opens in the burn test; the 4 straddle exit doors ride `walkreal_07000015_live` |
| Keypad locks 0x2E | ×3 | `the_keypad_burn_unlocks_its_door` + the pickups test's second burn (the Q-Decryptor beam → out channel → the door opens) |
| Load-level exits 0xE8 | ×2 | `walk_07000015_live` (the USE exit loads 07000016; the gated #673's auto-fire pinned off — ch 0x42 has no writer [D]) |
| Destroyables 0xE1 | ×11 (8 barrels + 3 BoxSatellites) | **NEW** `the_room_touch_smashes_its_satellite_through_the_kill_channel`, **NEW** `the_oil_barrel_dies_to_the_gunshot_and_writes_its_death_channel` |
| Death plates 0xE4 | ×9 | `the_silo_fall_dies_on_the_death_plate` |
| Ladders 0x22 | ×3 | `the_dead_ladder_never_grabs_at_point_blank` + its mesh sister (admission-dead, the level's own data) |
| Apocalypse 0x4a | ×4 | the east-room test (the ch-0xf6 whiteout latches armed, the fade runs) |
| Shooter 0xd2 | ×1 | `the_shooter_fires_and_the_sound_plays_on_the_room_touch` |
| Sound trigger 0xF9 | ×1 | the same test (sound 1402 on ch 0xfb) |
| Mines 0xFE | ×9 | `the_mine_explodes_when_bond_walks_into_its_beam`, `the_mine_blast_is_a_persistent_object_that_spawns_no_debris` |
| Spawner 0xF1 | ×1 | **NEW** asserts in `the_east_room_touch_wakes_its_squad` |
| Drones 0x0F | ×20 | **NEW** asserts in the same test (the spawned squad present + wake-armed) |
| Start 0x24 | ×1 | every load + `walkreal_07000015_live`'s cold-start spot assert |
| Cover 0xE5/0xE6, AI_Point 0xE9 | ×20 | not player-touchable — the AI reads them (documented absence, the 014-report convention) |
| Emitters 0xF2, coronas 0xFC, decor | — | draw-only, not player-touchable |

**The two new destroyable tests** (the kind had no test on this level; the census row claimed
only a 0700000d family proof):

- `the_room_touch_smashes_its_satellite_through_the_kill_channel` — `Destroy_Create` 0x12c318
  stores placement param 1 into tag+0x10 [M disasm 0x12c3b8-0x12c3bc: `lhu 0x30($s0)` →
  `sh 0x10($a0)`] and `Destroy_Update` 0x12c4f8's second arm smashes the tag while that channel
  is ON (the word clears, :32; the port's `Props::channel_kills`, proven live on 07000008's
  fusebox family). The east room's BoxSatellite (69.52, 43.41, 10.26 [D census]) carries
  p1 = 0xf6 — the room touch #717's own out channel — so the real walk into #717 (no shot
  fired) smashes it. Sisters: the tag sits alive with the parsed kill channel before the walk;
  the 0xed/0xec satellites (channels nothing fires on this leg) survive.
- `the_oil_barrel_dies_to_the_gunshot_and_writes_its_death_channel` — the great-hall barrel
  (0.65, 40.30, 28.99 [D census]) carries health 10 (p3), death channel 0xfd (p0 — the level's
  only 0xfd writer/reader [D census]) and the bullet mask p4 = 0xf8 → tag+0x14 = ~0xf8 = 0xff07
  [M disasm 0x12c40c-0x12c420: the word load, the zero check, the `nor`]. The handhelds'
  weapon word 0x0008 ANDs the mask to 0, so the shot is ADMITTED ([L ee_start_05.bin
  weapon_data+0x10]; the same `Collide_FilterBullets` 0x1ece30 filter the 07000014 windows pin
  runs, opposite data — the laser 0x0002 / watch 0x0100 would drop). The real trigger-path shot
  kills it and the death writes `switch_channels[0xfd]` [M 0x12bd48:35]. Sister: the barrel
  stands at health 10 with ch 0xfd off.

**The new spawner/drone asserts** (the census claimed "live" via spawners.rs without a pin):
the east-room test now pins that the level's one spawner #718 (group 1, mode 3, max alive 2,
start ch 0xf6 [D params]) holds before the touch with its group-1 templates — the two
0xf6-wake drones by their spots (50.10, 41.46, −0.08)/(56.73, 41.45, 0.06) [D] — ABSENT from
the world [M DroneSpawner_Init's "templates, absent until spawned"], and that the touch brings
the first template in (mode 3 spawns only when none of its own is alive [M 0x137e68's mode
arm], so ONE now; the second follows when the first dies), present and ch-0xf6-armed
([M the +0x134 wake read]).

**Census corrections that fell out** (docs/levels/07000015.md §7): the destroyables row's
satellite "p1 = 0xf6/0xed/0xec" was listed with no meaning and the channel map said
"BoxSatellite #767's group" — the p1 is the tag+0x10 EXTERNAL-KILL channel [M 0x12c318:36], so
the room touch SMASHES the satellite (the map line is corrected); the barrels' p4 = 0xf8 is
the bullet-mask source (now in the row with the admit/drop split); the spawner row said
"group 3" — p0 = 3 is the MODE, p1 = 1 is the group (corrected); the drones row now notes
#699/#700 (p4 = 1) are the spawner's group-1 templates, absent until it spawns them.

**Port gaps found: none in the class code.** Both new destroyable arms, the spawner's
trigger/mode-3 path and the template-absence parse behaved per the decompile on the first
real-input drive. The fixes during bring-up were test-side: my first barrel lookup matched by
position and grabbed the neighbour barrel 0.79 m away (the assert then read ITS channel 0), so
the lookup keys on the level-unique death channel 0xfd with the position kept as a sister
assert; and my first spawner assert expected both templates immediately, contradicting the
mode-3 arm the port implements [M 0x137e68] — the TEST was wrong, fixed, not the code.

## Verification

- The task's binary, fresh process:
  `NF_ISO=$NF_ISO NF_SILENT=1 ./target/release/deps/interactables_07000015_live-05acd79af6304bda --test-threads=1`
  → `test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 40.09s`.
- The level's walk binaries (touched by the same docs/census claims, unmodified code):
  `walk_07000015_live` → `2 passed; 0 failed` (5.85 s); `walkreal_07000015_live` →
  `1 passed; 0 failed` (14.59 s).
- The fast runner (`tools/test/suite.py`), started 21:39 UTC on the branch tree (master + the
  session's files; the runner's build finished `build ok: 967 test binaries in 1227 s` — its
  build predates the final test-file edits, so this run is the baseline). Its problem lines
  through the run: the 4 documented slim-ISO cross-check failures — `identity_disc` +
  the 3 `disc_golden` — the same 4 the level-14 and level-07 reports quote as the slim-ISO
  baseline; no CRASH/TIMEOUT. [The final tree differs from the runner's tree by the one test
  file quoted above and the two docs files — no source file changed.]

## Live checks for the main agent

None required: every new pin rides the decompile chain ([M] with addresses) plus [D] disc
params; the satellite's external-kill arm is the same code path already [L]-verified on
07000008 (props.rs's fusebox → SmallExplosionBox). A screenshot check on the east room after
the #717 touch (the satellite box gone, the whiteout running) would be a welcome visual
confirmation but pins nothing the tests don't.

## What's left

- Nothing on the task. The full-suite total line (the runner prints it only after its last
  binary) may still be pending at session end; the report quotes the runner's problem lines
  and the per-binary totals above instead — the next session can read `suite.out`'s tail.
- The 4 slim-ISO cross-check failures are environmental (the slim disc lacks the compared
  data), not port failures; they fail identically on the untouched master tree with this ISO.

## Usage

Not visible to me: this sandbox shows no token, credit or context-window figures for the
session (no environment readout is exposed to the worker). I cannot copy numbers I cannot
see; the driver's own telemetry is the source for this session's usage.
