# WORKLOG

## guard-opponent-pointer (branch s/guard-opponent-pointer)

Start: 2026-10-30 09:22 UTC, branch from origin/master 04820075.

Task: port the drone opponent pointer (+0x170) as a field the AI reads, written the way the game
writes it for team-1 guards (FindOpponent tick fold [M 0x1424e0:100-121], SendAttackMessage,
ChangeToAttackMode, the vision/comms path), so the NoOpponent 0x60 return to Attack and
NDrone2_DoorGuardHandler 0x1416d8 follow the game's pointer, not the "guard currently sees Bond"
stand-in. Read done/30-combat-sight-loss-exit.md first; pin with real-input tests on a door-guard
level (07000014 or 07000007) and a guard that loses sight of Bond.
