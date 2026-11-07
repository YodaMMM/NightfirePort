# real-health-00agent-06-4 — the report

Task: the 06 00 Agent walk keeps its sister ignored. Continued from `done/50-real-health-00agent-06-3.md`
(queue), took its levers on current master (22d5836b7), real input only, helpers level-local
(the only files touched: `crates/nf-view/tests/walkreal_07000006_live.rs`, `docs/WORKLOG.md`,
this report).

## Status: the sister stays `#[ignore]`d — with this session's measured reason in the ignore
## text. The walk moved from the strip arrive (06-3's t5879 death) to the east strip (t8893),
## and the courtyard that killed 06-3 is now cleared and measured.

## What's done (all committed and pushed; every number is from a rebuilt-binary run of this branch)

1. **The census probe (06-3's lever 1) is in and answered.** Frame-stamped censuses at every
   phase boundary (drone pos/present/enabled/hp/d/rays + every hit Bond took with its frame),
   plus the four heads' sweep geometry (pos/mid/half/pitch/channel). The pair's timeline
   [L, runs 2-4]: dormant at (22.3, −39.1)/(24.1, −39.6), enabled exactly when ch55 rises
   (mid walk-back), spawn-walk west at ~0.06 u/tick, muzzle rays open only inside d ~13.5
   (r0 at 15.4, r1 by 13.4), pause at d 11.8-12.9 of Bond's stand spot — INSIDE their own
   12 u sight [D props]. No present-false blinking.
2. **The pair engagement is fixed and measured**: the fixed 600-tick landing watch lets them
   walk in and pause, the stands engage at d 12.3-12.5 with both rays open — OUTSIDE their
   sight — and the still-gate taps take both for **16.5 total** (runs 4-10, was 60.4 at the
   post-pause engagement and "never tapped" mid-walk in 06-3). Two variants measured NEGATIVE
   and reverted with the ledgers: a back_off kite (no-op — [M player.rs:1006/1671] the aim
   stance forces forward to 0; and `shoot_bounded` is the walker's own flank loop, which never
   reads `DamageLog::back_off`), and a conditional "first r1 past d 12" watch (broke 84 ticks
   early on a far r1; #20 charged to d 9.4 and unloaded 64.2 in 12, run 6).
3. **The tunnel-roof guard #11 is killable**: a full patience hold at the strip tier (idle
   re-picks until his lean-out opens, each pick a 600-capped stand) killed him in run 9/10
   for **11.0 total** (he dealt 22-47 in every run that lost him; the strip stand-off itself
   is safe — run 4's watches took zero hits at d 19-21 [L]).
4. **06-3's open question is moot on this master**: the deck posts (#12/#29/#30) do not need
   to leave their posts — the balcony stands kill all three directly from the lip (the pick
   census reads them r1 at d 7.8-21.3 [L run 2]; they are absent from every later census).
   No live check needed on the chase-fall behaviour for this tier.
5. **The ch55 cone dodge measured NEGATIVE** (06-3's lever 2 direction): the landing spot
   (4.4, −23.0) is swept by the head on ch63 at (7.13, 47.57, −23.60) ~120-240 ticks after
   the drop wherever Bond stands in the landing band; ch55 rises with or without the mouth
   walk (skipping the mouth only moved the rise 100 ticks earlier). The heads' geometry is
   dumped in the run log for any future lane work.
6. **The east post #24 dies to the ledge tier's patience** (run 8: hp −2; run 10: hp 7) —
   the posts' rays open in lean-outs and the re-pick loop catches them.
7. **The d1/d2 walks are untouched**: the d2 Agent walk reproduces its master ledger
   byte-identically (13031 ticks, end health 3.7, 51 hits for 96.3) — caught and fixed after
   the first whole-binary run (the 600 stand cap and the strip hold were shared-tier and
   shifted the d2's razor-thin timeline; both are d3-gated now).

## What's left (the next session's named levers)

The walk ends at 100.0 taken at the east-strip arrive (9.73, −35.26) t8893. The ledger:
#4's guardroom melee 33.0 in 12 (every run, identical frames — no lever found: the pre-door
`fight_through` displaced the route into a resupply wedge, the kite is a no-op [M player.rs:1006]),
#24's east lane 33.0 in 6 at d 19 (he survives at hp 7 — the ledge tier engages AFTER the legs
that eat his fire), the pair 16.5, #11 11.0, #16 2.8. The gap to green is ~33 hp. Levers:
1. **Engage #24 before the east-strip legs**: his rays open from the east floor side (the d2
   note's (18.56, −37.04) d 9.7 measure) — a stand leg EAST of (14, −36.7) before the
   (10.5, −34.3) nav is the unmeasured spot.
2. **Shave #4's doorway fight** (12 × 2.75): a faster-kill idiom for a melee charger in the
   pre-clear (the fights' `aimed_fights` tight gate was not tried there).
3. The ledge/dressing phases have never been reached at 00 — unknown costs beyond the first
   east-strip leg.

## Verification (this session, on the final pushed tree 57580735a)

| Binary | Result |
| --- | --- |
| `walkreal_07000006_live` (whole, 7 tests) | 6 passed, 0 failed, 1 ignored — the ignore IS the 00 sister, with the measured reason in the ignore text |
| 00 sister iterations | 10 measured runs (baseline + one per lever), each rebuilt first; the censuses/ledgers quoted above are in the session logs (not committed) |

The full suite did not fit the window (same as 06-3): the touched file is test-only (no
`crates/*/src/` change; the branch diff vs origin/master is the 06 test file, WORKLOG, this
report), so the lesson-10 set is the 06 binary — green, every test named above.

## Live checks for the main agent

None required this session — 06-3's open question settled in-sim (the deck posts die to the
balcony stands directly). If the next session wants the east-posts lane: a PCSX2 watch on
#24's position + enable timeline during the east-strip legs would pin whether his d-19 engage
on this branch matches the game's own post lane (the disc census has him at (31, 42, −37);
the port's runs read him at (27.2-28.8, −35.1..−37.2) mid-lane).

## Usage

Not visible to me: this environment exposes no token, cost or context-window figures for the
session.
