# gas-guard-repin — report (2026-11-06)

Branch `s/gas-guard-repin` = `origin/s/gas-draws-8` (7e8051e18) + current `origin/master`
(d6245281a) merged; every gas draw kept. Pushed through ffcb70a5d.

## 1. The merge (three conflicts, by hand)

- `crates/nf-view/src/scene.rs`: master's wading-step comment is the evidence-backed one
  ([L PCSX2 07000002 (-79, 28): byte 0xE, step 0x28] — the probe keeps the water planes);
  the call keeps THIS branch's explicit `rng` arg (the gas-draws re-pin; the merged env.rs
  kept `env_water_ripple(gas, rng, material, hit_point)`).
- `crates/nf-view/tests/common/damage_log.rs`: union — master's `eye_ray_stands` opt-in in
  the aim pick + this branch's `NF_RF_GEOM` first-blind-tick diagnostic, both kept.
- `crates/nf-view/tests/mp_perf_16_live.rs`: kept the gas-draws digest
  (0x986f_90e8_e5d6_b12f — measured WITH the draws master reverted); **not re-measured this
  window** — the 3600-tick run did not fit. The pin note in the file says the re-pin is
  pending; next session's first run.

## 2. grenade_live's 0x68 flee — diagnosed and the gate fixed [M-cited]

The red is `a_guard_in_aim_stand_fire_0x68_flees_his_own_landed_frag` (3/1, reproduced).

**Root cause (decompile-read, not memory):** the port triggered the explosive scan ONCE at
the grenade's bounce with a snapshot position (`drone.rs`'s `explosive_queue`, whose own
comment called the per-frame rescan "a documented deviation"). The game is different:

- `Drone_BuildDynamicAwarePoints` 0x138b10:18-19 CLEARS the 64×0x30 record table at
  DAT_00299e20 and re-walks the live object list EVERY frame; each class-5 weapon record
  (0x34 frag among them) pushes a record holding the object pointer and the position COPY
  at obj+0x30 (no flight-state check — the only state gate is the stuck mine's
  `obj+0xF4 == 3` [M :164]).
- `DroneFunc_HandleExplosives` 0x1435f0:19-52 runs per drone per frame: not scared (+0x2b),
  type-2 record with fear prop 0x3b/0x3a, `Vec_SqDist3D(drone+0x30, rec+0x10) < r²`, then
  `DroneVision_ObjectVisibility ≥ 0.7` + `NDrone2_CanSeeObject(5,0)` → `SendMsgSelf(0x21)`.

**The fix (drone.rs):** the broadcast now scans the LIVE grenade list every frame
(`state < 2`, classification unchanged: 0x34 → r 12/run 4/2 s/ret 0x87, 0x3b-0x40 →
r 12/run 14/10 s); the `explosive_queue` field is gone. Per-frame conditions per guard are
the game's own [M 0x1435f0:30-52]. The in-flight record is not a hazard to the thrower: the
game's thrower is in GRENADE_THROW 0x2d then, which `ConsiderExplosive`'s state gate
REFUSES [M 0x1431c0:40-67].

**The test's premise re-pinned to the game's own sequence** (the old pin held 0x68 THROUGH
the flight, so the faithful scan scheduled the flee mid-flight and the pin's own
`next_state` wipe + the +0x2B latch made the premise self-defeating — probe-verified):
0x2d during the flight (refused), 0x68 from the bounce on, the pin stops when the delivery
latches `scared`; the +0x330 check compares against the position at consider_explosive's
run [M 0x1431c0:172-175].

**Still red, and why (probe data):** on the gas stream the frag bounces at 11.52 u and
rolls AWAY from the thrower (d² 160 → 177 across the window, r² 144; LOS false out there),
so no scan passes while the thrower is flee-eligible — the delivery step times out. The
pre-gas stream's "rolls back to ~5 u" came from the corridor rebound, not the throw shape:
the guard's lob lands at the ~11.5 u max-range spot for ANY target distance (Bond at 6 u
and off-axis tried — same 11.5 u landing). **The remaining lever is the scenario's corridor
geometry** (a guard spot whose rebound rolls the frag back inside 12 u); the gate itself is
now the game's.

