# WORKLOG

## level-0c-route-teleports (branch s/level-0c-route-teleports)

- start (master e79794da). Task: walkreal_0700000c_live's full walk teleports Bond three times —
  out of a shaft he can drop into but not climb out of, onto a mezzanine reached (as far as the
  tests show) only by the Q-Claw from the warehouse, and onto an east deck no collision connects
  to. For each: find the game's route [D collision + placements: ladders, grapple points, lifts,
  doors; M the classes], fix any port gap with the address cited, walk it on real input; a
  grapple route drives the real Q-Claw. Delete each replaced teleport; keep the rest with a
  sister assert and name the live check. Run walkreal_0700000c_live,
  walkreal_0700000c_eastroute_live, interactables_0700000c_live and nf-game; quote the totals.
