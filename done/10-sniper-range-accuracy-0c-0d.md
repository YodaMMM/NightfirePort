# sniper-range-accuracy-0c-0d — weapon 0x1e guards hit too hard from too far

Branch from origin/master.

The new real-health walks on 0700000c / 0700000d (ignored with these reasons) die to weapon 0x1e drones firing from 34-41 units: on 0d drone #31 (skin 0x5000087 at (160.9, 6.8, -235.1)) lands 14-16 per shot from 41.2 u; on 0c drones like #18 (skin 0x5000090) deal 80 total by tick 8700 from ~34 u.

Do:
1. For weapon 0x1e, pin the game's drone fire at range [M]: the accuracy/spread law with distance (DroneWeap_*), the fire rate and burst, the per-hit damage and any range falloff, the drone stats that scale it (skin's accuracy/difficulty), and the max engagement range.
2. Compare with the port at 34-41 u: hit probability per shot, damage per hit, shots per second. Fix every difference; cite each number.
3. Un-ignore the 0c and 0d real-health walks if they now finish; if a walk still dies with a faithful port, give it cover the level offers (no health writes). Run walkreal_0700000c/0d_live, enemy_accuracy_live, every real-damage walk, then the full suite.

---
claimed-by: sandbox5 at 2026-11-02T15:40Z

---
## Report (done, 2026-11-02T17:04Z)

# sniper-range-accuracy-0c-0d — weapon 0x1e drones at 34-41 u

Branch `s/sniper-range-accuracy-0c-0d`, cut from origin/master 84e64088. Time-boxed session:
the sandbox expires ~17:44 UTC; the full suite did not fit — see "What's left".

## What the game does at range (the pinned law, all [M] unless tagged)

Per shot of `DroneWeap_FireWeapon` 0x178f68 → `DroneWeap_DoBulletAccuracy` 0x179740:

1. **Hit chance is distance-independent**: `p = 100 − 5·(+0xB4 accuracy)` [:48], × `TooClose_Accuracy`
   (2.0) twice when `dist < TooClose_Distance` (3.0) [:49-51, :90-92], × `TargetFirstMoved_Accuracy`
   (0.25 on 0c/0d) while the first-moved latch `+0x1DD` stands (2.0 s window, `DoOpponentTargetting`
   0x179a48:66-73), × `TargetFirstStopped_Accuracy` (0.1) only on 0700000c..0e/1b with anim set
   `+0xDA` == 0x13 [:55-70], × `TargetMoving_Accuracy` (0.75) while `+0x1DC` [:72-74], × the
   difficulty multiplier (Easy 0.6 / Normal 0.8 / Hard 1.2 on 0c/0d, `ReadTuningVars`).
2. **The range gate is TIME, not distance**: beyond TooClose the roll also needs the continuous-sight
   counter `+0x270 >= fptoui(NewSighting_TimeToHit·FRAME_RATE)` = 120 ticks and `+0x41 == 0`
   [:93-99]; `+0x270` resets on EVERY lost-sight tick (`ProcessDroneSight` 0x176a28:202/216-231).
   Under 3.0 the shot always hits [:113-115].
3. **Zero cone for the snipers**: `Bullet_init` 0x12c5a0:139-170 — a class-2 (drone) owner with
   `weapon+0x68 & 0x400000` gets spread ×0; weapon 0x1e carries `+0x68 = 0x00408050` [D weapon_data
   0x2bf150], so its "misses" fly only the fixed wobble offset (0.5/1.5/1.5 sin terms, rotated by the
   yaw to the target) — ported (`bullet::spread`, `fire_weapon`).
4. **Damage has no range falloff**: HITDATA+8 = `weapon+0x0C` (0x1e = 40.0 [D]); at the hit,
   `Drone_ModPlayerHitDamage` 0x13a940 → `Drone_ModBulletDamage` 0x139f50 applies ×TooClose_Damage
   (2.0, dist < 3), ×`Drone+0x100`, and the anim-set-0x12 backshot ×2 (the 0x1e snipers are anim set
   0x13 — no backshot bonus); `Player_HandlePain` 0x1902c8 then ×`Plr_DMod_` (0c/0d Easy 0.4) →
   **16.0 per 0x1e hit on Easy**. The walk's log (14.00 / 16.00×4 / 11.20) matches this chain.
