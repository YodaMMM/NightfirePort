# WORKLOG

## drone-nobunching-side-bits (branch s/drone-nobunching-side-bits)

- start (master fd24fbf4). Task: level-14-no-route-hold traced 14's last door-guard
  divergence: in the game the guard with no route holds COMBAT_NO_ROUTE 0x5f because the
  Can* move probes refuse on the +0x31C side bits that `DroneMove_NoBunching` 0x150ad0
  stamps every tick [M via NDrone2_ControlSTANDARD 0x1490f0:202-226]; the port's mirror is
  inert (NB_SCAN_LIVE), so ChooseCombatMove picks the step-aims. Port the NoBunching scan
  and the side-bit stamp faithfully [M 0x150ad0, Steer 0x14eba8:83 pairing radius 3.0, the
  Can* readers], un-ignore
  `door_guards_locked_door_07000014_live::the_door_guards_hold_combat_no_route_like_the_game`,
  and keep every walk green. Run nf-game, every *guard*/*drone*/*combat*/*door* binary and
  every walkreal_* binary; quote the totals.
