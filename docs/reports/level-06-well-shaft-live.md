# level-06-well-shaft-live — is the well shaft open below the hanging body?

Branch `s/level-06-well-shaft-live` (from origin/master `3f760e7d`).

## The verdict: the port was right, the test's premise was wrong — re-pinned and un-ignored

`interact_07000006_live::the_well_shaft_is_open_and_the_hurt_volume_is_game_inert` assumed Bond
over the well mouth centre drops into the shaft. He does not — and the game would not let him:
**body #65 at the mouth centre is the game's own solid prop** (a ~0.46×0.92 barrel-shaped body),
and the port's block of it is game-faithful. The second symptom (the −356 fall-through 0.7 off
centre) no longer reproduces on the current tree.

### [D] the disc data

- Body #65: kind-0xD8 placement at (31.82, 39.79, 17.26), type halfword rec+0x34 = **1**, mass
  rec+0x2c, glist **anonymous** (hashcode u32::MAX — the port's shape key is the synthetic
  `0x8000_0000 | map 3 << 16 | object 0x3b`). Its glist carries a **94-triangle collision block**
  (the 0x2E block before #65's gfx [D docs/formats/collision.md]) with extents ±0.233 x,
  −0.923..+0.001 y, ±0.202 z — the "0.2 wide, 0.92 tall" body, pivot at the top (it hangs below
  the mouth lip: top ~39.79).
- The shaft: the mouth hole is open around the barrel — floor probes 0.7 off centre (x or z) hit
  **33.59**, ~6.2 below the mouth [D the shaft's static triangles; 10 floor-ish tris in y
  32.5..35]. The courtyard floor west of the mouth sits at ~39.33, the north rim at ~41.

### [M] the game's pick chain admits the body for the player's walk query

1. `Player_Collision` 0x194df0:88 zeroes the query mask (HITTEST+0x98 = 0) and :205 (walk case)
   sets the query kind 0x800 (BL+0x7d4).
2. `Collide_Pick` 0x1eb320:314-329 walks each candidate cel's linked objects, skipping only on
   HITTEST+0x98 & 0x10 (absent) or obj+0xF0 & 0x30000000 (absent — `RB_Create` 0x1d6bf0 writes
   obj+0xff = class 0x1a and obj+0xfa |= 0x44 only, :49-56).
3. `Collide_PickObj` 0x1eae80:88-90: for a kind ≠ 0x100 query the object needs obj+0xfa & 8 (the
   joint bit — clear for RB bodies, so the `Collide_Jointy` sphere arm at 0x1ebaa8:97-105 is out)
   OR a non-empty registered shape list at glist+8 — #65's 0x2E block gives it geometry.
4. `Collide_Intersect` 0x1ebaa8:89-108 then copies obj+0x90 and runs the narrow phase over the
   glist's triangles; a glist+8 == 0 body returns no contact.

So a kind-0xD8 body with a collision block blocks the player capsule — exactly what the port does
since the shape-key registration ([M RB_Create 0x1d6bf0:81 pins +0x13c/RIGIDBODY plumbing;
`Collide_Pick`'s cel-linked-object walk is the registration consumer]). The [S] link the decompile
export does not show is the loader-side write of glist+8 (the level-load code is not in the
exported set); the observable — every glist with a 0x2E block collides, draw-only glists don't —
is what the port implements, and it is listed as a live check below.

### The port's behaviour today (measured in this sandbox)

- Bond settled over the mouth centre: rests **grounded at y 39.96** (on #65's top), health 100,
  hurt volume #373 never starts (ch 0x62 low).
- 0.7 off centre (x 31.12 or x 32.52, or ±0.7/1.0 in z, or (0.5, 0.5)): **lands grounded at
  y 34.62-34.71** — standing on the shaft floor 6.2 down. **No −356 fall-through at any of the
  seven offsets dropped** (the old symptom predates the shape-key registration; the shaft floor
  triangles ride the same anonymous-shape keys that registration fixed — the well barrel note in
  `rigid_body_wiring_live` records the lamps' zero-collision ancestor bug).
- A real-path walk west across the mouth crosses it (the capsule bridges the ~1 m hole) and the
  walk-real damage came from elsewhere, not the well.

## The re-pin (un-ignored)

`the_well_shaft_is_open_and_the_hurt_volume_is_game_inert` now asserts the game-faithful shape of
the hazard, every assertion carrying its citation:

- over the mouth centre Bond stands ON #65 (y 39.4..40.4) [D the 94-tri block; M 0x1eae80:88-90];
- 0.7 off, the mouth is open: Bond stands grounded on the shaft floor (y 33..36) [D the shaft
  floor at 33.59];
- the hurt volume stays inert: health intact, ch 0x62 low [M Hurt_Update 0x17e5d0's
  `param4 & collected` AND — D #373 params [6, 1000, 0, 0xffff, 1, 0], who-mask 1 vs the
  collected {0x10, 8, 2}].

## Runs (this sandbox; NF_ISO = the slim on-foot image)

- `interact_07000006_live`: **9 passed, 0 failed, 0 ignored** (the well test un-ignored; 9.9 s).
- `interactables_07000006_live`: **5 passed**; `walk_07000006_live`: **2 passed**;
  `walkreal_07000006_live`: **3 passed**; `models_sweep_07000006_live`: **2 passed**;
  `searchlight_bullet_07000006_live`: **2 passed**; `searchlight_flip_07000006_live`: **3 passed**;
  `searchlight_off_07000006_live`: **2 passed**; `census_07000006`: 0/2 ignored (probe binary).
- `rigid_body_draw_live`: **3 passed**; `rigid_body_wiring_live`: **2 passed**.
- `nf-game` lib tests: **631 passed, 0 failed, 5 ignored**.
- Full four-crate suite: **not run to completion in this sandbox — the disk filled** (9.9 GB root:
  the 1038-binary suite build alone needs ~6-7 GB beside the ISO, work/ and the toolchain; the
  first background attempt died at `probe0c_parapet_live`/`walkreal_probe0c_bridge_live` with
  linker `Bus error` at 100 % disk, reproduced clean after freeing space). The touched-surface
  binaries above all pass; the port code is untouched by this task (test-only change), so the
  regression surface is those binaries. A disk-freer sandbox should run the full suite on this
  tree before merge.

## What's left / live checks for the main agent

1. [L] Fresh boot into 07000006 in PCSX2, walk Bond onto the well-mouth barrel (31.82, ~40,
   17.26): expected — he stands on it at ~39.9 (feet), not through it. This settles the one [S]
   link (the loader-side glist+8 registration for kind-0xD8 glists) definitively.
2. [L] Step off the barrel into the mouth: expected — a ~6.2 m drop onto the shaft floor (stand
   ~34.6), survivable, no hurt-volume fire (#373 inert).
3. The full four-crate suite on this tree in a disk-freer sandbox (the totals line this report
   could not produce).

## WORKLOG

`docs/WORKLOG.md` no longer exists on master (dropped in batch-1023c, 3db74285; the repo holds
only what the port uses). This report is the task's record; no WORKLOG file was recreated.

## Usage

Token/credit/context figures: not visible to me (no token, context or cost readout is exposed in
this worker environment). The visible quota-like number is the task clock: sandbox expiry ~09:25
UTC, worker started 07:26 UTC.
