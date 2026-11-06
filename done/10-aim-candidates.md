# Aim snap: the object stage stops on objects the game does not have

Branch from `origin/s/aim-finish`, merge current `origin/master`, push to `s/aim-candidates`.

`s/aim-finish` regresses 8 walks (08 x1, 0c x1, 14 x3, 16 x1, 1b x2). Its report blames the snap for passing script entities. Measurement says the opposite: the port stops on things the game's ray passes.

Measured on 0700001b:
- Port, the aim stage on this branch: at the walk's console shots the level stage returns t = 1 (no hit, the same as the game) and the object stage snaps to entities at (38.74, -5.21, +/-9.415), (25.72, -2.91, +/-9.415), (25.72, -2.90, +/-9.60), (25.95, -4.35, -11.37), (25.72, -2.90, -11.39). Hashes 0x2000a41 (f0 0x10), 0x2000963, 0x20007e6, 0x20007f6 (f0 0x80). All in cel 26 with the shooter, so cel scoping does not explain it.
- PCSX2: a level ray from head (32, -5.209, 0), dir (0.582, 0, 0.813), straight through (38.741, -5.209, 9.415): the aim ray return at 0x1a4d90 has hit = NULL. One stance only.
- PCSX2: the dynamic object list (head at 0x2705b4, next +0x14) holds 370 objects shortly after level start, none within 1.0 of those positions by position or built sphere. The panel at (36.41, -6.36, -4.62), class 0, f0 0x81, is there. The static list was not read and later phases were not sampled.

Do these, in order:
1. Fix the pre-filter at both sites in `aim_snap`: the walk's constant is `lui s1,0x3000` = 0x30000000 [M 0x1eb910], not 0x3000. The f0 word is `glist+0x44 | old f0 & 0x30008000` [M Control_SetGList 0x134960:13].
2. Add the gates `Collide_PickObj` 0x1eae80 applies that the stage lacks: with 0x200 set in the hittest word class 5 is skipped, and `obj+0xfa & 8` or `glist+8 != 0` must hold for a non-0x100 type.
3. Make the candidate set the objects the game really links: per-cel chains plus ForcedList [M 0x1eb8e8-0x1eb93c, 0x1eba44-0x1eba5c]. Find what those stream entities are, and why `sp_bullet_targets` returns them when no live object sits there: a wrong pose, a script that has not created them, or a list the game keeps apart. Cite it.
4. `Door.bullet_pass` still reads the placement stand-in; the f0 word is the real gate. Change it if the 08 pins allow, and say what they measured.

Keep the [L] 08 pins. Quote fusebox_geometry_07000014, interactables_07000014, rocket_stop_plane08, rocket_glass08, rocket_origin, solar_panel_hits, walkreal 08 / 0c / 14 / 16 / 1b.

---
claimed-by: sandbox7 at 2026-11-06T08:23Z

---
## Report (done, 2026-11-06T09:42Z)

