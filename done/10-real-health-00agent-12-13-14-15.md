# real-health-00agent-12-13-14-15 — 00 Agent real-health walks for 12, 13, 14, 15

Read docs/reports/real-health-00agent-05-07.md's approach (on origin/s/real-health-00agent-05-07) and the Agent sisters on master. Add a 00 Agent (difficulty 3) sister of each level's real-damage walk, same route and end, `Drones::difficulty == 3` asserted at load. The 15 Agent walk is ignored on master (level-15-agent-doors owns it): build 15 on its Operative walk. Real input only; no top-ups; helpers opt-in. Done = each sister green, or ignored with a measured reason and the next lever; every walkreal_*_live; full suite total line.

---
claimed-by: sandbox1 at 2026-11-04T02:20Z

---
## Report (done, 2026-11-04T03:55Z)


---
## Report from the worker (done, 2026-11-04T03:56Z)

# real-health-00agent-12-13-14-15 — report (2026-11-04)

Branch `s/real-health-00agent-12-13-14-15`, cut from origin/master ecfaca48. Round 1 of the 00
Agent (difficulty 3) real-health walks for 07000012/13/14/15.

## What's done

**The 12 and 13 sisters already exist on master and are green — verified, not assumed.** The
pre-sister master's own real-damage walks on these two levels run at difficulty 3 (the route
needs the Agent+ keypad law, so the master authors picked 00 Agent; the d2-sisters branch kept
the d1/d3 call sites byte-identical):

- `walkreal_07000012_live::walk_spawn_to_the_shaft_exit_real_damage` — d3, `Drones::difficulty
  == 3` asserted at load [M the assert inside `walk_spawn_to_the_shaft_exit_at`; GameState+0x28
  0x2a3790 → Drones::difficulty]. **Green on this branch: the binary 3 passed / 0 failed (25 s).**
- `walkreal_07000013_live::walk_the_warm_chain_to_the_front_entrance_and_out_real_damage` —
  d3 end to end (the chain's one pick rides the warm saves [M GameState+0x28 has no
  transition-path writer]). **Green: 3 passed / 0 failed (35 s).**

Adding a second d3 test per level would duplicate these, so the deliverable for 12/13 is the
verification plus this report; the `_00agent`-named entries exist for 02–11 where the base walk
is Operative.

**The two missing sisters were added** (the 05-07 shape: the same route and end, the fights
HEAD-HUNT, tactics behind difficulty gates):

- `walkreal_07000014_live::walk_the_infiltration_to_the_delta_exit_real_damage_00agent` —
  `with_real_damage_head_hunting || with_agent_path_clear || _at(3)`. The Agent grind
  (back_off 4.0 + lane_fights) and the path-clear stands run behind `difficulty >= 2` gates in
  the body, so the d3 run inherits them. **IGNORED with the measured run** (below).
- `walkreal_07000015_live::walk_spawn_to_the_omega_sector_exit_real_damage_00agent` —
  `with_real_damage_head_hunting || _at(3)`. The d2 tactics (the plank/west-room grenadier
  stands, the doorway guards) sit behind `difficulty >= 2` gates and are inherited; two
  d3-gated grenadier stands were added to the body (the fence-mark stands at 28 u never
  fired — the 0.02-rad aim gate is a torso width there — so the door[38]-north point is in
  the code for the next round). **IGNORED with the measured budget** (below).

Both ignores carry the measured reason and the next levers in the test comments and the ignore
messages, per the task's "or ignored with a measured reason and the next lever".

## The measured failures (both [L] this branch's run ledgers)

**14 (d3) — a wedge, not a death.** At tick 26340 the "1.72 landing" resupply leg floats Bond
AIRBORNE at (33.84, 0.82, 30.95), 0.41 over the scoped floor, d 2.70 short of the drop
(32.50, 0.83, 33.29), `grounded=false`, and the leg cap panics. By that tick the fights had run
40 return-fire engagements for 22 hits / 50.0 health — the d3 grind (pain row 1.0 [M
bullet.rs `damage_factor` 0x14..=0x16 → (0.4, 0.6, 1.0)], accuracy Hard 1.3 [M
DroneWeap_DoBulletAccuracy 0x179740]) displaces Bond off the leg's laid line and the resupply
approach cannot re-ground him. Levers: a d3-gated re-ground before the landing resupply (the
skip-hand-back's settle pattern), a direct `walk_to` the drop's cell, the stairwell AIM-pair
stands (the 05-07 sniper-stand law — the first 50.0 went in the infiltration grind).

**15 (d3) — the measured budget exceeds 100 health before the plank legs.** Two measured,
unavoidable costs: (a) the opening burst **63.0 in 9 hits** from the d3 roster's 0x50000b2
guard behind the spawn (the ledger's #15, (72.35, 41.34, −9.44) at his hits; the return fire
finished him at tick 867). A pre-emptive spawn stand was TRIED and cost MORE: the line from
the gantry is blocked (`segment_to_target Some(0.068)`) and the shoot machinery's flank
through door (81.7, −1.0) burned 56.0 and two 3000-tick caps. (b) the **#526 ankle-beam
rim-shot blast ~69.3 at d3**: the trough hides every farther lane shot [D the rim geometry;
the walk's own shot is from just east of the rim], and the pain row scales blasts [M
bullet.rs `damage_factor` 0x14..=0x16 → 1.0 at d3; grenade.rs:609 routes the blast's player
hit through it; L the d1 27.73 and d2 41.60 ledgers ÷ their 0.4/0.6 rows = 69.3 raw]. 63.0 +
69.3 = 132.3 > 100 before the plank legs, let alone the endgame fights. Levers: the #526 shot
from door[32]'s threshold (~14.9 u down the lane) with the door[32] guard's own dropped 0x1a —
the grenade's arc clears the rim and the case-0x4B chain sets #526 [M Explode_Propagate
0x17c518] at a blast reach of ~0.5 raw; a pre-emptive #15 kill needs his spawn spot and a
clear line (a live scout); the door[38]-north grenadier stands.

