# retune-nb-0d-20 — round 20 of the 0d arrival-to-C5-door real-damage walks: the roof-penthouse lever is DEAD BY DESIGN (its two doors are fakedoors with an unwritten lock channel), the scan's third armour (the north-east ground pocket) measured DEAD as wired twice, and the armour-volume lever class is exhausted — still RED, the ledger refreshed

Branch `s/retune-nb-0d-20`, from origin/master 7562f1e70 with origin/s/retune-nb-0d-19 merged in
first (0d-19 is not on master; clean carry). **The task walk
`walk_arrival_to_the_c5_door_real_damage_agent` is still RED — the ignore stays, refreshed with
the round-20 ledger and the next lever.**

Mid-round note: **this sandbox RESET at ~09:01Z** (the second driver window). The clone, the game
files, rustup and the ALSA prefix were rebuilt from the driver message; the branch was recovered
from origin with both earlier commits intact (`1b650fd00`, `a0433e5a3`), and the three lost
edits (an eprintln arg fix, the `NF_0D_NARMOUR=2` composition block, the ignore-message
refresh) were re-applied and re-verified tick-exact against the pre-reset measurements.

## The levers this round

The 0d-19 ignore message's next untried lever was **the roof armour via the committed route's own
waypoint class** (`Wp::new(147.0, 21.76, -252.77)` between the west-roof acts, NOT
`walk_nav_to` which fell off the edge [L 19-6]). Measured:

- **(a) The roof penthouse is SEALED BY THE LEVEL'S OWN DATA.** The armour sits on the west-roof
  guards' floor: the guards patrol at origin y 22.70 [L probe_0d_roof_rays wake census], the
  teleport probe settled Bond on that floor (origin 22.73) and the touch took, armour 0 -> 50
  [L probe_0d_roofarm_path]. But the floor's ONLY doors are **fakedoors #517/#520** at
  (147.12, 22.94, -258.12) and (148.27, 22.94, -236.15), both locking on ch 200 which has NO
  writer in the level's data — locked forever in BOTH implementations [D docs/levels/
  0700000d-audit.md kind-0xDB table]. And every staged exterior ascent from the main roof
  (floor 18.58) fell or stalled: the north diagonal to y 7.77, the band line to y 11.46, the
  long climb to y 1.04, the hop variant to y 16.6, and the north-edge hug stalled at z -266.8
  [L probe_0d_roofarm_path/probe_0d_roofarm_path2]. The walk_to class CANNOT reach the
  penthouse — the lever is dead by design, and the `Wp`-leg variant the ignore message asked
  for cannot exist.
- **(20-1) The north-east ground armour** (the 18-12 scan's third, (205.41, 0.11, -205.31),
  ground level directly under the warehouse start — the one untried member of the scan's four
  armour sources): the take **ARMS AT d2** [L]. The probe chain measured: the blind-strip band
  approaches stall on a player-collision wall at z ~-208.9 for x 202-208.5 (invisible to
  bullet rays — `segment_hit` shows nothing there), but the x-210 lane north is open (x 209.5
  and x 213.5 both pass to z -204) [L]; the pocket settle takes the touch mid-walk (armour 0
  -> 50.0 at d2 [L probe_0d_north_armour3], and armour 22 -> 50 in the real run); the return
  lane south rejoin measured OK [L]. `NF_0D_NARMOUR=1` wired the detour into the band phase
  (after the strip-mouth #10 act, before the east-3 leg; d2 real damage only; unset =
  byte-identical). **MEASURED DEAD as wired**: the ~1900-tick open-ground detour walked into
  the deck line's field — death **t3953** in the pocket's east-out leg, 8 hits for 100.0 plus
  ~100 armour eaten: the deck sniper #22 (205.4, 12.1, -289.0) landed **68.0 in 4 hits at
  d 84.8**, the deck guard #20 (196.9, 14.0, -264.1) **28.0 at d 57.5**, the bay #11 4.0 [L].
  The ground-band snipers #36/#21 stayed SILENT through the whole detour — the deck/bay line
  owns the open pocket ground.
- **(20-2) The composition** (`NF_0D_NARMOUR=2`): the strip-mouth #22 head act FIRST (the act
  landed its kill in the round-1 ledger [L 0d-4]) before the detour. **MEASURED DEAD**: the
  act's box ([209.0, -289.3] ± (7.0, 3.5)) picked **#11** — the nearest live in the box; #22's
  round-1 pick was its own roll's west reposition — #22 still fired **56.0 in 3 hits**, and the
  act itself cost **34.0 pool** of strip-mouth dwell (100 -> 66, +273 ticks); death **t4174**
  [L]. Both runs reproduce tick-exact (re-measured on the rebuilt tree after the sandbox reset:
  t3953 and t4174).

