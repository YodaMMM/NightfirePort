# WORKLOG

## audit-pickup-calls
Started 2026-11-05 02:32 UTC. Branch s/audit-pickup-calls (cut from origin/master 17a784980).
Task: sweep Pickup_Handler 0x18e650 and every caller of the inventory and ammo give paths for
on-foot play and multiplayer — conditions, amounts, caps, sounds, lines, respawn timing. Fix
what is missing or wrong at a clear port site, pin each with a test on real level data, report
the rest as a table. Merge origin/master first.

## local-match-bot-pickups
Started 2026-11-05 04:31 UTC. Branch s/local-match-bot-pickups (cut from origin/master
17a784980, origin/s/audit-pickup-calls 602bf0061 merged). Task: bots take pickups in a local
match with no net session — confirm the dead pass by test (a bot walks over a pickup, nothing
happens), call Sim::step_mp_pickups (scene.rs:13688) from the per-tick MP driver for local
matches only (the online path untouched), wire mp::unregister_pickup on delete
[M 0x18e650:341] + lifetime expiry [M 0x18e148:257] and reset_pickup_bot_visit_times on a bot's
respawn (0x171518), and pin through a real local match: bot takes a weapon, the sound plays at
the pickup, the respawn timer starts, and the MP Auto Aim row is read when the two rows differ.
Run mp_pickups_visible_live, mp_world_sync_live, every mp_*_live, pickup_live, nf-game; suite
via tools/test/suite.py in the background + once more on the final tree. Sandbox expires
~06:29 UTC; stop-new-work 06:09 UTC.
Round 2 (2026-11-05 05:40 UTC). The dormant pass wired: step_mp_pickups called from the MP
driver (step_bond) right after step_mp_bots, LOCAL matches only (the online net_bodies path
untouched); reconcile_mp_pickup_rows at the driver head before the bots' refresh (the delete
arm's unregister [M 0x18e650:341 → 0x18a058] + the expiry 0x18e148:254-258); the rows'
available re-read as the registered law [M pickGoal 0x126b88:386-390 skips only a gone
object's row]; reset_pickup_bot_visit_times at both BotDead ports [M 0x171518:17]. Pins
local_match_bot_pickups_live 3/0. Re-pins: mp_bots 7200-tick hunt window, mp_perf_16 digest,
pickups_g029 respawn 1201 (the audit's fix-4 law), mp_pickup_take's sound read. 43 of 45
mp/pickup binaries green; pickup_live 1/0; nf-game lib 662/0+5i; mp_online_failures +
mp_stress_explosives killed by the sandbox on the base tree too (pre-existing). Suite on the
final tree running in the background.
