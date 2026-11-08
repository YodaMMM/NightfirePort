# 0700001b: the Agent real-health walk to Drake's defeat never finishes

walk_arrival_to_drakes_defeat_on_real_health_agent in walkreal_0700001b_live runs past 590 s even alone and has never produced a result. Find where it loops (log the walk's leg and Bond's state per 600 ticks), fix the port if a game rule is stuck (a console, a wave that never thins, Drake's state), or re-route the walk as a player would; make it finish under 300 s or report why it cannot.

Reproduce on the real level first through real input, not a teleport. Compare with the decompile, fix, and test red on the old code. Run the walkreal_* binary of every level the change touches, one level per tool call. Branch from origin/master, push to `s/level-1b-drake-walk-time`, report in docs/reports/level-1b-drake-walk-time.md. No draw changes: name what is wrong if it is a draw issue.

---
claimed-by: sandbox5 at 2026-11-08T22:49Z

---
## Report (done, 2026-11-08T23:16Z)

