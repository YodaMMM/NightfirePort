# Driving track world geometry ("GeoPrim") — placements, models, packets

Status: decoded for all 8 track `.crp` files (`crates/nf-formats/src/geoprim.rs`, tests
`crates/nf-formats/tests/geoprim_disc.rs`). Evidence tags: [M] DRIVING.ELF decompile
(function + address), [D] disc data, measured. Where only [D] is given, no consumer code was read.

## 1. There is no run-time road generator [D]

`MISC.VIV` → `data\render\eaglrm.o` (260,735 B, plain) is a MIPS ET_REL object with one
`.data` section (0x3c360 B) + `.rel.data`/`.symtab`/`.strtab`/`.shstrtab` and **no code section**.
Its 113 symbols are VU1 microcode blocks (`LitTexture0__EAGLMicroCode`, `ClipTexture0__…`, …) and
render-method tables (`LitTexture`, `ClipTexture`, `TextureFog`, `ScrollTexture`, `XFadeTexture`, …).
It renders packets; it does not build geometry.

The road tiles are ordinary EAGL models baked into the track's embedded module (`ELFd`+`ELFr`,
crp_mesh.rs). In EAGL terms a GeoPrim is one VIF packet of a model: each packet's header names it
`__GPRenderMethod_<article>::{as  0000}::Model[0].00_<n>_name`, and the module's undefined symbols
include its states (`__EAGL::GeoPrimState:::STATE::…;primtype=tristrip`). docs/systems/driving-load-path.md §5's
"parameterized primitives generated at run time" reading is corrected by this file.

## 2. Placements: `Map ` → `in#0` [M + D]

