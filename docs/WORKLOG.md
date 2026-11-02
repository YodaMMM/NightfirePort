# WORKLOG

## level-02-interactables-2
Started 2026-11-01 23:40 UTC. Branch s/level-02-interactables-2 from origin/master 6042ee53.
Task: interactables_07000002_live.rs has 16 tests; using interactables_0700000b_live.rs as the model
(one live test per KIND of player-touchable placement — doors, switches/consoles, fuseboxes, lifts,
pickups, breakables, plates, monitors, rigid bodies, SimpleScripts the player triggers), run the census
of 07000002 [D every placement kind], and for every kind a player can touch, add the live test that
drives it through the real path (walk/E/shoot) and asserts the game's result [M the class's
Update/Activate; D its params], with a sister that fails on the wrong result. Fix any port gap the
tests expose, with evidence. Run interactables_07000002_live, walkreal_07000002_live and nf-game;
quote the totals.
Plan:
1. Census of 07000002 placements [D every placement kind].
2. Read the model (interactables_0700000b_live.rs) + the existing 07000002 tests; map each kind to the port's class Update/Activate.
3. Add one live test per player-touchable kind + sisters; fix port gaps with [M]/[D] citations.
4. Run the target tests + nf-game; suite.py in background; report + finish.

Work log:
- (23:40) sandbox reused (repo/work/ISO/nf-queue present); new token set, fetched; branch created from origin/master 6042ee53; WORKLOG started; first checkpoint pushed.
Work log:
- (00:08) Census re-run [D] (probe): 21 kind-0xD8 bodies — bucketswing #4 (mass 3), 12 lamps (1), 8 pans (2), ALL type 1; no 0xDF wind cel on 07000002. The kind's player-touchable path pinned: class 3 walk touch falls through RB_RestingContactRot 0x1d73a0 (sister: rest-hold 240 ticks + walk-into-bucket moves nothing + the body blocks the 0x800 walk query), gunfire lands the class-5 arm (bucket swings, anchor never translates [M 0x1d7810 type-0-only]). 2 tests added to interactables_07000002_live.rs (now 18, 18/0 green); walkreal_07000002_live 2/0; nf-game 1028/0/9ign. The doc's stale "unported (drawn at rest)" RB row corrected (the evolution landed in rigid-body-cylinder-sweep). No port gap exposed by the new tests.
