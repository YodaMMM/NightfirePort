# WORKLOG

## level-12-audit (2026-10-26, sandbox expiring ~09:16 UTC)
Branch `s/level-12-audit` off origin/master (b604242f). Task: the 07000012 behaviour audit —
every placement on the main map [D] (characters 0x0f, doors, triggers, scripts/SPs, switches,
monitors/sensors, pickups, destroyables, breakables, ladders, grapple points, lifts, special
objects), every script stream (ops, camera streams, hand-backs) and every mission record
(objectives, channels, fail conditions): the game's creator/update/handlers [M], the port's, and
a GAP column, written to docs/levels/07000012-audit.md. Small gaps fixed directly [M] with a test
each; large gaps become report sections (what, where, the [M] chain). The known symptoms: the
walk test passes but real play finds characters walking through Bond, guests that never talk,
doors that never lock, consoles whose action does nothing, cut-scenes with broken cameras.
Plan: reuse the sandbox (work/ + ISO + nf-queue + rust + alsa all present); branch pushed at
start; suite in the background early; the level doc docs/levels/07000012.md as the base; census
via a disc dump; the decomp per kind from work/decomp/ACTION.ELF.
(Note: the previous session's task fusebox-inverter-2 was closed first — its finish.sh + ntfy
had been lost with the sandbox; the queue shows it done with the report attached.)

### level-12-audit — the audit written; the script/record sections + the death-channel pin (2026-10-26 ~07:55 UTC)
docs/levels/07000012-audit.md: all 792 main-map placements as rows grouped by 33 kinds (each with
the game's [M/D] chain, the port's, GAP), plus: the four SP scripts decoded per stream (the intro
0x06000530 has NO camera op — the level doc's "entities + camera" was wrong on the camera half;
#662's script is 0x060005bb, the doc's 0x060005FB a misread); the full kind-7 sweep — exactly four
channel writes on the level (intro ch 0x3c + sub 0x89 pulse, both with NO consumers; the death
script's ch 0x62); the mission record (2 objectives, fail 0xffffffff — the civilian ch 0x63 and
shaft ch 0x62 fail OUTSIDE the record); the writer census (no writer of the door lock channels
0xc8/0xc5 nor the alarm 0x61 — those doors never lock by data, in the game as well; the door
"lock" channel's rising edge is an UNLOCK: label 0x2000003 = "Door Unlocked" [D door-text-labels.md,
M 0x1363c8:88-112 — the burn opens #754]); the dispatcher-skip rows (0x19/0x47/0x31 — the port
matches behaviourally). Large gap #1 in the report: the drone HITTEST (the walk-through-Bond
symptom) — the [M] chain cited from drone.rs's own spec comment. New tests
(scripts_07000012_live): the shaft-fall ch 0x62 pin (green); the intro park pin (mask p5=2 ->
state 0 [M 0x1db9a8:74-86], the 07000006 #808 precedent). The two test binaries got SIGKILLed
under suite memory pressure — re-run after the suite drains.

### level-12-audit — the finish (2026-10-26 ~08:50 UTC)
The suite ran to 413 of 820 binaries before the wall: 1414 passed / 0 failed / 115 ignored; the 6
CRASHes are exit -9 SIGKILLs of the net_*/mp_* memory hogs (mp_online_failures SIGKILLs standalone
too — the 4 GB sandbox ceiling, not a failure; no lib code changed on this branch). Suite killed at
a hung binary; the next session owns the remaining 407 binaries + the formal total. Report at
docs/reports/level-12-audit.md with the live checks (the intro's activation beat, the guard bump,
the civilian barks, the death-cam render).
