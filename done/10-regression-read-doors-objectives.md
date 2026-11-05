# regression-read-doors-objectives — Doors, switches and objectives: read the recent commits for player-visible breaks

Read every commit since 2026-10-20 touching: crates/nf-game/src/triggers.rs crates/nf-game/src/mission*.rs crates/nf-game/src/nis.rs. For each, read the diff and ask what a player would see change. Check the claim against the game [M], then prove any suspect by running it through the real App path (HeadlessApp keys, at 1/240 as well as 1/60), not by reading alone. Name the commit that broke each thing and its first tag. Fix what you prove, with a test that fails first. Not casings, driving or gamepad. Report every commit range read and what was clean.

---
claimed-by: sandbox1 at 2026-11-05T15:07Z

---
## Report (done, 2026-11-05T16:06Z)

