# level-0a-ring-beams-3 — land the beam contact with the ring walk re-timed

Branch from origin/s/trigger-beams-3 (merge origin/master first). That tree scopes the strict trigger law to laser-beam glists (flags 0x645, `Trigger::seg_beam`); every other trigger keeps master's contact, because the segment on all triggers turned 17 tests red across 01, 02, 05, 06, 0c, 11, 4a by firing floor plates ahead of the box. Do not widen that scope. Master now also carries the riding-Bond toucher (level-05-truck-class5-contact): keep it when you merge.

State: low_beam_621_fires_standing and fix_truck_live green. walkreal_0700000a_live::walk_office_floor_qworm_keyboard_hatch and its three real-damage sisters are red: the walk's timing shifted and it now crosses beam #625 (out 0x26, x 18.88, z 2.38-7.64, y 0.39-0.65 [D]). Re-time those crossings against the beams' key sweeps the way s/level-0a-ring-sweep-timing did for #614.

Done = those four green plus the WHOLE of every walkreal_*_live, interactables_*_live, fix_truck_live, fix_npc_doors_live, fix_level_chain_live, interact_07000005_live, smg_close_range_live, use_key_census_live and mp_world_sync_live, each listed with its count; then the full suite total line.

---
claimed-by: sandbox7 at 2026-11-04T13:24Z

---
## Report (done, 2026-11-04T15:09Z)

