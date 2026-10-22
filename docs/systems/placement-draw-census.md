# Placement draw census — every placement the game draws must be in the port's draw list

Evidence tags: [M] decompile (ACTION.ELF file+line), [M-dis] instruction-level, [D] disc data,
[S] suggested. The census test: `crates/nf-view/tests/placement_draw_census_live.rs` (ISO-gated).

## The game rule

`parsemap_block_map_data_dynamic` 0x1d1b68 walks the main map's dynamic placement records and
calls `parsemap_create_dynamic_objects` 0x1d04f0 only when the record's kind word has neither
skip bit clear-checked (`kind & 0xa000 == 0` — 0x8000 records are the static cel pass) AND the
placement's glist resolved (by index into the cel list, or `hashtable_getitem(hash)` which
returns 0 on a miss) [M 0x1d1b68:77-97]. The dispatcher switches on the kind byte (flags & 0xff)
and calls one create fn per kind; every case is gated on the load pass flag except 0xde
[ M 0x1d04f0:64-544]. A placement "draws" when its create fn attaches draw data to the created
object:

- **glist attach**: `Control_SetGList` / `Control_CreateObjEx` (obj+0xd8 = glist, obj+0xF0 =
  *(glist+0x44) | mask) or a direct `obj+0xd8 = glist` store — the created obj renders its
  glist's meshes (an empty glist draws nothing; the glist's own 0x10 no-draw / 0x400 mirror-only
  bits gate it where the creator copies glist+0x44 [M scene.rs's fix-striped-boxes2 census]);
- **skin drone** (0xf, `Drone_Create` 0x136be0): the character pass;
- **script entity** (SP): `SP_Create` / `SP_CreateScriptPlayer` instantiate objects by hashcode
  (`Script_CreateEntity` 0x1d8968 → `hashtable_hashcode_to_celglist`);
- **sky object** (0x2a / 0xd4).

## The Exchange helicopter (the user report) — root cause

Placement #45 on 07000008 (kind 0x2F, hashcode 0x200035d "Helicopter-Body" [D #45]): the game's
`Copter_Create` 0x17d340 creates a class-0x7c control obj with **NO glist** (obj+0xD8 = 0,
obj+0xFA |= 0x44 [M 0x17d340:60-66]) and hands the draw to a script player — `SP_Create` with the
placement's params 4/5 as the SCRIPT hashcodes (0x06000051 the parts assembly, 0x06000050 the
flight [D #45, copter.rs]; the props block holds prop N at +0x2c+4N, so +0x3c/+0x40 are params
4/5 [M 0x1d1b68:62-76]). Prop 0 != 0 marks a marker copter with no object [M 0x17d340:43-45].
The draw data is the placement's OWN hashed object. The port had no pass for 0x2F — the body
never drew. **Fix**: a copter pass in scene.rs (after the doors pass) adding the placement's own
hashcode object as a World instance at the placement transform (the SP's create pos).

Rook: every kind-0xF placement on 07000008 resolves its param-0 skin (0x5000012/13/14 guards,
0x500002e/2f/30/3b — the probe test prints them) — the character pass draws them at level start,
so the census finds no missing actor placement there; "Rook missing" needs the main agent's
render check of the ride itself (the copter's flight script poses the body; Rook boarding it is
script-side).

## The per-kind table (create fn [M], verdict)

| kind | create | draws? |
|---|---|---|
| 0xf | Drone_Create 0x136be0 | skin character |
| 0x20 | Break_Create 0x12c230:19 Control_SetGList | glist |
| 0x21 | Ripples_Create 0x1acc90:26-36 | glist |
| 0x22 | Ladder_Create 0x1966c0:21 | glist |
| 0x24/0x2d | Player_AddNewStartPos 0x195c00 | no (start positions) |
| 0x25/0x26 | MP_Register* | MP-only |
| 0x27 | GT_Create 0x17ffa0:70-71 Control_CreateObjEx | glist |
| 0x28 | Sensor_Create 0x1af010:124 | glist [S — needs the live no-draw-bit check] |
| 0x29 | Switch_Create 0x1adcc0:44-45 SP_Create(prop 1) | SP |
| 0x2a | View_AddSkyObj 0x1e79a0 | sky |
| 0x2b | LeafGen_Create 0x1acbd0:17 | glist (leaf FX) |
| 0x2c | Lightning_Create 0x18cde8 | no glist (bolt renderer) |
| 0x2e | Lock_Create 0x1ae1f8:55 | glist |
| 0x2f | Copter_Create 0x17d340:68-70 | SP (body = the placement's own hash) |
| 0x30 | Monitor_Create 0x1b08b0:26 Control_SetGList | glist |
| 0x31 | FuseBox_Create 0x1b0448:29-30 SP_Create(prop 0) | SP |
| 0x32 | RainBox_Create 0x18d840:19 | glist (rain FX) |
| 0x33 | Hint_Create 0x1b3248:29 Control_SetGList | glist (difficulty-gated) |
| 0x34 | GunImp_Create 0x131f20:68-69 | glist (MP-gated) |
| 0x35 | PCQWorm_Create 0x1b07a8:12-13 | glist |
| 0x36/0x37 | Sub_/MiniSub_Create (AnimObjectNew) | anim object |
| 0x38/0x39 | SpaceLaser/Missile | no glist (dedicated renderers) |
| 0x3a | Grapple_Create 0x198298:17 | glist |
| 0x3b | DynamicObject_Create 0x18dbf8:19-23 | glist (hidden-until-channel) |
| 0x3e | Wire_Create 0x1981d8:24 | glist |
| 0x3f | CreepWall_Create 0x196fc8:23 | glist |
| 0x41/0x4e | Inverter/Light | no |
| 0x4a | Apocalypse_Create 0x18dcc0:20 | glist |
| 0x71 | rotor_init 0x18f4b8:24-25 | glist |
| 0xd2 | Shooter_Create 0x17d030:16 | glist |
| 0xd4 | Cloud_Create 0x18d620 | sky |
| 0xd5 | OneSided_Create 0x1e7ce8:14 | glist |
| 0xd8 | RB_Create 0x1d6bf0:52 | glist |
| 0xd9 | SP_CreateScriptPlayer 0x1db9a8 | SP (no-op if both hashes 0) |
| 0xda | Car_Create 0x1305b0:20-21 | glist (MP-gated) |
| 0xdb | Door_Create 0x136100:71 Control_SetGList | glist |
| 0xdc/0xe8/0xea-0xef/0xf4 | Trigger_Create 0x1b1720:92-95 | glist [S — no-draw-bit check] |
| 0xde | Searchlight_Create 0x1cb4f8:73-75 | glist (parts pass) |
| 0xdf | Env_Create 0x17b968 | no obj |
| 0xe0 | Creature_Create 0x134ab8:55-56 | glist |
| 0xe1 | Destroy_Create 0x12c318:23 | glist |
| 0xe2 | SS_Create 0x1a9c58:53 Control_SetGList | glist |
| 0xe3 | Flicker_Create 0x1acb00:49-52 | glist |
| 0xe4 | Hurt_Create 0x17e870:52 | glist |
| 0xe5/0xe6/0xe9 | cover/AI nodes | no obj |
| 0xe7 | ThirdCam_Create | no glist |
| 0xf0 | Pickup_Create 0x18dda0:134-137 | glist |
| 0xf1 | DroneSpawner_Create | no glist |
| 0xf2 | Emitter_CreatePlist | no glist (particle renderer) |
| 0xf3 | Env_Tree_Create 0x17c200:17 AnimObjectNew(prop 0) | anim object |
| 0xf5/0xf8 | Drone_AIVolume_Create | glist as bounds only (+0xF0 \|= 0x30) |
| 0xf9 | SoundTrigger_Create 0x1b2628:55-57 | glist [S] |
| 0xfa | ThirdIcon_Create 0x197070:33-37 | glist |
| 0xfb | MusicTrigger_Create 0x1b30e0:25-27 | glist [S] |
| 0xfc | Corona_Create 0x18da00:17-20 | glist (corona pass) |
| 0xfd | CamSubject_Create 0x1bc5d8:17-20 | glist |
| 0xfe | Mine_Create 0x17ce28:50 | glist |

## The port's coverage (what the census checks)

statics (0x8000) via `scene_instances(true)`; breakables 0x20; unanimated 0xE2 (+ the animated
SS pass); characters 0xf (resolvable skins) via `character_instances`; sky 0x2a/0xd4 via
`sky_instances`; the MISC_KINDS pass 0x2e/0x30/0x34/0x35/0x3b/0x71/0xd5/0xd8/0xe1/0xe3/0xda/0xfa/
0xfd/0xfe/0x4a; grapple 0x3a; wires 0x3e; ladders 0x22; doors 0xdb; searchlight parts 0xde;
LOAD_LEVEL triggers 0xe8; pickups 0xf0; coronas 0xfc; **copters 0x2f (this task)**.

## The remaining misses (the census table, 378 placements over 21 levels)

Grouped (level, kind, count) — each row's create fn is in the table above; the full placement
list prints from the census test (`--nocapture`):

- 0xd9 SP script players: 01(1) 03(1) 05(1) 06(1) 07(1) 08(1) 09(1) 0a(10) 0b(12) 0c(1) 11(1)
  12(6) 13(1) 14(24) 16(24) 4a(1) — the SP's own placement hashcode holds the entity object; the
  port's script system does not draw SP placement entities yet.
- 0x29 Switch SP models: 09(24) 0a(18) 0b(?) 14(4) 16(2).
- 0x28 sensors: 01(1) 02(3) 0a(28) 0b(?) 12(10) 14(28) — [S] the glists may be no-draw-flagged.
- 0xea/0xeb/0xf4 triggers: 08(6) 0a(28) 11(10) 12(38) 16(2) 41(8) — [S] same.
- 0x27 GT turrets: 0a(12) 0b(10) 12(6).
- 0x31 fuseboxes: 06(1) 08(1) 14(10).
- 0xf3 env trees: 02(6) 04(2) (+ others — anim-object support needed).
- 0x3f creepwalls: 41(4).

A NEW (level, kind) miss row fails the census; fixing a kind removes its row from
`KNOWN_MISSES` in the test.

## Render spots for the main agent (the copter fix)

- `--bond` at the Exchange extraction pad watching placement #45 (Helicopter-Body): the copter
  body should now stand at its placement before the extraction; the flight still won't move it
  (the sim's `body_obj` wiring is not posing the instance yet) — the static pose is this task's
  deliverable.
- 07000012/07000014: the SP entities (0xd9) listed in the census — expected still missing
  (documented misses).

## Live checks for the main agent

- 07000005's touch-once trigger (0xea) glist: gdbwatch `obj+0xF0` for a trigger obj during play —
  does *(glist+0x44) carry 0x10 (no-draw)? If NOT, the game renders trigger boxes and the port's
  trigger rows are real misses; if yes, they are invisible volumes and the census rows should move
  to the not-drawn side.
- Same for 0x28 sensor glists.
- The copter at rest on 07000008: does the real game show the body AT the placement transform
  before the extraction (vs hidden until the show channel)? SP_HideObj gates it [M 0x17d340:72-75]
  — the port currently draws it unconditionally; if the game hides it at start, the pass needs the
  channel gate.

## Usage

not visible to me — this session's interface shows no token, credit, cost or context-window
figures.
