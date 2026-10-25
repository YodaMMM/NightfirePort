

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
- (16:05) The stall root-caused: after the guard-15 chase took him out through the hall's north
  lane, Bond stands in the west corridor at (1.38, 6.7, 11.61) with the hall's west wall segment
  (x≈1.93, z 11.1-13.1 [D the disc's collision cels — the port's ray fan,
  walkreal_probe11::dump_07000001_west_wall]) between him and the contact parked at route node 15
  (6.9, 6.3, 9.1) in AllyLeadWait 0x20. Every close/walk-past target sits inside the hall; the
  straight-line steering wedges on the wall face (the diagonal to the walk-past point catches the
  jamb at (1.85, 11.05), t 0.29 [D rays]), the nav's own first hop is a hall-side node ACROSS the
  wall (the AI's per-path A* routes from the corridor side straight to node[6] (6.99, 11.78)),
  and the stuck ladder's sidesteps slide ALONG the wall, never the ~1.7 north to the wall's north
  edge. The openings [D rays]: the middle one (z≈7.3-10.8) is a raised sill (the corridor floor
  drops to 4.41 south of z≈10; the sill top 6.19), the north one (z≈13.3-16.5) is the lane the
  AI itself uses — path[0] node[8] (−0.39,14.28) → node[23] (3.39,14.69) → node[7] (6.88,14.62)
  [D nav nodes+links], and the AI's bounds walls seal z≈9.16 (x −12..2.29) and z≈13.07
  (x −10..6.14) west of the hall, so the north lane is the walk road.
- (16:08) Fix (test-side, the player's play): in the no-guards round, when the world segment
  Bond→contact is blocked and Bond is west of the wall (x < 2.0), walk the north lane back in
  ((0.6, 14.3) → node 23 → node 7) and close on the contact. The release [M
  NDrone2_DSTATE_AllyLeadWait 0x15eda8 case 3: no armed peer within 10 (0x41200000, flag 1) →
  pd = ObjectMissionPathDistance(5.0); pd ≤ −7.0 stays; pd == 0.0 && dist2d ≥ 5.0 stays;
  otherwise SetState(0x1d)] — checked against the decompile: the port's ALLY_LEAD_WAIT UPDATE
  arm matches exactly (drone.rs 8892-8903). Inside the hall pd != 0.0 → the release fires; the
  contact walks his route west and the run reaches the plate → ch 0x10 → level_exit 07000002.
  walkreal_07000001_live 3/3 on the merged tree (master 4c1f5ffb in).
- (16:35) Final-tree test runs (master 4c1f5ffb merged in before all of them): the priority set —
  walkreal_07000001_live 3/3, escort_follow_07000001_live, gondola_windows_live,
  zoe_gondola_realpath_live, stairwell_bullets_07000011_live, fix_foliage_bullets_live,
  walk_07000001_live — 16 passed / 0 failed (suite.py --only). The lib binaries
  (nf_game/nf_formats/nf_audio): 657 passed / 3 failed / 4 ignored on the first pass — all three
  failures (nf_game profile round_trip + delete_in, nf_formats identity rejects_missing) were
  the sandbox DISK FULL (save_to_dir(/tmp) ENOSPC; df 100%); after freeing 1.2 G all three pass
  directly → the real lib result 660/0/4. The walker-sharing slice (87 binaries, --limit 60)
  ran ~32 of them in the window left: one red —
  interactables_0700000a_live::vending_glass_breaks_under_fire ("one dart scores ONE contact
  0.1f, got {h1} dead {dead1}" at line 165) — NOT touched by this task's files (test-only
  change); the prime suspect [S] is the -2 branch's glass-drain fix (kind-2 panes no longer
  stop the round) giving the vending pane a second same-tick contact, the same class of
  re-anchor the gondola pin needed; left open for the next session with the full suite.

## level-11-stairwell-bullet-wall-4 (branch s/level-11-stairwell-bullet-wall-4) — 07000011 the remaining invisible walls in the stairwell
- (18:58 UTC) Sandbox reset → full setup redone. Branch s/level-11-stairwell-bullet-wall-4 created from
  origin/s/level-11-stairwell-bullet-wall-3 (cd9ef225, its tip) and origin/master (aa94ce02) merged in; pushed.
  Goal: after wall-3's [M Collide_Update 0x1ea398:216-229] 0x40 pass bit, bullets fired from
  Bond (22.40, 30.09, 1.42, yaw -1.505, pitch -0.035) frame 2944 into the well still trace flat planes —
  bullet holes hang mid-air (one diagonal under the stair flight, one upright across the middle of the well).
  Plan: raycast the decal planes from that spot, list every surface the port stops on (owner/placement,
  glist flags, surface byte, collision class, drawn geometry [D]), match each against the game's remaining
  pass arms [M], fix, then the real-input test + fast suite + full suite.
