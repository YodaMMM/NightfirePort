# WORKLOG

### level-14-camera-view (started 19:36 UTC, 2026-10-26)

Task: on 07000014 the security consoles prompt "Press Action for Camera View" but pressing Action
does nothing. Find the console's activation chain [D placements near (33.71, 0.65, -18.11) + their
triggers/scripts] and the game's camera-view mode it enters [M: ThirdCam_Create 0x1bc510,
ThirdCam_GetPos 0x1bc590, the Player_Activate arm that switches the view, and how it exits], then
port the sim side: Action at the console switches the active view to the camera the game picks,
input stays as the game routes it while viewing, the view returns the way the game returns it.
Expose the active view camera (position/orientation per tick) for the renderer; overlay/draw side
is done separately. Tests: Action at the console enters the camera view with the game's camera id
[M + D]; the exit restores the player view.

Plan: branch s/level-14-camera-view from origin/master (70035871) pushed as checkpoint 1; suite
runner started in the background; census the console action chain in the port (interact/placement
prompts -> Player_Activate) and the game's ThirdCam path in the decompile; write the failing
tests through the real input path; fix with [M] citations; full suite; report + finish by 21:14 UTC.

Progress (21:30 UTC, checkpoint 3 = fcd205b0, master merged): the research pass disentangled the two
systems — the "Press Action for Camera View" console is the Monitor (class 0x38, kind 0x30) whose feed
is the watched Sensor's own world matrix; ThirdCam (0x1bc510/0x1bc590) is the creepwall/wire rail
camera, NOT the security view. Ported the sim side: Monitor_Activate 0x1b0058 state-0 enter (head
walk, display mask 0 hide), Monitor_Update 0x1afcc8 state-2 body (death re-pick forward+head, the
action-0xE edge polled INSIDE the update — forward cycle no wrap, exit arm state 9 + Player_Enable
restore, channel latch gated `if (ch != 0)`, +0x60 unhide, cooldown drain 9→…→5→4→0 as separate
switch arms so the exit tick leaves state 9), Sensor.display_mask = obj+0xF8, player input freeze
while viewing (Player_Disable 0x195f70 → Player_Update 0x193600:80-87 early-return), and
Sim::monitor_camera() exposing (pos, yaw, pitch) from the watched sensor's matrix (Mat_Copy
sensor+0x90 → viewer+0x120 [M 0x1afcc8]; yaw = atan2(row2.x, row2.z), pitch = asin(row2.y)) hooked
into nf-view's camera pick. Tests: 9 monitor/sensor unit tests + 3 live ISO tests on 07000014
(enter on the game's first camera = camera(a)-lens #1035 [D], freeze while viewing, forward cycle +
exit restores the player view, idle without Action) — all green. An earlier draft had the cycle go
through a re-routed Player_Activate and the exit tick drain 9→8; fixed to the decompile structure
(edge inside Monitor_Update, separate drain arm) with the decompile read directly
(work/decomp/ACTION.ELF/001afcc8). Full suite re-running on the merged tree for the report.
