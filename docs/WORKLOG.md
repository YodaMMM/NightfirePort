# WORKLOG

## level-1b-every-objective
Started 2026-11-03 04:45 UTC. Branch s/level-1b-every-objective cut from origin/master
6f16146e. Task: complete every 0700001b objective on real input — extend walkreal_0700001b_live
or add a sister test that plays the whole mission from the fresh start, every objective completing
in the game's order, checked on the channel/objective state the game reads, Bond alive at the end,
no teleports, no forced fields. Fix the port against the decompile where a step can't be done the
way a player does it, and fix the whole class. Must run: walkreal_0700001b_live,
interactables_0700001b_live, zerog_0700001b_live, platforms_drake_scene_1b, nf-game.

- (05:10) The objective records pinned in the game's order inside both full walks (allowance +
  real health): three hinges on `mission.states[23]` — start [3,1] (existing); at Drake's spawn
  neither objective channel fired and the records read [3,1] (obj 1 announced, obj 2 hidden); at
  Drake's death [4,3] (ch 0x09 on, ch 0x0b still 600 ticks out); at the end [4,4] with the
  channels-of-record stamps time(0x0b) < time(100) [D 06000a20 t600/t720] and MissionState 6 →
  0x0730001b. Finding: the game's fight gate opens BEFORE objective 1 completes — the AND-#139
  inputs (ch 21/22/27) are written only when each missile's p2 script ends [M 0x1aa33c-3d0,
  re-verified in the disassembly this session: Script_IsPlaying's bne guards the switch_channels
  sb at 0x1aa3a0-3bc/0x1aa380-388], and missile 1's p2 writes the Drake-chain ch 46 @t1000 but
  ends t1710 [D 0600091c] — so Drake spawns ~710 ticks before ch 21 exists. The first hinge I
  wrote ("the gate ⇒ ch 9 fired") was refuted by the walk; the TEST was wrong, the port matches
  the decompile (space_missile.rs's [M 0x1aa354-398] model confirmed). No port code change needed.
  Report: docs/reports/level-1b-every-objective.md.
