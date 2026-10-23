# The escort lead's mission re-path (`NDrone2_ReFindMissionPath` 0x153a30)

The escort lead's cover behaviour leaves the mission route (`Drone+0x970`, the port's
`Drone::route`) stale: `FindRunToPoint` 0x153168 invalidates the combat route (+0x860) and runs the
lead to a group-9 AI point, off the escort route. The game re-joins the lead at the hide's exit;
the port used to resume the stale route and wedge. This page pins the rule.

## The state flow

- `DSTATE_AllyLead` 0x15e698 case 3: armed hostile within 10 → `FindRunToPoint` → `+0x5A2 = 0x1f`
  (LeadHide) + `SetState(0x28 AllyGoToGoalPosition)` [M 0x15e698:93-99]; none →
  `UpdateMissionRoute` (= `FollowRoute(+0x970)` [M 0x154a80]) and the Bond bands.
- `AllyGoToGoalPosition` 0x15f938: `MoveToGoalPosition` → arrived (3) → the return state `+0x5A2`
  (LeadHide) [M 0x15f938 case 3].
- `DSTATE_AllyLeadHide` 0x15ec08: cower until `NotNearArmedDrone(14)` (0x41600000) latches a
  2-tick timer C [M 0x15ec08 case 3]; **timer C (case 0xc) runs `ReFindMissionPath` FIRST; its
  non-zero return → `SetState(0x20 AllyLeadWait)`, its 0 leaves the state (the call already parked
  the drone in state 99 or aimed it at the player)** [M 0x15ec08:56-62]. The old port arm went
  straight to AllyLead 0x1d — the walkreal_07000001 arrival wedge.

## The re-path itself [M 0x153a30]

The mission route record sits at `Drone+0x970`, so the route's own node index `+0x80` IS
`Drone+0x9f0` and its node count `+0x82` IS `Drone+0x9f2`; the ordered node list `+0xf0` (`+0xa60`)
indexes the path's node array `+0xf4` (`+0xa64`, 0x40-byte nodes, position at `+0x10`). The scan
walks the whole node list:

- reach test: `NDrone2_MoveTest(feet, node, 0, drone, 0, 0, 1)` [M :235-237] — the gate pair (0,0)
  runs no post band and records no hit; skipped entirely under the movement lock (`+0x4f8 & 0x10`)
  [M :225]; a node is only tested while no candidate passed and its SqDist2D < 400.0, or when
  closer than the incumbent [M the `(uVar13 == 0xffff) || (fVar15 < fVar16)` gate, `fVar16` from
  400.0] — the reachable node nearest the drone's position wins;
- the closest node by SqDist2D is tracked regardless (the first unconditionally) [M :249-267].

Then (the anchor being reachable, else the closest, else none):

1. reachable anchor [M :172-209]: `+0x9f0 = anchor`, `LinkCreep_Calc(route, 0)` (its Sc arg is the
   back flag [M 0x159b10:16,89] — the creep re-calcs at the route's index) + the `Drone+0x670`
   refresh via `LinkCreep_Dest(route, 0)` [M :176-201], `InvalidateRoute(+0x860)` twice,
   `SetupGoalPosition(2.0, node)` + `MoveToGoalPosition`, return 1;
2. unreachable-but-closest anchor [M :104-158]: same anchor work, `if (route+0xd8 == 9)` re-resolve
   `Drone+0x954 = NavPathForPosition(feet, 1)` (not modelled — the port carries no route+0xd8,
   *suggested*), `ResetRoute(+0x860)` + invalidate, `SetupGoalPosition(1.0, node)` +
   `MoveToGoalPosition`; move code 0..3 → `SetState(99)` (the guards' `DSTATE_GoToGoalPosition`,
   the port's `GO_TO_GOAL` 0x63) and return 0;
3. no waypoint or the waypoint route failed [M :160-169]: goal = the player
   (`SetupGoalPosition(2.0, player)`), return 0.

`SetupGoalPosition` 0x15acf8 stores the accuracy on the AITarget (`+0x14`), not on the drone's
arrival threshold `+0x8ec` — that stays the `DefaultInit` 0x14b300:579 `2.0f` [M], so the port's
fixed 2.0 arrival test in `move_to_goal` is faithful.

## The arrival question (the "within 2.0" wedge)

`NDrone2_MoveToGoalPosition` 0x1517d8 returns 3 when `AINetwork_GetRouteDistance(route) < +0x8ec`
[ M 0x1517d8:50-54]. `GetRouteDistance` 0x156320 is 2-D throughout (Vec_Dist2D): feet → next node
+ the links' stored lengths + last node → route dest `+0x40`; with no next node, the straight 2-D
feet → dest [M 0x156320:47-106]. The port's `Route::distance` mirrors this, so the observed
"return 3 at (3.10, 6.34, 7.98) with the AI point (2.95, 5.35, 9.94) a level below" (direct route,
dxz 1.966 < 2.0) matches the game's rule GIVEN the port's route build: its documented `move_test`
deviation (static collision line of sight at +1 in place of the game's `Collide_StraddleCels` +
boundary-ring walk, `AINetwork_BoundsTest` 0x157d80) lets the drop-edge segment pass. The fix does
not depend on which way that lands: whichever state the cover run ends in, the hide's exit
re-anchors and the lead resumes.

## Tests

- `escort_hide_findrun_live::the_hides_exit_reanchors_the_mission_route_before_the_lead_resumes`
  — through the real path (`nf_view::load` + Walker ticks): the hide's exit re-anchors
  `route.index` to the game-scan anchor and rebuilds the +0x860 route to it. Fails on the old arm
  (index 2 where the follow stopped vs the game's 6 — verified by stashing the fix).
- Sister pins around it: `escort_still_leads_after_the_hide_arm`, `escort_cover_live`,
  `escort_follow_07000001_live`, and `walkreal_07000001_live` (3/3, was the wedge panic).
