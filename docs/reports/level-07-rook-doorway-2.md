# level-07-rook-doorway-2 — Rook still freezes in the 07 west doorway: the clamp's wall probe

Branch `s/level-07-rook-doorway-2` off `origin/s/level-07-rook-doorway` + `origin/master`.

## Done

1. **The per-cel bounds test is ported** (`crates/nf-game/src/nav.rs`, `world.rs`):
   `AINetwork_BoundsTest` 0x157d80 now runs the game's exact shape — `build_FindCel(a)` starts a
   `Collide_StraddleCels` 0x1eac98 flood (the port's existing cel-graph walk, driven with the
   game's +1.0 y lift on both endpoints [M 0x157d80:106-107] and the 0x201 ray portal test
   [M :109]), BLOCKED unless the to-cel is among the flooded cels ([M :114-127] — the `param_4`
   membership gate; a NULL from-cel blocks at :112), then walks ONLY the walls whose post cels are
   straddled — the post cel is `build_FindCel` of the post position [M `AIBounds_Link2Cel`
   0x1d2f18:30-34], and `AIBounds_Prepare` 0x1d3008:20-28 chains every edge into its two end
   posts' cels' +0x34 lists, so an edge is tested iff one of its two post cels is in the straddle
   set. `Wall` carries `ca`/`cb`; `bounds_test`/`node_scan_ok` take the world; the flat
   all-walls walk remains as the stand-in for worlds without a cel scope (the tests' unscoped
   worlds) and is marked `[S]` in the doc comment.
2. **The real freeze cause found and fixed**: the clamp was live in the port off a misread.
   Both push-move gates read `+0x4F8` **bit 0** ([M `DroneMove_SetBoundryFlags` 0x14f140:47
   `(x ^ 1) & 1 == 0`; `Drone_Control` 0x139268:80 `uVar2 & 1`]), and `NDrone2_DefaultInit`
   0x14b300:60's composite `+0x4F8 = (x & 0xffea05e2) | 0x801e2` **clears bit 0** (the ~mask
   0x15fa1d carries it) and never re-arms it — a full-decompile scan of every `0x4f8` writer found
   no `| 1` set. The earlier fold-doc "correction" armed "bit 1" (0x801e2's bits are
   1,5,6,7,8,15,23); the doc is struck in `docs/systems/boundary-flags-fold.md`'s port section
   (drone.rs carries the corrected citation at both init sites). `push_move` is now `false`: the
   game's undrawn raw slide is unclamped for default-init drones, and the closed-door stops come
   from the door-node parking (`ReachedDestNode` 0x153358 → `DSTATE_OpenDoor` 0x81, binding
   +0xBA8 while the door works).
   Measured: walker22 stood at his spawn (80.146, −50.459) with `boundary 0x12` (the forward
   probe crossed the east jamb wall #685, cel 8, at 0.006 into the 0.5 probe — the wall IS in the
   straddled set there) and the clamp zeroed the 0.02 slide every tick (travelled 0.00 over 1800
   ticks). With `push_move=false` the same probe geometry is inert, exactly as in the game.
3. **The target test is un-ignored and green**: `walkreal_07000007_live::
   the_mission_walker_completes_his_doorway_leg` passes (3.4 s) — Rook (drone 22, Path_3) crosses
   the west doorway to the far node (74.39, −44.97).

## Verified (what ran)

- `cargo test --offline --release -p nf-view --test walkreal_07000007_live` — **3 passed, 0
  failed** (the full party walk start→exit→07000008 carry, the group-plate zone test, and the
  doorway leg).
- `cargo test --offline --release -p nf-view --test walkreal_07000001_live` — **3 passed, 0
  failed** (the contact walk included).
- `cargo test --offline --release -p nf-view --test npc_doors_live` — **5 passed, 1 FAILED** (see
  Left).
- `cargo test --offline --release -p nf-game --lib nav` — 8 passed (the nav unit tests).
- `cargo build --offline --release` of nf-game / nf-formats / nf-extract: clean.
- **No full-suite total line this session** (this sandbox kills background jobs between tool
  calls; the fast runner could not survive). Foreground slices covered the touched binaries.
  The final tree still needs `python3 tools/test/suite.py` for the total line.

## Left (the next round continues from here)

- **npc_doors_live::the_01_escort_driven_through_the_shoji_opens_it_and_crosses is red**: the
  0x63-driven Mayhew crosses the CLOSED, never-activated shoji (closest approach 1.33, door state
  stays 0x2). The clamp was the substituted stopper; removing it (correctly, per the decompile)
   exposed that the port lacks the game's real stop for a route-driven drone at a closed door.
  Diagnosed by elimination, with addresses:
  - NOT the route build's trims: `AINetwork_OptimiseRoute` 0x156760 tests only the FIRST hop
    (start → node[1], [M :59-67]) and the LAST hop (dest → node[count−1], [M :107-116]) — no
    mid-route link MoveTest. The A* itself (0x1571b0) has no MoveTest at all; the direct-check
    MoveTest in `AINetwork_CalcRoute` 0x156ba8:43 runs with the proximity carve-outs (arg 1,1).
  - NOT the LinkCreep_Handler post probe: its radius is 0.2 ([M 0x3e4ccccd at 0x15a2c8:73]) and
    the port implements it (`route.follow`'s `bounds_node_test(0.2, …)`).
  - Remaining candidate: **the drone body vs the closed door leaf's world collision**
    (`NDrone2_Collision` 0x140e90 — the game pushes the drone out of cel-linked objects, the
    closed leaf among them). The port's drone consults only `nav.push_out` (the AI boundary push,
    which skips 0x800-flagged walls) — the doors ARE in `set_dynamic` (drone.rs:16281), but the
    walk's step application (`move_by` 3209 / the raw slide) appears not to run a capsule push
    against the dynamic geo. Trace the port's drone-vs-dynamic-leaf push on the shoji approach
    (the leaf's posed matrix is in the part list; `capsule_push_scoped` reads the dynamic geo) —
    the game's Mayhew stalls on the leaf, the port's walks through.
- The full suite on the final tree for the total line (see above).
- Live checks for the main agent (would settle the Left item fast):
  1. `tools/trace/gdbwatch.py` on the shoji door object's collision fields during the 01 escort:
     whether the game's driven Mayhew ever penetrates the leaf (expect: no — the push holds him
     ~0.4 out), and on `drone+0x4F8` bit 0 for the escort drone (expect 0 — confirms the clamp
     gate never arms).
  2. A render probe at `--bond 6.7,6.3,12.4,0,0` (the 01 courtyard, facing the shoji): the closed
     shoji leaf fills the doorway; Mayhew stopped ~0.4 south of it.

## Usage

not visible to me (no token/context/cost figures are shown in this environment).