## 3. The re-measured red list (§5 re-run whole on the merged tree)

Master moved again since gas-draws-8 (jump landings force two steps, landing/step probes
read water, the sighted guard ignores sound) — the list differs from §5 exactly as the
task anticipated. All 26 walkreal binaries ran WHOLE (`--test-threads=1`, NF_ISO, slim
ISO), plus nf-game's ai_live/grenade_live and nf-view's npc_doors/fix_guard_misc (drone.rs
was touched):

| binary | gas-8's §5 | this tree |
|---|---|---|
| walkreal_07000001_live | 7/1 | 7/1 (the d3 walk, §4's ledger stands) |
| walkreal_07000002_live | 3/0 + 1i | 3/0 + 1i (the corridor lob holds green with the per-frame scan) |
| walkreal_07000003_live | 4/2 + 2i | 4/2 + 2i (same two reds) |
| walkreal_07000004_live | 10/0 | **7/3 — NEW: the stats trio** (`every_stats_row_matches_the_game_formulas_from_the_real_finish`, `the_platinum_finish_shows_platinum_on_records`, `the_results_page_opens_stats_and_dossier_and_scores_the_real_tallies`) |
| walkreal_07000005_live | 13/1 | 13/1 (same red) |
| walkreal_07000006_live | 4/2 + 1i | 4/2 + 1i (same two) |
| walkreal_07000007_live | 6/0 | 6/0 |
| walkreal_07000008_live | 5/1 | 5/1 (same red) |
| walkreal_07000009_live | 8/0 | 8/0 |
| walkreal_0700000a_live | 13/0 + 2i | 13/0 + 2i |
| walkreal_0700000b_live | 7/0 | 7/0 |
| walkreal_0700000c_live | 2/1 + 1i | 2/1 + 1i (same red) |
| walkreal_0700000c_barrelrow_live | 1/0 | 1/0 |
| walkreal_0700000c_eastroute_live | 1/0 | 1/0 |
| walkreal_0700000d_live | 1/1 + 1i | 1/1 + 1i (same red) |
| walkreal_07000011_live | 2/1 + 1i | **2/3 — worse: 3 red, 0 ignored** (the merge lifts master's un-ignore onto this stream) |
| walkreal_07000011_route_live | 1/0 | 1/0 |
| walkreal_07000012_live | 2/1 | 2/1 (same red) |
| walkreal_07000013_live | 1/2 | 1/2 (same two) |
| walkreal_07000014_live | 8/1 + 1i | 8/1 + 1i (same red) |
| walkreal_07000015_live | 5/1 + 1i | 5/1 + 1i (same red) |
| walkreal_07000016_live | 14/0 + 2i | **11/3 + 2i — the chasers pair regressed**: `walk_the_chasers_defeat_completes_the_mission`, `bond_is_armed_at_the_chaser_fight_on_a_direct_load`, `bond_is_armed_at_the_chaser_fight_after_the_15_handoff` |
| walkreal_0700001b_live | 17/0 + 2i | **17/0 + 2i — the Drake pair holds** |
| walkreal_07000041_live | 15/0 | 15/0 |
| walkreal_0700004a_live | 4/0 | 4/0 |
| walkreal_0700004a_satchel_live | 2/0 | 2/0 (the per-frame satchel scan breaks nothing) |
| nf-game ai_live | — | 37/0 |
| nf-game grenade_live | 3/1 | 3/1 (§2: the gate is fixed; the delivery premise needs the corridor-rebound lever) |
| nf-view npc_doors_live | — | 6/0 |
| nf-view fix_guard_misc_live | — | 6/0 + 1i |

**One total line for the run above (30 binaries, whole): 202 passed / 23 failed / 15
ignored.** The full four-crate suite (the standing ~3 h measurement) did not fit the
window; mp_perf_16_live (the digest re-pin) also did not fit and is named in §1.

## 4. Classification so far (the task's per-red question)

- **grenade_live 0x68**: a GUARD BEHAVIOUR deviation, fixed this window (the per-frame
  scan, §2); the leftover is the test scenario's stream-tuned geometry (the corridor
  rebound lever), not the gate.
- **16's chasers trio**: the defeat-walk economy was tuned on the pre-master-wave stream;
  master's jump-landing two-steps change the route timing into the window (the punch pin's
  armour-0 assert rides the same economy). Stream-tuned → route/timing levers; the 04-style
  stats pins are not involved.
