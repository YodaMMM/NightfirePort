# MIS01 (Paris Prelude): the opening sniper section

What the original shows (PCSX2, reached through the real menus: Play without saving, Operative,
Paris Prelude) [L 2026-11-20]: the mission does not start behind the wheel. Bond holds the
**Sniper Rifle** (HUD "24 / Sniper Rifle") on a moving vehicle above the riverside road and
shoots at pursuing cars below (black saloons and a truck), at night with lamp-lit facades. The
driving executable is already loaded at this point (the ACTION.ELF GameState words read as
garbage), so the section runs in DRIVING.ELF, not in an on-foot level.

Where its pieces live [D, `/DRIVING/MIS01.VIV` entry list]:
- `data\actors\anims\sniper.dat` / `sniper.rel`: an animation ELF object whose symbols include
  `e_sniper_fire`, `s_sniper_fire`, `s_sniper_idle`, `s_sniper_init`; `paris0.txt` names it.
- `data\audio\banks.ini` / the level `.h`: `sfx_par_sniperinit`, `sfx_par_sniperfire`,
  `sfx_par_sniperreload`, `sfx_par_sniperbolt`.
- `data\car\model\paradis_car.crp` + `paradis_car.atr`, with the driver model
  `data\actors\models\c_domm1.*`: the vehicle Bond rides in for this section [S: the pairing is
  read from the entry list, not traced].
- DRIVING.ELF `FUN_0020f2b8` 0x20f2b8 builds the driving engine's weapon table, which names
  "Sniper Rifle" [M]; so the sniper is a driving-engine weapon mode.

The mission number [M]: `FUN_001266a8` 0x1266a8 string-compares the mission name mailbox
0x1004F4 against six names (0x36a0f8..0x36a120) and returns 1..6 (0 for none); `FUN_00125bb8`
0x125bb8 stores it in `DAT_003c8234`. Its only other readers, `FUN_001265c0` 0x1265c0 and
`FUN_00126768` 0x126768, use it to index a per-mission table at 0x3277b0 (stride 0x70) of actor
model names (pedestrians and drivers). No code branches on the mission number to run a sniper
phase, so the phase is data-driven: the track's Map `AIEl` / script records are the next place
to look.

Not traced yet: the mission state machine that runs the sniper phase and hands over to the
Vanquish drive, the aim/zoom input, and how the pursuers are scripted (the Map `AIEl` record is
the candidate [driving-load-path.md]).

The port's driving view (`crates/nf-view/src/driving.rs`, behind `NF_DRIVING=1`) starts in the
Vanquish and has none of this section yet.
