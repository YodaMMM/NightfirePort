# los-mask-repins-16 — report

Branch `s/los-mask-repins-16`, base `origin/master` 0ee8478fd (the guard-sight turn-on + the
Chain Reaction walks green). Task: the 07000016 walks green again under the game's guard sight.

## Done

**`the_chasers_punch_lands_the_record_damage` (the task-named ignore) is un-ignored and green**
— binary whole 15/0/3i, three consecutive runs byte-identical.

The ignore's death, traced end to end (NF_WALK_TRACE + NF_RF_TRACE + NF_DMG_TRACE on the
committed base): the corridor fights' own flank answer (the shoot loop's 540-tick flank)
navigated Bond 9 u east into the east strip pocket — "no exposed part for 540 ticks — flanking
to (20.5,0.8,−95.5) (side 1, offset 3.5)" — the determined pass killed #1 there, and the resumed
straight steer could not cross back west over the trench's east edge: Bond pinned at
(19.03, 0.92, −103.36) for ~5000 ticks (measured bounce −100.2 ↔ −103.36, grounded/airborne),
dead at tick 10864 under the ramp-foot rifleman **#16 (skin 0x500008c, weapon 0x1a, hp 10,
(14.5, 0.8, −92.4)) pouring 28.6 in 20 hits from 11.8 u** — the run's biggest dealer, never
once visible from the pin, so the return-fire pick ("shooters [16] none visible", 317 lines)
could not answer. 69 hits for 100.0.

The fix is player-shaped and keyed to the re-arm (real-damage) walks only — the tops-up walks
keep the plain route byte-for-byte:

1. **The east-lane leg rides the NAV web** (`walk_nav_to` instead of the straight steer): the
   flank nav itself demonstrably crossed lane ↔ strip over 4 nav nodes (17.15,−92.35) →
   (21.04,−94.43) [L the same trace], so the leg RE-ROUTES from wherever the fights leave Bond
   instead of wedging; the ring legs already ride the web for the same reason.
2. **A bounded stand for the ramp-foot rifleman** at the lane head (pick: team 1, y < 2.0,
   x > 12, z −97..−88, alive; `shoot_bounded` 1800) — a player kills the hp-10 rifleman dead
   ahead before walking the lane.
