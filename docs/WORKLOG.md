### level-14-capture-placement — start (2026-10-27 ~07:26 UTC)

Branch `s/level-14-capture-placement` created from `origin/master` @ 3040e649.
Task: on Countdown Part 1 (07000014), after the capture cut-scene that sets the
objective "Sabotage base security and escape from room", the port leaves Bond at
about (-107.07, -89.41, 73.28): outside the level, in the void. Find where the
game puts the player when that scene ends [M the scene's script ops that
move/teleport the player, the SP/entity base the scene composes, the level's
start-point or marker it uses; the cell's placement data [D]] and port exactly
that, so Bond stands in the cell and the escape objective is playable. Drive it
on the real path in a test (reach the scene, let it play, assert Bond's
position/room after it ends against the game's rule) [M].

Plan:
- Start the fast suite runner in the background in the first minutes.
- Read the decompile for 07000014's capture scene: script ops that place the
  player, the scene's SP/entity composition, the start-point/marker it uses.
- Reproduce through the real input path (reach the scene, let it play).
- Port the game's placement; cite addresses for every pinned behaviour.
