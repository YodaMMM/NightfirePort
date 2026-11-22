# retune-nb-08-16 — the 08 whole-mission 00 Agent real-damage walk, round 16

## What's done

- Branch `s/retune-nb-08-16` from `origin/master` (d57c3fdac) with `s/retune-nb-08-15`
  (d1373ac73) merged in — its only content over master is the 08 line's own test-file ledger
  and reports, so no cross-level surface. Fresh sandbox again (the third reset in the task's
  run): the repo, the game files (work-slim verified, the 369823203-byte ISO archive verified
  byte-size), the toolchain (rustc 1.100.0, the extracted libasound2/libasound2-dev debs in
  `~/alsa-prefix`, `cargo fetch`) were all set up this session; the downloads ran at
  ~30-64 MB/s so setup cost ~10 minutes instead of the round-15 hour.
- **The baseline reproduces round 15 byte-exact on the merged tree**: death t29924 (frame
  29926), 63 hits for 100.0, 19 return-fire fights [L the `NF_DMG_TRACE` run, 24.35 s].
  This round went UPSTREAM of the killer chain per the task line, and the baseline's own
  damage ledger reshaped the lever list before anything was coded:
- **The full upstream budget is now measured** [L the NF_DMG_TRACE rows]: the station hall
  **−48.0** (12 × −4.00 rows t738-2102 — the trio #14 4/#11 3/#2 2 + #9 1 + #10 2, all on
  the y-46 deck) + **#16 −8.0** (t5227) + **the machine-fight volume zaps −38.0** (the
  approach burst 9 × −2.00 t25702-25799 standing, the sustained flip 3 × −2.00 t26198-26209,
  and **seven −2.00 rows landing on the CROUCHED capsule at the anchor** t26318-27902) + the
  chain −6.0. Chain entry health 6.0; the ring needs ~10-12.
- **Ledger item (2) the pre-crouch — DEAD BY DATA, no run spent.** The item's premise (the
  flip's 3 standing rows are the sustained window's cost) is false: after the flip the zaps
  KEEP landing on the crouched capsule at the anchor (113.0,13.1,-179.3) — the hurt volume
  reaches the pocket on BOTH stances — so pre-crouching on any arming frame saves ZERO. The
  08-13 plan's zap-free-trench premise holds on the plate walk's transit, not a parked sit.
