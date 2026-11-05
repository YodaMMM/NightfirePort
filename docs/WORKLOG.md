# WORKLOG

## level-1b-platform-touch
Started 2026-11-05 08:12 UTC. Branch s/level-1b-platform-touch (cut from origin/master 27c191518).
Task: on 0700001b the Drake-fight platforms break at the slightest touch. Find the game's break
rule for them (damage source, threshold, what counts as a hit) in the decompile, compare it to
the port, and fix the difference. Prove with a test that touches a platform by walking on it
(no break) and one that does what breaks it in the game. Cite [M] addresses.
