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
  The parts of one article share the table and differ in their colour data: paris `S04_s4s_00`'s
  36 parts all draw the same 2 packets.
- Group table: `{0xA0000000, 0, 2n}` then n × `{0xA000FFFF, packet}`. The table is one array for
  the whole module; each article's pointer is its own first group: the first group's packet header
  names that same article for every article with models (paris 467, MIS4 479, MIS3 416 — no
  exception).
- Packet: `+0x00` → GeoPrim header (`+0x00` render-method list, `+0x14` name string), `+0x08` DMA
  tag (`0x6000xxxx`, low 16 bits = quadwords), `+0x10` VIF codes. The render-method list's first
  `0x0300000N` word is followed by a `__EAGL::TAR:::{…}TEX0::<name>` pointer = the texture. A list
  without one keeps the previous packet's texture (inferred; the EAGL state machine is not traced).

## 4. The VIF batch [D]

One batch per packet (paris: 1,778 of 1,779 packets carry one header + one `MSCNT`; the other is
empty):

| unpack | meaning |
|---|---|
| `V4-32` ×1 at VU 0 | word 0 = vertex count |
| `V4-8` ×1, one to three | strip-end list: bytes in order, 0-terminated, each = 3 × the first vertex of a new strip; the last strip ends at the count |
| `V3-16` usn | position = (u16 − 0x8000) / 256 |
| `V3-32` (masked; mask leaves x/y/z as data) | position as floats (sky, water, some windows and lights) |
| `V2-16` usn / `V2-32` | UV, same rule / floats |
| `V4-8` ×count, `V3-8` | colours / normals (not decoded) |

Proof of the rules: the /256 rule reproduces each article's `Base` box to 1/256 (e.g.
`S12_RiverBnk_n_Road_00` −14.0078..14.207 × −3.5..4.5 × −30.0078..30.0078); every strip-list byte
of all 8 tracks is a multiple of 3 below 3 × count and the ends fall on the geometric breaks; road
strips span v = 0..1. With the placements of §2 (tests, `NF_ISO`):

| track | placements (unresolved) | strips | triangles | road nodes | surface within 1 m |
|---|---|---|---|---|---|
| MIS01 paris | 2552 (86) | 58,207 | 247,883 | 880 | 772 |
| MIS11 underwater | 1958 (107) | 45,598 | 295,427 | 136 | 3 (open water) |
| MIS13A | 3021 (76) | 55,571 | 329,931 | 88 | 78 |
| MIS13B | 1412 (24) | 20,325 | 120,176 | 56 | 46 |
| MIS13C | 371 (21) | 5,260 | 30,583 | 0 | — |
| MIS3 snow1a | 3689 (65) | 58,496 | 261,927 | 348 | 307 |
| MIS4 snow2a | 3331 (45) | 43,017 | 185,514 | 440 | 272 |
| RACE snow2a | 3314 (45) | 42,742 | 181,958 | 345 | 198 |

Paris cross-check: under 739 road nodes both the render surface and the `CDat` collision surface
exist; they agree within 5 cm at 730. The `CDat ci#0` items are the inverse kind of matrix
(world → local): `world_j = row_j · (local − t)` with rows at +0x00/(0,1,0)/+0x20 and t at +0x30.
Every texture named by MIS01/MIS11/MIS13B/MIS13C/MIS4/RACE is in the track `.ssh` or
`data\render\common.ssh`; MIS13A and MIS3 each name one missing `badshapereference`.

## 5. Sky and ground [D]

- **Ground** has no separate system: terrain, road and river banks are placed articles like any
  other (`PTSR2_*`, `MountainHill`, `River_Gorge1a`, `S12_*`, `T_s4s_*`).
- **Sky**: the `sky*` articles (`sky01`/`sky02`, `sky01_G`/`sky02_G`, `sky01_snow2`/`sky04_snow2cloud`,
  `skycloudlayer`) are placed through `in#0` too, with uniform scales 0.0136 (paris), 3.6-3.8
  (jungle) and 1 (snow), and use `V3-32` positions. Placed as data says, paris's dome is ±30 units
  around (120, −7.5, 0): it is presumably drawn camera-relative by a separate pass ("Sky blend" /
  "Sky bright" strings in DRIVING.ELF), which is not traced.

## 6. Open

1. The VU1 microcode (eaglrm.o) was not disassembled; the /256 scale and the strip-list unit are
   data-proven only.
2. MIS4/RACE: fewer road nodes have a decoded surface (272/440, 198/345), and render and collision
   coverage of the nodes differ there (both present at 225 of 440 within 4 m). Some articles' render
   model is far smaller than their collision tile (`PTSR2_dd_45`: a 10-vertex piece vs a 40 × 24
   collision tile). Whether another article draws those surfaces, or a mission-phase switch hides
   them, is not settled.
3. The sky draw rule; colours/normals; the texture carry-over between packets.
4. The car start used by the tests is the first `rn` node (the PR #5 rule), not a traced game rule.
