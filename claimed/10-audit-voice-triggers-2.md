# audit-voice-triggers-2 — the four barks' effect on the walks

Continue from origin/s/audit-voice-triggers; merge origin/master first. The four added triggers read correct against the decompile, so do not remove a draw to make a test pass unless a line is proved wrong. Merged with master these four are red, and they are green without the branch:
- walkreal_07000001_live::following_mayhew_down_completes_the_level (the AND-touch never fires, walkreal_07000001_live.rs:1522)
- walkreal_07000001_live::walk_from_the_spawn_to_the_exit_with_the_contact_real_damage (Bond dies at tick 3253)
- walkreal_07000003_live::walk_the_full_route_and_complete_every_objective (the east lane cap)
- smg_close_range_live::smg_rounds_damage_a_close_guard_and_the_pistol_sister (the pistol control at 1 u)
For each, say which draw moved what, then re-route or re-pin the test, keeping every pin honest. Run every walkreal binary and the full suite, and quote the total line.

---
claimed-by: sandbox4 at 2026-11-05T05:15Z
