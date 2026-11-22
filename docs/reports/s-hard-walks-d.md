# s/hard-walks-d

Scope: the ignored walks on 01, 06, 0a and 16. The 13, 1b and 0d walks went to other branches, and this one does not touch them.

## 0a `the_corner_crosses_like_the_game`: route, now green
- The ignore note was stale. On current master the walk reaches the corner. It then wedges at (-17.874, 4.872), where y climbs to 1.18-1.23 and Bond leaves the ground.
- This is not the port's corner collision. Bond is placed at the game's spot (-17.87, 1.12, 4.87), facing yaw 2.987, and held forward for 60 frames. He ends at (-17.9712, 1.1144, -0.4195). The game ends at (-17.9715, 1.1159, -0.4162) [L 2026-10-26]. Starting heights 1.035, 1.07 and 1.10 also cross.
- The cause is the walk's own door. It opens the west office door on the way in, and the open leaf (pos01 0) covers the corner. The same placed hold wedges at (-17.874, 4.872) once the door has been opened.
- The fix: the walk steps east off the leaf, waits for the auto-close (360 quiet ticks [M Door_Update 0x1363c8]), then walks the line.
- The walked exit-y pin now checks how much y changes during the crossing. The walk enters at the height the door passage leaves (1.035).
- New pin: collide_push_live `the_corner_line_matches_the_game`, the placed hold against the game's numbers.

## 01 00 Agent: route
- Why Bond died to #12 in the ch-0xF pocket: the walker's cylinder friendly-fire shield, not sight. The contact settles in the 1-u lane between Bond and #12, at (-30.24, 13.99). The cylinder blocked every tap for the whole stand while #12 stepped out beside the contact.
- With the head pick, the P2K's spread put every round into the lane's east jamb. The eye-to-box ray read clear, but the round stopped at t 0.68-0.92 of its segment.
- Changed in the lane: the hit-box shield and torso aim. Bond now leaves the pocket at 57.4 health.
- Still red: only #17 dies. The mouth rounds then hold on the walled stairwell #2, and the contact dies in the pocket at t50960, which fails the mission. Ignore note updated.
- No port difference found.

## 06 and 16 (stream batch 1121b): not reworked
Their notes already carry the game's rules: 16 has the glass pass [M Bullet_update 0x12e7b0, Collide_Update 0x1ea398] and 06 has the dealers' clear world line. Both were swept hundreds of times. This branch found no new port difference.
