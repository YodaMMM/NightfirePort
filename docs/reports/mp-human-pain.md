# mp-human-pain — players take multiplayer damage by the game's rules

Branch `s/mp-human-pain` (from origin/master 22d5836b7). Task: find how a bullet, a blast and a
melee hit actually reach a human and a bot in a match on master, compare each with
`Player_HandlePain` 0x1902c8 line by line, route every hit through the ported arm, and pin the
Player Mods settings through a real match.

## The live path on master (the census)

Every MP hit on master flows through one site, `Sim::mp_hurt` (crates/nf-view/src/scene.rs), fed
by the shared combat's `MpCombat::step` events:

- **A bullet** (hitscan/auto/shotgun/rocket contact, the guided rockets) — `Hurt::Bullet`: the
  friendly-fire/last-shooter register [M `MP_RegisterBulletHit` 0x1857c0:15-57], the assassin
  rule [M `Player_DealWithObjHit` 0x194458:139-193], then the multipliers + armour. The game's
  route: case 5 → `Player_HandlePain(HITDATA+8, BL, 0, part)` — **pain type 0, located**
  [M 0x194458:195]. The melee (the fists) rides the same case-5 round shape (the record's 1.75
  range), so type 0 + part as well.
- **A blast** — `Hurt::Blast`: no register gate, the falloff, then the multipliers + armour. The
  game's route: `Explode_Propagate` → `Player_Hurt` 0x196060 → `Player_HandlePain(amount, 0, −1)`
  — **pain type 0, location −1**.
- **A bot victim** — the same two events; on master the bot arm computed the multipliers and
  subtracted straight from `health`.

## Line by line vs `Player_HandlePain` 0x1902c8 (the MP arm, `MPSettings._384_4_` != 0)

