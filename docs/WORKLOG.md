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
