# level-0a-band-settle-3 — the radial-dir arm landed with the [L] running-sum ref; the accumulator pinned by a live test; the slide mechanism remains open (the [M] chain and the [L] pushes are mutually exclusive at the tail)

## What was asked

Start from `origin/s/level-0a-band-settle-2` + the driver's two live reads [L 2026-10-26]:
port the ref as the running sum of pushes (the add-back), land the radial-dir rule, drop
NF_WALK_KILL if the kill becomes unnecessary (or fix its 01 regression), un-ignore the three
`walkreal_0700000a` tests with their [L] pins, keep every other walkreal test green.

## What the live reads pin [L 2026-10-26], and what the code says [M]

- **HITTEST+0x50 (BL+0x790) is the lifetime push accumulator**: boots (0,0,0) [L
  ee_start_0700000a], never reseeded on a walk, EXACTLY constant on the free hall walk
  (frames 20-60+), grows by EXACTLY the per-tick push under the wall press (0.27723 →
  3.27709, +0.1 per contact tick, y 0.08088 / z −22.77985 unchanged), carries over a reload.
- **HITTEST+0x10 (BL+0x750) = (0.1, 0, 0) per contact tick** at the west hall wall; the first
  touch 0.01056; Bond's x alternates −45.856/−45.956 — the push = the walk step along the wall
  normal (the depenetration).
- The tail's ref is the per-object stack copy's +0x50 [M 0x10a540 `lqc2 $q1, 0x50($s5)`], and
  the copy's +0x50 IS the world accumulator's object-local image — proven at the metal this
  session [M disasm]: the copy base is `$sp+0x50` (the amemcpy 0xa0 at 0x1ebc84-94), the
  `RotPreTransVec` out for HITTEST+0x50 lands at `$sp+0xa0` = copy+0x50 (0x1ebcbc-cc4), and
  `RotTransMatrix`+`Mat_Inverse`+`RotPreTransVec` = Rᵀ(world − t) — a RIGID transform
  (0x1e2478/0x1e3578/0x1e49e8 decompiles; no scale).
- **The contradiction, now sharp**: a rigid transform preserves distances, and the [L] walked
  accumulator sits 18-63 m from every contacted q (the fresh walk: 18 m at the jamb; the wall
  press: 63 m) — so the radial tail's `|v| < radius` gate [M 0x109da0:447] can NEVER pass on a
  walked line, and its `dot(v, n) > 0` gate [M :421-431] also blocks every wall contact (v
  points into the face). The radial tail is behaviourally INERT on walked lines; the [L]
  pushes therefore come from a mechanism outside the decoded tail (the only `_DeltaP` writer
  the decompile shows). The port's spine-depenetration rule reproduces the [L] push shape
  exactly (r − dist = the penetration: 0.1 at the held press, 0.01056-style at a soft first
  touch), so that rule stays as the measured stand-in, marked [S] for the game address.

## What landed

1. **The radial-dir arm, landed [M]-faithfully** [crates/nf-game/src/world.rs
   `capsule_tri_contact`]: `v = q − ref_v`, gates `dot(v, n) > 0` and `|v| < radius`, push =
   `normalize(v)·(radius − |v|)` [M Intersect_CylGeom 0x109da0:421-453]; when the gates fail
   the port's spine-dir depenetration applies (the [L]-matching wall-step behaviour). `ref_v`
   = the caller's HITTEST+0x50 accumulator, plumbed through `capsule_push_scoped`,
   `capsule_push`, `capsule_pick_contact_scoped`, `capsule_pick_contact` from
   `Bond.pick_ref` (the walk, the climb and the redo paths; the world.rs unit tests and the
   two nf-view examples pass a far ref, which keeps the radial gates closed and the old
   measured behaviour byte-identical). The battery confirms the arm is inert on walked lines:
   every previously-green number is unchanged.
