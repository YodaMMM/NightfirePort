# Aim: missile timing first, then the last routes

Branch from `origin/s/aim-route-3`, merge current `origin/master`, push to `s/aim-route-4`. Read docs/reports/aim-route-3.md.

1. Check the missile start before more walk work. The report says all eight missiles leave state 0 in one tick because the state-0 countdown runs at 512.0 per tick, though their seeds differ: 105/40/20/0/60/105/105/60. Seeds that differ suggest a staggered start. Read `MissileDeploy` 0x1aa218 and its state-0 arm in the decompile and in `space_missile.rs`: what the rate really is, where 512 comes from, and what units the seeds are in. Cite it. If the port's rate is wrong, fix it with a test and recompute the deadlines; the walk may then fit without more levers.
2. If the timing is right, finish 1b from the report's margins.
3. Then 08, 0c, 16 with the levers in docs/reports/1b-zero-g-floor.md section 3. For 0c check what Bond holds first: pistol-class weapons have range 50 and those snipers sit at d 61-90.

Keep the [L] 08 pins and the 0x30000000 fix. Walk changes are test-side only.
