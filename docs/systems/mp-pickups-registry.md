# The MP registered-pickup registry and the enviro toggles

mp-pickups-respawn-2 (the re-land of the never-merged `s/mp-pickups-respawn`, task 648, onto
master). The weapon-set table, the random re-roll and the spawn loadout are documented in
`nf-game::mp::equip`'s module docs (mp-lobby-settings) — this page covers what that task's
branch carried that master still lacked: the `MPpickups` registry, the visit stamps and the
enviro-toggle creators. Code: `crates/nf-game/src/mp/pickups.rs`, wired through `MpMatch`
(`crates/nf-game/src/mp/mod.rs`). Evidence tags: [M] decompile (line numbers per
`work/decomp/ACTION.ELF/<addr>_<name>.c`), [D] ACTION.ELF bytes, [L] EE snapshot, [S] port
choice. Ghidra names offsets in DECIMAL (`MPSettings._472_2_` = hex +0x1D8).

## The registry (the game's data)

- 64 records of 0xa0 bytes @0x2a4b50 (`MPpickups`), `memset 0x2800` at `MP_Init` 0x183438:37
  [M]. Fields the game reads: record+0 the registered `obj_tag*`, +0x10 the position (obj+0x30),
  +0x20 the cel, +0x80 four f32 visit stamps [M 0x189d80:79-89, 0x18a168:18-26].
- Cursors: `PickupNextAddIndex` @0x30d77c (init 0) and `PickupLastDeletedIndex` @0x30d778
  (init −1) [M 0x183438:40,43]. The registered count is the u16 at `MPSettings+0x1D8` [M
  0x189d80:107] — the port's `MpSettings::registered_pickups`.

## MP_RegisterPickup 0x189d80

Gates: MP on (`MPSettings+0x180` != 0), the obj present, its kind byte 0x2f [M :5-9]. Slot
choice, in order: `PickupLastDeletedIndex` while it still points at a free row (then cleared)
[M :29-31]; the first free row from `PickupNextAddIndex` up to 0x40 [M :33-39]; the wrap scan
1..PickupNextAddIndex (only when the cursor > 0) [M :42-51]; else evict — the registered pickup
with the smallest non-zero `PICKUPINFO+0x2E` lifetime among the first `count` NON-NULL rows [M
:52-73] — through `MP_UnregisterPickup` [M :76]. Nothing evictable → −1. The store writes
record+0/+0x10/+0x20, stores the slot into `PICKUPINFO+0x30` [M :98], runs
`AINetwork_InitEmitter` under the debug byte and `MP_Pickup_PostLoadInit` when the map is
loaded [M :89-97], then `PickupNextAddIndex = slot + 1` with the wrap to 0 when slot > 0x3e
[M :101-105] and `count += 1` [M :107]. Count semantics: a normal register is +1; an evicting
register is −1 (the unregister) then +1 — net unchanged, and `PickupLastDeletedIndex` ends at
the reused slot.

Reachability note: the eviction scan's NON-NULL counting is the decompile's literal form but
unobservable — every null row is findable by the slot search (the forward scan covers
next_add..0x40, the wrap scan 1..next_add), so an eviction only fires on a fully-occupied ring
where both readings coincide [S analysis].

## MP_UnregisterPickup 0x18a058

