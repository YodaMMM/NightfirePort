# level-15-agent-doors — the 15 Agent walk with the guards using their door routes

On master (after level-14-cascade-servers-4) guards take their door routes, and walkreal_07000015_live::walk_spawn_to_the_omega_sector_exit_real_damage_agent is ignored: Bond takes a -41.6 world hit (live wire / blast / hurt volume) at frame 5400, -22.5 from #11 (skin 0x50000b2) at 6191, and dies to #5 (skin 0x500008c) at 7885 near (-0.02, 41.32, 46.87). The Operative walk stays green.

Find what the -41.6 world hit is (which blast or volume, and why the door change moved Bond or a guard into it) and re-route the walk the way a player would. Real input only; no top-ups. Remove the ignore when green; run every walkreal_*_live; full suite total line.

---
claimed-by: sandbox3 at 2026-11-03T22:28Z

---
## Report (blocked, 2026-11-03T22:59Z)

