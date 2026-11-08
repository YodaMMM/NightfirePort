# 0700000a: the corner crossing is not walk speed (second pass)

`the_corner_crosses_like_the_game` (crates/nf-view/tests/walkreal_0700000a_live.rs ~line 1460, ignored) still fails on master: the crossing takes 836 ticks where the game walks it. An earlier task (level-0a-corner-walk-speed) did not make it pass. Start from that report, find on the real path what holds Bond at the corner (collision push, step-up, slide along the wall, a snag on a model) by logging Bond's position and contact each tick, compare with the decompile's player move and wall slide, and fix it. Un-ignore the test when it passes.

Reproduce on the real level first through real input, not a teleport; a repeat means earlier tests passed without reaching the real path. Compare with the decompile, fix, and test red on the old code. Run every walkreal_* binary if player movement changes, one level per tool call. Branch from origin/master, push to `s/level-0a-corner-walk-speed-2`, report in docs/reports/level-0a-corner-walk-speed-2.md. No draw changes.

---
claimed-by: sandbox3 at 2026-11-08T13:31Z
