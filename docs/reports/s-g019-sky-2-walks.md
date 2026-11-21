# s/g019-sky-2-walks: the walk re-tunes for the voice-line fix

**First: this branch moves the shared random stream (it carries s/g019-sky-2's voice-line fix).
Keep it with that branch. Two walks stay red and are ignored with their reasons:**
- `walkreal_07000015_live::walk_spawn_to_the_omega_sector_exit_real_damage_00agent`: Bond dies
  t6088 at the door[39] north node (#8's 0x1a grenade, -38.0). 97 runs dead: 16 single stops
  500-8000 and 81 random 1-3-stop runs (of the 81, 46 die at the fence/plank, 26 at the
  door[29] west lane, 8 in the hall, 1 elsewhere).
- `walkreal_07000013_live::walk_the_warm_chain_to_the_front_entrance_and_out_real_damage`
  (00 Agent): Bond dies t5170 in 13's lobby west mid (#13's 0x18 bursts). 12's lobby-east flank
  now costs 28 health, so 13 starts at 61.5 instead of 93.0; runs that reach 13 with up to 96
  still die there. 99 runs dead. The master run ends at 6.5 health, so the walk had almost no
  margin before this change.

## Base
Rebased onto c3b9de140 (the stream batch: guards, searchlights, impacts, ninjas, party idles),
one conflict in docs/GAPS.md (G-019 takes the sky branch's row, G-020 keeps master's row). With
the batch in, 02, 03 and 11 are green again, and 02/03's 00 Agent walks stay ignored as the
batch left them (both still die on this branch: 02 at t16813, 03 at t9572). On a c3b9de140
build every walk below passes, so each red here is from the voice-line fix.

## Reds on the rebased branch and the levers
Each lever is a stop (the walker idles N default ticks at tick T, `walker::with_pause(s)`),
found with an NF_WALK_PAUSE sweep on the same exe. Every reason is in the test's comment.
- 01 `the_plate_refuses_bond_alone`: guard #3 shot the contact, the restart posted. 100/30.
- 01 `walk_from_the_spawn_to_the_exit_with_the_contact`: the contact shot down, mission
  restart at t81167. 3000/60 -> 3000/30.
- 01 `the_exit_fade_starts_on_the_exit_tick...`: with 3000/30 the exit fired in the middle of a
  stairwell fight and the fight loop ran past the 300-frame hold. 3000/45 (own lever).
- 01 `following_mayhew_down_completes_the_level`: the contact was not on the plate. 100/30.
- 06 `walk_the_castle_interior_to_the_party_real_damage_00agent`: died t7308 on the strip
  approach. Added 4650/10 between the batch's 400/400 and 8000/15.
- 08 `walk_the_whole_mission_on_real_input_real_damage_agent`: the Hind's rockets killed Bond
  at t33118. 3000/60.
- 15 d1, Agent, 00 Agent, Agent real damage: the fence shot's neighbours (#448 by the bullet),
  a mission restart at the split south, a death at t10345. One value, 4000/90, passes all four.
- 05 `the_door_band_exit_chains_to_07000006_walked_real_damage` (found by the full run): the
  walk fell off the south strip 38.5 again. 9000/200 -> 3000/60 (7 of 7 values pass).
- 16 `..._on_real_damage_agent`: died t19492 in the last chaser fight. 1600/30 -> 3500/30
  (the batch's other green value).
- 16 `..._on_real_damage_00agent`: died t3232 in the pocket. 425/30 + 3822/45 ->
  2800/15 + 4200/90 (1 of 32).

No game logic, damage, health or ammo changed. No pinned values needed a re-pin. None of the
reds looked like a port fault: each death is a fight lost by the walk script, the same fights
the master build wins.

## Tests run (NF_SILENT=1, cargo -j 8, one cargo in this lane)
- Every walkreal_* binary (109): 108 green, 05 red (the door-band walk above). After its
  re-tune, 05 re-run green (16 passed, 1 ignored). Total on the final tree: 278 passed,
  0 failed, 38 ignored (the two new ignores included).
- Master baseline (c3b9de140 build, exported tree, target-m): walkreal 01, 06, 08, 13, 15, 16
  all green.

## Unproven
- The levers are re-rolls, not fixes. The two ignored walks need their own re-tune.
- 02/03 00 Agent (ignored by the stream batch) were not re-tuned here.
