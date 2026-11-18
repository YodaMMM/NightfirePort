# retune-nb-08 — report

Branch `s/retune-nb-08` = `origin/master` 73822d3a0. Task: commit 3412a89a3 (the spacing scan
on every level + the close-range alt attack from the fire states) left the 08 whole-mission
real-damage walks `walk_the_whole_mission_on_real_input_real_damage_agent` and `_00agent`
ignored with their measured deaths; re-tune the walks for the new fights.

## Done

**The Agent (difficulty 2) walk is GREEN and un-ignored.** End health 79.0, armour 24.1,
14 hits for 21.0 total damage, five rockets (502.5 damage vs hp 448.5, best single 101.0), the
whole mission walked on real input in 31070 ticks, both objectives folded, the exit hand-off
fired [L the run on this branch]. The margin is wide (the 11-10 green run ended at 14.9
health).

**The 00 Agent (difficulty 3) walk stays IGNORED with its full ledger and the next lever** in
the ignore message (its fight code is reverted to the original measured state; the death
reproduces exactly at t28298 [L the whole-binary run]).

## The diagnosis (all [L] fresh runs on this branch unless marked)

- The nb-scan-all upstream walk is ~6500 ticks FASTER than the 11-10 green runs (the old fight
  started ~t35500; the new one ~t25700-28900), so the Hind fight starts at a different point of
  the machine's deterministic route circuit. On the Agent phase the machine's passes line up
  WITH the west doorway lane; on the 00 Agent phase the machine flies HIGH OVERHEAD (y 25-29)
  and no cabin stance gets a sight ray past the lintel (the eye ray stops t0.106 at
  (111.1, 15.0, −179.4) [L the ray dump]).
- The Agent ledgered death (t31483, 4 rockets landed 401.8 of 448.5, dead at the 5th launch) is
  the alternating dodge's drift: the missile-dodge episodes walked Bond 2.3 u east of the slit
  into the pane-hole gallery, 59 machine drips (−1.20/−1.40) for 72.6 world damage, the
  upstream guards only ~15.4.
- The 00 Agent death is a bleed-out while the launch gates starve: the gun chips (the Agent
  run's −1.40 drips at the d3 pain 1.0 = −2.00) plus the missile blasts drain the 78-point
  budget (health 40 + armour 38) with ZERO-ONE launches in ~2600 fight ticks; the upstream
  y-46 hall trio (#14/#11/#2, d 96-107 snipes) plus secure-room #5 cost 48.0 of it before the
  ride.

## The levers shipped (each judged on its own real run)

1. **The dodge-side bias and the drift walk-back open from d3 to difficulty >= 2** (the
   anchor-ward side pick instead of the alternating sign). Gated on difficulty; the d<2 else
   branches are the old code paths — the d1 real-damage Operative walk's timeline is
   byte-exact by construction and its test passed in the whole-binary run. The topped-up
   walks never enter the dodge (tops_up()).
2. **The Agent fight opens crouched at the slit anchor (110.98, −179.31)** (d2-gated). The
   movement inputs (dodge episodes, drift walk-back) stand Bond up into the lintel-window
   launch geometry — the green run's launches all leave from the standing eye y 14.22 [L].
   The crouch blunts the opening doorway-lane bursts (the 11-14 measured lane rounds descend
   y 16 → 14 over x 103 → 111).

## Levers tried and retired (measured worse or dead; the code is reverted)

- d3, the crouched slit anchor: death t29087 — the crouched eye's sight stops on the lintel,
  ZERO launches, clip 4 intact.
- d3, the standing slit + the aim gates relaxed to the sisters' 0.25: death t27383 — the
  pain-1.0 drips with still no launch windows.
- d2, the standing slit with the bias only (no crouch): death t30395 — the −1.40 × 44 drips at
  the slit spot and the killing −49.20 blast land INSIDE the doorway gap, where the dodge's
  strafe (perpendicular to a westward view) cannot leave the round's line.
- d2, the deeper crouched anchor (112.5, −179.4): death t31495 — the drips stop but the
  crouched sight starves the launches and two missile blasts (−20.10/−56.94) close the fight.

## What ran (release, --test-threads=1)

| binary | result |
|---|---|
| walkreal_07000008_live (whole) | 6 passed / 0 failed / 1 ignored (the d3 ledger), 153.6 s |

The passed 6 include the d1 real-damage Operative walk, the plain real-input walk and the
ally tests — the opt-in gates keep their timelines byte-exact.

**What did not run.** The full four-crate suite does not fit this sandbox's disk: the suite's
`cargo test --no-run` build filled the 9.9 GB disk twice (BUILD FAILED at 100% used, the same
constraint the los-mask-repins-08-agent window documented). The changed set is one test file
plus docs/ — no `crates/*/src/` and no `tests/common/` files — so no other level's walk or
library test can move; the per-binary sweep above is the run this window could afford. The
diff touches `crates/nf-view/tests/walkreal_07000008_live.rs`, `docs/WORKLOG.md` (created on
this branch, as every task branch carries its own) and this report.

## What's left

- The 00 Agent walk: the ledger and the next lever are in its ignore message — (1) a
  station-hall route/stance/fight-order change that answers the y-46 mezzanine snipers early
  (36.0 of the 78 budget), (2) pin the machine's route period from the flight log and outlast
  the high phase into the low passes.
- The full-suite pass on the merged final tree, when a sandbox with disk for it exists.

## Usage

not visible to me — this sandbox shows no token, credit or context-window figures.
