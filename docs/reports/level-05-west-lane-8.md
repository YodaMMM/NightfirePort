# level-05-west-lane-8 — 07000005: the kink walked at full forward; the notch grab + strafe ride un-ignored and green

Branch `s/level-05-west-lane-8` (from origin/master 26543436). Fresh sandbox; setup (clone,
work-slim + ISO, toolchain) took the first ~15 min. Carried from the lane-7 branch (not on
master): `examples/probe_kink_05.rs`, the root `WORKLOG.md`, and the walkreal `pub mod walker`
visibility.

## What's done

1. **The kink is answered — no port divergence; it is a walk-input shape.** The 45° ramp
   (mat 0x0c, n (−0.520, 0.680, 0.517), base edge (−26.027, 33.0, 44.441) → (−23.393, 33.0,
   47.090) [D `examples/probe_kink_05.rs`]) never grounds a capsule-tangent rest in EITHER the
   game or the port:
   - the feet-ray band needs |pos − hit| − 0.1 ≤ cc [M `Player_FeetOnPoint` 0x194188:106, the
     decompile's `*(float *)(hit + 0xc) - 0.1 <= cc`], but resting the walk capsule (B = pos −
     0.485, r 0.55 [M `Player_Collision` 0x194df0 disasm 0x195444/0x195464]) tangent on the 47°
     face leaves the upright ray 0.485 + 0.55/0.68 = 1.294 long — over the 1.135 band always;
   - the pick word's bit 0 needs dot(normal, axis) > 0.7 [M `Intersect_CylGeom` disasm
     0x10a72c-0x10a734: `lhu +0x9c` / `bnez` skip / the 0x3f33 0.7 gates / `ori 1` / `sh`] and
     the ramp's 0.68 never sets it;
   - the pick word's second feed — `Collide_Intersect` ORs a stack halfword into HITTEST+0x9c
     per intersect-loop pass, gated by `+0x94 & 0x200` [M disasm 0x1ec138-0x1ec150; cel+0x94
     holds the glist flags per `MapObject.flags`, level.rs] — is CLOSED on this ramp: its
     instances (`Room_Outerwall_Corner`, `Room_Outerwall_B2`, `Room_Road_B`) carry glist flags
     0x0 [D `examples/probe_flags_05.rs`], so the store is skipped.
   So the game's player on this ramp is push-borne ungrounded exactly like the port (gravity
   applies while ungrounded [M `Player_Collision` 0x194df0:104]), and the push's horizontal
   part drifts him NW — up-slope-ward, toward the base edge and the wall cliff — measured
   ~0.024/tick once airborne [L `probe_kink2` run 1: at the walker's 0.45 near-target forward
   the drift beat the walk; Bond crossed the base edge at tick 11 of a raw steer and fell to
   the y-29 roof — the lane-6/7 symptom, reproduced per-tick with contacts: the ramp tri at
   pen ≈ 0.000, then the wall face at exactly 0.550 sliding contact, no upward force ever].
2. **The walk takes the kink at FULL forward with a 0.35 SE (down-slope) aim bias per leg**
   [L `probe_kink2` run 2: all 16 legs from the spine-4 line through the ledge run arrive in
   6-61 ticks]. `walk_the_kink_ramp` added to `walkreal_07000005_live.rs` — real inputs only
   (turn + forward, no teleports, no state pokes), a fall guard (centre y > 33.2; the y-29 roof
   rest is 30.1) and a 900-tick per-leg wedge guard. `walk_the_gap_to_the_notch` now walks
   spine 1-3 with the walker's own steering, then the kink ramp + the ledge run to the notch's
   north edge (−23.55, ~34.79, 18.83); the walked-end assertion moved there (x −24.2..−22.8,
   y 33.4..35.8, z 16.5..20.5).
3. **`the_notch_grab_and_ride_parked` UN-IGNORED and green.** Two harness gaps fixed with real
   input, both now cited in the test:
   - the grab press needs the settle: the ctx-3 zone arms already at the ledge-18 end
     (`icon_ctx == 3` [L the zone diag; the game's own armed-grab signal, `BLData+0x95F = 3`
     M `Player_HandleJump` 0x1916b0:126-147]), but the ledge-18 rest flaps grounded/airborne on
     the push dither and a down-push on a press tick sets head_bump, which skips the whole
     HandleJump arm [M the port's jump arm gates: grounded_history, head_bump, the 0.95 head
     LOS]. The harness settles (clear head LOS, residual drift under 0.032/tick, no head_bump)
     then pulses the press one edge per 6 ticks [L the press diag runs: a single press from the
     drifted rest was eaten with no jump and no grab].
   - the ride is STRAFE-driven: the old forward-only ride never advanced (z stuck 8.2-8.9 for
     1200 ticks [L the ride diag]) — the wire ride's motion comes from the strafe axis picking
     the stroke mode (7 toward B, 8 toward A, 6 idle) [M `Player_Wire` 0x1977c0-0x197834].
     Strafe 1.0 rides south across the notch; the landing pins z < 8.0, y > 24.0 [M the travel
     segment 0x1976f8-0x197724].
