# Aim ray: the object stage

Branch from `origin/s/aim-snap-final`, push to `s/aim-object-stage`. Read its report. The aim ray still has no object stage, so the game's snap (shutter object at (139.562, 50.792, -96.706) [L]) and the laser's cut of the 14 wire differ. Add the object stage with honest glist posing, in the order `Collide_Pick` 0x1eb320 and `Collide_Intersect` 0x1ebaa8 walk it.

Must end green: interactables_07000014_live, fusebox_geometry_07000014_live, rocket_glass08_live, rocket_stop_plane08_live, rocket_origin_live, solar_panel_hits_live. Then run walkreal_07000008, 0c, 14, 16 and 1b and report each. The 1b console walk needs firing spots above the panels. Cite the rule for the rocket's midpoint gate, or leave it alone. Strip the NF_AIMPROBE prints.

Run each binary whole and quote results.

---
claimed-by: sandbox5 at 2026-11-05T23:01Z
