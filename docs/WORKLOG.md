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

Progress (12:35 UTC): setup rebuilt from scratch (sandbox reset — clone, work-slim, ISO,
rustup, ALSA user prefix with the runtime lib + a fixed alsa.pc prefix). The investigation:
the whole use chain is ALREADY wired on master — trigger #597 (0xEA, out 0x24, mask 4) →
Player_Activate 0x1a5170's use sphere → Trigger_Activate 0x1b2318:38-43 → ch 0x24 → the 0x3b
screen pair #682/#683 (create/tick 0x18dbf8/0x18d3b8) + door[0]'s unlock; GAP-16-1's draw
re-sync landed with the 1b glass task. The split-half tests existed (poke-only screens,
door-only walk); added
walkreal_07000016_live::the_station_use_swaps_the_screens_end_to_end — the full chain on
real input from a fresh load: the 16-leg walk, the E, ch 0x24, both screens' sim+palette
swap, door[0] unlock+open. Green first run. Probe note: the screens' world spot is the
station console (−15.16, 7.23, −94.56), not the corridor. Report + render spots in
docs/reports/level-16-security-station-use.md; full suite chunking in progress.
