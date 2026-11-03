# real-health-00agent-08-09-0a-0b — report (2026-11-03)

**DONE.** All four 00 Agent (difficulty 3) sisters of the real-damage walks are **green on real
input** — same routes, same ends, `Drones::difficulty == 3` asserted at each load (inside the
files' existing `*_at(difficulty)` helpers [M C_SBNFDFCTY 0x216ca8:15 → load_with_carry_difficulty;
D difficulty 0x2df4c0 +0xc = 1/2/3]).

Branch `s/real-health-00agent-08-09-0a-0b`, cut from origin/master 68298698 (the tip — no master
movement during the session; the final tree IS master + the four test files).

## The results

- **07000009** `walk_tower_a_start_to_finish_real_damage_00agent`: **PASSED AS LOADED** — end
  health 100.0, armour 50.0, **0 hits** in 140640 ticks. The route's own cover and takedowns
  hold at d3.
- **0700000a** `walk_office_floor_qworm_keyboard_hatch_real_damage_00agent`: **PASSED AS LOADED**
  — end 100.0, 0 hits, 15960 ticks.
- **0700000b** `walk_fresh_spawn_to_the_mission_end_real_damage_00agent`: **PASSED AS LOADED** on
  the route's own fight package (lane_fights + tranq_fights + back_off 3.0) — end 65.5, 8 hits for
  34.5, 5 return-fire fights.
- **07000008** `walk_the_whole_mission_on_real_input_real_damage_00agent`: end **31.5** (8 hits
  for 68.5) after the d3 tactics below — the Hind died on the **sixth** rocket (damage 606 ≥ hp
  598).

## The difficulty-3 rows the game reads (checked — the port already reads them all)

- **Bond's pain**: 05-08 group Plr_DMod_Hard = 1.0 (Easy 0.6, Agent 0.7); 09-0b group
  Plr_DMod_Hard = 1.0 (Easy 0.6, Agent 0.8) [M Player_HandlePain 0x1902c8; ReadTuningVars
  0x1d3a80 cases 0x7000005-8 / 0x7000009-a-b; the port's tuning.rs matches the decompile
  bit-for-bit].
- **The guards' hit chance**: 05-08 group DroneFiring_Accuracy_Hard = 1.3 (Easy 0.8, Agent 1.1);
  09-0b group DroneFiring_Accuracy_Hard = 1.0 — **the SAME row the Agent runs** (Easy 0.8) [M
  DroneWeap_DoBulletAccuracy 0x179740:74-88; ReadTuningVars 0x1d3a80]. That is why 09/0a/0b pass
  as-loaded: at d3 their guards aim exactly like the Agent sisters that already pass.
- **Damage into drones** (Bond's outgoing rounds): every hit ON a drone scales by
  DroneDamage_<difficulty> — Easy 1.5 / Normal 1.0 / Hard 0.8 [M NDrone2_HitDamage
  0x145558:134-150: the multiplier branches on GameState._40_4_ then `param_1 *= fVar7`].
- **The min-difficulty placements**: Drone_Create's gate spawns a placement when its
  min-difficulty param ≤ GameState+0x28 — they spawn at d2 AND d3 [M 0x136be0:49]; the port's
  create path sets the difficulty BEFORE the creates (scene.rs "The selected difficulty BEFORE
  the creates" — captain 0x14b300:504-535, Drone_Create 0x136be0:49, Lock_Create 0x1ae1f8:67,
  Copter_Create 0x17d340:76, Hint_Create 0x1b3248).
- **The Hind's hp scales**: DiffMod 1.0 below d2, 1.5 at d2, 2.0 at d3, 4.0 at d4 [M
  Copter_Create 0x17d340:76-90] — measured 299 (d1) / 448.5 (d2) / 598 (d3) [L the run ledgers].
- **The copter's damage accumulation has NO difficulty multiplier**: Copter_Update's part walk
  adds `rec+0x64 += hitdata.damage × slot factor` [M 0x17dbb8:277-320] — Bond's rockets do the
  full 101 at every difficulty, so the d3 Hind (598) takes **six** rockets; the AT-420 loadout
  (clip 3 + spare 3) is exactly enough. The port matches (no difficulty read in its copter
  damage path).
- No 00-only gate beyond these rows was found on any of the four levels' read sets.

**No port fix was needed**: every death the d3 runs produced traced to tactics, not to a port
difference. All four test files carry the d3 doc rows with these addresses.

