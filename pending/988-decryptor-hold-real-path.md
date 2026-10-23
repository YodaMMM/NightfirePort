## Your task: `decryptor-hold-real-path`, the decryptor must scan and unlock when held (REPEAT report: 700u1 did not fix it)

User playtest 2026-10-22, on the playtest build at master 6bbf81b2:
- check decrypt12m (07000012, `--bond 7.0,0.5,2.4,3.1416,0 --give 0x56`, aim at the red keypad lock beside the door, HOLD fire): "doesnt work, holding doesnt continue the animation."
- check vent14r (07000014): "decryptor doesnt work".
Task 700u1-decryptor-gadget said it fixed this. It did not, on the real input path.

1. **First write a real-path test that FAILS on master**: load 07000012, place Bond as above, equip 0x56 through the real switch, then HOLD the fire input through `step_bond` for the game's scan duration (no direct state pokes). Assert the view-model anim keeps advancing while held, the digits scan, and the lock's channel opens. Report the failure you see.
2. Find why holding stops the animation: the hold vs press handling of the decryptor in `Player_Weapon`, the gadget's fire script, the lock's scan state [M, cite each address].
3. Fix it, then pass the same test on 07000014's decryptor lock too (find the placement [D]).
