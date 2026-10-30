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
