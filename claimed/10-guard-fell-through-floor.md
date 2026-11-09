# 07000001: a guard falls through the floor near (-28.7, 27.6)

With solid guard bodies (master 0a9d39435), in one 01 walk variant guard #4 fell through the floor near (-28.7, 27.6) and read y about -1e6. Reproduce through real input around that spot (guards pushed by Bond's body, crowding against the hall walls), log the guard's position, floor contact and state per tick, and find why he loses the floor (a push into geometry, a missing floor snap, a cel change). Compare with the game's drone floor handling (Drone_FeetOnPoint, the bounds push) and fix the port.

Reproduce on the real level first through real input, not a teleport. Branch from origin/master, push to `s/guard-fell-through-floor`, report in docs/reports/guard-fell-through-floor.md. Run every walkreal_* binary if the guard movement changes. No draw changes.

---
claimed-by: sandbox1 at 2026-11-09T19:57Z
