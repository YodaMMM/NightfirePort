# weapon-fire-sound-ids — every weapon's first-person fire sound (G-029 remainder)

The first-person gun's fire sound comes from the weapon's animation script, not from a per-weapon
field: `AnimProcessScriptCmds` **0x1c1b38 op 1** (cmd[0] = op|len, cmd[1] = frame, cmd[2] = sfx id)
plays `Sound_Play3D(cmd[2] → remapped, pos, vol 100, outer 100, inner 0, pitch = Rand(500)−250, 0, 0)`
when the anim frame crosses cmd[1] (`SFXPreCache` up to 0x33 frames early for flagged scripts).
This page pins, per weapon, what that op-1 id becomes — from the decompile's remap and the disc's
own scripts — and separates the guards' fire path from it.

## The op-1 remap [M 0x1c1b38 case 1]

The id `cmd[2]` is replaced based on the firing object's owner (`obj+0x1C`) weapon byte —
`*(char*)(*(int*)(obj+0xDC) + 0x62)`, the player's current weapon record id (the TWIN id when a
fire-mode variant is out, which is why the twins appear below). Six rules, in file order:

| script id | weapon byte(s) | plays |
|---|---|---|
| 1 | 0x07, 0x09 | 2 |
| 0x403 | 0x03, 0x05 | 2 |
| 0xfa | 0x14 | 0x1fb |
| 0x106 | 0x1e..0x23 (`(w−0x1e) < 6`) | 0x284 |
| 0x1e1 | 0x10 | 0x602 |
| 4 | 0x16, 0x17 (`(w−0x16) < 2`) | 0x604 |

Anything else plays as-is — the P2K (0x02) is NOT in the table and its fire scripts' 0x403 is the
real id. Port: `nf_view::scene::remap_op1_sound` (called from the view-gun tick, the gadget loop
stop, and the MP remote-shot path); the table test `fix_weapon_modes_live.rs::
op1_remap_table_matches_the_decompile` pins the rule table itself.

## The per-weapon fire-sound table [D]

