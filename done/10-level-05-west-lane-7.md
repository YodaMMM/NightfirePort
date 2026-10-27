## Your task: `level-05-west-lane-7`, 07000005: past the step shelf's kink, then the notch grab and the wire ride

Master now walks the west lane green through the band gap (the band's south-west continuation, the z 44.5-45 corridor, the x -20 seam, the cable ledge and the step's spine start: `walkreal_07000005_live::the_cable_legs_the_gap_and_the_ledge_step`). The one blocker is the step shelf's kink at about (-23.5, 33, 46.5): `Room_Outerwall_Corner` (instance 13) carries a 45-degree ramp (33.0 -> 34.0, base line (-23.39, 47.09) -> (-26.03, 44.44), normal (-0.52, 0.68, 0.52)) across the spine, and every walk and hop so far lands on the y-29 roof west of the ramp base. Read `docs/reports/level-05-west-lane-6.md` on `origin/s/level-05-west-lane-6` first (merged to master; the report itself is not on master) and the probe `crates/nf-view/examples/probe_gap_05.rs`.

1. **The kink.** Find how the game's player gets past it [M]: does `Player_FeetOnPoint` 0x194188 / `Player_Collision` 0x194df0 ground the pick on a 45-degree ramp (the slope limit, the step-up), does the capsule push slide along it, or does the route leave the spine earlier (round the ramp from its east, a lower shelf)? Dump the ramp's triangles and the capsule's contacts along the spine. Then walk it with real input only.
2. Un-ignore `the_notch_grab_and_ride_parked` and `the_notch_holds_the_walk_and_the_grab_needs_its_zone`; the grab and ride legs are written and height-checked.
3. If time remains: the south ledge after the ride, the window-ledge crawl #35 (-34.10, 37.28, -31.65) and the exit door #112.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-05-west-lane-7`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.



---
claimed-by: sandbox1 at 2026-10-24T08:22Z

---
## Report (done, 2026-10-24T09:52Z)

Closed by the driver: the report is on `s/level-05-west-lane-7` (docs/reports/level-05-west-lane-7.md); the worker didn't run finish.sh.

---
## Report from the worker (done, 2026-10-24T09:52Z)


---
## Report from the worker (done, 2026-10-24T09:52Z)

# level-05-west-lane-7 — 07000005: the kink probed to the tri; the game's 45°-ramp traversal named from the decompile

Branch `s/level-05-west-lane-7` (from origin/master 26543436). The sandbox was reset at round
start — fresh clone, fresh work-slim + ISO, fresh toolchain; setup consumed the first ~25 min.

## The kink is now a triangle-level answer

New scratch probe `crates/nf-view/examples/probe_kink_05.rs` (committed; scratch print, run with
`NF_ISO` set) dumps three things for the step shelf's z-46.0 kink at (−23.5, 33, 46.5):

- **The kink box tri dump** (x −28..−19, z 41..49.5, y 30..36, over `level.instances(true)` —
  the world's exact static set) [D]: `Room_Outerwall_Corner` (inst 13) contributes **30 vertical
  wall tris** (normal.y = 0, mat 0x0a, spanning y 29.0→33.0) whose top edge runs exactly
  (−26.027, 33.0, 44.441) → (−23.393, 33.0, 47.090) — **the ramp base line is the wall's top
  edge**. The 45° ramp itself rides on top of the wall: **mat 0x0c**, normals
  (−0.520, 0.680, 0.517) / (−0.516, 0.684, 0.516), rising south-east 33.0→34.0, with sloped
  neighbours n (−0.693, 0.693, 0.196) and (−0.184, 0.705, 0.686); `Room_Outerwall_B2` (inst 12)
  has the facing slope n (−0.698, 0.698, −0.162). There is no separate "wall tri at the stall" —
  the wall is the whole west flank under the ramp.
- **The fine floor profile** (x step 0.25, z step 0.5) [D]: at the kink the ONLY floor is the
  ramp strip itself — z 46.0: x −24.0..−23.0 at 33.2-33.9, then **void east** (the cable ledge
  exists only at z ≥ 46.5); west of x −24.25 the y-28.8-29.0 roof. The walked spine 1-3 (green)
  already rides the ramp's upper region; the spine parallels the ramp's strike line.
- **The stall reproduced** [L the probe run]: from the walked spine-3 end, steering toward
  spine 4 gives up at the 3000-tick cap with Bond fallen to the y-29 roof
  (−24.48, 30.08, 46.78, floor_below 28.96; a later attempt walks the roof east to
  (−19.96, 30.33, 48.55)) — the round-6 symptom, now with the walk capsule recorded
  (cc 1.035; walk capsule A = pos+0.275, B = pos−0.485, r 0.55).

## How the game's player traverses a 45° ramp — [M], with addresses

The driver's three options are settled by the decompile (option b + a; NOT grounding via the
pick bit, NOT avoidance):

