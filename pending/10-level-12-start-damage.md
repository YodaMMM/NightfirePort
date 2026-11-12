## Your task: `level-12-start-damage`, the 07000012 start spot hurts Bond as hard as the game does

Bond stands still at the 07000012 start (22.49, 1.09, 19.16) from the level load, no input, difficulty 1 (0x2a3790 = 1). Health (BLData+0x894) every ~120 frames from the load [L PCSX2, three runs] vs the port (load_with_carry_difficulty(.., 1), step_bond idle):

| frame | game run 1 | game run 2 | game run 3 | port |
|---|---|---|---|---|
| 0-240 | 100 | 100 | 100/98 | 100 |
| 360 | 100 | 86 | 93 | 90 |
| 480 | 72 | 53 | 68 | 79 |
| 600 | 51 | 35 | 48 | 76 |
| 720 | 44 | 18 | 32 | 74 |
| 840 | 37 | 0 | 21 | 58 |

The game's guards land the opening hits about twice as hard or as often (frames 360-840: game -49..-86, port -32). Find which drones hurt Bond here in the port and in the game (shooter, weapon, burst length, hit-chance roll / damage per hit at difficulty 1 - e.g. breakpoint the damage path 0x1902c8 and the hit-chance 0x179740 in the game and log each event), name the arm that differs, fix it against the decompile and pin the first-14-s damage at difficulty 1 against these [L] numbers (state the tolerance; the game's spread across runs is real). Sister task: 10-level-04-start-chip-damage (there the port hurts MORE) - check whether one cause explains both.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/10-level-12-start-damage`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

---
## Main agent addendum (2026-11-12): where the hits go missing
Same per-hit sizes in both (d1, level-12 Plr_DMod_Easy 0.5): guard A = weapon 0x18, **3.5** per hit; guard B = weapon 0x19, **1.75**; rounds 6 frames apart, bursts of 3-5 [L PCSX2 per-change BLData+0x894 log: guards 0x1877240 (A, ~(13.8, 1.0, 17.5)) and 0x1876400 (B, ~(13.3, 1.0, 16.6)), states logged]. Game A hits in states **0x70, 0x74, 0x68 and 0x6f** - a burst every ~50-60 frames from frame 384 (e.g. 0x74 hits at 511/543/549/555, 0x68 hits at 597/603/609/615); 96 damage by frame 900.
Port (master a275e4a32, drone #7 = A at (13.0, 1.0, 17.9), dist 9.6): the 0x70 bursts hit almost every round (t 306-368), but **every 0x68 round misses** - three 4-round bursts at t 478-496, 606-624, 737+ fire (last_shot advances, burst counts down) and `player_hits` does not grow - and the **0x74 state never fires** (firing false, anim 3.0x5b). That is the whole gap (port 56 damage by t 900).
So: (1) why the port's AimStandFire 0x68 rounds miss Bond from the same spot the crouched 0x70 rounds hit from (the muzzle bone height vs the counter/cover geometry in front of him, the stand-fire accuracy roll DroneWeap_DoBulletAccuracy 0x179740, or the aim point) - compare a live round in PCSX2 (Bullet_init origin/dir for drone 0x1877240 in state 0x68); (2) whether 0x74 fires in the game (it does: the hits above) and what the port's 0x74 is missing.
