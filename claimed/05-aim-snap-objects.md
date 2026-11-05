# Finish the crosshair aim snap: objects and glass

Branch from `origin/s/panel-aim-snap`, push to `s/aim-snap-objects`.

Proven live in PCSX2 on 0700001b with breakpoints on Collide_PickObj and Bullet_CollisionHandler: the game's beam round leaves the muzzle at y -1.23 and rises to -1.09 by x 3.47, clearing the panels, because `Player_WeaponInitBullet` 0x1a4bf0:154-164 snaps the aiming point with `Collide_RayIntersect` (mask 0x20a, 0x208 with +0x6C & 0x800) and aims from the muzzle midpoint at that point. Rounds crossing a panel's r 4.77 sphere get no hit entry unless they touch its triangles.

`Drones::aim_snap` covers the level and the panels only. The game's ray also hits objects, so a round aimed at a thin object converges on the wall behind it and misses. Port the object side of the ray: drones, doors, wires and fuseboxes, breakables, script entities. Read `Collide_RayIntersect` 0x1ec690 and `Collide_Pick`. Read what mask bit 2 means for glass; `aim_snap`'s glass argument may be inverted, and the rocket tests suggest the ray must behave as the game's does at the 08 window. Check `+0x6C & 0x800` against the records.

Failing on the branch (suite 4386 passed, 13 failed):
- fusebox_geometry_07000014_live, interactables_07000014_live: the laser cut
- rocket_stop_plane08_live (2), rocket_glass08_live: these are live-proven lines, make them pass by fixing the ray
- rocket_origin_live: `aimed at the aiming point, missed by 4.10`. This test assumes the point 256 out; update it to the snapped point
- walkreal_07000014_live (2), walkreal_0700000c_live real_damage, walkreal_07000016_live (2): check whether each is the same ray gap before touching any route
- walkreal_0700001b_live (2): "console 1 latched". The walk fires up through panels its stray shots used to destroy. Use new firing spots above the panels, as a player would. Console order is 3, 5, 6 ... and it stalls at console 8 from (44.77, -8.66, 1.68)

Keep `firing_along_the_walkway_breaks_no_panel` green. Keep the panel-only triangle stage unless a live check says otherwise. Run each binary whole and quote the results.

---
claimed-by: sandbox3 at 2026-11-05T19:29Z
