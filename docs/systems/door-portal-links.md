# door-portal-links — doors open and close their portals (GAPS G-016 / G-035)

The cel-visibility walk honours the game's portal↔door link: a closed door ends its portal, so
the room behind it is not walked into and its World geometry does not draw; an open door's portal
crosses as before. This page documents the link, the walk gate, and the per-level facts the tests
pin. The port is in scene.rs (the door-link portal gate); this
page adds the 07000007 facts and the never-adds/draw-count verification.

## The link [M]

- `Door_Link2Portal` **0x136788** = `obj+0xFD >> 3 & 1` — the door's "straddle" flag, set by
  `Door_Create` 0x136100 when the placement's param 4 != 0 (the port's `Door::straddle`).
- `build_LinkDoors2Portals` **0x1c8ac8** walks the straddle doors and links each to the portal
  quad it sits in: the quad's PLANE within **0.25** of the door's gfx sphere centre
  (`ABS(dot) < 0.25` @0x1c8c34, centre = obj+0x80) AND the centre inside the quad
  (`vecutil_point_on_poly` 0x1ec940 over the quad's two triangles (c0,c1,c2)+(c2,c3,c0)).
  A match writes the door into `portal+0xC` on BOTH halves (first door wins,
  `if (puVar7[3] == 0)`) and sets `obj+0xF0 |= 0x8000`.
- Port: `crates/nf-view/src/scene.rs` `CelGraph::portal_doors` (one `Option<u32>` per edge),
  `link_doors_to_portals`, called at load in the Sim build (scene.rs `nf_view::load` path).

## The walk gate [M]

`Vision_Recurse` **0x1e82e0** (the core `Vision_Portal_Recurse` 0x1e86e8 drives), per portal node
of the eye cel's `cel+0x28` chain:

1. the node's visible byte (+0x27) gates the loop;
2. the eye-side plane test (the `_vmul`/`_vaddbc` dot with `_ViewPoints`): `fStack_f0 < 0.0` = the
   portal faces the eye;
3. **`Door_IsClosed(portal+0xC)`** (`puVar17[3]`) — if it reads closed, the portal is skipped
   (no corner copy, no clip, no destination-cel add);
4. otherwise the four corners go to `PortalsIn` and clip against `NumPortalPlanes[depth]` with
   `ClipPortalWithPlane` 0x1e7fa0; a non-empty polygon marks the portal seen and adds the
   destination cel (`VisCelList`, budget `VisionRecurseCount > 100`, depth > 45).

`Door_IsClosed` **0x1367b0**: obj 0 → false; kind (`obj+0xF6`) == 1 → true (always); group
(record at obj+0xE0, `record+0x6A`) != 0 → `DoorGroupsStates[g] == 2`; else
`obj+0xF4 == 2` EXACTLY — a door mid-OPENING/CLOSING (state 8/0x10) does not block the walk.
Port: `nf_game::door::Door::is_closed` + `Doors::closed_flags()` per tick; the walk entries
`portal_visible_cels_from_cell_doors` / `portal_visible_cels_game_from_cell_doors` /
`portal_cel_chain_from_cell_doors` take `door_closed: Option<&[bool]>` (`None` = the legacy
every-portal-open walk, byte-identical to the pre-link port). The draw path feeds the per-tick
snapshot through `Renderer::cel_visibility`'s `doors` argument.

## Culling scope

The gate only ever ENDS portals in the walk — it can only SHRINK the visible-cel set, never grow
it (sister assertion, every tested eye and direction). What draws is unchanged for characters,
items and doors: the room mask hides a hidden room's linked objects as before (View_AddCels 0x1e6ba0
adds only visible cels' glists + the cel's own objects [M docs/atlas/cel.md]); a portalled door
itself straddles BOTH cels of its portal (`Collide_StraddleCels` 0x1eac98), so the near cel's
visibility keeps it drawn while shut (pinned by door_portal_live.rs's skin-palette assertion).

## 07000007 (the fix-perf level, the party): NO door links a portal

[D level data, parsed by the port's `PortalQuad::parse_block` + `Door_Create` params] — 15 doors,
7 carry placement param 4 (straddle, doors 0–6), 14/15 CLOSED at create (door 7, state 4 = OPEN,
is not straddle-flagged). 34 portal records. **Zero links**: every straddle door fails the game's
own link test (nearest portal-quad plane distance, measured on the port's parse of the disc):

| door | sphere centre | nearest portal | plane dist | link? |
|---|---|---|---|---|
| 0 | (132.63, 44.13, −55.38) | 11 | 0.634 | no (≥ 0.25) |
| 1 | (102.64, 44.13, −55.38) | 7 | 0.637 | no |
| 2 | (94.89, 44.13, −47.88) | 22 | 0.131 | no — within 0.25 of the plane but the quad's centre is (76.75, 45.63, −47.75), ~18 units away: `vecutil_point_on_poly` fails |
| 3 | (80.64, 44.13, −49.87) | 5 | 0.636 | no |
| 4 | (94.84, 50.12, −72.62) | 13 | 0.635 | no |
| 5 | (46.64, 44.13, −44.37) | 28 | 0.637 | no |
| 6 | (132.27, 50.12, −72.63) | 11 | 0.275 | no (≥ 0.25) |

So on 07000007 the gate is a no-op BY THE GAME'S OWN RULE — the walk returns the same set with and
without the door snapshot, and that set is the game's measured one.

## World draw counts (before / after the gate)

World materials with the walk mask (the `world_counts` rule: pass != 4 sphere test + cel mask),
headless at the pinned fixtures:

- **07000007 start view** (`scene.start` + 1.6 eye, start yaw, 60°/16:9): **177** with the legacy
  walk (every portal open — the pre-link port), **177** with the door gate at create, **177**
  all-open. The game measured **177** World draws on this view [L live PCSX2
  measurement recorded in docs/atlas/cel.md — "4526 → 177"]. The
  gate neither adds nor removes a draw here, matching the empty link table above.
- Other 07000007 ground-truth spots: ballroom window (cell 25) 268 → 268; clock hall (cell 33)
  294 → 294; the sister sweep (16 yaws around the start eye, plus both spots) adds **0** cels.
- **07000001** door-0 fixture (eye 2 units off portal 3's centre, cell 7): legacy **511** draws →
  **38** with the gate at create (the shut SlidingDoor's rooms — dest cell 40 and 45 and everything
  reachable only through them — leave the set); all-open **511** = the legacy set per-cell. With
  the three portalled doors OPENED through the real use path (`Player_Activate` 0x1a5170's door
  arm → `Door_Activate` → `Door_Interp`), the gated walk returns the legacy set per-cell again —
  the secretdoor leaves (224/225) gate nothing because the game's own rule links them to no quad.

## Tests

`crates/nf-view/tests/door_portal_links_live.rs` (NF_ISO-gated; numbers/ids only):

1. `the_07000007_start_view_returns_the_games_cel_set` — the empty-link table pins, the gate ==
   legacy == all-open set per-cell, the 177-draw [L] pin, the 16-yaw never-adds sister.
2. `with_the_portalled_doors_open_the_07000001_walk_returns_the_old_set` — shut: dest hidden,
   adds 0 over 16 yaws; the three portalled doors open through the real use path → the gated set
   == the legacy set per-cell (and the draw count returns to the legacy count).
3. `the_walk_never_adds_cels_in_any_direction` — 16 yaws × 3 pitches on both levels' fixtures:
   gated ⊆ legacy always (adds == 0), all-open == legacy per-cell in every direction.

`crates/nf-view/tests/door_portal_live.rs` stays: the 07000001 links, the
shut-door block from both sides, the draw-while-shut pin, the real-use open.
