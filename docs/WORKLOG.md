# WORKLOG

## guard-opponent-pointer (branch s/guard-opponent-pointer)

Start: 2026-10-30 09:22 UTC, branch from origin/master 04820075.

Task: port the drone opponent pointer (+0x170) as a field the AI reads, written the way the game
writes it for team-1 guards (FindOpponent tick fold [M 0x1424e0:100-121], SendAttackMessage,
ChangeToAttackMode, the vision/comms path), so the NoOpponent 0x60 return to Attack and
NDrone2_DoorGuardHandler 0x1416d8 follow the game's pointer, not the "guard currently sees Bond"
stand-in. Read done/30-combat-sight-loss-exit.md first; pin with real-input tests on a door-guard
level (07000014 or 07000007) and a guard that loses sight of Bond.

Outcome 11:05 UTC: the +0x170 pointer is a field the AI reads — the ELF-wide sw-sweep settles the
writer set (SetOpponent 0x1422d0:20 sole writer; SendAttackMessage/ChangeToAttackMode verified
non-writers by disassembly; Drone_Delete 0x136ce8:53-54 clear; the 0x1399ec hit is an MP struct
copy). Ported: `opp_bond` + the per-tick fold [M 0x1424e0:100-121], the +0x238 stamp for every
team behind the reaction gate [M 0x175ef0:66-68] with the pointer-gated visible branch
[M 0x176a28:222-231], HasOpponent unified pointer-or-grace for every team [M 0x14ad58:8-15]
(master had lost the every-team grace from combat-sight-loss-exit's merge), NoOpponent exits +
Interogate family + abseil arms on the pure pointer, DoorGuardHandler's zone gate on the pointer
not sight [M 0x1416d8:9-11]. Touched binaries 20 passed / 0 failed (door_guard_live 5, ai_missing_
states_live 9, find_opponent_live 3, vision_bit_census 1, walkreal_07000007_live 2); the full
suite did not fit — this sandbox kills background processes (verified), suite.py survives only in
≤10-min foreground chunks. Report: docs/reports/guard-opponent-pointer.md.

## guard-opponent-pointer-2 (branch s/guard-opponent-pointer-2)

Start: 2026-10-30 12:37 UTC, branch from origin/s/guard-opponent-pointer merged with origin/master dca2f368.

Task: the merged -1 branch turns two tests red (both green on master, both red alone):
grenade_live::guard_decides_to_throw_grenade_unforced (can_throw_grenade never passes within 1800
ticks) and bathhouse_shout_mixer_live::the_bathhouse_floor_guards_shout_reaches_the_mixer_too (no
wake-group guard shouts). Find which reader of the pointer/grace changed each path [M], fix the
reader or the test (with the decompile cite), un-break, full-file greens + guard/drone/alarm/shout
binaries + walkreal 03/09/0b/13.

Outcome 13:40 UTC: both reds are HARNESS ARTIFACTS pinning master's stand-in behaviour — the -1
branch's port readers are the game's own and no source change is needed. (1) grenade_live's
blanket `set_channel(1..=255)` set the standing fail-hook channels 0x60/0x61 [M 0x1827d8:98-117]
and the internal 99/100 [M 0x182c60:23-33] → Mission_Status ≥ 2 within a tick →
`PlrStat_OkToUpdate` 0 [M 0x1993e0] → FindOpponent zeroed +0x170 every tick [M 0x1424e0:106-108]
→ the router head parked the guard in 0x60 forever [M 0x148338:20-31] and CanThrowGrenade's own
+0x170 gate [M 0x17a798:16] could never pass — the probe run printed stat_ok=false with
bond_alive=true. The harness now leaves {0x60,0x61,0x62,99,100} off (the objective channels
0x48/0x4b/0x4d announce but do not move Mission_Status [L ee_start_05.bin 0x2a3ed0]). (2) the
bathhouse floor walk buried Bond at plate centre y−1.0 (the drop the bunker sibling's comment
already calls out) and he drowned mid-walk (death frame 2143, health 0.00 — probed) → ch 0x62
[M 0x182c60:21] → Mission_Status 3 → the pointer zeroed on the dead-class arms [M 0x1424e0:111-114]
→ the visible-sight branch never acks [M 0x176a28:222-231] → no AttackTalk site [M 0x13eac0's
callers] fires; master passed the same run by acking sight on a dead Bond (no pointer gate). The
walk now stands on the plate centres like the bunker test. grenade_live 2/2,
bathhouse_shout_mixer_live 2/2; the full final-tree suite is running (totals to the report).
