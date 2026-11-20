# retune-nb-08-10 — 08 whole-mission 00-Agent real-damage walk, round 10

Branch `s/retune-nb-08-10`. `origin/s/retune-nb-08-9` was not on master, so it was merged in
first (clean — the merged tree is exactly the 08-9 final tree). Still red, but the death moved
1453 ticks deeper — t28256 at (113.08, 13.47, −178.96), the fight ran ~2000 ticks (was ~600) —
and the round-9 ledger's priority lever (the pre-fight walk-down zaps) is structurally answered.

## What was done

1. **The re-measure on the merged tree** (origin/master + 08-9; no `crates/*/src/` changes): the
   baseline reproduces the round-9 ledger EXACTLY — death t26803, fight-entry armour 0.0 /
   health 20.0, the 12 walk-down zaps t25666-25793 [L the 2026-11-20 baseline NF_DMG_TRACE run,
   18.9 s]. The zaps' anatomy (new, from the same run's rows): the ride ends INSIDE the parked
   car at (111.66, 13.54, −179.17); the ~1500-tick bottom-of-mountain cinematic runs with Bond
   standing there ZAP-FREE (the hurt volume is not armed yet); the volume ARMS with the machine's
   approach ~106 ticks after `wait_enabled` returns — the first zap t25666 lands mid-shoot-out
   (8 zaps during the shoot-out, Bond still at the ride-end spot), then 4 more during the anchor
   walk + drop press t25775-25793. So the whole −24.0 was the pre-fight tail standing in the band
   AFTER the arming, not the drop itself.
2. **SHIPPED lever (1) — the ledger's priority item**: the whole d3 drop sequence (the anchor
   walk_to + the drop press + the crouch drop + the lockout + the stand-off walk + the stand
   press) MOVED above the pane shoot-out, running right after the cinematic ends. The drop takes
   ~65 ticks from the ride-end spot and lands the zap-free plate (feet 13.086) BEFORE the arming;
   the shoot-out then runs from the rim. All blocks stay d3-gated, byte-identical — only their
   position in the walk changes, so the d1/d2 sisters' timelines are untouched by construction.
   MEASURED [L the lever-1 run, 20.5 s]: the pre-fight zaps drop 12 → 1 (ONE −2.53 row at t26175
   at the stand spot); the fight entry returns to health 41.5 (was 20.0 — the earned 44.0 minus
   the one rim row); fight t0 26248 (the tick-neutral reorder — the machine's route phase at
   fight t0 is unchanged); death t27636 at (113.08, 13.47, −178.93) — 833 ticks deeper.
