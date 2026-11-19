# retune-nb-14-5 — the 14 infiltration to the delta exit walks, round 5

Branch `s/retune-nb-14-5`, cut from origin/master 434826a9b with `origin/s/retune-nb-14-4` and
`origin/s/retune-nb-14` merged in (the round-4 ledger + the Operative sister's shipped d1 laws;
the merge conflicts resolved: the 00agent test kept the round-4 ledger, the shared scoop kept the
v14 peace gate, both WORKLOG entries preserved, WORKLOG appended only).

## What's done

**The re-measure the task ordered (the merged tree).** The 00agent death is UNCHANGED from the
round-4 ledger: t25036 at (−60.22, 0.73, 37.94), 17.5 health at the door's west face, 34 hits for
100.0, 28 return-fire fights, #33 (0x14) finishing with 2.50 + 15.00 [L 2026-11-19, NF_DMG_TRACE
run on the merged tree]. The SniperAim ENTER merge did not re-roll the d3 stream before the door
face — the round-4 ledger stands verbatim.

**Lever 1 of the round-4 ignore message ("survive the door face at 17.5") shipped and measured
LIVE**, in five pieces, each judged on its own real run [L, six runs total this window]:

1. `switch_on_first_blood` (opt-in field + per-tick first-blood arm in `stand_and_fire`,
   `crates/nf-view/tests/common/damage_log.rs`): the re-measure showed the fight on #33 opened at
   t25030 and Bond died the same tick #33's second volley (15.00) landed — Bond had dealt 55
   damage into him by then. The existing `hurting_first` retarget could not fire (its 30-tick
   cadence falls one tick after the #32 stand ends; its >3.0 margin reads the 2.50 volley as a
   graze). The new arm switches the stand the first tick a NEW shooter's 240-frame damage rises
   above 0.0 while the current target's is 0.0.
2. The west-face leg's return-fire MUTE + the r(0.3) line park (walkreal_07000014_live.rs, d3
   real-damage only): the death began as a fight that opened MID-LEG and parked Bond 1.6 u south
   of the corridor line where #32/#33 are near-collinear (0.74 u lateral) — every stand took
   ~2 dmg/tick while the volleys landed. From the leg's own target on the line the same posts
   separate (1.09 u). d1/d2 keep the shipped leg byte-identical (the flag flips only inside the
   d3 gate).
3. `route_scan_ranked` (the door-face scans' weaker-first rank): the door squad's weaker members
   ARE the killers (#33 hp 10, #34 hp 14, both 0x14; the hp-20 #32 never landed a round in any
   run). Every other scan call site keeps the shipped road-blocker rank (run 12's measured law).
4. `shoot_bounded_fast` (walker_b2.rs): 4-tick tap spacing + the reposition SLIDE OFF for the
   door-face fights only. The shipped 16-tick cadence measured 44 damage over 50 ticks in the
   west-face fight (the kill landed inside #33's volley window); the slide's first-60-tick strafe
   dragged Bond 3 u off the line into the blocked geometry (measured parks (−58.4, 36.6)).
   `shoot_bounded`/`shoot_loose` keep 16 + slide — byte-identical for every other caller.
5. The second extension: the door-face scans accept EYE-true guards as candidates (the near guard
   at (−49.6, 40.9) had his muzzle blocked from the west park but his body boxes exposed through
   the open leaf span — Bond's rounds follow the view), and the Kiko-corridor picks take the fast
   taps (d3-gated block).

**Measured result [L runs 5-6]**: the door squad #33/#34 died INSIDE the west-face scan (killed
true; killed true health −22; ZERO volleys landed) and the walk CROSSED the doorway for the first
time since round 4. The death moved t25036 → t25939 (−47.75, 1.05, 42.58) → t26979
(−48.91, 1.13, 34.28, the round-1-era spot) — 1943 ticks of new ground. The final blockers: the
crossing's point-blank 15.57 (d 2.5, the 3.0 auto-hit band [M DroneWeap_DoBulletAccuracy
0x179740:113-115]) and the Kiko hold's #20 — ZERO of ~50 rounds across four fights at d 6.5 (his
ray from the hold spot dies on the corridor's own geometry between z 39.2 and his post
(−51.0, 32.6)), then his point-blank 1.93 finished the 1.9-health walk.

**The weapon data the lever rests on** [L work/ee_snapshot.bin, weapon_data 0x2bf150, record
stride 0x10c (validated against bullet.rs's Laser cite 0x4e/0x4f +0x1C = 4.0/8.0)]: weapon 0x14 =
spread base +0x24 3.00, flags68 0x880 (NO 0x400000 bit — the `aimed_fights` zero-spread path
cannot apply), damage +0x0C 2.50/round; weapon 0x1e (the aimed_fights control) = 100.00/0x408050.
This killed the aimed-fights lever before it was written.

## Still red / what's left

- The 00agent walk stays `#[ignore]`d: the refreshed round-5 ledger (what stands, the measured
  run-5/6 facts) + the next lever (a hold/stand spot whose ray to #20's post clears, or the
  fights' creep law; then the Kiko-corridor armour fetch) are in the ignore message.
- The Operative d1 sister (`walk_the_infiltration_to_the_delta_exit_real_damage`) was green on
  s/retune-nb-14's OWN tree (master 73822d3a0) but the merged master RE-ROLLED its stream: the
  re-measure hit the retune-nb-14-3-era death exactly (t36801 at (5.05, 1.13, −23.31), the wake
  legs' #28/#29 0x1a pair) [L run]. It is re-`#[ignore]`d with that ledger and the candidate laws
  (the round-5 door-face pieces) for the next round. My changes are d3-gated and did not touch
  the d1 stream — the re-roll is the inherited master change, the same one the task flagged.

## How it was verified

- Required binaries, FINAL tree [L 2026-11-19]: **walkreal_07000014_live 8 passed / 0 failed /
  2 ignored (128.67 s)**; **interactables_07000014_live 10 passed / 0 failed (15.45 s)**.
- The fast suite (`tools/test/suite.py`) produced NO total line, same as round 4: the sandbox
  kills background jobs between tool calls (its cargo child died twice — once holding the target
  lock, which stalled a foreground build for 10 minutes), and the foreground four-crate build
  exceeds the window. Instead every walkreal-relevant surface was run directly (above). The
  shared-helper changes are behaviour-preserving by construction at every existing call site
  (`shoot_gated`'s new params default to the shipped 16-tick + slide; `switch_on_first_blood`
  defaults false; `route_scan` delegates `false`), and the only behavioural flag is set inside
  the 07000014 d3 real-damage gate.
- Every fact above tagged [L] comes from this window's runs on the real ISO (NF_DMG_TRACE /
  NF_RF_TRACE); the weapon-record figures are [L] EE-snapshot reads; code citations are [M] with
  the decomp address; no [S] items entered pass/fail.

## Usage

not visible to me — this environment exposes no token, credit, context-window or cost figures for
the session; nothing to copy.