3. `w.bridge_wedged_nav = true` (the 01 walk's existing opt-in) — a wedged nav leg re-routes
   over the clear nav lines.

No `common/` file is touched by this task (the whole change is in
`crates/nf-view/tests/walkreal_07000016_live.rs`; a `flank_hold_x`-style common flag was
considered and not needed). No damage/health lowering, no top-ups, no teleports after the load;
the walker's own return fire / finish-fights / level-fights economy is unchanged from the
pre-ignore walk.

**The defeat sisters got honest fight-loop fixes (not yet green):** the squad phase's bare
10800-tick fallback `shoot` (no threat reaction — the measured death ran inside exactly it) is
now a bounded 1800-tick `shoot_bounded`; the member-kill slices are 300 ticks (36, the same
budget) with `hold_the_band` re-checked between; a dealer that hurt Bond in the last 240 frames
and is exposed/visible gets its own bounded stand before the member (the measured 14785 death:
pit-room guard #15 poured 45.8 in 26 hits through the whole ladder while the hold only faces
the nearest INSIDE-3.0 hostile); the d1 walk sets the existing `hurting_first` opt-in (a cooled
failed-fight guard is answered when he is the real dealer). These did not yet turn the three
defeat walks green — the ignores stay, with the current deaths below.

## How it was verified

- `walkreal_07000016_live` whole: **15 passed, 0 failed, 3 ignored** (the punch walk among the
  passed; the three remaining ignores are the defeat sisters, below). Release, `--test-threads=1`,
  `NF_ISO` the slim ISO.
- `interactables_07000016_live` whole: **13 passed, 0 failed**.
- The punch run's evidence lines: `[punch] the punch landed: [20.0] = 50 [D record 99] × 0.4
  [M Operative]`; end health 9.0, armour 0.0 (the window's armour-0 pin held), 55 hits for 91.0;
  her approach stars 0.6 = 1.5 [D 0x12] × 0.4 as pinned.
- Determinism: three consecutive runs produced identical damage ledgers (55 hits for 91.0,
  same per-owner rows).

The full `cargo test` suite did **not** run: `tools/test/suite.py` and every background process
die between tool calls in this sandbox (the build process was killed mid-compile twice before
this was confirmed; the 12-2 entry recorded the same constraint). The per-binary coverage above
is the substitute. No `crates/*/src/` code changed, so no other binary's coverage is owed.

## What's left (the next message continues from the branch)

The three still-ignored defeat walks, each re-measured on THIS tree (post the east-lane fix,
so their ledgers are current):

1. **Operative d1** (`walk_the_chasers_defeat_completes_the_mission_on_real_damage`): dies at
   tick 14833 in the squad phase at (−7.6, −22.3, −172.8), 98 hits for 100.0 — the dealers are
   the pit-room AR guards **#14 (skin 0x50000b1, weapon 0x14, hp 10, d 1.6, 25.8 in 13 hits)**
   and **#15 (skin 0x50000b2, hp 20, 46.8 in 26 hits)**, which close during the far-clone wait
   (#12, hp 100, exposed false for most of the 10800-tick ladder). The squad loop's member kill
   needs the player's advance: walk INTO the pit room toward the member (the clones come anyway
   — 26.9 → 5.5 u during the futile stands) and head-tap each at arrival, with the dealer
   stands between. The pit-room armour #648 take (−14.41, −22.97, −172.75 [D]) also reads
   marginal — the loot walk arrived 0.6 off and the run ended armour 0.0 (the box rests ~1 u
   below the walk plane; the take radius is under a unit [M Pickup_Handler 0x18e650]) — worth a
   live check or a tighter arrival.
2. **Agent d2**: dies at tick 2176 on the ring north (−0.2, 8.0, −79.4), 38 hits for 100.0 —
   the ramp-foot fights killed #4/#0/#1/#2 but cost 97.9 → 18.1, and the y 6.7 gallery pair
   **#5 (skin 0x50000b2, weapon 0x1a, hp 20, z −91.4, 43.3 in 11 hits)** and **#3 (skin
   0x500008c, z −110.5, 31.5 in 15 hits)** plinked the rest free while `level_fights` (±2.0)
   kept the pick off them. The gallery-rifleman stand ("the ring 1") is keyed
   `re_arm && !station_armour` — the defeat walks (station_armour=true) skip it; extending the
   stand to them and/or clearing #5 from the ring west legs (he sat 15.2 u out, just past
   `clear_hostiles_16`'s 15.0 gate) are the next levers.
3. **00 Agent d3**: dies at tick 1020 at the ramp foot (12.2, 0.9, −89.0), 21 hits for 100.0 —
   the same gallery pair at Hard accuracy (7.0 a hit: #2 (9.5, 6.7, −110.6) 30.0 in 5 hits,
   #5 28.0 in 4) before the climb. Same levers; the fights' flank/exposure work at d3 rides on
   the d2 fix.

"Live checks for the main agent": none this round — the trace evidence above is all harness-side
and the route/fight fixes are player-shaped without needing a PCSX2 poke. (A fresh-boot watch of
the pit-room armour #648's take radius would settle item 1's marginal take if the next round
wants it: watch the pickup's gone flag while Bond stands at the spot.)

## Usage

The environment did not expose any token, credit or context-window figures for this session —
not visible to me. (No usage numbers were shown by the sandbox, the CLI, or the git host; the
bash `time` figures for the test runs are in the transcript, not usage figures.)

## ntfy failed

All three fallbacks failed (the sandbox's outbound egress cannot reach ntfy.sh — the
level-05-basin-floor-6 report recorded the same wall):

1. curl: `curl: (28) Connection timed out after 20002 milliseconds`
2. python3 urllib: `urllib.error.URLError: <urlopen error [Errno 101] Network is unreachable>`
3. wget: DNS resolved (159.203.148.75) then the TCP connect hung — `wget exit=124` at the
   30 s timeout, no response bytes.