`FUN_0022faf8` 0x22faf8 ("Init World") finds the `Map ` article, then
`FUN_002d5920(map, 'in  ', 0, -1)` and walks `count` (the record's +8) items of **0x40 bytes**
[M lines 195-205, the `iVar3 + 0x40` steps]. For an item it builds the instance matrix from words
0-2, 4-6, 8-10, 12-14, zeroing words 3/7/11 and setting word 15 = 1.0f (`uStack_100..uStack_c8`,
then `FUN_001c9258`) [M]. So:

| offset | field |
|---|---|
| +0x00 / +0x10 / +0x20 | the images of the local x / y / z axes (3 × f32; may carry a uniform scale) |
| +0x0c | u8 flags (0x10 = dynamic instance, 0x20 checked too) [M 0x22faf8, 0x1b1988] |
| +0x0e | u16 index into `ps#0` (0x20-byte records) [M 0x22faf8] |
| +0x1c | reference: `'sr'` short tag + index into the Map's `sr#N` strings (a pointer after load) [M+D] |
| +0x30 | origin (3 × f32) |

`world = x·axis0 + y·axis1 + z·axis2 + origin`. The direction is [D]: read this way the decoded
road lies under the road network and on the collision world (§4); read as the inverse it does not.
The strings are `CARP::<article>::{as  NNNN}` — a part of an article. Paris: 2,552 items, 2,466 name
a part with a model. `Wmap` + `wn#1..94` index these items spatially (`wn` u16 lists, max index
2,551 = the item count − 1) [D].

## 3. Part → model → packets [D]

- Article window `as#NNNN`: payload ends in `{u16 index, "sr"}` → the article's `sr#index` =
  `EAGL::<model article>::{as  0000}::Model[0].XX` (XX = NNNN for most parts, not all: paris
  1,811 equal / 79 different; the decoder follows the string).
  Parts whose tail is not `sr` are effects (`ef#`), the invisible `{as  ffff}` barriers, and the
  `trafficlight`-form tails — not decoded (paris: 86 placements).
- `__Model:::<…>::Model[0].XX` symbol (a `.data` offset; the image holds `.data` at file offset
  0x40, and every relocated pointer is `.data`-relative): `+0x00` parent (`Colours` block with the
  texture name), `+0xc0` u32 = XX, `+0xc8` u32 group count, `+0xcc` pointer to the group table.
  The parts of one article share the table and differ only in `+0xc0` (the part index, which
  selects per-part colours and textures, §4b) and the small `+0xbc` block; paris `S04_s4s_00`'s 36
  parts all draw the same 2 batches with 36 different colour sets.
- `+0xb8` → the model's `Colours` list when it has one: `{0, → "Colours", n, 0}` then n × 16-byte
  `{u16 vertex count, u16 7, stride, base, flag}` — one per group entry, `base` = part 0's colour
  bytes, `stride` = the distance between two parts' colour uploads [D]. The draw does not read it
  (the interpreter indexes the op-4 table instead, §4b); it serves here as an independent check.
- Group table: `{0xA0000000, 0, 2n}` then n × `{0xA000FFFF, packet}`. The table is one array for
  the whole module; each article's pointer is its own first group: the first group's packet header
  names that same article for every article with models (paris 467, MIS4 479, MIS3 416 — no
  exception).
- Packet: `+0x00` → GeoPrim header (`+0x00` render-method list, `+0x14` name string), `+0x08` DMA
  tag (`0x6000xxxx`, low 16 bits = quadwords), `+0x10` VIF codes. **The game does not draw this
  batch for this entry** — see §4b for what it draws. Texture: §4b. A list that names no texture
  keeps the previous entry's (inferred; the EAGL state machine is not traced).

## 4. The VIF batch [D]

One batch per packet object (paris: 1,778 of 1,779 carry one header + one `MSCNT`; the other is
empty). Which batch an entry draws, and its colours, is in §4b.

| unpack | meaning |
|---|---|
| `V4-32` ×1 at VU 0 | word 0 = vertex count |
| `V4-8` ×1, one to three | strip-end list: bytes in order, 0-terminated, each = 3 × the first vertex of a new strip; the last strip ends at the count |
| `V3-16` usn | position = (u16 − 0x8000) / 256 |
| `V3-32` (masked; mask leaves x/y/z as data) | position as floats (sky, water, some windows and lights) |
| `V4-32` unmasked at VU 0x36, num = the count | position as floats (x, y, z, w = 1.0f): MIS11's `AnthWater`, `Siloroom1Water`, `Tanker_Ocean1`/`1b`/`1c`/`2b`/`3` — the slot the `EndCoast_Ocean_*` water fills with `V3-16` (colours at VU 0x7a in both, no UVs in either) |
| `V2-16` usn / `V2-32` | UV, same rule / floats |
| `V3-8` masked, usn | per-vertex colour R, G, B (§4b) |
| `V4-32` (other addresses), second `V3`/`V2` sets | not decoded (layout depends on the microcode) |
| `V4-32` at VU 0x10 after strip lists at 0x0d..0x0f | not decoded: also (x, y, z, 1.0) positions, but the strip-list bytes are not multiples of 3 (`Rock_Collapse`: 14, 30, 40 … for 54 vertices), so that layout's strip unit is unknown (`Rock_Collapse`, `JA_SAL`, `JB_SAL`, `LookOutTowerExplosion`: 22 chains on MIS11/13A/13B/3) |

Proof of the rules: the /256 rule reproduces each article's `Base` box to 1/256 (e.g.
`S12_RiverBnk_n_Road_00` −14.0078..14.207 × −3.5..4.5 × −30.0078..30.0078); every strip-list byte
of all 8 tracks is a multiple of 3 below 3 × count and the ends fall on the geometric breaks; road
strips span v = 0..1. With the placements of §2 and the draw chains of §4b (tests, `NF_ISO`):

| track | placements (unresolved) | strips | triangles | road nodes | surface within 1 m (own-batch reading) |
|---|---|---|---|---|---|
| MIS01 paris | 2552 (86) | 60,327 | 257,412 | 880 | 806 (772) |
| MIS11 underwater | 1958 (107) | 40,307 | 281,672 | 136 | 0 (3) (open water) |
| MIS13A | 3021 (76) | 55,963 | 325,807 | 88 | 88 (78) |
| MIS13B | 1412 (24) | 22,347 | 121,974 | 56 | 50 (46) |
| MIS13C | 371 (21) | 5,657 | 32,086 | 0 | — |
| MIS3 snow1a | 3689 (65) | 61,842 | 283,733 | 348 | 336 (307) |
| MIS4 snow2a | 3331 (45) | 43,977 | 184,494 | 440 | 419 (272) |
| RACE snow2a | 3314 (45) | 43,421 | 180,393 | 345 | 332 (198) |

