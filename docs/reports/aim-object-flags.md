# aim-object-flags — the real object flag word

Branch `s/aim-object-flags` = `origin/s/aim-object-stage` 504ccdf75 + `origin/master`
merged (8141c3c…). Written ~01:50 UTC 2026-11-06, inside the single session window.

## What is sourced now ([M], committed)

- **The flag word's disc source.** `parsemap_block_entity_params` 0x1d1508 stores the
  entity-params payload **+4** dword at the runtime celglist's **+0x44**
  (`pCurrCelList[0x11] = *(int *)(FileNextBlock + 8)`), and `Control_SetGList` 0x134960:11
  copies `glist+0x44` into `obj+0xF0` (`| old & 0x30008000`). The port's `MapObject.flags`
  IS that word (level.rs:363, `p.u32(4)`). `celglist+0x08` is the collision surface-byte
  pointer — 0 when the object has no 0x2E block — which is `Collide_PickObj` 0x1eae80's
  glist requirement (`obj+0xD8 && *(obj+0xD8+8)`).
- **`Door.f0`** carries the word per door (the `Doors::create` object closure returns
  `(glist, centre, radius, o.flags)`). Doors have no CreateObjEx create param
  (`Door_Create` 0x136100 uses plain `control_create_object` + `Control_SetGList`; the
  object is memset 0 first), so door obj+0xF0 = glist+0x44 exactly.
- **`world.glist_flags(hash)`** — the word table on `CollisionWorld`, keyed like `shapes`
  (real hashcodes + the anonymous `(map,object)` synthetic keys).
