# capsule-cull-and-los-masks — the head-on cull and the per-caller LOS masks, unflagged

## State: in progress — the flags are ON and removed from the code; 4 of 10 flagged-off test groups are
green, the rest are diagnosed or pending (see What's left). Branch tip ac8c16bb has the unconditional
behavior.

## What's done

1. **Both flags are on and then removed** (`crates/nf-game/src/world.rs`): `line_of_sight` now always runs
   `line_of_sight_masked` (the per-caller drop semantics of `Collide_Intersect` 0x1ebaa8:300-319 and
   `Collide_Pick` 0x1eb320:140-152), and `capsule_tri_contact` now always applies the normalized-spine
   head-on cull `(n·d̂ > 0.999 ∧ f > 0.5) ∨ (n·d̂ < −0.999 ∧ f < 0.5)` [M Intersect_CylGeom 0x109da0:302-322].
   The old `|n·y| > 0.7` top-end-side stand-in is deleted.
2. **The world.rs unit tests are un-ignored and green** (8/8 with the flags on): `head_on_cull_uses_the_raw_spine_dot_and_the_crossing_half`,
   `walk_spine_culls_only_near_flat_faces`, `line_of_sight_applies_the_callers_game_mask` — plus the rest of
   the module.
3. **One test-vs-decompile fix** (the LOS unit test): the surface-0x40 card does NOT block the headroom mask
   0xd — the surface byte's 0x40 ORs into `HITDATA+0x51` bit 1 [M 0x1ebaa8:305-309] and mask bit 0x8 drops
   every hit carrying that bit [M 0x1ebaa8:317-319], and 0xd (9|4) carries bit 0x8. The old assert ("no 0x800
   bit — the card blocks") contradicted the decompile; fixed with the addresses and a true contrast (a mask
   with neither 0x800 nor 0x8 blocks the card).
4. **ai_live `guard_takes_cover_like_the_game` green** (36/36 in the file). Diagnosis (the [los] env-gated
   probe, since removed): every flipped ray passed one **surface-0x40 catwalk face** at (62.467, 34.0, 88.96)
   — the guard's 0xa27 vision [M DroneVision_CanSeeObjectFrom 0x176f18:69 passes `param_6 | 0xa27`] drops it
   via the 0x800 bit, so the first sight lands ~60 ticks earlier and the shared RNG stream shifts at the
   first-sight roll. Fix: the test's seed re-picked 2 → 3 (the file's own documented calibration practice —
   the trace pins themselves are unchanged: state order [0x58, 0x5a, 0x8c, 0x93, 0x94, 0x95, 0x96],
   UnderCoverInit 30, UnderCoverAim 34, cover node 1, pos error 0.011 from the live (58.83, 99.32)).
5. **interact_07000006_live green (9/9)**, including the guardhouse E door. Diagnosis: door #47's walkable top
   sits at y 40.008 while its object pos.y (the harness's settle-cast start) is 40.000378 — the cast started
   0.008 UNDER the top, found nothing, and the fallback buried Bond half into the door, which the game's cull
   correctly treats as jump-through-from-below. Fix: `settle_near` casts from 0.05 above the spot — the game
   places objects ON the hit surface plus a normal offset [M Collide_PlaceObject 0x1ecfb8's tail:
   `pos = hit+0x40 + normal·arg`], never half-buried. The test's pins are unchanged.
6. **exit_aim_01_live green** (the 01 stairwell descent) with the cull on — the redo branch's spine-direction
   reading (d̂ = B − A, f from the top end) is confirmed on the real path.
7. **interact_07000009_live** — the body-armour walk: the settle spot y −368.9 sits 0.37 UNDER the walkable
   floor top (−368.53 [D probe]); `settle_near` now casts from +0.5. **Still failing** (13/14 green): Bond
   is placed on the right floor but the take never fires — needs the next window.

## How it was verified

`NF_ISO` set, `cargo test --offline --release -j4`:
- `-p nf-game --lib world::tests` — 8 passed (flags on, then flags removed: same result).
- `-p nf-game --test ai_live` — 36 passed.
- `-p nf-view --test exit_aim_01_live` — 1 passed.
- `-p nf-view --test interact_07000006_live` — 9 passed.
- `-p nf-view --test interact_07000009_live` — 13 passed, 1 failed (the armour take — above).
- The full fast-runner suite (`tools/test/suite.py`) was NOT run (the background runs get killed between
  tool calls in this sandbox and a foreground run does not fit the remaining window).

## What's left

- interact_07000009_live's armour take (placed right, take never fires — the next window prints Bond's path
  and the pickup's take-gate inputs).
- Re-run on the final tree: level_end_live (07000016 straddle door), walkreal_07000008_live (laser burn ch
  134), gun_turrets_live, bathhouse_shout_mixer_live, mp_net_slots_post_move_live, zoe_gondola_realpath_live
  (an ally round aimed at Bond — likely the same surface-0x40 class as ai_live: an ally's aim/vision ray
  passes a 0x800-dropped face and the scenario's RNG stream shifts).
- The full suite on the final tree.

## Live checks for the main agent

1. The 0xa27 vision pass at the 05 catwalk: fresh boot 07000005, Bond at (79.86, 22.92, 96.0), the gate guard
   at (62.4, 35.1, 88.5) — watch `DroneVision_CanSeeObjectFrom`'s sight bit (drone +0x228 & 4) appear within
   the first ~5 ticks (the masks say the 0x40 face at (62.47, 34.0, 88.96) never blocks it).
2. The head-on cull on the guardhouse door: fresh boot 07000006, Bond walking east over door #47's top
   (36.5, 40.0, −16.25) — expected: standing (the plane crosses the spine's bottom half, f > 0.5 from the
   top end, not culled).

## Usage

Token, context-window, cost and credit figures for this session: not visible to me (no such counters are
exposed in this environment; nothing to copy exactly).
