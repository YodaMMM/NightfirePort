# Guards parking in door arcs

On 07000008 guards #7/#8 park in the guard-room door's swing (state 0x63); the real game's door also bounces off a guard standing there (PCSX2 check), but it is not known whether the game's guards ever stop in that spot. Trace why the port's guard ends in state 0x63 at the door (his route, the door node pair, the stand rule) and compare with the game's route and state logic; fix the port if the game would not park him there.

Reproduce on the real level first through real input, not a teleport. Compare with the decompile, fix, and test red on the old code. Run the walkreal_* binary of every level the change touches, one level per tool call. Branch from origin/master, push to `s/guard-door-park`, report in docs/reports/guard-door-park.md. No draw changes: name what is wrong if it is a draw issue.

---
claimed-by: sandbox1 at 2026-11-09T11:29Z