| Step (the game) | Where | Had it on master? |
|---|---|---|
| The victim's health ≤ 0 / raw damage ≤ 0 early-outs | [M :17-23] | yes (`mp_dead_ticks`/`mp_hp` gate; the raw gate was the scaled one — equivalent, now exact) |
| `master_copy` + GameFlow ≠ 2 gate, `BL+0x8BC = 1.0` | [M :24-27] | the match tick covers the gate; the port's BL carries no such word [S shape] |
| The difficulty multipliers | [M :28-50 — the SP arm ONLY] | **wrongly present** in `net_world_pain` (the SP `damage_factor`) — removed |
| `× Plr_DMod_Multi` when located | [M :51-53] | yes (`mp_damage`) |
| Location Damage part multipliers (+0x1C8, `_456_`) | [M :54-78] | yes for humans/bots (`mp_damage`), **absent** from the world route — added |
| Professional Mode ×3 (+0x1BC, `_444_`) | [M :80-82] | same — added to the world route |
| The pain-type switch: types 1..7 armourFraction 0 | [M :84-99] | **absent everywhere** (the port's MP had no pain type) — added to the arm; the world route (types 5/6/7: hurt volumes `Player_DealWithObjHit` case 0x27, falls `Player_CollisionHandler` 0x1949a8:120, drowning `Player_MonitorAir` 0x191a70:142) now passes the real type |
| The MPGame last-shooter word 0xfffe for types 5/6/7 | [M :91-93] | covered by the death credit (killer None → the self-kill message shape, `Player_CheckForDeath` 0x190810:73-80 reads −2/−1 alike) |
| The armour share `min(d × frac, armour)` | [M :100-104] | yes (frac 1 only) |
| The HEALTH loses the full scaled damage minus the share | [M :105-108] | yes for frac 1; the frac case arrived with the arm (the share gates the armour split only — the health always takes `d − absorbed`) |
| The below-1.0 → 0 floor | [M :131-133] | yes in `mp_hurt`, **absent** from `net_world_pain` — added |
| `Player_SetHealth` write | [M :134] | yes |
| The pain grunt `Rand_Rand(4)` | [M :110-119] | yes, same stream position (the arms draw nothing) |
| The HUD flash `+0x968` / glow `+0x967` | [M :120-129] | yes, local view only; the flash now accumulates the returned scaled amount [M :126] |
| BOT: the armour pool `BOT_vars+0x769` absorbs | [M `BOT_handlePain` 0x125ed8:80-92] | **absent** — the bot arm ignored the pool the armour pickup's case 3 fills [M 0x18e650:119-133]; now routed through `bot_pain`, the pool written back (hashed) |
| BOT: the same multiplier block | [M 0x125ed8:48-79] | yes (`mp_damage` shared) |
| `Drone_ModPlayerHitDamage` between register and pain | [M 0x194458:190-193] | SP-only ([M 0x13a940:5-24] returns 1 with `master_copy` set) — no MP gap |

The ported arms now own the arithmetic: `player_pain_mp(health, armour, damage, rules, location,
pain_type)` (the whole MP arm, returns the scaled amount) and `bot_pain(damage, rules, location,
pain_type, armour)` — both calling the shared `combat::mp_damage` block [M 0x1902c8:51-82 =
0x125ed8:48-79]. `mp_hurt`'s human arm and `net_world_pain` route through `player_pain_mp`; the
bot arm through `bot_pain`. `apply_pain`/`mp_damage` stay in combat.rs (the arm's halves; the
live tests pin through them).

## Determinism

The arms draw nothing from the shared stream; the grunt's `Rand_Rand(4)` [M :111] and the bot
barks keep their exact positions, so every machine's stream is untouched. The bot's armour-pool
write is the brain's `vars.arsenal.armour`, covered by `BotBrain::state_hash` — and every hurt
event is identical on every machine, so the writes stay identical (pinned: the two machines'
hashes agree every tick).

## Pins (crates/nf-view/tests/mp_pain_options_live.rs, two headless machines through `net_tick`)

- `location_option_scales_the_head_and_off_does_not` — the same P2K head round: ×4×4 with
  Location Damage on (the boot default [M bootup 0x1c7e78 @0x1c7ff8]), ×4 with it off.
- `professional_mode_triples` — the torso round ×4×3 with `capture_reward` set.
- `armour_spends_first` — a 14-scaled hit on 10 armour: the pool drains, the health pays 4.
- `armoured_bot_spends_the_pool` — a located type-0 hit on a bot holding a 40 pool through the
  shared hurt path on both machines: the pool pays, the health untouched.
- The existing `mp_combat_weapons_live` rows (armour-first on a full pool, the head round 56−6,
  the kill credit, the melee ×Multi) stay green unchanged.

## What ran

Every binary whole, `--test-threads=1`, on the final tree (the sandbox disk filled twice during
the full-suite build — the linker died on a 100%-full volume twice; after pruning the orphaned
executables the per-binary runs below cover the task's list; the remaining long binaries did not
fit the window and are named at the end):

- `nf-game` lib: **686 passed / 0 failed / 5 ignored** (includes the extended `bot_pain` /
  `player_pain_mp` unit rows).
- **mp_pain_options_live (new): 4/0** — the option on/off head shot, Professional ×3, the
  human armour-first remainder, the armoured bot's pool (two machines, hashes agree).
- The task's named binaries, all green:
  mp_world_sync_live **8/0**, mp_combat_weapons_live **11/0**, mp_core_live **5/0**,
  mp_bots_live **3/0**, mp_bots_moving_live **3/0**, mp_bot_weapons_live **2/0**,
  mp_bot_armour_online_live **1/0**, mp_bot_objectives_live **10/0**, mp_bot_voices_live
  **3/0**, mp_bot_body_sounds_live **1/0**, mp_death_effects_live **3/0**,
  mp_site_destruction_blast_live **4/0**, mp_site_damage_offline_live **4/0**,
  mp_16_slots_live **3/0**, mp_bodies_live **3/0**, mp_body_anim_live **2/0**,
  mp_clock_gate_live **1/0**, mp_hud_16_live **2/0**, mp_hud_real_live **4/0**,
  mp_match_end_live **2/0**, mp_net_slots_post_move_live **2/0**, mp_host_match_live **2/0**,
  mp_pickup_take_live **2/0**, mp_pickups_visible_live **2/0**, mp_remote_fx_live **2/0**,
  mp_respawn_ammo_live **3/0**, mp_scenarios_live **19/0**, mp_shot_effects_live **2/0**,
  mp_shot_gate_live **5/0**, mp_status_strings_live **1/0**, net_desync_report_live **4/0**,
  perf_state_hash_live **0/0/1 ignored** (the binary's own env gate).
- The full suite (`tools/test/suite.py`): its build phase compiled the whole tree but the
  linker hit the full disk twice (documented above) — BUILD FAILED on the volume, not on code.
  After the prune, the named binaries ran whole as listed. **Did not fit the window:**
  mp_debriefing_flow, mp_defaults_setup, mp_lobby_one_screen, mp_menu_flow, mp_menu_lists_live,
  mp_net_lobby, mp_online_failures, mp_perf_16_live, mp_screens, mp_stress_explosives_live,
  mp_death_hands_online, net_match_live, net_e2e_loopback_live, net_join_live,
  net_internet_live, net_match_end_clients_live, net_resync_live, net_soak_faults_live — none
  of them touch the pain arithmetic beyond what the green set covers (the menus/lobbies are
  input plumbing; the remaining net binaries exercise the same `net_tick` + hash loop the green
  net_desync_report_live/mp_world_sync_live rows pin), but the next message should run them
  plus one full suite pass on this tree.

## Live checks for the main agent

Nothing visual changed (no draw path touched), so no render spots are needed. One optional probe
if a PCSX2 session is handy: with Location Damage ON (the boot default), a head-shot damage read
at `BL+0x894` should drop by `weapon_damage × 4 × 4 − armour` for a P2K round on arena
07000023 — the ×4×4 shape the pin asserts.

## What's left

- The `MpSettings` doc comments in mp/mod.rs carry two Ghidra-decimal/hex slips in the field
  docs (`master_copy` says "+0x388" where the decimal convention makes it +0x184;
  `team_game` says "+0x396" = +0x18C) — the field MAPPINGS are right (checked against
  `MP_Start` 0x183a50:69-70 and `MP_RegisterBulletHit` 0x1857c0:33); left untouched to keep
  this branch's diff on the pain path only.
- The SP path's hurt volumes/falls/drownings still go through the SP pain shape
  (`PlayerHealth::pain_generic` etc.) — the SP arm is a different decompile block, not this task.

## Usage

not visible to me (no token/context/cost figures are shown in this environment).
