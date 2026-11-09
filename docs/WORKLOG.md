# WORKLOG

## guard-fell-through-floor

Branch cut from origin/master 0a9d39435. Task: 07000001 — with solid guard bodies
(master 0a9d39435), one 01 walk variant had guard #4 fall through the floor near
(-28.7, 27.6) and read y about -1e6. Reproduce through real input around that spot
(guards pushed by Bond's body, crowding against hall walls), log the guard's position,
floor contact and state per tick, find why he loses the floor (push into geometry,
missing floor snap, cel change), compare with the game's drone floor handling
(Drone_FeetOnPoint, the bounds push), fix the port. No draw changes.

Progress (21:45 UTC): setup redone after a sandbox reset (clone, work-slim, ISO, rustup+ALSA
user-prefix, nf-queue). Recon: guard #4 = d#4 skin 0x500003a, spawn (-27.23, 10.22, 2.28) on the
UPPER west floor (y 9.22-10.22, switch ch 0xF pair with d#12) [L probe roster, ee path d2]. The
reported fall spot (-28.7, 27.6) is VOID at every height on 07000001; the upper west floor ends
at z ≈ 16.7 where AI boundary walls close the north edge [L probe floor scan + wall dump].
Mechanisms read: Drone_FeetOnPoint 0x140660 (hip+0.6 ray, 0.2 contact band, the 0x60-byte
HITDATA memcpy), Drone_CollisionHandler 0x140968 (DoCollision/DoGravity, the 0.125 settle), the
bounds push AINetwork_GetBoundsPushVectorForSphere 0x157bd0 (XZ-only, y band [q.y-0.4, q.y+h],
pushes AWAY from the wall — a crossed wall is never recovered — walls flagged 0x1c00 skipped),
DealWithObjHit case 3 0x13fff0 (the +0x26 bump latch, msg 0x22). Tooling: NF_GUARD_FALL_TRACE
(additive env-gated walker trace) + walker::guard_fell + the red test
no_live_guard_falls_through_the_floor_on_the_walk (the whole real route, d1). Runs: d1 walk
12.18s pass, d2 real_damage_agent 8.22s pass — NO guard fall on either; the repro did not fire
on the two quick variants (the driver's variant is likely the ignored d3 00agent run's long
door-phase crowding, tick 22k+). Full suite not run (the window went to setup + recon; the test
binaries finished building ~21:35). Next: run the d3 variant / crowd d#4 at the upper floor's
north edge, pin the crossing tick, then fix per the report's candidates.
