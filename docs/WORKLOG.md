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
- Ported the faithful NoBunching pieces behind NB_SCAN_LIVE: the obstacle's movement-lock refusal [M 0x150ad0:22] (a locked nearest neighbor returns 0, nothing stamps — the Steer filter has no lock term), the +0x17 return byte (`no_bunching_hit`, ControlSTANDARD 0x1490f0:225-226 → ChooseCombatMove 0x150c18:58's cVar1) and re-verified the blockers with the scan LIVE: walkreal_07000001 passes 5/0 with the flip (the old 01 blocker is stale), but walkreal_07000014 regresses 6/0 → 4/2 (two corridor legs wedge at the 6000-tick cap, stash-verified pre-flip pass) — the flip stays inert pending that walk's live re-pin; the release-to-door divergence refined: only #783 routes to the door, the group-9 pair #778/#780 wake into no-route combat at their spawns and shuffle ±1.8 (dump + [D audit] spawns in the test's ignore note).
