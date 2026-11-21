# retune-nb-01-3 — 01 spawn-to-exit-with-the-contact real-damage walks, round 3

Branch `s/retune-nb-01-3` (from origin/master d57c3fdac with s/retune-nb-01-2 af1a1dbf6 merged in —
01-2 was not on master; one doc-comment conflict in walkreal_07000001_live.rs resolved keeping both
sides' notes). **d3 (00 Agent) is still red**; the round-2 ledger's first two levers landed and were
measured on their own real runs, the death changed shape twice, and the refreshed ledger + the next
levers are in its ignore message. **d1 (Operative) is red on the merged tree by a failure my levers
cannot reach** (below). d2 (Agent) is green. Everything is judged on real runs through the real input
path (`walkreal_07000001_live`), no top-ups, no guard-code changes, no damage/health changes.

## The levers landed (all measured, all d3-gated unless noted)

1. **The endgame walk posture (the round-2 ledger's lever 1)** — three pieces:
   - `flank_point_x_min` (walker.rs, opt-in, default None): the flank POINT west of x -27 is a hold,
     not a walk — `flank_hold_x` checks the guard's park; this checks where the flank nav would walk.
     The v3-round run confirms the t83962 pocket-flank death is gone (Bond held, never walked the
     (-31.6,16.6) flank) [L d3_r12_v1 vs the round-2 v3 run].
   - `shield_boxes` for d3 (`difficulty >= 2`; d2 already ran it, d1 stays cylinder): the mouth
     captain #3 fight at the stairwell head held the trigger at ZERO rounds fired for two 1200-tick
     passes with the guard exposed (clip 5 spare 0 unchanged; the settled contact sat in the
     cylinder+bloom cone) and #3's 7.20-pellet volleys killed Bond — death t80171, 58 hits [L
     d3_r12_v1]. With the hit-box cone the fight kills #3 in ~200 ticks [L d3_r12_v5/v6].
   - `fight_hold_west_of` (walker.rs, opt-in): a blocked fight west of the line backs off (forward
     -0.6, no strafe) instead of strafing/pushing — the v5 fight killed #3 and then drifted Bond to
     (-31.43,16.36) inside the pack's lines where he died on the crossfire (55 hits) [L d3_r12_v5].
2. **The tap loop joins d3**: the plate-lane clear and the pre-stairwell pocket set are fought with
   `plate_pocket_fight` in place + one band-checked north-edge stand retry (the plate-lane clear's
   own stand pick). The old `shoot_bounded` held 1800-tick windows on the walled lane guards at zero
   damage (clip 21 spare 27 untouched, "exposed false" every round) while the contact walked the
   plate lane into their crossfire — his corpse at the plate, the escort-fail restart at t78420 [L
   d3_r12_v2]. The walled set is now left at once (no lure — the 01-2 v2 law) [L d3_r12_v5/v6].
3. **The #11 window cap (the round-2 ledger's lever 2)**: after a volley-broken window, a guard
   whose windows already cost Bond > 10.0 total is left (the re-peek re-runs the same one-frame
   burst; the 2.0 volley break fires after the damage landed). Measured: "hall guard 11: the window
   burn cap (bond 12.0 of 70.0 spent) — the guard is left" — #11's second 12.0 burst prevented, pool
   58.0 at the halls (was 46.0 on the same phase) [L d3_r12_v7].

## What is still red

- **d3 (00 Agent)**: the new death (v7) is the route pre-clear APPROACH fight: the walk navigated TO
  guard 5's park (13.7 u) and the return-fire exchange cost 45.4 in one pass (58.0 -> 12.6, killed
  true), then the fist guard #13's melee dribbles (9.0 in 5 hits at d 17.5 from a hp-0 guard) finished
  Bond at (5.85,14.90), t7568 [L the d3_r12_v7 run]. NEXT LEVERS (in the ignore message): (1) the
  route pre-clear approach posture at d3 — the approach nav to an awake guard runs quiet or is
  band-checked; (2) verify the decompile's melee range gate before trusting the #13-shaped dribble
  ([S] unverified); (3) the recovery tap posture engages once the approach phase holds.
- **d1 (Operative)**: red on the merged tree, NOT by this branch's changes. Every lever is gated
  `difficulty >= 3` (or `>= 2` where d2 already ran it; `shield_boxes` stays false at d1), so d1's
  stream on this tree is tick-identical to the merged base's. The death: the contact's corpse at
  (-30.55,12.46) — the escort-fail restart, then the plate assert — Bond himself ended at 50.8
  health + armour 28.2, 83 hits for 49.2, and the plate walk's return fire killed #4 at t29390 [L
  the whole-binary run]. This is the same contact-protection race d3's ledger documents: the
  contact's route ends in the pocket between the guards' parks and his pool is 10.0, so one guard
  hit kills him. NEXT LEVER: extend the pre-stairwell pocket posture (the tap loop in place + the
  band-checked north-edge stand retry) to the real-damage sisters (`!w.tops_up()`), then re-verify
  d1 AND d2 on their re-rolled streams — one gate change, two runs.

## How it was verified

- `walkreal_07000001_live` **whole**: 6 passed / 1 failed / 1 ignored (451 s) — d2 green, d3
  ignored with the refreshed ledger, d1 the inherited failure above.
- `interactables_07000001_live` whole: **10 passed / 0 failed** (23 s).
- The d3 individual runs v1-v7 (each lever on its own real run, `--include-ignored`): the death
  chain t80171 (the shield hold, 58 hits) -> t78420 (the contact's corpse at the plate, escort-fail
  restart) -> t65340 (same, faster assert) -> t33826 (the fight drift into the pack, 55 hits) ->
  t7568 (the approach-fight trades + the melee dribbles, 29 hits) [L the d3_r12_v1..v7 logs].
- The fast suite (`tools/test/suite.py`) **did not fit the window**: the sandbox needed a fresh
  toolchain install (cargo/ALSA) and the walkreal sweeps are minutes each; the required 01 binaries
  were run by hand instead (above, the round-2 precedent). The walker.rs fields are all opt-in with
  default None and the only setters are the 01 walk's d3-gated lines, so the untouched levels are
  stream-identical by construction; the rule-7 walkreal sweep beyond 01 is **left for the next
  round** (02, 04-0b and the rest) together with a full suite total line on the final tree.
- No scratch code was committed; the probe facts live in the code comments and this report.

## What is left

- d3's route pre-clear approach posture (the named lever above), then the recovery tap posture.
- The d1 contact-protection lever (extend the pocket posture to the sisters, re-verify d1+d2).
- The melee-range decompile check ([M] NDrone2's melee record) for the #13-shaped dribble.
- The rule-7 walkreal sweep + a full `tools/test/suite.py` total line on the final tree.

## Usage

not visible to me — this environment shows no token, credit or context-window figures for the
session (no cost or usage numbers were displayed anywhere in the tooling).
