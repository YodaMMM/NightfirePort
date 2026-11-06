# Aim: missile timing first, then the last routes

Branch from `origin/s/aim-route-3`, merge current `origin/master`, push to `s/aim-route-4`. Read docs/reports/aim-route-3.md.

1. Fix the missile countdown. Already read: `SpaceMissile_Update` state 0 at 0x1aa1f0-0x1aa1f4 loads 0x30d0dc, `REC_FRAME_RATE` = 1/60, so `COUNTDOWN_STEP` 512.0 in space_missile.rs is wrong and the seeds are seconds, as that file's own note says. All eight missiles start at once in the port and staggered in the game. Set it to 1/60, in its own commit.
   With that change on master, 1b goes 15 passed / 2 failed: both Drake walks stop at the console latch assert (walkreal_0700001b_live.rs:644). Re-pace them: read the true deadlines (launch + seed x 60 + P0 length) and the console order they imply. Not proven live: a raw channel 19 write in PCSX2 started nothing in 118 s, so if you can pin what else the launch needs, say so.
2. Finish 1b from the true deadlines.
3. Then 08, 0c, 16 with the levers in docs/reports/1b-zero-g-floor.md section 3. For 0c check what Bond holds first: pistol-class weapons have range 50 and those snipers sit at d 61-90.

Keep the [L] 08 pins and the 0x30000000 fix. Walk changes are test-side only.
