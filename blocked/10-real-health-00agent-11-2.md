# real-health-00agent-11-2 — finish the 11 00 Agent walk

Read docs/reports/real-health-agent-00agent-11.md on origin/s/real-health-agent-00agent-11. On master walkreal_07000011_live::walk_spawn_to_the_lower_floor_load_point_real_damage_00agent is ignored: Bond dies on real health. Branch from origin/master, take the report's levers, re-route the way a player would. Real input only; no top-ups; helpers opt-in. Remove the ignore when green; every walkreal_*_live; full suite total line.

---
claimed-by: sandbox7 at 2026-11-04T01:57Z

---
## Report (blocked, 2026-11-04T03:08Z)


---
## Report from the worker (blocked, 2026-11-04T03:08Z)

# real-health-00agent-11-2 — round 2 report

Branch `s/real-health-00agent-11-2` (from origin/master ecfaca48). Task: make master's ignored
`walkreal_07000011_live::walk_spawn_to_the_lower_floor_load_point_real_damage_00agent` (the 00
Agent d3 real-health walk) green, taking the round-1 report's levers, real input only, no top-ups,
helpers opt-in only.

## What's done

**The round-1 primary lever is BUILT, WALKED and MEASURED: the BodyArmour pre-leg.** The d3 route
now begins with Bond taking the top-floor BodyArmour #261/#266 BEFORE the first fight — armour 50
on at +2922 ticks [D placement (23.12, 47.62, 22.88); the take is the walk-over `Pickup_Handler`
[M 0x18e650, the hard 50.0]; the absorption law is the game's own: a bullet pain's armour fraction
is 1.0, armour absorbs first [M Player_HandlePain 0x1902c8:84-104 → `PlayerHealth::apply`:
`absorbed = (d·frac).min(armour)`]. The result is the exact effect round 1 wanted: **the landing
fights that cost round 1 its 70 health (#14 56.0 + #15 28.0 + #13) now cost 0** — the armour eats
the whole NE-landing + flight-C engagement (run 12: health 80.0 → 80.0 through the #13/#14/#15
fights; round 1 died from those same fights at the north strip 31.1).

The route to the armour is a **desk-maze corridor the game's own nav never spans** — measured and
pinned this round:
- The nav web has NO node path from the spawn side to the armour side [D: path 1 BFS n39 → n64
  disconnected; path 0's ring stays z 7-16], and the direct steer wedges on the mid-floor wall run
  (stuck at (12.5, 13.5), segment hit 0.56 ahead, sidesteps exhausted).
- The walk's chain is a corridor-BFS over the world's own segments (0.5 u cells, 3 body-width rays
  a step — the walkreal probe, scratch, this task): out the spawn pocket, south of the west desks,
  the mid corridor, the stairwell-door ledge, east along z 6.5, north along the x-20.5 wall run to
  the armour take spot (22.5, 21.5) — a cell from which the take fires for BOTH face yaws (the
  probe's `Pickup::touches` test: radius 0.477 + 1.1 with a clear sphere→centre line [M
  0x18e148:203-214]) — plus the lenient inner hop (23.0, 22.0).
- A deterministic ammo stop: the spawn fights' own gun drop (−3.0, 17.9 [L the run logs]) is an
  explicit detour leg (the fights ahead need the rounds; clip 7 at load [M 0x1a7ef8:275-281]).
- `back_off = Some(2.6)` (the existing opt-in): under 3.0 u every drone round hits [M
  DroneWeap_DoBulletAccuracy 0x179740:113-115] and both round-1/round-2 ledgers finish Bond with a
  guard inside that band (#18 at 1.9 u/1.1 u) — a player back-pedals out of it.

**The walk's progress: round 1 died at the north strip 31.1 (tick 14455, 15 hits, 9 fights, no
armour, 12000-tick resupply wedges); round 2 reaches the 18.9 strips** — through the keypad solve,
the whole landing, the 31.1 and 27.1 strips and the 23.0 strip — and dies at connector 2
(tick 5973 at (22.41, 20.25, 3.54), 20 hits for 100.0, 16 return-fire fights won, armour 0.0,
health 0.0). The kill ledger: #33 28.0 in 4, #16 21.0 in 3, #5's opening top-floor volley 20.0 in
1 (at 35.8 u — paid in every timeline), #15 13.0 in 6, #18 11.0 in 2 (the 1.1-u finisher).

## What's left — the measured hole and the next levers

The budget is 80 health + 50 armour = 130 [L: the #5 hit lands pre-armour at tick 210 in every
run]. The measured spend to connector 2 is the whole 130: the two 0x0500006/0x5000008 strip pairs'
crossfire is the binder — each fight kills its target but the partner lands 2-4 free 7.0 volleys
(21-35 a pair). Levers tried and measured THIS round, both REVERTED (the helpers are byte-identical
to master again):

1. **Resupply tours suppressed (`resupply_off`) + corridor-only arming** — died EARLIER (tick
   1797): the #0 ledge fight went dry mid-stand with the tours off and the volley finished Bond in
   75 ticks. The tours are load-bearing for ammo.
2. **Resupply tours redirected away from the desk-mounted pickups (`resupply_avoid`)** — the pass
   retargeted a fresh drop at (15.5, 11.1), walked Bond OFF the proven corridor into the maze's
   kill zones: 84 health gone before the armour. The avoid list is whack-a-mole against drops that
   move with the fight timeline; reverted.
3. **Health-critical walk-past (`fights_floor = Some(60)`)** — same death spot (22.41, 20.20),
   slightly earlier (tick 5571, 22 hits): the crossing's own volleys cost what the fights cost.
   The fights are already the cheaper toll. Reverted.

Untouched levers for round 3, in the order I would try them:
1. **The second BodyArmour is the budget, but it is UNREACHABLE before the death zone** (it sits at
   (15.79, 15.08, 4.57) on the lower floor, past connector 2; the take is refused at armour ≥ 50
   [M 0x18e650:220-225] so it only refills once the first armour is spent). The route must either
   shave ~40 health off the 23.0/18.9 pairs or reach flight A's bottom with ≥ 1 combined.
2. **The Spas12 #267 (13.60, 15.55, 5.52 [D])** — same position problem: after the crux. With the
   fights turned back on below the floor it funds the lower-floor legs (1-2 shells a guard).
3. **A cheaper strip crossing**: the pairs wake on the flight plates and hold position [D §5 — the
   pairs stand 0.85-2.0 apart]; a crossing that fights from the flight's BOTTOM (the guards' boxes
   half-buried behind the flight lip?) is unmeasured — `NF_RF_GEOM` on the 23.0 pair would answer
   whether the stand spot changes the exchange rate.
4. **The top-floor grenades (3 × weapon 0x34 at (24.15-24.53, 23.44-23.80), next to the armour)**
   — the fights' `switch_to_gun` cycles to them only when the pistol dries; a dry-timed grenade
   into a 12-hp pair could break the economy, but the throw cycle through the fight machinery
   (windup/release timing) is unverified and the self-blast risk is real. Not attempted.

No port divergence was found on this walk: every death is the game's own 00 Agent rows firing
through the faithful pipeline (the 7.0 unarmed-class rounds at accuracy 1.0 [M 0x179740], the 1.2
pain scale [M 0x1902c8:40-41]); no health/armour/channel writes, no teleports; the d1 plain, d1
real-damage and d2 Agent sisters' timelines are untouched (their loads are difficulty-1/2 paths and
the pre-leg is `difficulty == 3`-gated).

