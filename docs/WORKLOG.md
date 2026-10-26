# WORKLOG

## level-14-audit (branch s/level-14-audit)

Task: the 07000014 behaviour audit — every main-map placement, the level's scripts and the
mission records against the game; write docs/levels/07000014-audit.md; fix small gaps with tests,
report large gaps.

Progress:
- branch created from origin/master (7e35bd77), worklog started, pushed.
- next: run census_07000014.rs dump (NF_ISO), assemble the per-kind audit table.

Progress (2):
- census_07000014 --ignored run: 1034 m54 rows [D]; docs/levels/07000014-audit.md written — every
  placement rowed under its kind (35 kinds), each kind with the Game [M] chain (0x1d04f0 cases +
  create/update/use addresses), the Port side and a GAP verdict; the 10 NIS scripts (101 streams)
  + mission records audited (all none).
- Kind corrections: 0x41 = InverterTrigger_Create 0x1b3040, 0xf9 = SoundTrigger_Create 0x1b2628
  [M 0x1d04f0]; census counts 0x29=2 (both p4=1), 0x2e=3, 0x31=11 resolve the doc §1b
  discrepancies.
- FIXED with a test: the weather env takes the LAST 0xDF placement (Env_Create 0x17b968 stores
  GlobalEnv + InitDrops on every call; m54's winner is #588 Bubble_gen p0=2, the port read #425
  Weather p0=0xff). drone.rs .rev().find + Drones::pending_env_drop_type accessor +
  weather_env_07000014_live.rs (run green).
- Large gaps reported (own tasks): the 0x30 monitor feed draw, the 0x28 sensor meshes/SFX, the
  Lock_ReLock caller + key-ring clear. Four live checks listed in the report.
- suite restarted 09:59 UTC after two dead starts (PATH, then ALSA pkgconfig); total pending.
