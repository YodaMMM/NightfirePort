
## guard-alt-voice-set

Branch `s/guard-alt-voice-set` from `origin/master` (22d5836b7). Task: every second guard
speaks from the game's alternate line set — the `+0x108 & 1` serial that
`Drone_SM_InitObject` 0x175838 stores (DAT_00299090 + 1, per object init); six talk sites to
re-pin (AttackTalk 0x13eac0:86, HitTalkExtra 0x13ed38:49, SeenPlayerTalk 0x13f1d0:66/77/88/96,
InjuredTalk 0x13fa20:45, RunForCoverTalk 0x13fd08:45). Confirm the record, find the counter
reset, list the game's init order (every serial-taking object shifts later guards' parity),
give the guards the game's serial, fix the six sites, keep the Rand-draw count identical, pin
the parity through real play on 07000014. Entry started — work follows.

Outcome: the six sites fixed on the game's own serial. Evidence chain ([M] disassembly from
work/ee_snapshot.bin, file offset = EE address — the slim decomp folder has no main-binary C):
`Drone_SM_InitObject` 0x175838 reads the u16 counter at 0x299310 (DAT_00299090+0x280; a3 =
0x2a0000-0x71f0 + 0x280), stores counter+1 at SMI+0x108 [M 0x175858-0x175878], SMI = obj+0xE0
[M 0x17584c]; `Drone_LevelReset` 0x13a5a8 zeroes it [M 0x13a614 `sh $zero, 0x280($s0)`] — the
serials restart per level. The only caller of SM_InitObject is `NDrone2_PostLoad_Init`
0x14d890 [M jal 0x14d8f0]; PostLoad's callers: Drone_PostLoad_Init's level-load walk [M
0x136b90, the 0x299094 list head = creation order, `LList_Add` appends], the spawner's
0x137e04, Drone_CoderCreate's 0x13a19c, BOT_init's 0x124df4 (MP). The six sites' bit-reads
confirmed [M]: HitTalkExtra 0x13ee5c (0x133|0x138), AttackTalk 0x13ecb4 (0x11b|0x122 — the
type-7 arm via the 0x2f4a30 table entry 7, level-gated 14-16, odds 0), SeenPlayerTalk
0x13f2f4/0x13f328/0x13f35c/0x13f380 (per-level 14/15/16 pairs 0xf3|0xf5, 0xf7|0xf9, 0xfb|0xfd
+ the common tail 0x10d|0x10f, each pair 2 wide, two Rand(2) draws: pair-vs-tail then the
offset), InjuredTalk 0x13fb10 (0x133|0x138), RunForCoverTalk 0x13fdf8 (0x11b|0x122). Port
fixes: `sm_serial` on Drones (the game's counter; zeroed at the reset, incremented per init),
the load serials handed out in the spawner block (skipping the templates — the game's
templates have no object until spawn), the CoderCreate + spawner-spawn arms take the next
serial; `walk_anim` reads the serial directly (behavior-neutral: the old `sm_id+1` == the
serial); all six sites use `sm_id & 1`. REACHABILITY correction found in the decompile:
HitTalkExtra's types-5..7 arm is game-DEAD (its only caller is PainTalk's type-0x10 arm [M
the 0x2f4a80 table entry 15]; the 5-7 entries take the default 0xf/0x17) — the LIVE parity
sites on 14-16 are SeenPlayerTalk (every type) and AttackTalk (type 7). Tests: the nf-game lib
`alt_voice_set_follows_the_serial_parity` pins odd → the alternate half, even → the base half
across HitTalkExtra/SeenPlayerTalk/AttackTalk; the live 07000014 pin
(guard_voice_parity_07000014_live) verified the serial hand-out (the three load-created guards
#0/#1/#22 at serials 2/3/1 [L]) but produced no 14-16 line (the stealth opening + one-hit
kills) — ignored with the measured state, the sighting legs listed as the next run's work.
Binaries: guard_barks_07000013_live 4/0, escort_follow_07000001_live 1/0, fix_06_driver_live
2/0, hostage_killer_07000002_live 5/0, ai_traces_live 4/0, nf-game lib 691/0/5. The full
fast-suite run (tools/test/suite.py) did not complete in the window. Report:
docs/reports/guard-alt-voice-set.md.
