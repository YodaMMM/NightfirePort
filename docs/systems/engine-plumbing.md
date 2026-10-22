# Engine plumbing the guard AI reaches: parse / list / loader equivalents

The call-tree check (`tools/calltree/calltree.py`) walks the game's code from every guard-AI DSTATE
the port mentions and lists functions the port never names. This doc covers the 31 engine-plumbing
functions task `plumbing-equivalents` owns: the map parser's block handlers, its binary-reader
primitives, the generic list/stack containers, and the streaming loader tail. For each: what the
game does [M], and where the port does the same work (with the citation the check greps for), or
why the port never reaches it on the shipped path.

Game callers were located by grepping `work/decomp/ACTION.ELF` for each mangled name [M]. Block-id
counts are the measured census over all 2,230 map entries on the disc (docs/formats/map.md) [D].

## The map block dispatcher (parsemap_handle_block_id 0x1d1020)

The game's block-id → handler switch (decompiled 0x1d1020 [M]) and where each handler lives in the
port. `Level::parse` (crates/nf-formats/src/level.rs) is the port's dispatcher: same block set, same
per-block payloads, dynamic `Vec`s instead of the game's count-based preallocation.

| Ids | Game handler | Port |
|---|---|---|
| 0x00, 0x08 | `parsemap_block_dict_xyz` 0x1d2310 | never occurs on our disc [D census]; see below |
| 0x01 | `parsemap_block_dict_uv` 0x1d23a0 | never occurs [D census] |
| 0x02 | `parsemap_block_dict_rgba` 0x1d23b8 | never occurs [D census] |
| 0x03, 0x0a, 0x0b | `parsemap_block_dict_comlist` 0x1d24b0 | never occurs [D census] |
| 0x20 | `parsemap_block_dict_norm` 0x1d23d0 | never occurs [D census] |
| 0x22, 0x23 | `parsemap_block_coll_data` 0x1d24c8 | never occurs [D census] (the shipped collision block is 0x2e/0x2f `parsemap_block_Coll_Data_New` 0x1d1810, which the port parses in crates/nf-formats/src/collision.rs) |
| 0x25 | `parsemap_block_datums` 0x1d24f8 | never occurs [D census] |
| 0x11 | `parsemap_block_palette_data_pc` 0x1d2278 | never occurs [D census] |
| 0x17 | `parsemap_block_texture_data_GameCube` 0x1d2148 | never occurs [D census] |
| 0x18 | `parsemap_block_texture_data_Xbox` 0x1d21e0 | never occurs [D census] |
| 0x29 | `parsemap_block_texture_data_pcdx` 0x1d2080 | never occurs [D census] |
| 0x0e | `parsemap_block_map_header` 0x1d1430 | cited in level.rs: the game allocates `MapEntityCount` 0x50-byte cel-list slots and `m_paths` from the header (0x1d1430 [M]); the port's per-map `Vec`s grow as blocks arrive, so no count preallocation exists |
| 0x0f | `parsemap_block_palette_header` 0x1d1fc0 | cited in texture.rs (`textures()`) |
| 0x10 | `parsemap_block_texture_header` 0x1d1f80 | cited in texture.rs (`textures()`) |
| 0x12 | `parsemap_block_palette_data_psx` 0x1d20b0 | cited in texture.rs (`textures()`) |
| 0x15, 0x16 | `parsemap_block_texture_data_pc` 0x1d2048 | cited in texture.rs (`textures()`); 0x16 carries all 25,082 pixel blocks on our disc [D census] |
| 0x1c | `parsemap_block_memory_discard` 0x1d2538 | cited in levelbin.rs: the game records `FileDiscard = FileNextBlock` (0x1d2538 [M]) so the streamed loader can free the read buffer past that point; the port reads whole archive entries through LevelBin, nothing streams, so there is no discard point to record |
| 0x2c | `parsemap_block_hashlist` 0x1d1e78 | ported in map.rs/level.rs: `hashlist_entries` reads the same `(size−4)/4` dwords and drops `0x06000000` (the game's sentinel), `Level::parse` stores the rest per map |

### The never-occurring handlers (proof)

Every one of the eleven handlers above is only reachable through `parsemap_handle_block_id`
(0x1d1020) dispatching on a block id [M 0x1d1020 cases 0x00–0x29 as tabled]. The measured census of
all 2,230 map entries on the NTSC-U disc (docs/formats/map.md, "Block ids on our disc") lists the
ids that occur: 0x04 0x05 0x0e 0x0f 0x10 0x12 0x16 0x19 0x1a 0x1c 0x1d 0x21 0x26 0x27 0x28 0x2b
0x2c 0x2d 0x2e 0x30 [D]. None of the eleven ids appears, so the game never dispatches to these
handlers from any level load on the shipped disc: dict blocks (the shared-geometry-dictionary
path), the old coll-data blocks, datums, and the PC / GameCube / Xbox texture & palette variants.
The port's dispatcher comments carry the same ids + addresses (crates/nf-formats/src/map.rs, the
`id` module), so the check sees them; they are deliberately not implemented.