Disc data: `weapon_data` (115 × 0x10C at EE 0x2bf150, filled at boot [M 0x1b3488; docs/formats/
weapon-data.md]) — fire script hash at **+0xB0**, fire-on-last-round script at **+0xB4** (the
random pick under the +0x68 flag 0x80000 [M 0x1a5460]); scripts are archive kind-5 entries
(`nf_formats::anim::Script::parse`). The table lists every op-1 command of both scripts in command
order as `frame:id` with the id AFTER the remap (R marks a remapped one). Player-holdable weapons
are the selectable records (+0x04 == 1 [M 0x1a1894, the cycle's gate]) plus the fire-mode twin
records the view runs through the variant swap (0x03/0x05/0x07/0x09/0x0b/0x0f/0x13/0x15/0x17/
0x1b/0x1d/0x1f/0x21/0x23/0x25/0x27/0x29/0x2b/0x2d/0x2f/0x31 — the game's own remap keys on their
bytes [M 0x1c1b38], so they do fire in first person).

| w | fire (+0xB0) | fire-last (+0xB4) | drone/guard +0x60 |
|---|---|---|---|
| 01 fists | 7:0x142 | 9:0x142 | — |
| 02 P2K | 1:0x403 | 1:0x403 | 0x42c |
| 03 P2K sil. | 1:**2** (0x403 R) | 1:**2** (0x403 R) | 0x42b |
| 04 | 1:0x403 | 1:0x403 | 0x42c |
| 05 sil. | 1:**2** (0x403 R) | 1:**2** (0x403 R) | 0x42b |
| 06 | 1:0x1 | 1:0x1 | 0x42a |
| 07 sil. | 1:**2** (0x1 R) | 1:**2** (0x1 R) | 0x42b |
| 08 | 1:0x1 | 1:0x1 | 0x42a |
| 09 sil. | 1:**2** (0x1 R) | 1:**2** (0x1 R) | 0x42b |
| 0a | 1:0x1f7 | — | 0x41a |
| 0b | 1:0x1f7 | — | 0x41a |
| 0c | 1:0x1f7 | — | 0x41a |
| 0d | 1:0x1f7 | 1:0x1f7 | 0x41a |
| 0e | 1:0x1e1 | 1:0x1e1 | 0x1e1 |
| 0f | 1:0x1e1 | 1:0x1e1 | 0x419 |
| 10 | 1:**0x602** (0x1e1 R) | 1:**0x602** (0x1e1 R) | 0x603 |
| 11 | 1:0x449, 43:0x44b | 1:0x449 | — |
| 12 | 1:0x3 | — | 0x42e |
| 13 | 1:0x3 | — | 0x42e |
| 14 | 1:**0x1fb** (0xfa R) | — | 0x426 |
| 15 | 1:0xfa | — | 0x427 |
| 16 SG5 | 1:**0x604** (4 R) | — | 0x605 |
| 17 SG5 var. | 1:**0x604** (4 R) | — | 0x605 |
| 18 | 1:0x4 | — | 0x430 |
| 19 | 1:0x4 | — | 0x430 |
| 1a | 1:0x40d | — | 0x428 |
| 1b | 1:0x40c | — | 0x429 |
| 1c | 1:0x1de, 53:0x1e0 | — | 0x42f |
| 1d | 1:0x1de | — | 0x42f |
| 1e | 1:**0x284** (0x106 R), 54:0x107 | 1:**0x284** (R), 54:0x107 | 0x41f |
| 1f | 1:**0x284** (R), 54:0x107 | 1:**0x284** (R), 54:0x107 | 0x41f |
| 20 | 1:**0x284** (R), 54:0x107 | 1:**0x284** (R), 54:0x107 | 0x41f |
| 21 | 1:**0x284** (R), 54:0x107 | 1:**0x284** (R), 54:0x107 | 0x41f |
| 22 | 1:**0x284** (R), 54:0x107 | 1:**0x284** (R), 54:0x107 | 0x41f |
| 23 | 1:**0x284** (R), 54:0x107 | 1:**0x284** (R), 54:0x107 | 0x41f |
| 24 | 1:0x106, 54:0x107 | 1:0x106, 54:0x107 | 0x41e |
| 25 | 1:0x106, 54:0x107 | 1:0x106, 54:0x107 | 0x41e |
| 26 | 1:0x106, 54:0x107 | 1:0x106, 54:0x107 | 0x41e |
| 27 | 1:0x106, 54:0x107 | 1:0x106, 54:0x107 | 0x41e |
| 28 | 1:0x106, 54:0x107 | 1:0x106, 54:0x107 | 0x41e |
| 29 | 1:0x106, 54:0x107 | 1:0x106, 54:0x107 | 0x41e |
| 2a | 1:0x40b | — | — |
| 2b | 1:0x40b | — | — |
| 2c | 1:0x10c | — | 0x42d |
| 2d | 1:0x10c | — | 0x42d |
| 2e | 1:0x10c | — | 0x42d |
| 2f | 1:0x10c | — | 0x42d |
| 30 | 1:0x10c | — | 0x42d |
| 31 | 1:0x10c | — | 0x42d |
| 32 | 2:0x421, 269:0x423 | — | 0x423 |
| 33 | 2:0x420, 14:0x422 | — | 0x422 |
| 34 | 16:0x1e4 | 22:0x41 | 0x1e4 |
| 35 | 16:0x1e4 | 24:0x41 | 0x1e4 |
| 36 | 16:0x1e4 | 24:0x41 | 0x1e4 |
| 37 | 18:0x458 | — | 0x1e4 |
| 38 | — (no fire script) | — | — |
| 3a | 19:0x445, 64:0x446 | — | — |
| 3b..40 | 22:0x447, 84:0x448 | — | — (3b/3c..40: +0x60 = 0) |
| 41 | 2:0x27f | — | 0x27f |
| 42 | 2:0x41b | — | — |
| 43 | 2:0x5ce | — | 0x27f |
| 44 | 2:0x5ce | — | 0x27f |
| 45 taser | 15:0x41 | — | **0x5ef = the guard path SKIPS it** [M 0x178f68] |
| 46 | 1:0x1 | 1:0x1 | — |
| 4a | 1:0x3d | — | — |
| 4c | 1:0x3d | — | — |
| 4e | 1:0x12 | — | — |
| 4f | 1:0x12 | — | — |
| 50 | 1:0x1f5 | — | — |
| 51 | 1:0x1f5 | — | — |
| 52 | 29:0x450, 31:0x44e | — | — |
| 56 | 11:0x452, 18:0x452, 26:0x452, 33:0x452, 39:0x452, 48:0x452, 55:0x452, 62:0x452 | 9:0x452, 19:0x452, 26:0x452, 34:0x452, 40:0x452 | — |
| 57 | same 8× 0x452 | same 5× 0x452 | — |
| 58 | 16:0x456, 72:0x454 | — | — |
| 59 | 24:0x458, 65:0x448 | — | — |
| 5b | 24:0x458, 65:0x448 | — | — |

No fire and no fire-last script in 0x01..0x5b: **0x38, 0x39, 0x47, 0x48, 0x49, 0x4b, 0x4d, 0x53's
scripts carry no op-1 (its handles exist), 0x54, 0x55, 0x5a, 0x5c..0x68** — these make no
first-person fire SOUND via op 1 (0x53's row: the probe resolves its handles but finds no op-1
command; the test pins the empty op-1 list, not the handle state). The MP-only variant records
0x69..0x6f carry scripts (0x420/0x422, 0x447/0x448, 0x10c) but no selectable flag and no SP
loadout ever equips them.

Verification: the port's `remap_op1_sound` matched this table on every row — no fix was needed.
The one decompile-vs-table subtlety: the remap keys on the CURRENT weapon byte, so the silenced
twins (03/05/07/09) remap even though their +0x04 selectable flag is 0 — they fire through the
variant swap [M 0x1a2e38 case 0xb→0xc + the +0xC8 script].

## The guards' fire path — unchanged by any of this [M]

`DroneWeap_FireWeapon` **0x178f68** plays `weapon_data[w] +0x60` (u16, the third out-param of
`AnimDatumGetWeaponInfo` 0x1c7900 beside +0x4C/+0x80 — docs/fields/weapon_data.md), SKIPPED when
the value is 0x5ef, at `Sound_Play3D(id, drone+0x30, vol 25, outer 100, inner rand, 0, 0, 0)`.
That path never reads an animation script and never applies the op-1 remap — the port reads the
same record field directly (`nf_game::drone.rs`, the `DroneWeap_FireWeapon` shot block) and the
sister test pins that the remap is the identity on every weapon's +0x60 value. Sample rows [D]:
w02 → 0x42c, w03 → 0x42b (the silenced twin's own, quieter guard sound), w16 → 0x605, w14 →
0x426, w45 → 0x5ef (skipped — the taser's guard shot makes no DroneWeap sound).

## Tests

`crates/nf-view/tests/weapon_fire_sound_ids_live.rs` (NF_ISO-gated, NF_SILENT, the real
`nf_view::load_dev` path — the weapon-asset merge loads the FP scripts):

1. `every_holdable_weapons_fire_script_op1_matches_the_table` — the full table above, computed
   live from `weapon_data` + the parsed scripts and asserted row by row.
2. `the_remap_never_touches_the_guard_fire_path` — sister: `remap_op1_sound(+0x60, w) == +0x60`
   for all 115 records; the +0x60 sample rows; the 0x5ef skip on 0x45.
