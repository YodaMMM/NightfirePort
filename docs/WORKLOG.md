# WORKLOG

## level-13-chain-difficulty (branch s/level-13-chain-difficulty)

- start (master 9f201030, 2026-11-01 ~10:38 UTC). Task: the 07000013 walk chain plays 07000012 at difficulty 3
  and 07000013 at difficulty 1, so Bond arrives at the 13 lobby with 43-50 health and dies on real damage. A
  player keeps one difficulty through a mission. Make the chain carry one difficulty end to end (the warm save
  carries GameState+0x40 [M the save/restore path]), check the 13 lobby fight on real damage at the default
  difficulty, fix the walker if Bond dies to tactics (never game code without a cited difference), and add a
  real-damage variant for 13. Run walkreal_07000012_live, walkreal_07000013_live and nf-game.
- Fresh sandbox: repo cloned, work/ and ISO being restored.
