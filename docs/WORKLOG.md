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
- [08:15 UTC] The driver's premise REFUTED by the hit trace: no melee ever landed on Bond. The
  death-log's "weapon 0x0" labels are the dead drones' RESET weapon (the summary re-reads
  drone.weapon after death); the real killers were bullets — #9 (pistol 0xe: 7.0, ×2 TooClose
  14.0 at 1.7-2.7 u), #13 (SMG 0x12: 21 pellet hits 1.05/2.10 grinding at 1.3-4.4 u for 600
  ticks), the rest pistol/shotgun rounds. The melee pipeline itself verified against the
  decompile (reach 1.75/2.0 [M 0x1789c0], the 60° facing gate, records 99-102 damage 50/15 [D
  probe]) — no divergence, and it never fired in either difficulty's run.
- [08:25 UTC] The clearing fights FIXED with player tactics (all gated difficulty >= 2, the
  Operative timelines byte-identical): the pellet carriers killed first (SMG 0x12/shotgun 0x1c
  before the pistols), the approaches stop at 8 (not 3.0-3.5 — the old runs walked INTO the
  ×2 band), keep_off/back_off 6.5. The three clearing fights: 79.4 damage → 23.5. All
  Operative masters green after the gate.
- [08:30-08:55 UTC] The hall fight (the named ch-0xE crowd) is the remaining blocker, mapped
  with the geom dump: the crowd hunts the hall's south door (7.9,11.1) — the only nav entry
  from the meet room — in full cover; from the south their parts never expose (plain AND
  scoped probes) while with the door OPEN their rounds cross the same plane (timings
  consistent, no port bug found); the door funnel's nav hops cannot stop short (the 8.0
  arrive overshot to 0.7-0.9); the west door #2 is the known nav wedge; the doorway trade at
  6.8 u lost on DPS (two 8-pellet shotguns vs the pistol). Tried: approach-8, the door-side
  hold, the west flank, the south-door lure, the in-door stand-and-trade — all documented.
- [09:00 UTC] Final state: 6/7 walkreal_07000001 green (every Operative master, real damage
  included), the Agent sister red at the hall (tick 3366, the stage walk into #6 at 0.5 u).
  nf-game lib 655 green. The sister + the hall analysis ride the branch for the next session.
