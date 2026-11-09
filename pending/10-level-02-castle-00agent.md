# 07000002: the 00 Agent castle walk dies at tick 683

walk_the_castle_all_objectives_real_health_00agent (walkreal_07000002_live, ignored) dies at tick 683 before geisha 1 is saved (ch 0x49 [M 0x1441c0]). Find who kills Bond that early (damage log), compare those guards' accuracy, damage and engage rules on 00 Agent with the game, and fix the port if it differs; otherwise re-route as a player would and un-ignore when it passes.

Reproduce on the real level first through real input, not a teleport. Compare with the decompile, fix, and test red on the old code. Run the walkreal_* binary of every level the change touches, one level per tool call. Branch from origin/master, push to `s/level-02-castle-00agent`, report in docs/reports/level-02-castle-00agent.md. No draw changes: name what is wrong if it is a draw issue.
