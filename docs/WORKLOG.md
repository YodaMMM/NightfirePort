# WORKLOG

## cutscene-handback-sp-path

Branch s/cutscene-handback-sp-path (from origin/master @ fcf6f334). Task: the kind-2 (anim) event-6 hand-back landed on master via cutscene-handback-anim-prev (census 26 rows 0 VOID, the four kind-2 scenes hard), but the REAL SP runtime path still diverges on two points [L NF_SCRIPTLOG: 0x06000084 `prev kind Some(2) -> NO entity`, hand-back falls back to the player's matrix]: (1) `Scripts::update_player` loses the kind-2 stream's object slot before the event fires — find which SP-path kill/freeslot clears it and match the game; (2) the runtime `Data` lacks the cut-scene packs' anim sets (078/079/07A-000NN [D]), so `anim_obj_matrix` returns None for their sequences — merge them the way the census's `AnimWorld::for_level` does [M LoadableLoad 0x1d69e8]. Real-path test: play 07000002's SP #399/#400 and 07000003's #475 scenes and assert Bond's position after each matches the census resolution (on a floor).
