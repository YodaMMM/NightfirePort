# Explosion debris: the game throws more pieces than the port

On 07000011, one MGL grenade at the steel shelf by the MGL pickup: the PS2 threw 12-15 metal pieces (live debris list read in PCSX2) where the port throws 6, and the PS2's pieces stayed in the room longer (see audits/render-1109-grenade/ in the main workspace: fade_ps2_* and ps2_log.txt; the capture script mgl_live.py). Read the game's debris count and launch rules (Explode_Update 0x17c350's debris arm, Debris_CreateEx 0x1ac808, the count/param words of the explosion record, Rand draws), compare with the port's blast debris spawn, and fix the count, speed and life to the game's. Keep the Rand draw count equal to the game's.

Reproduce on the real level first through real input, not a teleport. Branch from origin/master, push to `s/debris-piece-count`, report in docs/reports/debris-piece-count.md. Run walkreal_07000011 and the debris/blast test binaries. No draw changes.

---
claimed-by: sandbox3 at 2026-11-09T16:44Z
