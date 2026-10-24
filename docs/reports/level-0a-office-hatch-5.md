# level-0a-office-hatch-5 — the post-spawn settle, then the hall and the corner

## What's done

**1. The settle is found, bit-exact, and ported.** The round's premise held: the placement is
1.1000 in both. [L fresh boot] the game's spawn ray from (−3.9319, 0.1, 10.3978) hits y 0.0000 in
the spawn cel (surface 0x15); [M `build_PointOnFloor` 0x1c95a8 writes the hit point back into the
placement vector and `Player_StandAtNewPosition` 0x190d88 adds 1.1f] — root 1.1000. The
snapshot's 1.0891 is the AFTER-spawn settle, and its exact value pins the mechanism:

- [L ee_start_0700000a.bin 0x1b78640+0x34 = **0x3f8b67fe** (1.0891110897064210)].
- That reproduces **bit-exactly** as `1.1f + f32( 4·(−9.8f·(1/60)) · (1/60) )` — FOUR
  `vel += WldGravity · REC_FRAME_RATE` adds ([M Player_Collision 0x194df0:107]; the constants
  [L] 0x2d8894 = −9.8f, 0x30d0dc = 1/60, FRAME_RATE_MUL 0x30d0d8 = 1.0) integrating ONCE
  ([M 0x194df0:189-197]), then the feet band catches the dropped root ([M Player_FeetOnPoint
  0x194188:106 `hit − 0.1 ≤ cc`]; dy 1.0891 ≤ cc 1.0347 + 0.1). Whole-tick falls at −9.8 give
  0.0027/0.0082/0.0163 — never 0.0109; the 4-adds-one-integration shape is the only fit (checked
  in f32 against the snapshot bits; the 4-free-falls shape misses by 1 ulp).
- The boot therefore holds the placed root for four ticks while the fall velocity accumulates,
  then releases it into the band. The handler phase cannot be running during the hold (its
  grounded tail zeroes the accumulating velocity, [M 0x1949a8 tail]).
- The anim path was cleared before settling on the hold: the stance script 0x06000a00 ([M
  Player_Init 0x1910f8's AnimScriptAdd]) is one op-0 command playing seq 0400084f — 2 frames with
  the root FLAT at 1.0108399 [D 07F0000a.bin; parsed with the port's own anim parser], and cc =
  root 1.0108399 + AnimObj+0xD0 0.0238784 = 1.0347183 [L bit-exact]. No root motion exists at the
  spawn, so the −0.0109 cannot be anim root motion.

Port: `Bond::settle_hold` (crates/nf-game/src/player.rs), armed in `stand_at` (the
`Player_StandAtNewPosition` analog): four held ticks (no move, no collision), the release tick
integrates the held velocity once in the game's op order (`ps2float::mul(vel, REC_FRAME_RATE)`),
the push/band tail re-grounds. Probe `crates/nf-view/examples/probe_0a_settle.rs` prints the
per-tick root: held 1.1000×3, then the drop, then the flat rest (1.08913 — the 2e-5 residual is
the port's floor-ray f32 noise under the placement; the game's ray hit exactly 0.0000).

**2. `the_spawn_height_matches_the_game_snapshot` un-ignored and GREEN** (waits the settle out,
asserts the game's 1.0891 rest + grounded). walkreal_0700000a_live: 4 passed, 3 ignored.

**3. The two other sisters re-noted honestly (still ignored, with this round's evidence):**
- `the_west_hall_glide_height_matches_the_game`: the settle is in (the port now glides the
  settled 1.0891; the old 1.1000-flat note is stale). The remaining gap is the base climb's
  +0.0256 settle itself: the game brushes the doorway jamb base (the tris (−8.604..−8.304,
  y 0..0.19) at z 16.278 [D port world tris — the probe run lists them]), peaks 1.18, settles
  1.1147 [L]; the port's capsule (bottom riding dy − cc = 0.054 above the floor) pushes off the
  base's south face for +0.0036 only, and a dead-on r-0.4 approach wedges the walker.
- `the_corner_crosses_like_the_game`: needs the glide's climb settle (above) to enter at the
  game's 1.1147; note updated.
- The main walk `walk_office_floor_qworm_keyboard_hatch`: this round's run walked the ring to
  "east past the alarm-panel corridor" (+98 ticks) and wedged at "the ai lane to the N-S
  corridor" — Bond (−11.25, 1.14, −14.60) ungrounded, the segment to (−8.60, −14.20) hit at 0.21
  (the run's STUCK dump is in the ignore note). Same south-leg wall the -2/-4 rounds met; the
  mouth's 0.5-wide guard slot still cannot take the 1.1 capsule.

**Verification runs:** nf-game lib 546 passed / 0 failed (the settle's input semantics fixed
`jumps_and_falls_with_gravity` to measure from the post-settle rest); walkreal_0700000a_live 4/0,
keyboard_0700000a_live 1/0, walk_0700000a_live 3/0, walkreal_07000009_live 3/0.
**NOT run: the full four-crate suite** (the 2-hour window closed first — see What's left).

## What's left

1. **KNOWN REGRESSION, fix first:** 07000005's `walk_to_the_gatehouse_and_ride_the_truck` now
   misses the ch-0x70 plate ("the crouched ride parks on the ch-0x70 plate [D; fix-truck]") — the
   −0.0109 spawn shift moves the ride's park point off the plate. The settle itself is [L]-solid;
   the 05 ride route needs re-calibration (or the plate-box check re-derived at the settled
   height). I ran out of window before diagnosing it.
2. The hall glide's base-climb settle (the +0.0256 shape through the doorway jamb base) and then
   the corner test.
3. The main walk's south-leg route re-derivation at the player's radius (the 0.21 wall + the
   0.5-wide mouth).
4. The full four-crate suite on the final tree (nf-game lib and the 0a/05/09 walkreal files were
   run; the rest of nf-view/nf-formats/nf-audio was not).

## Live checks for the main agent

1. **The 4-tick boot hold's gate** — the settle's shape is bit-exact but WHICH gate holds the
   first four ticks is not readable from the decompile. With `gdbwatch.py` on a FRESH boot of
   0700000a: watch writes to the player obj+0x30 (+0x34, obj 0x1b78640 in ee_start_0700000a —
   the address moves per boot; break on `Player_StandAtNewPosition` 0x190d88's return to catch
   it) and to BLData+0x50..0x5c (the velocity) for the first 10 ticks after the placement.
   Expected if the hold model is right: obj+0x34 stays 1.1000 for 4 ticks while BL+0x54
   accumulates −0.1633/tick to −0.6533, then ONE write drops obj+0x34 to 1.0891111 and BL+0x54
   zeroes. Also record whether ANY `Player_CollisionHandler` (0x1949a8) call happens in those 4
   ticks (expected: none).
2. **The doorway base climb's settle** — walk Bond west along z ≈ 16.29 through the doorway at
   x −8.3 on a fresh boot and record obj+0x34 + the BLData+0x750 push per tick from x −8.7 to
   −8.2. Expected per [L]: the root rises toward the 1.18 peak and settles at 1.1147; the trace
   should show whether the lift is the jamb base's edge push (which tri/surface the push reads)
   or a step-up — that names the port fix.
3. **The 05 truck ride with the settle** — the port's regression: on a fresh 07000005 boot, when
   the crouched truck ride parks, record Bond's obj+0x30 at the park moment and the ch-0x70
   plate's box check (which side Bond missed by). Expected: the game parks inside the plate with
   the settled spawn — the delta names the route re-calibration.

## Usage

not visible to me — this environment shows no token, credit or context-window figures.
