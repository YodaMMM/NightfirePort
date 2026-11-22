# s/stream-1122-a: 01-07 walks re-tuned on the s/stream-1122 join

**Moves the shared random stream (built on s/stream-1122, which joins seven stream-moving guard
rules). Not merged to master.** No game code changed here: every joined rule is kept as is.

## What a player notices

Nothing in the game. Test walks for 01, 02, 03, 05, 06 and 07 and the 07000012 kick-node test
complete again on the joined stream.

## Cause per red test

Each joined rule was switched off one at a time (a temporary env gate, removed before commit) and
the red tests re-run.

| Test | Rule that turned it red | Fix |
|---|---|---|
| walkreal_07000001: exit fade, contact walk (plain) | fallback-goal routes pulled back from the AI boundary (AlterDestFor_DROUTE_Nearest 0x156088) | walk lever 500/60 and 100/30 |
| walkreal_07000001: contact walk real damage / Agent | the combat move choice without the 0x5E arm (ChooseCombatMove 0x150c18) | walk lever 500/200 and 1000/60 |
| walkreal_07000002: all objectives 00 Agent | AlterDest pull-back | third stop 20000/60 |
| walkreal_07000003: full route real damage / 00 Agent | combat move choice | lever 100/30; 150/30 + 20000/90 |
| walkreal_07000005: door band exit (plain) | either rule alone turns it green | lever 100/30 |
| walkreal_07000006: castle interior Agent | combat move choice | added stop 250/30 |
| walkreal_07000007: party 00 Agent | combat move choice | lever 100/30 |
| nav_kick_live: kick node pair | AlterDest pull-back | the guard's goal radius set to 0.5 |

Both rules were checked against the decompile again: ChooseCombatMove has no +0x31C read at its
head [M 0x150c18], and AlterDestFor_DROUTE_Nearest pulls the destination 1.0 back from the
boundary crossing toward the route's last node [M 0x156088]. The reds are walk timing, not
mismatches.

nav_kick_live: the probe goal (-1.0, 0.4, 17.4) is past the corridor's AI boundary, so the route
is a fallback-goal one and the destination is pulled back to about x 0.65, just past n248. With
the DefaultInit arrival radius 2.0 the guard stopped before n144 and never kicked. The test now
sets the 0.5 radius that the game's cover goals use (SetupGoalPosition(0.5, ..) [M 0x152548]). The
test still requires the KickObject entry, the ScriptPlayer leaving its wait, and exactly one kick.
Without the radius line it fails on this branch.

walkreal_07000005: the plain test now wraps its body in its own pause. The real-damage sister
calls the body directly, so its 2000/30 lever stays the same.

## Tests run (target-o, release)

- walkreal_07000001_live: 7 passed, 1 ignored
- walkreal_07000002_live: 5 passed
- walkreal_07000003_live: 7 passed, 3 ignored (probes)
- walkreal_07000005_live: 17 passed
- walkreal_07000006_live: 8 passed, 1 ignored
- walkreal_07000007_live: 6 passed
- nav_kick_live: 5 passed, 1 ignored

## Unproven

- In the joined 01 contact walk, the stairwell guards whose fallback destination gets pulled back
  stand at (-30.9, 10.6), beside the plate. That holds the contact in LeadHide until the guards
  are cleared. The decompile supports the port's pull-back, but this exact standoff has not been
  compared with the real game.
- The port's BoundsTest walks every wall, while the game walks each cel's own list [M 0x157d80],
  so a pull-back point may differ where walls cross cel borders. Not checked here.
