# WORKLOG

### level-14-skip-handback (started 18:20 UTC, 2026-10-29)

Task: Countdown Part 1 (07000014) — where Bond lands after a skipped cut-scene. A real play
run ended at (26.70, 1.03, 36.27, yaw 2.122) @frame 3046 (inside the wall beside the
control-room window) and (26.58, 1.03, 48.57) @frame 3496; an earlier run put Bond outside
the map after a cut-scene. Find which cut-scene(s) on 07000014 can be skipped, the game's
skip path [M Script_* skip handling / hand-back on skip — level-02-skip-handback test shape],
and where the game places Bond after each skip. Fix the port's skip-path hand-back, pin each
skip with a real-input test (load, play to the cut-scene, press skip, assert position/yaw).
FULL suite, quote the total line.

Base: origin/master (b25b5f34). Setup intact from earlier sessions (repo, work/, ISO,
nf-queue, target cache). Known prior symptoms from level-14 branches: fresh-load cut-scene
ends ~frame 3000; skip lands Bond in the window wall.
