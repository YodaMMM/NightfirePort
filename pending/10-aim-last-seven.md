# Aim on master: the last seven

Branch from `origin/s/aim-on-master` (aim-route-6 merged with master), push to `s/aim-last-seven`. Full suite: 4428 passed, 7 failed, 352 ignored.

First, because a player would see it: `stairwell_impacts_render_07000011_live every_stairwell_impact_sits_on_drawn_geometry`. If the snap moves bullet marks off drawn geometry, that is a real fault: name the cause.
Then `panel_sequence_07000014_live` x2 (one_cut_box_arms_nothing..., the_panel_sequence_runs...) and `walk_07000014_live the_facility_chain_runs_to_the_delta_sector_exit`: likely the 14 route or the cut shot; see how the 14 walk was fixed in aim-route-finish.
Then `walkreal_07000015_live` real_damage_agent. `walkreal_07000001_live` 00agent and `walkreal_07000008_live` 00agent may take an ignore with a measured reason.

Keep the [L] 08 pins and the 0x30000000 fix. Quote one total line.
