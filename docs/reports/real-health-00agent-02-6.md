# real-health-00agent-02-6 — report

Branch `s/real-health-00agent-02-6` (cut from `origin/s/real-health-00agent-02-5` fa1a89882,
merged with `origin/master` 22d5836b7). The tree touches ONLY
`crates/nf-view/tests/walkreal_07000002_live.rs` (+ the WORKLOG entry) — **no `crates/*/src/`
change, no shared-helper change** (`walker.rs`/`damage_log.rs`/`walker_b2.rs` untouched), so
every other test binary is master's by construction.

## Status: DONE — `walk_the_castle_all_objectives_real_health_00agent` is GREEN and its
## `#[ignore]` is REMOVED. The whole `walkreal_07000002_live` binary: 4 passed / 0 failed /
## 0 ignored.

The walk completed the full objective route in **22463 ticks with 78.0 health at the bunker
door, no falls** — the first time this walk has survived the courtyard sniper (the 02-5 session
died at the lip at t14641-14820; the 02-03-04 session died in the geisha-1 stand-off at t6833).

## The sniper fix (lever 1), and what the runs measured

1. **Run 1 (repro on the merged tree)**: the 02-5 restructure's lottery stand ran at the lip
   (21 u) with `aim Some` from t0 and err converged to 0.000 by t60 — and **25 rifle rounds hit
   NOTHING** (his hp pinned 12.0 through t180). The mechanism is the spread, not the exposure:
   the 0x12's tap spread is 21 [M Bullet_init 0x12c5a0:139-170 — the port's `player_spread`:
   base 16 + the 5-per-pull bloom; no 0x400000 record ⇒ mul = 1] = ±0.62 u at 21 u, and the
   wall eats every round off the slit's sliver (the same eye-ray-clears/rounds-die-on-the-jamb
   shape the 13 security-room stand measured [L that ledger]). Shot 1 landed at +335 and shot 2
   at +501 from the bottom arrival, exactly the 02-5 cadence — dead at the lip.
2. **Run 2 (the suppression + the shells discovery)**: pushing a `dmg.cooldown` entry on the
   0x1e drone before the crossing suppressed the lip fight — the crossing walked CLEAN. The
   door leg's own `engage_in_view` (the file's 14-u node-end sweep) then met the sniper
   **EXPOSED at 13.2 u** and fired the in-hand shotgun: **one 0x1c shell took him 12.0 → 2.0**
   [L the run-2 trace]. Two shells kill him in the band. Also measured: the 8.0 stand-off ring
   experiment REGRESSED the geisha-1 pool (55.2 vs 78.0 at the stair top — the wider retreat
   walked Bond out of the corridor's line and the cascaded draws cost a full extra volley), so
   the ring stays at 6.0, and the stair-top shotgun arm lost 2 of the 4 shells to the descent's
   captain fight (the descent fights must burn the 0x12, not the shells).
3. **Run 3 (the winning shape)**: the ring back at 6.0; the shotgun armed AFTER the stair-bottom
   scoop (the lip-drop take lands at d 0.5 — lever 3's take verified again, +21 spare); the
   crossing clean (48.0 → 78.0 pool with the ring reverted); the door leg's first in-view shell
   at 13.3 u **KILLED the sniper outright** — "the band stand at the door: the sniper is already
   down" at t15890 with 78.0 held. The walk then died deeper, in the bunker corridor ambush,
   dry (the last shells burned on the corridor's in-view fights).
4. **Run 4 (final tree)**: + the corridor rearm (`arm_fullest` after the band stand — the 0x12
   armed with 47 rounds for the ch-0x81 ambush): **the walk PASSED**. The corridor fights cost
   zero (#14 killed 78.0 → 78.0).

The levers in the task's order, closed: (1) the sniper duel — WON, but not by a mouth duel: the
measured answer is the ≤12-u band on the door leg with the 8-pellet shells, the lip/mouth stands
are measured dead ends (the probe question resolved by the runs' own rf-trace exposure/aim
sampling instead of a separate per-u pass — `exposed_aim(9)` was Some from the lip onward, so a
per-u map had nothing left to distinguish); (2) the health headroom — the k=0/k=1 gap was
attacked with the 8.0 ring, MEASURED WORSE, reverted: the 6.0 ring's 78.0 pool is what survives
48+30 with the band kill; (3) the crossing ammo — the lip-drop take lands (d 0.5, +21 rounds)
and the fights run on 28 rifle rounds + 4 shells as the 02-5 ledger hoped.

## What was verified and how

- `walkreal_07000002_live` WHOLE (final tree): **4 passed / 0 failed / 0 ignored / 139 s** —
  the d1 `real_health`, d2 `real_health_agent`, `to_the_exit` and the d3 00 Agent walks all
  green; the d3 gating keeps the d1/d2 walks byte-identical (their code paths behind
  `difficulty >= 3` never run).
- `walkreal_07000003_live` WHOLE (final tree): **6 passed / 0 failed / 1 ignored** (its own
  00 Agent ledger stays `#[ignore]`d — that is the 02-03-3/4 chain's blocker, not this task's).
- The four deterministic run iterations (20-100 s each) with `NF_RF_TRACE` traces, quoted above.
- The full suite (`tools/test/suite.py`, background from 05:44 UTC on the near-final tree — the
  only later diff is this one test file, which cannot affect another binary): **8 problem lines,
  all environmental**: `missing_scenery_live` ×2 and `probe_pylon_render::probe_render_pylon_only`
  fail on `adapter: NotFound` (the sandbox has no wgpu backend — no GPU), and 5 `CRASH (exit -9)`
  are the 600-s per-binary kills of the multiplayer/network binaries waiting for peers
  (`net_desync_report_live`, `mp_stress_explosives_live`, `mp_online_failures`, `net_join_live`,
  `mp_shot_effects_live`). None is on this task's path; the slim-ISO `disc_golden` allowance is
  untouched. The final-tree suite re-run's total line is quoted in the WORKLOG when it lands
  (the sandbox reaps background jobs between messages, so the re-run may not finish in-window).

## What's left

Nothing on this task. The next 00 Agent blocker in the family is 07000003's own walk (its
`#[ignore]` ledger: the door-8 cluster approach and the exit lawn) — real-health-00agent-02-03-3.

## Live checks for the main agent

1. The sniper's band exposure, game-side: a fieldprobe/gdbwatch on drone 9 (skin 0x5000036,
   weapon 0x1e, post (−52.05, 6.34, −14.43) [D]) while Bond walks the lane → bunker-door line at
   00 Agent — from WHERE does the game expose him? The port's walker says `exposed=true` at
   13.2-13.3 u from the door-leg nav node (−50.8, −1.6) and `aim Some` already from the lip
   (21 u) — but the lip rounds die on the slit's jamb. Pinning the game's own exposure window
   would confirm the band-kill geometry the walk now relies on.
2. The 0x1c's one-shell damage at 13 u: the port measured 10.0 (run 2) and ≥12.0 (run 3, the
   kill) from the same spot — the pellet draw variance. A live hit-count on the sniper at 13 u
   would pin the game's own pellets-per-shell-on-box at that range.

## Usage

not visible to me — this environment shows no token, credit, or context-window figures for the
session (nothing to copy).
