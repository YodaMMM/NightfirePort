# gas-rng-draws-7 — report (2026-11-06)

**PARTIAL** — master is merged (the voice draws landed through, every gas draw kept), the
full walkreal list is RUN and quoted per binary (the task's named gap from draws-6), and 1b's
Drake fight has a measured root cause with probe data but the fix did not land in the window.
Branch `s/gas-rng-draws-7` = `origin/s/gas-rng-draws-6` 6bd978d11 + `origin/master` merged at
402e4f57c.

## Done this session

- **The master merge, every draw kept** (402e4f57c). One conflicted file
  (`walkreal_07000001_live.rs`, 6 hunks): master's side was the older text of the same
  doorway blocks (dc4b7a16f's content, which the gas branch built on and generalized to every
  difficulty); the gas branch's side was taken. `far_field_sweep` (the gas branch's post-AND
  field clear) is defined and called; master's `doorway_step` variable is gone with its own
  block (the gas restructure replaced it). `2f99f073a`'s shield_bloom hunk merged clean.
  nf-game lib + nf-view + the 1b binary build clean on the merged tree.

- **The Drake fight root cause, measured (probes run on the merged tree, then reverted):**
  `fight_opts`'s fire gate is `ray_on && view_level && clip ≥ 12` with
  `view_level = |look[1]| < 0.01` — the eye fires only LEVEL, at cover height (the file's own
  convention; hide_and_snipe's doc says it outright). On the merged stream Drake rises from
  his spawn (−7.3, −1.4, 0.1) to a hover band **x 9-17.8, y 4.2-7.6, z −5.8..0.4** [L the
  probe: 30 samples every 120 ticks, `state 0xc0/0xc1` alternating] while Bond wedges at
  (12.7, −3.0, −1.6) under the deck — the slab cuts the ray (ray false, exposed_aim None from
  t120 on), Drake's hp stays exactly 60.0 through the 9000-tick window, and the clip never
  drops below 100 (zero shots land). A candidate-hold scan (t600 and t3000, x −14..32 ×
  y −6..12 × z −8..4, LOS = the game's own `segment_bullet` to Drake + reach = a clear
  segment from Bond's position): **no reachable candidate sits at Drake's hover height** —
  the y 3-12 rows are empty at t600 (the deck blocks the float path), and the level-look fire
  gate makes the reachable low candidates (best intersections (10,−6,0) exp7/8, (6,−6,0)
  exp6/10, (14,−6,0) exp8/11) unable to hit a target 9-13 u above. The draws-6 plan ("re-derive
  the hold spot") is therefore answered with data: **a static re-anchored hold cannot fix this
  stream** — Drake's merged-stream float leaves the deck's under-deck holds. The next
  session's move: fight Drake during his RISE (t0-360 he passes y −1.4 → 2.7, level with the
  lower openings — the probe's t0 sample has ray true + aim Some, and the fight's
  `last_tap = 0` init blocks the first 12 ticks of exactly that window), or give the fight a
  floating pursuit that climbs with him (the Agent `waves_first` machinery already re-picks
  cover but only ±5 around a fixed hold).

- **Every walkreal binary run whole on THIS tree** (`--test-threads=1`, NF_ISO set, slim
  ISO). The task's named gap — draws-6's "the full list was not run" — is closed:

  | binary | result |
  |---|---|
  | walkreal_07000001_live | 7 passed / 1 failed (the d3 00agent walk; draws-6 had 5/3 — the merge IMPROVED it, one red left) |
  | walkreal_07000002_live | 3 / 0 + 1i (82.10 s) |
  | walkreal_07000003_live | 6 / 0 + 1i (234.25 s) — stays green |
  | walkreal_07000004_live | 10 / 0 (13.60 s) |
  | walkreal_07000005_live | 14 / 0 (73.77 s) |
  | walkreal_07000006_live | 5 / 0 + 2i (15.34 s) |
  | walkreal_07000007_live | 5 / 0 (17.72 s) |
  | walkreal_07000008_live | 6 / 0 (144.46 s) |
  | walkreal_07000009_live | 8 / 0 (15.74 s) |
  | walkreal_0700000a_live | 13 / 0 + 2i (52.16 s) |
  | walkreal_0700000b_live | 7 / 0 (41.15 s) |
  | walkreal_0700000c_live | 3 / 0 + 1i (31.12 s) |
  | walkreal_0700000c_barrelrow_live | 1 / 0 (7.45 s) |
  | walkreal_0700000c_eastroute_live | 1 / 0 (11.12 s) |
  | walkreal_0700000d_live | 1 / 0 + 2i (20.18 s) |
  | walkreal_07000011_live | 3 / 0 + 1i (36.56 s) |
  | walkreal_07000011_route_live | 1 / 0 (6.95 s) |
  | walkreal_07000012_live | 3 / 0 (20.10 s) |
  | walkreal_07000013_live | 2 / 0 + 1i (24.89 s) |
  | walkreal_07000014_live | 9 / 0 + 1i (149.72 s) — stays green |
  | walkreal_07000015_live | 6 / 0 + 1i (32.37 s) |
  | walkreal_07000016_live | 12 / 2 + 2i (68.93 s) |
  | walkreal_0700001b_live | 15 / 2 + 2i (155.18 s) — the Drake pair, above |
  | walkreal_07000041_live | 15 / 0 (25.65 s) |
  | walkreal_0700004a_live | 4 / 0 (23.68 s) |
  | walkreal_0700004a_satchel_live | 2 / 0 (8.93 s) |

  The `walkreal_probe*` files are scratch diagnostics, not the walk set. Fourteen-level
  green: every level's walks pass on the merged tree except the three carried reds below.

## What's left

1. **1b's Drake pair** (both `walk_arrival_to_drakes_defeat` and its real-health sister; the
   d2/d3 sisters keep their measured ignores). The root cause and the two candidate fixes are
   above — the rise-window intercept or the floating pursuit.
2. **016's two chasers tests** (`the_chasers_punch_lands_the_record_damage`,
   `walk_the_chasers_defeat_completes_the_mission_on_real_damage`) — the same pair documented
   with full ledgers on the previous branch's report (the chasers' pistol-chip grind −0.6/−1.0
   at 4.6-13 u, the 0.2-u #9 finisher, the live-wire world hit at tick 10959). Not a
   gas-draws regression: the same pair was red before this merge on a tree without the gas
   draws.
