## Your task: `dead-pane-collision-2`, breakable panes solid while alive, without phantom collision

On master, `Drones` builds the breakable panes' collision parts (`0x400_0000_0000 | index`, `Break::shape_key` at `Break::matrix`) but does NOT add them to the per-tick `set_dynamic` set (drone.rs, the `_pane_parts` block). The rejected-step drain into `Breaks::drone_step` is passed 0.0. `dead_pane_live`'s two tests are ignored until this lands.

With the parts added, these real-path tests broke: elevator_buttons_09_live (both), sight_frustum_live (both: placements 158/261 occluded at the start view), guard_first_shot_01_live (both), fix_objects_live (a death's drop), fix_player3_live (death drops, firing at a marked civilian), render2_live (dropped guns), escort_cover_live (a blast hurts the hiding escort). None of those spots has a pane in the way, so the parts add collision where the game has none.

**Also: the bullet world stop.** `update_player_bullets` gives a round ONE contact at the nearest breakable and kills it there [M Collide_Update 0x1ea398:216-229], but the check that the contact comes before the static world's first bullet stop is held off (`BREAK_WORLD_STOP = false`, drone.rs): the breakables' own meshes still sit in the static world, so `segment_bullet` reports the pane or vase itself first (07000007 vase #49 at point-blank: world stop at t 0.01, bounds at 0.058). Once live breakables leave the static set, turn the flag on and keep `interact_07000007_live` (the party vase), `interact_07000009_live::shooting_the_lobby_roof_glass_breaks_it` (from its stance every round met the world at t ~0.31 before #123's plane at ~0.47 — name that triangle) and `interactables_0700000a_live` green.

1. Find why: is the shape already in the static world (a double), is its triangle data already in world space so `b.matrix` moves it a second time, or does `shape_key` resolve to the wrong object (by_hash vs the anonymous key)? Print each pane's part bounds against its placement bounds on 01, 08, 09 and 0a, and cite the game's rule [M Collide_PickObj / Break_Create / Break_Kill 0x12b970:59-63].
2. Fix the pairing, then put the parts in the set and pass `blocked_reach` again. All the tests above must pass, plus dead_pane_live (un-ignore both), gondola_windows_live and breakables_live.
3. Sister: a census over every SP level showing no pane part lies more than its own bounds away from its placement bounds.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/dead-pane-collision-2`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.
