# Aim: finish the 14 cut and re-quote

Branch from `origin/s/aim-node-pass-2`, merge current `origin/master`, push to `s/aim-finish`.

The branch carries src changes from the aim stack (door.rs, drone.rs, nis.rs, world.rs, scene.rs), so its binaries must be re-run on the merged tree. "Test-side only" in its report covers the last step only.

1. Rewrite the cut loop in `interactables_07000014_live` the way `fusebox_geometry_07000014_live` does it: aim from `bond.head()`, set `bond.yaw/pitch` directly, hip-fire with plain input. Keep every assertion.
2. Run and quote counts for: fusebox_geometry_07000014_live, interactables_07000014_live, rocket_stop_plane08_live, rocket_glass08 tests, rocket_origin_live, solar_panel_hits_live, walkreal 08 / 0c / 14 / 16 / 1b.
3. Check the two aim filters read `inst.placement.flags`, not the object flags, and print a measured count of filtered instances per level. A filter that drops nothing makes a pass void.
