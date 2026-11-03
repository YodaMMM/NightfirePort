# real-health-00agent-05-07 — report (2026-11-03)

**DONE.** Both 00 Agent (difficulty 3) sisters of the 05/07 real-damage walks are **green on real
input** — same routes, same ends, `Drones::difficulty == 3` asserted at each load — and the Agent
(difficulty 2) sisters ride along as byte-exact restores of the difficulty-branch runs.

Branch `s/real-health-00agent-05-07`, cut from origin/master 01917076.
`origin/s/real-health-agent-difficulty-01-07` is already an ancestor of master (the merge is a
no-op) — but master's 2879088f "Revert the Agent-difficulty walks" removed its seven sisters, so
the 05/07 Agent sisters were restored from f1b37c21 (the two test files differ from that commit
only by the revert; the restored 05 Agent run reproduces the documented 57.6-health finish [L
docs/reports/real-health-agent-difficulty-01-07.md — bit-exact]).

## The difficulty-3 rows the game reads (checked, all already in the port)

- Bond's pain: 05-08 group Plr_DMod_Hard = 1.0 (Operative 0.6, Agent 0.7) [M Player_HandlePain
  0x1902c8; tuning.rs ← ReadTuningVars 0x1d3a80].
- The guards' hit chance: DroneFiring_Accuracy_Hard = 1.3 (Easy 0.8, Normal 1.1) [M
  DroneWeap_DoBulletAccuracy 0x179740:74-88].
- Bond's outgoing rounds also scale: the observed fight damage 10.0 → 8.0 a torso round
  (DroneDamage_Hard 0.8) [L the 05 fight traces — tgt hp 20 → 11.6 on one landed round].
- The min-difficulty placements: Drone_Create 0x136be0:49's gate spawns them at d2 AND d3 — the
  00 Agent runs meet the ambush set the Operative walks never see (the gallery riflemen, the
  walkway pair, the two 0x24 snipers) [M 0x136be0:49; L the ledgers].
- No 00-only gate found beyond these rows on either level's read set.

## What it took (the 05 00 Agent tactics — the damage story, all [L] the run ledgers)

The first d3 run died at tick 8534 on the gallery spine: 100.0 in 24 hits — the two 0x24 snipers
of the castle approach (the ditch placement 30.0 at tick 609, the wall placement 30.0 at 1381),
the lane rifleman's bursts (38.5), and the spine's two riflemen 16.6 u BELOW (48.0 — their rounds
reach Bond but the fights' pick never saw them). The shipped tactics, gated `difficulty >= 3`
inside the level's own test file (the Agent d2 sister keeps the exact restored RNG):

1. **The sniper stands at the spawn** (the walker's own `shoot`): the ditch placement at
   (99.7, 16.8, 101.6) and the wall placement at (62.1, 34.5, 104.2) are engaged before crossing
   their fields — the vis probe read the wall placement `exposed_aim SOME` from the spawn itself
   (43.1 u, dy +9.5), and the ditch one opens as the road descends past its ditch (shoot()'s
   forward push). Both die pre-fire ("killed true" at 1510/1782).
2. **The gallery-sentry stands from the corridor mouth** (`shoot_point` — zero movement): the
   two stationary 0x16 riflemen at (−46.9/−49.6, 22.0, −58.5/−60.7) are 16 u below the spine; the
   plain shooters window (20 u, dy 6.0) dropped them BEFORE the pick, so the fights never started
   and the spine cost 46.8-53.5. The down-ray clears from the corridor legs (`exposed_aim SOME`
   at dy −15.3 [L the NF_VIS_PROBE per-leg map]); the stands kill both (9356/9455) and the spine
   price collapses. The Operative master (d1) skips — same placements, same route, master
   behaviour.
3. **back_off 4.0** (the shared opt-in, the 03 report's tactic) and **the fights' head-hunt**
   (`with_real_damage_head_hunting`, the 07000012/15/16 walks' law): one head round drops a
   10-hp guard; the lane rifleman's fight went from 49.4 health of bleed to zero.
4. **`Walker::reload_full` after the stands** (additive API): the fights must not open on a
   drained clip (the a05g repro's fight opened at 0.9 u with clip 1 and reloaded mid-fight).

**07 needed no tactics** — the party route's own cover holds at d3; the sister passed as loaded.

## Shared helper changes (rule 7 compliant — opt-in, checked, swept)

- `DamageLog::shooters_window: Option<(f32, f32)>` (on its own line): the shooters list's
  (horizontal, vertical) reach; `None` keeps the master (20, 6) / lane (48, 7.5) windows; the 05
  d3 body sets (48, 18). Read in exactly one place (`shooters()`).
- `Walker::reload_full()` and `Walker::shoot_bounded(di, label, cap)` (ADDITIVE — `shoot`
  delegates at the old 10800): no existing caller changes behaviour.
- The `NF_VIS_PROBE` visibility probe (env-gated diagnostic prints, no behaviour) in the 05 file.

## What was run

- `walkreal_07000005_live`: **14 passed / 0 failed** (77 s — the masters, the real-damage walk,
  the Agent sister at 57.6, the 00 Agent sister at 56.0).
- `walkreal_07000007_live`: **5 passed / 0 failed** (17.9 s — incl. the Agent and 00 Agent
  sisters).
- nf-game + nf-formats + nf-audio (`--test-threads=1`): **136 passed, 3 failed** — exactly the 3
  allowed slim-ISO `disc_golden` (bigf header/tables/directory).
- **EVERY `walkreal_*_live` binary** (rule 7): the 19 per-level `walkreal_0700*_live` binaries
  plus barrelrow, eastroute, 11_route, 4a_satchel, celprobe, dump — **25 binaries, 0 failures**
  (01: 6, 02: 2, 03: 5, 04: 8, 06: 6, 07: 5, 08: 4, 09: 6, 0a: 10+3i, 0b: 5, 0c: 2+1i, 0d: 1+1i,
  11: 2, 12: 2, 13: 2, 14: 8, 15: 5, 16: 14, 1b: 17, 41: 11, 4a: 2).
- NOT run: the nf-view `fix_*`/`probe_*` test binaries and suite.py's all-at-once build — the
  sandbox disk (9.9 GB, ~5 GB free with the ISO + work/) cannot hold their ~12 GB of artifacts
  (three disk-OOMs measured across sessions); the chunks that fit all ran green above.

## What's left

Nothing for this task. The sisters pass, the masters stay green, the sweep is green.

## Live checks for the main agent

1. The 05 ditch sniper: fresh boot at 00 Agent, walk the spawn road — the 0x24 guard in the ditch
   at (99.7, 16.8, 101.6) should fire one rolled shot (~30.0 at d3) as the road drops past it
   [L the ledger tick 609] — gdbwatch Drone_ModBulletDamage's return would pin the d3 damage.
2. The gallery riflemen's line of fire into the corridor: their rounds reach z −38.95 INSIDE the
   roofed corridor [L the a05agent2 ledger] — a PCSX2 check that the game's own corridor has the
   same open sight-line (the port's cel-scoped flight may differ) would settle whether the
   corridor is real cover at d3 or only at lower angles.

## Usage

not visible to me (no token/credit/context figures are exposed to this session; nothing in the
sandbox reports them).
