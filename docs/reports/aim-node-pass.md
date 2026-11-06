# aim-node-pass — the node pass rule, and honest fusebox tests

Branch `s/aim-node-pass` = `origin/s/aim-object-flags` af18e350a. Written ~02:50 UTC
2026-11-06, inside the single session window (the hard stop caught the tail — see Left).

## 1. The node-rule hypothesis: FALSIFIED on the disc

The task asked to verify that the 08 shutter leaves' glist nodes carry the 0x40 flag and let
the node rule carry the shutters. **The disc says no** (`nf-formats/tests/shutter_node_probe.rs`,
new):

- `DOOR_Shutter_1` / `DOOR_Shutter_3` (the wood leaves, 262 tris each): surface bytes
  **{3: 52, 4: 64, 10: 146}** — no byte ≥ 0x40.
- `DOOR_Shutter_2` (the glass leaf, 18 tris): **{16: 18}** (mat 0x10, the glass class).
- The glist words: 0x0 (leaves 1/3), 0x201 (leaf 2) — no 0x60 bits [D glist_flags_probe].

So every pass source `Bullet_CollisionHandler`'s walk reads is quiet on the shutters: the
node word (+0x51 bit 1), the surface byte's 0x40/0x80, the glass class 0x10, and
obj+0xF0 & 0x60 [M 0x12cf00-0x12cfd0]. Yet [L] the round passes with the shutter object in
its hit chain. The walk's structure is now decoded [M 0x12d800-0x12d810]: **a passed node
sets the bullet's state 1 (keep flying) and walks on; the first non-passed node exits the
loop with its state-2 verdict** — so the game's pass needs a set bit somewhere this probe
hasn't found. The port's `Door.bullet_pass` therefore KEEPS the placement-word stand-in
(0xdb & 0x60 = 0x40 — the kind-byte accident the previous session identified): replacing it
with the true word stops the round the game passes, and the node rule cannot carry it. The
open question goes to the live checks (below).

## 2. The mask gate and the cel-linked doors: LANDED

Both halves are wired into `Drones::aim_snap` this session:

- The pick gate [M Collide_PickObj 0x1eae80 disasm 0x1eafc-0x1eb2c + the per-cel walk's
  f0 & 0x3000 pre-filter 0x1eb914]: the aim/bullet hittest word 0x20a/0x208 [M
  Player_WeaponInitBullet 0x1a4bf0:154 / Bullet_update 0x12ef18 `t3 = 522`] builds the skip
  mask 0x20 | 0x40 = 0x60; candidates whose glist word (obj+0xF0's source) carries the mask
  or 0x3000 are skipped. [L anchors] the 1b whole panel (f0 0x81) stays, the pieces (0xc1)
  are skipped — the disc words match both.
- The cel-linked doors [L the driving task: the 08 rocket's aim ray hits the class-0x1c
  object at (139.562, 50.792, -96.706)]: each glist-backed door walks the same gate, the
  built-sphere broad phase (`live_sph`), then the leaf's own collision triangles at the
  posed matrix.

**rocket_glass08_live 3/0 and rocket_stop_plane08_live 4/0 — green** (last session's red
was the f0-based `bullet_pass` stopping the flight, not the snap; with the stand-in the
whole chain reproduces the [L]: the snap at the shutter object, the round passing to the
cage wall).

## 3. The 14 fusebox tests: the walk built, the cut still red

`fusebox_geometry_07000014_live` now walks the player's route (the rewrite is committed):

- The stand-in transit to the door stance (the module header's documented wedge covers the
  longer haul), then **the E press through the real `Player_Activate` → `Door_Activate`
  path opens the leaf** [M 0x1a5170 → 0x135be0] — the mechanics that made it work: the
  stance is the model test's `dpos + (0, 0.6, 1.6)` with the aim at `dpos + (0, 1.0, 0)`
  (the same `OBJECT_Door` record interactables_07000014's green test uses), and
  `Sim::step_bond`'s **fourth** argument is the use key (the first is fire — the first
  attempts pressed fire).
- The leaf leaves CLOSED (0x8 OPENING → 0x4 OPEN) and **stands open at the pinned stance**
  (the walk out is quick against the 360-tick auto-close [M 0x13660c-0x136700]).
- The cut itself stays red: the beam bends west-up off the wire slab (the slab's posed
  centre (-89.412, -0.615, 30.895); the round's crossing lands at x ≈ -89.75) — the aim
  snap's posed-glist west shift, the same pivot question the aim-object-stage report
  documented ("the posed collision glist measured ~0.1-0.3 u behind the room's stat wall
  face; the models-table bound sphere poses 10+ u away"). The adaptive aim-by-miss loop
  (the interactables pattern) is in and does not converge past it.

`interactables_07000014_live`'s rewrite was not started (the same blocker; the window
closed). Both keep their assertions; the routes are the only change.

## Runs (each binary whole, --test-threads=1, NF_ISO set)

| binary | result |
|---|---|
| rocket_glass08_live | **ok. 3 passed; 0 failed** |
| rocket_stop_plane08_live | **ok. 4 passed; 0 failed** |
| fusebox_geometry_07000014_live | FAILED. 0 passed; 1 failed (the walk opens the leaf; the cut blocked by the snap shift) |
| solar_panel_hits_live / rocket_origin_live | not re-run in the window — the aim change touches their levels only via the gate (the panels' word 0x81 is gate-clear; the origin test's line has no door/object candidates); the stage-tip numbers stand: 1/0, 1/0 |
| interactables_07000014_live | not re-run in the window; the stage-tip number 9/1 stands (the same cut blocker) |
| walkreal 08 / 0c / 14 / 16 / 1b | not run in the window. The sim delta vs the stage tip is the aim gate + the door stage (the 08 pair re-verified green above); the stage-report walkreal numbers (08 5/1, 0c 2/1/1, 14 6/3/1, 16 13/1/2, 1b 15/2/2) are the baseline the next session must re-quote |

## Live checks for the main agent

1. **The shutter pass bit** (settles the door arm's replacement): during a live 08 straight
   rocket that passes the shutters, record (a) the shutter object's runtime `obj+0xF0`
   (watch writes by `Control_SetGList` 0x134960 and any other writer after the create), and
   (b) the walked HITDATA node's `+0x51` and `+0x50` when the handler passes it [M the walk
   0x12cf00-0x12d810]. Expected: one of them carries a pass bit the disc parse doesn't
   surface (a runtime OR, or a node flag outside the 0x2E surface bytes).
2. **The 14 wire snap** (settles the cut): from the pinned stance, the aim ray's snap point
   and the hit object (gdbwatch on `Collide_Pick` 0x1eb320's object arm). Expected: the
   snap's west shift traces to the wire's posed collision glist or the models-table sphere
   (the pivot question), and the honest fix follows (the gfx frame for the snap, as the
   contacts already ride).
3. **The keyless kind-2 door** (the 14 leaf): whether the game's leaf moves at all (the
   placement has no 0x19 anim keys — the port's leaf never moves either); if the game's
   leaf is fixed geometry, the player's cut line never crosses it and the pinned stance's
   chain needs re-measurement (the beam's honest path to the cable).

## Left for the next stage

- The interactables_07000014 rewrite (the same walk + E pattern; the route code is written
  in the fusebox test to copy).
- The cut's snap fix per live check 2; then both 14 tests should go green with the
  assertions untouched.
- Re-quote the five walkreals + the full suite on the final tree.

## Usage

not visible to me — this environment shows no token, credit, cost or context-window
figures anywhere I can read (no banner, no status line, no API surface exposed to the
worker).
