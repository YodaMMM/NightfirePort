# s/g053-mp-scenarios: MP scenario rules against the decompile (GAPS G-053)

**Random stream:** this branch can move the shared stream, so it stays on the branch. Two
cases: a bot touching a hurt volume now barks (`Rand(n)` [M 0x1266b4]) and dies, and a GoldenEye
item dropped, re-taken and dropped again now reaches its 30 s re-pick (`Rand_Rand` [M
0x187568:191-200]) sooner. A run where neither happens draws exactly as before.

## What was checked

Every scenario update and its helpers, read line by line against the port
(`crates/nf-game/src/mp/rules.rs`, `mod.rs`, the scene drivers): `MP_FlagUpdate` 0x1861a8,
`MP_UplinkUpdate` 0x186d98, `MP_KOHUpdate` 0x186ae0, `MP_DemolitionProtectionUpdate` 0x1870a0,
`MP_GoldenEyeUpdate` 0x187568, `MP_BluePrintUpdate` 0x1882d8, `MP_BluePrintReachedBase` 0x188970,
`MP_PlayerKilled` 0x188f70 (scoring, Top Agent lives, Assassination, Team Arena team effects,
the carried-object drop), `MP_EquipPlayer` 0x1855f8, and the bots' hurt arm in
`NDrone2_DealWithObjHit` 0x13fff0. Uplink (flip, claim order, drip, truncated team sums), KOTH
(drip, on-hill bit, sounds), Demolition/Protection (HP store, own-goal, timeout), the GE strike
launch/landing, the assassin reset and the kill scoring matched the port already.

## Differences found and fixed

1. **Return counter survives a re-take.** The game zeroes `MPOBJECT+0x04` only on the
   dropped-state returns (CTF teammate touch, every 30 s expiry) [M 0x1861a8:195-197,226-231,
   0x187568:191-195, 0x1882d8:150-153]; takes, captures, deliveries, the strike completion and
   the GE death re-pick write nothing. The port zeroed it on every take. Player effect: a flag,
   GE item or blueprint dropped, picked up and dropped again comes back sooner (the old count
   resumes). Test: `mp_match_rules.rs` `g053_return_counter_survives_a_retake`.
2. **Blueprint expiry goes back to its spawn.** The 30 s expiry copies `MPOBJECT+0x10` (the
   spawn matrix) into the object [M 0x1882d8:150-160]; the port left it lying where it fell.
   Test: `g053_blueprint_expiry_goes_back_to_its_spawn`.
3. **The quick return after a kill-floor death.** `MP_PlayerKilled` preloads the carried
   objects' counters when the dying body's hit list holds a hurt volume of type 5..7 (`4 < type
   < 8`) [M 0x188f70:190-195,281-283]. Census [D]: 0700004b (8 volumes), 07000025 (6), 07000026
   (4), all type 6, damage 1000; the other arenas have none. The port passed no preload for
   Bond's offline death at all, and online used pain types 4..7 (taking type 4, falls and
   drowning, none of which the game counts). Now Bond's offline hurt deliveries stamp the flag
   (`Sim::mp_bond_hurt_explosive`) and the online world pass passes only the slot's type 5..7
   hurt contact. Player effect: a flag carried into a kill floor is back at its base two frames
   later instead of lying in the pit for 30 s. Test: `mp_g053_scenarios_live.rs`
   `kill_floor_death_returns_the_flag_at_once` (sister `an_ordinary_death_leaves_the_flag_down`).
4. **Bots die in kill floors.** The game's bot hurt arm takes the volume's damage off the bot's
   health and kills it [M 0x13fff0:320-335]; the port's MP bots took no hurt-volume damage. Now
   `Sim::step_mp_bots` runs the contact (capsule root - 1.0 to eye, r 0.4, the SP drones'
   contact law [S]), barks, and scores the death with no credit and the preload. Test:
   `a_bot_dies_in_the_kill_floor_and_the_flag_comes_home`.

Each new test failed on the old code (run with the old file checked out) and passes now.

## Docs corrected

`docs/systems/mp-match-rules.md`: the preload source, the counter rule, the blueprint expiry,
the bots' hurt contact, and the respawn loadout line (the grapple word is `+0x1D0`; the old
text said `+0x1BC` gave a second weapon, which `MP_EquipPlayer` does not do).

## Open, not ported

- With the `+0x1BC` Player Mod set, `MP_BluePrintReachedBase` gives the delivering carrier the
  next weapon of the weapon-set row it lacks and drops its previous one (human:
  `Player_EquipWeapon(.., 999)` + owned/clip clear + switch request; bot: `BOTWEAP_EquipWeapon` +
  `BOTSTATE_pickupWeaponChangeChoice`) [M 0x188970:56-160]. The port has no standalone
  `Player_EquipWeapon` 0x1a8a58, so this needs that port first.
- The bot hurt capsule and the exact frame the game's hit list is read at the bot's 0xF2 entry
  are the port's choice [S]; only the real game can confirm a bot standing at a kill floor's
  edge dies on the same tick.
- The body sphere 1.0576 and the bot helpers mp-bot-brain-rest.md lists as unwired stay as
  they were.

## Tests run

- `nf-game --test mp_match_rules`: 20 passed (2 new).
- `nf-view --test mp_g053_scenarios_live`: 3 passed (new).
- `nf-view`: mp_scenarios_live 19, mp_bot_objectives_live 10, mp_world_sync_live 8,
  mp_site_damage_offline_live 4, mp_bots_live 3, mp_16_slots_live 3, local_match_bot_pickups_live
  3, mp_respawn_ammo_live 3, mp_net_slots_post_move_live 2: all passed.
- `nf-game`: lib `mp` filter 144 passed; mp_lockstep 10 passed; mp_debriefing (no NF_ISO-free
  cases run) ok.
- The MP arenas have no walkreal binary.
