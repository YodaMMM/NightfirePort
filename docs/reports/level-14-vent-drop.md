# level-14-vent-drop — the vent drop carries Bond into the facility

Branch `s/level-14-vent-drop` from `origin/master` (08f06953).

## WORKLOG entry

level-14-vent-drop (2026-10-27): branch s/level-14-vent-drop from origin/master. Task: the
walkreal_07000014_live::walk_the_infiltration_to_the_delta_exit stop at leg 2 — the SP #276
(0x06000833) vent-drop cut-scene runs (ch 0x47 written) but the port never moves Bond off the
mezzanine (41.88, 2.79, 39.03) into the facility; the south wall (z 38.21, top 3.5-3.7) is not
jumpable [M apex 0.78] and nav is None. Plan:

1. Read the walk test and its current blocker comment.
2. Decode SP pack 0x06000833 streams/events [D]: look for player ops (SetPosRot, Player_Enable
   hand-back) and who consumes the #301 touch out channel 0xd9.
3. Read the decompile: Script_EventHandler 0x1d94c0 case 6, Script_Free 0x1da880:44-48, the
   script player ops [M].
4. Port the move (evidence-tagged), keep 07000012/14 tests green.
5. Continue legs 2..6 on real input; un-ignore when green, else re-ignore with the exact
   blocker.
6. Full suite on the final tree (merged with origin/master); report the total line.

## What's done

(in progress)
