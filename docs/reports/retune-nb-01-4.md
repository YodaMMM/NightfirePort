# retune-nb-01-4 — 01 spawn-to-exit-with-the-contact real-damage walks, round 4

Branch `s/retune-nb-01-4` (from origin/master 14126d9e5 with s/retune-nb-01-3 2cdd30e3f merged in —
01-3 was not on master; one conflict in walkreal_07000001_live.rs). **The 00 Agent real-damage walk
is green again, and the Operative (d1) and Agent (d2) sisters are green with it.** No lever needed:
the walk's red state round 3 left behind was already fixed on master by walks-1121 (the stream-batch-1121b
rework, master's un-ignore note dated 2026-11-21), and the round's work was making the merge reproduce
that green tree.

## What happened on the merge

1. The merge conflict in `walk_from_the_spawn_to_the_exit_with_the_contact_real_damage_00agent` was
   master's UN-IGNORED state (walks-1121: the hall phase passes, the ch-0xF ambush is fought from the
   pocket lane, the walk completes at 8.4 health, t63360 [L master's own runs, the walks-1121 note])
   vs 01-3's round-3 `#[ignore]` ledger. First resolution kept master's un-ignored attribute and kept
   the ledger as a history comment — but left the walk BODY as git auto-merged it: master's reworked
   route with 01-3's round-3 lever machinery grafted on (plate_pocket_fight, flank_point_x_min,
   fight_hold_west_of, no_fetch_zone_x_min call sites, the d3_aperture_fight / advance-dance grafts —
   ~550 added lines vs master).
2. **That hybrid was measured on its own real run and is red** [L the merged-tree whole run,
   walkreal_07000001_live 5 passed / 3 failed, 159 s]:
   - **d3 (00 Agent)** died at tick 4844 at (15.47,5.99,0.59) — drone #16 (0x500003a, weapon 0x12 SMG)
     at d 2.3 u for 46.6 in 23 hits (the per-hit 3.6/1.8 pattern is the TooClose doubling [M
     Drone_ModBulletDamage 0x139f50]), #6 for 28.8, #0 for 12.0 — 39 hits for 100.0 total, 4 return-fire
     fights. Death in the meet-walk phase, far before master's ambush phase.
   - **d2 (Agent)** wedged in the plate nav: "the plate: gave up at the 3000-tick cap", target
     (-33.11,2.30,15.43) at d 20.24, grounded=false — the plate assert fired at t77801 with the contact
     still alive (2.5 health).
   - **d1 (Operative)**: the escort resupply crawled 24 nav legs across the map ("pre-stairwell ammo
     resupply 0 nav 8..24" ending at (-4.04,-8.05), then "resupply attempt 1 at (-31.0,5.3,15.5) (d 36.4)"
     — a west-pocket drop the 01-2 x-scope lever made fetchable) and gave up at the cap while the contact
     walked his route unescorted: contact #1 corpse (state 0x47, health 0.0), the escort-fail restart,
     then the plate assert at t38326 [L the same run]. That is round 3's documented contact-protection
     race, reached here by the pre-1121 lever pulling the walker away from the escort.
3. **Root cause**: the round-3 levers were measured on the pre-1121 route. Grafted onto walks-1121's
   reworked route they engage in phases that route no longer has, and they were never measured together
   with it. The walk body is master's walks-1121 one again (`git show origin/master:...` — the file's
   delta vs master is comment-only, verified by diff). The round-3 ledger is kept verbatim as history in
   the test doc, marked measured-dead on both streams; the walker.rs opt-in fields from 01-3 stay
   (default None, unused by the 01 walk now, byte-identical by construction for every other walk).

## Verification (all on the final tree, real input, no top-ups, no guard-code changes)

- `walkreal_07000001_live` **whole**: **8 passed / 0 failed / 0 ignored** (183.5 s) — the 00 Agent
  real-damage walk, the Agent real-damage walk and the Operative real-damage walk all pass on the same
  tree [L this round's runs].
- `interactables_07000001_live` **whole**: **10 passed / 0 failed** (25.2 s).
- `walkreal_07000002_live` **whole** (the rule-7 spot check for the walker.rs merge delta):
  **5 passed / 0 failed** (117.7 s). The delta vs master in `crates/nf-view/tests/common/walker.rs` is
  three opt-in fields (`flank_point_x_min`, `fight_hold_west_of`, `no_fetch_zone_x_min`), default None,
  plus their gated branches (`is_some_and(...)` false on None) — the other walks' streams are
  byte-identical by construction; the 02 run is the empirical check.

## The fast suite

**Could not build on this sandbox** (the same limitation round 3 reported, now measured): the workspace
has 1159 nf-view test files plus 227 in the other three crates — each test target costs 10-25 MB of
build-script artifacts plus its linked binary, and the suite's `cargo test --no-run` over all four
crates had passed 6 GB on this 10 GB disk at the halfway mark and filled the filesystem (the run was
killed at 96-99% full; the target tree was pruned to keep the required binaries buildable). The
required binaries were run whole by hand instead (above). A sandbox with more disk (or a suite run
before the 1159-target growth) is needed for the total line.

## What is left

- Nothing blocking on this level's walks: d1/d2/d3 are green on the merged tree.
- The d3 lane fight remains "a narrow pass, not a margin" (master's own sweep note in run_slab_ambush)
  — the next stream shift will re-open it.
- The full `tools/test/suite.py` total line on a bigger-disk sandbox, plus the wider walkreal sweep
  beyond the 01 + 02 whole runs.

## Usage

not visible to me — this environment shows no token, credit or context-window figures for the
session (no cost or usage numbers were displayed anywhere in the tooling).