4. **`the_notch_holds_the_walk_and_the_grab_needs_its_zone` UN-IGNORED and green.** One line
   fixed: the sister's forced yaw was FRAC_PI_2, which faces +x (east, off the ledge's east
   void) in the port's forward = (sin yaw, cos yaw) convention — the test never ran (ignored
   since birth) and its "south" comment needs yaw π (the notch's dip is at z 4-10, south =
   −z of the walked ledge-18 edge). The walk without the grab press is held by the dip as
   pinned [D the 1-unit map].

**`walkreal_07000005_live`: 8 passed / 0 failed / 0 ignored** (foreground,
`cargo test --offline --release -j2 -p nf-view --test walkreal_07000005_live -- --test-threads=1`,
57 s). The suite's 3 allowed slim-ISO `disc_golden` failures are the only failures anywhere:
nf-game 546 unit tests + its live binaries all green; nf-formats green except the 3 allowed
`disc_golden`; nf-audio green (run with `--test-threads=1`, the user-prefix ALSA link path).

## What's left (round 9)

1. The south ledge after the ride, the window-ledge crawl #35 (−34.10, 37.28, −31.65) and the
   exit door #112 — the next legs south of the notch landing; the ride's landing state is the
   walking start (grab Bond's post-drop position from the ride end).
2. The full four-crate suite via the fast runner on the final merged tree — this sandbox kills
   background jobs between tool calls (the lane-5/6/7 finding reproduces), so the runner
   cannot survive here; the three non-view crates were run foreground (green modulo the 3
   allowed disc_golden), and nf-view's touched test binary is green. The remaining nf-view
   live binaries were not re-run this round (time box); the crate's only source change is the
   test file + two new examples.
3. Live checks for the main agent (still the fastest settle, carried from lane-7): fresh boot,
   stand at (−23.5, 33, 46.5), push south-west along the spine — gdbwatch obj+0x30 and
   BLData+0x750: if the game's player is push-borne ungrounded on the ramp (BLData+0x60 bit 3
   NEVER sets while on the ramp), the walk-input answer above is the game's own; if bit 3
   sets, the pick word's second feed fires on the real hardware and the port's grounding needs
   the +0x9c arm re-read.

## Evidence index

- [D] `examples/probe_kink_05.rs` (carried from lane-7): the kink tris, the fine floor profile.
- [D] `examples/probe_flags_05.rs` (new): the glist flags of the kink instances (0x0 — the
  +0x200 gate closed).
- [L] `examples/probe_kink2_05.rs` (new): run 1 = the hover + the NW drift + the base-edge
  crossing per tick with capsule contacts; run 2 = the full-forward + SE-bias walk, 16/16 legs.
- [M] Addresses: Player_FeetOnPoint 0x194188 (the band, decompile line 106; the grounding
  `BLData+0x60 |= 8` when the pick word != 0 OR the band hit — decompile lines 176-185);
  Intersect_CylGeom 0x109da0 (the push; the pick bit-0 block disasm 0x10a5e0-0x10a748);
  Collide_Intersect 0x1ebaa8 (the +0x9c OR disasm 0x1ec138-0x1ec150, the per-tick clear in
  Collide_Update 0x1ea46c); Player_Collision 0x194df0 (the capsule arms disasm 0x195444/0x195464,
  the gravity gate); Player_CollisionHandler 0x1949a8; Player_Wire 0x197658 (the strafe stroke
  0x1977c0-0x197834, the travel segment 0x1976f8-0x197724, the drop paths 0x197748-0x197834).

## Usage

Task-system token/context figures are not visible to me this session. Session wall-clock:
15:51-17:30 UTC. One fresh clone + two downloads (work-slim 36689089 B; ISO 369823203 B exact),
one toolchain install (rust 1.99.1, py7zr, capstone for the disassembly, the user-prefix ALSA
link path), four pushes (claim; the carried lane-7 artifacts; the kink + notch green checkpoint;
this report), three probe examples written and run (probe_kink2 twice, probe_flags once), the
walkreal test binary run to 8/8 green, and the nf-game/nf-formats/nf-audio suites run foreground
(green modulo the 3 allowed disc_golden). The background fast-runner suite died between tool
calls as in lanes 5/6/7 — the suites above ran foreground instead.
