# level-04-interactables — report

Branch `s/level-04-interactables` (master 5ae5d60d). Task: census every player-touchable placement
on 07000004, add the missing kinds in `crates/nf-view/tests/interactables_07000004_live.rs`
(`interact_07000004_live.rs` as the base, `interactables_0700000b_live.rs` as the model), fix every
port gap the tests expose.

## Census

docs/levels/07000004.md's 2026-10-17 table over `nf-extract placements-all 07000004.bin` [D 70
placements, main map 49 only — `parsemap_parsemap` 0x1d1cf0's state-2 epilogue creates the main
map's dynamic objects [M]]. The player-touchable kinds and their tests:

| kind | n | test |
|---|---|---|
| Pickups 0xF0 | 4 (3× StormUzi, 1× BodyArmourHeavy) | `interact_07000004_live::the_weapon_pickup_arms_the_storm_uzi` (the walk-over take + the hidden/state-2 respawn pin), `the_armour_pickup_raises_the_armour` |
| Target drone 0x0F | 1 | `the_target_dies_to_the_real_fire_path` |
| TouchOnce 0xEE | 1 | `the_out_trigger_copies_only_after_the_kill` (a pure channel mux, no touch check [M 0x1b1a78:147-160]) |
| ScriptPlayers 0xD9 | 2 | `the_script_players_settle_the_story_handoff` |
| Ripples 0x21 | 4 | `the_pond_ripples_spawn_while_bond_walks_the_pond_edge` |
| LeafGens 0x2B | 2 | `the_leaf_generators_spawn_leaves_over_the_park` |
| Trees 0xF3 | 2 | `the_trees_carry_model_and_anim_hashes` |
| Env 0xDF | 1 | `the_env_generator_sets_rain` |
| **Rigid bodies 0xD8** | 2 | **NEW** `interactables_07000004_live.rs` (this task) |
| Sky 0x2A / Corona 0xFC / Rainboxes 0x32 / Room meta 0x47 / Statics 0x19 | 3/2/3/2/40 | not player-touchable (presentation/meta/cel carriers); the rainboxes feed the env test |

The zero-on-this-level kinds (doors, lifts, switches, locks, copters, monitors, PCQWorm, grapple,
creepwalls, turrets, MoviePlayer, mines, CamSubjects) are pinned by
`no_phantom_switches_or_ai_points_from_the_template_maps` + the doc's dispatcher-table check.

## Done

**The rigid-body kind (the census's one remaining gap — its "UNPORTED" row predates the
rigid-body port).** Two port gaps the new tests exposed, both fixed:

1. **`Rbodies::create` read the wrong param words and keyed the rows by the object index**
   (`crates/nf-game/src/rigid_body.rs`). It read `p.extra[2]`/`p.extra[4]` for the mass and the
   type halfword — the parse leaves `extra` zeroed on these rows, so every body read as type 0 /
   the default mass 3.0 — and it stored `placement: p.index` (the OBJECT index), collapsing two
   placements that share one object (both 04 lamps → object idx 26) into one id. The reads now go
   through `p.params` (the (slot, value) pairs): the mass at rec+0x2c = slot 2, the type halfword
   at rec+0x34 = slot 4 [M `RB_Create` 0x1d6bf0:52-56; D the lamps carry (2,1) and (4,65535),
   07000006's buckets (0,8) and (4,65535)] and `placement` is the placement's own index. After the
   fix the lamps parse as ty −1 (0xFFFF), mass 1.0, placements 39/40 [D].
2. **The player's bullets never reached the bodies** (`crates/nf-game/src/drone.rs`). The game's
   collision pass appends the round's contact to the body's `obj+0xD0` chain (the bullet is class
   5) and `RB_CollisionHandler` 0x1d7a80 — the dispatch pair's second member {0x1d6e38, 0x1d7a80}
   [M 0x26f9c8/0x26f9cc] — walks it into `RB_RestingContactRot` 0x1d73a0's per-hit arm; the port
   only routed blasts. Two halves:
   - `update_player_bullets`' **rigid-body-bullet arm**: the flying round's segment vs each body's
     world sphere (the game's picker is the body's world sphere, `Control_BuildWorldSph` 0x133070
     over its glist — the port's `st.s13c`/`st.trans`) pushes an `RbHit` and stops the round (the
     solid-prop default). The record: the unit flight direction as hit_vel, scale 1.0 — **[S]
     unverified**: the game's per-bullet `HITDATA+0x8` float for an RB contact is not pinned from
     the decompile (the live check below asks gdbwatch for it); no pass/fail assertion rests on
     the scale (the test pins the routing, not the kick's magnitude).
   - the walk's kind-7 slot now runs the pair's second member per tick: `rbodies.
     handle_collisions` consumes the armed hits before `rbodies.tick` evolves [M the pair — the
     blast route's one-tick shape, generalized]. The handler's impact sounds play through
     `play3d` (the game's arm plays Sound_Play3D per hit [M 0x1d7168's class-5 arm]).

## Tests

- `interactables_07000004_live` (NEW): `lamps_are_created_and_rest_bit_identical` — both bodies
  created with the disc's type/mass [D params; M RB_Init/Build/SetState], a resting body
  bit-identical over 600 ticks [M 0x1d6e38's type!=0 no-linear arm + 0x1d79ac's zero-omega gate];
  `shot_lamp_leaves_rest_and_the_sister_stays` — the walk stands under lamp #40, `shoot_point`
  fires the real path, the lamp's omega/matrix/pos leave rest [M the class-5 chain above], and
  the unshot sister #39 keeps zero omega and empty accumulators. **2 passed / 0 failed / 0
  ignored** (release, --test-threads=1, NF_ISO on the slim ISO).
- The neighbours the fix touches: `interact_07000004_live` **10 / 0 / 0** (8.12 s),
  `rigid_body_wiring_live` **2 / 0 / 0** (2.40 s — the 06 blast routing holds under the
  param/identity fix), `interactables_07000006_live` **3 / 0 / 0** (3.72 s).
- The full suite on the final tree: see Suite.

## Suite

`tools/test/suite.py` on the final tree: `total: SUITE_TOTAL`.

## What's left

- The per-bullet HITDATA+0x8 scale for the RB contact ([S] above) — the live check settles it;
  the port's scale 1.0 is a documented stand-in.
- The lamp's impact SOUND id (the game's arm plays a fixed 100.0-priority sound; the port plays
  the handler's collected ids) — presentation, not pinned.

## Live checks for the main agent

1. 07000004, lamp #40 at (8.01, 7.86, 23.03): stand under it (~(7.7, floor, 27.7)), aim up at the
   lamp, fire one round. Expected (game): the lamp swings (its obj+0x90 matrix leaves the
   placement pose); `tools/trace/gdbwatch.py` on `RB_RestingContactRot` 0x1d73a0's per-hit arm —
   record the HITDATA+0x8 float the arm reads for the bullet contact (the port's [S] scale 1.0
   stands in for it) and the hit_vel the arm reads (hit+0x30).
2. Control: the same stance without firing — the lamp's matrix must stay at the placement pose
   (the port's rest-bit-identity pin).
3. The blast route on 07000006's buckets (the wiring test's stance) — the arm's falloff scale is
   already [M]-cited; nothing to check there.

## Usage

not visible to me — this environment shows no token, credit, cost or context-window figures for
the session (no usage display is exposed to the agent; nothing to copy).
