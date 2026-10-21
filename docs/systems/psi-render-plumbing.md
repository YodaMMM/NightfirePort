# psi render plumbing — the never-mentioned front-end/render cluster (unported-front-end-menus-1)

The whole-ELF scan (master 20874c31, 2026-10-20) flagged 25 ACTION.ELF functions in the
`psi*`/GS-render/`Car_*` area whose names appeared nowhere in `crates/*/src`. Survey result:
**all 25 are reached by the shipped game** (chains below, [M] = decompile + grep caller links;
ELF word-refs/jals verified in work/elf/ACTION.ELF), so none go to `docs/port-map/unreached.md`.
21 are **cited as replaced** at the Rust code that does the same job; 4 `Car_*` functions are
**ported** in `crates/nf-game/src/car.rs`.

## Live roots (all [M], grep-verified caller links)

`main` 0x109080 → `CreateDB` 0x10b000(0); `GameFlow_Main` 0x1c98c8 → `Game_Draw` 0x1e6f50
(→ psiPreDraw 0x121590, psiFadeView 0x10b9c8, DrawDrops 0x18cb90, View_DrawObjList 0x1e69a0) /
`ResetMap_Load` 0x1d6360 (→ psiPreGameRun 0x11e2e8, psiResetResources 0x11f390,
ResetMap_GameInit 0x1d5080) / `Game_Run` 0x1c9790 (→ psiPreGame_Run 0x109520, MP_Update
0x184bd0, control_movement_object_handler 0x1336f8).

## Table

| Address | Function | Reached via | Outcome |
|---|---|---|---|
| 0x1095d0 | CacheFlipBuffers | psiPreDraw 0x121590:20 ← Game_Draw, per frame | cited, gpu.rs texture block |
| 0x10b000 | CreateDB | main 0x109080:33 (flag 0); psiPreGame_Run 0x109520:11 (flag 1) | cited, gpu.rs texture block |
| 0x117ce8 | CreateZClearSprite | psiPreGameRun 0x11e2e8:70; consumed psiClearZ 0x10b9a0 ← psiPreDraw:19 | cited, gpu.rs depth clear + module block |
| 0x1181b0 | CreateTotalScreenSprite | psiFadeView 0x10b9c8:9 ← Game_Draw:206,354, per frame | cited, hud.rs fade quad |
| 0x118718 | CreateMoveRG2BA | psiPreGameRun:68; consumed psiDrawFogPass 0x10b960:7 | cited, gpu.rs fog/module block |
| 0x119228 | CreateFogSprite | psiPreGameRun:69; consumed psiDrawFogPass:8 | cited, gpu.rs fog/module block |
| 0x11a160 | CreatePointerSpriteList | psiDrawParticleList 0x122d50:106 ← DrawDrops 0x18cb90 / Emitter_Draw 0x17b708 | cited, world_fx.rs ParticleSprite |
| 0x11a900 | CreateTexTranPacket | CreateSpaceAndPackTextures 0x11dcf8:248,277 | cited, gpu.rs texture block |
| 0x11ad08 | CreateTexUploadPacket | PS2WipeVram 0x11cab0:87 ← psiPreGame_Run:10 / psiResetResources:17 | cited, gpu.rs texture block |
| 0x11cc70 | CacheTextureSpace | CacheAndFillNewGlistTextures 0x11f180:48, psiFont_SetTexture 0x120880:20 | cited, gpu.rs texture block |
| 0x11d7d8 | CreateSortedTextures | psiPreGameRun:80, per level load | cited, gpu.rs texture block |
| 0x11d8b8 | CheckAndAddTextureToSpace | Reduce 0x11daf0:69 ← psiPreGameRun:81 | cited, gpu.rs texture block |
| 0x11dcf8 | CreateSpaceAndPackTextures | psiPreGameRun:82, per level load | cited, gpu.rs texture block |
| 0x11e640 | PageConv4to32 | Conv4to32 0x11e8d8:161 | cited, texture.rs + gpu.rs block |
| 0x11e8d8 | Conv4to32 | CreateSpaceAndPackTextures:208; BlockConv4to32 0x11e530 | cited, texture.rs + gpu.rs block |
| 0x11eba8 | PageConv8to32 | Conv8to32 0x11eeb0:156 ← CreateSpaceAndPackTextures:212, psiDecompressWoman | cited, texture.rs + gpu.rs block |
| 0x11f180 | CacheAndFillNewGlistTextures | psiDrawSkinObjectMatrix 0x111828:117, DrawThisBox 0x1230d8:14 | cited, gpu.rs texture block |
| 0x120928 | CheckSizeAndShrink | the Create* packet builders incl. psiCreateShard 0x1178e0:85 | cited, gpu.rs texture block |
| 0x120970 | CheckSize | same + CheckSizeAndShrink; `end − start` (8 bytes) | cited, gpu.rs texture block |
| 0x121a60 | CheckObjectClip | psiDrawSkinObjectMatrix 0x111828:81 ← AnimObjectDraw 0x1c4490, per frame | cited, gpu.rs frustum block |
| 0x121e30 | CheckObjectClipBox | RecurseAndDrawBoxes 0x1233e8:11-37 ← psiDrawObjectMatrix 0x124708 | cited, gpu.rs frustum block |
| 0x130060 | Car_SetBits | Car_CollisionHandler 0x131838 (jal 0x131bf0) ← control_funcs[0x28]+4 dispatch | **ported**, car.rs `Car::set_bits` |
| 0x132d78 | Car_Init | ResetMap_GameInit 0x1d5080 (jal 0x1d51ec) | **ported**, car.rs `CarSystem::init` |
| 0x132da8 | Car_Reset | MP_RestartScenario 0x183fc0:13 (jal 0x183fec) | **ported**, car.rs `CarSystem::reset` |
| 0x132e78 | Car_Delete | control_delete_object 0x133178:16-18 via control_funcs[0x28·0xC+8] | **ported**, car.rs `CarSystem::delete` |

