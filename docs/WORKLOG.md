# WORKLOG

## regression-read-weapons (branch s/regression-read-weapons)

Started 2026-11-05 ~18:07 UTC. Task: read every commit since 2026-10-20 touching
crates/nf-game/src/weapon*.rs crates/nf-game/src/bullet*.rs crates/nf-game/src/grenade.rs;
for each, diff -> what a player would see -> check vs game [M]; prove suspects through the
real App path (HeadlessApp keys, 1/240 and 1/60); fix what is proven with a failing-first
test. Not casings, driving or gamepad. Name breaking commits and their first tag. Report
every commit range read and what was clean.

Progress log:
- Sandbox reused from previous task (repo, work/, ISO, nf-queue all present).
- Branch s/regression-read-weapons created from origin/master (2f0c88e2c) and pushed.
- Commit census in progress.
