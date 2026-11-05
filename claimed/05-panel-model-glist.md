# Panel model glist for the aim ray and the flight

Branch from `origin/s/aim-snap-objects`, push to `s/panel-model-glist`. Read docs/reports/aim-snap-objects.md first.
## Update: read this first (live, PCSX2 breakpoint right after the aim ray, 0x1a4d90)

The aim ray from head (16.741, -1.137, 4.62) stops on LEVEL geometry, not a panel: hit point (6.958, -1.137, 4.62), hit object 0, surface 0x1c76680. It is the same surface the round's own contact lists later. So the first job is the level side: find that surface in the 0700001b data and why `segment_aim_scoped` passes it (t 1.0). Check the cel scope and the aim_pick filter first. Measured on the port at the same head: `segment_flight_scoped` and `line_of_sight_hit` both return x 6.958338, an exact match with the game; `segment_aim_scoped` returns None. So the level data is right and the `aim_pick` filter drops this surface. Check first whether world.rs's `aim_pick` is indexed wrongly (`stat.owners` post-chunk index against the pushed instance ordinal), then whether the 0x8000 rule is the right test: the 08 shutters must stay out of the ray and this surface must stay in.

The panel glist is still needed for round-vs-panel contact only if the walkway test stays red after the snap lands on (6.958, -1.137).

The port poses `world.shape` triangles for script entities and they are mis-framed (the 14 wires' posed bounds landed at x +42). `Collide_Intersect` 0x1ebaa8's 0x201 arm tests the object's OWN glist (obj+0xD8). Parse the panel model's glist and use it for both the flight's panel stage and the aim ray.

Live numbers from PCSX2 on 0700001b, Bond at (16.74, -1.84, 4.62) yaw -1.57 pitch level: the round starts at (16.2, -1.23, 4.53) and passes (11.97, -1.18), (7.72, -1.14), (3.47, -1.09). A rise of 0.0118 a unit needs a snap point near x 8.6, y -1.14, so the game's aim ray IS stopped by something there; a far point gives a rise of 0.0004. Find what it hits: the panel's real triangles are the likely answer. Panels are objects with r 4.77 at (10.14, -1.72, 4.62), (16.71, -2.88, 4.62) and every 6.57 along x; f0 0x81. A round touching their triangles lists the panel in its hit chain; the walkway round has no entry.

Then: `firing_along_the_walkway_breaks_no_panel` green, walkreal_0700001b_live's consoles from spots a player would use, walkreal_07000014_live (3 red, one may be the ray's object side) and walkreal_07000016_live (1 red) checked. Strip the NF_AIMPROBE prints. Keep rocket_stop_plane08, rocket_glass08, fusebox_geometry_07000014, interactables_07000014 and walkreal_0700000c green.

Run each binary whole and quote the results.

---
claimed-by: sandbox5 at 2026-11-05T21:32Z
