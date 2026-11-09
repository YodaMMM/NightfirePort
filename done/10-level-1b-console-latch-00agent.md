# 0700001b: console 1 on the 00 Agent walk

walk_arrival_to_drakes_defeat_on_real_health_00agent (walkreal_0700001b_live, ignored) fails 'console 1 latched by the real shot'. Find why the shot does not latch console 1 on 00 Agent (aim, range, the console's hit test, difficulty-gated scripts [D the console UI scripts]) and fix the port or the route.

Reproduce on the real level first through real input, not a teleport. Compare with the decompile, fix, and test red on the old code. Run the walkreal_* binary of every level the change touches, one level per tool call. Branch from origin/master, push to `s/level-1b-console-latch-00agent`, report in docs/reports/level-1b-console-latch-00agent.md. No draw changes: name what is wrong if it is a draw issue.

---
claimed-by: sandbox4 at 2026-11-09T02:23Z

---
## Report (done, 2026-11-09T03:05Z)

