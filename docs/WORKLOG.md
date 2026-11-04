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

- [05:12 UTC] Round outcome. 16 (d2): the death ledger attributed — 71.4 of the 100 is the
  SOUTH GALLERY (four AR guards, 0x500008c/0x50000b2/0x50000b1, 10-25 hp, patrolling
  z −102..−112 at y 6.7, #1 at y 0.8) plinking free from tick 455 (the ramp foot — their
  fields never reach the spawn legs; they sit above the plain shooters window's dy 6.0 so the
  ramp-foot fights trade with the close ground pair). Four levers measured this round: the
  load-time (48,18) window (the fights crossed the pit from the lane east — dead 1271, one
  fight), the ring-east-gated (48,18) window (the fights TOOK the gallery 1v1 and killed
  #5/#1/#3/#1 on the ring but the ring trades cost 44 — entered the chaser fight at 7.6,
  dead 8435), zero-movement shoot_point stands from the spawn (every guard fails visible()
  from there — the ring structure blocks; timeline byte-identical), static-spot stands at the
  ring east (dead AT the stands, 2189). The ignore message now carries the law + the levers;
  the file is otherwise master-identical. 1b (d2): the current master state measures 36 hits,
  death 9856 (the prior branch's 32-hit state was its own tree); the death ledger — the wave
  ring (six 0x5000078 astronauts at 7.9-10.0 u, −3.5/hit) outpaces the fight's dps, Drake
  (#19, 0x30) 28.0 in 2 hits. The prior round's named lever (the dry window 60 → 30) measured
  37 hits / 9405 — neutral-to-worse, reverted. 1b left master-identical. Both binaries whole:
  16 14/0+1i, 1b 17/0+1i. The other 23 walkreal binaries: untouched by construction (no
  shared-file change this round — the common/ dir is clean). Next round's levers in the
  report: the 16 cover-shaped gallery approach / the gallery's foot access (a live scout);
  the 1b exposure scoring against the spawn band above the deck (the waves spawn at y 10-12,
  the shelter floats at y 3-6 — a hold the down-shooters cannot see).

## creature-ambient (started 2026-11-04 04:40 UTC)

Branch s/creature-ambient cut from origin/master (ecfaca48). Task: Creature_Update, Creature_Move,
Creature_Hurt and Creature_Delete are never mentioned in crates/*/src — find from the decompile and
the disc which on-foot levels place creatures [D], what they do and how they die, and port them with
addresses (movement, hurt, delete; draw stays a stub if a mesh path is missing). If no on-foot level
places one, prove that from the placements and cite it in the code. Done = a real-input test on a
placing level, or the citation; every walkreal_*_live green; the full suite total line.

Progress:
- [04:40 UTC] Sandbox start: repo + work/ (slim) + ISO + nf-queue worktree all present from an
  earlier session; branch cut from origin/master; WORKLOG restored from the running copy, entry
  started. Next: decompile survey of Creature_* and the creature placement census.
- [05:58 UTC] The port is in: `crates/nf-game/src/creature.rs` (Creature_Create 0x134ab8 /
  CalcPoint 0x134cf0 / CalcWaypoints 0x134e58 / Init 0x135208 / Delete 0x135230 / Update
  0x135268 / Hurt 0x1352d0 / Move 0x135340, six unit tests), `Drones.creatures` field +
  the kind-0xE0 attach + the class-0x23 control dispatch (`what == 8`). Fixed the impl
  split (move_tick/calc_waypoints are per-creature). Lib test compiles. Next: lib tests,
  census re-run, full suite, walkreal sweep, report.
- [09:05 UTC] Suite state: nf-view run complete via the per-binary resumable runner
  (cargo's own invocation restarts each window; 948 binaries: 2912 passed / 0 failed /
  318 ignored; every walkreal_*_live, npc_doors_live, fix_npc_doors_live,
  fix_guard_misc_live green). nf-formats + nf-audio + nf-game chunks green (the 3
  disc_golden slim-ISO failures only). The independent decompile audit (subagent, 8
  functions + Spline_Interp + a whole-ELF state-3 store scan) confirmed the 7 load-bearing
  claims and found 6 real divergences, ALL now fixed in creature.rs: A the create-tail
  stall (game sets state 1 THEN calcs at create; the port's deferred calc left rate 0
  forever — initial state is 0 now, deviation 3 reworded), B mode-1's euler ignores the
  chase 0.2 scale, C CalcPoint's f13 and mode-1's off are the EXTENT (+0xD8), not the roam
  radius (+0xC8), D the roam clamp is the mode-0/≥2 arm only, E Spline_Interp's position
  is Catmull-Rom in both modes (mode 0 swaps only the rotation to identity), F deviation
  1's "dead downstream" claim was false — the Mat_GetNorm dir realign persists across the
  CalcWaypoints loop (doc reworded). New mode-1 test (steps = extent, clamp skipped);
  7/7 creature tests green in release.
- [10:08 UTC] Report written (docs/reports/creature-ambient.md): the citation path (kind
  0xE0 ships on no level — census test green on the slim ISO), the 8-function port, the
  audit's 6 fixes, the full-suite totals (1060 binaries: 4102 passed / 3 allowed
  disc_golden failures / 326 ignored; the 81 walkreal+door/guard binaries re-run green
  after the audit fixes). Finishing: push → finish.sh → ntfy.
