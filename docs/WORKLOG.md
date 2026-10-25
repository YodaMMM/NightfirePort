
## level-14-cutscene-fall

Branch `s/level-14-cutscene-fall` (from origin/master). Window 09:17-11:17 UTC.

Task: 07000014 — the broken cut-scene camera and the fall below the map after it. Bond spots:
the broken cut-scene at (40.87, 2.98, 40.85, -3.179, 0.210) frames 1931/2300 (camera inside
rock, flat blue void); the camera-view console at (33.71, 0.65, -18.11) frame 9909 ("Press
Action for Camera View" shows, nothing happens); panels at (54.21, 1.03, 46.15) frame 5739
(vent/hatch, nothing beneath) and (-88.58, -1.10, 46.15) frame 25538 (wall panel, nothing
behind); the fall: (-49.65, 0.65, 36.85) frame 20889 (black screen) then (-107.07, -70.14,
73.28) frame 21799 — Bond below the map after a broken cut-scene, objective "Sabotage base
security and escape from room". (1) Find which script plays at the cut-scene spot (trigger/SP,
its camera stream) and why the port's camera sits inside rock [M Script_UpdateCamera 0x1db518,
the SP's base matrix, the parent chain, the camera key interpolation]; fix it. (2) The fall:
find the cut-scene's hand-back (the marker entity that re-enables Bond, Player_Enable
0x195498 — the matrix origin placed directly [M :57-68]) and why the port's hand-back pose
lands off the floor; fix it [M]. (3) Real-input tests from the trigger to the hand-back: the
camera stays in the room, Bond is back on the floor, the walk continues.

Progress:
- (09:20) Repo + work/ + ISO survived; branch created from origin/master 8de76e31 and pushed;
  WORKLOG started.
