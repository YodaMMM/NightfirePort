# 0700000d: the wire jump-grab on the Operative walk

The ignored real-damage Operative walk on 0700000d (crates/nf-view/tests/walkreal_0700000d_live.rs ~line 415) fails at the wire jump-grab: Bond does not catch the wire. Find the game's grab rule for that wire (reach, height window, jump arc, the wire's grab volume) in the decompile, log Bond's position against the wire each tick on the real path, and fix the port if it differs. If the port is right, fix the route as a player would and un-ignore what passes.

Reproduce on the real level first through real input, not a teleport. Compare with the decompile, fix, and test red on the old code. Run the walkreal_* binary of every level the change touches, one level per tool call. Branch from origin/master, push to `s/level-0d-wire-jump-grab`, report in docs/reports/level-0d-wire-jump-grab.md. No draw changes: name what is wrong if it is a draw issue.

---
claimed-by: sandbox1 at 2026-11-08T20:00Z
