# s/g016-world: G-016 world assembly (portal endpoints, dynamic-object census)

RNG: no pinned 07000012 test moved (walkreal, walk, enemies, corridor guards, interactables). The
change does widen 07000012's portal graph, which feeds the drones' drawn gate and the collision
cel scope, so a shift of the shared random stream on other 07000012 play paths is not ruled out.

## What changed (logic and data, `nf_view::scene::parse_cel_graph`)

`parsemap_block_portal_data` 0x1d15f8 resolves each block-0x21 record's two endpoints by walking
`glb_world`'s cel chain for `cel+0x38 == pCelList + idx*0x50` [M]. Read from the disassembly
(0x1d160c-0x1d1780):

- `s3`/`s4` (celA/celB) are zeroed once before the record loop and only `movz`-written on a match.
  An endpoint that names no cel keeps the previous record's cel.
- `build_alloc_cel` 0x1c93f8 prepends (`cel+0x14 = old head`), so the walk is newest-first and the
  last match is the first parsed cel holding the record.
- `pCelList` is allocated per map file (`parsemap_block_map_header` 0x1d1430), and a by-hash cel's
  record is the one `parsemap_block_entity_params` 0x1d1508 registered under its hash.

Port before: endpoints keyed by bare index across all maps, by-hash cels never keyed, last parsed
cel won, unresolved endpoints dropped. Port now: (map, record) keys, by-hash cels keyed through
`level.by_hash`, first parsed wins, unresolved endpoints keep the previous record's cel.

Disc data [D, all 34 `nf_view::dev::LEVELS` archives, `Level::parse`]:
- Portal blocks exist only in main maps. Every by-hash cel placement's hash exists only in its
  own map. No shared record (several cel placements on one record) carries a portal endpoint.
  So the keying changes alter nothing on the disc; they make the rule exact.
- Only 07000012 has unresolved endpoints: records 35-52 (18 of 53) name record 552 on both sides,
  the "Sky bridge" script player (a 0xd9 placement, no cel bit). The game hangs all 18 quads on
  record 34's pair, `corridors_a_8` (#203) and `Office-C` (#550). The port dropped them; it now
  adds them. The quads form a ring around the origin (x -11..11, z -3..5, y -1..1), away from both
  rooms. No door links to any of them.

## Dynamic objects (`parsemap_create_dynamic_objects` 0x1d04f0)

- The switch is on the full flags word (`addiu v1,a2,-0xf; sltiu v0,v1,0xf0` at 0x1d04f4). No
  non-cel placement on the disc has bits above 0xff, so the port's `flags & 0xff` reads agree.
- Only the main map's placements are created (docs/systems/cover-nodes-load.md, `ResetMap_Load`'s
  single flag-1 walk); sub-map placements never become objects.
- Main-map census, every level, placements with `flags & 0xa000 == 0`, all with a resolved glist
  (0 null-glist placements, so none is skipped by the `iVar5 != 0` gate):
  0f 555, 20 443, 21 33, 22 20, 24 38, 25 252, 26 198, 27 14, 28 29, 29 33, 2a 74, 2b 3, 2c 15,
  2d 18, 2e 24, 2f 5, 30 17, 31 9, 32 129, 33 49, 34 15, 35 7, 38 1, 39 8, 3a 57, 3b 75, 3e 15,
  3f 5, 41 6, 4a 26, 4e 4, 71 15, d2 5, d4 2, d5 1, d8 35, d9 172, da 29, db 443, dc 3, de 54,
  df 17, e1 189, e2 110, e3 2, e4 64, e5 551, e6 351, e7 16, e8 16, e9 129, ea 346, eb 174, ec 46,
  ed 21, ee 5, ef 28, f0 378, f1 79, f2 69, f3 8, f4 1, f5 223, f9 104, fa 44, fb 15, fc 113,
  fd 5, fe 10, ff 4 (ff has no case: nothing is created).
- Each of these kinds has a port module that cites its creator. I did not re-audit each module's
  placement-to-object counts against this census; that remains unproven per kind.

## Visual consequences (not changed here; recorded)

The renderer uses the same graph. Measured on 07000012 with 144 views (9 eyes in each of the two
rooms, 8 yaws): 32 views now mark one more cel visible, always the other room of the pair, reached
through a Sky bridge quad when record 34's own doorway is out of view. That room's World draws
are submitted in those views (more draws, matching the game's walk). Whether any of it shows on
screen (it sits behind walls) is not checked; a `--shot` compare was not run.

## Tests

- New: `crates/nf-view/tests/g016_portal_endpoints_live.rs` (2 tests). Both failed on the old code
  (07000012: 18 records dropped) and pass now.
- Green: load_scene_equality, walkreal_07000012_live (4), walk_07000012_live (4),
  door_portal_links_live (3), door_portal_live (2), portal_live (4), use_key_census_live (20),
  corridor_guards_07000012_live, enemies_07000012_live (3), interactables_07000012_live (12).

## Unproven

- The entity hashtable's result on a duplicate hash insert (`hashtable_additem` 0x1ca060); no cel
  placement on the disc needs it.
- Per-block (not per-map) reset of the endpoint registers: the port resets per map. They are the
  same on the disc because no block's first record misses.
- Whether the extra drawn-gate cel on 07000012 changes any guard's behaviour in play.