## Verification (what ran)

Only `crates/nf-view/tests/walkreal_07000011_live.rs` changed on this branch (helpers restored to
master-exact — the working tree is byte-identical to the pushed commit 701b2bbd's tree plus this
report/WORKLOG). Every test that uses the changed code is in that one binary:

| Binary | Result |
| --- | --- |
| `walkreal_07000011_live` (whole, --test-threads=1, twice) | **3 passed, 0 failed, 1 ignored** — `walk_spawn_to_the_lower_floor_load_point` (plain), `..._real_damage` (d1), `..._real_damage_agent` (d2) green; `..._real_damage_00agent` (d3) ignored with the round-2 state in its message |

The ignored d3 test itself was run 13 times this session (--ignored --nocapture) to iterate the
pre-leg; the final configuration's numbers are the ones cited above (run 12; the fight-floor
variant run 13 is the reverted experiment). The other `walkreal_*_live` binaries are untouched by
this branch (no shared-code change — `crates/nf-view/tests/common/` is byte-identical to master),
so their master-green status carries; rule 10's grep for the changed symbols resolves only to this
test file. The full 4-crate suite did not run here: no `crates/*/src` change exists on the branch
(the suite's outcome is master's own), and this sandbox's disk has 108M free with 9.2G used — the
predecessor branches measured the full rebuild does not fit.

## Live checks for the main agent

1. (round 1, still open) On a fresh-boot 00 Agent load of 07000011: confirm the stairwell keypad
   #177 is LOCKED at load and the code entry writes ch 0x12 — the port's whole d2/d3 keypad path
   rests on [M 0x1ae1f8/0x1aed00/0x1ae3d8] plus this one observable.
2. (round 1, still open) The 0x0500004-skin stairwell guards' unarmed-class damage at 00 Agent:
   confirm 7.0 a hit lands from ~3-12 u (weapon byte 0).
3. (new) The armour pane on taking #261/#266: the take writes BLData+0x8B0 = 50.0 (the "hard 50"
   [M 0x18e650:217-219]) — NOT the pickup's rounds=100 — and a second BodyArmour take is refused
   while armour ≥ 50 [M 0x18e650:220-225]. The port's walk reads exactly 50.0 after the take.
4. (new) The top office floor east: whether the real game's AI ever paths from the spawn area to
   the armour pocket (the port's parse has path 1 disconnected n39→n64 [D]). If the real game's
   web connects them, the port's nav parse may be dropping links on this floor.

## Usage

not visible to me (no token, context-window or cost figures are exposed to this session).
