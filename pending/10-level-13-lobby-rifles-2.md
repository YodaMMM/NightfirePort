# level-13-lobby-rifles-2 — land the 13 lobby rifles without breaking other levels' walks

Branch from origin/master; merge origin/s/level-13-lobby-rifles in first.

That branch makes walkreal_07000013's real-damage walk pass, but merged onto master it kills Bond on two other real-damage walks:
- walkreal_07000001_live::walk_from_the_spawn_to_the_exit_with_the_contact_real_damage (killed at tick ~14958 by drone #12 near (-31, 6.3, 14.6))
- walkreal_07000016_live::walk_the_chasers_defeat_completes_the_mission_on_real_damage

Both pass on master and with the branch reverted. The cause is the SHARED walker resupply in tests/common/walker_b2.rs: the fetch cap went 45 → 12 units, the pickup floor filter went 1.5 → 0.5, a tried-target list was added and the ammo count now includes the inventory. Other levels' walks relied on the old resupply.

Do:
1. Keep the 13 fix, but do not change shared walker behaviour for other levels: make the new rules opt-in for the 07000013 walk (or prove a single rule set that passes every real-damage walk).
2. No health or ammo writes; real input only.
3. Run every walkreal_*_live real-damage test (01, 02, 03, 13, 14, 16 at least), walkreal_07000013_live, then the full suite.
