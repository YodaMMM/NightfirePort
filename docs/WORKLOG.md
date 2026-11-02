
## level-15-mines-shot-from-range (branch s/level-15-mines-shot-from-range, from origin/master c628a2fa)

Started 2026-11-02 09:52 UTC (sandbox reused from the 13-lobby session: repo, work/, ISO, alsa prefix, queue worktree all present; stale test binaries pruned, 4.7G free).
- Task: on 07000015 the nine 0xFE laser mines (Mine_Update 0x17c930) must explode when shot from range (>8 u) through the real input path — no test currently reaches a mine from a distance. Plan: (1) read Mine_Update + the bullet-vs-object pick + the level's loadout, (2) walk a real route to a clear line-of-sight spot, aim with the level's own gun, fire, (3) if the mine does not take it, find why in the decompile and fix; add the live test; run interactables_07000015_live, walkreal_07000015_live, mine_bob_live, then the full suite.