- **The probe** `nf-formats/tests/glist_flags_probe.rs` pins the disc words [D]: the 1b
  whole panel `OBJECT_SolarPanelWhole` 0x2000a42 word **0x81** and the ten panel pieces
  `OBJECT_SolarPanel1-10` word **0xc1** — both match the [L] 0700001b audits verbatim
  (panels f0 0x81, shutter pieces 0xc1), which settles `MapObject.flags == live obj+0xF0`
  for those objects. Debris 0xa8a0 is runtime-created (Destroy_Smash's CreateObjEx param
  0xa000 over the disc's 0x8a0 — the 14 word census has one 0x8a0 object) — not a disc
  placement word. Every 07000014 door placement's own word is the bare kind byte 0xdb, and
  the 14 doors resolve the anonymous `OBJECT_Door` (word 0x0, bound r 0.4937).
- [L ee_start_05.bin, this session] all five doors boot with obj+0xF0 = 0x0, matching
  their glist words; no door-path writer touches the word afterwards (Door_Update reads it
  only).

## What is wired and what deliberately is not

- **Wired**: `Door.f0` + the word's plumbing + the `glist_flags` table + the probe. These
  are additive — the sim's behavior is byte-identical to the stage tip (confirmed below).
- **NOT wired (the blocker analysis)**: `Collide_PickObj`'s mask gate in the aim stage and
  the cel-linked doors in the aim ray. Both halves were implemented and measured this
  session; **either one alone breaks the [L]-pinned 08 lines**:
  - The mask gate (0x20 base | 0x40 by hittest bit 0x8 = 0x60 for the aim/bullet hittest
    0x20a — both call sites build it [M Player_WeaponInitBullet 0x1a4bf0:154
    `uVar11 = 0x20a/0x208`; Bullet_update disasm 0x12ef18 `t3 = 522`], plus the per-cel
    walk's `obj+0xF0 & 0x3000` pre-filter [M 0x1eb914]) correctly skips the 08 corridor's
    script-entity pickups (words 0x8c0/0x8a0, & 0x60 = 0x20) — and the straight shot's
    snap then lands on the cage back wall, whose muzzle-to-snap line crosses the shutter
    WOOD band → the round stops at −96.706 → the [L] −100 pierce fails
    (rocket_stop_plane08_live 2/2, rocket_glass08_live 2/1).
  - Adding the cel-linked doors (the game's per-cel object walk 0x1eb8e8-0x1eb93c +
    the ForcedList second walk 0x1eba44-0x1eba5c, the built-sphere broad phase then the
    posed-glist triangle refine — the sphere node is broad phase only, +0xC = 1e8 nodes
    are freed [M 0x1eb1b4 + RayIntersect's 1e8 drop], matching the [L] "no hit entry
    without a triangle touch") does NOT restore the pierce either: the default aim ray's
    own x crosses the wood, so the game-honest snap would stop there too.
  - **The remaining divergence is the snap's exact candidate/position chain** (which node
    wins, and what +0x40 the winning node carries) — that needs the live debugger, see
    the checks below. Both halves are written and can be landed once the snap is settled.
- **`Door.bullet_pass` stays the placement-word proxy** — and the session proved WHY it
  ever worked: `p.flags & 0x60` reads the KIND BYTE, and 0xdb & 0x60 = **0x40** — every
  door passes by accident of the kind byte overlapping the mask. The true glist word for
  the 08 shutters (0x0 / 0x201) has NO 0x60 bit, so wiring the true word stops the round
  the game passes [L the live pass shot]. The game's shutter pass must ride the **node**
  rule instead — HITDATA+0x51 bit 1, the glist node 0x40 flag [M 0x12cea0-0x12cea8], which
  the port's flush already feeds as `node_pass` (`penetration_roll_node`) — so the next
  stage is: verify the 08 shutter leaves' node flags carry 0x40 in the port's shape data,
  then replace the proxy with the true word and let the node rule carry the shutters.

## The 14 blocker, fully measured (the fusebox/interactables reds)

The near leaf (door list 51, placement 384, `OBJECT_Door`) sits in the beam's own cel
(43) and stops the laser round on its posed triangles at t 0.383. Its BUILT SPHERE
(centre (−89.338, −0.681, 31.023), r 0.4937 — the glist bound through the closed matrix)
also catches the beam (sqDist 0.0147 < r² 0.2437), so **the game's own pick would stop a
closed-door shot too** — the disc word (0x0) has no pass bits and no door-path writer adds
any. The game-real flow opens the leaf (the walk-in proximity keeps it open; the posed
sphere swings clear with the matrix). The two 14 tests teleport in, so the leaf is closed.
Fixes, in game-honest order: (a) the tests walk the real route (lesson 8's no-teleport
rule) so the leaf stands open; (b) the live check below settles the door's runtime pose
and f0 during the real cut.

## Runs (each binary whole, --test-threads=1, NF_ISO set, this tree)

| binary | result | vs the stage tip |
|---|---|---|
| rocket_glass08_live | ok. 3 passed; 0 failed | unchanged |
| rocket_stop_plane08_live | ok. 4 passed; 0 failed | unchanged |
| rocket_origin_live | ok. 1 passed; 0 failed | unchanged |
| solar_panel_hits_live | ok. 1 passed; 0 failed | unchanged |
| fusebox_geometry_07000014_live | FAILED. 0 passed; 1 failed | unchanged (the closed-leaf analysis above) |
| interactables_07000014_live | FAILED. 9 passed; 1 failed | unchanged (same blocker) |
| walkreal_07000008_live | FAILED. 5 passed; 1 failed | unchanged (the stage report's number) |

The other walkreals (0c, 14, 16, 1b) did not fit the window: the branch's sim behavior is
byte-identical to the stage tip (the only sim-visible delta is the additive `f0` field;
`bullet_pass` evaluates to the same value for every door — the proxy line is restored),
so their stage-report numbers (0c 2/1/1, 14 6/3/1, 16 13/1/2, 1b 15/2/2) stand. The full
suite did not fit either; the changed-code set (door.rs/world.rs/drone.rs closures) is
covered by the six binaries above plus the walkreal spot check.

## Live checks

1. **The 08 snap chain** (settles the aim-gate + door-stage landing): with
   gdbwatch on `Collide_PickObj` 0x1eae80 and the aim ray from the gate-0 start
   (default aim, no input), record (a) which objects enter per straddle cel (the cel+8
   chains) and via the ForcedList, (b) each candidate's obj+0xF0 and the sphere
   broad-phase verdict, (c) the surviving node's HITDATA+0x40/+0x54/+0x58/+0xC. Expected:
   the winning snap node for the straight shot and what position it carries — that is the
   number the port's snap must reproduce for the [L] −100 pierce to hold under the mask.
2. **The 14 cut's door state**: during a real laser cut of the east-wing fusebox wire,
   record the near leaf's (pl 384) obj+0xF4 (state) and obj+0x90 (the pose), and whether
   `Collide_PickObj` ever tests its object during the cut's ticks. Expected: the leaf
   OPEN (the proximity held), the sphere swung out of the ray.
3. **The 08 shutter node flags**: the shutter leaves' glist node words (the collision
   records' per-node flags) — expect bit 0x40 set on the leaves' nodes, which is the game's
   pass for the rocket hit chain (HITDATA+0x51 bit 1) and the replacement for the port's
   kind-byte-accident proxy.

## Left for the next stage

- Land the mask gate + the cel-linked door stage together, against the live snap numbers
  from check 1; re-derive the 08 steer test's calibrated offset from the corrected snap.
- Replace the `Door.bullet_pass` proxy with the true word once the node-0x40 rule carries
  the shutters (check 3), keeping rocket_stop_plane08/glass08 green.
- Make the two 14 tests walk the real route (the leaf open) or take check 2's answer.

## Usage

not visible to me — this environment shows no token, credit, cost or context-window
figures anywhere I can read (no banner, no status line, no API surface exposed to the
worker).
