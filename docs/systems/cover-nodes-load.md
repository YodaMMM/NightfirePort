# Cover node loading: which placements become live nodes (U-050)

An earlier port change built cover nodes from every map entry of the level archive. This
note is the decompile + disc answer to whether the game does that: **it does not — the game
creates live dynamic objects (cover nodes included) from ONE map's placement block only: the
LAST map entry the loader parses, which on every mission level is the kind-1 main map.** The
shared sub-map 010000f3's template pair never becomes live. `cover::build` staying main-map
only is correct.

## The creation path [M]

Live dynamic objects are created in exactly one place per level load:

- `ResetMap_Load` 0x1d6360 (ACTION.ELF): after both `LoaderLoad` passes finish, it calls
  `parsemap_block_map_data_dynamic` 0x1d1b68 **once**, as
  `block_map_data_dynamic(&DynamicBH, DynamicPtr, 1)` — flag 1 is what arms creation.
- `parsemap_block_map_data_dynamic` 0x1d1b68 walks the placements of the block recorded in the
  global `DynamicPtr` (0x30cff0), skips placements with `flags & 0xa000`, and calls
  `parsemap_create_dynamic_objects` 0x1d04f0 per placement, passing its own third argument
  (the create flag) through as the `param_7` gate.
- `parsemap_create_dynamic_objects` 0x1d04f0 case 0xe5 → `Drone_CoverCornerNode` 0x136ff0,
  case 0xe6 → `Drone_CoverLowNode` 0x137340 — every case is gated on `if (param_7 != '\0')`
  (0x1d04f0 cases 0xe5/0xe6), so a flag-0 walk creates nothing.
- `Drone_CoverCornerNode` 0x136ff0 appends to the live array with **no position or validity
  filter**: it memsets node `DAT_00299104 * 0xb0` at base `DAT_00299108` (0xb0 stride),
  fills pos/direction/side flags/angle (+8, `param * 0.017453294`) / depth / offset bytes
  (defaults 8) from the placement params, runs `AINetwork_InitEmitter`, and increments the
  count `DAT_00299104`. Its only guard is `Drone_bDisableSystem == 0` (0x30ca9e). A (0,0,0)
  no-param placement fed to it WOULD become a live node — the guard is not here.

## The guard that keeps the sub-maps out [M]

Two facts in the parse path make `DynamicPtr` point at the main map's block when the create
runs:

1. **Per-file reset.** `parsemap_parsemap` 0x1d1cf0 state 0 — the open of every map file —
   stores `DynamicBH = 0` and `DynamicPtr = 0` (disassembly 0x1d1dc8 `sw $zero, -0x3018($at)`
   → 0x30cfe8, 0x1d1dd0 `sw $zero, -0x3010($at)` → 0x30cff0, unconditional). Each map entry
   is parsed to completion before the next (`LoaderLoad` 0x1d4a00 case 4 advances
   `pDirDataCurFile` only when `LoaderProcess` 0x1d4880 reports the parsemap state machine
   done), and `LoaderProcess` routes every type-0/1/2/0xb entry through `parsemap_parsemap`.
2. **The record is per-file.** `parsemap_handle_block_id` 0x1d1020 case 0x1a (jump-table entry
   0x1a → 0x1d11f4) unconditionally stores `DynamicBH = {block id, size}` and
   `DynamicPtr = FileNextBlock` (0x1d121c `sw $v0, -0x7680($gp)` → 0x30cff0) and then runs the
   static build (`jal 0x1d1998`). So after the whole container is parsed, `DynamicPtr` holds
   the **last** map file's block 0x1a — the same block the port parses as `MAP_DATA_STATIC`
   (0x1a, `crates/nf-formats/src/map.rs`) whose placements are `LevelMap::placements`.
3. The parse-time walk never creates anything: `parsemap_parsemap` state 2 calls
   `block_map_data_dynamic(..., 0)` (flag 0 — all create cases inert), and only when
   `MP_NeedFile` 0x120ca0 asks for the file at all.

## The disc order [D]

Archive entry order in the level .bin containers (probe: `probe_cover_entries` over the ISO;
entry kinds are the loader's file types — kind 1 = the main map, kinds 0/2/11 sub-maps):

| level | 010000f3 entry | kind-0/2/11 sub-maps before the main map | main map (kind 1) entry | main-map 0xe5/0xe6 placements |
|---|---|---|---|---|
| 07000001 | 47 | 46..97 | **614** `01000093` | 20 |
| 07000002 | 58 | 46..759 | **760** `01000094` | 9 |
| 07000004 | 52 | 46..432 | **433** `010000c5` | 0 |
| 07000005 | 58 | 46..700 | **701** `01000099` | 18 |
| 07000006 | 53 | 46..674 | **675** `01000098` | 114 (80 corner + 34 low) |

Every mission archive: the kind-1 main map is the **last** map entry, and 010000f3 sits early
among the sub-maps carrying its template pair — "Cover Corner Node" 0xe5 + "Cover Low Node"
0xe6, both at translation (0,0,0) with no params [D placement bytes].

So the create at `ResetMap_Load` sees the main map's placements only: the live cover-node
list is the main map's 0xe5/0xe6 placements in placement order, nothing else. 010000f3's pair
is dead data in mission levels.

## Why the every-map scan broke the three measured tests

The scan (cover nodes from `level.maps` — every entry) added two (0,0,0) nodes the game never
creates. Prepended, they shifted the live indices (the 07000005 chase-trace node moved 1 → 3
[L work/chase_trace.tsv, ai_live / cover_selection_live]); appended, the two free (0,0,0)
nodes became selectable cover for guards whose measured walks the tests pin
(guard_reaction_live, escort_follow_07000001_live, npc_doors_live). The tests are right; the
scan contradicted the loader. `cover.rs`'s doc comment and the two `#[ignore]`d tests in
`cover_nodes_live.rs` claimed the every-map creation — since fixed (the decompile
chain above).

## Tests

`crates/nf-view/tests/cover_nodes_submap_live.rs` pins the order (main map last, 010000f3
present with the template pair, per-level main-map cover counts — all [D]) and the live list
(the port's covers on 07000004/07000005: main-map-only counts, no (0,0,0) node, list order ==
placement order — the [M] chain's consequence). `cover_nodes_live.rs`'s two scan-era tests now
pin the main-map counts (07000006: 114 = 80+34) and that the template pair stays out of the
live list (07000004: empty) while its bytes stay pinned in the level parse.

## Live checks (open)

PCSX2, fresh boot (a save-state resume skips the boot path):

1. 07000005: after load, dump the live cover-node array — count word at 0x299104 (expect 18),
   array base 0x299108 (0xb0 stride). Expected: node 0 pos (+0x20) = (59.07, 26.04, 94.05),
   node 1 = (59.11, 26.04, 98.69) [D order], and NO node at (0,0,0).
2. 07000004: same dump — expected count 0 (its main map has no 0xe5/0xe6 placements), proving
   the 010000f3 pair is not instantiated.
