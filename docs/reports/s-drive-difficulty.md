# s/drive-difficulty: driving missions follow the picked difficulty

## What a player notices

- The driving missions now use the difficulty picked on the difficulty page. Before, every
  drive ran the middle band whatever was picked.
- Enemy vehicles are tougher on harder settings, and Bond's car starts with less health, by
  the track's own numbers (MIS4: enemy health x1.0 / x1.2 / x1.4, start health 3000 / 2640 /
  2250).
- On the hardest band of the snow chase (snow2a_mis4), the henchman cars become the snow jeep
  (`junglesnow`) with no riders, as in the game.
- These numbers were not applied at all before, on any band: the view never found the track's
  MSet block, so the hostile health factor read 1.0 and the player start health used the
  built-in defaults. At the default pick (2) the drive is now harder than before: the start
  health drops on all 8 tracks (band-1 percents 75..95), and enemy health rises on all but
  MIS13C (factors 1.1..1.5) [D MSet of each track].

## Raw word or remapped value, per reader

ACTION writes the raw pick to PTP+0x984 (0x100984 in DRIVING) [M Boot_GetPTPData 0x1c8198:13-21]:
`GameState+0x28` (1..4), except base map 0x9000001 with the mini-mission flag `DAT_002e1b10`
set, which writes 1. DRIVING main saves the raw word in 0x100624 and remaps 0x100984 in place
(1 -> 0, 2 -> 1, 3/4 -> 2, else unchanged) before the session, then restores it after
[M DRIVING 0x174bf0:70-81, 304-306]. 0x174bf0 is the only DRIVING function that writes 0x100984
(three functions name it: 0x174bf0, 0x204de8, 0x12b8b0). So every reader inside the drive sees
the remapped 0..2:

| Reader | Form | Citation |
|---|---|---|
| mission+0x3b8 (copied at construction) | remapped | 0x204de8:67-69 |
| hostile health factor (MSet +0x2c4/+0x2c8/+0x2cc) | remapped via +0x3b8 | 0x2084a8 |
| hostile speed factor (MSet +0x2b8/+0x2bc/+0x2c0) | remapped via +0x3b8 | 0x208468 |
| player start health / armour percents (MSet +0x294..+0x2a8) | remapped via +0x3b8 | 0x206ec0:63-64 |
| `SRuleDifficulty` (script facts) | remapped via +0x3b8 | 0x20aac0 |
| AIEl difficulty filter | remapped via +0x3b8 | 0x12cb08:113 |
| AI aim wobble | remapped via +0x3b8 | 0x14ca60:429-437, 570-578 |
| snow2a_mis4 `paris_hench` swap | remapped, reads 0x100984 itself | 0x12b8b0:98 |
| the return to ACTION (PTP+0x144 / +0x984) | raw (restored) | 0x174bf0:304-306, Boot_LoadPTPData 0x1c8560 |

## What changed

- `nf_game::ptp::ptp_difficulty_word`, `driving_difficulty` and `key_for_viv`.
- `nf_game::driving::hostiles::snow_hench_swap`: the 0x12b8b0 swap. Track key exactly
  `snow2a_mis4` [D 0x36aea8], word == 2, vehicle records (+0x44 == 1) whose attribute's
  RENDER_FILENAME starts with `paris_hench` [D 0x36b0c0, strncmp 11] get the name `junglesnow`
  [D 0x36b0d0] and rider count 0.
- `nf_view::driving::load_driving_scene_at(iso, viv, game_difficulty)`; `load_driving_scene`
  keeps the old band (pick 2 -> 1). `DrivingState::difficulty` holds mission+0x3b8 and feeds the
  mission rules, the script facts, the hostiles and the player start health.
- The view's MSet lookup now falls back to the `Map` window copy (`map_script`), where the game
  reads it [M 0x22faf8]; neither top-level finder reached it on any of the 8 tracks [D].
- MIS4.VIV has no `junglesnow` attribute or model; MISC.VIV has both [D], so a swapped mission
  reads them from there.
- The front end passes `selected_difficulty`; the headless driving `--shot` path reads
  `--difficulty N` like the on-foot one.

## Tests

- nf-game unit: `ptp::tests::{driving_difficulty_maps_the_word_like_driving_main,
  ptp_difficulty_word_is_the_pick_except_the_paris_mini_mission, key_for_viv_follows_the_archive_suffix}`,
  `driving::hostiles::swap_tests` (2).
- nf-view `driving_difficulty_live` (3): MIS4 at picks 1..4 checks +0x3b8, every vehicle's
  health against the record x the MSet factor, the player start health and armour against the
  MSet percents, and the swap only at picks 3/4; RACE at pick 4 has no swap; the default loader
  keeps band 1. With the MSet fallback removed it fails (start health at pick 1).

## Unproven

- The game's archive order when it looks up `junglesnow` for MIS4 (MISC.VIV is the only archive
  that has it, so it is taken from there).
- `FUN_001967a8`'s branch for an attribute with no RENDER_FILENAME (the port uses `<name>.crp`);
  no MIS4 vehicle takes it.
- The Paris mini-mission's forced word 1 is modelled in `ptp_difficulty_word` but the port's
  menus have no mini-mission launch, so the front end always passes the pick.
- `PtpBlock::build` (the dev stand-in for the handover) still treats every 0x9000001 handover
  as the mini-mission case; left as is.
- Not checked against a live capture.
