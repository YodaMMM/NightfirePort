# WORKLOG

## shallow-water-wade
Started 2026-11-06 12:57 UTC. Branch s/shallow-water-wade (from origin/master d6245281a).
Task: wading in shallow water — PCSX2-measured at 07000002 (-79, 4.68, 28) on a shallow 0xE
surface: the contact byte reads 0xE, anim+0x60 carries bit 0x10 alongside 8 while Bond stands
there, and steps play 0x28. Trace bit 0x10 of anim+0x60 to its writer and every reader; check
what the game changes while wading against the port (walk speed, ripples from Env_WaterRipple
on each step, footprint decals on 0xE, the eye height). 553 shallow cells on 02, 24 on 06 by
the port's plain ray (06's (24,-7) scan point is another surface in the game — confirm a spot
first). Fix each gap with a test on the real level, prove it red on the old code.
