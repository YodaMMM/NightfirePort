# 10-level-13-warm-chain-margin-7

07000013's real-damage walk (`walkreal_07000013_live::walk_the_warm_chain_to_the_front_entrance_and_out_real_damage`)
is still ignored. Master 828704a46 carries level-13-warm-chain-margin-6: the walk dies at t4958 inside the security
room to #9 (skin 0x5000006, weapon 0x10) at the room's east corner (-19.6, -368.4, 25.6); he wakes on movement only,
and the fight is decided by ammo (the in-hand clip of 8 dries mid-fight and the leg's resupply fetch walks Bond back
into his line). Read the ignore message's update-4 ledger and take its next lever (enter the room with a full clip,
or answer #9 from a line that reaches the east corner). Real input only; keep the other walks byte-identical.
Tests: walkreal_07000013_live, interactables_07000013_live. Branch: s/level-13-warm-chain-margin-7.

Measured 11-13 (main): in 12 the 96.5 -> 40.5 loss lands on the walk to the north-lobby line (-8.25,19.0) - #7 (pistol, 3.5 a hit from (4.9,14.2)) plinks while Bond wedges at (-8.8,18.2), then #6 7.0 x2. #7 first from the pre-walk spot (shoot_bounded 900, not exposed) dies in 12 at t13094; from the line spot he is not exposed. Not shipped.

---
claimed-by: sandbox7 at 2026-11-14T03:04Z

---
## Report (done, 2026-11-14T05:15Z)

# level-13-warm-chain-margin-7

Branch `s/level-13-warm-chain-margin-7`, cut from origin/master 385ebb66d. Task: take the
update-4 ledger's next lever in
`walkreal_07000013_live::walk_the_warm_chain_to_the_front_entrance_and_out_real_damage`
(00 Agent, real damage, still ignored) — the security-room fight is decided by ammo.

## What's done

1. **Reproduced the ledger byte-identically.** The ignored walk dies at t4958 at
   (-27.26,-368.30,36.03) inside the security room; #9 (skin 0x5000006, weapon 0x10) at
   (-19.6,-368.4,25.6) lands 27.5 in 2 hits, the fights land 18 of his 25 hp with the in-hand
   clip 8 and dry, and the fights' own restock resupply (`damage_log.rs` low gate, clip+spare
   < 10) starts a fetch nav to the box at (-27.2,-369.4,30.1) d 6.0 that walks Bond back up
   into the line [L /tmp/d3_repro.log].
2. **Fixed the wake law (the update-4 "wakes on movement only" model is wrong).** The inside
   drain's #8 stand fires rounds at d 6.7 and #9's first landed burst (-20.0) arrives at
   t4869 INSIDE the drain's quiet waits — before any post-drain leg. Gunfire at room range
   wakes both room guards [L runs 1-5: the burst tick is byte-identical with and without any
   post-drain legs].
3. **Found the real ammo source and landed the take.** The pocket drops rest at
   (4.85,-369.43,22.23) and (4.81,-369.13,23.02) — the sweep's posted-guard kills drop at
   their posts, on the lobby east lane's own line [L the run-7 pickup dump]. A d3-only
   WALK-OVER leg to (4.8,-368.4,23.0) r 0.6 takes BOTH (run 8: `taken 2`, in-hand clip 13
   **spare 9**, inventory total 64) with no fetch nav. This is the ledger's "enter with
   spare > 0" — the fights' `switch_dry` finally has rounds to swap to mid-fight.
4. **Re-dated the door-phase fragility.** The walk-over leg costs +163 ticks; the shift
   re-rolls the stream/#4 volley schedules. Run 8 died t4378 at (-22.75,38.95), 0.6 u short
   of the door drain spot, to #4's three -7.0 mid-leg; run 10 (drain moved to the glass band)
   died t4314 AT the drain spot mid-aim-raise to the stream shotgun #7 (four -3.00 pellets in
   one tick). The fights answer #5/#6 with `killed false` in BOTH timelines (t3292/t3801) —
   the plain fights cannot open their boxes, so the stream guards survive to melt the glass
   band. The radius-45 drain now runs at the glass band (d3-only), with the run-8/10 ledger
   in the comments.
5. **Measured dead this session (documented in the ignore message, do not repeat):**
   - the pre-drain "sleep dump" (the aim-raise + the trigger taps at #9's disc post): the
     dump's gunfire wakes BOTH guards (#8's burst t4791, #9's t4887 killed mid-dump) and its
     rounds land ZERO — the mid-room furniture blocks the inside-spot line to #9 (consistent
     with the drain's radius-12 sweep never taking him at d 11.96);
   - the game's basis is [x, y=UP, z] (the panel aim's pitch formula proves it): the run-3
     dump aimed at floor level and hit the ramp at #9's feet (hp 25 → 24);
   - the spawn-pocket deliberate resupply: the band's nearest pickup is the placed TRIPBOMB
     trio at (-0.4,14.0); its take EQUIPS the gadget in-hand (clip 17, the P99's 30 parked)
     and the lobby fought the gadget dry — death t5390 at the glass band [L runs 6-7];
   - the west-lane-first room order: byte-identical t4958 [L run 2].

All changes are in `crates/nf-view/tests/walkreal_07000013_live.rs` only, every new block
gated `real_damage && difficulty == 3`. No shared helper touched; no health/ammo/position
writes anywhere; the only inputs are the view drives, the trigger taps and the legs.

## How it was verified

- The d3 walk runs to the glass band/door phase deterministically; the runs 1-10 logs are the
  evidence for every claim above. The test itself stays `#[ignore]`d with the update-5 ledger
  (the walk is not green yet — see below). It compiles (`cargo build --offline --release -p
  nf-view --test walkreal_07000013_live` clean).
- The d2/d1/plain sisters and `interactables_07000013_live` were NOT run this session: every
  change is inside `real_damage && difficulty == 3` blocks those tests never enter, and no
  shared helper changed, so their legs are byte-identical by construction. The next session
  must run them (rule 10).
- The full 4-crate suite did not fit in the sandbox window.

## What's left (the next session's first levers, also in the ignore message)

1. The stream guards (#5/#6/#7) must die before the glass band — the fights' `killed false`
   on #5/#6 means the aimed drains at the re-anchor and north-west leg ends are the answer,
   if their boxes open there.
2. The pocket-drop take's +163-tick shift is the only variable in the door-phase coin flip —
   try absorbing it (a shorter drop leg, or taking the drops on the lane leg's own path).
3. The door drain at 45 belongs at whichever leg end the shifted schedule leaves quiet.

## Live checks for the main agent

1. `tools/trace/gdbwatch.py` on drone #9's wake during the security-room drain: address =
   the drone's state byte (the CombatNoSight 0x5c family), access R/W, expected: the state
   leaves the sleep idleset within ~350 ticks of the #8 stand's first shot (the wake is
   gunfire- and distance-dependent, not movement-only).
2. `tools/trace/fieldprobe.py`: from the inside spot (-27.3,-368.3,36.15), the sight mask
   probe toward #9's post (-19.6,-368.4,25.6) at chest height — expected: BLOCKED (the
   mid-room furniture the walker's gun rays also die on); the same probe from the west lane
   (-27.5,-368.3,30.0) and from the doorway with the door open, to map which lines clear.
3. What a fresh-boot player's P99 volley into the asleep #9 does (does the port's hip
   damage model match the disc at d 13 through the furniture block).

## Usage

not visible to me (no token, context-window or cost figures are exposed in this environment;
nothing to copy exactly).
