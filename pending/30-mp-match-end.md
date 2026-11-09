# Multiplayer: how Assassin, King of the Hill and Demolition matches end

Two-instance checks showed Assassin and King of the Hill ending before their time limit because bots reached the score limit, and it is open whether Demolition ends by rounds or by the clock. Read the game's match-end rules for each mode (score limit, time limit, round count; the MP setup records and the end checks) and pin each through a scripted match; fix the port where it differs.

Reproduce on the real level first through real input, not a teleport. Compare with the decompile, fix, and test red on the old code. Run the walkreal_* binary of every level the change touches, one level per tool call. Branch from origin/master, push to `s/mp-match-end`, report in docs/reports/mp-match-end.md. No draw changes: name what is wrong if it is a draw issue.
