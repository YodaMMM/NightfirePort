# 03: the walk after the one-way door fix

`origin/s/door-fix-on-master` is reviewed: one-way doors refuse from the game's side, the hostage music cue plays, the scripted death cry fires on the right event. Full suite 4426 passed, 1 failed. Branch from it, push to `s/door-walk-03`. One small job.

The red: `walkreal_07000003_live walk_the_full_route_and_complete_every_objective`, "servant 1's captor #11 died to the chase": Bond reaches the chase dry (clip 1 spare 0) and shoots nothing for 10800 ticks. Reverting only the door direction turns it green, so the route rode the old mirrored door. Adding `switch_to_armed` did not help: nothing else to hold.

The door fix is proved. PCSX2 07000003, door #1 at (13.96, 6.63, -30.27), facing +z, mode 1: from z -28.9 every press takes the refusal path (ra 0x135d24) and the door stays shut; from z -31.6 it opens and closes (0x17 / 0x18). Do not touch door code.

Find where the walk goes through that door, route it the way a player must, and fix the ammo: pick up what the old route collected. Quote walkreal 03 whole, and 02, 04 and the interactables_07000003 binary if it exists. Test-side only. Do not loosen pins tagged [L].

---
claimed-by: sandbox7 at 2026-11-06T15:01Z

---
## Report (blocked, 2026-11-06T16:17Z)

