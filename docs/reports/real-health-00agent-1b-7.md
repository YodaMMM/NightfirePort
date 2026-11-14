# real-health-00agent-1b-7

Task: take the round-6 ledger's named next lever (the k6 chain start timing — the
melee-override fight loop) on 0700001b's 00 Agent real-health walk
(`walkreal_0700001b_live::walk_arrival_to_drakes_defeat_on_real_health_00agent`, still
ignored), measure each change on its own real run, and write the ledger back.

## What's done

All work is in `crates/nf-view/tests/walkreal_0700001b_live.rs` (the 00 Agent walk only —
every change is inside the `hard` branch or hard-gated, so the d1/d2 walks are
byte-identical; `CONSOLE_ORDER` itself is untouched). The test is still ignored (still red),
but the round advanced from "death 7442 with three consoles green" to "death 7594 with ALL
FIVE mid consoles green":

1. **The fight loop is solved (the round-6 lever 1).** The loop existed because `cooled`
   counts from the fight's START and a cap-long fight ends exactly when the cooldown
   expires — the baseline's fights 6714..7014 / 7014..7314 / 7314..7442 ran back-to-back
   with ZERO console ticks between, so no k6 hold ever answered. The fix that survived
   measurement is the **deadline-tightness screen window**: a zero-kill full-cap fight whose
   pending console's remaining read is under `CHAIN_NEED = 790` (one fight 300 + the
   measured re-acquisition 256 + the chain 230) sets `last_defense = fight_end + 300`, so
   the next defense waits 600 real ticks while the console work runs; a kill or an early
   exit resets the streak, and streak 2 (two consecutive zero-kill full-cap fights) opens
   the window too. Result: the route stays byte-identical to the round-6 baseline through
   7014, the window opened at 7014, k6's holds answered 7301/7409/7517 and **k6 latched
   7519, margin -255** — the exact round-6 good shape.

2. **Four anti-levers measured and reverted** (all documented in the ignore message so the
   next round does not re-run them):
   - a time-based no-progress exit (a fight with no kill by 150 ticks ends there): the
     measured kill shape needs 286-388 ticks, the exit truncated every kill fight, the wave
     grew 3 -> 9 un-thinned, m3 LAUNCHED, death 6580;
   - the window after the FIRST zero-kill full-cap fight: the baseline's 3105..3405 is
     zero-kill full-cap and its follow-up 3405..3556 thins 3 -> 1; delaying it grew the wave
     to 10, death 6530;
   - chain-state windows (window when the pending console's chain has not started): the
     consoles latch OUT OF PHASE with the fights — a fight-end diagnostic pinned pending 4
     at BOTH 3105 AND 3405 (k5's write lands ~3104 right after hold#3 3103 in the baseline
     too; the "latch k5 3556" print was fight-lagged) — death 6626 in every variant;
   - the deadline-yield melee arm (the 12 u band collapses to 3.5 under a tight read):
     skipping the 6714 fight left the station flow uncovered 1060 ticks, the wave closed and
     shredded it, death 8502 with m6/m8 launched.

3. **The k8 binder decoded and solved.** k6 and k8 share the modeled read 7774
   (missile_start 5375 + P0_LEN - 1) and BOTH missiles read on time (m6 launched on the
   +14-late k6 latch; m8 launched mid-chain) — the modeled deadline IS the real read for the
   +5999 pair (m4's +1200-late p0 is the outlier, not the law). With the 6->8 order the
   crossing (137) + k8's chain (216 + the write) lands ~7872, ~98 past the read. So the
   00 Agent console order is now `CONSOLE_ORDER_HARD = [5,4,3,8,6,1,2,7]` (hard-mode only):
   k8's holds answered 6452/6560/6668 and **k8 latched 6670, margin -1104**.

4. **The chain fence tightened.** The old `deadline_in > 900` arm admitted a fight 50 ticks
   after k6's hold#1 (6773); it ran zero-kill and the uncovered window stand died 7595. A
   started chain now admits only the 3.5 u always-hit emergency (the doubled-damage band);
   k6's holds then answered 6723/6842/7118 and the write landed before the read (k6 <=-180).

5. **The tail wave fight took the mid-phase kill shape** (hard-gated): the pinned approach
   lane + the nearest-shooter prio + the 2-kill exit + the band-safe max_y 7.0, replacing
   the plain hold-the-station fight that held Bond uncovered for 1382 ticks and died at
   8502.

## How it was verified

- 11 real runs of the walk itself, each a full deterministic in-process sim from the shuttle
  arrival on real input only (no top-ups, no health/ammo/channel writes): `/tmp/baseline.log`
  (the merged master shape, reproduced the round-6 ledger exactly) and
  `/tmp/lever1..11.log`. Every lever was measured on its own run; every anti-lever was
  reverted after its measurement.
- `walkreal_0700001b_live` whole binary (19 tests): **18 passed, 0 failed, 1 ignored** (the
  00 Agent walk itself) — includes the d1 and d2 real-health walks, byte-identical paths.
- `interactables_0700001b_live`: **9 passed, 0 failed**.
- Lint: the touched code is a test file; `cargo test` builds it clean (warnings only, all
  pre-existing).

## What's left

The walk still dies at **7594 inside the k1 tail wave fight**: the 5-6-strong wave FOLLOWS
Bond to the x20 stations, the kill shape cannot reach it (zero kills in 474 ticks — the same
out-of-reach shape the mid-phase loop fought) and the cover floats did not outlast its fire
(hp 0.0, 2751 ticks before the +9599 tail read ~11253). The next lever is named in the
ignore message: thin the following wave BEFORE the tail (the mid-phase's last zero-kill
fights at the x46 crossing grew it 4 -> 7), or give the tail fight a reachable target (wait
the ceiling-band shooters down to the <=8.3 aim heights, or hold the x20 bank's low shelter
— the d3 run survived this phase at hp 39.3 with a thinner wave); port the CHAIN_NEED streak
window to the tail block if a tail fight stalls. Then the Drake phase (the y > 9 spawn-band
re-weight). The full ledger with all evidence tags is in the test's ignore message.

## Usage

not visible to me

## ntfy failed

All three notification methods failed from this sandbox:

1. `curl -sS -m 20 -H "Title: NightfirePort" -d "..." https://ntfy.sh/nightfire-yoda-8k3x`
   → `curl: (28) Connection timed out after 20002 milliseconds`
2. `python3 -c "... urllib.request ..."`
   → `urllib.error.URLError: <urlopen error [Errno 101] Network is unreachable>`
3. `wget -q -O- --post-data "..." https://ntfy.sh/nightfire-yoda-8k3x`
   → timed out after 60 s and was killed (no output).

The finish step itself succeeded: `finish.sh` reported `OK already closed in done/, report
attached`, and `done/50-real-health-00agent-1b-7.md` is present on `origin/queue`
(e7fc79c79).