**The armour-volume lever class is now MEASURED EXHAUSTED**: the committed annex grab is taken;
the yard-west armour (107.75, -275.16) sits ~15 u beyond the budget death [L 19-2]; the roof
penthouse is sealed [D+L]; the north pocket is dead [L 20-1/20-2]. The round-19 composition's
premise — absorb the conserved residual bill with armour volume — has no untried member left.

## How it was verified

- The whole `walkreal_0700000d_live` binary on the final tree: **2 passed / 1 failed / 2
  ignored in 70.21 s** — the plain walk + the d1 Operative walk GREEN (the byte-identity holds
  with all knobs OFF — every knob is `difficulty == 2 && real_damage` + env-gated, default
  OFF), the 00agent failure the pre-existing batch-3 signature (t3995, 4 hits 100.0 [L 16-0]),
  the Agent walk ignored (its own runs above).
- Lesson-10 grep: the round's diff touches ONLY `crates/nf-view/tests/` (the 0d test file +
  five assertion-free probe files, `probe_0d_roofarm_path*` and `probe_0d_north_armour*`,
  never run in CI) — no `crates/*/src/`, no common helper — so the nf-game / nf-formats /
  nf-audio suites are unaffected by construction (0d-16 measured them 2153 passed / 71 failed /
  11 ignored on this same merged line). Lesson-7: no common-helper change, so no cross-level
  walkreal re-run is required by the rule; the whole 0d binary above ran anyway.
- The fast suite (`tools/test/suite.py`): re-fired twice this window (once with the full env
  after the first attempt died on `FileNotFoundError: 'cargo'` — the reset had also wiped
  rustup). **Both died between tool calls** (the second: 0 processes, no output file — the
  rounds-13..19 signature, re-confirmed at ~09:20Z). The substitute, per the 0d-16..19
  precedent: the touched binary whole (above); the round's diff cannot change the other three
  crates' suites.
- No guard code changed, no damage/health constants changed, no top-ups: every new path is a
  REAL-input act (the walk's own act machinery) or a REAL walk leg (the Wp/try_walk_to class),
  and every knob defaults OFF so the other walks and replays stay byte-identical (the whole
  binary's plain + d1 greens are the proof).
- Live checks for the main agent: none required (no visual work; the deaths are fully
  instrumented by the test's own damage log).

## What's left

- The Agent walk is still red. The refreshed ignore message carries the round-20 ledger and the
  next lever: **the TRENCH-LANE DETOUR to the yard-west armour** (107.75, 6.84, -275.16) — the
  direct lane's chewers (the yard-EAST pair #7/#1) fire west along the yard band; the
  under-dock trench (floors 3.0/0.0, z -266..-272, the yard-side wall 3.9 at z -272 [D probe
  walkreal_probe0d.rs]) sits north of the pair's line and its 3.9 rim plausibly blocks their
  low line into it [S unverified — geometry-derived, never measured]; the trench-edge climb is
  the route's own proven class [M the walk's return leg]. A Wp-leg detour: the exit stand ->
  the trench entry -> west inside the trench -> the trench-edge climb out at x ~108 -> the
  armour touch -> the return. Judge it on its own real run (the phase shift re-rolls every
  downstream window [L the rounds' law]).
- The 00agent walk still needs its own re-green on the batch-3 tree (pre-existing, unchanged by
  this round).
- The full four-crate suite still needs a bigger window or a sandbox that lets background
  processes live (the rounds-13..20 wall); the per-binary evidence above is this round's
  substitute.

## Usage

not visible to me — this environment shows no token, credit, cost or context-window figures for
the session (no gauge, no billing panel, no API counters are exposed to the agent).

## ntfy failed

All three transports timed out (the round-17/18/19 blackout signature, re-measured 2026-11-23
~09:28Z):

1. `curl -sS -m 20 -H "Title: NightfirePort" -d "..." https://ntfy.sh/nightfire-yoda-8k3x`
   → `curl: (28) Connection timed out after 20001 milliseconds` (exit 28).
2. `timeout 25 python3 -c "import urllib.request as u; u.urlopen(...)"` → exit 124 (killed at
   the 25 s timeout, no output).
3. `timeout 25 wget -q -O- --post-data "retune-nb-0d-20: done" https://ntfy.sh/nightfire-yoda-8k3x`
   → exit 124 (killed at the 25 s timeout, no output).

The queue entry (`done/50-retune-nb-0d-20.md` on origin/queue, commit after 67869bc4c) carries
the full report body (attached on the second pass — the first finish.sh pass missed the
relative path because the script cd's to the queue worktree; the header committed then, the
body appended and pushed right after). The notification could not be delivered from this
sandbox.