Paris cross-check: under 762 road nodes both the render surface and the `CDat` collision surface
exist; they agree within 5 cm at 760 (the own-batch reading: 730 of 739). The `CDat ci#0` items are
the inverse kind of matrix (world → local): `world_j = row_j · (local − t)` with rows at
+0x00/(0,1,0)/+0x20 and t at +0x30. Every texture named by the 8 tracks is in the track `.ssh` or
`data\render\common.ssh`, except the exporter's `badshapereference` placeholder (`bads`) on
MIS4/RACE.

## 4b. What a group entry draws: the render list's DMA chain [M + D]

**Draw path** [M `FUN_002843a0` 0x2843a0]: the model draw sets the global
`DAT_003483fc = model +0xc0` (the part index), then for each group entry `{0xA000FFFF, E}` calls
the render-method interpreter `FUN_002a3290(*E)` — on the GeoPrim header, whose `+0` is the
instruction list. The interpreter [M 0x2a3290] switches on the top byte; the lengths used to walk
a list are: ops 1 and 4 = 2 words, 2 = n + 2, 3 = 1 + n, 5/6/7 = 3 + n (n = low 16 bits),
0xe/0xf = 1, 0 = end. Every list of the 8 tracks walks cleanly with these and holds exactly one
op 4 [D].

**Op 4** (`0x04000002`, pointer T) copies 2 doublewords from `T + 16 × DAT_003483fc` into the DMA
chain [M case 4]. Each table entry is a DMA `call` tag with no payload (`0x50000000`, address X)
[D, every entry reached in the 8 tracks]. X is either
- a `call` tag (DMA id 5): payload = the part's colour upload (`STMASK 0x40404040`, optionally
  `STROW (0,0,0,0x80)`, `UNPACK V3-8` masked usn, num = vertex count), address = a `ret` tag
  (id 6) whose payload is the geometry batch shared by all parts; or
- a `ret` tag (id 6): one batch with the geometry and, when present, its colours.

Paris counts: 6,091 call-shaped and 1,450 ret-shaped table entries reached over all models. The
colour calls of one batch sit one stride apart; in all 1,057 paris runs the first call is
referenced by nothing and every later one by a table [D]; the first one's purpose is unknown.

**The geometry is not the entry object's own batch.** Paris `S04_s4s_00` entry 0x36858's list
calls the batch stored in object 0x355f8 (8 vertices, header named `P_Fern … _218`); the batch
stored in 0x36858 itself (60 vertices) is called by the next entry's list. Proof, all 8 tracks
(`the_draw_chain_not_the_entry_object_holds_each_models_geometry_and_colours`):

| track | `Colours` lists | match chain counts | match own counts | models with a box | chain inside box | own inside box |
|---|---|---|---|---|---|---|
| MIS01 | 1357 | 1357 | 81 | 1890 | 1871 | 394 |
| MIS11 | 314 | 304 | 0 | 727 | 716 | 254 |
| MIS13A | 743 | 743 | 24 | 1283 | 1232 | 424 |
| MIS13B | 851 | 851 | 86 | 2105 | 1834 | 472 |
| MIS13C | 237 | 237 | 36 | 709 | 616 | 244 |
| MIS3 | 762 | 744 | 8 | 1823 | 1750 | 754 |
| MIS4 | 1142 | 1136 | 94 | 1889 | 1792 | 452 |
| RACE | 1142 | 1136 | 106 | 1886 | 1789 | 448 |

