# WORKLOG

## guard-fell-through-floor

Branch cut from origin/master 0a9d39435. Task: 07000001 — with solid guard bodies
(master 0a9d39435), one 01 walk variant had guard #4 fall through the floor near
(-28.7, 27.6) and read y about -1e6. Reproduce through real input around that spot
(guards pushed by Bond's body, crowding against hall walls), log the guard's position,
floor contact and state per tick, find why he loses the floor (push into geometry,
missing floor snap, cel change), compare with the game's drone floor handling
(Drone_FeetOnPoint, the bounds push), fix the port. No draw changes.
