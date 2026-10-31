# WORKLOG

## level-0b-parachute-step-up (branch s/level-0b-parachute-step-up)

- start (master 3afcdce1). Task: `walkreal_0700000b_live.rs` falls back to a stance sweep for the
  parachute pickup #280 (2.53, 58.45, 42.34 [D]) — the helicopter body's volume blocks the walk
  2.4 m short (cel 189); the pickup floor is 57.92 vs the deck 57.03 at z 42-44. Find how the
  game lets Bond reach it [D the copter's collision/glist and the pickup's touch volume; M
  Pickup_Handler 0x18e650, the player step-up in Player_Collision 0x194df0], fix any port gap
  with evidence, and replace the stance sweep with a walked take. Run walkreal_0700000b_live and
  any *pickup* binary; quote the totals.
