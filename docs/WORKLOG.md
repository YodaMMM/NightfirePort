# WORKLOG (branch running copy)

## smg-close-range-hits

Branch `s/smg-close-range-hits` created from `origin/master` (f3b24398).

Task: Bond's SMG (full-auto) rounds do zero damage to guards at close range on 07000001 — 32
rounds at 0.5-2.1 u, aim settled, rounds spawned (`last_shot` advances), zero damage, while the
pistol (0x2) hits the same guards at 3.5-9 u (docs/reports/level-01-agent-hall-fight.md on
origin/s/level-01-agent-hall-fight, "Remaining" item 3). Candidates: the first-segment spawn
origin for full-auto records (`fire_player`'s `shot_from` in crates/nf-view/src/scene.rs, the
muzzle/rocket-origin midpoint path) starting the segment past the target, or the drone hit-box
test's near-field gate. Port the game's rule with addresses [M Bullet_init 0x12c5a0,
Bullet_update 0x12e7b0, the drone collision handler].

Done = a real-input test: Bond fires the SMG at a guard 1 u and 2 u away and the guard takes the
record's damage per round; the same for the pistol as a sister; every walkreal_*_live green; the
full suite total line.

Progress:
- [18:26 UTC] Sandbox start: repo + work/ + ISO present; branch created from origin/master,
  WORKLOG started. Queue worktree to refresh next; then the level-01 report re-read and the
  bullet/drone code study.
- [19:05-20:05 UTC] The premise REFUTED by the repro: the SMG 0x12 registers per-record damage
  at 1.12 u (3 rounds → 3 impacts, 2.25/round, drop 6.750 == expected) and at 1.88-1.90 u, the
  pistol sisters land at 1.09/1.87/5.89 u — while the ch-0x5 "dormant" pair takes NOTHING at
  0.78-1.02 u because the game ITSELF holds them unhittable: NDrone2_DSTATE_WaitSwitch 0x15b960
  ENTER → NDrone2_Enable(0) 0x149dd0 sets obj+0xF0 |= 0x20 and Collide_PickObj 0x1eae80:26-30
  drops those — the port's skip is that rule, not a bug. The level-01 report's 32 zero-damage
  rounds measured the script hold. Prober's trap found: the escort's own 0x1c volleys land in
  the same player_impacts/dr.hit records (rec-dmg 3.0 vs 3.5 identifies them) — the test
  attributes by hit.weapon. Test: crates/nf-view/tests/smg_close_range_live.rs (ok, 18.23 s);
  no src/ changes on this branch. Report: docs/reports/smg-close-range-hits.md.

## real-health-agent-00agent-11 (started 2026-11-03 21:45 UTC)

Branch s/real-health-agent-00agent-11 cut from origin/master (68298698). The difficulty-01-07 line
(origin/s/real-health-agent-difficulty-01-07) is already merged into master — no merge needed.
Plan: add the Agent sister (difficulty 2) and the 00 Agent sister (difficulty 3) to
walkreal_07000011_live, loaded the menu way with load_with_carry_difficulty(..., d), asserting
Drones::difficulty == d at load; check the difficulty rows the game reads — the pain scale
(Player_HandlePain 0x1902c8), the accuracy (DroneWeap_DoBulletAccuracy 0x179740:74-88), the
min-difficulty spawns (Drone_Create 0x136be0:49) [M]; where Bond dies read the damage log against
that pipeline, fix the port where it differs (with the address) else the tactics; Operative walk
stays green; real input only.

## real-health-agent-00agent-11 (round 1 outcome, 2026-11-03 23:18 UTC)

The Agent (d2) sister walk_spawn_to_the_lower_floor_load_point_real_damage_agent is GREEN
(load_with_carry_difficulty + the Drones::difficulty assert; the keypad #177 solved on real input —
created LOCKED at d2/3, Lock_Create 0x1ae1f8:67-69 auto-unlocks only at d1). The 00 Agent (d3)
sister is RED at the east connector (tick 14455): the d3 min-difficulty spawns
([M Drone_Create 0x136be0:49]; 37 drones at load) outpace the 7-round pistol — the 0x0500004-skin
guards' 7.0/round volleys (accuracy Hard 1.0) drained 84 health before the first kill-drop
resupply. Built the opt-in lane_floor_drops band (the fight drops rest ON the floor ray, 1.09
under Bond's pos y — the lane 0.5 band could not see them); with it the NE-landing pass walked
onto a fresh kill drop and armed clip 30. The remaining levers listed in the report.

## real-health-00agent-01 (branch cut, 2026-11-04 00:10 UTC)

Task: the 00 Agent (difficulty 3) sister of walkreal_07000001_live's real-damage walk —
Drones::difficulty == 3 asserted at load; the difficulty3_07000001_live pins and the
level-01-agent-hall-fight-2 report are the reading; the Agent walk belongs to
level-01-agent-stairwell-doors, reuse what it lands; guards take their door routes on master
(door 2 at (-15.0, 6.3, 14.1)). Real input only, no top-ups, helpers opt-in. Branch cut from
origin/master a71f9233.

## real-health-00agent-02-03-3 (branch cut, 2026-11-04 00:20 UTC)

Task: the 02 and 03 00 Agent d3 walks under the door routes. On master both sisters are
ignored; 03 passed at t59079 (health 28.8, exit loads 07000004) before the guards took their
door routes and now dies at tick 59112 near (-1.38, 5.96, -34.47) to #16 (skin 0x5000038,
weapon 0x1c); 02 still dies in the courtyard kill zone (the stair/lane sniper 0x1e, 48.0 a
hit). Branch cut from origin/master a71f9233 (the fire_through_disabled and switch_dry opt-ins
are there). Real input only, no top-ups. Remove each ignore when green; every walkreal_*_live;
full suite total line.
