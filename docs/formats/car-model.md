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
normal (byte − 0x80) / 127 + two `V4-16` arrays (undecoded); B = five `V4-32` arrays
(position, normal, UV, two undecoded); positions-only. 249,532 of 249,596 normals have unit
length ± 5 %; there is no colour upload in any car chain. Strip list: 3 bytes per vertex, 2 for
positions-only batches, wrapping past 255.

**Old reading** (`crp_mesh::TrackModule::extract_car_mesh`): treats `vd#1` as `as` indices,
places `as#k` with item k's matrix, drops mirrored items, decodes the batch behind
`*(entry − 0x18)`, reads UVs at /1024 but normals as colours. On the vanquish 4,221 of its
10,375 vertices lie on the car the new reading places.

**Open**: which `vd` list the game selects for each vehicle (the caller of the `FUN_001d2618`
vtable slot is not found; the constructor leaves none, which draws every item); the run-time
`Skel` / 0xef matrices (identity at rest is not proven); the texture of 85 strips whose model
sets none before them (GS state carried from an earlier draw, not traced); the ultralights'
`blck` texture, which no sheet of their archives holds; the two undecoded per-vertex arrays; the
car microcode (`MatCar*`, in the car module) is not disassembled, so the /1024 UV rule, the
normal bias and the strip-list unit are data-proven only.
