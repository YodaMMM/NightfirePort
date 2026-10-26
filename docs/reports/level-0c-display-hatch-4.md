# level-0c-display-hatch-4 — the 0x800 flag is the cylinder SHAPE word, not the filter flags: the blanket glass body reject reverted

## What was asked

The hatch-3 blanket body reject of mat-0x40 faces (`capsule_tri_contact`:
`if mat & 0x40 != 0 { return false; }`) broke six tests on the full suite
(walkreal_0700001b_live `the_boundary_wall_stops_the_float_and_passes_bullets`,
walkreal_07000001_live, walkreal_07000008_live, walkreal_07000011_live,
fix_npc_doors_live, mp_perf_16_live's digest). Find exactly which player query
carries the 0x800 collide flag and which 0x40 faces the game treats as
body-solid vs walkable-through, port that rule only, and keep all six plus the
0c roof-walk test green.

## The finding: hatch-3 misread the struct — the 0x800 writes are the SHAPE word

- `Collide_Filter` 0x1ed888:7-9 rejects `(surf & 0x40) && (flags & 0x800)` [M]. The
  flags word reaching the filter from `Intersect_CylGeom` 0x109da0:533-540 is
  `*(ushort*)(HITTEST+0x98)` (the decompile's `param_1[9] + 8` is address decay of
  `undefined1(*)[16]`, i.e. +0x90+8), gated on `HITTEST+0x9A & 0x10`.
- `Player_Collision` 0x194df0 writes `BL+0x7d4` = **HITTEST+0x94** = 0x800 in every
  substate (:205/:229/:257/:282/:315/:363/:381) — that is the **shape word**
  (0x800 = cylinder), read by `Collide_Update` 0x1ea398:78 (`+0x94 == 0` → skip the
  query) and the `Collide_Intersect` 0x1ebaa8:181-296 dispatch (0x101 point,
  0x201 ray, 0x400, else `Intersect_CylGeom` at :296). It is NOT the filter flags.
- The filter flags `BL+0x7d8` = HITTEST+0x98 are written **0** for every substate by
  0x194df0:88; only substate 15 (zipline) writes 0x1c0 (:382) — no 0x800 bit either.
  Substate 0xc additionally clears +0x94 to 0 at :308 (no cylinder query at all).
- **[L]** `work/ee_start_05.bin`, player BL 0x16cf0f0 (the ground HITTEST at
  BL+0x740 = 0x16cf830 matches docs/fields/obj-tag-player.md:52's pinned
  0x016cf830): **+0x94 = 0x0800, +0x98 = 0x0000, +0x9A = 0x0000, +0x8C = 0.550**.
  The walk body query carries no 0x800 and its +0x9A gate bit is clear — the filter
  never even runs for the body.
- The zero-G float substate (8/9) takes the `default` arm → +0x98 = 0, same as the
  walk. The feet/ground ray is the query that DOES reject glass:
  `Player_FeetOnPoint` 0x194188:100 passes flags **0xc** to `Collide_RayIntersect`
  — bit 0x8 rejects hits whose surf byte has 0x40/0x80 or whose cel carries the
  glist 0x40 flag (via `Collide_Intersect` :143-145's `HITDATA+0x51 |= 2` and the
  filter's `uVar1 & 2` arm) — you cannot stand on glass. Bullet/sensor rays carry
  the 0x800-family masks (already ported: the segment paths). So: **glass is
  body-solid** (the 0700001b boundary wall stops the float [L]) **and bullet-/
  feet-ray-transparent** (the same test's "passes bullets").

## What landed

1. The blanket `mat & 0x40` reject in `capsule_tri_contact` (nf-game/src/world.rs)
   **reverted**, with the full [M]+[L] evidence in the comment. Nothing else changed.
2. No new rule needed for the 0c roof walk: with the reject gone,
   `from_the_duct_top_the_roof_walk_descends_to_the_balcony` is green — the [L]
   route (spawn (5.8, 18.6, −110.8) south of the pane plane z −110.66, ramp,
   crouch, south walk) never contacts the lantern pane. The hatch-3 freeze at
   (5.75, 18.28, −110.11) belonged to the OLD route it replaced.
3. [D] coll-scan (`nf-extract coll-scan 0700000C.bin 5.75 18.28 −110.5 2`): the
   lantern glass is map 45 obj 89 `room_sk1`, mat 0x40, glist flags 0x0000c047,
   cel placement — the disc data is as hatch-3 read it; no 0x20 cel-skip flag
   (`Collide_Pick` 0x1eb320:337 skips cels with `cel+0x94 & 0x20` for queries
   without +0x98 bit 0x40 — 0xc047 has no 0x20), no placement 0x10 pass-skip.

## How it was verified

- The six previously-broken files, all green with the revert:
  - walkreal_0700001b_live: 8 passed / 0 failed / 1 ignored (pre-existing ignore)
  - walkreal_07000001_live 3/0, walkreal_07000008_live 4/0,
    walkreal_07000011_live 1/0, fix_npc_doors_live 1/0
  - mp_perf_16_live 2/0 (`sixteen_participants_fixed_run_digest_is_pinned` ✓)
- fix_0c_hatch_live: 5 passed / 0 failed (the roof walk, the bay drop, the two
  route sisters, the sweep).
- `cargo test --offline --release -p nf-game --lib -- --test-threads=1` →
  **578 passed / 0 failed** / 5 ignored.
- Not run: the full suite (sandbox background jobs die between tool calls and the
  foreground full suite does not fit the window; suite.out holds only the killed
  runner's startup). The touched crates are covered by the runs above (world.rs is
  nf-game; every red-listed test is nf-view and was run by name). The next session
  should run `NF_ISO=... python3 tools/test/suite.py` once on this tree.

## What's left

- The 0c skylight-lantern [L 2026-10-26] walk-through has no ported mechanism and
  currently needs none (the pinned route doesn't touch the panes). If a future
  route does, the candidates are data-side (the panes' cell pick, or a face
  property the port's parse drops), NOT the body filter — the [L] runtime flags
  and 0x1eb320:337 close the query-side doors.
- The full-suite run on this tree (above).

## Live checks for the main agent

- Optional, only if a future route needs the lantern crossing: fresh boot 0700000c,
  walk Bond against the room_sk1 pane (plane z −110.66, x 4.21..7.33, y 17.21..24.47)
  at roof level and record whether the body is pushed (and which substate/position) —
  plus a watch on BL+0x7d8 (the walk HITTEST+0x98) to confirm 0x0000 in that level
  like ee_start_05 shows.

## Usage

not visible to me
