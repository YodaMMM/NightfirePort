# 0700000d: a ~10 m drop kills Bond outright

The ignored walk `walk_arrival_to_the_c5_door_real_damage_agent` (crates/nf-view/tests/walkreal_0700000d_live.rs, ~line 836) takes a ~10 m fall that the game survives [M 0x1949a8], and the port takes Bond from 26.8 to 0. Read the game's fall-damage rule (fall height or landing speed, scale, cap, difficulty factor) at 0x1949a8 and its callers, compare with the port's landing damage, and fix it. Pin the damage for that drop and for one short and one lethal drop on another level, through real input.

Reproduce on the real level first through real input, not a teleport. Compare with the decompile, fix, and test red on the old code. Run the walkreal_* binaries for every level whose walks fall (grep tests for falls), one level per tool call. Branch from origin/master, push to `s/level-0d-fall-damage`, report in docs/reports/level-0d-fall-damage.md. No draw changes.

---
claimed-by: sandbox1 at 2026-11-08T13:53Z