## Binary stream readers (BIN_Get*)

`BIN_GetByte` 0x1e8ad8 / `BIN_GetWord` 0x1e8af0 / `BIN_GetFloat` 0x1e8b48 read 1 / 2 / 4 bytes
through a moving `u8*` cursor and advance it [M]. Game callers: `Script_Load` 0x1d8248 and the per-
stream `Script_*Start` readers, plus `Emitter_LoadDefs` 0x17ab40 [M]. The port reads the same
fields of the same structures at the same offsets: crates/nf-formats/src/nis.rs (level scripts,
`Script_Load`'s format) and crates/nf-formats/src/level.rs `EmitterDef::parse_block` (the emitter
defs `Emitter_LoadDefs` reads) — fixed-offset `Buf` reads replace the cursor, consuming the
identical byte sequence [M]. Cited at both sites.

## Generic containers (DList / LList / Stack)

- `DList_MoveFromFree2InUse` 0x1cccb8 / `DList_RemoveFromInUse2Free` 0x1cc868: move a node between
  a pool's free and in-use doubly-linked lists (free count at +0x10, in-use count at +0x14) [M].
  Callers: `Light_Create` 0x1cc3d8, `Searchlight_Create` 0x1cb4f8, `Effect_Bullet` 0x1ce8a8 /
  `Effect_Player` 0x1cec48, `Sound_Play3D` 0x1dd548, `Text_DelMsg` 0x1b1640 [M]. The port models
  these pools as owned `Vec`s: the light pool (crates/nf-game/src/light.rs), the 200-slot decal
  FIFO (crates/nf-game/src/effect.rs), the searchlight record (crates/nf-game/src/searchlight.rs) —
  push/evict does the allocate-to-in-use / return-to-free work. Cited at each.
- `LList_AllocnNodes` 0x1ccbe0 / `LList_Cut` 0x1cca80: allocate hit-list nodes and move a filled
  list onto an object [M]; callers are the collision pickers (`Collide_Update` 0x1ea398,
  `Collide_Pick*` 0x1eae80/0x1eb320, `Coll_AddHitToList` 0x1ede68) [M]. The port appends contact
  records to the struck object's list in crates/nf-game/src/door.rs (the `Collide_Update` pick +
  forward) — cited there.
- `LList_Remove` 0x1ccac8: unlink an object's node from a shared list at deletion [M]; callers are
  every object `*_Delete` (`Drone_Delete` 0x136ce8 et al.) [M]. The port removes the drone from its
  `llist` index list (crates/nf-game/src/drone.rs `llist.retain(...)`) — cited there.
- `Stack_IsEmpty` 0x1cceb0 / `Stack_Pop` 0x1ccf68: the STACKINFO stack test/pop [M]. Callers:
  `Camera_PopStates` 0x1bbcd0 (pop every viewer's saved camera state), `Manager_SendMessage`
  0x21c858 (menu handler stack), `P_NFMAP_Handler` 0x20b608 [M]. The port folds the camera
  save/restore into the script-camera flag (crates/nf-game/src/drone.rs, the `spawn_fade_view`
  doc: "Camera_PushStates/PopStates save only the +0x29/+0x2a pair") and runs the menu page
  messages through the front-end page stack (crates/nf-view/src/lib.rs page-change arm) — cited.

## Sort / loader tail

- `Swap` 0x1d3518: byte-wise swap of `n` bytes [M]. Its only caller is `QuickSort` 0x1d30a8 [M];
  the port replaces QuickSort with `sort_by` at the same call sites (e.g. `Lights_CalcClosestLights`
  0x1cb2e0's closest-lights list, crates/nf-game/src/light.rs) — cited there.
- `LoaderLoadAnims` 0x1d6b48: a tail call of `LoaderLoad` 0x1d4a00 (the animation container load)
  [M]. The port's level-animation load is the `LoaderLoad` flow in crates/nf-view/src/lib.rs —
  cited there.
- `LoaderShrink` 0x1d6b68: after a streamed file finishes loading, memmove the used bytes down and
  shrink the block (`Mem_ShrinkToEndOfBlock`), trimming the over-allocated read buffer [M]. Its
  only caller is `parsemap_parsemap` 0x1d1cf0 [M]. The port reads whole, exact-size entries through
  LevelBin (crates/nf-formats/src/levelbin.rs) — nothing streams, nothing to shrink; cited there.

## Port-side changes in this task

- `parsemap_block_hashlist` 0x1d1e78 ported as `map::hashlist_entries` (crates/nf-formats/src/map.rs):
  `(size−4)/4` little-endian dwords, `0x06000000` entries dropped (the game skips that value before
  its `hashtable_getitem` 0x1ca5b0 / `isLoadable` check [M 0x1d1e78]). `Level::parse` stores the
  kept hashes per map. Tests: synthetic parse + filter (can fail), and an ISO-gated census test
  over every map entry on the disc with a sister assertion.
- Citations added at the port's equivalent-work sites listed above; nothing else behavioural.

## Live checks for the main agent

None needed: every claim here is [M] decompile or [D] the measured disc census; no new field
meanings were read from memory.
