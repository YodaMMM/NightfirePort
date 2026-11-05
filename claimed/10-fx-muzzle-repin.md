# fx-muzzle-repin — land s/fx-muzzle-and-trail on current master

Live PCSX2 on 07000005 settled the open question: over 40 guard shots the flash object is created and its three jitter draws run every time (0x178f68 entered 40 times, 0x179248 and 0x179268 hit 39, the run ended at the 40th). The draws are right and stay.

Rebased on master they shift the random stream and turn 30 tests red: enemy_accuracy_live, fix_guard_weapons_live, walkreal 01/02/03/05/11/14/16, smg_close_range_live, upward_shot_0700000c_live, fix_grenade_types_live, fix_npc_doors_live, gun_turrets_live, fix_pickups_trace_live, start_corridor_guard_07000011_live. Merge master, keep gas.rs's hidden/born/child/rot fields on every literal, then fix each test without removing or reordering a draw: walks fight or move like a player, seeded pins re-derive from the game's rule. Run the full list above and report each result, then correct the stale comment in drone.rs that says these draws never fire.

---
claimed-by: sandbox1 at 2026-11-05T09:31Z
