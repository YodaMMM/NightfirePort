
## guard-batch-repin

Branch cut from origin/s/guard-batch 1c0249b79 (per-goal arrival radii + guard rounds through
glass, suite there 4399 passed / 8 failed). Task: first the two non-walk reds —
enemy_accuracy_live astronauts_hit_like_the_chain ("diff 2 d 5: 1 shots in 10 s, interval 180":
did the astronaut's +0xc4 mode 0x1d change alter his firing?) and kiko_button_real_walk_live
("the pit squad member #14 dies to Bond's fire") — fix with a citation if game-code faults.
Then the walks test-side: walkreal 16 real_damage, 01 00agent, 03 real_damage, 13 real_damage,
08 00agent (the shoot-out must kill every end pane), 06 real_damage; no [L] pin loosened; each
binary run whole with quoted results.
Progress: both non-walk reds reproduced. The astronaut red is NOT the +0xc4 change (verified
bisect: reverting both attack_mode lines changes nothing; so does reverting the fire-path
glass_pass — which DOES flip the test green, so the branch's fire-path glass pass causes it).
The glass law itself matches the decomp [M Collide_Update 0x1ea398:222-229 — the +0x9a bit-0x20
query forces the pass on surface-0x10 contacts; raw-byte equality, the port's & 0x3f variant
tried and not the fix]. Mechanism: the glass-passed 0700001b plasma rounds live their 0x3b-family
fuse (14400 ticks) through the harness's 4000-tick settle and the measured window; their DoTrails
flyby draws shift the shared RNG stream the sight acquisition rolls ride — k=0 never acquires in
2400 ticks at diff 2 d 5 (los false throughout), k=1 acquires late and fires once. Bond is healthy
at the measure start (health 100, substate 8). The kiko red (the pit squad member's shoot budget
expires) reproduced, not diagnosed. The six walks untouched. All further work next session.
