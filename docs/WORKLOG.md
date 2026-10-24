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

- (19:26 UTC) Sandbox had the repo/work/ISO from the previous session: skipped all setup, branched
  s/ai-dmode-inits from origin/master, pushed the WORKLOG checkpoint. Two parallel subagents
  re-derived the task's [D] evidence. Table subagent: DroneModeSettings 0x29ad40 (file 0x194C40)
  = 35 rows x 12 bytes, all 32 non-NULL fn pointers exact-symbol matches (row 0 Normal 0x14d958,
  team 1, alert word 0.0); rows 23-33 named (23 CivilianGuard ... 29 SniperAlert ... 33 Bot
  0x14e568; 31/32/34 NULL); the reader byte-maps confirmed in disassembly (0xC5/0x44/jalr).
  Census subagent: main-map histogram {100: 555} MATCH; PLUS 33 kind-0x0f template records
  (param5 = 0) in the type-0 map 01000166 across 41 archives (588 total) — runtime reachability
  [S] unverified, row 0 anyway; 35 archives have main maps (21 with drones). CORRECTIONS found:
  the coder-event hashes in ai-dmode-inits.md/unreached.md/drone.rs/the row-0 test were wrong —
  the disc's only Cmd::Event kind 8 is 07000001 entry 0600005B with params [0x0500001B,
  0x060000A6, 16, 1] (not 0600003B / 0500005B / 06000106); word 3 → param 7, word 2 (16) dropped
  [M 0x13a108 disasm: template words 11/12/20 = sp+44/48/80]. Skin 0x0500001B takes DefaultInit
  case 0x500001b → +0xd8 = 9 [M 0x14b300:153-156] (extras still don't fire: 9 ∉ {0x10,0xd,0x11,
  0x13}). CoderCreate's +0x5A2 = 4 [M 0x13a108:0x13a1a0] is overwritten by DoTypeSettingsOLD's
  unconditional by-mode write → 6 InitPatrol via Defaults' prop 0x24 [M 0x14d2e8:0x14d32c/0x14d3bc]
  — the port's start_state already computes 6; pinned with a new assertion. The row w+4 →
  +0x500/+0x50C copy is real but FPU: lwc1 f0,4(s2) @0x14d010 → swc1 f0,0x50C/0x500 @0x14d014/
  0x14d01c. Fixed drone.rs comment + test hashes, appended the addendum to ai-dmode-inits.md and
  a correction section to unreached.md. dmode_row0_near_player_live 2/0 green on the corrected
  tree. Merged origin/master (78b02cfd batch) → final tree bb394a09; suite restarted clean
  (prior run was a mixed tree). Baseline known-fails on the slim ISO: identity_disc + 3
  disc_golden (same set as the older /home/z/suite.log).
