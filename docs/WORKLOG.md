# WORKLOG

## cite-14-skinless-drones

- (12:35 UTC, 10-24) Branch `s/cite-14-skinless-drones` created from `origin/master` (26543436).
  Task: a [D] census for the 07000014 skinless drones — which archives the 14 load path pulls in,
  which hold or lack skin 0x050000b1 (archive, entry, offsets), name the five placements; if the
  game would find the skin somewhere the port does not look, cite the [M] address and change the
  test to the game's rule; if the claim cannot be settled, mark the pin
  `missing_models_census_live.rs:224` `// [S] unverified` and name the settling PCSX2 read.
  Note: master dropped the root WORKLOG and docs/WORKLOG.md (batch-1023c); re-adding docs/WORKLOG.md
  on this branch per the current task-branch convention.
- (13:05 UTC, 10-24) [D] census done: 0x050000b1 type-3 sub-files ride ONLY 07000016.bin entry 139
  (name 050000b1, offset 0x8594a0, len 0x860, disc start 0x31870000) and 07000015.bin entry 102
  (offset 0x609e60, len 0x860, disc start 0x33782000); the 14 load path (front-end 07000048 +
  mission::LEVELS before 14 + 14, each level's 070000xx.bin type 1 and 07F000xx.bin type 0 [M
  LoaderLoad 0x1d4a00 case 0; ResetMap_Load 0x1d6360]) carries b1 nowhere; 07000014's own archive
  holds 0x0500008c/93/b2, not b1. The 11 raw b1 placements enumerated (448,449,465,468,471,472,479,
  703,704,1029,1030, all kind 7); the boot difficulty creates 5: 449, 468, 472, 703, 704.
- (13:40 UTC, 10-24) THE CLAIM FLIPPED: the pin's evidence chain was wrong. NDrone2_DefaultInit
  0x14b300:82-95 REWRITES the param-0 0x050000b1 to 0x0500008c in Drone+0xe0 on GameState+12 ==
  0x07000014 BEFORE the kind switch, the captain block and the AnimObjectNew handoff (:563-565
  reads +0xe0) — the game renders these drones from the own-archive 0x0500008c, never asks the
  table for b1. Only-writer audit: AnimLoadFile 0x1bdf40's AnimProcessSkinData 0x1bd500 is the
  single skin registration (single LoaderProcess 0x1d4880 case-3 caller). Port fixed: Drone::new +
  captain_skin apply the rewrite; scene.rs's drone_skins rebuild covers final!=raw (rewrites +
  captains); census pin skinless_drones 5 → 0 with the address; drone_skins 14-arm pins
  b1_final ∈ {8c, b2} with a non-captain 8c; models_characters airbase pins 8c characters; the
  load_scene_equality 07000014 digest re-pinned (+5 characters). nf-game 86/86 binaries ok,
  nf-view drone_skins/census/models_characters/load_scene_equality ok. The sister docs corrected
  (drone-skins.md, GAPS.md G-021, characters.md, missing-models-census.md).
- (13:58 UTC, 10-24) Left: nf-formats/nf-audio/nf-view full-crate runs + the report + finish steps.
