# WORKLOG

## level-gates-world

- (20:12) branch `s/level-gates-world` cut from origin/master c70343b3a. Task: sweep all 127 `GameState._12_4_`-vs-level-code comparisons in the decompile; for my set (Destroy_*, Break_*, Door_*, Car_*, spawners, scripts, level setup, *Talk*) report PORTED (with port location) or MISSING per branch; fix each MISSING with a real-level test proved red on old code.

- (21:55) the sweep landed: 90+ `_12_4_` branches over 40 non-drone functions read and
  classified (report: docs/reports/level-gates-world.md). Three fixes shipped —
  Player_Move's 07000007 ×0.9 party walk lock (red-proved), AttackTalk case 0x11
  (0x93+Rand(2) 1-in-3 on 07000007, walkreal_07000007 6/6 with it), and the inverted
  first-wave spawner stinger removed (the game's gate is waves_completed != 0; the sim site
  drew rand(2) in a phase the game never draws). AttackTalk 5/6 + 7 implemented then
  bisected OUT: the faithful arms shift the 0d/15/16 RNG streams and kill pinned
  real-health legs (evidence + PCSX2 live checks in the report). nf-game lib 688/688;
  every touched level's walkreal green (07 6/6, 0c 3/3+1/1, 0d 2/2, 14 9/9, 15 6/6,
  16 14/14, soldiers_0c 2/2, spawner_kill 9/9). Left: Destroy_Smash's named-target stat
  gate (needs name-table resolution), the View_DrawSky wiring (visual), the 5/6+7
  unblocking via the main agent's debugger.
