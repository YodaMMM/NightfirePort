# s/g026-guard-ai

**RNG-shifting: this branch moves the shared random stream (one extra Rand_Rand(0x1e) per stand reload). It stays on the branch for a stream batch and needs a re-pin; 8 walkreal tests go red on it and pass on master.**

## What changed

- `NDrone2_DSTATE_AimStandReload` 0x1666d8 ENTER [M :17-27] is now ported in full:
  `Drone_AlertStatusSet(1)`, then `Rand_Rand(0x1e)` on every entry, and a 0 with
  `CanDoAnimState(0x4a, 0)` plays the alternate reload anim 0x4a (follow-on 0x68), else 0x52.
  The draw comes before the anim test, so the stream moves for every anim set. The crouch
  reload 0x167370 has neither (unchanged).
- What a player would notice: about one stand reload in 30, a guard with anim set 9 plays the
  other reload animation. Set 9 is used by 3 to 15 guards each on 01, 02, 03, 11, 12, 13 and 4A
  (`g026_reload_census`). No other level has a guard that can play it.
- No draw or visual code was touched; the anim is the game's own script.

## DSTATE census

`NDrone2_StateFuncs` 0x29b300 (250 entries) read from the ELF and matched against the port's
state ids (`drone.rs`, `kiko.rs`, `mp/bots.rs`) and docs/port-map/:

- Every story-level id has a port arm, or a recorded no-producer proof (docs/port-map/unreached.md:
  0x3B, 0x57, 0x5D, 0x61, 0x6A, 0x6B, 0x6C, 0x80, 0x9D, 0xA1, 0xB5, 0xB7-0xBC). The bot family
  0xC3-0xF9 is in `mp/bots.rs`.
- 0x02 `DSTATE_Disabled` 0x172440 has no port arm, and no literal `Drone_SM_SetState(..., 2, ...)`
  site was found in the decompile. Variable-id and table routes into it were not checked: its
  reach is unproven.
- A stand-in still sits in a story handler: CombatNoMove 0x1643e8's type-0xC Rand(3) to
  CrouchCover 0x6E roll [M :103-111]. The only type-0xC placement on any level is 07000008's
  escort VIP (skin 0x0500003B). The port's 07000008 start list (difficulty 1) has no type-0xC
  drone (`type_c_drone_probe`), so whether she is ever created and enters 0x59 is unproven. Not ported.
  The old code comment called this a voice check; it is the drone type `+0xD8`, corrected.

## Tests

- New `crates/nf-game/tests/g026_guard_ai_live.rs`: `stand_reload_rolls_the_alternate_anim_one_in_thirty`
  (07000001 set-9 guard: 0x4a at least once over stream offsets, 0x52 otherwise, status 1; crouch
  reload never 0x4a, status untouched) and `stand_reload_draws_once_even_without_the_alternate_anim`
  (07000002: stand reload draws exactly one more than crouch reload). Both fail on master
  (status 0; equal draw counts) and pass here. The tests wake a WaitSwitch guard by setting
  `enabled` directly; the reload ENTER is the unit under test.
- `crates/nf-view/tests/g026_reload_census.rs` (ignored census dump).
- nf-game `ai_live`, `fix_ai_remainder_live`, `ai_missing_states_live`: green.
- walkreal, all 22 per-level binaries (01-0D, 11-16, 1B, 41, 4A). Red here, green on master
  (each re-run by exact name on a master build):
  - 01 `the_exit_fade_starts_on_the_exit_tick_...`, `walk_from_the_spawn_to_the_exit_with_the_contact`
    (Bond never reaches the LoadPointActivator plate)
  - 02 `walk_the_castle_all_objectives_real_health_00agent` (doorway sensor ch 0x81 not raised)
  - 0D `walk_arrival_to_the_c5_door_real_damage`, `..._agent` (Bond dies to drone #0)
  - 11 `walk_spawn_to_the_lower_floor_load_point` (stuck at the Load Point)
  - 13 `walk_the_warm_chain_to_the_front_entrance_and_out_real_damage_agent` (Bond dies)
  - 15 `walk_spawn_to_the_omega_sector_exit` (#448 blast chain)
  Every other binary is green.

## Unproven

- The alternate reload's timing and look were not compared against the real game.
- Disabled 0x02 reach, and the 07000008 type-0xC roll, as above.