## Also found (for the merge owner)

`walkreal_07000014_live::walk_the_infiltration_to_the_delta_exit_real_damage_agent` (the d2
sister, master) does **not** wrap `with_real_damage` — it runs with the health top-up ON while
its name and doc say real damage. Its "pass" pins nothing about d2 real health. One-line fix
(`walker::with_real_damage(|| with_agent_path_clear(|| ...at(2)))`) — left to the owner, out of
this task's scope.

## How it was verified

Per-binary runs on this branch HEAD, release, `--test-threads=1`:

- `walkreal_07000012_live`: **3 passed / 0 failed** (the master d3, the real-damage d3, the
  agent d2 — the 00 Agent real-health walk is the second of those).
- `walkreal_07000013_live`: **3 passed / 0 failed** (same shape).
- `walkreal_07000014_live`: **9 passed / 0 failed, 1 ignored** (the new d3 sister).
- `walkreal_07000015_live`: **5 passed / 0 failed, 2 ignored** (the pre-existing d2 agent
  sister + the new d3 sister).
- **EVERY `walkreal_*_live` binary** (25): 01: 7, 02: 3+1i, 03: 6+1i, 04: 10, 05: 14, 06: 6,
  07: 5, 08: 5+1i, 09: 8, 0a: 12+3i, 0b: 7, 0c: 3, 0c_barrelrow: 1, 0c_eastroute: 1, 0d: 1+1i,
  11: 3+1i, 11_route: 1, 12: 3, 13: 3, 14: 9+1i, 15: 5+2i, 16: 14+1i, 1b: 17+1i, 41: 13, 4a: 3,
  4a_satchel: 2, celprobe: 2, dump: 1 — **0 failures anywhere**; the d1/d2 runs on 14/15
  confirm the d3-gated body blocks disturb nothing.
- NOT run: the nf-game/nf-formats/nf-audio lib suites. The branch touches two nf-view test
  files, the WORKLOG and this report — no lib code — and the sandbox window closed before their
  build+run fit; the sweep above is the binary coverage. (The 05-07 round measured those lib
  suites at 136 passed / 3 allowed slim-ISO `disc_golden` failures on a tree whose lib code
  this branch does not change.)

## What's left

The two ignored sisters (14, 15) with their levers above; both are one focused round each.
12/13 need nothing. The 14 d2 top-up one-liner for the merge owner.

## Live checks for the main agent

1. **07000015, 00 Agent, the spawn-behind guard**: fresh boot at 00 Agent, stand on the spawn
   gantry and walk west — the 0x50000b2 guard south of the gantry (his hits landed from
   (72.35, 41.34, −9.44) [L the d3 ledger]) fires a 9-round burst across the opening leg.
   A gdbwatch on his spawn position (Drone_Create 0x136be0's write) would give the spawn spot
   the pre-emptive-kill lever needs, and a `--bond 94.5,41.4,0.2` render looking at
   (72.35, 41.4, −9.44) would show whether the gantry rail is the line's blocker (the port's
   segment says the block sits 6.8% along).
2. **07000015, the #526 blast's pain row**: gdbwatch `Player_HandlePain` 0x1902c8 during the
   walk's own #526 rim shot at 00 Agent — confirm the blast's player damage enters through the
   pain path (the port routes it through `damage_factor` [M bullet.rs:340, grenade.rs:609]);
   if the game's blast path bypasses Plr_DMod, the 69.3 raw shrinks to ~41.6 and the 15 d3
   route may be feasible after the grenadier stands.
3. **07000014, the "1.72 landing" cell**: a `--bond 33.8,1.0,31.5` render looking at
   (32.5, 0.8, 33.3) would show whether the drop's cell floor is real there (the port's
   scoped floor reads 0.41 below the floating feet).

## Usage

not visible to me (no token, credit, context-window or cost figures are exposed anywhere in
this sandbox session; nothing in the environment reports them).
