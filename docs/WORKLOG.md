# WORKLOG (branch running copy)

## level-01-agent-escort-fight

Branch `s/level-01-agent-escort-fight` created from `origin/master` (2879088f — the difficulty
feature is on master at 0cb53397; the Agent walks were reverted at the tip because 01/06 were red
on the batch). The difficulty branch is merged, so this branch carries the feature; the sister
tests are restored level by level as they pass.

Task: restore 01's `walk_from_the_spawn_to_the_exit_with_the_contact_real_damage_agent` and make
it pass — the contact-escort fight at Agent (difficulty 2): the unarmed punchers (drone #9, ~10.5
a punch [M DroneWeap_CloseCombatImpact 0x1789c0 → Player_HandlePain 0x1902c8 × Plr_DMod_Normal])
grounded Bond by tick 4023 in the 01-07 branch's run. First the melee law (reach/cadence/damage vs
the port), then the fight tactics (kill order, keep the contact moving, the distance).

Progress:
- [07:55 UTC] Branch created; the reverted level-01 Agent sister restored from
  origin/s/real-health-agent-difficulty-01-07 (the walk body parametrized, the sister test,
  the difficulty assert at load). The prior task (stun-release-no-stun-anims) was closed first:
  report + finish.sh + the ntfy-failure record all pushed on its branch.
