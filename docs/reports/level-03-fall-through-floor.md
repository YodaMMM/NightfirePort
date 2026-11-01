# level-03-fall-through-floor — done

Branch `s/level-03-fall-through-floor` (from origin/master 3f760e7d). Task: on 07000003 Bond fell
through the world (y −5577) while a walk flanked servant 1's guard in the courtyard
[origin/s/stealth-los-2's first suite run]; reproduce on real input, find the collision gap, fix it
with the address cited, pin Bond on the floor, run every *07000003* binary + the 0a/01 walks +
nf-game.

## The fix: the capsule pick bit now grounds, exactly as the game's ground flag does

**[M Player_FeetOnPoint 0x194188:176-185]** — the game's ground flag is
`obj_data+0x60 |= 8` when EITHER `HITTEST+0x9c != 0` (the capsule pick bit, set by Intersect_CylGeom's
pick-contact block [M 0x10a5e0-0x10a734]) OR the feet ray found a floor in band [M :106-109]. The
port measured the pick bit but did NOT gate `grounded` (the hatch-crouch-0c4 divergence,
player.rs `grounded = walkable || band`). One line changed in `crates/nf-game/src/player.rs`:

```rust
self.grounded = walkable || band || pick;
```

**The case that settles it (reproduced on real input):** walking west from mid courtyard and
pressing into the courtyard's west lip at (12.61, −19.69), Bond hung UNGROUNDED — held up by the
lip's capsule push while the ray band missed by 0.025 u (floor 1.16 below the root; cc+0.1 =
1.1347) and no walkable contact qualified (capsule bottom 0.13 above the floor). With the pick arm
landed the same press grounds for every tick. That ungrounded-hover state is the fall state: the
same flank's body-block carries Bond airborne onto the wall at (6.36, −17.73) [c118ed46: "climbs
to y 7.3, airborne"; 00310262: "parks Bond west of a wall at (6.4, −17.7)"], where the old rule let
him drop through — the stealth-los-2 run's y −5577.

## The geometry (the gap the fall uses) [D 07000003.bin block 0x2E]

A through-world void slot **x 8.0..8.9, z −19.5..−16.0**: the flat feet-ray grid (the port's
Intersect_RayGeom path, from y 8 down to −120) finds NOTHING under those columns, and the raw tri
dump (every tri whose box meets x 7.6..9.4, z −20.2..−15.4, y 3..8) shows no floor face at all
there — only y 7.6+ fence tris. The slot sits between the west ground (floor 4.4 at x ≤ 8.0) and
the courtyard plateau (floor 5.34 at x ≥ 8.94), right off the courtyard's west edge. A walker
pushed over/along the wall crosses the slot airborne; with no floor at any depth, the old
ungrounded state fell to −5577. The game bridges ~1 u seams with the capsule's rim-edge contacts
(r 0.55 [M Player_Collision 0x194df0] spans 1.1 u; Dist2Tri flags 1..7 edge regions [M 0x10a324 /
0x10a528]) — the pick-bit fix restores exactly that support for the lip-pressed states.

Also ruled out: the Dist2Tri flags-0/99 skip [M 0x10a330] — the inverted-winding floor candidate
was checked (raw winding vs stored plane normal dumped for the hole box: 1 OPPOSED tri, a y 7.69
fence piece, mat 0x05); the void is genuinely absent geometry, not a cull artifact.

## What was pinned (tests on the branch)

- `crates/nf-view/tests/flank_fall_repro.rs` — `the_old_courtyard_flank_stays_on_the_floor`
  (master's own flank legs, node by node; the fall must never open) and the sister seam probe
  `pressed_against_the_west_lip_bond_stays_grounded` (the lip press: 0 ungrounded ticks, the press
  must reach west of x 12.8). Both green with the fix; the seam probe hangs ungrounded without it.
- `walkreal_07000003_live` still carried master's un-retraced flank leg (blob 61bb92a2 =
  00310262^ — the driver's "route no longer passes there" is not what master shows). With the fix
  the leg wedges at the wall — the game's own park spot — instead of falling; adopted
  **00310262's route** (retrace the opening legs + the 0x25 doorway fallback; that commit's own
  note: "Every assertion is unchanged"), citing it in the test. No assertion weakened.
- Probes (assertion-free, `--ignored`): `courtyard_wall_probe` (floor/vert tri dump + the lip
  walk), `feet_ray_hole_map` (the flat down-ray grid), `hole_tris_probe` (raw winding vs plane),
  `slot_cross_walk` (the slot approach). They document the seam for the next pass.

## What was run (totals)

All on the branch tree with the fix, `--test-threads=1`, the slim on-foot ISO:

- `walkreal_07000003_live` **4 passed / 0 failed** (227.7 s)
- `walk_07000003_live` **2 / 0** (1.8 s)
- `interactables_07000003_live` **8 / 0 / 1 ignored** (82.7 s)
- `wiring_07000003_live` **4 / 0** (2.5 s)
- `walkreal_0700000a_live` **7 / 0 / 2 ignored** (22.2 s)
- `walkreal_07000001_live` **5 / 0** (187.1 s)
- `flank_fall_repro` **2 / 0** (26.4 s)
- `nf_game` (lib test) **631 / 0 / 5 ignored** (1.3 s) — green with the pick-gate change.

The pick-gate change relinks every nf-view test binary; the required set above plus the probes was
what fit the window. The full fast-suite pass did not fit this sandbox (the same 2 constraints as
the previous session: background processes are reaped between tool calls, and the 9.9G disk filled
during the ~1039-binary link phase — cleaned twice). Run `tools/test/suite.py` on a full-fit
machine for the complete total.

## What's left

- The suite total for all ~1039 binaries on a machine that fits them.
- The slot itself remains open disc geometry (no floor at any depth under x 8.0..8.9, z
  −19.5..−16.0); the port now survives it the way the game's own support does. If the driver wants
  the seam hardened further, the next step is a walk that crosses the slot rim-to-rim with the
  captor body-blocking — the slot_cross_walk probe is the starting point.

## Live checks for the main agent

1. gdbwatch on `BLData+0x60` (bit 3, the ground flag) while pressing Bond against the courtyard's
   west lip from mid courtyard (walk west at z ≈ −18 until he stops): expect bit 3 SET throughout
   the press (the pick bit path [M 0x194188:176-185]) — master-before-fix showed the port hanging
   ungrounded there.
2. fieldprobe Bond's y while he is airborne over the wall at (6.36, −17.73) (the flank's park
   spot): expect the game to bring him down grounded (on the wall top or the west floor 4.4), not
   into an unbounded fall; the port with the fix grounds on the lip.

## Usage

not visible to me — this CLI session exposes no token, cost or context-window readout; nothing to
copy. The observed limits were the sandbox clock (started 07:20 UTC, work stopped 09:05 UTC ahead
of the 08:59 cutoff — the verification runs above ran past it as the finish work) and the 9.9G
disk.
