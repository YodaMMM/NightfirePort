# retune-nb-08-9 — 08 whole-mission 00-Agent real-damage walk, round 9

Branch `s/retune-nb-08-9`. `origin/s/retune-nb-08-8` was not on master, so it was merged in
first (clean). Still red, but the walk's deepest point moved 4154 ticks down the route: the
death is now t26803 at (113.1, 13.5, −178.9) — the shipped tree died t22649 — and the pit
pair's 28.0 bleed is structurally answered by the shipped lever.

## What was done

1. **The re-measure on the merged tree** (origin/master + 08-8; no `crates/*/src/` changes
   this window either): the baseline reproduces the round-8 ledger EXACTLY — death t22649,
   fight-entry armour 5.0 / health 20.0, the y-46 trio 36.0 at the wedged deck pin
   (149.5-149.6, 46.1, −93.1) t738-1009, #16 28.0 in 5 on the pit-excursion descent legs
   t5168-5583 [L the 2026-11-20 baseline NF_DMG_TRACE run, 18.1 s].
2. **Lever (1) the deck pin — the stand pull west 2.5 u: TRIED and DEAD on its own real run**
   (the ledger's priority item). Implemented as an opt-in `stand_pull` walk in
   `damage_log.rs` (default `None`, d3-only setter in the east-deck-head legs; the pull is a
   real lenient walk_to under a 600-tick nav cap, `fighting=true` so the pull's own
   return-fire gate cannot recurse). MEASURED [L the lever-1 run, 7.9 s]: the pulled stands
   STILL took the −4.0 rows at the pulled spots — the trio's lanes cover the WHOLE deck line,
   not just the pin (#2 hit at (148.4, −93.8) d 13.7, #11 at (145.1, −92.4) d 10.4, #14 at
   (147.5, −92.3) d 6.2) — and the fights ran LONGER from the lip (the dry pistol's 1-round
   cycle walked back east between taps): the deck-head fight armed 12.0 (baseline ~52.0) and
   the walk died t7313 on the pit ramp to #16 — 15336 ticks WORSE. REVERTED (the helper is
   back to master's bytes; the dead-lever note stands at the walk site). The conclusion: there
   is NO out-of-lane spot on the deck; the deck-pin bleed is not a stand-spot problem.
3. **Lever (2) the aim press — TRIED and DEAD on its own real run** (the ledger's next item,
   marked [S] unverified). Implemented as a 30-tick aim hold before each excursion descent
   leg (the raise queues on the aim-PRESSED edge and completes under the held bit [M
   0x1a24c8:113-152]). MEASURED [L the lever-2 run, 11.4 s]: the raise is NOT the stands'
   latency — the hip cone fires during the raise [M Player_SetWeaponAnimObj case 0xf; the
   sniper_stands settle note], so the first visible tick already fired — and the holds (5
   legs × 30 ticks) wobbled every downstream guard roll: #16 still cost 24.0 and the walk
   died t18365 to #26's burst rifle point-blank (d 1.0) at (106.9, 28.4, −120.3) with the
   armour drained — 4284 ticks WORSE. REVERTED (the dead-lever note stands at the excursion
   site).
4. **SHIPPED: the pit fight's legs run under `level_fights`** (the ledger item (2)'s
   substance, found while measuring the mechanism: the descent legs' return-fire stands on
   the below-level pit pair (dy 3.1-6.1) were ALL blind and ALL failed — killed false
   t5044/t5384 [L the baseline run] — and each stood Bond in #16's lane for up to 300 unseen
   ticks). The flag is the EXISTING opt-in ±2.0-y pick filter (drone-block-swaps-g055-4);
   it is set around the ONE `fight_guards_wide("the pit before the burns", …)` call, d3 +
   real-damage gated, restored after. The d1/d2 sisters and the top-up walks never run that
   block, so their timelines are untouched. MEASURED [L the lever-3 run, 18.3 s]: the blind
   stands never open; the excursion's own machinery owns the pair (the ramp-mid stand killed
   #15 at t5250 — 290 ticks faster than the baseline's t5540); **#16's bleed drops 28.0 in 5
   → 8.0 in 1** (the one walk-into hit at (153.6, −113.4) t5227); the walk crosses the
   mid-game at health 44.0 (the shipped tree crossed at 20.0, flat from t6331). The boarding
   chunk then drains the ARMOUR (100 → 0.0, health intact through t22616), and the terminal's
   upper-floor hurt-volume band ZAPS THE PRE-FIGHT WALK-DOWN: −2.00 × 12 at (111.7, 13.5,
   −179.2) t25666-25793 (feet y 13.5 — the round-4 ledger's upper-floor band; the lower floor
   13.086 is zap-free) — the +24.0 mid-game gain is spent standing on the band before the
   crouch drop, and the walk enters the fight (fight t0 = frame 26206 [L the fight log's own
   t0 line]) at armour 0.0 / health 20.0 — the SAME entry health as the shipped tree. The
   fight runs ~600 ticks and dies to the −2.33/−13.22/−4.45 world rows at (110.9-113.1, 13.5,
   −178.8..−180.9) — death **t26803** (was t22649). The route's pre-fight phase is ~4700
   ticks longer this run (the wobbled transits), so part of the death-tick gain is pace; the
   ATTRIBUTABLE lever effect is the #16 saving (28.0 → 8.0) and the mid-game health 44.0.

