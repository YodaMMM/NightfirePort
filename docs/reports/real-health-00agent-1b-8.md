# real-health-00agent-1b-8

Task: take the round-7 ledger's named next lever (the tail-phase survivability — a
REACHABLE target for the k1 tail wave fight) on 0700001b's 00 Agent real-health walk
(`walkreal_0700001b_live::walk_arrival_to_drakes_defeat_on_real_health_00agent`, still
ignored), measure each change on its own real run, and write the ledger back.

## What's done

All work is in `crates/nf-view/tests/walkreal_0700001b_live.rs` (the 00 Agent walk only —
every change is inside the `hard` branch, an `aim_close`-flagged fight, or `!hard`-gated, so
the d1/d2 walks are byte-identical: the lever-3 run prints the exact round-7 latches k5 -18 /
k4 -457 / k3 +140 / k8 -1104 and the exact fight frames 3105..6114). The test is still
ignored (still red), but the round advanced:

1. **The tail fight's ZERO-TAP shape is solved (the round-7 lever).** A per-60-tick dump
   around the round-7 k1 fight (7120..7594) showed the fight fired ZERO shots — clip only
   recharged 36→100 — because the threat-ranked aim pick chased ONE riser's aim height the
   whole fight (the wave rises ~0.04 u/tick, faster than the aim lift) and the level-only
   view's ray never sat on a hit box, while the wave closed to d 2-3 u (doubled damage) with
   LINED 6 at every sample. The wave's reachable aims (2.5-7.4 vs the max_y 7.0 cap) sat one
   pick away the whole time.

2. **The round-8 levers (kept in the code, measured on their own runs):**
   - aim_close targeting: the smallest |aim − eye| gap first, threatening over blind, nearer
     next, the 3.5 u melee band on top (the hard tail fight only);
   - muzzle-offset compensation on the aim_close taps (ShotCal learn/eye_target, the
     hide_and_snipe pair) — the uncalibrated taps MISS at the tail's 6-15 u (lever-1 fight:
     3 taps, 1 kill, the pool 36→9 dry);
   - the climb-fallback: the level view hits only the eye's own height, so from the low cover
     every shooter at aim 5-11.9 reads exposed_aim None — the lever-2 fight sat 1200 ticks
     with clip 100 and zero taps bleeding hp 23.7→4.9; the pick now takes the LOWEST shooter
     whose estimated stance is inside the cap and climbs to it;
   - the tail fights capped at 600 (the 2800 cap let one stall run 1853 ticks while the wave
     grew 5→9) with the defense cooldown anchored at fight END+300 (a start anchor chained
     fights back-to-back with zero console ticks — lever-1: 7120..7937/7937..8772, no k1 hold
     ever answered, death 8772), plus the tail hold-chain fence and the shared 300-tick
     tail_cooled gate (hard only; the d1/d2 walks keep the gateless gate).

3. **Result [L /tmp/lever3_r8.log]:** the fights END (600 caps) with taps and kills landing
   (fight 1: clip 36→9, astro 6→5, lined 6→0 by its end), the console gaps run 600 ticks,
   and **k6 LATCHED −54** (its write rode fight 1's end — better than round-7's ≤−180).
   Death moved 7594 → 9134 (hp 47.7 → 0.0, astro 8, k1/k2 unlatched).

4. **The remaining binder:** the x20 station's exposure. The wave REPLENISHES (astro 5→8
   across the 8320..8920 fight — the tail missiles m1/m2/m7 deploy ~8075 and their spawn
   waves land with them [S — the trigger is the deploy, not pinned]); the con1 ring scan
   reads 12/120 stances shootable+roomed, but their exposure at lined 5-8 is unmeasured, and
   the hard stance gate (exposure ≤ 1) never passed in any gap — no k1 hold answered in any
   round-8 gap.

## Live checks for the main agent

- At 00 Agent, on the tail approach (~t7100+, Bond at the x20 station area, hp ~47): stand at
  each of the 12 shootable+roomed con1 ring stances (the [ring] scan at t4670 lists them) and
  record how many astronauts hold a muzzle→eye line from each — the round-9 lever needs the
  stance with lined ≤ 1. Address range: the con1 station (20.0, −2.0, 0.0) ± 14 u, eye at the
  screen's aim height.
- Whether the tail wave's spawn waves stop once m1/m2/m7's p3s finish (~t11253+): if the
  wave thins naturally after the reads, a pure survival hold (the under-deck shelter) is a
  valid round-9 fallback.

## How it was verified

- 4 real runs of the walk itself, each a full deterministic in-process sim from the shuttle
  arrival on real input only (no top-ups, no health/ammo/channel writes):
  `/tmp/diag_r8.log` (the round-7 shape with the tail-fight dump — reproduced the round-7
  ledger exactly: death 7594, astro 5, zero taps) and `/tmp/lever1..3_r8.log`. Every lever
  was measured on its own run; the levers that measured WORSE than lever 3 (the 2800 cap /
  start anchor in lever 1, the deadlock in lever 2) are documented in the ignore message.
- `walkreal_0700001b_live` whole binary (19 tests): **18 passed, 0 failed, 1 ignored** (the
  00 Agent walk itself) — includes the d1 and d2 real-health walks, byte-identical paths.
- `interactables_0700001b_live`: **9 passed, 0 failed**.
- The full four-crate suite did not fit: this sandbox kills background jobs between tool
  calls (the nohup runner died twice), so `tools/test/suite.py` cannot run to completion in
  the 2-hour window; the build for it was run (all four crates' test binaries build clean)
  and the two binaries my change touches are green above. No other test binary calls the
  changed code (the change is confined to this one test file; `grep -l fight_opts_y
  crates/*/tests/*.rs` matches only `walkreal_0700001b_live.rs`).

## What's left

The walk still dies at **9134** with the wave 8 strong and k1/k2 unlatched. The next lever is
named in the ignore message: the k7-FIRST order swap (CONSOLE_ORDER_HARD [5,4,3,8,6,7,1,2] —
latch k7 at the clear x46 nest while the wave trails at x16-39, then the k1/k2 chains at the
x20 station with the wave chasing from behind; the +9599 shared deadline gives every order
huge slack), with the con1/con2 stance-exposure probe measured first, and the round-8 fight
shape (aim_close + cap 600 + END+300) as the gap filler. Then the Drake phase (the y > 9
spawn-band re-weight, the round-6 lever 3). The full ledger with all evidence tags is in the
test's ignore message.

## Usage

not visible to me

## ntfy failed

All three notification methods failed from this sandbox (10:0x-10:4x UTC):

1. `curl -sS -m 20 -H "Title: NightfirePort" -d "..." https://ntfy.sh/nightfire-yoda-8k3x`
   → `curl: (28) Connection timed out after 20002 milliseconds`
2. `python3 -c "... urllib.request ..."`
   → `urllib.error.URLError: <urlopen error [Errno 101] Network is unreachable>`
3. `wget -O- --post-data "..." https://ntfy.sh/nightfire-yoda-8k3x`
   → hung on `Connecting to ntfy.sh|159.203.148.75|:443...`, killed by timeout (exit 124), no output.

The finish step itself succeeded: `finish.sh` reported `OK done 50-real-health-00agent-1b-8`
(the report path arg is repo-relative and the script runs from the queue worktree, so the
first pass attached only a stub) — the full report was then attached to
`done/50-real-health-00agent-1b-8.md` and pushed to `origin/queue` (9fdf77b04).
