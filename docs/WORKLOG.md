
## guard-alt-voice-set

Branch `s/guard-alt-voice-set` from `origin/master` (22d5836b7). Task: every second guard
speaks from the game's alternate line set — the `+0x108 & 1` serial that
`Drone_SM_InitObject` 0x175838 stores (DAT_00299090 + 1, per object init); six talk sites to
re-pin (AttackTalk 0x13eac0:86, HitTalkExtra 0x13ed38:49, SeenPlayerTalk 0x13f1d0:66/77/88/96,
InjuredTalk 0x13fa20:45, RunForCoverTalk 0x13fd08:45). Confirm the record, find the counter
reset, list the game's init order (every serial-taking object shifts later guards' parity),
give the guards the game's serial, fix the six sites, keep the Rand-draw count identical, pin
the parity through real play on 07000014. Entry started — work follows.
