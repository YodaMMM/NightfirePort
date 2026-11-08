# agent-01-sight: 07000001 guards beyond the paper screens

Question: do the guards near (6.95, 6.34, 2.20), (6.98, 6.34, 8.48) and (5.42, 6.34, 14.44) see Bond
standing at (11.43, 6.47, -3.51)?

Game [L live-1108 section 6]: fresh load, intro skipped, Bond placed there and held 600 frames; all
three stayed in state 0x01 (WaitSwitch) every frame.

Port (tests/sight_screens_07000001_live.rs): same load, intro skipped through the skip request, same
placement via `Bond::stand_at`, 600 ticks, each guard's state recorded every tick. Drones #16, #10
and #6 sit at exactly the game's positions and stay 0x01 all 600 ticks. Port matches; no fix.

Extra readings (diagnostic, not pinned): the three are not enabled yet (switch channels 0x5, 0x5,
0xe), so this spot does not exercise their sight at all. The port's guard-vision line from each
guard's head to Bond's head is blocked (hit fractions 0.20, 0.22, 0.08).

Unproven: whether these guards see through the screens once their channel enables them; the game
run did not cover that state.
