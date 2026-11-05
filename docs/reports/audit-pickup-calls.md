# audit-pickup-calls — the Pickup_Handler and inventory/ammo-give sweep

Branch `s/audit-pickup-calls` (cut from origin/master 17a784980). Scope: Pickup_Handler
0x18e650 and every caller of the inventory and ammo give paths, on-foot and multiplayer —
conditions, amounts, caps, sounds, lines, respawn timing. Fixes at clear port sites, pinned on
real level data; the rest as a table. The decomp census was swept first-hand this session
(Pickup_Handler 0x18e650 body, Pickup_Update 0x18e148 touch/respawn, Player_EquipWeapon
0x1a8a58 / Player_EquipAmmo 0x1a8d18 / Player_EquipKey 0x1a97a0, Pickup_Create 0x18dda0 /
CreateSimple 0x18f348 / CreateFromSet 0x18f1b0, the MP bot arm, the ammo_data cap table).

## What's done (the four fixes, all pinned)

1. **The take message's duration is 180 SP / 90 MP** — `Text_AddMsg(..., 180.0|90.0)`
   [M Pickup_Handler 0x18e650:324-327]. The port always sent 180. Fixed in the `taken` drain
   (`Sim::hud_tick`, scene.rs): `Msg::new(text, if is_mp { 90.0 } else { 180.0 })`.
2. **The MP weapon-take message binds `weapon_data[raw]+0x3C` BARE** (no 0x200000D "%s"
   wrapper) [M 0x18e650:249-256]; the MP ammo message formats through the C string at
   0x30cc28 = `"%d x %s"` [D elf dump, read this session] over (rounds taken,
   `ammo_data[type]+8`) [M :279-288]. The port used the SP label path for both. Fixed: the MP
   arm in the same drain; new `WeaponData::mp_name_label` (+0x3C, own lines, weapon.rs).
3. **The take sound plays for EVERY taker** — the shared tail `Sound_Play3D(100.0, -1.0, -1.0,
   info+0x2A, obj+0xC0, 0, 0, 0)` [M 0x18e650:334-337] runs for the bot arm too (it jumps
   straight there, :137 — the placement's OWN +0x2A word; the 0xF5/0x141/0x14C overrides live
   in the message block the bot path skips [M :290/:300/:319]), and for remote humans with the
   human overrides. The port's `step_mp_pickups` bot/remote branches consumed silently. Fixed:
   the Bot branch plays `p.sound`; the Remote branch plays the human override map
   (Weapon|Ammo → 0xF5, Armour → 0x141, Bonus → 0x14C, else the placement's word).
4. **The respawn restore is one tick late, not on the limit** — the game hides while
   `elapsed <= FRAME_RATE_INT · respawn · 10` [M Pickup_Update 0x18e148:183-185], so the
   restore lands on elapsed == limit+1. The port restored on elapsed == limit (one tick
   early). Fixed in `take_pickup` + `consume_pickup` (`respawn_ticks = 60·r·10 + 1`) and the
   unit test `respawn_hides_for_frame_rate_times_respawn_times_ten` re-pinned to the game's
   `elapsed <= limit` shape.

## How it was verified

- `cargo test --offline --release -j2 -p nf-formats --lib`: **92 passed / 0 failed**.
- `cargo test --offline --release -j2 -p nf-game --lib -- --test-threads=1`: **662 passed /
  0 failed / 5 ignored** (includes the re-pinned respawn test and the full pickup.rs unit set:
  kinds, amounts, caps, refusal, sets, label rides, gold-medal gate, take-ray origin).
- New pin `crates/nf-view/tests/mp_pickup_take_live.rs` on the real arena 07000023 through
  `nf_view::load_mp_carry` (**2 passed / 0 failed**):
  - `mp_take_message_is_ninety_ticks_with_the_mp_label_and_the_sound_plays`: the local take
    through the real touch walk — 0xF5 heard at the pickup in the hearing list; the message is
    the +0x3C label text ("Auto 12") at the game's 90-tick duration (the pane decays per tick;
    the assert bands it under 90).
  - `bot_take_plays_the_placements_own_sound_word`: a bot take (the bot's own teleport
    `fell_out_map`, a weapon its arsenal does not hold) through `Sim::step_mp_pickups` — the
    placement's 0xF5 is heard at the pickup (before the fix the bot take was silent).
- **The full suite did not fit** (the session started 02:26 UTC with a hard 04:25 deadline;
  setup + the sweep consumed the budget). Remaining per lesson 10 — code touched: scene.rs
  (`hud_tick`, `step_mp_pickups`), drone.rs (take/consume tails), weapon.rs (+1 accessor),
  pickup.rs (comment/test): every `walkreal_*_live`, plus `fix_pickups_ammo_live`,
  `fix_pickups_trace_live`, `fix_pickups_remainder_live`, `pickups08_live`,
  `mp_pickups_visible_live`, `mp_world_sync_live`, `fix_hud_live`, `interactables_*_live`,
  `pickup_live`, `gold_medal_gate.rs`, `fix_objects_live`, and the nf-view lib suite whole.
  The changed sites' own suites (nf-formats, nf-game libs) are green as listed above; the
  nf-view lib ran green earlier on the fx branch base but not after this task's scene.rs edits.

## The sweep table (verified present in the port — no action)

