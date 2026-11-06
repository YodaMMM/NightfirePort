# 03: the Agent walk after the door reroute

Branch from `origin/s/door-walk-03`, merge current `origin/master`, push to `s/door-walk-03b`. Read docs/reports/door-walk-03.md.

Run on that branch merged with master a6665d115: `walkreal_07000003_live` 5 passed, 1 failed, 1 ignored. The plain walk and real_damage both pass, so the unverified front-gate edit works. Left: `walk_the_full_route_and_complete_every_objective_real_damage_agent`, Bond died tick 52087 at (23.39, 6.37, 11.87).

Fix that one walk test-side. The reroute changed the fight order, so read its death ledger first. The 06 `stand_shooters_down` sweeps are a candidate if the far shooters sit past the 20 u window. Do not touch door, collision or game code.

Also say whether Bond falling out of the world needs a game fix: the report says he fell to y -62000 off the route. Where did he leave the geometry, and could a player do the same? Give the exact spot; do not fix it here.

Finish with the whole 03 binary.
