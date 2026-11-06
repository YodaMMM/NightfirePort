# bullet_pass and the water planes

`segment_surface_bullet` returns no 0xE hit at all on real levels (02: 0 against 1377 for the plain query), so `Geo::bullet_pass` treats every water plane as pass. A ray with flags 0xc keeps those planes in the game: the feet ray reads byte 0xE at 07000002 (-79, 28).

Read `Collide_Intersect` 0x1ebaa8:311,317 and the node bit behind `bVar7 & 2`, then find what bullet_pass reads and whether water really carries it. List every caller of the `_bullet` queries and say for each whether the game's flags would see water. Check what a bullet fired into water does in the game: a splash, a stop or a pass. Fix each difference with a test on real level data, proved red on the old code.

---
claimed-by: sandbox5 at 2026-11-06T18:17Z