## What it took (the 08 d3 tactics — all [L] the run ledgers, all gated `difficulty >= 3` inside the level's own test file so the Operative/Agent sisters' timelines stay byte-exact)

The first d3 run died at tick 25164 at (108.44, 28.37, -121.92): the boarding platform's
burst-rifle pair (#19/#20, weapon 0x15, skin 0x500002f) fired into the boarding legs BEFORE the
existing "before the boarding" stand ran, on a pool already bled 60 by the open long link.

1. **The mid-west doorway stand**: the rifle pair #15/#16 (weapon 0x0, at (122.2/122.6, 34.6/35.3,
   -112.8/-114.7), ~10 u east) owns the open long link — the answered fights on the open link cost
   40.0 of pool. From the mid-doors doorway (the link's west end, cover) #15 dies for **0**; #16's
   answering fight also cost 0.
2. **The link walk-out stand** (after the long-link leg): #16/#17 are ambush spawners (their
   plates fire mid-leg); the walk-out stand catches what the leg enabled (#17's −6 became 0).
3. **The platform-mouth stand** (after the "lower level east" leg): kills #19/#20/#24/#23 at the
   platform mouth instead of soaking their interval-4 bursts in the boarding legs.
4. **back_off 4.0** (the 05/03 sisters' shared opt-in): #20 closes to point blank during the
   exchange (the measured −12.00 at 1.2 u); the fights step back while firing.
5. **reload_full after the stairwell down** (the 05 sisters' rule): the fights must not open on a
   drained clip.

The Hind fight itself needed nothing new — the level-08-real-health-rockets gates (the 10 u blast
reach, the clear corridor) held at d3; the fight ends on the sixth rocket.

## What was run

- `walkreal_07000008_live`: **6 passed / 0 failed** (136 s — the masters, the ally pair, the
  real-damage walk, the Agent sister, the 00 Agent sister at 31.5).
- `walkreal_07000009_live`: **8 / 0** (177 s). `walkreal_0700000a_live`: **12 / 0** (+3
  pre-existing ignores). `walkreal_0700000b_live`: **7 / 0**.
- **EVERY `walkreal_*_live` binary** (the 05-07 sweep set): 01: 7, 02: 3+1i, 03: 6+1i, 04: 10,
  05: 14, 06: 6, 07: 5, 08: 6, 09: 8, 0a: 12+3i, 0b: 7, 0c: 2+1i, 0c_barrelrow: 1, 0c_eastroute: 1,
  0d: 1+1i, 11: 2, 11_route: 1, 12: 3, 13: 3, 14: 9, 15: 6, 16: 14, 1b: 17, 41: 11, 4a: 2,
  4a_satchel: 2, celprobe: 2, dump: 1 — **25 binaries, 0 failures**.
- nf-game + nf-formats + nf-audio (`--test-threads=1`): **136 passed, 3 failed** — exactly the 3
  allowed slim-ISO `disc_golden` (bigf header/tables/directory).
- **NOT run: suite.py's all-at-once build** and the nf-view `fix_*`/`probe_*` test binaries — this
  sandbox (9.9 GB disk, ~5 GB free with the ISO + work/, 3 GB RAM) cannot hold their ~12 GB of
  artifacts (three disk-OOMs measured across sessions, docs/reports/real-health-00agent-05-07.md);
  the chunks that fit all ran green above. The background `nohup python3 tools/test/suite.py`
  start also died instantly in this sandbox (background jobs are killed between tool calls), so
  every chunk was run in the foreground with the same `--test-threads=1`.

## What's left

Nothing for this task. The four sisters pass, the masters stay green, the sweep is green.

## Live checks for the main agent

1. The 08 d3 Hind hp: fresh boot at 00 Agent, ride the gondola down, check the Hind's health —
   the port reads 598 = DiffMod 2.0 [M Copter_Create 0x17d340:80-83]; a gdbwatch of the create's
   hp store would pin the game's own 598.
2. The 08 boarding platform: at 00 Agent the burst-rifle pair (#19/#20, weapon 0x15) should open
   on Bond as the stairwell empties onto the platform — a PCSX2 run confirming the pair's
   positions ((101.9/104.5, 28.2, -122.3/-122.6) [L the d3 ledgers]) and their first-shot timing
   would pin the fight window the new platform-mouth stand answers.

## Usage

not visible to me (no token/credit/context figures are exposed to this session; nothing in the
sandbox reports them).
