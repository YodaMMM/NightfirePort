# real-health-agent-0c-0d — Agent real-health walks for 0c and 0d

Follow the Agent sisters on master (the load via `load_with_carry_difficulty(..., 2)`, `Drones::difficulty == 2` asserted). Add the Agent sister of 0700000c's whole-mission real-damage walk and of 0700000d's real-damage walk, same route and end. 0d's own Operative real-health route is low-priority work elsewhere (50-level-0d-real-health-crossfire-5): build on what is green on master. Real input only; no top-ups; helpers opt-in. Done = each sister green, or ignored with a measured reason and the next lever; every walkreal_*_live; the full suite total line.

---
claimed-by: sandbox5 at 2026-11-04T12:20Z

---
## Report (done, 2026-11-04T13:13Z)

# real-health-agent-0c-0d — the Agent sisters of the 0c/0d real-damage walks

## What's done

Branch `s/real-health-agent-0c-0d` cut from origin/master (8deea246).

**The wiring (both sisters exist, the d1 replays byte-identical).**

- `walkreal_0700000c_live.rs`: the whole-mission walk is parameterized —
  `walk_the_whole_mission_from_the_start_to_the_0d_handoff_at(difficulty)`. Difficulty 1 keeps
  master's exact expression (`walk_spawn_to_the_band` → `load_with_carry`); 2+ loads the menu way
  [M C_SBNFDFCTY 0x216ca8:15 → `load_with_carry_difficulty`; GameState+0x28 0x2a3790 →
  `Drones::difficulty`, asserted after every load — the 0c cold load AND the 0d warm hand-off
  reload]. New test `..._real_damage_agent` = `with_yard_aimed_stands` + `with_real_damage` +
  `_at(2)`, the same route and the same end as the Operative real-damage walk.
- `walkreal_0700000d_live.rs`: the same shape — `walk_arrival_to_the_c5_door_at(difficulty)` (the
  cold menu pick and the warm level change both run the picked difficulty), new test
  `walk_arrival_to_the_c5_door_real_damage_agent` = `with_real_damage` + `_at(2)`.
- `common/walkreal0c.rs`: `walk_spawn_to_the_yard_ladder_base_at(iso, difficulty)` and
  `walk_spawn_to_the_band_at(iso, difficulty)` — the d1 delegates are byte-identical, the three
  existing callers (the 0c walk, the eastroute replay, the upward-shot test) untouched. Behind the
  `difficulty >= 2` gate (opt-in by construction — no existing caller loads 2+): `dmg.lane_fights`
  and the new `dmg.escalating_cooldowns`.
