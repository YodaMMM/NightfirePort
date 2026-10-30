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
