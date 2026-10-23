## Your task: `laser-watch-0c`, the laser watch on 0700000c must work like the PS2 (read the decompile, no shortcuts)

User playtest 2026-10-22 (check qworm0cm): "lasers arent working like they should, took an f12. compare to the ps2 version. look at the decompile and dont take shortcuts". F12: dist/nf-shots/015-0700000c.png (`--bond 7.61,15.71,-113.39,5.783,-0.363`, frame 1876): Bond firing the laser watch at the bolted hinge of a grated gate.

1. Identify the target at that spot [D placements]: a laser-burnable lock or hinge (the Lock / class-0x22 / Destroy records)? What does burning it do in the game (channel, door) [M]? Compare with 831 laser-watch-contact and the 07000008 bolt (ch 134; 84 vs 88 frames, verified against PCSX2).
2. Real-path test: equip the watch, hold fire on the hinge from the F12 spot, and the game's outcome happens in the game's time. If the port already does that, say so with the trace, and list what else on 0c the laser should affect.
3. If a live PCSX2 number is needed (the burn time, the channel), say exactly what the main agent should read.
