# WORKLOG

## level-08-every-objective (2026-10-28, branch s/level-08-every-objective)

Task: make crates/nf-view/tests/walkreal_07000008_live.rs walk the WHOLE 07000008
mission from a fresh load on real input (step_bond only; no teleports past a
blocker, no channel or state pokes): every objective the level's mission data
lists [D objective records / Mission_MonitorObjectives 0x1827d8 M], in the order
a player does them, each asserted complete, ending on the level exit firing.
First list which objectives the current tests already cover; add legs for the
rest. Every blocker is a port gap: find the game's behaviour [M/D] and fix it,
or — if drawing only — list it for the main agent. Full suite; report; finish
steps.

Plan:
- Fast runner started in the background in the first minutes.
- [D] the level's objective records (07000008.bin) + [M] Mission_MonitorObjectives
  0x1827d8: the objective list and completion channels.
- Survey walkreal_07000008_live.rs: which legs exist, which objectives covered.
- Add the missing legs on real input; fix the port gaps the blockers expose.
- [L ee_start_05.bin @ 0x2a3e88] the level's mission data lists exactly TWO objectives for
  07000008: "Retrieve guidance chip from Safe Room" (ch 0x4A, enable 0x49) and "Escape with Zoe
  in gondola" (ch 0x4B, enable 0x4A). The old tests covered both but the ride leg stood in a
  ch-0x4A CHANNEL POKE for the briefcase chain and re-stood Bond next to the Hind every trigger
  pull — both anti-patterns the task forbids.
- Merged the two walk tests into walk_the_whole_mission_on_real_input (one fresh load, real
  input end to end): intro plate -> ch 0x49 (objective 1 announced) -> the fusebox laser (ch
  137) -> the safe bolts (134/135 -> 136) -> the briefcase take (ch 0x4A, objective 1 -> 4) ->
  the descent walked back down (the ambush plates 42/43/44 fire on the walk) -> the boarding
  (ch 4) -> the ride on the REAL ch 0x4A -> the cinematic (ch 5) -> the Hind shot down with the
  carry's REAL gun (the P2K 0x02: 27 tracked-aim trigger pulls, damage 350 vs hp 299) ->
  objective 2 -> 4 -> the mission hand-off (6, 0x0730008) [M 0x182c60:85-116]. 32460 ticks, no
  falls, no stuck events. The old fight's sim.bond.pos re-stands removed: the laser (0x4E) the
  walk arrives with carries no bullet contact at the route's 24-37 u — the old test's 2.5 u
  re-stand hid that; a player switches to the gun through the real weapon cycle (unlocked by
  the carried ch 0x48 [M Player_Weapon 0x1a24c8:242-275]).
- Full suite on the final tree; push; report; finish steps.
