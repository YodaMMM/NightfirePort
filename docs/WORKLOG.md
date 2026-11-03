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

## level-01-agent-hall-fight
Started 2026-11-03 13:33 UTC. Branch s/level-01-agent-hall-fight cut from origin/s/level-01-agent-escort-fight,
origin/master merged (clean, 05075e4d). Task: win the hall fight on real input the way a player
would (the escort report's weapon/grenade lever, approach angle, kill order); no health top-ups,
no forced guard state, shared helpers stay opt-in. Done = the Agent walk green, every other
walkreal_07000001_live test green, every walkreal_*_live green, then the full suite total.

Work log:
- (13:33) branch start, WORKLOG entry. Sandbox survived (no reset): ISO + work/ + toolchain + the
  ALSA prefix all in place. Next: fast runner in the background, then the escort report.
- (14:20-15:00) the hall fight, the probe campaign (probe01_hall_research.rs, scratch): the meet
  hand-off gives the SMG 0x12 at clip 32 spare 0 [L the probe dump]; the two dormant ch-0x5 SMG
  guards (10 at the plate, 16 south) never fire on this route (ch 0x5 never rises [L the runs]);
  the SMG's bullets do NOT register on guards at 0.5-2.1 u (32 rounds, aim settled, zero health
  change, last_shot advances — spread 16+5 bloom is tiny, damage 1.5 [L the record dump]) — a
  port bug, filed for the next session. The door trade/funnel/entry are unwinnable: from the
  south the hunters' parts never expose (plain AND scoped probes, door open or shut) while their
  volleys cross, and every aperture crossing cost 30-70 (the funnel concentrates both shotguns).
  THE WIN: the sound lure — Bond holds south of the pre-opened door and fires into the hall; the
  guards hunt the gunshot hearing and cross the aperture ONE AT A TIME (V8 probe: all four named
  + the west pair died in the doorway line, the AND fired at 76.5 health — zero door-phase
  damage). Landed in the level's own file, difficulty >= 2 gated: the 0x2 switch (the deepest
  pool: clip 7 + the type-1 pool 47), the door pre-open + the hold line (7.2,8.8), keep_off 3.2
  (the 6.5 backing re-occludes the aim through the door), the bounded lure + the hold + the kill
  per named guard, the dry ladder gated to resupply-only in the door phase (the ladder's stage
  leg crosses the funnel under fire), the doorway reposition after the AND (the lead's 5.0
  release band [M 0x15eda8 case 3] — the contact's route runs back through the hall, stall
  (5.9,14.1) route 18/37), the route pre-clear (the west corridor's 12/17/3) and the own-radius
  sweeps (the follow legs bled 76.5 -> 42.9 crossing their volleys). Best run: the AND fires at
  4950, the escort runs the FULL route to (−28.2,14.3) (5 u from the plate), the pre-clear kills
  3/14/5, Bond dies at 1.6 health to the last route guards (12/17). The walk is still red at the
  stairwell — the remaining bleed is the door-phase settle gap + the route attrition; the next
  session continues from this branch.
- Next: the report, the full walkreal_07000001 binary, the walkreal_*_live sweep, the suite.

## level-01-agent-hall-fight-2 (started 2026-11-03 16:15 UTC)

Continue origin/s/level-01-agent-hall-fight (19cc9be3) + master merge (845e7973). Task: make
walkreal_07000001_live walk_from_the_spawn_to_the_exit_with_the_contact_real_damage_agent pass —
Bond dies at 1.6 health to the last route guards at the stairwell. The prior report's two drains:
the door-phase settle gap (the crossing guard's volley lands 76.5 → 42.9 in the worst run) and the
route guards' attrition (12/17). The next move: keep the aim on the crossing guard's live position
through the gap, fetch the pickups before the stairwell, set the kill order guards 3, 14, 5. Real
input only; never lower damage/health; helpers opt-in only. Run walkreal_07000001_live,
escort_follow_07000001, nf-game. Suite started on the tree.
