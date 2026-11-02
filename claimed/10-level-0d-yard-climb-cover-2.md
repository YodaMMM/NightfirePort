# level-0d-yard-climb-cover-2 — finish 0d on real health

Continue from origin/master (it carries level-0d-yard-climb-cover: the perch kill on drone #31 is proven free; probe_0d_climb_cover; the report on origin/s/level-0d-yard-climb-cover lists the next levers — kill #33 from the platform before the drop, avoid the standing fights whose noise turns #31).

Do: apply those levers with real input (no health writes), un-ignore walkreal_0700000d_live::walk_arrival_to_the_c5_door_real_damage, run walkreal_0700000d_live, every walkreal_*_live, then the full suite.

Rules: shared test helper changes (crates/nf-view/tests/common/) must be opt-in per level (the code must check the flag); re-run every walkreal_*_live before finishing. A visible fix ships with a real-input test that reaches the spot from the level start.

---
claimed-by: sandbox7 at 2026-11-02T23:38Z
