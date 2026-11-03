# level-01-agent-hall-fight-2 — the 07000001 contact escort passes on real health at Agent

Branch `s/level-01-agent-hall-fight-2`, cut from `origin/s/level-01-agent-hall-fight` (19cc9be3)
with `origin/master` (f9c1d69c) merged clean at 845e7973. The prior session won the hall fight
(the AND, ch 0x99, at tick 4950) and died at 1.6 health to the last route guards at the
stairwell; this session landed the report's three remaining levers and the walk is green.

## Status: the Agent sister passes

`walkreal_07000001_live::walk_from_the_spawn_to_the_exit_with_the_contact_real_damage_agent`
**passes**: level_exit to 07000002 at tick 14181, end health **35.6** armour 0.0, lowest 35.6,
24 hits for 64.4, 11 return-fire fights, no falls. The prior failure mode (the stairwell bleed
past 1.6) is gone: everything after the hall AND costs 7.3 total (42.9 → 35.6 across the route
pre-clear, the own-radius sweeps, the pre-stairwell sweeps and the plate leg — the route
pre-clear itself ran 42.9 → 42.9 while killing 5/14/15).

## The three changes (all gated `difficulty >= 2` in the level's own test file; real input only)

1. **The door-phase lure's aim tracks the crossing guard's LIVE position** (the report's "the
   middle" move). The old burst captured the aim point ONCE before the loop
   (`let at = [d.pos…];` outside the tick loop) and fired at the placement snapshot while the
   hunter crossed the aperture; the stream now recomputes the aim from his live `d.pos` every
   tick, so the taps ride him through the doorway. The 300-tick bound, the on-target tap
   discipline and the scoped-LOS handover to the fight are unchanged.
2. **The route clears run the driver's 3 → 14 → 5 order** (a new `route_order` helper): the
   corridor's 24-health guard 3, the SMG sprayer 14, the named hall guard 5 first, then the
   pellet-carriers-first Agent order for the rest. Applied to all three sweep loops (the route
   pre-clear after the AND, the escort rounds' own-radius clears, the pre-stairwell sweeps).
   The indices are the difficulty-2 roster's — deterministic across the runs [L the d2 logs];
   dead or absent guards drop out of the priority pass.
3. **A pickup line before the stairwell**: after the route pre-clear, Bond walks the ground-
   level pickups within 9 u of the route line (the west-hall placed pair (−14.4, 6.5, 16.1) /
   (−13.2, 5.6, 10.4) and whatever drops lie there) through the real walk-over take [M
   Pickup_Handler 0x18e650]; two stops ran in the green walk. `resupply()` stays the dry
   fallback.

No port changes: every hit in the log is the game's own pipeline (the prior session verified it
per-difficulty; nothing this session contradicts it). No damage or health was lowered anywhere.

## What ran

On the final tree (this branch HEAD, master merged):

- `cargo test --offline --release -p nf-view --test walkreal_07000001_live -- --test-threads=1`:
  **7 passed / 0 failed** — the Operative master, the real-damage walk, the plate/filter
  sisters, the exit-fade pin, the escort-follow master and the Agent sister all green.
- `cargo test --offline --release -p nf-view --test escort_follow_07000001_live`:
  **1 passed / 0 failed**.
- `cargo test --offline --release -p nf-game --lib`: **656 passed / 0 failed / 5 ignored**.

The fast runner (`tools/test/suite.py`) was started in the background twice (16:14 and 16:25,
the second detached with setsid); this sandbox kills background processes between tool calls
(both instances died within minutes, the second after the report's per-crate runs above were
already done in the foreground). A 1096-binary sweep needs ~148 min here and does not fit the
window under that constraint; the per-binary totals the task names are the three quoted above.

## Remaining / open

- **The door-phase bleed remains** (~33.6: 76.5 → 42.9, 8 hits from cover-firing guard #5 whose
  parts never expose from the south [L the prior probe runs]) — the walk survives it with 35.6
  to spare; a future margin lever is crouching the hold line or extending the lure's track to
  the follow-through fight.
- **The SMG close-range registration question** (carried from the prior report): 32 settled SMG
  rounds at 0.5-2.1 u register zero guard damage while the pistol registers at 3.5+ u. Still
  unfixed/unexplained; the live checks below stand.
- The full-suite total line (see above).

## Live checks for the main agent

- `gdbwatch` on the SMG's bullet spawn during a hip-fired burst at a guard ~2 u away: the game's
  BU spawn position/direction and the hit — the port spawns (last_shot advances) but never
  damages at 0.5-2.1 u while the pistol (0x2) damages at 3.5+ u.
- `fieldprobe` poke: Bond 2 u from a guard, weapon 0x12, fire one round — expected the guard's
  health −1.5×(the difficulty scale); observed in the port: no change.

## Usage

not visible to me — this environment shows no token, credit, or context-window figures for the
session (nothing to copy).
