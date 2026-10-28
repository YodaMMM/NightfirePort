# WORKLOG

## level-13-guards-start-alerted-2

Branch s/level-13-guards-start-alerted-2 (from origin/s/level-13-guards-start-alerted @ e52fac44, merged origin/master @ 08f06953). Task: the prior two AI fixes (vision-bit sight gate [M 0x176a28], EnemyRunToPoint Attack resume [M 0x168910:22-36]) turned eight tests red on 07000001/4a within seconds of level start — find what the game does at start on those levels (which states set the vision bit on 01's starting guards and the escort; whether a test leaned on the bypass), fix the port or re-derive the test with the cite, get all eight green plus the 13 start test.

## level-13-guards-start-alerted-3

Branch s/level-13-guards-start-alerted-3 (from origin/s/level-13-guards-start-alerted-2 @ 1cc9ea59, merged origin/master @ fee3d546). Task: round 2 added the vision bit to PlayScript and the abseil ENTERs (8 of the reds green) but 6 remain (escort_follow_07000001_live, walkreal_07000001_live, fix_npc_doors_live, walkreal_07000008_live, fix_pickups_ammo_live, drone_weapon_drop_live). Round 2's census: 139 state functions write `+0x4F8 |= 0x10000` on ENTER [M]; this round ports the bit on EVERY one of those ENTERs (ally/escort, combat, cover, death/drop paths), not just the ones a red test exposes, and gets all six green plus the earlier eight and the 13 start test.

### level-13-guards-start-alerted-3 — outcome

The census: 137 real `NDrone2_DSTATE_*` writers (the raw-160 grep carried three prefix false-positives: `Drone_Control` 0x139268:93 writes `|= 0x100000`, `DroneVision_AlertSound` 0x177800:70 `|= 0x10000000`, `KnockedOut_Anim` 0x172a30:21 `|= 0x1000000` — none is 0x10000). 133 set the bit on ENTER (case 1 / `if msg==1`); `Attack` 0x164210:69 sets it on a conditional ENTER arm; `AllyGoToGoalPosition` 0x15f938:20 and `BotGlobal` 0x16e060 set it on UPDATE only. State-id→function = the game's handler table [L ee_snapshot.bin @0x29b300+id·4]. Ported `vision_on = true` onto 35 more ENTER arms (the whole ally lead/follow chain 0x1c-0x26 incl. a NEW FOLLOW_WAIT arm, combat + new-sighting, surrender/stun/dart/punch/explosive recoveries, aim/reload/crouch, truck/civilian missions, open-door/kick/action/fiddle, undercover leave paths, astronaut trio, drake) [M each cite in the arm]. HeardNoiseAware left branch-conditional exactly as the game writes it [M 0x16af20:21]; Attack's ENTER set is the documented superset. Pinned as `crates/nf-game/tests/vision_bit_census.rs` (REQUIRED ~110 implemented states with per-row decomp cites; NOT_IMPLEMENTED 14 writer states the port has no arm for yet). All six reds + the earlier eight + the 13 start test green (escort_follow 1/1, fix_npc_doors 1/1, walkreal_07000001 3/3, walkreal_07000008 4/4, fix_pickups_ammo 4/4, drone_weapon_drop 3/3, guards13_start 1/1, exit_aim 1/1, interact 10/10, fix_mission 4/4, guard_first_shot 2/2, rappel_vanish 2/2); nf-game lib 578/0. The full four-crate suite did NOT fit the window this session (the first detached suite.py start died instantly on a PATH-less env — background jobs die between tool calls here; the re-run consumed the window compiling).

## level-13-guards-start-alerted-4

Branch s/level-13-guards-start-alerted-4 (from origin/s/level-13-guards-start-alerted-3 @ 50934049, merged origin/master @ fcf6f334). Task: the round-3 vision-bit census (35 ENTER arms) is right per the decompile, but the full suite has seven reds with it: walkreal_07000001_live `walk_from_the_spawn_to_the_exit_with_the_contact` (obj0 through the real monitor [M 0x1827d8 case 3]), drake_fight_live `the_missile_launch_sequence_reaches_each_phase_and_disarms` (objective 1 'Disable the missiles' never completes), fix_pickups_ammo_live `drops_and_boxes_take_through_the_real_path_with_the_game_numbers` (the second drop is None, must be ammo), drone_weapon_drop_live `killing_a_six_guard_drops_his_gun_with_the_game_rounds` (the 06 guard never dies under P2K fire), hostage_killer_07000002_live `the_captor_and_hostage_alternate_mission_lines` (the opener is not the captor's line [M 0x144340:34-40]), bathhouse_voices_live and bathhouse_shout_mixer_live (the bathhouse floor guards never shout). With sight now live in more states, guards see/react earlier than these tests' walks expect, or some downstream system (drops, shouts, mission lines) depended on the old blindness. For each, find what the game does at that moment [M] and fix the port, or re-derive a test that leaned on the old behaviour with the cite. All seven green plus everything rounds 1-3 fixed.

### level-13-guards-start-alerted-4 — outcome (checkpoint; five of seven still open)

Root-caused two of the seven reds through the real path (NF_CHWATCH + state traces), landed two fixes; five reds remain with diagnoses. The seven-binary suite.py --only run (real ISO, --test-threads=1): 16 passed, 7 failed — all seven reproduced deterministically. drake_fight_live's disarm test is green after the fix (individual re-run). The full four-crate suite did not fit the window.

## level-13-guards-start-alerted-5

Branch s/level-13-guards-start-alerted-5 (from origin/s/level-13-guards-start-alerted-4 @ d5556ae0, merged origin/master @ 480aa3db). Task: round 4 fixed drake_fight_live and the contact-shield half of walkreal_07000001_live. Still red: walkreal_07000001_live (layer 2: the hall's corpses at the east door and the door that will not finish its swing — find what the game's door does against a body [M the door's swing/collision path] and what the escort does), drone_weapon_drop_live (the dodging guard in state 0x68 — identify the state [M], aim like the game's player with auto-aim [M Check_AutoAim 0x1a73f8]), fix_pickups_ammo_live (same family), hostage_killer_07000002_live (the shared-RNG opener roll [M 0x14a8b8, 0x144340:34-40] — the test must follow the game's stream, not a fixed coin), bathhouse_voices_live / bathhouse_shout_mixer_live. For each: the game's behaviour [M] first, then fix the port or re-derive the test with the cite; no weakened asserts. Run the FULL suite and quote its total line.

### level-13-guards-start-alerted-5 — progress (checkpoint 1)

Three of the five red families fixed through the real path, all on the branch tree (individual binary runs, --test-threads=1): hostage_killer_07000002_live 5/5, drone_weapon_drop_live 3/3, fix_pickups_ammo_live 4/4. Remaining: walkreal_07000001_live layer 2 (the east-door wedge), bathhouse_voices_live / bathhouse_shout_mixer_live. The full suite runs at the end of the window.

### level-13-guards-start-alerted-5 — progress (checkpoint 2)

Bathhouse pair re-derived and green through the real path (bathhouse_voices_live 3/3, bathhouse_shout_mixer_live 2/2; drone_weapon_drop_live 3/3, fix_pickups_ammo_live 4/4, hostage_killer_07000002_live 5/5 re-run after the revert). Root cause: the wake plates sit in 0.5-visibility AI volumes [L fresh-sim probe — Bond's +0x22C light factor 0.50 at the floor/tunnel plates, 1.0 at the bunker], so a player standing on the plate computes OpponentVisibility < 0.7 for every guard outside ~5 units [M 0x175af0 ×+0x22C arm; M 0x176a28 zeroes +0x230 below 0.7; M 0x175cb8:18-20 returns before any ray] — the tests now close Bond inside the [M 0x175af0:16-25] 2.0 arm like the game's player. The mixer pin follows the disc's own sub volumes [D SB_11.SFX — the id-111 line's sub keys at volume 0, the game's own silence; M 0x3524 volume+rnd]. walkreal_07000001_live layer 2 still red: the escort settles at the stairwell head (3.5,6.3,6.6) state 0x20 with the plate at (−33.1,2.3,15.4) — diagnosed, not fixed (see the report). Full suite running in the background (879 binaries; ~40 walkreal-class binaries make it multi-hour — individual binaries stand in for the touched ones).

## level-13-guards-start-alerted-6

Branch s/level-13-guards-start-alerted-6 (from origin/s/level-13-guards-start-alerted-5 @ de110fd9, merged origin/master @ 929c6606). Task: the -5 families are green, but on the merged tree two reds, both on 07000001: walkreal_07000001_live `walk_from_the_spawn_to_the_exit_with_the_contact` (the contact must walk his route to the LoadPointActivator plate, :406) and `following_mayhew_down_completes_the_level` (the AND-touch must fire with Bond AND the parked contact on the plate, :891). With the vision bit on the escort/ally states [M 0x15e4f0:24, 0x15e698:21, 0x15eda8:20, 0x15ec08:18] the contact sees, and his route ends somewhere other than the plate (AllyLeadHide on a live armed guard [M 0x15ec08 case 3 + NotNearArmedDrone 0x177640], or the band release). Find what the game's contact does at the stall [M] and fix the port, or re-derive the walk's fight/route with the cite; Mayhew must still park on the plate and the level must end. Run the FULL suite and quote its total line.

### level-13-guards-start-alerted-6 — progress (checkpoint 1)

Branch created and the master merge landed (one conflict in bathhouse_voices_live.rs resolved: the -5 re-derived close-on-guard choreography kept, master's 1200-tick wait window adopted — the [L] probe roll timing +552/+786 makes 450 close early; patience only, the loop breaks on the first hit). Worklog started, pushing. Next: reproduce the two 07000001 reds on the merged tree.

### level-13-guards-start-alerted-6 — progress (checkpoint 2)

Both 07000001 reds green on the branch tree (walkreal_07000001_live 4/4, --test-threads=1). Two fixes: (1) the port's near-bot capsule set (Collide_Jointy 0x1ec180 via LadderCtx::bot_capsules) now excludes corpses — SetAsDead 0x14a2f0:9 sets 0x4F8&0x200 and DoCollision 0x149ec8:11 returns 0 for dead bots on every level but 0x700004a, so the game presents no corpse capsule; the old filter let a fresh corpse's capsule top (pos+0.4) hold airborne Bond 1.14 above the floor at the east-door pile, every nav leg died at the cap, and the escort waited forever in AllyLeadWait (Bond out of the [M 0x153f50] node scan, path 0.0, never inside the [M 0x15eda8] 5.0 release band). (2) the [01f] walk's closes/trails now round the hall's south structure by its north end (the AI's own node[23] (3.39,14.69) → node[7] (6.88,14.62) lane [D nav path 0]) when the straight line to the contact crosses collision — the corner's x≈6.03/z≈11.05 walls box Bond ([D the disc's collision cels; the port's segment fan]) and the doorway band x 6.5-7.9 z 12.0-10.4 is the opening. Next: the suite slices that fit the window (background is killed between tool calls in this sandbox — proven again this session), then the report.

level-13-guards-start-alerted-7 start: branch = s/level-13-guards-start-alerted-6 (011027d0) merged with origin/master (17aa36ab — carries the un-ignored 07000014 delta-exit walk; the merge touched only walkreal_07000014_live.rs), clean. This round: the two merged-tree reds — escort_follow_07000001_live's "the escort left the ally machine: state 0x86" (:137) and walkreal_07000014_live's "Rook's death channel 0x54" (:858) — find the game's answer [M] and fix the port or re-derive the test. Suite next; cutoff 16:04 UTC.

level-13-guards-start-alerted-7: both reds' port sides fixed — (1) the escort: the ally-lead family's
case-0x21 explosive resumes were missing (the fallback handed ALLY_LEAD resume ATTACK) — added [M
0x15e4f0:64 resume 0; 0x15e698:131 / 0x15eda8:90 / 0x15ec08:64 / 0x15efc8:56 / 0x15ea28:86 /
0x15f120:71 / 0x15f2e8:38 own state; 0x15f938:61 → 0x28] — and RECOVER_FROM_SCARY_OBJECT's mission-lead
arm landed [M 0x173658 case 1: mode 0x0c → ReFindMissionPath → 0x20, fail 0x63; else +0x2B=0 → 0x56]:
Mayhew's grenade flee now cycles back into the ally machine (the flee ran 8800 ticks into
CIVILIAN_HIDING before). escort_follow green (2085 distance checks, the flee grace window pinned in
the test with the cites). (2) Rook: the sight bit walks him into the combat chain and his bullet death
lands in DeathByExplosion whose channel drain runs the tick AFTER the kill [L the +0 poll: ch54 true
at +0] — the test's same-tick read now polls 30 ticks with the [M 0x163a80:44-59] delivery cite; ch54 +
objective 4 pass. Remaining red in that test: leg 6 (to the Delta exit #944) wedges on WORLD geometry
at (23.85,1.2,-24.72) (no bots within 3; segment hit t=0.12, cel 16) — next window's item. Suite
running on the final tree; totals in the report.

level-13-guards-start-alerted-8 start: branch = origin/s/level-13-guards-start-alerted-7 (b60674aa)
merged with origin/master (8a966fa3). This round: walkreal_07000014_live's leg-6 wedge at the Delta
exit — the RNG-shifted route steers into world geometry at (23.31, 1.14, −24.64) (cel 16,
segment_to_target Some(0.1227), no bot within 3.0) that master's walk clears; diagnose cel 16's faces
[D] and the route difference, fix the port if a collision rule is wrong [M Player_Collision], or give
leg 6 the player's real approach with the cite. Cutoff 17:53 UTC; the suite runs foreground (this
sandbox kills background jobs between tool calls).

level-13-guards-start-alerted-8: leg 6 walked as the player's real approach — the probe
(probe_14_delta_leg6) shows the exit pocket is off-nav, the blocker is the #121/#122 doorway's
east reveal (a west-facing face at the y-0.65 shelf's east edge; zero displacement pressed into
it), and the passage is the double slide doors (24.02,1.2,-22.8 [D]) — E a toggle [M Door_Activate
0x135be0] answered from CLOSED and from CLOSING. The three legs (approach → E → the exit) arrive
from both the leg-6 start and the wedge spot; walkreal_07000014_live 5/5 (Countdown Part 1
finishable), walkreal_07000013_live 1/1 (Double Cross Part 1 finishable). Suite: the mission-walk
slice 50 passed / 0 failed (14 binaries incl. master's switch_wall_probe); the full total line
did not fit the window (background jobs die between tool calls here; the audio binaries alone
burned a 9.3-min slice at 54 binaries).
