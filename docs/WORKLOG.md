# WORKLOG

## level-41-audit (branch s/level-41-audit)

Task: the 07000041 behaviour audit — every main-map placement, the level's scripts and the
mission records against the game; write docs/levels/07000041-audit.md; fix small gaps with tests,
report large gaps.

Progress:
- branch created from origin/master (7e35bd77), worklog started, pushed.
- next: census_07000041 dump (NF_ISO), per-kind research (subagents), assemble the audit table.

Progress (2):
- census_07000041.rs added + run: main map m99 = 120 placements, 35 kinds [D]. docs/levels/
  07000041-audit.md written — every placement rowed, all 35 kinds with Game [M]/Port/GAP blocks;
  the 7 NIS scripts (real set 0600004f/6f/27/44/45/52/7c4) + mission records audited (all none;
  the wrapper's dormant tail verified [D]: zero Event 0x11, zero Entity 0x0f, no 0xd9 mount).
- KEY [M] find: the creators' "+0x2c" reads are param slot 0 of the zero-defaulted param struct
  [0x1d1b68:61-76] — the level doc §6's "0x71 spawner scale-float window" claim is retracted:
  0x71 = rotor_init 0x18f4b8 (rotors), 0xf1 spawners absent. Also §6's "#82-#85 icons" → 0x2c
  lightning; real icons #105/#114 (ctx 3 always-on ~5 m from wire #26 — the "no grab" premise
  stale). Doc §1/§6 + walkreal comments corrected with cites.
- 0xf5/0xf8 share Drone_AIVolume_Create [M 0x1d04f0:496-497] — the port's dual filter is correct,
  not an over-filter (settles the level-14 audit's open question).
- Sim-side GAPs found: none on this level (the port is faithful; the cosmetic set: the 0xf4
  movie fade, 0xd8 RB physics, FX presentation). No behavioural fix needed — the fixes this
  session are the doc/comment corrections.
- suite: 822 binaries built; run in progress (the sandbox's 4 GB ceiling kills several heavy
  nf-view binaries with CRASH exit -9 — the level-12 run's known pattern).
