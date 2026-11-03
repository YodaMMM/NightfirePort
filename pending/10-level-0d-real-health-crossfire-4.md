# level-0d-real-health-crossfire-4 — keep the C5 walk alive while finishing the crossfire route

Branch from origin/s/level-0d-real-health-crossfire-3 (merge origin/master first) and read its report. That branch moves the 0d real-health route 7250 ticks further (to the annex descent, where #32 the ground spawner kills Bond) and the non-damage twin is green.

But it breaks a test that is green on master: walkreal_0700000d_live::walk_arrival_to_the_c5_door_real_damage now dies at tick 4482 at the ladder top (213.07, 17.37, -234.51): five -16 sniper hits from #18 (207.6, 12.1, -288.6), #16 (198.9, 14.0, -260.4) and #28. The route changes are shared with that walk. Keep the C5 walk's old route (or make the new one survive there) and finish the crossfire: deal with #32 at the annex descent.

Run every test in walkreal_0700000d_live and list each result; then every walkreal_*_live; then the full suite total line. Real input only; no top-ups.
