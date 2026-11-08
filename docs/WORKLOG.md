# WORKLOG

## markers-drone-c

Branch from origin/master. Every `[S]` marker in crates/nf-game/src/drone.rs from line 18000
to the end (17 on master today) is a guess the port made; this round reads the game code
behind each (with addresses) and settles it one of three ways: port the game's behaviour and
drop the marker, prove the port equals the game with [M]/[D]/[L] evidence, or prove the path
unreachable on the shipped disc. Unsettled ones stay, with the exact live read listed for the
main agent. Rand draws keep the game's count; play-altering changes pinned through real input.
Report: docs/reports/markers-drone-c.md.
