# 0700000d: the Agent real-damage walk dies at tick 3310

The ignored Agent real-damage walk on 0700000d dies at tick 3310, before the wire. Find the guards and rounds that kill Bond there (damage log), compare their accuracy, engage range and damage with the game's rules for those guards, and fix the port if it differs; otherwise re-route as a player would and un-ignore it if it passes.

Reproduce on the real level first through real input, not a teleport. Compare with the decompile, fix, and test red on the old code. Run the walkreal_* binary of every level the change touches, one level per tool call. Branch from origin/master, push to `s/level-0d-agent-real-damage`, report in docs/reports/level-0d-agent-real-damage.md. No draw changes: name what is wrong if it is a draw issue.
