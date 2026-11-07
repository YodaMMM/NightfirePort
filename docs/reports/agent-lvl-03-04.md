# agent-lvl-03-04: levels 07000003 and 07000004

Moves the shared random stream on 07000004 and 07000016 only: the assassin's states and hit rolls change.

## Fixed

| what a player notices | game | test (red on old code) |
|---|---|---|
| 04's score never moved: the story scene and the fight stayed on the opening section | `UpdateMFX_02MayhewD` 0x112bf8: marker 2 at the 0x6000082 start, 3 on the first 0x172 sound, 4 on the second or on a skip | `the_story_scene_walks_the_score_to_the_fight_section`, `skipping_the_scenes_still_reaches_the_fight_section`, `music::mayhew_d_*` |
| The assassin could not land a round beyond 3 units | `NDrone2_DefaultInit` 0x14b300:744-753 runs after the mode table read and sets +0xB4 = 0; the port kept the disc's 80, so `100 - 5*80` never passed the roll. Properties 0x35/0x36/0x3c were missing too | `the_assassin_has_her_own_init_block` |
| She stood still on the run-in instead of closing | 0x16cb70 case 2 falls through into the moving arm (anim 0x67); the port dropped the fall-through | `the_run_in_advances_on_a_direct_route` (old: 0 of 60 ticks on the anim, no movement) |
| Rounds did full damage while she flipped or swung | 0x16cdd0, 0x16cf68, 0x16d100, 0x16d298, 0x16d430 call `NDrone2_DoHitEffects` with 0.5 | `a_flip_or_a_swing_halves_the_round` (5.25 -> 2.625) |
| Inside 2 units she swung 9 times in 10 where the game backs off | 0x16bb50:48-69: Rand(10) gates the sideflip only, its miss still reaches the -5 backflip probe | `inside_the_band_she_backs_off_before_she_swings` (old 2 of 20, new 20 of 20) |
| Flips curved round to face Bond | only the somersault's case 3 calls `NDrone2_SetAngleToObj` | `a_flip_holds_its_heading` |

NoRoute's ENTER also takes the shared `+0x3B` tail [M 0x16d878:27-29].

## Checked, no change

Every placement kind, channel writer and reader, objective and scripted scene on both levels against the audits and the decompile's level-keyed sites (`grep 0x7000003/4` over all functions: tuning, watch clock, skins, grenades, reaction time, sound bank, fog, Mission_Update, the intro wrap): all ported. The rescued servants #388/#389 leave HostageHide on the next tick through their second switch (alt state 0x85, then Attack, then CivilianHiding), and their nav path 3 holds no AI point, so the cower is the read of the code.

## Test changes

- `audit_07000004_live`: the trigger is now consumed the tick it lands; the pin reads the marker-3 jump.
- `walkreal_07000004_live`: Agent and 00 Agent skip the pickup lap and fight first; the two Accuracy rows compare capped at the par [M 0x1984b0:176-179] (a run read 49 hits on 48 shots).
- Ignored with reasons: 04's 00 Agent real-damage walk (dies with her at 24) and 16's Operative real-damage walk (dies to the pit squad). Both are bot tactics against ninjas who can now hit.

## Found, not changed

- `music.rs` has no per-level half for `UpdateMFX_02MayhewA` 0x111ed8 (01), `03TowerA` 0x113558 (09), `03TowerB` 0x1137b0 (0a), `PowerStationA1` 0x113c18 (0c), `PowerStationA2` 0x113fc0 (0d): those levels get only the exit block.
- `HostageHide` 0x15cfe0 and `HostageIdle` 0x15ce60 case 3 (the `NearArmedDrone(20)` poll) are not ported; HostageIdle's bullet resume should be 0xe. On the disc only 07000002's d23 (no second tag) stays in the hide long enough to reach them.
- The ninja states are not in the `SeenAndAttacking` music gate list in `Drones::tick`; no effect on 04, it feeds `under_attack` on 16.
- Visual, left alone: Hostage ENTER plays cower 0x19 where the game calls 0x21 [M 0x15cb78 case 1]; `NDrone2_CreateNinjaEyes` 0x149600's two eye objects are not drawn.

## Unproven

No PCSX2 run: the fight's feel at the new hit rate and the marker sections by ear. 49 hits on 48 shots in one 04 walk is unexplained.
