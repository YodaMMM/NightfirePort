

## level-11-stairwell-bullet-wall

Branch `s/level-11-stairwell-bullet-wall` (from origin/master). Window 08:46-10:45 UTC.

Task: 07000011 stairwell — rounds at the abseiling guards stop on something invisible in the
well; decals land on the far wall. Three Bond spots given; rope guards at (18.895, 46.516,
1.434) / (19.745, 46.580, 0.363) [D]. (1) Cast the bullet path from each spot toward the rope
guard, list every triangle/object the port's collision meets (owner, placement kind, glist
flags, material/surface byte, collision class), find which one the game's bullet passes
[M Bullet_update 0x12e7b0 / Bullet_CollisionHandler 0x12ce80, the pick mask, Collide_Pick
0x1eb320 node/surface drops, pass flags surface 0x40/0x80, 0x10 render-only, glass 0x10].
(2) Fix [M] + real-input test from one spot (round reaches and damages the guard); sister:
a round into a real wall still stops.

Progress:
- (08:48) Sandbox survived (repo + work/ + ISO + rust present); branch created and pushed.
- (09:00) [M census + disasm] The blocker identified: map43#126 "stairwell_blocking_ploys, this
  may look od buts its a fudge!!!! do not change or thre game will break!" (glist 0x10, triangle
  surface byte 0x40) crosses every Bond->guard ray at t 0.05-0.10 (census test
  stairwell_bullet_census_07000011 in world.rs, --ignored). The game's flight passes it: the
  flying bullet ARMS its HITTEST pick every tick (+0x94 = 0x101/0x201, +0x9a = surface mask 0x20
  for guns without weapon+0x6c & 0x800 [M Bullet_update 0x12e7b0 disasm 0x12f030-0x12f058,
  0x12f134/0x12f154]) and Collide_Update's walk goes past every pass-flagged hit to the first
  solid one [M 0x1ea398:216-229]. The earlier "the pick never arms" reading (Bullet_init's
  0x12c668 zeroing) was the init value only — the flight re-arms per tick.
- (09:10) Fix: the player- and drone-bullet flight traces use segment_bullet (drop node/mat 0x40)
  [drone.rs update_player_bullets + bullet.rs Bullet::update]; the foliage-test premise corrected
  (its flight assertion flipped to the walk rule — the tree stop moves to the unflagged wood
  core; the bulletproof LOOK unchanged, the core's decals hide inside the canopy).
- (09:40) Tests: NEW stairwell_bullets_07000011_live 2/2 (the round from the reported spot
  reaches and damages the rope guard through the real input path; sister: a wall round stops);
  fix_foliage_bullets_live 3/3 (corrected); nf-game lib 547/547.

## level-11-stairwell-bullet-wall-2

Branch `s/level-11-stairwell-bullet-wall-2` (from origin/s/level-11-stairwell-bullet-wall
53dca022, merged with origin/master 34ac31a5 → 1ce7edf7). Window 10:20-12:20 UTC.

Task: after the stairwell flight fix (the armed pick + the pass-flag walk [M 0x12e7b0/0x1ea398])
merged with master, three tests go red, each green on master: gondola_windows_live (08 — does
the round now pass the glass it should break?), walkreal_07000001_live and
escort_follow_07000001_live (01 — which shot now reaches or misses whom?). Find the contact
each new pass rule skips (or newly stops on), fix where the port departs from [M]; walks that
depended on the old wrong blocking get played the player's way. All three + fix_foliage +
the stairwell test green; the FULL suite on the merged tree; the total line in the report.

Progress:
- (10:24) Branch created and merged; WORKLOG started.

## level-11-stairwell-bullet-wall-3

Branch `s/level-11-stairwell-bullet-wall-3` (from origin/s/level-11-stairwell-bullet-wall-2
8f5bf8c3, merged with origin/master). Window 15:24-17:24 UTC.

Task: the one red that remains: `walkreal_07000001_live::walk_from_the_spawn_to_the_exit_with_
the_contact`. The fight resolves differently, parking the contact in AllyLeadWait (0x20) at
route node 15 (6.9, 6.3, 9.1) with Bond 6.1 away; the release [M 0x15eda8 case 3: pd > -7.0
&& (pd != 0.0 || dist2d < 5.0)] reads pd == 0.0 and needs Bond within 5.0, but Bond's walk
stalls at (1.38, 6.7, 11.61) in the hall (the nav gives up at its 3000-tick cap). Find why
Bond's walk stalls there and walk it the way a player would with real input; check the release
rule against [M]. Then green + stairwell + gondola window + escort + foliage; FULL suite on
the merged tree; the total line in the report.

Progress:
- (15:26) Branch created, master merged, WORKLOG started. Fast suite starting.