- `common/damage_log.rs`: the opt-in `escalating_cooldowns` flag (+ the `fight_fails` history) —
  a failed return-fire fight doubles its re-fight wait per further failure on the same drone
  (240 → 480 → 960 → 1920, capped). Off (= master's flat 240) everywhere the flag is not set;
  the only setter is the 0c d2 gate.

**Verification of the d1 base (the refactor must not move master's walks):**
- `walkreal_0700000c_live` whole: 3 passed / 0 failed / 1 ignored (the base top-up walk, the
  Operative real-damage walk — 26.2 hp finish as master pinned — and ladder495; the Agent sister
  is the ignore).
- `walkreal_0700000d_live` whole: 1 passed / 0 failed / 2 ignored (the base walk green; the
  Operative real-damage ignore is master's own, untouched; the Agent sister is the second).
- `walkreal_0700000c_eastroute_live`: 1/0. `upward_shot_0700000c_live`: 2/0.

## The sisters' results — both IGNORED with the measured runs and the next levers

**0c Agent (`..._real_damage_agent`, ignored):** the Operative walk is byte-identical through
tick 4054 (the same fights, the same landed hits — the shared RNG draws pass both accuracy rows
[M DroneWeap_DoBulletAccuracy 0x179740:74-88]) and finishes at 26.2 hp; the Agent rows land the
draws d1 dodges. Four deterministic lever rounds:
1. master fights — DIED t4054 (12.96,12.35,−124.38), 65 hits for 100.0: the plain nearest-visible
   pick stood 900-tick fights on the display-ground guards whose own boxes the hall slab eats
   (#27 twice, t2707/t3637), while the d 19.8 machine pistol #30 chewed 26.3 in 25 hits.
2. + lane fights (the 07000013 law) — the fights WIN (#33/#27/#24/#30 killed) — DIED t6716 at the
   display ground: #31 (y 3.2, below the floor edge) is unkillable from every stand the route
   reaches and chewed the last 12.5 hp.
3. + escalating cooldowns — DIED t8579 at the warehouse plates (3.0 hp left): the fights kept
   winning (#37 killed) but the t5903 fight on #30 paid 19.0 to the machine pistol's cadence.
4. + dry_taps 3 (faster abandonment) — RESHUFFLED WORSE, DIED t4234 — reverted.
Next levers (in the ignore message): a d2-gated staircase-head down-shot stand on the
display-ground pair (the game's own hall-floor opening x 9..12.4 gives the line — the 0c
yard-stand law), an aimed stand on #31 from the display pocket, the NF_RF_TRACE/NF_RF_GEOM read
of the blocked fights.

**0d Agent (`..._real_damage_agent`, ignored):** DIED t4850 (202.25,16.14,−255.06) — the deck
arrival, the same crossfire the Operative ignore audits — with the Agent deltas measured: 0
return-fire fights, 6 hits for 100.0, of which #21 (skin 0x5000087, sniper 0x1e,
(162.8,1.0,−215.0)) landed 2 × 28.0 from d 58.3 and #36 (0x1e, (157.7,1.0,−228.1)) 1 × 10.0 from
d 54.2 — both OUTSIDE the fights' 48 u lane window and outside every pinned kill box, so no fight
ever engaged them; #20's one 28.0 at d 7.1 finished it. Next levers: the
50-level-0d-real-health-crossfire-5 lever set first (this sister must build on a green Operative
base), then a d2-gated aimed stand on the two d2 snipers from the strip (the zero-spread aimed act
[M Bullet_init 0x12c5a0:139-170] reaches d 55+ with the still-gate tap law).

## The walkreal sweep (rule 7 — common helpers touched)

The `walkreal_*_live` binaries run whole, sequentially (`--test-threads=1`), this session:

| binary | result |
|---|---|
| walkreal_07000001_live | 8 passed / 0 failed |
| walkreal_07000002_live | 3 passed / 1 ignored (master's own) |
| walkreal_07000003_live | 6 passed / 1 ignored (master's own) |
| walkreal_07000004_live | 10 passed / 0 failed |
| walkreal_07000005_live | 14 passed / 0 failed |
| walkreal_07000006_live | 6 passed / 1 ignored (master's own) |
| walkreal_07000007_live | 5 passed / 0 failed |
| walkreal_07000008_live | 6 passed / 0 failed |
| walkreal_07000009_live | 8 passed / 0 failed |
| walkreal_0700000c_live | 3 passed / 1 ignored (this task's Agent sister) |
| walkreal_0700000c_eastroute_live | 1 passed / 0 failed |
| walkreal_0700000d_live | 1 passed / 2 ignored (master's Operative ignore + this task's Agent sister) |
| upward_shot_0700000c_live | 2 passed / 0 failed |

(Plus walkreal_0700000a_live was mid-run when the window closed — not counted.)

**Not reached this session** (the sandbox window closed): walkreal_0700000a_live (rerun),
0700000c_barrelrow, 07000011(+route), 07000012–16, 1b, 41, 4a(+satchel), celprobe, dump,
probe014_param26 and the walkreal_probe0c_* probe set. Risk argument for the merge owner: every
helper change is inert when its flag is off — `escalating_cooldowns` defaults false and only the
0c d2 gate sets it (no existing caller loads difficulty 2+ through walkreal0c.rs); the
`fight_fails` Vec is only read behind that flag; the walkreal0c.rs `_at` splits keep the d1
delegates byte-identical and the three existing callers green (run whole, above). The full
4-crate suite total line did not fit the window; the nf-view binaries listed above are this
task's own blast surface and they are green.

## What's left

1. The remaining walkreal_*_live binaries listed above (mechanical: the flags are off there by
   construction).
2. The full 4-crate suite total line.
3. If the sisters should go green later: the levers in the two ignore messages.

## Live checks for the main agent

- 0700000c, Agent difficulty: stand at the stair head (≈11.3, 11.3, −123) after the vent drop and
  confirm the display-ground guards below (#24/#27 at (8.1,7.3,−114.7)/(8.6,7.3,−114.4)) take
  damage from a DOWN shot through the hall floor opening — expected: the aimed act's rounds land
  through the opening the game's own ramp descends through.
- 0700000d, Agent difficulty: confirm the two d2 snipers (#21 (162.8,1.0,−215.0), #36
  (157.7,1.0,−228.1)) are killable from the strip-mouth stand the route already uses (the
  still-gate aimed tap law) — expected: a zero-spread aimed round reaches d ≈ 55.

## Usage

not visible to me (no token, cost or context-window figures are exposed in this environment).
