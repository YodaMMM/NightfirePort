## level-16-security-station-use (started 11:18 UTC 2026-10-29)

Task: Countdown Part 3 (07000016) — make the security station work when the player uses it
on real input. The existing test dynobj_glass_live::the_16_screens_swap_with_their_channels
only SETS channel 0x24 directly. Reproduce on step_bond real input (walk, face, press Use —
no pokes, no teleports): find the station's placements, what raises 0x24 [D SecurityStation
touch/interactable, mask, out channel; M Player_Activate 0x1a5170, Trigger_Activate
0x1b2318], what the use triggers downstream, port every missing link, and assert end to end
from a fresh load. List what must DRAW for the main agent. Full suite, quote its total line.

Plan: branch pushed from origin/master (1b2309ea) now; downloads (work-slim, ISO) next,
then build + suite runner in the background while reading the decompile for the station's
interactable path.
