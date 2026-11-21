# s-crouch-latch-walks: the 0d c5-door walks on the game's crouch

Built on s/crouch-latch (7d4a3204a). No game code changed: only
`crates/nf-view/tests/walkreal_0700000d_live.rs`. No shared random stream change (walk input only).

## Why the two walks went red

Base is 82765b537 (the s/crouch-latch merge base). Base and branch logs run side by side in this lane.

- **d2 (`..._real_damage_agent`)**: identical to base up to the claw. On base the corner crouch
  press kept the crouch flag through the claw hang and the ride (the capsule stood, y 14.60, but
  the sight drop and 0.75 visibility were on). On the branch the claw's substate change stands Bond,
  as the game does, so he rode standing: about 19 chips of 1.0 from x 177 onward, and the yard
  burst killed him at t20428 (base: 2 chips on the ride, green at 22.8).
- **d1 (`..._real_damage`)**: the deck crouch presses were lost on base (the jump-grab's stand
  armed the flat 60-tick lockout and the wire ride froze it), so base walked the deck standing.
  On the branch they crouch him (deck walk +404 ticks, was +281). The ride then ran standing
  (claw) and took #15's 16.0; death t19198 in the yard.

## What changed in the walk (each with its cause, also in the walk's ledger comments)

1. **Crouch again aboard the gantry** (one press after landing on it, real damage only). Cause:
   the claw now ends the crouch. d2 ride: 3 chips instead of ~19.
2. **The stand press before the hop only fires when Bond is crouched.** Cause: a press on a
   standing Bond would now crouch him; a crouched jump press only stands him (Toggle).
3. **The real-damage slot hop holds 13 ticks** (was 20; the plain twin keeps 20, the d3 container
   hop keeps 36; `NF_0D_HOP_HOLD` overrides). Cause: the truly crouched ride boards at offset
   1.09 (base 2.00), and the 20-tick hold landed Bond at x 133.2 next to #3 (d 1.6, 1.2 a chip):
   d1 died t19467 even with a 63.6 ride. Measured holds (one port run each, landing x, result):
   - d1: 8 137.5 dead, 10 135.4 green 6.6, 11 135.2 dead, 12 137.8 green 11.6, **13 137.8 green
     20.0**, 14 137.7 green 18.8, 16 137.5 dead, 24 134.1 dead.
   - d2: 10 dead, 12 134.6 dead, **13 138.0 green 10.2**, 14 138.0 dead, 16 dead, 24 dead.
   d2 is green only at 13. That is a narrow roll: the landing spot is the same at 14 and the yard
   dice decide it.
4. **The d1 deck crouch is kept** (now effective). Measured on the branch: with it off, #17 and
   #28 land three more 16.0 rounds (t12388 catwalk, t13479 roof) and the ride starts at 28.6
   instead of 63.6.

Levers not needed: the kind-3 armour scan and the wait/idle sweep. The deaths were all in the
yard after the drop, and the hop hold alone moved the landing out of #3's reach.

## Results

- walkreal_0700000d (all four, `--include-ignored`): `walk_arrival_to_the_c5_door`,
  `..._real_damage` (end 20.0), `..._real_damage_agent` (end 10.2) green.
  `..._real_damage_00agent` (ignored) is red, as on base, but it now fails earlier: see below.
- Every walkreal_* binary (109, `NF_SILENT=1`, built with one `cargo test -j 8 --no-run`, each
  binary run whole, three at a time, 590 s cap each): 107 green. The other two match the
  s/crouch-latch report's base runs:
  - walkreal_07000013: `walk_the_warm_chain_to_the_front_entrance_and_out_real_damage_agent` is
    red in the whole-binary run (death t4583) and green alone (16.6 s). The s/crouch-latch report
    measured the same split on base and branch.
  - walkreal_07000001: `following_mayhew_down_completes_the_level` was still running at the
    590 s cap, so no verdict. The s/crouch-latch report hit the same cap on base and branch.
    The other non-ignored tests in that binary passed before the cap.

## The 00 Agent walk (ignored, red on base too)

Base dies t19772 on the yard floor. On the branch:
- With a standing ride (no press aboard), Bond dies on the beam at t19152 (1.8 chips from g2/g3 from x 176).
- Crouched aboard, the ride reaches the container break at 24.0. Then the jump on the tick after
  the stand press **freezes Bond in the air** at y 14.69: grounded false, vel.y 3.59 held, lockout
  frozen, five hops in a row. The beam carries him back east to x 170 and the hop assert fails.
- Standing up earlier on the ride (3.5 u or 16 u before the break) settles him at y 14.50, where
  the boarding stand reads 14.60. The same freeze follows, or the early stand dies to g18 on the ride.

This looks like a port physics question: a stand from the crouch on the moving beam leaves
Bond 0.1 low, and a jump from there hangs. It is not traced. The crouch port was not changed.

## Unproven

- Whether the game's Bond can hang in the air like that after standing up on a moving
  platform. Only the real game can settle it. Question: stand from a crouch on the 0d gantry
  while it moves west, then jump at once. Does he leave the beam?
- All tuning numbers are single port runs. The d2 green is one hold value wide.
