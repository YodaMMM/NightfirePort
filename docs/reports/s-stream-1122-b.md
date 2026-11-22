# s/stream-1122-b: walk re-pins for the stream batch 1122 joins (08, 0b, 0c, 0d, 15)

Base: s/stream-1122 (21d022502). Moves the shared random stream (inherited from the joins); no code change here, test levers only.

## Cause per red walk (one joined rule reverted at a time, local env switch, not committed)

| walk | cause | fix |
|---|---|---|
| 07000008 real damage Agent | ChooseCombatMove no 0x5E on scan bit 2 [M 0x150c18: no +0x31C read] | stop 1000/60 -> 2000/30 (12-value sweep: 2000/30, 8000/60 complete) |
| 0700000c real damage Agent | the same ChooseCombatMove rule | stop 2000/60 -> 5000/60 (12-value sweep: 5000/60, 7000/60, 10000/60 complete) |
| 0700000d arrival real damage | AlterDestFor_DROUTE_Nearest pull-back [M 0x156088, called on CalcRoute == 1 from 0x15aee8 / 0x15afe0] | first stop 1500/90 -> 1500/180, end health 83.2 (24-run grid) |
| 07000015 Agent | the same pull-back rule | stop 4000/90 -> 2000/30, the Operative walk's value (13-value sweep) |
| 07000015 real damage Agent | the ChooseCombatMove rule | stop 4000/45 -> 1000/60, exit at 72.9 health (12-value sweep) |
| 0700000b real damage Agent | none: green on this tree with no change (base, every single-rule revert but the beam touch one, and the whole binary) | none |

With every joined rule reverted all six pass; with only the pull-back and the combat-move rule reverted all six pass;
every other single revert (Mayhew nav re-find, beam touch, the 2 s timers, fiddle, PlayScript impact, CombatNoMove and AimStandFire gates) leaves the reds red.
The two causing rules match the decompile as read (ChooseCombatMove has no +0x31C read; AlterDest moves the destination 1 u back from the MoveTestFromNode boundary hit and both CalcRouteTo* callers call it on result 1), so they are kept.

## Tests (target-o, release)
walkreal_07000008_live 6 ok 1 ignored; walkreal_0700000b_live 8 ok; walkreal_0700000c_live 6 ok; walkreal_0700000d_live 2 ok 2 ignored (ignored on the base too); walkreal_07000015_live 9 ok.
Each old lever fails on this tree (the sweep's first value) and each new one passes.

## Unproven
- The pull-back's port leaves out the route flag 0x40 and the dest cel update (FindCel) the game writes, and the 7th BoundsTest argument (a char out flag that zeroes MoveTestFromNode's result) is not visible in the decompile. Whether that changes any walk is not measured.