- **The round-6 note's zap-free park spot (111.99,−179.51) — TRIED and DEAD on its own real
  run.** Implemented as the PARK walk-back (d3-gated, post-ride t>200, band-closed-only
  target swap in the walk-back; the dive prefix byte-exact through the t49 launch; no
  common/-helper change, no crates/*/src change). MEASURED [L the lever run]: the park spot
  is INSIDE the volume's reach too — **12 × −2.00 rows land AT the park** (t26077-29068,
  both stances: (111.9,13.5,-179.5) standing then crouched 13.1) — and the walk died
  **t29795 at (113.04,13.47,-180.84) standing on the south rim** during the chain, 129 ticks
  WORSE, the lever-2b rim-drift death class again. The volume's reach covers the whole west
  pocket band (~1.2 u along the trench at this latitude): **no zap-free park exists near the
  pocket.** Reverted; the shipped round-14 state is the final tree.
- **A new law measured this round**: line_ok NEVER reopens after the dive launch — 0 of 210
  [08 gates] samples true, and the t4100 eye sweep stops 0.02-0.12 u out in all 16 azimuths
  [L the baseline log]. The ONLY machine damage in the whole fight is the t49 dive launch
  (−101.0 of 598): from the anchor the fight cannot be WON, only survived — and the
  exit_ticks walk-out machinery is vestigial (`let _ = exit_ticks`). This reframes the next
  rounds: the chain-entry health can only rise from the HALL (−48.0, route/order/cover) or
  from breaking the volume (a live check), not from any terminal-fight tweak.

## How it was verified

- Every lever judged on its own real run of the d3 00-Agent walk with `NF_DMG_TRACE`: the
  baseline t29924 (24.35 s), the park-lever run t29795 (reverted) [L two runs this window];
  the pre-crouch needed no run — the baseline rows themselves measure it dead. The prefix
  through the dive window stayed byte-exact in every run (the [08 rocket] t49 launch —
  101.0 landed — printed identically each time).
- The whole `walkreal_07000008_live` binary on the reverted final tree: **6 passed / 1
  failed (the target walk), 171.09 s** (`--include-ignored --test-threads=1`) — the d1/d2
  real-damage sisters, the plain walks, the d2 Agent sister and both ally tests byte-exact
  green, matching the round-14/15 final-tree numbers [L the final-tree run]. No
  `crates/*/src/` and no `crates/nf-view/tests/common/` change was made (the lever lived
  entirely in the test file and was reverted), so the affected-binary set is
  `walkreal_07000008_live` alone.
- The full suite (tools/test/suite.py): the background fast runner was started and died
  between tool calls (the same kill rounds 13-15 verified twice), so the suite ran in
  foreground chunks, plus a fresh failure mode this sandbox introduced mid-window:
  **the disk filled (a 9.9 GB rootfs; the target tree alone is ~7 GB) and an unlocked
  `cargo fetch` (run to fetch the missing `ab_glyph`) bumped Cargo.lock and churned the
  fingerprints** — `Cargo.lock` was restored to HEAD and `cargo fetch --locked` re-run, but
  the churn re-invalidated parts of the prebuilt set and the remaining builds flap on
  ENOSPC. Measured result [L the foreground chunk runs, `--test-threads=1`]:
  - **nf-game: 1193 passed, 3 failed, 5 ignored** — the 3 failures are
    `drive_ai_land_disc`'s `/DRIVING/MIS01.VIV` trio (BIGF header / disc-data reads), the
    allowed slim-ISO disc-golden class (the onfoot ISO has no driving disc content).
  - **nf-formats + nf-audio: 258 passed, 2 failed** — the two `carved_dir` tests
    (`carved_folder_reads_the_same_bytes_as_the_image`,
    `carved_index_names_cannot_escape_the_folder`) read carved folders back from the ISO
    image; this branch touches NO crate code (test-file ledger text + docs only), so these
    are environmental/disc-image-dependent failures to flag for the main agent, not
    regressions of this branch.
  - **nf-view: the affected binary ran WHOLE (6/1, above)**; the remaining nf-view
    binaries could not complete under the disk wall (their builds flap on ENOSPC; a
    sampled sibling `walkreal_07000001_live` is minutes-long per test and does not fit the
    window). No `crates/*/src/` and no `crates/nf-view/tests/common/` change survives in
    the final tree (the lever was reverted; the only test-file change is the ledger text
    inside the ignore attribute), so every other nf-view binary is byte-identical in
    behaviour to the merged 08-15 state whose whole-binary numbers are in
    docs/reports/retune-nb-08-15.md.
  - Suite total line: **not obtainable this window** — the wrapper could not finish under
    the ENOSPC wall; the chunks above are the measured portion.

## What's left (the next window's levers, in priority order)

1. **The station hall −48.0** (the largest upstream block, route/order/cover, upstream of
   the machine fight entirely): the 9 trio rows all land at the lane-mouth nav wedge
   (149.5-149.6,46.1,−93.1) and ZERO trio rows ever landed west of x 149.5 [L the ledger] —
   a hall-mouth COVER stand west of that line (probe: can the walker's eye/shoot see the
   trio from behind the mouth cover?) is untried; the 08-2 lane-mouth PRE-EMPT died on the
   nav wedge, a cover STAND is a different shape.
2. The volume's powering: the pocket zap rows cluster in machine-proximity bursts
   (approach 9 rows/100 ticks, sustained 3/11, tail sparse 7/1600) — a live check (gdbwatch
   on the volume's damage tick vs the copter's range) decides whether the volume is
   machine-powered, i.e. whether ANY local park can ever be zap-free.
3. The chain-entry math: entry 6.0, ring un-survivable below ~10-12, the park and the
   pre-crouch both measured dead — the entry rises only via the hall (item 1) or the volume
   (item 2). The trio 36.0 and reply-cadence live checks stay carried.

## Live checks for the main agent

- The volume's powering (new): trace the hurt volume's damage application (the −2.00 world
  rows at (113.0,13.5/13.1,−179.3)) against the copter's distance during f25700-28000 — is
  the zap cadence a function of the machine's proximity, or locally scheduled?
- The reply cadence (carried, rounds 11-15): a gdbwatch on the w62 spawn after the t49 dive
  launch — provoked or scheduled?
- The sustained burst's flip latency (carried, now moot for savings but still the anatomy):
  trace BL+0x960 through f26196-f26215.

## Usage

not visible to me

## ntfy failed

All three fallbacks failed from this sandbox (ntfy.sh unreachable — the same failure as
rounds 5-15; each attempt at 2026-11-21 ~17:20-17:21 UTC):

1. curl: `curl: (28) Connection timed out after 20002 milliseconds` (rc 28)
2. python3 urllib: killed by the outer 35 s timeout (rc 124) — no response within the 20 s
   urlopen timeout, no traceback, nothing written
3. wget: killed by the outer 35 s timeout (rc 124, quiet mode, no output)

The notification text that could not be delivered:
`retune-nb-08-16: upstream round — the ledger pre-crouch dead by data (the crouched tail
zaps at the anchor), the round-6 park spot tried and dead on its own run (12 zaps at the
park, t29795); shipped state kept, death t29924, whole binary 6/1; the station-hall cover
stand next`