- **04's stats trio**: the finish-screen tallies are pinned to the walk's run (kills,
  accuracy, time) — the master wave's landing/water changes move the tallies. Stream-tuned
  premise; needs the stat rows re-measured against the merged run before any pin moves.
- **11's third red**: master's un-ignored test rides the gas stream red — inherited
  interaction, needs the same per-test read as §5's originals.
- The §5 inherited singles (01 d3, 03 ×2, 05, 06 ×2, 08, 0c, 0d, 12, 13 ×2, 14, 15):
  re-measured unchanged this window, not individually re-diagnosed — the §5 ledger stands
  as the starting point.

## What's left

1. mp_perf_16_live's digest re-pin on the merged tree (§1 — the note in the file says
   pending).
2. The 16 chasers route levers against the master-wave landing timing; the 04 stats rows
   re-measured then re-pinned with the merged-run numbers; 11's third red read.
3. grenade_live's corridor-rebound lever (a guard spot whose frag rolls back inside 12 u).
4. The §5 inherited singles' per-red classification (the ledger above).
5. The full four-crate suite from one run.

## Live checks for the main agent

1. Level 02 corridor 11: does the real game's frag (guard lob at a 10 u Bond) bounce at
   ~11.5 u and roll BACK to the guard, or roll away? The port's gas-stream roll-away is
   what times out the flee premise; a real-game roll-back would point at the port's bounce
   physics, not the test's spot.

## Verification notes

- The drone.rs change is game-code FAITHFULNESS (the documented deviation removed), not a
  walk accommodation; every test that uses the touched code ran: ai_live, grenade_live,
  npc_doors_live, fix_guard_misc_live + all 26 walkreal binaries (§3).
- No pin tagged [L] was loosened; no ignore was added; the 16/1b d2/d3 sisters keep their
  measured ignores; the 04 trio's pins are untouched (they fail on the merged run's
  tallies).
- `git status --short` checked before every commit; only the four task files
  (drone.rs, grenade_live.rs, WORKLOG.md, this report) were ever staged.

## Usage

not visible to me (no token, credit or context-window figures are exposed to this
session; nothing in the sandbox reports them).

## ntfy failed

All three fallbacks failed in order (2026-11-06 ~11:11-11:14 UTC):
1. `curl -sS -m 20 -H "Title: NightfirePort" -d "..." https://ntfy.sh/nightfire-yoda-8k3x`
   → `curl: (28) Connection timed out after 20002 milliseconds` (rc 28)
2. `python3 -c "... urllib.request ..."`
   → `urllib.error.URLError: <urlopen error [Errno 101] Network is unreachable>`
3. `wget -q -O- --post-data "gas-guard-repin: done" https://ntfy.sh/nightfire-yoda-8k3x`
   → rc 124 (killed at the timeout), no output.

The finish step itself succeeded (`bash ../nf-queue/finish.sh 10-gas-guard-repin done
docs/reports/gas-guard-repin.md` → `OK done 10-gas-guard-repin`; the script's stderr
noted its own `cat docs/reports/...` ran from the queue worktree's cwd — the report file
is in the repo, pushed at ee6f0761d). ntfy.sh was equally unreachable in the gas-draws-8,
gas-rng-draws-7 and guard-radii-finish sessions.