2. **The ref = the running sum, pinned by a live test** [`walkreal_0700000a_live::
   the_pick_ref_accumulator_tracks_the_game_ref`, ACTIVE]: boots (0,0,0) [L]; holds on the
   free hall walk (x EXACT; y/z < 1e-3 — the port's skin-arithmetic wobble measured at 5e-11 /
   3e-5, three decades under the push scale, vs the game's exact VU0 equilibrium); the wall
   press accumulates ≥ 1.5 over the held window at the +0.1-per-contact-tick rate with y/z
   still [L 2026-10-26]. The port rides the hall skin at z 16.8288 — the [L] line exactly.
   `pick_ref` is never reseeded mid-walk (the only writer is the add-back `pick_ref += push`
   [M 0x1ebaa8:353-373]); a full Sim rebuild zeroes it (the fresh-boot state [L]).
3. **NF_WALK_KILL stays env-gated OFF** (default): the kill is still the only measured
   crossing mechanism for the corner (870 ticks with it, a wedge without — band-settle-2), so
   it is NOT unnecessary; the 01 `walk_from_the_spawn_to_the_exit_with_the_contact` regression
   (the AND-touch's plate approach shift) needs the live re-verification the budget didn't
   allow. Untouched this round.

## The three tests: still `#[ignore]`, with the reason sharpened

`the_west_hall_glide_height_matches_the_game`, `the_corner_crosses_like_the_game` and
`walk_office_floor_qworm_keyboard_hatch` stay ignored — their [L] pins need the jamb/corner
SLIDE, and the slide mechanism is not derivable from the [M] chain: the radial tail with the
TRUE ref is provably inert on walked lines (both gates, above), and the two radial ports with
a synthetic ref were measured-and-reverted in band-settle-2 (the up-lift to 1.7773 / the
re-wedge). The driver's [L] data kills the bottom-ref guess (the accumulator's y 0.045 ≪ the
bottom's 1.23 — no lift) but provides no magnitude rule that yields the [L] 0.02-0.1 pushes
from an 18 m ref. Measured this session: the trio's failure positions are byte-identical to
the pre-landing baseline (the jamb wedge at (−9.15, 1.09, 15.92), the corner crossing over
budget). The ignore notes still describe the state accurately; the pins are correct [L].

## Live checks for the main agent

Fresh boot of 0700000a, the west hall:

1. **Name the slide's writer**: watch the WRITES to HITTEST+0x10 (BL+0x750) while Bond walks
   west from (−8.0, 1.12, 16.29) through the doorway jamb (the game slides to z 16.83): which
   PCs write the per-tick push, and what VALUE lands per tick (expect ~0.02-0.06 tangential)?
   The decompile shows only the radial tail (0x109da0:448-457) feeding the add-back — if the
   writing PC is NOT in 0x109da0's push block, the decode missed a writer.
2. **The tail's live ref**: at the same jamb pass, dump the stack copy's +0x50 (the value
   `lqc2 $q1, 0x50($s5)` loads at 0x10a540) for 3 consecutive contact ticks. If it reads the
   world accumulator's local image (18 m out), the tail is confirmed inert and the slide
   writer is elsewhere; if it reads something near the capsule, the 0x1ebcc4 output mapping is
   wrong somewhere and the radial arm should be re-pointed.
3. **The 01 plate walk** (for the kill's regression): with the below-B kill forced on, walk
   `walk_from_the_spawn_to_the_exit_with_the_contact`'s route in PCSX2 and note whether the
   AND-touch plate's approach differs from the kill-off line (the port's shift suspect).

## What I ran

- `cargo test --offline --release -j2 -p nf-view --no-run` + the three walkreal binaries with
  `--test-threads=1`: walkreal_0700000a_live **5 passed / 0 failed / 3 ignored** (the 5th = the
  new accumulator test); walkreal_07000001_live **3 / 0**; walkreal_07000005_live **8 / 0**.
- `cargo test --offline --release -j2 -p nf-game -- --test-threads=1`: **925 passed / 0
  failed** (5 ignored) — includes the world.rs unit tests whose call sites the ref plumbing
  touched.
- `cargo test --offline --release -j2 -p nf-formats -p nf-audio -- --test-threads=1`: **133
  passed / 0 failed**.
- The 15 nf-view test files the ref plumbing touched mechanically (the far-ref call sites):
  all their binaries run green — fix_08_collision_live 4/0, fix_08_collision2_live 2/0,
  fix_objects2_live 6/0, probe_objects2 5/0, walkreal_probe81_live 1/0, probe979_band_hang
  0/2i, probe14_lid 1/0, probe1b_wall 0/1i, walkreal_probe2_live 5/0, walkreal_probe_door
  1/0, probe14_mouth3 0/7i, probe14_ghost 0/1i, walkreal_probe_fb5 1/0, walkreal_probe_c 5/0,
  walkreal_probe15 0/5i.
- The fast suite (`tools/test/suite.py`): the background start dies in this sandbox between
  tool calls, so it ran in the foreground in 590 s chunks on the final tree (the 838 binaries
  build in 2 s warm). Two chunks completed **132 binaries with no failures**; the only FAILED
  lines are `fix_0c_pylon_live`'s two pixel tests, which panic at the wgpu
  `request_adapter().expect("adapter")` — **this sandbox has no GPU** (the driver's lesson 6);
  they are environmental, not behavioural (the binary runs are the proof: the same binary
  without a display adapter cannot render). The chunk window ended before the full 838; the
  slow walkreal binaries that dominate the tail are covered by the direct runs above.

## Suite result

No suite total line fits this sandbox's 10-minute tool-call window (the run needs ~13 min of
execution after the build; three 590 s chunks reached 132 binaries, 0 failures, the only
FAILED lines the no-GPU `fix_0c_pylon_live` pair). The four crates' raw totals above are the
complete-crate verification record.

## Usage

not visible to me — this sandbox exposes no token, credit or context-window figures to the session.
