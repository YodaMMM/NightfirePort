# 0700000d: the Agent real-damage walk dies at the ladder top

The ignored Agent real-damage walk on 0700000d dies at tick 4275 at the ladder top; the no-wait #27 shot does not land on the guard that kills Bond (see docs/reports/batch-1109a-walks.md on s/batch-1109a-walks). Identify that guard, compare his sight/engage/accuracy on Agent with the game's, fix the port if it differs, otherwise take the ladder top as a player would (clear him from below first). Base on origin/s/batch-1109a-walks, push to s/level-0d-agent-ladder-top.

Reproduce on the real level first through real input, not a teleport. Compare with the decompile, fix, and test red on the old code. Run the walkreal_* binary of every level the change touches, one level per tool call. Branch from origin/master, push to `s/level-0d-agent-ladder-top`, report in docs/reports/level-0d-agent-ladder-top.md. No draw changes: name what is wrong if it is a draw issue.

---
claimed-by: sandbox3 at 2026-11-09T09:09Z
