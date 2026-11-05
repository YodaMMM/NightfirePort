# WORKLOG

## regression-read-drone-ai (branch s/regression-read-drone-ai)

Started 2026-11-05 ~16:55 UTC. Task: read every commit since 2026-10-20 touching
crates/nf-game/src/drone.rs that changes AI state handling, sight, hearing, cover, or
route following; for each, diff -> what a player would see -> check vs game [M]; prove
suspects through the real App path (HeadlessApp keys, 1/240 and 1/60); fix what is
proven with a failing-first test. Not casings, driving or gamepad. Name breaking
commits and their first tag. Report every commit range read and what was clean.

Progress log:
- Sandbox was reset; full setup redone (clone, work-slim, ISO, rust, cargo fetch).
- Branch s/regression-read-drone-ai created from origin/master (d3865df75) and pushed.
- Commit census in progress.
- Census: 332 non-merge commits since 2026-10-20 touching drone.rs, diff-triaged; ~50
  AI-machinery commits audited vs work/decomp by 4 parallel read-only batches + hand
  re-verification (G-055 swaps, state gates, sight pipeline, stats/spawn, accuracy,
  hearing, route/door trio — verdicts in the report).
- Proven + fixed (fail-first pins red at HEAD, green after): search-area/scripted sound
  attacks missing the game's sighted+dist gates [M 0x165270/0x165bf0 case 0x14,
  0x177800:43-50], the deaf-script window banking alertness [M 0x148a30:43-44 +
  0x15ba68:73/:84], the kick bind leaving the +0xBA8 door slot standing [M 0x153358:129].
- nf-game lib 680/0/5. Left: HeadlessApp 1/240+1/60 proof, the live binaries, the
  spawn-RNG starvation + pick modulus (global stream effect, needs its own task),
  enemy_alerts msg-aware sighted gate, full 4-package suite. See
  docs/reports/regression-read-drone-ai.md.
