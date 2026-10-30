# WORKLOG

## level-14-no-route-hold (branch s/level-14-no-route-hold)

- start (master 61c7e3ea). Task: 07000014's door guards hold COMBAT_NO_ROUTE 0x5f in the game (17 live samples CUR 0x5f/PREV 0x5a, grace 0 [L 2026-10-30]); the port's #25 cycles 0x5f -> 0x73/0x74/0x5c every ~90 ticks. Find why the port's NDrone2_ChooseCombatMove mirror picks the step-aims with no los/no seen and what the game's returns (0/0x67/0x6f keep the state [M 0x164c08 case 3]), fix it, extend door_guards_locked_door_07000014_live to pin the 0x5f hold; run every guard/drone/combat + walkreal binary.