3. **SHIPPED lever (2) — the fight anchor**: the lever-1 run measured the OLD anchor
   (112.5-112.6, −179.39) at rim height ZAPPING the fight away — 13 −2.00 rows in the last ~830
   fight ticks (the dense burst every 6 ticks t27409-27636) closed a walk that had survived every
   missile salvo — while the shipped stand spot (113.08, −179.02) measured ONE −2.53 row in ~430
   ticks and the round-6 launches FIRED from it (t377/t585). The d3 `ANCHOR` moves there (the
   dodge's anchor-ward side pick and the walk-back both read the one constant, so they move with
   it; the d2 anchor stays byte-exact). MEASURED [L the lever-2 run, 21.5 s]: the anchor zaps are
   GONE (Bond held 13.6 health for 981 fight ticks at the anchor), the fight reaches ~2000 ticks
   (the round-7 ledger's low-band window), death t28256 — 1453 ticks deeper than the shipped
   tree.
4. **The remaining wall — ZERO launches in BOTH lever runs** (line_ok FALSE at all 105 fight
   samples; the machine's hp 598 untouched): the machine's route phase at fight t0 (path t ~630 —
   the approach nearly done) flew the whole fight window on the southeast far legs (dist 36-61,
   y 16-22) and the HIGH overhead passes (y 24-29 at dist 15-27). From the stand spot the
   southeast bearings exit through the south-east corner post (the ray to (135.8, 16.5, −209.5)
   crosses z −182.18 at x ~115.4, ~1.8 u east of the shot-out south pane's east edge 113.62) and
   the high passes stop on the lintel. **The dive is now MEASURED** [L the approach dump shipped
   this window — the pre_fight buffer printed at the fight loop's start, an eprintln-only
   d3-gated diagnostic]: the machine spawns f~24873 (path t 0, ch5's gate) and its approach dips
   INTO the launchable band at frames ~25710-25800 (path t 424-470) — y 13.61 (f25710) → 13.08
   (f25740) → 12.84 (f25770) → 13.07 (f25800) → 13.93 (f25830) — at x 69-93, z −214..−219,
   dist ~49 southwest of the stand spot (the round-4 launch distance class 45-48). The fight
   loop opens f26248 — ~450 ticks AFTER the dive ends. **The tail is now MEASURED** [L the tail
   stamps shipped this window, eprintln-only]: the drop block runs f24872-25309 (437 ticks —
   ~140 of them the stand-off east walk WEDGING at k80, the (113.3, −179.25) target 0.22 u short
   of reachable), the shoot-out runs f25309-26196 (887 ticks for ONE pane round), the fight loop
   opens ~26248. The 2000 observed fight ticks never saw the machine below y 18.4, so the low
   band does NOT recur inside the fight window (the round-7 "low band arrives at fight
   t~1400-2100" was that tree's fight window).

## How it was verified

- Each lever judged on its own real run of the d3 00-Agent walk (NF_DMG_TRACE; 18.9 s baseline /
  20.5 s lever-1 / 21.5 s lever-2) [L the three runs above].
- `walkreal_07000008_live` whole binary on the final tree (merged with origin/master — master is
  already an ancestor; no new commits landed during the window): **6 passed / 0 failed /
  1 ignored** in 111.74 s — the d1/d2 real-damage sisters, the plain walks and both ally tests
  stay green.
- The final tree's only code delta vs the merged 08-9 state is
  `crates/nf-view/tests/walkreal_07000008_live.rs` (the d3 block relocation + the d3 ANCHOR move
  + the relocation notes + the ignore-message ledger). No `crates/*/src/`, no
  `crates/nf-view/tests/common/` change survives, so no other test binary compiles the changed
  file — the affected-binary set is `walkreal_07000008_live` alone, and it ran whole.
- The full suite (tools/test/suite.py) did NOT fit the window, for environment reasons: the
  runner's background start (15:28 UTC, the first minutes) was killed by the sandbox between tool
  calls and left an empty `suite.out` (the same failure as rounds 8-9); the foreground build of
  the ~200 nf-view test binaries does not fit the remaining window (a single `--tests` build
  already consumed 10 minutes at 15:41). What ran instead, on the final tree: the whole
  `walkreal_07000008_live` binary above, plus `nf-game --lib` 796 passed / 0 failed / 4 ignored
  and `nf-formats` disc_golden's documented slim-ISO trio excepted — unchanged code, same
  evidence as round 9's report (no `crates/*/src/` change this window, so those binaries are
  byte-identical in behavior; the numbers cited are round 9's final-tree runs on the same merged
  code). NOTE: the nf-game/nf-formats runs listed are round 9's, not this window's — this window
  changed only the one nf-view test file, whose binary ran whole here.

## What's left (the next window's levers, in priority order)

1. **Instrument the shoot-out's internal stages, then compress the tail** — the tail is now
   measured (437 + 887 + ~52 = the fight loop at 26248): the shoot-out's 887 ticks for one pane
   round need the internal split (the convergence ticks, the %12 fire boundary, the 120-tick
   sidestep retries), and the east walk's 140 wedged ticks free with a shorter target. Compress
   so the fight loop opens ≤f25710 — INSIDE the dive (f25710-25800) — with the panes shot and
   Bond at the stand spot; the compression budget is ~540 ticks.
2. **The east-pane bearing** — a stand spot a few tenths south (z −179.3..−179.5, still
   rim-height zap-free per this window's zap map) swings the machine's southeast circuit bearings
   into the east pane pair's windows (113.37-113.62, −180.7..−181.6).
3. **The trio 36.0** — unchanged: every deck-leg lever has measured dead; the live check for the
   STATION HALL MOUTH's (139.0, 49.5, −93.0) sight lines into the trio's placements stands.

## Live checks for the main agent

- The machine's approach dive: between the cinematic's end (~f25560) and the fight loop's start
  (f26248), confirm the machine's LOW pass (y 12.8-13.7, the round-4 launchable band) with its
  frame range and bearing from the stand spot (113.08, 14.18, −179.02) — which of the shot-out
  panes' windows (the south pair z −182.18, the east pair (113.37-113.62, −180.7..−181.6)) the
  dive's bearing crosses. That pins next window's lever 1 (the fight-loop start tick) and lever
  2 (the stand-spot shift).
- The station hall mouth (139.0, 49.5, −93.0) standing eye (unchanged from round 9): which of
  the trio's placements (#2 (161.6, 46.0, −97.4), #14 (152.4, 46.0, −96.2), #11 (154.3, 46.0,
  −87.6)) can Bond hit from the mouth — if any lane reads clear, the deck transit can pre-fight
  the trio from the hall side and the 36.0 never lands.

## Usage

not visible to me
