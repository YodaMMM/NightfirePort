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

## real-health-00agent-11-2 (started 2026-11-04 02:10 UTC)

Branch s/real-health-00agent-11-2 cut from origin/master (ecfaca48). Master's
walkreal_07000011_live::walk_spawn_to_the_lower_floor_load_point_real_damage_00agent (d3) is
IGNORED — Bond dies on real health (round-1 report: 37 d3 drones at load, the 0x0500004 guards'
7.0/round volleys, the 7-round pistol dry by the east connector). Plan: take the round-1 levers —
the BodyArmour take as a pre-leg BEFORE the first fight (23.12, 47.62, 22.88 [D]), the Spas12 #267
on the lower floor (13.60, 15.55, 5.52 [D]) where the route passes it, a dry-fight disengage so a
dry Bond keeps moving — re-route the way a player would, real input only, no top-ups, helpers
opt-in only. Remove the ignore when green; then every walkreal_*_live; full suite total line.

## real-health-00agent-11-2 (round 2 outcome, 2026-11-04 03:20 UTC)

The d3 armour pre-leg is BUILT and MEASURED: the desk-maze corridor-BFS chain (the game's own nav
never spans the east office floor — path 1 n39→n64 disconnected [D]) to the take ring at
(22.5, 21.5), the spawn-drop detour leg for ammo, back_off 2.6 — armour 50 on at +2922 ticks
[M 0x18e650 the hard 50; 0x1902c8's armour-fraction-1.0 absorption]. Round 1's 70-cost landing
fights now cost 0. The walk reaches the 18.9 strips and dies at connector 2 (tick 5973, 20 hits
for 100.0, 16 fights won) — the 23.0/18.9 pairs' crossfire is the remaining ~40-health hole;
three economy levers tried and reverted with their ledgers (resupply_off, resupply_avoid,
fights_floor). Binary 3 passed / 0 failed / 1 ignored (d1 plain + d1 real + d2 Agent green).
Helpers byte-identical to master. Report: docs/reports/real-health-00agent-11-2.md.

## real-health-agent-16-1b-2 (started 2026-11-04 04:02 UTC)

Branch s/real-health-agent-16-1b-2 cut from origin/master (ecfaca48). Task: finish the 16 and
1b Agent (d2) real-health walks — on master walkreal_07000016_live::
walk_the_chasers_defeat_completes_the_mission_on_real_damage_agent and walkreal_0700001b_live::
walk_arrival_to_drakes_defeat_on_real_health_agent are IGNORED (Bond dies on real health —
tactics, not a port gap). Read docs/reports/real-health-agent-difficulty-16-1b-41-4a.md on
origin/s/real-health-agent-difficulty-16-1b-41-4a, re-route the way a player would. Real input
only; no top-ups; helpers opt-in. Remove each ignore when green; every walkreal_*_live; the
full suite total line.

## real-health-00agent-16-1b-2

Branch s/real-health-00agent-16-1b-2 cut from origin/master (22d5836b). Task: the 16 and 1b
d2/d3 sisters — walkreal_07000016_live and walkreal_0700001b_live each keep two ignored
real-health walks (Agent d2 + 00 Agent d3). On current master, with the missile countdown in
(c70343b3a), read the ignore reasons, try the levers they name. Real input only. Done = green
with the ignore removed, or a fresher measured reason.

- [06:50 UTC] Sandbox start: repo + work/ + ISO present (hosting sandbox continued), branch cut,
  WORKLOG restored from the s/real-health-agent-16-1b-2 running copy. Queue worktree refreshed.
  Reading the four ignore messages + the blocked round-2 report next, then fresh baselines.
- [07:15-08:15 UTC] The round outcome. The missile countdown (c70343b3a) re-timed 1b's console
  phase and moved BOTH sisters' deaths: d3 now dies at tick 7590 at the k6 hub wait (30 hits ×
  5.0, the wave ring; consoles 5@4130/4@5288/3@6488, k6 never latched — the tail-only wave
  defense never covers k6), d2 at tick 8628 at the k1/k2 wait (43 hits × 3.5, the wave floating
  y 4.9-10.6 above the station Bond hovers at −4.4). The named levers tried, measured, and
  reverted: the wave-defense extension (same-station fights) SURVIVES the phase (one run: all
  8 latched, health 100/armour 40) but repositions Bond, shifts the wave's intercept and slips
  the UI shots past their reads — the k3 fight → m3 launched [2,2,3,2,2,2,2,2]; the k4/k6/k8
  fights → k6@9170/k8@9446 past the shared +5999 read, m6+m8 launched [2,2,2,2,2,3,2,3]; the
  exposure re-weight (above-term ×20 + the +6 scan) raised the holds but the fight-time cost
  dominates; the Drake early window never got reached. The console phase is DEADLINE-LOCKED
  (~720-tick UI margins vs a 6-shooter wave at ~12/s): the next lever is a route-level re-plan
  (an intercept spot before the wave reaches the stations, or an undisturbed k3/k6/k8 rush
  with the wave fought only on the crossing legs). All changes reverted; the 1b file is
  master+fresh-ledger ignores only. 16: both sisters re-measured on this tree — d3 tick 1030
  (was 1141), d2 tick 1638, both at the ramp foot (12.12,0.93,−88.14) — same law, ignores
  carry the fresh ledgers.