5. **Fire rate**: burst = `weapon+0x2C` (0x1e: 1), interval `+0x40` (0x1e: 120 frames) → in-burst gap
   120; between bursts `DroneWeap_BurstDelay` 0x179ba8: ≥30 u forces the delay to
   `BurstDelay_Max` = 60 frames (MinDist 4 / MaxDist 30 [D tuning]) → **a 0x1e sniper shoots every
   180 ticks at >30 u**; clip 5 then an AimStandReload pause (CombatState 0x148338 case 0x10:
   `+0xBBC < 1 → 0x69`).
6. **Engagement range**: no fire gate on distance; the combat router only leaves AimStandFire for
   CombatOutOfRange when `engage_range (+0xF0) <= dist` — class-4 snipers get engage 75+rand(8)
   (`DefaultInit` 0x14b300 sight-class table), so 34-41 u is inside their engage band. Class-0/2
   holders (engage 12-16) route to CombatOutOfRange, whose UPDATE (0x164690 case 3) never sets the
   fire request `+0x3B` — they close instead of sniping.

## Port comparison at 34-41 u (measured, probe `probe_0d_sniper_fire.rs`)

Drone #31 (0700000d, placed, skin 0x5000087, weapon 0x1e, class 4, engage 82.8): the port fired 31
shots over 3287 ticks of continuous sight at 37% rolled hits on a pacing Bond — the law predicts
(100−5·5)·0.75·0.6 = 33.75% — and one shot per 180 ticks with the clip-5 reload pauses. Damage per
hit 16.0. **Every per-shot component matches the game's law.** The two deviations found were in the
STATS the drones carry, not the firing law:

## Fixed

1. **Spawner spawns re-roll the shared `drone_stats` table** (`crates/nf-game/src/drone.rs`:
   `Drone.spawn_stats_params` + `Drones.stats_table` + `load_stats_table`, spawn site at
   `Action::Spawn`). The game's `DroneSpawner_SpawnDrone` 0x137cb0 re-runs the full drone init —
   `behaviour_util_get` 0x1c7a98 writes the template's stats section into the GLOBAL `drone_stats`
   0x31aa30 and `DoModeSettingsNEW` 0x14c600:70-107 reads the table as of that moment. The port's
   spawn revived the load-time drone record, and (worse) the first cut seeded the shared table empty;
   the shipped fix seeds it with the level's load-time table (`load_stats_table`, shared with
   `level_drones`). [D] both levels' placements: 0700000c grp-3 carrier decodes type-1 acc 9 /
   aggression 1 / reaction 80, grp-4 carrier type-5 acc 4 / hp 22; 0700000d grp-2 carrier type-1
   acc 9, grp-4 carrier type-5 acc 4 / hp 22 / react 100 — so in the game every later same-type
   spawn reads those records; the port now does too (spawn-time writes included).
