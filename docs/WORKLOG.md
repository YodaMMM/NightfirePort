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

## real-health-agent-difficulty-16-1b-41-4a (branch cut, 2026-11-03 23:08 UTC)

Task: Agent-difficulty sisters (difficulty 2 via load_with_carry_difficulty, the menu way
[M C_SBNFDFCTY 0x216ca8:15]) of the real-health walks for levels 16, 1b, 41, 4a — same route,
same end, Drones::difficulty == 2 asserted at load; port fixes where the damage log diverges
from the per-difficulty pipeline, otherwise tactics (cover, pickups, kill order, timing).
Real input only, no teleports, shared helpers untouched unless per-level opt-in. Branch cut
from origin/master 38052bf6 (the 01-07 sisters branch is already merged — merge-base confirms).

- (00:25 UTC, 11-03) The four Agent sisters wired (each walk body parametrized `_at(difficulty)`,
  the difficulty-1 call byte-identical, `load_with_carry_difficulty(..., 2)` the menu way, the
  Drones::difficulty assert at load [M 0x2a3790]): 16's finale, 1b's station, 41's two walks,
  4a's ride. Results — 4a GREEN (zero damage: the ride has no armed hazards on the route), 41
  both GREEN (zero damage), 1b RED and 16 RED. All four d1 walks re-run green on the branch.
  No shared-helper changes: the 1b tactics live in the level's own file (fight_opts/fight/
  play_the_mission_opts/hide_and_snipe, opt-in via `waves_first = difficulty >= 2`); no
  crates/nf-view/tests/common/ edit; no crates/*/src edit.
- (00:25 UTC, 11-03) The damage logs match the game's per-difficulty pipeline everywhere — no
  port divergence: 1b's astronaut rounds 5.0 × Plr_DMod_Normal 0.7 = 3.5 [D weapon 0x6a +0x0c;
  M 0x1902c8; 0x1d3a80 case 0x700001b]; Drake's own gun −14.0 = the 0x30 record × 0.7 [D]; the
  16 AR rounds the records × Normal. The deaths are tactics, not port bugs.
- (00:25 UTC, 11-03) 1b's grind (three iterations, each logged): the Drake fight's aim pick
  ordered Drake (r−50) over threatening waves (r−20) — the wave ring ground 43 hits for 100.0
  through the stand; waves-first scoring (threat r−30) → 41; the exposure-scored shelter hold
  (fight_opts re-scoring every 30 ticks, a prio-blind hold penalized 50) → Drake died-starved
  (the 6000 cap, no shot); the shelter = hide_and_snipe's cover point as the Drake hold →
  39 hits; the Drake-close band (r < 8 → r−60, his 0x30 gun is four waves' worth per hit) →
  32 hits, then 36 (Drake capped at 2×−14.0). Best run: died 10695, 32 hits. Next: the fight
  needs Bond's dps up (the ammo/reload cadence) or the waves' sight-lines held down longer.
- (00:25 UTC, 11-03) 16's ledger: the APPROACH fights cost ~86 (44 hits: #3 35.7/17, #5 16.8/4,
  #2 10.5/5, #1 8.4/4) before the pit phase; the pit-door guard pre-shoot (skin 0x5000093 at
  the death log's spot) added before the chaser kill. Next: the approach's dragged fights (17
  hits from one guard = the fight never won the trade) need the fight machinery's study —
  cover, or the guards' own drops armed earlier.
- (00:25 UTC, 11-03) The full 4-crate suite did not fit: the background suite.py was killed
  between tool calls (the sandbox's known behaviour), the chunk runner's target-cache build
  filled the disk (3.8 MB free, the linker failed; the cache wiped), and the repo-target
  4-crate build alone exceeds the window's remainder. The four touched binaries ran instead
  (the numbers above); the other levels' walks are untouched by construction (no shared file).