1. **The capsule push is slope-blind.** `Intersect_CylGeom` 0x109da0 pushes whenever
   dist < radius (gated only by response/surface flags via Collide_Filter); the only tri skip
   is the |dot(normal, axis)| > 0.999 perpendicular-face cull, which a 45° ramp (dot ≈ 0.68)
   passes. The push direction is unit(closestTriPoint − sphereCenter)·(radius − dist) — on an
   oblique face that carries +y. The push accumulates into DeltaP (0x2d9bd0), is rotated to
   world and added to BLData+0x750 by `Collide_Intersect` 0x1ebaa8 (read @0x1ebc00), and
   `Player_CollisionHandler` 0x1949a8 adds BLData+0x750 to obj+0x30 every tick (disasm
   0x194c60-0x194c70). **The player rides the push up the ramp.**
2. **Grounding has no slope gate either.** `Player_FeetOnPoint` 0x194188 grounds from its
   down-ray (dist ≤ cc+0.1) regardless of the face normal; the 0.7 dot gate lives in
   `Intersect_CylGeom` 0x10a650-0x10a734 (0x10a668 `c.lt.s`, 0x10a674 `bc1t` verified at
   instruction level) and gates ONLY the pick bit (HITTEST+0x9c bit 0) — the backup grounding
   signal, which a 45° ramp indeed never sets. The gravity gate reads the ground normal into
   BLData+0x110 (FeetOnPoint copies hit+0x14; `Player_Collision` 0x194f74 applies gravity iff
   ungrounded OR normal.y < 0.5 — 0.68 > 0.5 keeps him grounded on the ramp).

Consequence for the kink: the game's player walking the spine into the ramp face is pushed
up-and-along by the slope-blind contacts and stays grounded via the ray — the port's
`capsule_tri_contact` push math mirrors this (same push shape), so **no port divergence is
established yet**; the fall to the y-29 roof is so far explained by the WALKER (the stuck
sidestep shoving Bond west off the strip once steering stalls — the round-6 reading), not by
the collision model. Round 8 must dump the per-tick contacts DURING the stall approach (the
probe's contact scan ran at the wrong end position — fix that, then read which contact wins).

## What ran

- `cargo test --offline --release -p nf-view --test walkreal_07000005_live -- --test-threads=1`
  — **6 passed; 0 failed; 2 ignored** (the two notch tests stay ignored behind the kink, with
  their reasons; the only source change in this round is `mod walker;` → `pub mod walker;` in
  that test file so the probe example can reuse the walker — no test logic touched).
- `cargo build --release -p nf-view --example probe_kink_05` and one full probe run (the dump
  + the reproduction above).
- **The full four-crate suite did NOT run this round.** Background jobs die between tool calls
  in this sandbox (the lane-5/6 finding reproduced: the nohup'd fast-runner start died silently
  during its build, twice, including under setsid). The foreground build of all four crates
  completed (`cargo test --no-run` green; nf-audio needed the user-prefix ALSA link path via
  `RUSTFLAGS="-L native=$HOME/.local/usr/lib/x86_64-linux-gnu"`). The suite must run on the
  final tree (merged with origin/master) before merge; nf-game/nf-formats/nf-audio are
  untouched by this round (one test-file visibility word, one new example, docs).

## What's left (round 8)

1. Per-tick contact dump during the stall approach — which contact pushes Bond west (the
   wall-top edge pick vs the ramp face), and whether the port's push/grounding parity holds on
   the ramp (the game rides the push up; if the port does too, the fix is in the walker legs:
   aim along the ramp's strike / wider arrive so the stuck ladder never fires).
2. Walk the spine past the kink with real input; un-ignore `the_notch_grab_and_ride_parked`
   and `the_notch_holds_the_walk_and_the_grab_needs_its_zone`; the grab/ride legs run as
   written (heights re-verified in round 6 [D]).
3. The south ledge after the ride, the window-ledge crawl #35 (−34.10, 37.28, −31.65) and the
   exit door #112.
4. Live checks for the main agent (from round 6, still the fastest settle): fresh boot, stand
   at (−23.5, 33, 46.5), push south-west — gdbwatch obj+0x30 (position) and BLData+0x750 (the
   push): if +0x750.y > 0 lands in obj+0x30.y the game rides the push up the ramp, matching
   [M] above; if pos.y never rises, the route avoids it.

## Usage

Task-system token/context figures are not visible to me this session. Session wall-clock:
08:24-10:00 UTC. One fresh clone + two downloads (work-slim 34.9 MB; ISO 369823203 B exact),
one toolchain install (rust 1.99.1, py7zr, user-prefix ALSA with the RUSTFLAGS link path), one
branch setup + three pushes (claim; the kink tri dump + traversal answer; this report), one
probe example written and run, one foreground build of the four crates, one touched test binary
run (6 passed / 0 failed / 2 ignored). The fast-runner suite could not be kept alive in this
sandbox (background jobs die between tool calls — documented above).
