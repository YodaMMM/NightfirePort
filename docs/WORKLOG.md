# WORKLOG

## level-13-chain-difficulty (branch s/level-13-chain-difficulty)

- start (master 9f201030, 2026-11-01 ~10:38 UTC). Task: the 07000013 walk chain plays 07000012 at difficulty 3
  and 07000013 at difficulty 1, so Bond arrives at the 13 lobby with 43-50 health and dies on real damage. A
  player keeps one difficulty through a mission. Make the chain carry one difficulty end to end (the warm save
  carries GameState+0x40 [M the save/restore path]), check the 13 lobby fight on real damage at the default
  difficulty, fix the walker if Bond dies to tactics (never game code without a cited difference), and add a
  real-damage variant for 13. Run walkreal_07000012_live, walkreal_07000013_live and nf-game.
- Fresh sandbox: repo cloned, work/ and ISO being restored.
- The difficulty carry landed: [M] GameState+0x28 (0x2a3790) has exactly five writers and none
  runs in an on-foot level change (whole-decompile grep); the port re-seeded 1 per warm load.
  RamSave.difficulty + with_difficulty + load_with_carry seeding + warm_exit_save + bugreport
  entry.sav v3. The chain tests attach it; the 13 load pins difficulty 3.
- The 13 test factored into a shared impl; a real-damage variant (ignored, reason + live checks
  in the doc) runs the chain with the BodyArmour #414 detour. Walker tactic fixes (test helpers
  only): switch_to_armed no longer accepts a gadget as armed (the Q-Decryptor's clip 500), the
  shooters windows 7.5 vertical / 48 horizontal, dry resupply with a re-entrancy guard,
  damage-weighted fight targets, the 3.0 back-off opt-in for the real-damage chain. 12 now
  completes on real damage with 78.0 health; the variant dies at the glass band under two rifle
  guards whose boxes exposed_part never exposes (next session: the cover lane or the LOS check).
- Totals: walkreal_07000012_live 1 passed (7.6s); walkreal_07000013_live 1 passed 1 ignored
  (10.3s); nf-game 100/100 binaries ok. Report: docs/reports/level-13-chain-difficulty.md.
