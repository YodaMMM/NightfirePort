### ai-dmode-inits (started 19:05 UTC, 2026-10-24)

Task: the 29 `NDrone2_init_DMODE_*` functions (0x14d978..0x14e568: Guard, Retreater, Sniper,
Stealth, Attacker, RunToPoint, Assassin, HostageKiller, Hostage, HostageTied, JustStand4Demo,
DeleteMe, Civilian, CivilianScared, MissionFailer, Mayhew, Ninja, AlarmRaiser, SearchLight, Zoe,
PartyGirl, Interogator, CivDoorGuard, TruckDriver, CastleChatGuard1, CastleChatGuard2,
SniperAlert, PartyGirlLooker, Bot) are never mentioned in `crates/*/src`. The port sets up guards
through `NDrone2_DefaultInit` 0x14b300's mode table in its own words. Find the dispatch (the
mode byte -> init table) [M] and, from the placements [D], which modes each on-foot level uses.
For every init: compare what it writes (state, flags, weapon, voice, speed, alert set, the first
state) with what the port sets for that mode; fix every difference [M] with a test per mode
family on a level that uses it, and put the function name and address at the port's code for
that mode. A mode no on-foot level uses: cite the proof (the placement census) in a comment
instead. Tests pin only proven behaviour ([M]/[D]/[L] citations; [S] marked unverified).

Plan: start the fast suite runner in the background first; then dump the dispatch table
(0x14d978 callers / the mode byte switch near 0x14b300), grep the port's guard init for the
mode table, diff per mode, fix + test per mode family on a level that uses it, un-verify unused
modes with the placement census comment.

