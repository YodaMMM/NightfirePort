# level-11-interactables-2

Branch cut from origin/master 117f5b41 (2026-11-02 00:36 UTC). WORKLOG started fresh (master tracks no docs/WORKLOG.md; recreated per prior-task convention).

## Task
07000011: every player-touchable kind behaves like the game. Census 07000011 [D every placement kind]; for every kind a player can touch, add the live test (model: interactables_0700000b_live.rs) driving the real path (walk/E/shoot), asserting the game's result [M class Update/Activate; D params] with a sister. Fix exposed port gaps with evidence. Run interactables_07000011_live, walkreal_07000011_live, nf-game; quote totals.

## Progress (level-11-interactables-2)
- Kind→test matrix over the 07000011 census [D docs/levels/07000011-audit.md]: switches 0x29, monitors 0x30, breakables 0x20/0xe1, rigid bodies 0xd8, lifts have ZERO placements on 07000011 — nothing to test. Doors/lock/keypad/worm/grapple/TouchOnce/vent/hurt/pickups/sound/hint/icons/music/rotors already covered by the 14 tests.
- Verified the SP laws from the decompile: SP_CreateScriptPlayer 0x1db9a8:84-86 loads mode-A∉{0,1} scripts at create; SP_LoadScript 0x1dbc08's mode-8 arm Script_Plays LOOPING into state 6; SP_Update 0x1dbda8:221-229 writes the out only on a mask match; SP_Hit 0x1dc470:15-17 builds flag bit 0 from the SP's OWN out channel (mask 1 → the SP can never self-raise). Spawner law [M 0x137e68:53-62]: dormant until the trigger ch, latched started.
- Two new kind tests in interactables_07000011_live.rs: scriptplayer_0263 (plays from load, holds ch 0x38 only after the worm E, entities live) and spawners_220_and_240 (group 3 on the worm's 0x38, group 2 on the mux out 0x35 after the real touch #257 walk-in; sisters: idle before, touch out falls on leaving, mux out follows, the started spawner latches). The port matched every law — no port gap found.
- The vacuous sister at the fusebox test (|| true) replaced with a real Agent-load pin (ch 0x12 cold [D #177 p6=1; M 0x1ae1f8:67-69]) — the first attempt failed on Operative where the lock auto-unlocks at create (the test now documents that).
- Runs: interactables_07000011_live 16/16; walkreal_07000011_live 1/1; walk_07000011_live 3/3; fix_level_chain_live 5/5; nf-game 1031 passed / 0 failed / 9 ignored (100 binaries).