("box" = the article's `Base` box + 1/128.) The road-node coverage and the collision agreement
(§4) rise with the chain reading on every track with roads.

**Colours** [D]: every `UNPACK V3-8` reached by a chain has STMASK 0x40404040 (x/y/z from the
data, w from the row), VIF mode 0 and num = the batch's vertex count; the decoder takes the three
bytes as R, G, B (`LocalStrip::colors`). The values are night lighting: e.g. `S04_s4s_00` part 1
reads 0x3d4251 / 0x191d26 (dim blue-grey) with lamp-lit vertices up to 0x7e839e, where the GS
convention makes 0x80 = 1.0 for a modulated texture. Paris: 59,950 of 60,327 strips carry colours.
Unproven: that the VU1 microcode passes these to the GS unchanged (eaglrm.o is not disassembled),
the alpha (the row's w, 0x80 where the chain sets STROW, otherwise inherited VIF state), and the
purpose of the unreferenced first colour call.

**Textures** [M case 3 / case 6, D]: op 3 (`0x0300000N`) carries N words whose top two bits pick
the descriptor: 0 = the word, 1 = word + part × 0x60, 2 = a run-time table, 3 = none; op 6
(`0x06KK00NN`) carries one at +8, plus KK × part × 0x60. So a part can carry its own texture
(paris: 1,938 mode-1 words, all resolving to a descriptor; `P_building3_12seg` part 10 → the
descriptor labelled `pb01_10`). The sheet name is the descriptor's +4 word (≤ 4 ASCII bytes): it
equals the `TEX0::` tail on every `{T0@…}` descriptor, while the per-part `{T0|…}` descriptors'
tails are export labels (`pb01_1` → sheet `pb02`) [D].

## 5. Sky and ground [D]

- **Ground** has no separate system: terrain, road and river banks are placed articles like any
  other (`PTSR2_*`, `MountainHill`, `River_Gorge1a`, `S12_*`, `T_s4s_*`).
- **Sky**: the `sky*` articles (`sky01`/`sky02`, `sky01_G`/`sky02_G`, `sky01_snow2`/`sky04_snow2cloud`,
  `skycloudlayer`) are placed through `in#0` too, with uniform scales 0.0136 (paris), 3.6-3.8
  (jungle) and 1 (snow), and use `V3-32` positions. Placed as data says, paris's dome is ±30 units
  around (120, −7.5, 0): it is presumably drawn camera-relative by a separate pass ("Sky blend" /
  "Sky bright" strings in DRIVING.ELF), which is not traced.

## 6. Open

1. The VU1 microcode (eaglrm.o) was not disassembled; the /256 scale, the strip-list unit and the
   colour pass-through are data-proven only.
2. MIS4/RACE coverage: with the chain reading 419/440 and 332/345 road nodes have a surface within
   1 m (was 272 and 198). The remaining gap is not examined.
3. The sky draw rule; the texture carry-over between entries; the second UV/position sets and
   `V4-32` blocks some layouts carry; the role of the `+0xbc` block and of `FUN_00285ca8` (run when
   model `+0xd0` ≠ 0, before the part index is set).
4. The car start: traced, see docs/formats/trackstart.md (`nf_formats::trackstart::track_spawn`).
5. How the `Lighting` tuning (§7) combines with the per-vertex colours.

## 7. Render tuning: fog and lighting (`crates/nf-formats/src/rendertun.rs`)

Each track VIV carries `data\tuning\Render\Fog\<track>.tun` and `…\Lighting\<track>.tun`
(RefPack; `Key=value` text). Keys are the tuner names without spaces (`fog START` → `fogSTART`).

- **Fog** [M `FUN_001ac668` 0x1ac668 registers `fog START` → +0x10 (1..5000), `fog END` → +0x24,
  `fog mode` → +0x1c (0..3), `fog Density` → +0x18 (0..1), `Fog Colour` → +0x20]. The colour
  string `a,b,c,d` is scanned as four ints and stored `a<<24 | d<<16 | c<<8 | b` (b unmasked)
  [M `FUN_00144740`]; `FUN_001ac978` swaps bytes 0/2 and `FUN_0028cf98` swaps them back into GS
  register 0x3d (FOGCOL), so FOGCOL R/G/B = the stored bytes 0/1/2: paris (22, 15, 20), snow1a
  (40, 45, 50) from `255,-13488856,45,50`. The apply step also computes
  `density × 255 / (end − start)` [M 0x1ac978]: paris 1..1450.21 with density 0.000475 gives
  8.4e-5 — next to no fog — where the other tracks use density 1. `fog mode` is not read by the
  apply step; how the VU code uses the slope is not traced.
- **Lighting** [M `FUN_001ae630` 0x1ae630, the `RLightManager`]: four instances (`World`,
  `Character`, `Unused1`, `Unused2`, 0x70 bytes apart) of `Ambient Sky` (+0xe0), `Ambient Ground`
  (+0xd0), `Light 1` (+0xb0), `Light 2` (+0xc0) RGBA floats plus their angles, and globals
  (`Enable light maps`, `Light map lighting bias`, `Sun height`, `Moon size`). `FUN_001b0158`
  turns the angles into per-instance light matrices [M]. Paris `World`: sky ambient 0.72 grey,
  ground 0, Light1 (0.90, 0.94, 1.00), Light2 0.478 grey; light maps off. Directional lights need
  normals, which the colour-carrying world batches lack [D]; whether the `World` ambient scales
  the per-vertex colours in the microcode is not traced.
