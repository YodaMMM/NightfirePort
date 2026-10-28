# WORKLOG

## level-13-guards-start-alerted-2

Branch s/level-13-guards-start-alerted-2 (from origin/s/level-13-guards-start-alerted @ e52fac44, merged origin/master @ 08f06953). Task: the prior two AI fixes (vision-bit sight gate [M 0x176a28], EnemyRunToPoint Attack resume [M 0x168910:22-36]) turned eight tests red on 07000001/4a within seconds of level start — find what the game does at start on those levels (which states set the vision bit on 01's starting guards and the escort; whether a test leaned on the bypass), fix the port or re-derive the test with the cite, get all eight green plus the 13 start test.

## level-13-guards-start-alerted-3

Branch s/level-13-guards-start-alerted-3 (from origin/s/level-13-guards-start-alerted-2 @ 1cc9ea59, merged origin/master @ fee3d546). Task: round 2 added the vision bit to PlayScript and the abseil ENTERs (8 of the reds green) but 6 remain (escort_follow_07000001_live, walkreal_07000001_live, fix_npc_doors_live, walkreal_07000008_live, fix_pickups_ammo_live, drone_weapon_drop_live). Round 2's census: 139 state functions write `+0x4F8 |= 0x10000` on ENTER [M]; this round ports the bit on EVERY one of those ENTERs (ally/escort, combat, cover, death/drop paths), not just the ones a red test exposes, and gets all six green plus the earlier eight and the 13 start test.

### level-13-guards-start-alerted-3 — outcome

The census: 137 real `NDrone2_DSTATE_*` writers (the raw-160 grep carried three prefix false-positives: `Drone_Control` 0x139268:93 writes `|= 0x100000`, `DroneVision_AlertSound` 0x177800:70 `|= 0x10000000`, `KnockedOut_Anim` 0x172a30:21 `|= 0x1000000` — none is 0x10000). 133 set the bit on ENTER (case 1 / `if msg==1`); `Attack` 0x164210:69 sets it on a conditional ENTER arm; `AllyGoToGoalPosition` 0x15f938:20 and `BotGlobal` 0x16e060 set it on UPDATE only. State-id→function = the game's handler table [L ee_snapshot.bin @0x29b300+id·4]. Ported `vision_on = true` onto 35 more ENTER arms (the whole ally lead/follow chain 0x1c-0x26 incl. a NEW FOLLOW_WAIT arm, combat + new-sighting, surrender/stun/dart/punch/explosive recoveries, aim/reload/crouch, truck/civilian missions, open-door/kick/action/fiddle, undercover leave paths, astronaut trio, drake) [M each cite in the arm]. HeardNoiseAware left branch-conditional exactly as the game writes it [M 0x16af20:21]; Attack's ENTER set is the documented superset. Pinned as `crates/nf-game/tests/vision_bit_census.rs` (REQUIRED ~110 implemented states with per-row decomp cites; NOT_IMPLEMENTED 14 writer states the port has no arm for yet). All six reds + the earlier eight + the 13 start test green (escort_follow 1/1, fix_npc_doors 1/1, walkreal_07000001 3/3, walkreal_07000008 4/4, fix_pickups_ammo 4/4, drone_weapon_drop 3/3, guards13_start 1/1, exit_aim 1/1, interact 10/10, fix_mission 4/4, guard_first_shot 2/2, rappel_vanish 2/2); nf-game lib 578/0. The full four-crate suite did NOT fit the window this session (the first detached suite.py start died instantly on a PATH-less env — background jobs die between tool calls here; the re-run consumed the window compiling).

## vision-bit-unported-states (2026-10-28)

- Branch opened from origin/master (fcf6f334), merged origin/s/level-13-guards-start-alerted-3
  (the vision-bit census `vision_bit_census.rs` + the 35 ENTER arms live only on that lineage —
  the census must be in this tree for the NOT_IMPLEMENTED→REQUIRED row moves). Task: port the
  14 NOT_IMPLEMENTED writer states from the decompile (ENTER/UPDATE/EXIT), wire dispatch, move
  rows into the census's required table, test on levels that use them [D placements/scripts];
  AI/escort/walk families stay green.

### vision-bit-unported-states — outcome

- The census's NOT_IMPLEMENTED table corrected: the ninja fight bands 0xa8/0xa9/0xaa were
  ALREADY ported with vision_on (the census's old game-function names masked the port's
  NINJA_*_RANGE consts); NINJA_STAND 0xa6's arm was invisible to the parser (the literal
  (0xa6, msg::ENTER)) — now uses the new kiko::NINJA_STAND const and carries the game's
  ENTER vision bit [M 0x16b990:19-21]. Census REQUIRED 122 rows, NOT_IMPLEMENTED 3.
- Seven census-writer states ported from the decompile (CombatTooClose 0x5d, Obstructed
  0x61, HoldItRightThere 0x80, SeenDeadBody 0x9e, SeenSurrenderedDrone 0x9f, Prone 0x6b,
  ProneFire 0x6c) — arms, consts, impact_args/blast-group entries; id→function verified
  [L ee_snapshot @0x29b300+id·4]. SetDroneAlertRoute decoded as route-invalidate +
  goal-position build (NOT AssignAIPath) [M 0x154b20].
- Green on the final tree: census 1/0, nf-game lib 578/0/5, ai_live 37/0, escort 1/0,
  walkreal_01 3/0, walkreal_08 4/0, walkreal_14 2/0+1ign, probe_07 1/0, fix_finale 4/0,
  mp_net digest 2/0. Sweep 131/876 in two parallel foreground chunks (background runners
  die between tool calls here); bathhouse shout/voices pair FAILS on this lineage
  (audio-mixer path — needs a master bisect next window), campaign_flow_live hits the
  300 s runner cap, probe13's sweep TIMEOUT did not reproduce in isolation (8.3 s).
- Left: the level-13 reach diagnosis, CastleChatGuard1/InterogateWalk/AimStandDiscard
  (0x6a needs the dynamic +0x5a2 resume), the bathhouse bisect, the full sweep.
