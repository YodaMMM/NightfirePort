# regression-read-weapons — Weapons and damage: read the recent commits for player-visible breaks

Read every commit since 2026-10-20 touching: crates/nf-game/src/weapon*.rs crates/nf-game/src/bullet*.rs crates/nf-game/src/grenade.rs. For each, read the diff and ask what a player would see change. Check the claim against the game [M], then prove any suspect by running it through the real App path (HeadlessApp keys, at 1/240 as well as 1/60), not by reading alone. Name the commit that broke each thing and its first tag. Fix what you prove, with a test that fails first. Not casings, driving or gamepad. Report every commit range read and what was clean.

---
claimed-by: sandbox6 at 2026-11-05T17:35Z
