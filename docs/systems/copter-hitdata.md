# The Hind's hitdata — the 9-part damage walk (`Copter_Update` 0x17dbb8:277-353)

## The walk [M Copter_Update 0x17dbb8:277-320]

Reached by states 0-4 (cases 5/6 return before it — the exempt tail). Nine part slots live at
`rec+0x30 + 4i` (`Copter_Create` 0x17d340 memsets 0x24 bytes; `Copter_Register` 0x17e530:6 fills
slot `param_2 < 9` with each SP-created script entity and flags every one but slot 2 with
obj+0xF0 bit 0x20 — slot 2 is the body, `Copter_GetBody` 0x17e4b8 reads `*(rec+0x38)`).

Per slot i:
1. `Collide_FilterBullets(part+0xD0, Bits[i].filter)` [M 0x1ece30, jal 0x17e1bc]: for every chain
   entry whose +0x58 owner is a class-5 BULLET object, the bullet's weapon record's class word
   (`weapon_data+0x10`, read `*(ushort*)(*(int*)(*(int*)(bullet+0xE0) + 0x44) + 0x10)`) is masked
   against the filter; a non-zero AND marks the entry's +0xC with 1e8 and the sort pass frees it
   [M :26-33 + the SrtList rebuild]. Non-bullet owners pass untouched. Filter 0 keeps everything.
2. Every surviving entry with `+8 > 0` adds `damage · Bits[i].scale` to **both** `rec+0x64`
   (damage) and `rec+0x6c` (the attack accumulator) [M :293-307, the accumulate stores
   `swc1 → 0x64($s3)`/`0x6c($s3)` at 0x17e23c/0x17e250]. The +8 damage is the caller's store
   (`Coll_AddHitToList`'s float lands at +0xC as the sort key; the damage is written beside it —
   explosions: `*(float*)(node+8) = fVar10` [M Explode_Propagate 0x17c518:215-217]; bullets: the
   HITTEST's contact damage).
3. A counted entry whose bullet's shooter (`BU+0x30`) is a class-3 object (the player) latches
   `bVar14` [M :300-304]; after the walk `PlrStat_LogShotHitEnemy(0)` fires once [M :314-316, jal
   0x199840 at 0x17e268].

## The Bits table [L EE 0x2a37e8, 72 bytes, identical in ee_snapshot.bin and ee_start_05.bin]

| slot | filter | scale |
|------|--------|-------|
| 0 | 0x0000 | 2.0 |
| 1 | 0x0000 | 3.0 |
| 2 (body) | 0x0000 | 1.0 |
| 3 | 0x0000 | 2.0 |
| 4 | 0x0010 | 100.0 |
| 5 | 0x0000 | 1.0 |
| 6-8 | 0x0000 | 0.0 |

The weapon classes [L the same snapshot's weapon_data 0x2bf150 census of the +0x10 words]:
0x10 = records {51, 96, 97, 103, 106-113} (the heavy/explosive family, the copter turret's own
record 96 included); the handguns/rifles Bond carries on 07000008 are 0x8/0x1 — so slot 4 (×100)
counts their hits and drops only the 0x10 family. Slots 6-8 never add damage (scale 0).

## Explosions bypass the walk [M Explode_Propagate 0x17c518 case 0, :86-105]

The blast's sphere sweep hits the parts (class-0 script entities; the copter itself is 0x3b and
never matches), and per matching part the falloff amount
`damage · (1 − (int)(dist − obj_radius)/radius)` goes STRAIGHT into `rec+0x64` and `rec+0x6c` —
no Bits scaling, no filter, no HITDATA chain. Rockets/grenades therefore land through here.

## The smoke/gas branch [M :321-353]

Past half hp (`rec+0x64 > rec+0x68 · 0.5`) with slot 3 registered (`rec+0x3c != 0`), every tick
rolls `Rand_Rand(10)`; the ≥ 8 arm draws `Rand_FRand(π)` [:341] + `Rand_FRand(2.0)` [:346] (jal
0x1e3780 at 0x17e320/0x17e338) and spawns `Gas_Init` (0x1aacc8) at the slot-3 object's +0x80 with
mesh 0x200001f, kind 3, the size `FRand(2)+3`, and the colour word
`(1 − damage/hp)·127` packed `amount<<24 | amount<<16 | amount<<8 | (0xbf−amount)` [M :339-350].
Gas_Init's internal draws for kind 3: the spin pair [M 0x1aacc8:72-80; the default velocity branch,
no wind draws].

## The death [M :358-371, disasm 0x17e390-0x17e3c8]

`sltiu(state-4, 2); bnez → skip` — states 4/5 exempt; every other state with `rec+0x64 >
rec+0x68` (hp) goes to state 4, or straight to 6 on GameState 0x0700000b/0x07000012; the arrival
channel `rec+0x72` writes either way. State 4 finishes the path, adopts `rec+0x58` (the group's
kind-2 arrival path) with t = 0 and writes the channel [M :244-254] — on 07000008 that channel is
0x4B, which drives the mission's mux #276 → ch 100.
