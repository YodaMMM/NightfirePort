# s/agent-player-moves-crouch: the game's crouch walk speed

Built on s/agent-player-moves. `Player_Update` copies the whole +0x10 move quad into the +0x20
smoothers after the move [M 0x193600:296-299; disasm 0x193cd4 `lq v0,0x10(s2)`, 0x193cdc
`sq v0,0x20(s2)`]. `Player_Move` stores the lanes already multiplied by its argument
[M 0x191dd0:166-170], and the crouch passes 0.75 [M Player_SSCrouch 0x1924c0:58]. So the game's
crouch eases on the scaled lane and settles at 0.15/0.4 = 0.375 of the 0.105 target: 0.039/tick
at full stick. The port walked 0.075/tick crouched, about twice the game's pace.

Test: `player_moves_live::the_crouch_walk_settles_at_the_scaled_lane_on_07000002`. It is red on
the old code (lane read 0.1, the unscaled cap) and green here.

**Blocker before merge.** Five `walkreal_0700000a_live` stealth-route tests go red with this
change: `bond_stands_at_the_mouth_bridge_edge` (the mission fails at the ring's west-corridor
jink 3: alarm runner 8), `walk_office_floor_qworm_keyboard_hatch` (south-west lane leg 6) and
the three `..._real_damage*` walks (Bond dies near (−14.9, 1.1, −20.7) / (−24.4, 1.1, −8.8)).
They go green again when only the crouch lane is put back, so the cause is this change alone.
The routes were timed on the port's faster crouch. Shortening the #607 crouched crossings to the
plane itself did not recover them. The routes need a re-time against the slower crouch.

All other crouching walkreal binaries were green with the change: 01, 02, 03, 05, 06, 07, 08,
0b, 0c, 11, 13, 14, 15, 16. So were hatch_crouch_live, hatch_crouch_real_live,
start_crouch_live and camera_death_crouch_live.
