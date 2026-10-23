# WORKLOG

## level-15-ladder-2 (branch s/level-15-ladder-2, started 2026-10-23 19:15 UTC)

Task: 07000015 ladder grab (round 2). `crates/nf-view/tests/interactables_07000015_live.rs::the_ladder_takes_the_climb` is `#[ignore]`: the grab cannot fire from any reachable standing spot (east room's south wall at z -1.95). Goal: find how the game's ladder grab reaches it (use sphere / grab volume / allowed approach) from the decompile [M], port it, un-ignore the test with a sister that can fail. Real input only via `step_bond`.

Work Log:
- 19:15 UTC: sandbox alive (repo, work/, ISO, ../nf-queue all present). Branch s/level-15-ladder-2 created from origin/master a67f1a27e.
