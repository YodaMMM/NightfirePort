## level-14-cascade-servers-and-door-guards (started 01:57 UTC 2026-10-30)

Task: Countdown Part 1, the generator-hall blast aftermath on 07000014. (1) The server
racks the cascade smashes disappear in the port: for each Destroy/breakable the smash
hits, draw what the game shows after the smash [M Destroy_Smash 0x12bb60's
Control_SetGList swap to the destroyed glist (block-0x27 LOD / the parsemap fallback
slot); D each object's destroyed model] — a leaf whose destroyed glist is a real mesh
must show the wreck; only an undrawable slot vanishes. (2) The guards pressing the open
button behind the double door: find their state and what the game's guards do at a
locked/unlocking door [M the door-guard / door-use AI states, Door_Activate 0x135be0 for
NPCs]; fix the loop so they behave like the game's. Pin both with real-input tests. FULL
suite, quote its total line.

Plan: branch pushed from origin/master (21fbeed3) now; fast suite runner in the
background next; then read Destroy_Smash 0x12bb60 + the door-guard AI decomp, the port's
smash/glist draw path and the 07000014 walk tests, fix, pin, full suite.

Progress (03:30 UTC): part 1 DONE — the destroyed-glist swap's draw side: per drawn 0xe1 leaf the
tag[0] target resolved as Destroy_Create does (LOD-next else the index+1 fallback record), the
wreck baked at the leaf's matrix, drawn only while dead; undrawable slots not baked (the 19
satellite boxes vanish, the 25 barrels + 4 generators show their wrecks). Pinned by
destroy_wreck_07000014_live (2/2). Part 2: the wait pinned (door_wait_07000014_live, 0x81 at the
locked door, sustained), two decomp-exact dispatch fixes in (scream/shout drop [M 0x1681f8 no
0x11/0x12 case], explosive resume = own state [M case 0x21]); the post-unlock crossing still fails
in the probe (restore→combat→threshold grenades, nobody crosses) — live checks in the report.
Suite: partial (slim-ISO disc failures + own_sleeve timeout only, pre-existing); the final-tree run
missed the cutoff — no total line this session.
