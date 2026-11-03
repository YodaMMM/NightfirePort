# WORKLOG

## level-0d-real-health-crossfire
Started 2026-11-03 03:30 UTC. Branch s/level-0d-real-health-crossfire cut from origin/master
6f16146e. Task: un-ignore walkreal_0700000d_live's walk_arrival_to_the_c5_door_real_damage
(~line 728) — after level-0d-yard-climb-cover's perch kill on sniper #31, the climb and wire
crossfire still outguns Bond's 100 health (the west roof #27/#28/#29, the dock patroller #33,
the south heavy #10, #16 once the gunfire turns it). Check each shooter against the game (the
wake trigger [M], the sight/LOS at that range and through the canopy [M + D coll-scan], the
accuracy/damage at that distance on the current difficulty [M + D]), fix anything the port
fires/sees/hits harder than the game, then walk the route on real input only and un-ignore.

Work log:
- (03:30) branch start, WORKLOG entry; the suite runner next.
- (03:35-04:35) the accuracy audit: the port's hit_chance is a faithful 0x179740 port [L tuning:
  TargetMoving 0.75, Easy 0.6, base 100-5·acc]; the ONE over-hit was the move quad — the port
  zeroed BL+0x10/+0x18 every tick while the game's Vec_Zero pair sits INSIDE `if (substate == 1)`
  where case 1 IS Player_Climb [M 0x193600:129-134 + :43-46] — the wire rider retains the last
  walk's lanes and reads MOVING [M 0x179a48:34-41]. Fixed (the zero gated on SS_CLIMB; the
  climb/creep/zipline writes became the decomp ADD forms [M 0x196310:56,90; 0x1969d0:151;
  0x197f50:121]); the ride's incoming dropped 9-10 → 2-4 hits. The "-0.016 rad flat bias" was
  the scope's free-aim cursor (the auto-zoom + the ±0.4 window; the bullet follows the RAY
  [M 0x1a4a98]) — the calibration probe measured bias 0.0000 with the bit set and the cursor at
  0; shoot_far now aims the RAY. The kill order: the platform kills (#33 +190, #31 +195), the
  ladder-top kills (#16 +209, the roof sweeps) — the real-health survival 4177 → 9412; the
  repositioned roof guard's deck-arrival fire (54) + #10's climb shots (46) still eat the pool —
  the twin re-ignored with the measured reason. Batteries: walkreal_0700000d_live 1 passed/1
  ignored; enemy_accuracy_live 5 passed; nf-game 1040 passed/0 failed. Report:
  docs/reports/level-0d-real-health-crossfire.md.
- (04:40) finish: the report committed (with the ntfy-failed section — all three fallbacks died:
  curl (28) timeout, urllib Errno 101 unreachable, wget exit 124); finish.sh marked the task
  blocked with the report attached; the suite runner (third start, 04:37) was still buffering.

## level-0d-real-health-crossfire-2 (started 2026-11-03 08:35 UTC)

- (08:35) start: branch cut from origin/s/level-0d-real-health-crossfire a37cdff7, master merged in
  (fast-forward to 5ec77aea — master already carried the prior branch's code); WORKLOG restored from
  the prior branch (master strips the process file); goal: un-ignore walkreal_0700000d_live
  walk_arrival_to_the_c5_door_real_damage — the two remaining drains are the repositioned roof
  guard's deck-arrival fire (54) and #10's climb shots (46); real input only, no health/damage
  lowering.
- (09:35-11:25) the -2 audit: the [0d:ray] line in shoot_far (the eye->chest ray through
  segment_hit/segment_hit_static/segment_bullet_scoped every 120 ticks) measured the death
  anatomy — #10 (skin 0x5000093, the BAY guard (214.7,12.1,-289.5)) eats 46 in 3 hits during
  the ladder-top stand (hearing impossible [M 0x1ddc48 outer 40 < d 55.3; 0x1437a0:77-79 the
  50-from-Bond receiver gate; the port carries both], sight legal [D the canopy corner clears],
  45% rolls are the game's own law); #29 eats 54 on the ride+landing at the exact 180 cadence
  (5423/5603/5783/5964). The slab-edge law: the ladder-top eye is 4.9 BELOW the west-roof
  surface — the eye->chest ray dies at 0.972-0.974 (all three probes agree; the deck eye is
  worse), so #29 is unshootable from every route stand while its own descending rounds clear
  its roof edge — the player-vs-drone path difference the prior report suspected does not
  exist in the port (both paths run the same segment_bullet_scoped [M 0x1ea398:216-229]).
  The sanctioned tactics tried and retired with measurements: the ray-gated sweep (an inverted
  gate predicate found and fixed — Some=blocked, None=clear — then the gate still loses: the
  engaged guard steps into the slab shadow, a 3-shooter burst wall at 3709, death on the wire
  4313/5270; the blind sweep is measured-best, RESTORED — death 5964 reproduced exactly); the
  deck-arrival kill attempt removed (never landed, pure exposure); the fixed 500 settle wait
  is a grounded poll now. Batteries: walkreal_0700000d_live 1 passed/1 ignored;
  enemy_accuracy_live 5 passed; nf-game 1041 passed. The suite has no total line — this
  sandbox kills background jobs between tool calls (measured 3x); report:
  docs/reports/level-0d-real-health-crossfire-2.md.

## level-0d-real-health-crossfire-3 (2026-11-03)

- (start) branch s/level-0d-real-health-crossfire-3 from origin/master (cbed0983); WORKLOG restored
  from the prior branch (master strips the process file); reports -1/-2 re-read. Goal: un-ignore
  walkreal_0700000d_live walk_arrival_to_the_c5_door_real_damage. The driver's levers: kill the roof
  guard from a stand whose static eye-to-chest ray is clear BEFORE Bond goes up; take #10 before or
  between the climb shots with the canopy corner as cover; plan the route through any armour/health
  pickups [D the placements]; use the aimed act (DamageLog::aimed_fights) at range. Real input only,
  no health/damage lowering, shared helpers behind per-level opt-in only.
