# WORKLOG

### level-05-every-objective-3 (start, 23:55 UTC, 2026-10-29)

Task: The Exchange castle (07000005): from the crawl exit to the 07000006 exit — the roofed
corridor south of the crawl exit (x −36, z −38..−44; decode its mouth), the second crack
(z −46..−50, jumped), the south gallery, the SW loop, the SWITCH plate #269, the Q-Claw
grapple over the moat inlet to pulley #281, the east stairs and door #112's E → ch 0x1E →
the 07000006 exit. Fix any port gap the real legs hit [M/D]. The walk must stay green;
un-ignore what turns green. FULL suite; quote its total line.

Base: origin/master (21fbeed3). Note: master's merge dropped docs/reports/ (the
level-05-every-objective-2 report lives on origin/s/level-05-every-objective-2 at e704a4c3 —
read from there; its remainder list is this task's route).

Sandbox was reset (fresh clone 23:49 UTC); setup redone in order: branch+WORKLOG pushed
first, then game files, then rust/ALSA, then the suite runner.

### level-05-every-objective-3 (progress, 00:55 UTC, 2026-10-29)

Route decoded off the disc [D probe_corridor05's fine maps, committed]: the corridor runs SOUTH
from the crawl exit along x −36..−34.25 (36.0→37.8) and the x −35.5..−34.25 spine is CONTINUOUS
through the "second crack" (z −46..−50 — the crack cuts x −36..−34.25 to 18-25, the spine holds,
no jump needed) and the gallery (z −52..−64, the spine 37.4) to the widening at z −65; the SW
loop zigzags x −41..−33 (37s) through z −66..−78; the battlement lane (z −73.5..−72.5,
x −28.5..−16 at 37) carries SWITCH plate #269 (out 0x42); the moat inlet (x −15..−5, floors
20-25) is crossed by the Q-Claw grapple to pulley #281 (−3.40, 42.94, −71.77); the east lane
runs x −4..13 (37) and the covered staircase rises x 14→35.5 (37.1→43.4). The stairs→terrace
link is UNRESOLVED: the y-49 terrace's every face is 5.5+ tall off the stairs' 44.0 top [D MAP
8-12] — parked with a probe-grade door-chain test (E at #112 → ch 0x1E → 07000006) + a live
check. Legs written into walkreal_07000005_live (corridor/spine/loop/#269/grapple/stairs) —
build+run pending (alsa link fix: LIBRARY_PATH).

### level-05-every-objective-4 (start, 03:58 UTC, 2026-10-30)

Task: land the 05 walk extension. Base: origin/s/level-05-every-objective-3 (9dc521bc),
merging origin/master (f82fa6cc) into it. The -3 test
walkreal_07000005_live::the_door_band_exit_chains_to_07000006_climb_parked FAILS on
master's tree: "the terrace placement stands on the y-49 band [D the band maps], got
[21.805223, 50.035336, -68.09826]" (walkreal_07000005_live.rs:1361). Find what Bond
stands on at y 50.04 at (21.8, -68.1) [D the collision], fix the test's spot or the
port with evidence, then continue the remaining legs from the -3 report (the east
stairs, door #112's E -> 07000006). Every test in walkreal_07000005_live green; run
the touched binaries, quote totals.

### level-05-every-objective-4 (done, 04:45 UTC, 2026-10-30)

The -3 door test's two failures were both TEST bugs, fixed with evidence; the file is 10/10
green. (1) The terrace placement asserted the capsule CENTRE against the floor band: the
disc floor at (21.805, −68.098) is 49.001 [D probe_terrace05], the game parks the centre
1.086 over the floor [L ee_start_05 boot 24.9633 / floor 23.877; player.rs "standing live
1.086"], the port's rest is floor + cc = 50.035 [M 0x194df0 capsule arms] — the -3 draft
also buried the capsule 0.83 into the floor (the push = the west drift to 21.805). Placed at
the rest height, asserted the feet-band window. (2) The door #112 wp (35.4) stopped Bond at
x 34.88 — the use sphere (r 1.0 at head + look [M 0x1a5170:100-108]) reached 36.88, 0.24
short of the 0xE8's glist face (37.12). The arch wp (37.0, 50.1, −68.4) fires: E → fallback
ch 0x1E [M Trigger_Create 0x1b1720 p3→+0x2a; Trigger_Activate 0x1b2318:91-107] → exit
07000006. walkreal_07000005_live: 10 passed; 0 failed (99.31 s, slim ISO).

The ch 0x42 window decoded deeper: SP #284 (kind 0xD9, 0x06000952) runs the 100-frame camera
NIS — ev4 disables at its frame 8757, ev0xD sets script ch 0x62=1 at clock 93, the script
reaches done — and NO event 6 exists in 06000952/06000046/06000047, no placement consumes
ch 0x62, and the game's own SP_Update case 2 (0x1dbda8:143-154) ends a stay=0 player in
state 3 with no Script_Free: the hand-back mechanism is a live check (report §Live checks).
The east-stairs legs stay green-parked at the window. Suite (fast runner) still running at
wrap-up: only the slim-ISO artifacts failed (3 disc_golden + identity_disc's full-disc
sector assert). Report: docs/reports/level-05-every-objective-4.md.