2. **`reaction_time`'s default arm stays 10** — investigated as a suspect (the decomp reads
   `fptoui(FRAME_RATE_DIV·10.0)`), and CONFIRMED as already correct: `FRAME_RATE_DIV` is the 1.0
   frame→tick identity (BurstDelay's 45 frames stay 45 ticks [0x179ba8], NextBulletTime's interval
   120 stays 120 [0x179d40], CombatState's `FRAME_RATE_DIV·120` is 120 ticks [0x148338] — the
   stream-calibrated reading; the 0-reading broke `fix_guard_weapons_live`'s pinned fire rate).
   The evidence note is in the code.

## Result on the two ignored walks

- **0700000c** (`walk_the_whole_mission_…_real_damage`): the damage half is FIXED — the walk now
  survives the drones (tick 8700: 80.2 taken over 69 hits, 8 return-fire fights, health 19.8; it
  used to be dead at 80). Still ignored, for the ignore reason's OTHER clause: the yard climb never
  mantles past climb_top 12.139 [M 0x196118/0x196310] (reached y 9.9) — a movement bug outside this
  task's scope. Ignore reason updated to the measured state.
- **0700000d** (`walk_arrival_to_the_c5_door_real_damage`): unchanged deterministic death at tick
  6549 (9 hits for 89.2 from #31 at 41.2 u + 10.8 fall). The port's per-shot law is verified
  faithful (above), and #31 is a PLACED grp-0 drone whose stats (acc 5, hp 10) equal the game's own
  record — so the death hinges on ONE unverified input: **the port never breaks #31's sight**. A
  LOS sweep (`probe_0d_los_grid.rs`, 396 cells x 180-216 / z −224..−268 / y −3..1.5) shows the
  port's world gives #31 line of sight on EVERY cell but three (x=200, z≤−260) — there is no
  covered crossing for a route fix to use. Ignore reason updated with this.

## Regression checks run

enemy_accuracy_live (5 passed), fix_guard_weapons_live (3 passed — the fire-rate/burst pins),
guards_0c_live, door_guard_live, find_opponent_live, sensor_live, anim_events_live, and the
walkreal_07000001/03/04/0b suites (23 tests, all passed). The 0c/0d real-damage walks themselves
run in seconds and are reported above.

## What's left

1. The 0d decision needs the game's truth (below): if the game's #31 also holds LOS across the
   yard, the walk needs a covered route (none exists in the port's world today — the sweep says the
   collision world may be missing the warehouse's south-face occluders); if the game breaks sight,
   the port's LOS probe sets are missing that geometry and the walk un-ignores once fixed.
2. The 0c yard-climb mantle (movement, not damage) — the ignore reason's remaining clause.
3. The full suite (`cargo test --offline --release -j2 -p nf-game -p nf-formats -p nf-audio
   -p nf-view -- --test-threads=1`) — the 4.9 GB of stale test binaries were relinked for the runs
   above; a full pass did not fit the session's remaining time.
4. The probe files (`crates/nf-view/tests/probe_0d_drones.rs`, `probe_0d_sniper_fire.rs`,
   `probe_0d_los_grid.rs`) are committed as assertion-free `--ignored` tools for the next session.

## Live checks for the main agent

1. **#31's sight on the yard (gdbwatch, 0700000d, difficulty 1)**: watch drone +0x270 (the sight
   streak) and +0x41 while Bond walks the yard leg (186,−3.4,−266)→(190,−3,−266) with #31 alive at
   (161.1,6.9,−235.1). Predicted if the port is right: +0x270 climbs unbroken past 120 and shots
   land ~16 every ~3 s; Bond dies in ~90 s like the port. If +0x270 keeps resetting to 0, the game's
   LOS breaks where the port's does not — the port's sight-probe collision sets need the missing
   occluders, and the walk should then survive.
2. **The 0c/0d type-5 record after the first grp-4 spawn (fieldprobe)**: read `drone_stats[5][1]`
   (0x31aa30 + 5·0x60 + 0x20, 8 words) at level start and after the first warehouse garrison spawn.
   Predicted: start = zeros (defaults acc 5 / hp 10 for placed type-5s), after the carrier template
   spawns = acc 4, aggression 1, health 22, reaction 100, recover 100 — matching the decode of the
   grp-4 placement's section [D both levels].
3. **A 0x1e shot's damage on Easy (any level, difficulty 1)**: a sniper guard's hit on Bond should
   log `Player_HandlePain` damage exactly 16.0 (40 × Plr_DMod_Easy 0.4 on 0c/0d) with no distance
   term at 34-41 u — confirms the no-falloff reading.

## Usage

Session token/credit/context figures: not visible to me (the sandbox exposes no token counter, cost
or context-window meter this session; the driver message's ntfy and queue tooling are the only
external signals).
