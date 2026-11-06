# Wading in shallow water

Measured in PCSX2 at 07000002 (-79, 4.68, 28), a shallow 0xE surface: the contact byte reads 0xE, anim+0x60 carries bit 0x10 alongside 8 while Bond stands there, and steps play 0x28. Landing and step sounds were just fixed on master; nothing else was checked.

Trace bit 0x10 of anim+0x60 to its writer and every reader. Check what the game changes while wading against the port: walk speed, ripples from `Env_WaterRipple` on each step, footprint decals on 0xE, the eye height. Use 553 shallow cells on 02 and 24 on 06 by the port's plain ray (06's scan point at (24, -7) turned out to be another surface in the game, so confirm a spot first). Fix each gap with a test on the real level, not hand-built triangles, and prove it red on the old code.

---
claimed-by: sandbox3 at 2026-11-06T11:24Z