3. **01's d3 00agent walk** — the cross-aperture named-guard fight (the previous branch's
   report holds the ledger: guard 6's threshold fight works, guard 11's threshold runs the
   pool dry, lip guard #5 finishes).
4. The full four-crate suite did not fit the window again (the standing ~3 h measurement);
   every binary of the walk set ran whole instead, and no `crates/*/src/` file changed on
   this branch (the merge's src changes are master's own, already covered by master's runs).

## Live checks for the main agent

1. 1b's Drake float on the merged stream: his obj position every second from the fight gate —
   does the real game's Drake also take the high hover band (y 4-7.6 over the deck), or does
   he stay level with the hub floor where the pre-merge fight killed him? If the game's Drake
   stays low, the port's float drift is the bug, not the test.
2. The same fight: Drake's state byte during the hover (the probe saw 0xc0/0xc1 alternating)
   — which states are his hover/attack pair, and does the game's Drake descend to fire?

## Verification notes

- Every gas draw survived the merge: the only conflicted file was a test file; the src merge
  was clean (gas.rs's rng-signature sites re-based in draws-6's merge and untouched since).
- No test was weakened this session (the probes were reverted; the tree carries only the
  merge + the WORKLOG).
- The token never left the clone URL; `git status --short` checked before every commit.

## Usage

not visible to me (no token, credit or context-window figures are exposed to this session;
nothing in the sandbox reports them).

## ntfy failed

All three fallbacks failed in order (2026-11-06 ~04:20-04:24 UTC):
1. `curl -sS -m 20 -H "Title: NightfirePort" -d "..." https://ntfy.sh/nightfire-yoda-8k3x`
   → `curl: (28) Connection timed out after 20001 milliseconds`
2. `python3 -c "... urllib.request ..."`
   → `urllib.error.URLError: <urlopen error [Errno 101] Network is unreachable>`
3. `wget -q -O- --post-data "gas-rng-draws-7: done" https://ntfy.sh/nightfire-yoda-8k3x`
   → no output, killed at the 45 s tool timeout.

The finish step itself succeeded (`bash ../nf-queue/finish.sh 50-gas-rng-draws-7 done
docs/reports/gas-rng-draws-7.md` → `OK done 50-gas-rng-draws-7`). The previous session
(guard-radii-finish, ~03:00 UTC) saw the identical three failures — ntfy.sh is unreachable
from this sandbox.
