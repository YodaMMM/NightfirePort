## Your task: `level-12-start-damage`, the 07000012 start spot hurts Bond as hard as the game does

Bond stands still at the 07000012 start (22.49, 1.09, 19.16) from the level load, no input, difficulty 1 (0x2a3790 = 1). Health (BLData+0x894) every ~120 frames from the load [L PCSX2, three runs] vs the port (load_with_carry_difficulty(.., 1), step_bond idle):

| frame | game run 1 | game run 2 | game run 3 | port |
|---|---|---|---|---|
| 0-240 | 100 | 100 | 100/98 | 100 |
| 360 | 100 | 86 | 93 | 90 |
| 480 | 72 | 53 | 68 | 79 |
| 600 | 51 | 35 | 48 | 76 |
| 720 | 44 | 18 | 32 | 74 |
| 840 | 37 | 0 | 21 | 58 |

The game's guards land the opening hits about twice as hard or as often (frames 360-840: game -49..-86, port -32). Find which drones hurt Bond here in the port and in the game (shooter, weapon, burst length, hit-chance roll / damage per hit at difficulty 1 - e.g. breakpoint the damage path 0x1902c8 and the hit-chance 0x179740 in the game and log each event), name the arm that differs, fix it against the decompile and pin the first-14-s damage at difficulty 1 against these [L] numbers (state the tolerance; the game's spread across runs is real). Sister task: 10-level-04-start-chip-damage (there the port hurts MORE) - check whether one cause explains both.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/10-level-12-start-damage`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

---
## Main agent addendum (2026-11-12): where the hits go missing
Same per-hit sizes in both (d1, level-12 Plr_DMod_Easy 0.5): guard A = weapon 0x18, **3.5** per hit; guard B = weapon 0x19, **1.75**; rounds 6 frames apart, bursts of 3-5 [L PCSX2 per-change BLData+0x894 log: guards 0x1877240 (A, ~(13.8, 1.0, 17.5)) and 0x1876400 (B, ~(13.3, 1.0, 16.6)), states logged]. Game A hits in states **0x70, 0x74, 0x68 and 0x6f** - a burst every ~50-60 frames from frame 384 (e.g. 0x74 hits at 511/543/549/555, 0x68 hits at 597/603/609/615); 96 damage by frame 900.
Port (master a275e4a32, drone #7 = A at (13.0, 1.0, 17.9), dist 9.6): the 0x70 bursts hit almost every round (t 306-368), but **every 0x68 round misses** - three 4-round bursts at t 478-496, 606-624, 737+ fire (last_shot advances, burst counts down) and `player_hits` does not grow - and the **0x74 state never fires** (firing false, anim 3.0x5b). That is the whole gap (port 56 damage by t 900).
So: (1) why the port's AimStandFire 0x68 rounds miss Bond from the same spot the crouched 0x70 rounds hit from (the muzzle bone height vs the counter/cover geometry in front of him, the stand-fire accuracy roll DroneWeap_DoBulletAccuracy 0x179740, or the aim point) - compare a live round in PCSX2 (Bullet_init origin/dir for drone 0x1877240 in state 0x68); (2) whether 0x74 fires in the game (it does: the hits above) and what the port's 0x74 is missing.
Narrowed further (port, drone #7, `shot_rolls`): the 0x68 rounds' accuracy rolls DO come up hits (t 478 true, 484 false, 490 false, 496 true; seen_ticks 309-327 >= the 120 NewSighting gate, lost_sight false) but no hit reaches Bond - so the rolled-HIT bullet from the STANDING pose is stopped between the muzzle and Bond (the counter/cover in front of him at (13.0, 1.0, 17.9)? the muzzle datum in anim 0xa? the aim point opp_target?). Second factor: after the 0x74 stretch `seen_ticks` restarts (85 at t 606 < 120), so that burst's rolls are forced misses; the game's guard keeps hitting through 0x74 -> check what resets +0x270 in the port's 0x74 (Bond not "seen" while the guard ducks?) vs the game.
Narrowed again: the 0x68 shots fire (last_shot 479/485/491/497 advance, rolls recorded, gun_through_wall false) but produce NO BulletEvent at all - no World, no Player - while every 0x70 round ends in one (Player 7 at Bond, or World at (24.80, ~2.0, 19.2) behind him). Same weapon 0x18; the only difference is the muzzle: 0x70 crouched (14.15, 0.84, 17.46), 0x68 standing (14.03, 1.42, 17.82), drone at (13.10, 1.00, 17.41). A round that dies with no event at all looks like one born inside a collision mesh (the counter edge right in front of him?) or killed by a silent arm in Bullet_update - instrument the bullet path for owner 7 at t 478 and compare with a live PCSX2 Bullet_init/Bullet_update for drone 0x1877240 in 0x68.
FOUND where they go: every 0x68 round from #7 ends as `BulletEvent::Drone { drone: 6, part: 21, at: (14.07, 1.42, 17.83) }` - it hits guard B's body right at A's muzzle. In the port the pair stands ~0.7 u apart with B in A's line of fire (#7 (13.10, 1.00, 17.41), #6 (13.5, 1.0, 18.0)); in the game they stand ~1.0 u apart with B BESIDE/behind A's aim (A (13.8, 1.0, 17.5), B (13.3, 1.0, 16.6)) [L]. The round's pick leaves out only the shooter in both (friendly bodies block in the game too [M Collide_Update]), so the gap is the guards' POSITIONING: where each guard walks to on the alarm and the no-bunching spacing (`DroneMove_NoBunching` 0x150ad0 / the +0x314 neighbour side bits) - trace both guards' positions from frame 0 to 400 in PCSX2 vs the port and find where the port's pair ends up stacked.
The pair's paths (port, t from the load): starts match the game exactly - #6 north (13.09, 1.00, 18.55) state 0x5, #7 south (12.98, 1.00, 16.43) state 0x3, both weapon 0x19 [L game: 0x1877240 (13.1, 1.1, 18.6) 0x5, 0x1876400 (13.0, 1.1, 16.4) 0x3]. At t 180 combat starts: #7 switches to weapon 0x18 (3.5/hit) and holds (13.2, 16.9); #6 (0x68/0x73) drifts SOUTH to (13.5, 1.0, 17.95) by t 300 - beside #7's standing muzzle (14.03, 1.42, 17.82), where #7's rounds hit him. Game: the north guard ends EAST at (13.9, 0.5-1.0, 17.1-17.5) and the south one at (13.2-13.3, 16.5-16.6) [L]. Also check WHICH guard carries 0x18 in the game (the weapon switch at combat start) - read each drone's weapon field live; the per-hit sizes (3.5 vs 1.75) say one does.

---
claimed-by: main agent 2026-11-12 09:58 MST (worker daily cap: the main agent fixes this one)

---
## Main agent (2026-11-12 10:25 MST): fixed on s/12-start-pair-nobunching (b3f1ba816), suite next
Correction: live, the SOUTH guard (0x1876400) switches to weapon 0x18, as in the port (#7). The difference is DroneMove_NoBunching's +0x31C side bits: live the pair carries 0x14 (south) / 0xC (north) from frame 231; the port held the scan inert (NB_SCAN_LIVE false), so #7 stepped RIGHT (0x74) into his partner and his standing rounds hit #6. The port's scan (already a faithful copy) gives exactly 0x14 / 0xC; it is now live on 07000012 (the door-leaf-slide per-level pattern; 01 and 14 walks go red with it on, they wait). Idle Bond at d1 now 79 / 68.5 / 61.5 / 37 / 26.5 at 360..840 (game runs 86-100 / 53-72 / 35-51 / 18-44 / 0-37). Pinned: start_pair_07000012_live. The d1 walk's 543 crossing moved back to the diagonal.

---
DONE by the main agent 2026-11-12 11:30 MST: master 70555d76c, suite 4937/1 (the 1 = the 00 Agent warm chain, now ignored with its reason; re-tune on the main agent backlog).
