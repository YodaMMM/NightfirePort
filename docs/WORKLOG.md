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
- (18:15-18:45) the -3 stand scan (probe_0d_ray_stands, assertion-free): 22 stands × 18 guards,
  the matrix rewrote the route — the deck guard #16's ray is CLEAR from the warehouse/dock-edge
  stands (d 56-61) and #10's ray is CLEAR from the strip-mouth stand (196,−228.5) d 63.6 — the
  -2 slab-edge law was measured against REPOSITIONED guards; the PLACEMENT rays from the ladder
  top are CLEAR for #27 (d 70.5) and #29 (d 68.0). The pickups dump [D]: kind-3 armour at the
  start #525, on the west roof (147,21.76,−252.77), and in the annex (192.82,10.79,−272.03) —
  armour cannot absorb this level's bullet pain types [M 0x1902c8:84-104 armourFraction 0], so
  no detour. Route v1: the (178,−212) stand is OFF the walkable floor (fall 9.6 + 14 in hits,
  STUCK — retired); the dock-edge #16 attempt landed nothing (600 ticks — retired); the #10
  strip-mouth kill needed a present-not-enabled finder (the wake leaves #10 enabled false —
  first run's enabled-gated box found nothing, #10 alive, death 4072 at the ladder top). Route
  v2: #10 killed at the strip mouth (+201, pool 100→100 — zero incoming), death 5964 → 7270;
  the -2 blind sweep picked #29 first (kill +200) then burned 7 passes on the blocked-ray #28
  while #27 (CLEAR ray) sat untouched — the sweep is now explicit placement order (#29, #27,
  #28 best-effort) and #18's box rides its live-patrol band (it walks the bay east-west:
  207.26 → 212.19 measured).
- (18:50-19:25) the route end-to-end iterations (real-damage twin, ~5 s each): v3 sweep boxes
  found nobody (the guards had REPOSITIONED off the placements to x ~151 — the wake moves them);
  v4 (live-spot boxes) 1 kill + the two settled survivors (#28 north −238.3, #29 south −255.8)
  ate the ride for 94, death 6778; the trench #18 detour FAILED (the live-spot ray dies — the
  in-room geometry only clears from inside the bay) and cost 1200 settle ticks — REMOVED; v5
  (3×900 walker-catching passes, sweep BEFORE #16) — **ALL THREE roof guards killed**
  (+222/+754/+193) and the ride went SILENT; the new killer was #32 (the ground spawner, weapon
  0x1e, patrolling (157.6..158.9, −213..−229)): 124 hits for 86.4 across the bay/beam/yard/C5
  (d 92!), death 16287 at (197.95,10.56,−278.88) — the deepest yet (the annex descent); v6: the
  #32 patrol-band box (158,−220)±(5,11) found it (157.59,−213.01) but the strip-mouth ray DIES
  at 0.960-0.965 on a ~1.7-high ground obstacle at (160,−217) (Bond's eye 1.7 clips it; #32's
  own rounds clear from its higher eye — 0-damage return hits measured) — the #32 kill stand
  needs a higher eye or an east approach, NOT DONE. The in-bay #18 fight also failed (400
  ticks — the patrol spot vs the box). The non-damage twin + the batteries: NOT RERUN this
  iteration (the stop line); the route code is additive kill legs, the shared helpers untouched.

## level-0d-real-health-crossfire-4

Branch `s/level-0d-real-health-crossfire-4` cut from `origin/s/level-0d-real-health-crossfire-3`
(9f9f9631 — master f3b24398 already contained via aa8f67d2, the merge is a no-op).

Task: the -3 route breaks a master-green test — walkreal_0700000d_live::
walk_arrival_to_the_c5_door_real_damage dies at tick 4482 at the ladder top (213.07, 17.37,
−234.51), five −16 sniper hits from #18 (207.6, 12.1, −288.6), #16 (198.9, 14.0, −260.4) and
#28. Keep the C5 walk's old route (or make the new one survive there) and finish the crossfire:
deal with #32 at the annex descent. Then every test in walkreal_0700000d_live listed, every
walkreal_*_live, the full suite total line. Real input only; no top-ups.

Progress:
- [19:09 UTC] Branch start; WORKLOG entry. Next: the -3 report re-read and the C5 walk repro.
