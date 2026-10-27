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

### level-14-capture-placement — done (2026-10-27 ~09:35 UTC)

- The symptom pinned: after 07000014's capture cut-scene (ScriptPlayer #382, script 0x0600062F,
  pack 07900014.bin — the scene whose end completes ch 0x48 and raises the "Sabotage base
  security and escape from room" banner) the port handed Bond back at the kiko scene's event-6
  return matrix = (-107.0651, 1.5464, 73.2835): the void; he fell to (-107.07, -89.41, 73.28).
- Mechanism [M]: the scene's stream 11 event 6 (`Script_EventHandler` 0x1d94c0 case 6) snapshots
  the PREVIOUS stream's entity (stream 10's kind-0x31 hash-0 marker, created t1197) into
  si+0x40; `Script_Free` 0x1da880:44-48 → `Player_Enable(glb_players, si+0x40, +0xAB6)`;
  `Player_Enable` 0x195498 writes the matrix translation into the player +0x30.
- Root cause: the marker's disc key is WORLD-frame once `Script_CameraStart` 0x1d9140 has
  overwritten the script base with the identity — stream 10 key t1197 local (-55.4948, 1.5464,
  42.0171) sits between the camera's own keys (stream 0 (-51.2739, 0.5677, 39.6973) →
  (-55.7541, 0.8911, 40.6857)) and in the control center the walker stands in; the port's
  kind-0x31 compose still multiplied the SP #382 placement (-51.5703, 0.0, 31.2664) into it.
- Fix (nis.rs case-1 interp): the camera-started keys-alone scope now also covers 0x0600062F's
  unparented kind-0x31 streams (the marker AND the wire stream 5, hash 020006b1, same frame).
  Script-scoped: 07000001's intro 0x06000087 s5/s11 (also kind-0x31 hash-0, camera script) keeps
  the port's composed pinning its live walks pass on — re-checking that scene's key frame
  against the disc is the next-window item.
- The escape scene SP #407 0x06000848 needs nothing: its event 6 (stream 7, t949) has NO object
  in the previous stream (stream 6 runs events only) [D 07A00014 decode] → the game's fallback
  arm (the player's own matrix) [M 0x1d94c0 case 6 iVar5 == 0] — the port's player_at: None
  already matches.
- Tests: NEW crates/nf-view/tests/capture_placement_07000014_live.rs (the real trigger path:
  vent touch → opening scene → touch #381 → the capture scene → the return slot = the disc
  marker; then 0x8C+0x0C → the escape scene → the return slot empty). GREEN: it 1/1,
  walk_07000014_live 2/2, cutscene_camera_07000014 2/2, cutscene_door_07000014 1/1,
  countdown_cutscene_cameras 3/3, nf-game --lib 578/0/5. Probes added:
  probe14_capture_end.rs (assertion-free), dump_07000014_capture_scene.rs (nf-formats example:
  the three pack scripts' streams/events/entity+camera keys + the SP placements).
- Left: the full suite tally (the background runner died at a tool-call boundary; the targeted
  families above ran green on the final tree); the 07000001 intro-marker key-frame re-check;
  the report's live checks for the main agent.
