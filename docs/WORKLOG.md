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
