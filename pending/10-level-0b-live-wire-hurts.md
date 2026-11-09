# 0700000b: live wires in the dome may hurt more times than the game allows

A walk through the 0b dome took 74.8 damage from 39 live-wire hurts. One fusebox has at most 10 group-1 wires, and a dead wire's slot is cleared, so the port may repeat a hurt the game cannot (see docs/reports/los-mask-join.md on s/los-mask-join, the body armour detour). Read the wire hurt rule (the wire/fusebox update, the per-wire hurt timer or once-only flag, when a wire dies) in the decompile and log each hurt (wire id, tick, damage) on the real path through the dome; fix the port if a wire hurts more often than the game's rule lets it, with a test red on the old code.

Reproduce on the real level first through real input, not a teleport. Branch from origin/master, push to `s/level-0b-live-wire-hurts`, report in docs/reports/level-0b-live-wire-hurts.md. Run walkreal_0700000b. No draw changes.
