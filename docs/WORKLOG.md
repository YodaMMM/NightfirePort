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