| Item | Game [M] | Port | Verdict |
|---|---|---|---|
| Kind set 0-6 (+7-11 MP sets) | 0x18e650 switch; 7-11 intercepted at 0x18dda0:104-105 | pickup.rs kinds + create_placed's set intercept | ok |
| Amounts = PICKUPINFO+0x26 rounds | 0x18e650 (kinds 0/1) | the placement's `rounds` | ok |
| Weapon not-owned -> EquipWeapon; owned -> EquipAmmo | 0x18e650:172-207 | Inventory::take / equip paths | ok |
| EquipWeapon refusal (model not loaded, at cap, 0 rounds) | 0x1a8a58:20,63-67 | equip_weapon | ok |
| EquipAmmo empty-clip fill, taken-count return | 0x1a8d18:35-82 | equip_ammo | ok |
| Caps = ammo_data+2 (34 records @0x2c69b8) | [D docs/fields/ammo-data.md] | Inventory::cap via weapon.rs ammo_cap | ok |
| Key never refuses, sets BL+0x70C | 0x1a97a0; 0x18e650:217-218 | take kind 2 | ok |
| Armour: refuse >= 50, hard SET 50.0, flash 1.0 | 0x18e650:220-225 | take kind 3 (+ pain_alpha) | ok |
| Kind 6 Bond bonus / kind 5 message / kind 4 label | 0x18e650:229-236,304-305 | Taken::Bonus/Message/Label | ok |
| Take sounds: 0xF5 kinds 0/1, 0x141 armour, 0x14C bonus | 0x18e650:290/300/319 | take_pickup's take_sound map | ok |
| Switch channel + timestamp on take | 0x18e650:330-333 | channels.set | ok |
| Drop creation (clip fractions, refuse list, 0x6A/0x6E -> 0x33) | 0x18f348; 0x17a2a0:96-104 | create_simple + drone drops | ok |
| Set kinds 7-11 -> base id, clip*2, respawn 3, sound 0xF5 | 0x18f1b0:56-74 | create_from_set_row | ok |
| 07000008 vault 0x2C -> 4 rounds | 0x18dda0:116-118 | the level override + test | ok |
| Gold-medal gate on kind 6 | 0x18dda0:93-95 | create_placed + gold_medal_gate.rs | ok |
| Lifetime drops 60*120 SP / 60*30 MP | 0x18f348 stack args | DROP_LIFETIME | ok |
| Fall + rest + spin | 0x18e148:110-201 | Pickup::fall / spin_tick | ok (dynamic-hit branch below) |
| Touch: radius+1.1, LOS 0x721 from sphere centre, first taker wins | 0x18e148:202-220 | Pickup::touches | ok |
| Hidden (respawning) pickups stay takeable | 0x18e148 has no hidden gate | update_pickups (no hidden gate) | ok |
| Respawn tail: delete / 0xFFFF forever / hide+state 2 | 0x18e650:338-348 | take/consume tails | ok (+ fix 4) |
| Restore after 60*respawn*10 | 0x18e148:182-189 | respawn_wait | ok (+ fix 4) |
| Bot arm: EquipWeapon/Ammo, armour +20 hp + rounds cap 50, visit stamp 45 s | 0x18e650:34-137; 0x129958 | botweap take_pickup + bots.rs stamps | ok |
| MP id remap (Upgrade_MPWeapon, 10<->0xD) | 0x18e650:146-163 | mp/equip.rs | ok |
| MP messages: 90 ticks, +0x3C bare, "%d x %s" | 0x18e650:249-327 | scene.rs drain | ok (fixes 1-2) |
| Take sound for every taker | 0x18e650:334-337 | step_mp_pickups | ok (fix 3) |

## What's left / findings (no fix this session)

1. **`Sim::step_mp_pickups` has NO caller in the current tree** — the MP shared pass (the
   humans-then-bots loop, the bot/remote arms) is dormant: `update_pickups` handles only the
   local player's touch (`mp_shared_pickups` defaults false), so MP bots cannot take pickups
   in the port's current wiring. The pin drives the real pass directly. Wiring it into the MP
   tick driver (and the `mp_shared_pickups = true` mark's counterpart at lib.rs:8528's path)
   is the missing piece — a behavioural gap, not a values gap.
2. **`mp::unregister_pickup` / `reset_pickup_bot_visit_times` are dead code** — the game
   unregisters on delete [M 0x18e650:341] and lifetime expiry [M 0x18e148:257], and resets a
   bot's visit rows on respawn (0x171518). Clear port site once 1 is wired.
3. **Bot/remote takes skip `Taken` HUD messages** — matches the game (the bot arm skips the
   message block [M :137]; a remote human's message belongs to their pane), but worth a
   deliberate note when 1 is wired: `Text_AddMsg(cVar1, ...)` targets the TAKER's index.
4. **MP one-gun eviction** (`MPSettings+0x1BC && +0x444`) unmodelled in botweap.rs (documented
   skip) — only live under those MP settings.
5. **Pickup fall's dynamic-hit branch** (`Control_InheritVelocity(3.0)` when the landing ray
   hits a non-static object [M 0x18e148:118-149]) — the port tests the static world only
   (documented deviation).
6. **The remote-human take force-consumes** non-armour kinds when online (drone.rs documents
   the MP deviation); the game refuses at cap (pickup stays).

## Live checks for the main agent

1. MP take message: pick up any weapon in an arena match on PCSX2 — the HUD line should show
   the MP weapon name label (weapon_data+0x3C) and linger about 1.5 s (90 ticks), not 3 s.
2. Bot pickup sound: stand near a weapon pickup a bot grabs — the take sound (the placement's
   +0x2A word, 0xF5 for set pickups) should play from the pickup's position.

## Usage

not visible to me