Gates: MP on, obj present, kind 0x2f, the obj's `PICKUPINFO+0x30` != −1 and the row at that
slot still pointing at THIS obj [M :5-14]. Frees the row's AI emitter under the debug byte
[M :15-17], zeroes the 0xa0 row, records `PickupLastDeletedIndex = slot` [M :24], zeroes the
obj's 0x40 ext with ext+0x30 = 0xffff [M :25-28] and `count -= 1` [M :29]. The port's
`MpPickups::unregister` carries the row + cursor halves; the emitter and the obj-ext zeroing
are not modelled yet (the port's pickup records carry neither) [S].

## MP_Pickup_Process 0x18a168 and the visit stamps

`MP_Update`'s phase-0 arm calls it every tick [M 0x184bd0:38]. It walks the rows from 0 and,
per NON-NULL row, zeroes the four +0x80 stamps that are set and behind the `MPGame+0x19C`
clock — until `count` NON-NULL rows have been visited [M 0x18a168:9-27]. The walk counts rows,
not slots: null rows do not consume the count (pinned by a sister test).

`BOTSTATE_setPickupVisitTime` 0x129958 writes `MPGame+0x19C + 45.0` (+0.1 when that would be
0.0) into the record's stamp for the bot's roster id [M :9-15]. The record carries 4 stamps
(process manages exactly 4 [M 0x18a168:18-26]; the game's bot count clamps at 4 [M
0x183a50:133-135]). The stamps are per-bot "don't retarget until" markers, NOT respawn timers:
`BOTSTATE_pickGoal` skips/penalises recently visited pickups [M], and the actual pickup respawn
wait lives in the pickup obj (`PICKUPINFO+0x2C`, already ported in `pickup.rs`). The port's
own >4-bot extension keeps its stamps in the mp-bot-weapons `BotPickup` view [S mp-16-slots].

## The enviro toggles

- `GunImp_Create` 0x131f20:50-62 [M]: SP (`MPSettings+0x180` == 0) → class 0xb; MP with
  `MPSettings+0x1B8` == 0 → no emplacement; the Random set (the u16 at +0x1B4 == 10) →
  `Rand(10)`; else the class is the weapon-set id. Boot 0 [M 0x1c7e78:38].
- `Car_Create` 0x1305b0 [M]: MP with `MPSettings+0x1CC` == 0 → none; Ravine (GameState+0x12 ==
  0x700004b) → none; `7 < NumTanks` → none (the 8-vehicle cap, caller's); obj+0x112: 2 → 1
  (helicopter), < 3 → 1 unless the value is 1 (tank), ≥ 3 and != 3 → 1, 3 → `Rand(100) < 0x33`
  (51 %) → 1 else 0. A helicopter also gets +1.5 y and the entity gfx 0x2000194 [M :52-58].
  Boot 0 [M 0x1c7e78:42]. The port's `car_kind` returns the obj+0x112 byte; SP reaches the
  same tree with the boot 0 → helicopter.
- The grapple toggle `MPSettings+0x1D0` is consumed by `MP_EquipPlayer` 0x1855f8:27-34 (the
  spawn loadout, `equip::mp_equip_player`) — boot 0 [M 0x1c7e78:45; the re-land's report fixes
  the old branch's "boot 1" claim].

## What the re-land did NOT port (master already has it or replaced it)

- The corrected 11×5 `PickupMatrix`, `UseableGuns`, `random_weapon_row`/`weapon_row`,
  `upgrade_mp_weapon` (boot table) and `mp_equip_player` — `mp/equip.rs` (mp-lobby-settings).
- The old branch's `reroll_random_weapon_set` (mutating a live matrix copy per `MP_Init`) —
  master derives the Random row per read from the lockstep seed (`equip::weapon_row`) [S].
- The per-slot `Upgrades` index dimension of the old `upgrade_mp_weapon` — master models the
  boot state only (the on-disc `Upgrades` bytes @0x2beea8 are all zero [D], and the port has no
  MP upgrade shop); the ≥4-pick out-of-bounds read stays a documented deviation.
- The botweap matrix fix IS part of this re-land: `botweap.rs`'s copy read the table as 5 rows
  of 10 u16 (sets interleaved), arming bots of sets 1..4 with sets 2/4/6/8's spawn weapons —
  the game indexes `PickupMatrix + set·10` in BYTES [M 0x129e48:44]; it now re-exports
  `equip::PICKUP_MATRIX`.

## Tests

`crates/nf-game/tests/mp_pickups_respawn.rs` (10 tests, numbers only, no ISO) + the unit pins
in `mp/equip.rs`. The registry/spawn/toggle/loadout assertions each cite the decompile line
they pin; sisters assert the refused/negative arms.
