# 07000001: the 00 Agent walk with guards breaking the glass

The ignored 07000001 00 Agent (d3) real-damage walk is still red after several rounds (guard-pane-drain series; read the latest report on s/guard-pane-drain-6 if present). On current master (guards turn off Bond on contact; guard rounds break glass), find where Bond dies, compare those guards' rules with the game, fix the port if it differs, otherwise re-route as a player would and un-ignore when it passes.

Reproduce on the real level first through real input, not a teleport. Compare with the decompile, fix, and test red on the old code. Run the walkreal_* binary of every level the change touches, one level per tool call. Branch from origin/master, push to `s/level-01-d3-walk`, report in docs/reports/level-01-d3-walk.md. No draw changes: name what is wrong if it is a draw issue.
