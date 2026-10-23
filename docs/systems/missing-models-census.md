# Missing-models census (every on-foot level, every model path)

For each on-foot story level, a census through `nf_view::load` counting every path a model can go
missing in, pinned per level as numbers; zero misses is the goal. Test:
`crates/nf-view/tests/missing_models_census_live.rs` (ISO-gated; numbers/ids only).

## The census levels

The 21 on-foot story levels: `mission::LEVELS` (ACTION.ELF's level table [M 0x182630 area]) minus the three
codes with no FILES.BIN archive (0700000e/0f/10 — the game never loads them [campaign_flow_live]).
`0700004a` (Tower 2, part 2) is in the table and on the disc, so it is in. The cut-scene wrappers
(07000041/43/46/48/49) and the MP arena (0700004b) are out of scope.

## The lanes and where each model legally comes from

| Lane | Where the game finds the model | Evidence |
|---|---|---|
| Placement → object | `parsemap_block_map_data_static` 0x1d1998 (flag 0x8000) + `parsemap_create_dynamic_objects` 0x1d04f0 (the rest); by-hash or by-index into the map's object table | [M] |
| Map textures → batches | `psiCreateMapTextures` 0x11d200: **every** header gets a runtime slot at `map+0x10[iVar26]`, 1:1 with the disc headers, no holes; the batch texture indices index this array; a header whose pixel block is short is loaded anyway — the memcpy (`amemcpy(uVar16, *(header+8), uVar6)`) reads `width×height×frames` bytes from the block pointer, running on into the following map bytes (the blocks are contiguous) | [M 0x11d200, re-read from the decompile this session] |
| Drone skins | `AnimObjectNew` 0x1be910: hashtable get over the session table (the union of every archive the session loaded, nothing frees skins — `AnimLoadFile` 0x1bdf40 first-wins, `LoadableLoad` 0x1d69e8's type-4 wipe only) — see docs/reports/drone-skins-g021.md | [M] |
| Script entities | `Script_CreateEntity` 0x1d8968: creates ONLY for the dispatch's raw types — 0x20 Break, 0x22 (→ the 0x2d Ladder_Create arm), 0x29 Switch, 0xe1 Destroy, 0xea, 0xeb, 0xf0, 0xf2, 0xf5 (0xf5 creates as type 0x3e). A type with no arm — including a RAW 0x3e mention (0x3e is only a dispatch TARGET, never a tested raw value) and the ty-0x31 mesh mentions — creates nothing: the stream's current object is reset, no spawn | [M 0x1d8968, re-read from the decompile this session] |
| Pickups | 0x1d04f0 case 0xf0: the placement's resolved object (the glist pointer, not a re-lookup) | [M] |
| Animations | `LoaderLoad` 0x1d4a00 type 0/3 rewrites the hash nibble → `07F000NN.bin` alongside `070000NN.bin` | [M] |

## What the census found and what was fixed

**The 0700000d short-texture drop (fixed — the U-017 class).** The port's map-texture parser DROPPED textures
whose pixel block is shorter than the header implies, compacting the map's texture array. The game has no such
drop: `psiCreateMapTextures` 0x11d200 gives every header its slot (1:1, no holes) and memcpy's the full size from
the block pointer, overreading into the following map bytes. With a hole, every later binding drifted and the
batches the drift stranded fell back to the 1×1 white. On the disc exactly one story-archive texture is short:
0700000d's own map 49 ("010000e7", 186 headers) header 18 (0x12) — 512 pixel bytes for a 2,048-byte header
[Disc probe, matching the documented U-017 case; the first census pass's "six short textures" (18/45/136/161/
174/184) were the six distinct white-FALLBACK KEYS the drift produced, not short headers — corrected here].
Fix: `nf_formats::texture::textures` keeps every texture at its header position with the game's overread bytes
(zero-fill past the map buffer); still reported in the `bad` vector. After the fix the census's fallback column
is 0 on every level, and the models_world golden's `tex_oob` cell for 0700000d went 1 → 0 (the one out-of-bounds
batch pointed at the LAST header index, out of bounds only under the compacted array).

**The ty-0x31 entity mentions are not misses.** The first census pass counted every script `Cmd::Entity` hash
not in by_hash (01:1, 04:2, 08:1, 0a:1, 0c:10, 14:5). All were obj_type 0x31 (0x02 mesh hashes) — a type with
no arm in `Script_CreateEntity` 0x1d8968, so the game creates nothing for them. The census counts only the
dispatch's raw types; those all resolve (entity_miss = 0 on every level with the corrected list).

**Pinned non-zero numbers (game behaviour, not port defects).**
- 07000014 `skinless_drones` = 5: the 0x050000b1 non-captain drones — the skin rides only 07000015/16, which
  the fresh session has not loaded when 14 boots. The shipped game renders them skinless too ([D
  drone-skins-g021 scan of all 99 archives]; `drone_skins_all_levels_live` pins the same drones as
  `b1_drones − b1_captains`).
- 07000001 `skinless_script` = 1, RAW ARCHIVE (the one known remaining gap): script 0x0600005b stream 7 asks
  for skin 0x05000037, which 07000005 and 07000006 carry ([Disc probe: census carrier scan]). The game's
  session table keeps every earlier mission archive's skins (nothing frees them [M 0x1bdf40/0x1ca520, G-021]),
  so the game resolves it. Master's session-skins merge (lib.rs g-021, `wanted` + `session_container`) covers
  drone placements (0x0f param 0) and 0xd9 ScriptPlayer placements' DIRECT `Cmd::Anim` skins only — and
  0x0600005b is reachable from none of 01's 0xd9 placements (0x06000016/86/87) [Disc probe], so the sim still
  lacks the skin post-load (the census prints it as a diagnostic). Closing it is a loader change (the
  merge appends the foreign archive's maps, so it needs its own golden + render checks) — left open.

## Tests

- `on_foot_levels_census` — the pinned walk (all numbers asserted per level; the 07000001 raw-archive skin
  gap pinned at 1 with its analysis).
- `fallback_materials_diagnostic` (`--ignored`, `NF_CENSUS_LEVEL=<level>`) — prints a level's fallback
  material keys per map; the tool that localized the drift.
- Sister: `nf_formats::texture::tests::short_texture_stays_at_its_index_with_the_overread_bytes` asserts the
  kept-texture behaviour (position, the block's own bytes first, the read continuing into the following map
  bytes) and fails if the drop returns; the census's `fallback == 0` pin fails on any re-introduced drift.
