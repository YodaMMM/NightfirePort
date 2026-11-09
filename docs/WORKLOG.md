
## pretrans-arms-3

Window 1 (21:22-21:30 UTC, sandbox kept from level-03-guard12-door5; setup skipped). Branch
s/pretrans-arms-3 = origin/master 0a9d39435 + cherry-picked s/pretrans-arms-2 616d4b0e1 (the
+0x38 root-motion zero) as 3c6a8e503. One conflict, in the shared damage_log.rs return-fire
stand: master's fire_shield cone (opt-in) and pretrans-arms-2's return_fire_shield_blocks fold
(opt-in) landed on the same spot; both kept — the fire gate already reads `!blocked && !shield_line`.
Pushed. Next: re-run the two red walks of the -2 report on this tree (master's solid guard bodies
0a9d39435 sit under the root zero now) and fix them as a player would.
