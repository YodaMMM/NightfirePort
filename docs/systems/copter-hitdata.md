# The Hind's hitdata — the 9-part damage walk (`Copter_Update` 0x17dbb8:277-353)

## The walk [M Copter_Update 0x17dbb8:277-320]

Reached by states 0-4 (cases 5/6 return before it — the exempt tail). Nine part slots live at
`rec+0x30 + 4i` (`Copter_Create` 0x17d340 memsets 0x24 bytes; `Copter_Register` 0x17e530:6 fills
slot `param_2 < 9` with each SP-created script entity and flags every one but slot 2 with
obj+0xF0 bit 0x20 — slot 2 is the body, `Copter_GetBody` 0x17e4b8 reads `*(rec+0x38)`).

**Which entity lands in which slot** [M Script_EventHandler 0x1d94c0:172-175; D the script
dumps]: the callback runs from the script's event-0xF line with the PREVIOUS stream's entity and
the event's param 0 as the slot — not the stream index. 07000012 (0x060005BD): stream 0
Helicopter-Bodya 0x0200062A → slot 2, streams 11/13/16/18 → slots 5/6/7/8; slots 0/1/3/4 stay
empty and the main rotor (stream 4, 0x0200062B) is no part. 07000005 (0x06000046): stream 0 →
2, stream 2 → 3. 07000008 (0x06000051): 0→2, 2→3, 4→0, 6→1, 8→5, 10→6, 20→7, 22→8 (slot 4
unused). 0700000B (0x060008BE): one event (stream 4, slot 3) after an EMPTY stream 3 — a NULL
entity; no part is registered there. The port's `Copter::parts` holds the table (`Copter::register`,
fed by the `WireRegister` request); it used to take the stream index, which put the 07000012 rotor
(a 7.1-unit sphere) in slot 4 at ×100 — one P2K round (3.5) did 350 > 299.

**Only slot 2 is ever hit** [M Collide_PickObj 0x1eae80:58-82]: the pick mask always carries 0x20
(`uVar2 = 0x20`, widened by the HITTEST flags), and `(obj+0xF0 & mask) != 0` skips the object.
Every object pick goes through it (`Collide_Pick` 0x1eb320:320/373), the explosion sweep included
(`Collide_SphereIntersect` 0x1ec940:111 → `Collide_Pick`). `Copter_Register` sets 0x20 on every
part but slot 2, so bullets and blasts reach the body only (Bits scale 1.0). The port masks the
part targets with `copter::PICKABLE_SLOTS` in the bullet pass and the blast sweep. A round that
hits the body dies on it [M Bullet_CollisionHandler 0x12ce80 disasm 0x12cff0-0x12cff4], and a
world stop nearer than the body's sphere takes the round first [M Collide_Update 0x1ea398:216-229].

**Rounds to kill** (`copter::rounds_to_kill`; hp = DiffMod·299, death on `hp < rec+0x64`): the P2K's
contact damage 3.5 [D weapon record 2] → 86 rounds on d1 (85·3.5 = 297.5), the SG5's 2.75 [D
record 0x16] → 109 (108·2.75 = 297). Pinned by `crates/nf-view/tests/copter_hits_12_live.rs`.

**Rockets on the body** [M Bullet_CollisionHandler 0x12ce80]: the body contact runs the same explode
arms as any contact — the 0x4 arm `Explode_Create(+0x08, +0x08, +0x0C, …, HITDATA+0x40, …, 0x6000052)`
is gated only on the pierce flag, which the body's HITDATA leaves clear (`Collide_PickObj` memsets the
0x60-byte node, material 0; slot 2 has no obj+0xF0 0x20). `Explode_Propagate` case 0 then adds the
falloff ≈ +0x0C into rec+0x64. The AT-420 records 0x2C/0x2D: radius 8, damage 100, contact 1.0 (the
`Bullet_update` id override) [L ee_start_05.bin @0x2bf150] → 101 per body hit, **3 rockets** on d1
(303 > 299; d2 448.5 → 5, d3 598 → 6, d4 1196 → 12). The port used to kill the round on the body with
the 1.0 contact only; the bullet pass now queues the blast at the sphere entry point.

**07000008's shoot-down supply** [D/M]: script 0x06000051 registers 0→2, 2→3, 4→0, 6→1, 8→5, 10→6,
20→7, 22→8 — the body is stream 0 in slot 2, the only pickable part, as on 12; no slot 4. hp =
DiffMod·299 (DiffMod 1.0/1.5/2.0/4.0 by `GameState+0x40` [M Copter_Create 0x17d340]); no countdown
(p8/p9 = 0) and no channel kills it — only the damage test writes out ch 0x4B. Bond's kit is the 07
carry (`Player_InitWeapon` 0x1a7ef8 case 0x7000008 → RamLoad; P2K 48), the 9 mm boxes #292/#293 (20
each), and the AT-420 twice: the safe's #291 (4 rounds, forced [M Pickup_Create 0x18dda0:116-118]) and
the cable car's launcher + two rocket-ammo boxes (4 each, parented to the carriage [D 0x0600004C
streams 37-39], re-spawned at t1261). Rocket carry cap 4 [L ammo_data type 0xE +0x02] + clip 4 = 8 on
hand. The P2K alone cannot do it (86 rounds vs ≤ 48 + 40 + the guards' drops); the rockets are the
route. Pinned by `walkreal_07000008_live` (the guided rockets flown into the body, no refill).

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
record 96 included); the handguns/rifles Bond carries on 07000008 are 0x8/0x1. Slot 4's ×100 and
filter would apply to its chain, but no slot except 2 is ever picked (above), and no shipped copter
script registers slot 4. Slots 6-8 never add damage (scale 0).

## Explosions bypass the walk [M Explode_Propagate 0x17c518 case 0, :86-105]

The blast's sphere sweep hits the parts (class-0 script entities; the copter itself is 0x3b and
never matches; the pick gate above leaves the body as the only reachable part), and per matching part the falloff amount
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
