# guard-slide-03-01-repairs — the 03 wedge diagnosed (the level's own courtyard pocket) and the lane-exit re-take landed but not yet holding; the 01 Agent repair not started

## What is done

**Setup.** Branch `s/guard-slide-03-01-repairs` from `origin/s/guard-slide-gate-sweep`,
`origin/master` merged on top (fast-forward). Repo, `work/` and the slim ISO (the archive
exactly 369823203 bytes [L the stat]), the queue worktree. Rust stable 1.99.1 installed; the
ALSA user prefix rebuilt from the bookworm `libasound2 1.2.8-1+b1` debs — with one extra turn
vs the base window's note: the deb's `alsa.pc` hardcodes `libdir=/usr/lib/x86_64-linux-gnu`,
so `pkg-config` pointed the link at the system dir where no dev symlink exists; the `.pc` in
the prefix was patched to the prefix's libdir and `alsa-sys`'s build-script output force-
refreshed (rm of `target/release/build/alsa-sys-*`), after which `nf-audio` links clean.

**Item 1 — the 07000003 wedge, diagnosed with the ledger's own first lever.** A new
`#[ignore]` probe in `walkreal_07000003_live.rs` (`probe_plate_wedge_spot`) runs the d1 route's
legs verbatim into the servant loop and dumps the wedge spot (6.35, 6.3–7.2, −15.7..−17.7):

- **No slid guard.** No drone within 5 u of the spot at any phase (the nearest, #7, stands on
  the upper floor at (11.61, 10.09, −15.77); #16 sits disabled at its load spot 10 u away)
  [L the probe's A/B0/B1/B2 dumps]. The ledger's lever-2 premise (a slid guard in the nav
  lane) is false.
- **The spot is the level's own furniture pocket.** From inside it the 16-ray bullet sweep
  stops at 0.05–0.21 u on every side but south (az3.14–4.71, −z, clear t1.00) and 0.18 up
  [L the probe's C sweep]; Bond parks there grounded FALSE at y 7.02–7.21 AND grounded TRUE at
  y 6.41–6.61 (both phases measured across the runs). The A* legs ride path3's upper-row
  nodes (10.80/11.28, 9.64) through the band — the waypoints audit (lever 3).
- **The entry leg is the door-8 mouth walk's A* from the west loot end** (3.69, −28.84): 21
  give-ups in the probe, then every later nav gives up from the pocket (29 on the plate leg)
  [L the probe's walk echoes; matches the sweep run's ledger].

**The lane-exit re-take landed, measured twice, not yet holding.** `off_the_courtyard_obstacle_03`
(test-local, gated to `difficulty == 1`, self-gating on the parked state — the d2/d3 streams
are untouched by construction) fires at the four servant-loop points (after the plate walk,
after the ground loot, after the door-8 mouth stand, after each servant walk):

1. **v1** (east/west sidestep episodes off the block, then the lane anchor — the 08 pattern):
   Bond left the block top (grounded at (4.34, 6.42)) but the lane-anchor walk re-funnelled
   him into the pocket (STUCK at (6.35, 6.43) with `segment_to_target` 0.13 — the wall is
   1.1 u along every ray) [L d1ignored.out].
2. **v2** (south-extract first — the sweep's clear side — then the lane anchor): the re-take
   fired at all four points but Bond was back in the pocket for the next leg every time
   [L d1v2.out: the parked lines at (6.35, 6.41..6.61, −15.72)]. The test failed at the
   servant-0 state assert with 0x50 still off — the same signature as the base ledger.

The d1 test **stays ignored** with the updated ledger: the diagnosis, both measured attempts,
and the next three levers (the ground-level exit map — the sweep ran from y 8.5; the
extraction-anchor shape — walk_nav_to's own nodes or the west-lawn anchors instead of the
direct lane walk_to; the east-lane start gate — after the loot walk, if Bond is west of x 9.5,
walk the first visit's moat-band ladder so the A* re-derives from the lane the way the green
runs did).

**Item 2 — the 07000001 Agent repair continuation: not started.** The window went to the 03
diagnosis/fix rounds (the probe build+runs and the two fix attempts). The four-lever ledger on
the Agent test's ignore (the base branch's revert note) stands unchanged; the mayhew per-leg
re-time, the 900-tick step-out and the NF_RF_TRACE of the 13556 leg are the next window's work.

## How it was verified

- `walkreal_07000003_live` whole, `--test-threads=1`, on the final tree: **5 passed / 0 failed
  / 4 ignored** in 197.7 s [L bin03.out] — the d2 real-damage and d3 agent sisters green with
  the re-take helper in the tree (their `difficulty == 1` gate never opens), the ledgered d1 +
  the 00-agent ledger + the two probes ignored.
- Every number above tagged [L] is quoted from a run output file on this tree (probe.out,
  d1ignored.out, d1v2.out, bin03.out); the decompile-backed pins the base ledger carries
  ([M 0x15cb78:24-36] the empty-link save rule) are untouched — no test weakened, no assertion
  removed: the ignore message grew its evidence.
- Real input only: the probe and the re-take drive the walker's own input paths (walk_to,
  walk_nav_to, shoot); no teleports after the load, no health/damage writes, no channel pokes.

## What is left

1. The d1 pocket extraction (the three levers in the test's ignore message).
2. The whole `walkreal_07000001_live` + `walkreal_07000003_live` interactables binaries and
   the 07000001 Agent repair (item 2 of "What is left", the ledger on the Agent test).
3. The full suite with the fast runner on the final tree in a sandbox where it fits (the
   runner built ~25 min here but the cargo lock fought the probe rounds; the per-binary
   chunks are the substitute, as in the base window).

## Usage

not visible to me (no token, context-window or cost figures are exposed in this session).
