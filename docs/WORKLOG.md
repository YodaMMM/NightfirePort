# WORKLOG

Entries are appended at the end by each task. One named entry per task.

## level-14-interactables-2

Started. Branch s/level-14-interactables-2 from origin/master (6042ee53).

Task: `interactables_07000014_live.rs` has 9 tests. Using
`interactables_0700000b_live.rs` as the model (one live test per KIND of
player-touchable placement — doors, switches/consoles, fuseboxes, lifts,
pickups, breakables, plates, monitors, rigid bodies, SimpleScripts the
player triggers), run the census of 07000014 [D every placement kind], and
for every kind a player can touch, add the live test that drives it through
the real path (walk/E/shoot) and asserts the game's result [M the class's
Update/Activate; D its params], with a sister that fails on the wrong
result. Fix any port gap the tests expose, with evidence. Run
interactables_07000014_live, walkreal_07000014_live and nf-game; quote the
totals.

Progress log:
- setup reused from the sandbox (repo, work/, ISO, rust, queue worktree all present)
- branch created from origin/master 6042ee53
Progress log (continued 2026-11-02 01:03 UTC, fresh session):
- resumed the branch (its only prior commit was the start entry); merged origin/master 8f27b95e into it (516bb5b1); plan: census probe on 07000014 [D], audit the 9 tests' kind coverage, add the missing player-touchable kinds with sisters, fix gaps, run the three groups + suite chunks, report by 02:35 UTC.
- (01:30) The census re-run [D] settled the kinds: NO 0xD8 on 07000014 (the doc §5's "#385 0xD8" is the fusebox #711's wire-group word, not a placement kind — the doc needed no fix); the real gap = 3 placed keypad locks 0x2E (out 0x2C/0x2D/0x3C) absent from the kind table. New test the_keypad_lock_burns_and_its_door_pair_opens_on_e: the census + the load-time lock-edge consumption (ch 0x3C ON at load from the level-start scripts; the pair's ch_lock cleared [M 0x1363c8:101-107]) + the pair's E leg. The real-path beam burn at the Delta floor throttles in the port (~1 round/30 ticks, pending-cycle) → the report's live check. Battery 10/0. Doc gains the 0x2E row; the battery header's kind list updated.
- (01:52) Report written + totals collected: battery 10/0, walkreal 6/0, nf-game 1033/0/9ign, formats+audio 229/4/3ign (the 4 = the slim-image class), suite = the master-merge relink (~1070 binaries) exceeds the window; the log's 73-binary prefix 309/0/22 stands. Finish steps next.
