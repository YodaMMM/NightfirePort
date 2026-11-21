# The car models (`data\car\model\<car>.crp` + the shared `.ssh` sheet)

Status: **decoded to the draw buffers** (2026-10-15). Parser:
`crates/nf-formats/src/carmodel.rs`; goldens: `crates/nf-formats/golden/carmodel_golden.tsv`
(the five plain `MISC.VIV` cars) in `crates/nf-formats/tests/carmodel_golden.rs`; CLI:
`nf-extract carmodels <file.crp|file.viv> [out-dir]` (writes OBJ/buffers locally, never
committed). Evidence tags: [M] code (function + address, DRIVING.ELF), [D] disc bytes,
[S] suggested. EE address = file offset + 0x107080 (docs/atlas/driving.md). The container
layer is docs/formats/crp.md (the `.crp` CARP record pool) — this file is what is *inside*
the car `.crp`s.

## 1. Where the files live [D]

`data\car\model\<car>.crp` in every mission `.VIV` (RefPack-compressed, `nf_formats::refpack`)
and `MISC.VIV` (5 plain cars: `taxi`, `bus_paris`, `bus_tokyo`, `junglesnow`, `jungle_truck`);
next to each `.crp` a `.ssh` sprite/texture sheet (e.g. `taxi.ssh`, 44,464 B) whose filename is
the model's directory `ns#0` record payload [D]. The game resolves the filename from the
`rendername` vehicle attribute into `data\car\model\` [M attribute consumers FUN_001c8010
0x1c8010 / FUN_001c7360 0x1c7360 with the directory strings at EE 0x36b110/0x36c0e0/0x377c10/
0x379a50/0x37a470] and loads it through the article cache [M FUN_001c7488 0x1c7488:
cache probe FUN_001cb590, miss → `FUN_001a9cc8`/`FUN_001a9d70` (the async read that also runs
RefPack), the loaded image root stored at obj+0x10]. The car article is the pool `Arti`, found
with the pool scan `FUN_002d51f0(root,'Arti')` and matched by its window `Name` record
[M 0x1c7488]; the model object is built from it by `FUN_001c7658` 0x1c7658 → `FUN_001c7940`
0x1c7940.

## 2. The article window: record vocabulary [M + D, measured on all five MISC cars]

| Record | Payload / meaning |
|---|---|
| `Base` | 12 rows × 0x20 B of f32 4x4-ish transforms (taxi; jungle_truck has more) |
| `Name` | the article name string (`taxi`) [M 0x1c7658] |
| `Skel` | **the part table** — §3; the wheel/hub attachment points live here |
| `an#N` (1..4) | animation refs; payload = inline 4-byte refs + f32s |
| `aq#N` (1..2) | 8-record windows (animation queue) |
| `as#N` (0..7) | **the model-set variants** — one per game variant; payload = an inline script of 4-byte `(BE-char tag, u16 instance)` refs (`Base#1`, `ef#1`, `an#0`, `sr#34`…) mixed with f32s [M 0x1c7658 looks the set up with `FUN_001c6fb8(arti,'as  ')`; the `__Model:::` module symbols name them `{as  NNNN}`] |
| `cs#0` | empty (32 zero bytes in taxi) |
| `ef#N` (1..5) | effect streams (ef#1 = 1088 B: f32 position rows + u16 fields) |
| `el#N` (1..4) | element refs (inline tag refs) |
| `in#0` | **the instance list**: count × 0x40 B; +0xc = flags byte (bit 0x10 counted by the walker), +0x1c = the inline `sa#N` reference that the loader relocates to the sub-image [M FUN_001a1bf0 0x1a1bf0 walks 0x40-byte entries, reads +0xc and dereferences +0x1c] |
| `si#60` | 12 B |
| `sr#N` (12..46) | **named resources**, payload = NUL-terminated name, e.g. `GIOT::Cr_Dmg2`, `GIOT::Ex_Glass`, `CARP::taxi::{ef  0001}::896`, `EAGL::taxi::{as  0004}::Model[0].00` [D] — the sr names map the module's model descriptors |
| `ns#0` (dir) | the texture-sheet filename (`taxi.ssh`) [D] |
| `ELFd/ELFr/Sect` (dir) | the embedded MIPS ET_REL module — §4 [M loader FUN_001a9600 0x1a9600] |

## 3. The `Skel` part table = the attachment points [M 0x1c9910 / 0x1c9980 / 0x1c9a80]

`FUN_001c9910` reads the part count (the record's `count` field); `FUN_001c9980(obj, name)`
returns the 1-based part index: it walks 0x20-byte entries and `strcmp`s against
`payload + u16_at(entry + 0x1c)` (the string pool sits after the entries). Entry +0x0c is a
f32 (1.0 in every entry of all five cars). The vehicle setup resolves the attachment points by
name through this lookup [M FUN_001d2648 0x1d2648, the name-pointer tables at EE
0x334070 (`leftfrontwheel`,`rightfrontwheel`,`leftrearwheel`,`rightrearwheel`),
0x334080 (`leftfronthub`,`rightfronthub`,`leftrearhub`,`rightrearhub`),
0x334090 (`leftfin`,`rightfin`,`leftrudder`,`rightrudder` — boats)], storing the part indices
into the vehicle object (wheels at obj+0x248, hubs at obj+0x258).

Measured part lists [D]: taxi `hood, leftfrontwheel, leftrearwheel, rightfrontwheel,
rightrearwheel, trunk`; bus_paris/bus_tokyo the four wheels only; junglesnow adds
`leftrearhub/rightrearhub/lefttailpipe/righttailpipe/rearlicense`; jungle_truck adds hubs +
`leftsidemirror` etc. Every car carries the four wheel names (golden-tested).

## 4. The embedded module: ELFd + ELFr glue [M 0x1a9600; D measured]

`ELFd` (taxi 148,640 B) + `ELFr` (13,904 B) glue into one 32-bit LE MIPS **ET_REL** image —
the section table sits inside the glued payload (taxi `e_shoff` 0x27a00; ELFd+ELFr =
0x27af0). Sections: `.data` (WRITE+ALLOC — the whole model), `.shstrtab`, `.strtab`,
`.symtab`, `.rel.data` (1,174 relocations, all `R_MIPS_32` against `.data`; the addend is the
word in place). The loader [M FUN_001a9600 0x1a9600 → FUN_0029f7c8/0x29f820 load+relocate,
then FUN_002daa18 0x2daa18 consumes `Sect` (its payload+0x7c = the module's `.data` anchor)].

Symbol vocabulary (taxi) [D]:
- `__Model:::taxi::{as  NNNN}::Model[L].00` — the **model descriptors**, one per
  variant × LOD (values are .data offsets; taxi: 13 — `as#3/4/5` have only Model[0]);
- `__EAGL::TAR:::{T0@hash}TEX0::$$00` — texture references (resolved against the `.ssh`
  sheet by content hash; `{Tb@…}TEX0::spec`, `{T0@2834dd21}TEX0::dgla`, `{Td@…}TEX0::drkg`
  for specular/damage variants);
- `MatCarTexturePacked`, `MatCarGlassPacked`, `MatCarTextureDullPacked`,
  `MatCarTextureSwitchPacked`, `MatCarDash` + `…0__EAGLMicroCode` — materials and their
  microcode; `__EAGL::GeoPrimState:::STATE::…` render-state strings
  (`primtype=trilist/tristrip`, `modify=envmap/specmap/bumpmap/glaregen`, `texalpha`,
  `cull`, `alphatest`);
- `__LASTCHANGELIST:::4.06.01 Tue_June_25th_2002` — the export tool version;
- `SwitchData[8]`, `CarLighting*Props`, `ReflectionMap`/`SpecularMap`, `BondCarShadow`.

## 5. The model descriptor and the draw batches [D, measured taxi Model[0] @ .data 0x23698]

```text
+0x00 ptr  texture-set name ("$$00")           +0x9c/+0xa0 ptr  same string again
+0x04.. f32 4x4 (identity in taxi), a second matrix, ASCII "metalbin in \n" at +0x70
+0xac ptr  the model-name string table ("taxi::{as  000}::Model[0].00\0…")
+0xb4/+0xbc ptr  texture list { u32 0, u32 count, ptr × count } → TAR texture objects
+0xc4 ptr  { 0, 0x1fff5 }                      +0xc8 u32 1
+0xcc ptr  the batch table
```

Batch table: a sequence of groups `[0xa0000000][0x00000000][count]` then `count/2` ×
`[0xa000ffff][ptr]` (measured on taxi's 13 groups: 0x3a→29, 0xe→7, 0xa→5, 4→2, …; the `ptr`s
are .data offsets of the draw buffers). The `0xa000ffff` words read like DMA tags but the
group arithmetic above is what the data shows [S: their DMA semantics are unverified].

## 6. The draw buffers: VIF streams [M packet arithmetic = `nf_formats::vif`; D measured]

Each buffer starts with a 0x18-byte head (`[0x3f0][0][0x600000db][0][0][0]` for the first
taxi buffer, same shape across buffers) and then a VIF command list that does **not** fill
the whole `[ptr, next ptr)` range (tool strings and the next DMA head follow it). The stream:
`V4_32 NUM=1 addr=0` (a 4-word header quad), three `V4_8 NUM=1` quads writing VU1 addresses
0x1f/0x20/0x21, then `V4_32 NUM=n addr=0x22` — n 16-byte **(x,y,z,1.0) f32 quads = the
vertices**. The reader is the EAGL GeoPrim library in DRIVING.ELF (the 0x262000–0x2a7000
range, strings `EAGL::…` [D]) — its per-packet consumer is not pinned yet, so:

**Open (next pass):** the per-triangle index list (tri-strip/tri-list connectivity), the
exact role of the non-1.0 V4_32 quads (normals/UVs — `texalpha`/`envmap` states suggest
normals are present), and the DMA head words. The census therefore reports, per buffer:
V4_32 quads total, V4_32 quads with w=1.0 (the positions), V4_16/V4_8/V3_32 quads; the
golden pins both totals per model.

## 7. Measured numbers [D, golden/carmodel_golden.tsv]

| car | parts | models | draw buffers | vertices Σ (w=1.0 quads) | V4_32 quads Σ |
|---|---|---|---|---|---|
| taxi | hood + 4 wheels + trunk | 13 | 75 | 1087 | 3451 |
| bus_paris | 4 wheels | 10 | 63 | 1134 | 3373 |
| bus_tokyo | 4 wheels | 9 | 77 | 5 (V4_16/other streams carry the bulk) | 700 |
| junglesnow | 4 wheels + 2 hubs + tailpipes + rearlicense | 28 | 139 | 3204 | 9108 |
| jungle_truck | 4 wheels + 2 hubs + mirrors + tailpipes + license | 34 | 255 | 7070 | 19275 |

The variants are the game's damage/state sets: taxi variant 0.0 = 1087 verts → variant 7.1 =
0 (fully damaged LOD) [D].

## 8. Live checks (open)

- **C-1 — the wheel parts resolve**: breakpoint `FUN_001c9980` 0x1c9980 during a Paris drive
  with the player taxi; expect the name argument to point at `leftfrontwheel`/… and the
  return to be 1-based indices 1..6 [M expectation from §3].
- **C-2 — the model article open**: breakpoint `FUN_001c7488` 0x1c7488; expect a2 =
  `data\car\model\` and a3 = the rendername (e.g. `taxi.crp`) [M expectation].
- **C-3 — the loader relocation**: dump the loaded car image after `FUN_001a9600`
  0x1a9600: the `in#0` entry +0x1c words (in-file `sa#N` markers) should now be runtime
  pointers into the relocated module's `.data` [M expectation].
- **C-4 — GeoPrim consumer**: the EAGL function that reads the batch table (the `0xa0000000`
  group headers at .data+0x231c0 for taxi) — its address pins the last open format piece (§6).

## 9. The car draw path and the decoded parts (`crates/nf-formats/src/carmesh.rs`) [M + D]

Decoder: `carmesh::car_mesh(crp_bytes)` → `CarMesh { name, sheet, skel, parts, draw_lists }`;
each `CarPart` is one `in#0` item with its matrix, flags, attachment kind, `as` set, LOD list
and LOD-0 strips (positions, UVs, normals, texture) in the part frame. Tests:
`crates/nf-formats/tests/carmesh_disc.rs` (all 58 car files of the 8 driving archives) and the
unit tests in `carmesh.rs`.

**Draw path** (DRIVING.ELF):

- `FUN_001d35d8` 0x1d35d8 (vehicle draw, vtable 0x3885d8): all `in#0` items through
  `FUN_001ad6b8` while no draw list is selected (`car + 0x348 == 0`, as `FUN_001d2338` leaves
  it), else the items a `vd  #N` record lists (`FUN_001ad758`: `items + u16[k] × 0x40`;
  selected by `FUN_001d2618` → `FUN_001c8ba8`, record count = number of indices). So `vd` holds
  **item** indices (C-3's "`sa#N`" marker is the `as` short tag: the item's `+0x1c` word is
  `'as' << 16 | N`).
- `FUN_001ad498` per item: skipped when `+0x0c & 1`; matrix from the table at 0x333a60
  indexed by `+0x0d` (`FUN_001beee0`): 0xff = own matrix (`FUN_001bf140`), 0xfe = own matrix ×
  the run-time matrix of `Skel` part `+0x0e − 1` (`FUN_001bf600` → `FUN_001bef68`), 0xef =
  another run-time matrix (`FUN_001bf598`), < 100 = animated (`FUN_001a30c0`); then × the car
  matrix (`FUN_002cbb88`). Mirrored items (`+0x0c & 0x80`) store a negated x axis [D].
- The `as` payload is the LOD set read by `FUN_001ad2e0`: `+0x04` effect ref, `+0x08`
  animation ref, `+0x10`/`+0x14` f32 (`+0x14` = the y half-extent of the LOD-0 geometry on all
  686 sets with geometry, exactly [D]; `+0x10` not identified), `+0x18` u16 own index,
  `+0x1c` LOD count, `+0x1d` pass (2 = now, else queued by `FUN_001ab050`), `+0x1e`
  render-state hook (table 0x3337a0), `+0x20` `{f32 distance, sr ref}` per LOD.
- The model draw is the track's (`FUN_002843a0`, docs/formats/geoprim.md §4b): one visible
  group per car model (970 models) [D].

**Batch layouts** [D]: A = `V3-32` positions + `V2-16` UV (u16 − 0x8000) / 1024 + `V3-8`
normal byte / 127.5 − 1 [M §10] + two `V4-16` arrays (`DmgVec`/`DmgNrm`, §10); B = five `V4-32` arrays
(position, normal, UV, two undecoded); positions-only. 249,532 of 249,596 normals have unit
length ± 5 %; there is no colour upload in any car chain. Strip list: 3 bytes per vertex, 2 for
positions-only batches, wrapping past 255.

**Old reading** (`crp_mesh::TrackModule::extract_car_mesh`): treats `vd#1` as `as` indices,
places `as#k` with item k's matrix, drops mirrored items, decodes the batch behind
`*(entry − 0x18)`, reads UVs at /1024 but normals as colours. On the vanquish 4,221 of its
10,375 vertices lie on the car the new reading places.

**Open**: (the `vd` selection and the run-time `Skel` / 0xef matrices are §11); the texture of 85 strips whose model
sets none before them (GS state carried from an earlier draw, not traced); the ultralights'
`blck` texture, which no sheet of their archives holds; the
car microcode is read in §10: the /1024 UV rule, the normal rule and the strip-list unit 3 are
[M] there; unit 2 of positions-only batches is data-proven only.

## 10. Car shading: materials, CarRender, the lights and the microcode [M + D]

Code: `carmesh::CarMaterial` / `CarStrip::{material, lighting}`, `crp_mesh::TrackModule::
external_refs`, `rendertun::{CarRender, CarLightClass, light_direction, LightEnv::car_light}`.
Tests: `crates/nf-formats/tests/carshade_disc.rs` and the unit tests in `rendertun.rs` /
`carmesh.rs`. [M] here also covers the VU1 microcode, disassembled with `tools/vu1dis.py`.

### 10.1 Where the car microcode is [D]

`MISC.VIV` → `data\render\bondrm.o` (367,111 B, plain) is a MIPS ET_REL object with one `.data`
section and no undefined symbols. It defines every name the car modules import: the render methods
`MatCarTexture[Packed]`, `MatCarTextureDull[Packed]`, `MatCarTextureSwitchPacked`,
`MatCarGouraud[Packed]`, `MatCarGouraudDull[Packed]`, `MatCarGlass[Packed]`, `MatCarChrome`,
`MatCarDash`, `BondCarShadow`, `MatCarPerCar`. Each one has a `<name>0__EAGLMicroCode` block.
- The render method object: `+0x00` is its render-list template, `+0x10` its parameter table of
  `(name pointer, quadword count)` pairs. The template's unpack addresses give the VU1 layout of
  `MatCarTexturePacked`: `Parameters` 0, `MyMatrix` 1-4, `WorldMatrix` 5-8,
  `LightingProperties` 9, `Texture` 0x0a, `SpecularMap` 0x11, `ReflectionMap2` 0x18,
  `SplitReg1-3` 0x1f-0x21, `Coordinates` 0x22, `STQ` 0x4e, `Normals` 0x7a, `DmgVec` 0xa6,
  `DmgNrm` 0xd2, `ColourSlot` 0xfe, `ColourSet` 0xff, `EnvMapState` 0x107, `State` 0x10f. All of
  these are relative to the double-buffer base `vi14` (XTOP), so the two `V4-16` arrays of
  layout A are `DmgVec` / `DmgNrm`.
- The microcode block: `+0x00` is the instruction count (MatCarTexturePacked 638) and `+0x08` is
  the cache slot (−1 in the file). `+0x10` points at DMA chunks (`cnt` + `MPG`, then a `ref` tag to
  256 instructions each) [M `FUN_002a3290` case 1: it uploads them, adding `0x100 × k` to the MPG
  address]. The code itself sits just before the chunk table.
- `MatCarChrome`'s microcode is byte-identical to `MatCarTexturePacked`'s. Chrome differs only in
  its GS states (`modify=chromemap`) and the props it uploads.

### 10.2 The material of each strip [D, all 58 car files]

The material comes from the group entry's GeoPrim header (entry `+0` → header; header `+0` is the
render list `FUN_002a3290` walks [M 0x2843a0]). The header's `+8` word is relocated against the
render method symbol. The relocation's `r_offset` is `.data`-relative
(`TrackModule::external_refs`). An independent check: all 1,843 op-1 microcode loads of the 7,263
entries load the `<header method>0__EAGLMicroCode`.

The lighting class comes from the render list's op 5 (copy `n` doublewords from the `+8` pointer,
with `UNPACK V4-32` to VU1 9 = `LightingProperties`), relocated against
`__COORD4:::GAME::CarLighting<X>Props`. `FUN_001a8930` resolves those names to `FUN_001c1778(k,
current vehicle name)`, the 16-byte slot `k` of that vehicle's CarRender block:

| k | props symbol | tuning section |
|---|---|---|
| 0 | `CarLightingGlassProps` | `[Glass]` |
| 1 | `CarLightingSpecularProps` | `[SpecBump]` (no car module uses it) |
| 2 | `CarLightingGlossyProps` | `[Paint]` |
| 3 | `CarLightingDullProps` | `[Dull]` |
| 4 | `CarLightingChromeProps` | `[Chrome]` |
| 5 | `CarLightingDashProps` | `[Dash]` |

Most entries after a model's first entry of a material have a render list that is a bare op 4.
The strip then keeps the class the model last uploaded. That VU1 memory carries over is inferred,
as it is for the texture.

Strips by material (LOD 0 of the placed items, once per set; total 36,397) [D]:

| material | class | strips |
|---|---|---|
| `MatCarTextureDullPacked` | Dull | 16,431 |
| `MatCarTexturePacked` | Paint | 10,708 |
| `MatCarDash` | Dash | 5,653 |
| `MatCarChrome` | Chrome | 2,300 |
| `MatCarTextureSwitchPacked` | Paint | 972 |
| `MatCarGlassPacked` | Glass | 198 |
| `MatCarGouraudDullPacked` | Dull | 135 |

Each material carries exactly one class. No `BondCarShadow` strip and no positions-only strip is
among them.

### 10.3 The CarRender table [M `FUN_001c0930` 0x1c0930 + D]

`Render\CarRender\default.tun` is the same file in all 8 archives [D]. It has 52 instances, which
are the vehicle table, `Characters` and `Dynamic Objects` (`default` appears twice). Each instance
has a 0x80-byte block in `RReflection` private data `+0xa0`:
- `+0x00 + 16k`: class k as (Ambient, Diffuse, Specular, Reflect). This is the COORD4 that op 5
  uploads, so the microcode reads `.x` = Ambient, `.y` = Diffuse, `.z` = Specular, `.w` = Reflect.
  The constructor default is (0.5, 0.5, 1.0, 0.8).
- `[Dull]` and `[SpecBump]` register only Diffuse/Ambient.
- `+0x60` Fresnel, `+0x64` ReflectionWarp, `+0x68` SpecularSpot. `+0x6c` is written per car by
  `FUN_001c1760` (`0.3 × shade + 0.7`, `FUN_001adea8`).
- `+0x60..+0x6f` is `GAME::ReflectionInfo2` (`FUN_001c1828` per car). The car microcode never
  reads its `.w` [M grep of the 12 `MatCar*` programs].

The globals are `Sky bright` (u8 +0x00), `Sky blend` (u8 +0x02), `Desat` (+0x04, packed like
FogColour), `Desat ?` (+0x08), `Sky ?` (+0x0c), `Refl HEIGHT` (+0x10) and `Refl ANGLE` (+0x14).
Their consumers are the reflection-map pass (§10.6.3).
`EAGL::EnvMapState2C` / `2G` (`FUN_001c1950`) hold GS `ALPHA` values: FIX = Paint / Glass
Reflect × 255; mode `0x64` = (Cs − Cd)·FIX + Cd, or `0x68` = Cs·FIX + Cd when `DAT_00333e74` is set.
No car file of the 8 archives references them [D, carshade_disc], so they do not set the car
passes' blend.

### 10.4 The per-car constants (`MatCarPerCar`, absolute VU1 addresses) [M]

The per-car parameter object (`FUN_001ada20` / `FUN_001adea8`, `DAT_003337d0`) fills the slots of
the `MatCarPerCar` method. Its bondrm.o template unpacks them to these absolute addresses:
- `LightSources` → 0x49..0x4f (7 qw): `*(lm + 0x7c)`, the lighting environment that
  `FUN_001aec20(lm, 0)` copies from `RLightManager + 0x80` (the `World` instance).
- `SpecularMat` → 0x50..0x53 (lm + 0x20).
- `ReflectionInfo1` → 0x54 (`DAT_00355ba0`).
- `ReflectionInfo2` → 0x55.
- `DamageZones` → 0x56.

The LightSources layout (block offsets of `FUN_001ae630`'s registrations):

| qw | content |
|---|---|
| 0-2 | rows x, y, z of the light matrix: column 0 = Light 1 dir, 1 = Light 2 dir, 2 = (0, 1, 0), 3 = `normalize(L1 − c)` |
| 3 | `Light 1` RGBA × shade |
| 4 | `Light 2` RGBA |
| 5 | `Ambient Ground` RGBA × shade |
| 6 | `Ambient Sky` RGBA × shade |

- Column 3 is written by `FUN_001aefb8`. `c` is the camera's forward axis (§10.6.1).
- **Angle convention** [M `FUN_001b0038`, `FUN_002d11c8`, `FUN_002d1920`]:
  dir = `(cos X · sin Y, −sin X, cos X · cos Y)`, with X = `Angle X` and Y = `Angle Y` in degrees.
  This is (0, 0, 1) rotated about +x by X, then about +y by Y, as a row vector. The tuner limits X
  to −90..0, so dir points up towards the light. Column 2 (dual hemisphere) is overwritten with
  (0, 1, 0) for World and Character (`FUN_001b0158`). Every track sets `DualHemiAngleX` = −90,
  which gives the same vector anyway [D].
- **shade** [M `FUN_001adea8` → `FUN_001af2d8`]: the car's `+0x1a` byte / 255 × `FUN_001af200`
  (= 1.0). Light 1, Ground and Sky RGB are multiplied by it before the copy. For the sky the
  factor is the value left in the first float argument register, which the call passes on
  unchanged (inferred). Light 2 is not multiplied.
- **The shade byte's writer** [M `FUN_001adea8`, `FUN_001c8d70`]: when the drawn object has a
  world object at `+0x24` whose `+0x30` and `+0x34` are set, `FUN_001adea8` itself rewrites `+0x1a`
  = clamp(`FUN_001adce8(position, region)`, 0, 1) × 255 before using it. `FUN_001c8d70` writes it
  the same way with the sample × 1.4. `FUN_001adce8` samples a 4-bit grid bilinearly
  (`FUN_001ad808`) at the object's x/z, through the region table at `DAT_00333780`, and multiplies
  by `lm + 0x2dc`. With no grid loaded (`DAT_003d1b28 == 0`) it returns 0.7 × `lm + 0x2dc`. Where
  the grids come from is not traced.

### 10.5 The vertex formula [M, bondrm.o `MatCar*` microcode]

The programs set up with an `{I}`-pair convention: the immediate loaded by a pair is read by the
*next* pair. Two readings support this. The set-up GIF tag then reads FLG = PACKED, NREG = 3,
REGS = ST/RGBAQ/XYZ2. The normal unpack also comes out right: `ITOF0`, × 0x3c008080 (1/127.5), −1.
Read the other way, it would scale normals by 1/1024. This convention is inferred, not taken
from a manual.

Per batch, in order:

0. **Set-up.**
   - Colour C = (127, 127, 127, 127) per vertex (`ColourSlot` buffer at vi14+0x11b).
   - UV = raw/1024 − 32, normal = byte/127.5 − 1.
   - Damage pass: position += DmgVec · DamageZones[zone].x, normal += DmgNrm · zone.x, and
     C.rgb × (1 − DmgVec.w · zone.y). C.w × (1 − … zone.z) goes into DmgVec.w (call it C_w).
     On an undamaged car this leaves 127. The details are §10.8.
1. **Base pass** (every `MatCar*`; textured unless Gouraud).
   - n_w = the model normal rotated by the WorldMatrix with each row normalised (`ERLENG`).
   - L = D·L1rgb·max(0, n_w·dir1) + L2rgb·max(0, n_w·dir2) + Ground·max(0, n_w·(0,1,0)) + A·Sky,
     with A, D from `LightingProperties`. This is `LightEnv::car_light`.
   - RGB = min(255, C·L), stored as integers. Then a positional-light term is added:
     Σ_j col_j · max(0, 255 − k_j·|P_j − p_w|²) for 4 lights. The positions, k and colours sit at
     VU1 0x40..0x47 (`GAME::PositionalLights` content, not traced). The result is clamped to 255.
   - Output per vertex: ST, RGBAQ, XYZ2. The strip list values are 3 × vertex index; they set
     the ADC bit on the XYZ2 of the two vertices that restart a strip.
2. **Specular pass**: TexturePacked/Texture, Chrome, Glass[Packed], Gouraud[Packed] and Dash.
   - ST = 0.5 + SpecularSpot · ((n · W · SpecularMat).x, (… ).y), Q = 1.
   - RGB = Specular · C_w.
   - Texture: `SpecularMap` (`GAME::SpecularMap` = `RReflection + 0xd0` = `common.ssh` `carS`,
     §10.6.2). State: `EnvMapState`.
   - SpecularMat columns are b, c, a with a = normalize(L1 − f), b = normalize((0, 1, 0) × a),
     c = normalize(a × b) (§10.6.1).
3. **Reflection pass**: the specular materials except Dash.
   - v = ReflectionWarp · ((n + Fresnel · p) · W), where p = the (damaged) model position and W
     is the WorldMatrix rotation rows, unnormalised.
   - ST = (0.5 + v.x, 0.5 + v.z), Q = 1.
   - RGB = min(Reflect · C_w, 100000 · max(0, v.y)): faces whose vector points down get no
     reflection.
   - Texture: `ReflectionMap2` (`GAME::ReflectionMap` = `RReflection + 0xbc`, a 256 × 256 texture
     that `FUN_00291af0` binds to the reflection view, drawn as in §10.6.3).

The Dull family (TextureDull[Packed], GouraudDull[Packed], TextureSwitchPacked) runs only the base
pass. This matches the tuning: `[Dull]` has no Specular/Reflect. TextureSwitchPacked uploads
`[Paint]` props, of which only A and D are used.

For the vanquish's paint (A 0.79, D 0.42) under paris's World light with shade 1, an upward normal
gives L ≈ (1.17, 1.18, 1.20): 0.30 from Light 1, 0.31 from Light 2 and 0.57 from the sky. The base
vertex colour is then 149..153, which is above the GS's 128 = ×1.0 if the texture function is
modulate.

### 10.6 The passes on the GS side [M + D]

Code: `carmesh::{GeoPrimState, GsBlend, CarMaterial::pass_slots, CarStrip::{states, slot_textures,
passes, pass_vertices}, specular_vertex, reflection_vertex, entry_uploads}`,
`rendertun::{specular_basis, CarRender::{reflection_steps, reflection_camera, sky_level}}`. Tests:
the unit tests in `carmesh.rs` / `rendertun.rs` and `every_car_pass_finds_its_traced_state_and_textures`
in `carshade_disc.rs`.

#### 10.6.1 The SpecularMat basis [M `FUN_001aefb8`, disassembled]

- a = normalize(L1 − f), with L1 the Light 1 direction (LightSources column 0) and f the vector
  the caller passes. a is also stored as LightSources column 3.
- b = normalize((0, 1, 0) × a) and c = normalize(a × b). Each cross product is a
  `VOPMULA.xyz ACC, vf1, vf2` + `VOPMSUB.xyz vf3, vf2, vf1` pair, which gives vf1 × vf2; vf1 is
  (0, 1, 0) then a.
- The four rows stored at `+0x20` (= `SpecularMat`) are (b_i, c_i, a_i, 0) for i = x, y, z, then
  (0, 0, 0, 1) (`PEXTLW` / `PEXTUW` / `PCPYLD` / `PCPYUD`). A row-vector normal therefore maps to
  (n·b, n·c, n·a).
- f is `camera + 0x20`. The camera is `**(DAT_00333760 + 4)`: the current view's `+0` object, a
  0xb0-byte camera with its FOV at `+0xa4` (`FUN_001d40f0`, `FUN_001d4570`). When no view is
  current it is the HUD manager's camera, `*DAT_00333ed8` (`FUN_001aec20`, `FUN_001adc68`).
  `FUN_001d5700` writes `camera + 0x20` = normalize(look point − eye), so f points from the eye
  into the scene. a is then the half vector between the light and the eye direction, and a
  normal along a samples the map centre (0.5, 0.5).

The microcode forms `M = W · SpecularMat · SpecularSpot` (W = the WorldMatrix rows, unnormalised)
and computes ST = 0.5 + (n · M).xy from the model normal (0x1f4-0x229).

#### 10.6.2 What each pass kicks [M bondrm.o + D]

Each pass sends one GeoPrimState packet, one texture packet and the vertex buffer: an
`IADDIU vi02, vi14, N` + `XGKICK vi02` pair for each. The render lists fill the slots with op 6
(textures) and op 7 (states). The VU1 address of each upload is the `UNPACK` that ends the op's
VIF words.

| material | base | specular | reflection |
|---|---|---|---|
| TexturePacked, Chrome | 0x10f + 0x0a | 0x107 + 0x11 | 0x107 + 0x18 |
| Texture | 0xfe + 0x0a | 0x106 + 0x11 | 0x106 + 0x18 |
| Dash | 0xfe + 0x0a | 0x106 + 0x11 | none |
| GlassPacked | 0xfd + 0x0a (after a 0xfd + 0x23 damage draw) | 0x105 + 0x11 | 0x105 + 0x18 |
| TextureDull / DullPacked / SwitchPacked | 0xf0 / 0xf9 / 0xfb + 0x0a | none | none |
| GouraudDull / DullPacked | 0x11e / 0xc6 | none | none |

- Slot 0x11 always receives `__EAGL::TAR:::GAME::SpecularMap` and 0x18
  `__EAGL::TAR:::GAME::ReflectionMap` (1,906 uploads each) [D].
- `GAME::SpecularMap` is `RReflection + 0xd0`. The constructor fills it with
  `FUN_001cf340(FUN_001d0420(0), 0x53726163)`, the entry `carS` of the sheet registered as key 0.
  `FUN_001c4008` registers `data\render\` + `common.ssh` as key 0. `carS` is a 32 × 32 entry in
  every archive's common.ssh [D].
- `GAME::ReflectionMap` is `RReflection + 0xbc`, the 256 × 256 render target (§10.6.3).
- GlassPacked's lists also upload a state to 0xf5, which its program never kicks. Its base draw
  kicks the `name=glassdmgmap` state at 0xfd. Both strip/list paths of the program reach the
  0xfd + 0x23 kick (0x177-0x17c), so it is unconditional; its vertex alpha is the zone's glass
  word (§10.8).

**The states** (`FUN_001ce158` builds the 0x80-byte GS packet: `+0x10` PRIM, `+0x30` TEST_1,
`+0x40` ALPHA_1, then FRAME_1 / ZBUF_1 / FBA_1).

The start values are:
- PRIM = triangle | IIP | TME | FGE | ABE.
- TEST = ZTE, ZTST GEQUAL, ATE, ATST GREATER, AREF 0xa0, AFAIL RGB_ONLY.
- ALPHA = 0x44, i.e. (Cs − Cd)·As + Cd.

The `key=value` handlers (table 0x333f58) run in order. Those that return non-zero run once more;
only `modify=specmap` does. `carmesh::GeoPrimState::parse` lists every handler. The ones that
decide the passes:

- **`name=envmap`** (`FUN_001cda30`): the specular and reflection state of paint, chrome and glass
  (every such strip, [D]).
  - ALPHA = 0x48: **Cs·As + Cd**, additive and scaled by the source alpha.
  - PRIM = IIP | TME | ABE (fog off).
  - ATE on, but AFAIL = RGB_ONLY keeps the colour of failing pixels.
- **`name=specmap`** (`FUN_001cdad0`): the dash specular state (every dash strip).
  - ALPHA = 0x58: **Cs·Ad + Cd**, scaled by the frame buffer's alpha.
  - The same PRIM; ATE off.
- **The dash base** (`modify=…glaregen;modify=specmap;texalpha=…`): `glaregen` and `specmap`
  clear ABE and ATE. The later `texalpha` acts only while ATE is set, so it does nothing. The base
  draw is unblended, and the frame-buffer alpha it leaves is what the dash specular scales by.
- **The paint and chrome base** (`modify=envmap|chromemap;alphatest=off;texalpha=…`):
  `alphatest=off` clears ATE first, so `texalpha` is skipped and ABE stays on with ALPHA 0x44.
- `texalpha`'s own `ALPHA = 0x44` write is guarded by the caller's `$s1`, which is the string
  cursor of `FUN_001ce3b0` and never null. So that write never runs.

**Vertex alpha**: 127 in every pass. The set-up loop fills `ColourSlot.w` with FTOI0(127.0), the
base pass keeps it (`MOVEw`), and the overlay passes store `.xyz` only.

**Colours** (FTOI0, i.e. truncated):
- specular RGB = Specular · C_w;
- reflection RGB = min(Reflect · C_w, 100000 · max(0, v.y));
- C_w = 127 on an undamaged car.

#### 10.6.3 The reflection map [M `FUN_001c1ec0`, `FUN_001c1b18`, `FUN_001c1560`, `FUN_001c1358`]

Each frame (`FUN_001c5eb0` → `FUN_001c1ec0`) the reflection view clears the 256 × 256 map, then
draws the steps below. Each quad is full-map (`FUN_0019f960(0, 0, 256, 256, colour, texture)`)
with ABE on. s = (int)(Sky bright × 0.94) & 0xff.

1. If `Sky ?`: a quad textured `cref` with colour (s, s, s, 0x40), ALPHA 0x44.
2. The scene, drawn from the reflection camera (§10.6.4): the track items that rise over the
   car, then up to 4 vehicles. No sky dome is drawn; the sky is the `cref` quads.
3. If `Sky ?` and `Sky blend` ≠ 0: `cref` again with colour (s, s, s, Sky blend), ALPHA 0x44.
4. If `Desat ?`: a quad textured `rmsk` (common.ssh, 64 × 64) whose colour is the packed `Desat`
   word, ALPHA 0x62 = Cd − Cs: a subtraction.

When `DAT_003349e0 + 0x30` is 1, steps 2-4 are replaced by a flat (0x80, 0x80, 0x80, 0x80) quad.

Step 1 is drawn before that test, so it stays.

**`cref`** [M `FUN_001c1358` + D]: it comes from a sheet registered as key 4, else from
`data\render\ext.ssh` (key 5). Key 4 holds the sheets the loaded modules name in their `sn  `
records (`FUN_001a9600`, `"%s%s"` of the module's folder and the name). `FUN_001c4840` builds the
track world (`FUN_0021eef0`) before it looks `cref` up, so the track module is loaded by then.
- Each track module's only `sn` record is `#0`, naming its own sheet (`paris_mis01.ssh`, …) [D].
- Of every `.ssh` in the 8 archives, `cref` (128 × 128) sits only in `ext.ssh` and in the track
  sheets of paris, uw, snow1a and snow2a (both archives) [D]. So those tracks reflect their own
  `cref`, and the three jungle tracks use `ext.ssh`'s.
- `wref` (`RReflection + 0xc8`) is looked up the same way (ext.ssh has one). Only the getter
  `FUN_001c1ac8` reads it; its users are not traced.

Code: `rendertun::{CarRender::reflection_view, ReflectionView, ReflectionProjection,
ReflectionItemQuery, item_bounds}`. Tests: the reflection unit tests in `rendertun.rs` and
`every_tracks_reflection_map_inputs_decode` in `carshade_disc.rs`.

#### 10.6.4 The reflection view [M `FUN_001c5eb0`, `FUN_001c1ec0`, `FUN_001c1b18`, `FUN_00295910`, `FUN_0021fa58`, `FUN_001c2720`]

There is one reflection map per frame, and every car samples it.

**The camera.**
- p = the player car's position. `FUN_001c5eb0` passes `FUN_00201af0(0x339190)`: the first car
  of the sim's `+0xdcc` list (the car `FUN_001fee68` places at the track start), its body
  (`FUN_001ff088` or `FUN_001ff0b0`) `+ 0x10`.
- f = row 2 (`+0x20`) of the camera `**DAT_00333ed8` points at (§10.6.1), read at the top of
  `FUN_001c1ec0`.
- eye = p + (0, Refl HEIGHT, 0).
- forward F = normalize(0.8 · f + (0, 1, 0)).
- right R = normalize((0, 0, −1) × F), up U = normalize(F × R). Both are `VOPMULA` + `VOPMSUB`
  pairs (0x1c1c14, 0x1c1c50), fs × ft with fs = (0, 0, −1), then F.
- `FUN_001a4520` stores the rows (R, U, F, eye). `FUN_001a44f0` negates row 2 and `FUN_001a4468`
  inverts, so view space is x = (q − eye)·R, y = (q − eye)·U, z = −(q − eye)·F. The main camera
  goes through the same two calls (`FUN_001d4878`).
- R × U = +F, while a plain camera has R = F × U. So the map is a **mirror image** of what a
  camera looking along F would see.
- With F straight up, R = +x and U = −z, so the map's s grows with world x and t with world z.
  That matches the reflection pass's ST = (0.5 + v.x, 0.5 + v.z) (inferred).

**The projection** (`FUN_001c1560`: `FUN_00295910(Refl ANGLE, 1, 1, 2500, view)`, once).
- Clip = (k·x, −k·y, a·z + b, −z), with k = 1 / tan(Refl ANGLE / 2), a = −(far + near) / (far −
  near), b = far · (a + 1). The angle is the full angle, the same across and down (aspect 1).
- The view's `+0x190` scale divides x and y and multiplies the viewport, so it cancels.
- Nothing on the per-frame path calls `FUN_00295910` again. `FUN_001d4878` does, but only on the
  saved main camera when `FUN_001c2440` restores it.
- The `.tun` value is in place for the one call: `FUN_001426a8` reads each key from the loaded
  file as it is registered, inside `FUN_001c0930`, which `FUN_001c1560` calls first. The
  constructor defaults are ANGLE 120, HEIGHT 0, `Sky ?` 1, `Desat ?` 0, `Sky blend` 24,
  `Sky bright` 223, `Desat` 0x80787878. Every archive's file then sets bright 127 and `Desat`
  0x80919191 [D].
- Texel = 128 + 128 · clip / w (y down). This assumes the viewport maps −1..1 onto the 256 map;
  the GS step is not followed.

**Fog for the scene**: `DAT_0033376c + 4` is set to (24560, 0, 0, 4080 / 24560) and restored
after. Read as end 24560, start 0 (inferred from the `4080 / (w0 − w1)`), this is far past the
2500 far plane, so the reflection has almost no fog. `DAT_00339090 + 0x2024` is 0 during the scene
(meaning not traced).

**What the scene draws.**
1. Track items: `FUN_0021fa58(50.0, DAT_0033a5d4, list, p, (0, 1, 0, 1))`. The world
   (`FUN_0021eae8`) is the track module's `Map` → `in  ` items (0x40 bytes) with a cell tree.
   - The cells whose circle (size × 1.414) meets the 50-unit sphere round p (`FUN_0021f8d0`) give
     u16 item indices.
   - An item is kept when all of these hold:
     - its top rises over the car: `+0x34` + height > p.y + 10 (5 + `DAT_0033a5f0` = 5);
     - |`+0x30` − p| − 50 ≤ radius, with radius > 0 (`FUN_0021e9f0`; the direction's x and z are
       0, so its plane test only needs radius > 0);
     - |`+0x30` − p| < radius + 25 + 0.75 · height;
     - `+0x0c & 0x0d` == 0.
   - The `+0x3c` bounds word: scale s = 16 when bit 30 is set, else 0.25. radius = (w & 0x3ff) · s.
     height = ((w >> 10) & 0x3ff) · s when bit 31 is set, else 2 · radius.
   - On the disc no item has radius 0. The flags test drops 15 to 1,404 items per track (1,404 of
     jungleb's 1,412) [D, `every_tracks_reflection_map_inputs_decode`].
   - The kept items are sorted by address (table order) and drawn with `FUN_001ad618`, the
     `FUN_001beee0` item matrix and the `FUN_001ad2e0` LOD pick (§11).
   - Road and ground at the car's height fail the rise test, so the map holds what stands over the
     car: buildings, trees, poles, overhangs.
2. Vehicles: `FUN_001c2720` calls the vtable `+0x24` method of the up to 4 objects at
   `RReflection + 0x1f4`.
   - The base vehicle class (vtable 0x386a88) has `FUN_001caf08` there, the same `FUN_001ad6b8`
     part draw as the main view.
   - The 4 slots keep the smallest LOD measures. The vehicle draw `FUN_001c85f8` measures with
     `FUN_001ad0b8` and calls `FUN_001c2508` when the measure is below `RReflection + 0x5c`, the
     worst kept value. `FUN_001c2508` replaces the rank-3 slot and returns at once for an object
     already kept, without updating its measure. `FUN_001c25f8` removes a destroyed object and
     `FUN_001c27a8` re-ranks the slots.

The view returned by `CarRender::reflection_view(car_pos, main_forward)` holds eye, R, U, F, the
projection, the item query (`ReflectionItemQuery::keeps`), the vehicle count, the fog and the
steps of §10.6.3.

### 10.7 Unproven

- The texture function (TFX / TCC) of every pass. TEX0 comes from the texture packets
  (`GAME::SpecularMap`'s descriptor; the module descriptors), which are not decoded.
  - `FUN_001c0930` / `FUN_001c1358` set the low nibble of the texture objects' `+0x10` to 5 and
    clear bits 25-26; what those fields mean is not traced.
  - The draw rules above assume modulate, with 0x80 = 1.0.
- COLCLAMP, FRAME / ZBUF masks, and therefore whether the base pass writes the frame-buffer alpha
  that the dash specular reads.
- The reflection view (§10.6.4):
  - which value the map is cleared to (`FUN_00291ce8(target, 0)`, `FUN_00296548(view, 7)`, then
    `FUN_00291ce8(target, 0xff000000)`);
  - how the GS viewport maps the clip space onto the map (the texel formula is inferred);
  - the fog words' meaning, and `DAT_00339090 + 0x2024`;
  - whether the player's own car is among the 4 kept vehicles (its draw path into `FUN_001c85f8`
    is not followed), and what a subclass puts in the vtable `+0x24` slot;
  - when `FUN_001c27a8` runs;
  - the LOD measure `FUN_001ad618` uses for items while the reflection camera is current
    (`FUN_001ad128`);
  - `DAT_003349e0 + 0x30`, the flag that swaps the scene for a flat quad;
  - for a car on the `FUN_001ff0b0` (0x40-byte) body table, whether `+0x10` is its position.
- Whether `FUN_001b0158`'s light directions use the `.tun` values (the reading at registration,
  §10.6.4, settles it for the CarRender keys only).
- What uploads VU1 0x40..0x48 (the positional lights, and 0x48, which the clip/fog block uses).
- How the GS draws the glass-damage pass (alpha 127 against the `glassdmgmap` alpha test), and
  `MatCarTextureSwitchPacked`'s `SwitchData`. The `Glass` and `Gouraud[Packed]` slot layouts are not read (no car strip uses
  them).
- Which `default` row the name lookup lands on.
- Live check C-5: breakpoint `FUN_001c1778` during a paris drive. Expect k ∈ {0, 2, 3, 4, 5}, the
  vehicle name `vanquish`, and a return of `block + 16k` (§10.2).

### 10.8 The damage arrays and `DamageZones` [M bondrm.o + DRIVING.ELF; D all 58 car files]

Code: `carmesh::{CarDamageVertex, DamageZone, deformed, CarMaterial::{damage_slots, packed_setup,
glass_alpha}}`, `nf_game::driving::deform`. Tests: the unit tests in `carmesh.rs` / `deform.rs`
and `crates/nf-formats/tests/carmesh_damage_disc.rs`.

**The arrays** [M + D]. Every strip of every car file uploads two damage arrays, to exactly the
VU1 addresses its program reads them from (TexturePacked / Chrome / Texture / Dash 0xa6 / 0xd2,
TextureDull[Packed] 0x98 / 0xc4, SwitchPacked 0x99 / 0xc5, Gouraud[Packed] 0x73 / 0x9f,
GouraudDull[Packed] 0x65 / 0x91, Glass 0x9e / 0xc6, GlassPacked 0xa5 / 0xcd). They come as
unsigned `V4-16` (mode 0) for the programs with the packed set-up loop and as `V4-32` floats
(layout B, arrays 3 and 4) for Dash, the only non-packed material in use.
- Packed: `DmgVec` = raw/1024 + (−32, −32, −32, **−33**), `DmgNrm.xyz` = raw/1024 − 32. The w bias
  comes from `ADDi.w vf24, vf24, I` at 0x022 with I = −1 (loaded at 0x01f; both I-timing readings
  agree there). On the disc most `DmgVec.w` values lie in −1..−0.5.
- The zone index is the low 16 bits of the unconverted `DmgNrm` w word: `ILWR.w vi09, (vi08)`
  (the disassembler prints `BAD_LOWER(802943fe)`; lower-special 0x3e, index 0x0f), then
  `IADD vi09, vi05, vi09` with vi05 = 0x56. Every index on the disc is 0..15.

**The loop** (MatCarTexturePacked 0x036-0x05b; the 31 pairs from the first `ILWR` are
bit-identical in all twelve `MatCar*` programs; `BondCarShadow` has none):
- position += DmgVec.xyz · zone.x; normal += DmgNrm.xyz · zone.x (not renormalised);
- C.rgb = FTOI0(max(0, 127 · (1 − DmgVec.w · zone.y))), C_w = max(0, 127 · (1 − DmgVec.w · zone.z))
  (`MAXi` with I = 0 on all four, no upper clamp; C_w stays a float in DmgVec.w).
- Glass and GlassPacked then copy `DamageZones[zone].w` (`LQ.w`) into each vertex's alpha
  (`SQIw` to `ColourSlot`, GlassPacked 0x0f1-0x0ff).

**DamageZones** [M DRIVING.ELF]. `MatCarPerCar`'s parameter object (`FUN_001ada20`) holds
`(0x10, &DAT_00424620)`. Per drawn vehicle:
1. `FUN_001d35d8` (the vehicle draw) calls `FUN_001d3138`, which fills the render object's
   `EAGL::DamageZones` block (+0xe0; `FUN_001d2588` answers that name) from the owner's zone
   array (owner vtable +0x24 = `FUN_0018aa78`: `car + 0x2f0`, count 16): per zone
   `(a, b, 3·b, a > 0.75 ? 127 : 0)`, the last word an integer. No owner: all zero.
2. `FUN_001adea8` → `FUN_001d2b38` copies those 0x100 bytes to `DAT_00424620` (bit 28 of the
   object's +0x18, set by the constructor `FUN_001d2338`; the other branch `FUN_001d2a50` zeroes
   them).

`a` is the zone's first word (the `scuff` sum of `damage::inflict`), `b` the second. 0.75 is
`DAT_00332090`, which the ELF only reads. So a zone shows broken glass once `a` passes 0.75.

**Who writes the zones**: the damage virtual (`FUN_00189598`), `ESetVisualDamage`
(`FUN_00159660` → vtable +0x1a4 = `FUN_001876c8`: each zone set to a random point of
[a_lo, a_hi] × [b_lo, b_hi], two driving-RNG draws per zone, then the stage events and model
calls 10 / 9) and the repair (`FUN_00187578`, which also rebuilds the model:
`FUN_001c7940` clears the render object's +0x1c / +0x1e masks). Nothing decays them.

**Model calls 9 / 10** (`FUN_001caba8`): 10 fires effect event 0x12 for zones not already in
+0x1c and then stores the call's mask there; 9 adds new zones to +0x1e and runs the item swap
`FUN_001ca7a0` (not ported).

**Unproven**: what the 0x12 event and the item swap show; the GS result of the glass-damage
draw; how a darkening factor below 0 (DmgVec.w ≈ −1, which brightens) looks on screen. No live
capture backs any of this section.

## 11. Which list is drawn, and how the parts move (`crates/nf-formats/src/carpose.rs`) [M + D]

Code: `carpose::selected_list` / `CarMesh::list_for` (the list), `carpose::part_matrix` (one
item's local → car matrix), `WheelRig::new` / `WheelRig::update` (the wheel and hub matrices),
`dials`, `spin_step`, `lod_index`. Tests: the unit tests in `carpose.rs` and
`crates/nf-formats/tests/carpose_disc.rs` (all 63 car files of the nine `/DRIVING/` archives,
MISC included). Matrices are the game's row-vector 4x4s; `FUN_002cbb88(a, b, d)` is `d = a·b`
with `a` applied first [M 0x2cbb88].

### 11.1 The list [M]

- The vehicle update `FUN_00188200` (run by both `pvehicle` vtables 0x379ef0 and 0x37a9f8) calls
  the draw object's vtable slot +0x14 = `FUN_001d2618` every update that gets past its early
  returns [M 0x1883b4-0x18844c]. `FUN_001d2618` stores `FUN_001c8ba8(obj, N)` = the `vd  #N`
  payload at +0x348 (its count at +0x334); a missing record stores 0, and 0 draws every item
  (`FUN_001d35d8`).
- N: when the driver kind (+0x2c8) is below 2, **2** if the current camera mode has
  `interiorView` (mode record byte +7, `*(DAT_00333ed8+4)+0x120`), else **5** if AutoDrive is
  on (`DAT_0033a024+0x350`, written by `FUN_00206398` from the AutoDrive script event
  `10EAutoDrive`, `FUN_00148220`), else **1**. Every other driver kind: **1**.
- Driver kinds: the player's car is created with 0 [M FUN_00176658 :231]; script spawns with
  2 or 4 (4 is stored as 2) [M FUN_0012cb08 :202-206, FUN_001862c0 :170-174]; `FUN_0013fae0`
  with 3.
- An `AHelicopter` (`FUN_00193408`, vtable 0x37a4e8) has its own update, which never calls the
  slot, so it draws every item. A raw scan of the ELF for `lw v,0x3c(x)` followed by
  `lw r,0x14(v)` finds only the two call sites above; the slot function has no other pointer.
  So lists 3 and 4 are never selected.
- camera.ini [D]: `interiorView = 1` is set on `[VanquishDashboardCam0]` and `[BumperCam]`.
  A section whose name holds `Bumper` gets mode type 1 [M FUN_001a49e0 :641], and the player's
  car is not drawn at all while the mode, or the other mode of a running transition (+0x128
  while +0x12c > 0), has type 1 [M FUN_001d35d8 0x1d3650-0x1d369c].
- No list depends on damage or distance. Distance only picks each part's LOD (§11.3).

What that means on the disc [D, `carpose_disc.rs`]: `vd` numbers present: 1 on 7 files, 2 on 36,
3 and 4 on 16, 5 on 13. List 1 differs from "every item" on six files: the two vanquish copies
(MIS01, MIS4), the three supersnow copies (MIS3, MIS4, RACE) and the MIS01 helicopter (which
never selects). Supersnow: list 1 = items 1-13, 15-17 (14 is hidden) + 25-31; list 5
(AutoDrive) = 0, 18-24 + 25-31; list 2 (interior view) = 25-31 only. Vanquish: the 0xef items
3, 4, 5, 7, 20 are in list 2 (the dashboard view) and not in list 1; 21 is in list 1 only.
Which supersnow body looks like which form was not checked here.

### 11.2 The item matrix [M FUN_001ad498 → FUN_001beee0 + table 0x333a60]

`world = local · car`, `local` by the item's `+0x0d`:

| kind | items [D] | function | local |
|---|---|---|---|
| 0xff | 725 | `FUN_001bf140` | own |
| 0xfe | 333 | `FUN_001bf600` | `skel[+0x0e − 1] · own` (`FUN_001bef68`: past the count → matrix 0) |
| 0xfc | 16 | `FUN_001bf3d8` | `R(s16(v)/256 · frame °, +y) · own` |
| 0xfb | 11 | `FUN_001bf458` | `R(s16(v)/256 · frame °, +z) · own` |
| 0xef | 13 | `FUN_001bf598` | `R(dials[v] °, +z) · own` (`FUN_001befc8`: v ≥ 12 → dials[0]) |
| 0xf3 | 16 | `FUN_001bf338` → `FUN_001a30c0` | own (the keyframe branch needs `+0x0d` < 100) |
| 0xee | 1 | `FUN_001bfc20` | faces the camera; not ported |
| 0x64 | 0 | `FUN_001bf5f8` | not drawn |

- `v` = the item's `+0x0e`, or the u16 at `+2` of its run-time entry when `+0x0c & 0x10`
  [M FUN_001bf048]. No item on the disc sets 0x10 [D].
- `R(deg, axis)` = `FUN_002d11c8` (degrees; identity at 0). `frame` = `DAT_00334000 =
  (float)DAT_00339f54` [M FUN_001d1ef8], the sim tick counter (sim +0xdc4, +1 per tick unless
  sim +0xdb4 == 3 [M FUN_001fef20]). So a rotor with v = 4864 turns 19° per tick.
- The rotation is applied first, in the part's own frame, so a spun part keeps its origin. The
  own matrix of a mirrored item (`+0x0c & 0x80`) has x negated, which flips the sense of a +y
  turn: that is why the rig steers the right wheels by −s.
- The dials (`car+0x40`, 12 floats) are written by `FUN_001d34a8` (vtable +0x7c, the model
  update): slot 0 = the rig's front-left angle × −750 (a steering wheel); for the player's car
  only, slot 1 = `FUN_001d3440(speed · 3.6 · 1.5, 1)` (speed 0 when the velocity points
  backwards) and slot 2 = `FUN_001d3440(rpm, 2)` with rpm `FUN_0018aa88`. `FUN_001d3440` maps
  through the rows at 0x3340f0 [D]: row 1 `{0, 330, 120, −225}`, row 2 `{0, 7000, 121, −210}`
  (`base + (clamp(v) − min)/(max − min) · range`). No writer of slots 3-11 was found; they keep
  the constructor's 0.

### 11.3 LOD [M FUN_001ad2e0, FUN_001ad0b8]

Measure `m = max(0, |camera − car| − 5) · K`, K the f32 at +0x30 of the camera holder
(`*(DAT_00333760+4)`, not identified). `+0x0c & 8` → LOD 0; else the first LOD with `m ≤
distance`; none → nothing drawn; a LOD without a model draws nothing.

### 11.4 The wheel rig [M FUN_001d2648, FUN_001d2cb0]

Setup (`FUN_001d2648`, vtable +0x94): the four slots in the order of the name tables at
0x334070 / 0x334080 / 0x334090 (left front, right front, left rear, right rear) get the 1-based
`Skel` index of `<slot>wheel` and `<slot>hub` (`leftfin`, `rightfin`, `leftrudder`,
`rightrudder` instead of the hubs when the vehicle's +0x40 word is 1 and `IS_SUB` (+0xbc) is
set). Without a `Skel` record the car gets 4 matrices and wheels 1..4. The wheel's rest offset
(`car+0x238`) = `origin_y` of the first 0xfe item linked to that wheel, minus
`Base[+0x38] − (Base[+0x24] − Base[+0x14])`. The item scan does not restart per wheel: one
pointer walks on from the item after the previous match, `count` items per wheel (0x1d26c4 sets
it once; 0x1d27d0 advances it in each iteration's delay slot). On `bus_paris` and `bus_tokyo`
(MISC) the front wheels' items come after the rear ones, so both rear scans run off the end of
the item array and the rear offsets keep 0, unless the memory after the array happens to match.

`*(car+0xc)` is taken to be the `Base` payload: the model build `FUN_001c7940` reads it from item 0's `as` payload
word 0, and every `as` payload on the disc starts with the `Base` tag [D]. Base word 0 is the
0xffffffff placeholder, words 4-6 / 8-10 a box min / max [D]. The loader writing the pointer there
is not traced.

Per model update (`FUN_001d2cb0`, from `FUN_001d34a8`), only when the vehicle's +0x40 word is 1:

1. steer `s = clamp(vehicle +0x28c, ±0.075)` turns, moved at most 0.01 per update from the last
   value (+0x328); 0.002 when the driver kind is 0 or 1 and the camera mode has `interiorView`.
2. angles `[s, −s, 0, 0]`. An `IS_SUB` vehicle adds the fin terms: `a` = vehicle +0x244 and
   `b` = vehicle +0x254 (0 when +0x29c is set), each moved at most 0.075 / 0.1 from last time
   (+0x32c / +0x338); `angle[0..1] += 0.07·a`, `angle[2] = −s − 0.15·b`, `angle[3] = −s + 0.15·b`.
3. per slot k: hub = `R_y(angle[k] turns)` with translation y = `WHEEL_RADIUS +
   min(compression[k], SPRING_COMPRESSION_LIMIT_DRAW) − off[k] − SPRING_REST_LENGTH`
   (compression = vehicle +0x210 + 4k, the spring state of docs/systems/drive-suspension.md);
   wheel = `R_x(roll[k] turns) · hub`, roll = vehicle +0x230 for slots 0-1 and +0x234 for 2-3. A
   slot with index 0 writes a scratch matrix.

`R_y` / `R_x` are `FUN_002cc590` / `FUN_002cc510`, whose sin/cos (`FUN_002cbd50` / `FUN_002cbeb0`)
take turns. The rolls are wrapped into [0, 1] after each step (`spin_step`: both axles += the
per-tick step +0x274 [M FUN_0018ecb8 :532-551]; the player pass `FUN_0018cef8` :768-782 varies
it on flags not traced). The function returns `angle[0]`, which feeds dial 0.

Before the rig, `FUN_001cc738` runs a hinge for each joint bit in `car+0x70` (set by
`FUN_001cc6b0` for entries of type 8) and writes that part's `Skel` matrix. Which parts and what
drives the hinge (`FUN_001cc9b0`) is not traced, so those matrices stay identity in
`CarPose::rest`.

### 11.5 Unproven

- Which supersnow body is which form, and what the screen shows during each list: no render or
  live capture was made.
- The `Base` pointer (§11.4), and the memory past the item array in the bus scans.
- The 0xee camera-facing item (the MIS01 helicopter), the hinge parts, the LOD factor K, the
  run-time entry table of `+0x0c & 0x10`, and the player-pass roll variants.
- The sin/cos are f32 `sin_cos` here; the game uses VU0 polynomials. The values agree to about
  1e-6, not bit for bit.
