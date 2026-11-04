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

## level-08-00agent-doors (branch cut, 2026-11-03 22:31 UTC)

Task: re-route the late-mission leg of
walkreal_07000008_live::walk_the_whole_mission_on_real_input_real_damage_00agent (ignored on
master after the door-routes merge: Bond dies tick 25773 near (113.19, 28.28, -122.97) to #20,
skin 0x500002f, weapon 0x15, 9.8 u, 6.0 a-hit) the way a player would — kill #20 first, cover,
approach. Real input only, no top-ups. Operative and Agent walks stay green. Branch cut from
origin/master 38052bf6.

## level-15-agent-doors (branch cut, 2026-11-03 22:35 UTC)

Task: re-route
walkreal_07000015_live::walk_spawn_to_the_omega_sector_exit_real_damage_agent (ignored on
master after the door-routes merge: a -41.6 world hit at tick 5400, -22.5 from #11 skin
0x50000b2 at 6191, death to #5 skin 0x500008c at 7885 near (-0.02, 41.32, 46.87)) the way a
player would — identify the world hit (blast / hurt volume / live wire) and why the door
change moved Bond or a guard into it, then cover + approach on real input. Real input only,
no top-ups. The Operative walk stays green. Branch cut from origin/master 38052bf6.

- (00:10 UTC, 11-03) The -41.6 world hit IDENTIFIED [M Explode_Propagate 0x17c518's sweep:
  `damage * (1 − (int)dist / radius)`, the (int) truncation 0x17c61c-0x17c628, no LOS, no wall
  check]: it is mine #526's OWN blast — the walk's rim shot from 2.44 u, radius 15 damage 80,
  ×0.6 Agent pain (80·(1−2/15)·0.6 = 41.6; the Operative walk pays the same blast at ×0.4 =
  −27.7 and stays green) — NOT door-related. The door routes' actual delta is the chips (#15
  the gantry grenadier −2.1, #17 the plank burst rifle −5.4, #2 the desk grenadier −4.2). The
  measured death ledger (v1): 2.1+6.4(#462's own blast at 13.57 u)+5.4+4.2+41.6+22.5(#11's
  grenade in flight)+17.8(#5's grenade in flight) = 100.0 exactly. Route work so far: the 733
  pre-shoot moved to the door[50] threshold (VERIFIED: #5 died before its throw — 17.8 saved),
  the #10/#11 pre-shoots moved to the rim mark (partly effective: #11 4.2 vs 22.5; #10's fight
  still dragged 21.0), the #462 east mark and the #526 west-lane shot tried and REVERTED (the
  probe15e dump: the bullet trace to #526 is rim-blocked from every stand x ≥ 14, so −41.6 is
  the geometric floor; the east #462 mark saved 6.4 but reshuffled the fights to −21.0). v7
  ledger: died at 7502 at door[42] with 100.0 — the fence chain (#447 at 9.5 u) pops −4.8/−4.3
  and #10's drag 21.0 are the remaining 30. Agent test still RED; the Operative real-damage
  sister re-run GREEN on the branch; the rest of the binary unverified. Next: kill #10 in the
  first rim-mark fight (hp 20, one head hit), find the fence-pop law (radius ≠ 8?), re-check.

## level-15-agent-doors-2 (branch cut, 2026-11-04 09:45 UTC)

Task: finish the 15 Agent walk from s/level-15-agent-doors (merged origin/master) — the ledger
sits at exactly 100.0 with ~30 to find: #10's first rim-mark fight drag (21.0, the fight gave up
on its cap instead of the one head kill), the #447-fence pop chain (9.1, the law unpinned), and
the optional #462 east mark. Real input only; no top-ups; helpers opt-in. Remove the ignore on
walk_spawn_to_the_omega_sector_exit_real_damage_agent when green; every walkreal_*_live; the
full suite total line.

## level-15-agent-doors-2 (done, 2026-11-04 10:52 UTC)

The Agent walk is GREEN un-ignored — both remaining leads landed inside the walk, no sim-code
change, no helper change (tests/common untouched):

- **The #447/#448/#449 fence pops pinned and dodged [L the run probe]**: all three mine records
  carry damage 80, radius 10 (the v8 run printed them from the sim — `Mine::damage`/`radius`,
  inst+0x1C/0x20 [M worldobj.rs's Mine]); the measured pops at the old threshold stand were
  80·(1−9/10)·0.6 = 4.8 and 80·(1−8/10)·0.6 = 9.6 ×2 = 24.0 together [M Explode_Propagate
  0x17c518's `(int)` truncation]. The shot is now taken from a mark 2.2 u north — 11.1 u off
  #447, ≥ 10 u off all three — the pops drop to zero (the truncation floor), the assert's
  range rule (d > 8) holds with room.
- **The west-room grenadiers answered by PROXIMITY at the jog**: the rim mark's placement-spot
  pre-shoots missed (the door routes walk the guards off their spots — the v7 run's
  agent_drone_near(8.0) found none), and #10's return-fire stands cost 21.0. The sweep now
  finds the nearest live hostile within 14 u of Bond at the jog and repeats while one stands
  (the measured run took #11 at 12.5 u); #10's residue fell to 8.4 (his own fire during the
  jog leg's return fire) and the walk saves 12.6 net.
- The green run: exit USE #445 reached, Bond crosses at 26.2 health (73.8 taken of 100.0);
  `walkreal_07000015_live` whole: **6 passed 0 failed 1 ignored** (the 1 = the d3 00 Agent
  sister, ignored for its own documented reason — the d3 budget exceeds 100 health).
- Every `walkreal_*_live` binary on this branch: **28 binaries, 170 passed, 0 failed, 15
  ignored, 0 red** (01: 8/0/0, 02: 3/0/1, 03: 6/0/1, 04: 10/0/0, 05: 14/0/0, 06: 6/0/1,
  07: 5/0/0, 08: 5/0/1, 09: 8/0/0, 0a: 12/0/3, 0b: 7/0/0, 0c: 1/0/0, 0c_barrelrow: 1/0/0,
  0c_eastroute: 3/0/0, 0d: 1/0/1, 11: 3/0/1, 11_route: 1/0/0, 12: 3/0/0, 13: 3/0/0,
  14: 9/0/1, 15: 6/0/1, 16: 14/0/2, 1b: 17/0/2, 41: 15/0/0, 4a: 4/0/0, 4a_satchel: 2/0/0,
  celprobe: 2/0/0, dump: 1/0/0).
- The full suite: nf-game **1048 passed 0 failed 8 ignored**; nf-formats+nf-audio **136 passed
  3 failed** (the 3 = the slim-ISO disc_golden allowance, /DRIVING/MIS01.MUS BIGF). The rest of
  nf-view's non-walkreal set did NOT fit this sandbox window (the test-binary links alone
  outlast the remaining time; disk filled to 89% again). The next session can finish it; the
  code surface this task touched is the one test file.