## The Car dispatch row [M, ELF-verified]

`control_funcs` (base 0x26f890, rows of 3 fn ptrs update/collision/delete, stride 0xC, row =
`obj+0xff` type byte): the car row sits at 0x26fa70 — `Car_Update` 0x1307d8 / `Car_CollisionHandler`
0x131838 / `Car_Delete` 0x132e78 (ELF word refs at 0x26fa70/74/78; the update slot is dispatched at
0x1336f8:62-64, the delete slot at 0x133178:16-18; `Car_SetBits` is jal'd from 0x131bf0 inside the
collision handler). Car type = 0x28 [M Car_Create 0x1305b0 stores 0x28 at obj+0xff]. Cars spawn
from the disc's 0xDA placements on every arena except Sub Pen [D census,
mp_map_objects_live::probe_cars_on_all_arenas], so the whole row is live in MP.

## What was ported (car.rs) and what it returns

- `CarSystem::init` = Car_Init 0x132d78 (`NumTanks = 0; memset(&Tanks, 0, 0x20)` — 8 slots).
- `CarSystem::reset` = Car_Reset 0x132da8: per tank state(+0xf4)=0, timer(+0xf6)=0,
  driver(+0xec)=0, damage(+0xf8)=0, turret pitch/yaw(+0xd0/d4) zeroed, live matrix restored from
  TankSpawns[slot]. Does NOT touch heat/cooldowns/MG ammo (create-only, 0x1305b0:62-63).
- `CarSystem::delete` = Car_Delete 0x132e78 = tail-call Car_Deactivate 0x131d88.
- `Car::set_bits` = Car_SetBits 0x130060's data/flow: turret yaw/pitch integration
  (∓axis·FRM·0.05), yaw wrap to ±π, pitch clamp `[−0.705, (1−|yaw·1.2732395|)·0.12900001+0.08]`,
  cooldown decrements (unconditional), shell fire (action 9 ∧ cd≤0 ∧ !overheat → cd 6.0, heat +10,
  latch >100) and MG fire (action 0xC ∧ cd≤0 ∧ ammo≠0 → ammo −1, cd 120.0). The sound ids
  (0x10a shell / 0x40c MG) and weapon defs (0x2c55d0 / 0x2c1d48) are returned as a `SetBitsFire`
  for the caller to emit (the port keeps Sound/Bullet with the sim caller).
- New `Car` fields: `damage`, `turret_pitch`, `turret_yaw`, `mg_cooldown`, `mg_ammo`.

## Draw-side observation points for the main agent (no drawing done here)

1. **Fade colour**: `psiFadeView` 0x10b9c8 fades to the viewer's own r,g,b (the 4 u8 args of
   CreateTotalScreenSprite 0x1181b0, alpha in the RGBAQ top byte); the port's hud.rs fade quad is
   hardcoded black. `--bond x,y,z,yaw,pitch` at any level-start fade: the game's screen should
   come up from the fog/level colour, not pure black.
2. **Tank turret**: when a remote tank is driven, the turret sub-object rotates by
   RotMatrixY(turret_yaw) and the barrel by RotMatrixXY(turret_pitch, yaw) [M 0x130060:121-129];
   the barrel sits at car_pos + dir·(turret model +0x38 ·0.9) + up·0.162 [M :140-158]. The port's
   `Car` now carries turret_pitch/turret_yaw for the renderer to compose.
3. **Fog plane**: the game's fog pass draws FogOverlayPacket (PS2FogR/G/B, fog factor 0x7fff)
   behind Z 0x7FFF after the z-read pass; the port's per-pixel shader blend must match at
   depth discontinuities (existing `--no-fog` toggle isolates it).

## Evidence tags

[M] decompile files in `work/decomp/ACTION.ELF/` (file = function, `:line` = decompile line) and
grep caller links; ELF references verified against `work/elf/ACTION.ELF` (4-byte word scan + jal
scan). [D] only for the car-placement census (cited from mp_map_objects_live). No [L] items —
no live PCSX2 checks were needed; proposed live checks are in the task report.
