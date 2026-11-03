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