## How it was verified

- Each lever judged on its own real run of the d3 00-Agent walk (NF_DMG_TRACE; 18.1 s
  baseline / 7.9 s / 11.4 s / 18.3 s) [L the four runs above].
- `walkreal_07000008_live` whole binary on the final tree (merged with origin/master —
  master is already an ancestor; no new commits landed during the window): **6 passed /
  0 failed / 1 ignored** in 151.76 s — the d1/d2 real-damage sisters, the plain walks and
  both ally tests stay green.
- The final tree's only code delta vs the merged 08-8 state is
  `crates/nf-view/tests/walkreal_07000008_live.rs` (the d3-gated level_fights gate + dead-lever
  comments + the ignore-message ledger). No `crates/*/src/`, no `crates/nf-view/tests/common/`
  change survives (the stand_pull walk was reverted), so no other test binary compiles the
  changed file — the affected-binary set is `walkreal_07000008_live` alone, and it ran whole.
- The full suite (tools/test/suite.py) did NOT fit the window, for environment reasons: the
  runner's build step builds every one of the ~200 test binaries before `--only` can filter
  the run phase; the background attempt (started 07:31 UTC, first minutes) was killed by the
  sandbox between tool calls and left an empty `suite.out` (the same failure as round 8);
  the remaining window cannot cover the -j4 build. What ran instead, on the final tree:
  `nf-game --lib` **796 passed / 0 failed / 4 ignored**; `nf-formats` **98 passed / 0 failed**
  across its binaries EXCEPT the allowed slim-ISO trio (disc_golden: 11 passed / 3 failed —
  `bigf_header_fields_behave_as_documented`, `bigf_tables_match_the_golden_manifest`,
  `directory_tree_matches_the_golden_manifest` — the documented slim-ISO exceptions);
  `nf-audio` **35 passed / 0 failed** (lib 29, bank_live 1, fix_sound_remainder_live 4,
  music_live 1).

## What's left (the next window's levers, in priority order)

1. **The pre-fight walk-down zaps** — 12 × −2.00 on the upper-floor band at (111.7, −179.2)
   t25666-25793, all BEFORE the crouch drop: the walk stands on the zap band during the
   standoff/walk-down and the +24.0 mid-game gain is spent there. Shorten the standoff→drop
   path (or drop from a spot out of the band) and the fight entry returns to the earned 44.0
   — with a 44.0 entry the round-7 dance has the rounds to reach the low-band launch windows
   (the round-7 finding: the low band arrives at fight t~1400-2100).
2. **The fight's own world rows** — −2.33/−13.22/−4.45 in ~600 fight ticks at
   (110.9-113.1, 13.5, −178.8..−180.9): judge the round-7 rim answer (the dodge/walk-back
   package) from the pocket with a fuller entry.
3. **The trio 36.0** — every deck-leg lever has now measured dead (the crouch sweep, the
   pull west); before spending another one, a live check for the STATION HALL MOUTH's
   (139.0, 49.5, −93.0) sight lines into the trio's placements (the mouth is ABOVE the deck,
   dy 3.4 — a player fights the deck from the hall, not from the lanes).

## Live checks for the main agent

- The station hall mouth (139.0, 49.5, −93.0) standing eye: which of the trio's placements
  (#2 (161.6, 46.0, −97.4), #14 (152.4, 46.0, −96.2), #11 (154.3, 46.0, −87.6)) can Bond
  hit from the mouth (the hall floor is y 49.5, the deck y 46.0-46.1 — the height may clear
  the deck lip the pin wedges on)? If any lane reads clear from the mouth, the deck transit
  can pre-fight the trio from the hall side and the 36.0 never lands.
- The terminal's upper-floor hurt-volume band: the walk-down stood at (111.7, 13.5, −179.2)
  and took 12 × −2.00 (t25666-25793) BEFORE the crouch drop, while the round-4/5 ledgers
  measured the lower floor (feet 13.086) zap-free — confirm the band's exact footprint at
  standing height (feet 13.47-13.52) between the gondola-mouth walk-down and the hole at
  x ~111.99, so the next walk can route the standoff→drop path around it.

## Usage

not visible to me

## ntfy failed

All three fallbacks failed from this sandbox (ntfy.sh unreachable — the same failure as
rounds 5-8; each fallback ran its full attempt at 2026-11-20 ~08:10-08:15 UTC):

1. curl (twice, the second with the corrected result line):
   `curl: (28) Connection timed out after 20002 milliseconds`
2. python3 urllib: the request hangs — killed by the 25 s outer timeout (`exit 124`; no
   response within the 20 s urlopen timeout), no traceback, nothing written. (Two earlier
   attempts piped the process through `tail`, which masked the true exit status — the clean
   unmasked run above is the evidence.)
3. wget: `Connecting to ntfy.sh (ntfy.sh)|159.203.148.75|:443...` — connection hung, true
   exit code 124 (killed mid-connect by the outer timeout; the same signature as round 8).

The notification text that could not be delivered:
`retune-nb-08-9: shipped the pit level_fights lever, death t26803 was t22649 (4154 ticks
deeper), whole binary 6/0/1`

