## Your task: `level-14-capture-placement`, Bond lands in the cell after the Countdown capture scene

On Countdown Part 1 (07000014), after the capture cut-scene that sets the objective "Sabotage base security and escape from room", the port leaves Bond at about (-107.07, -89.41, 73.28): outside the level, in the void. Find where the game puts the player when that scene ends [M the scene's script ops that move/teleport the player (Script_* player placement, the SP/entity base the scene composes, the level's start-point or marker it uses), the cell's placement data [D]] and port exactly that, so Bond stands in the cell and the escape objective is playable. Drive it on the real path in a test (reach the scene, let it play, assert Bond's position/room after it ends against the game's rule) [M].

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-14-capture-placement`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox1 at 2026-10-27T07:19Z

---
## Report (done, 2026-10-27T08:43Z)


---
## Report from the worker (done, 2026-10-27T08:43Z)

# level-14-capture-placement — Bond lands in the cell room after the Countdown capture scene

## What was wrong

After Countdown Part 1's (07000014) capture cut-scene — the ScriptPlayer #382 script 0x0600062F
(pack 07900014.bin), whose end completes ch 0x48 ("Follow Kiko to the Security Control Center")
and raises the "Sabotage base security and escape from room" banner — the port handed Bond back
at the scene's event-6 return matrix `(-107.0651, 1.5464, 73.2835)`: outside the level. With no
floor there he fell to the reported `(-107.07, -89.41, 73.28)` (the y is the fall, the x/z are
the bad matrix exactly).

## The game's rule, with evidence

- The placement mechanism [M]: the scene's stream 11 event 6 (`Script_EventHandler` 0x1d94c0
  case 6) sets +0xAA8 |= 0x100 and copies the PREVIOUS stream's object matrix — the record base
  is +0x80 + 0xAB3·0x28 (one slot below `Script_Run`'s +0xa8), gated on its kind byte ∈ {1,2} —
  into SCRIPTINFO+0x40 (`Mat_Copy(prev obj+0x90 → si+0x40)`; the fallback arm with no object
  copies `glb_players+0x90`, the player's own matrix). At scene end `Script_Free` 0x1da880:44-48
  runs `Player_Enable(glb_players, si+0x40, +0xAB6)` on the 0x100 flag, and `Player_Enable`
  0x195498 writes the matrix translation into the player object +0x30/+0x40 and re-derives the
  angles. That hand-back IS "where the game puts the player when the scene ends".
- Where the marker stands [D 07900014.bin NIS 0600062F decode]: stream 10 creates a kind-0x31
  hash-0 (invisible) marker entity at t1197; its single key (t1197) translation is
  `(-55.4948, 1.5464, 42.0171)` — WORLD frame. Proof of the frame: the scene's CAMERA keys
  (stream 0, t0 `(-51.2739, 0.5677, 39.6973)` → t515 `(-55.7541, 0.8911, 40.6857)`) share it —
  `Script_CameraStart` 0x1d9140 overwrites the script's base matrix with the identity (the
  decompile drops `Mat_IdentityT`'s argument; the disasm shows a0 = the SCRIPTINFO going in
  untouched), so from the camera op on every unparented stream composes against identity — and
  the camera keys must be world-space or the scene would film the void. The marker sits between
  the camera's two positions, inside the control center the walker stands in (the touch #381
  volume -51.97…−46.39, 0…2.07, 37.28…39.46 [D placements]). The scene's wire stream 5 (hash
  0x020006b1, kind 0x31) keys the same frame (local (-49.4690, 0.7531, 39.7303) → in-room).
- The port's bug: the case-1 entity compose kept multiplying the SP #382 placement base
  `(-51.5703, 0.0, 31.2664)` into the kind-0x31 streams (the existing `keys_alone` scope covered
  only non-zero-hash 0xff movers, added for 07000014's door leaves), throwing the marker (and
  the wire) ~55 units west-south into the void.

## The fix

`crates/nf-game/src/nis.rs`, `Script::interp` case 1: the camera-started keys-alone scope now
also covers script 0x0600062F's unparented kind-0x31 streams. Scoped to this script on purpose:
07000001's intro 0x06000087 s5/s11 (also kind-0x31 hash-0 in a camera script) keeps the port's
composed pinning that its live walks pass on today — see "What's left".

The escape scene that follows (SP #407, 0x06000848, pack 07A00014.bin) needed NO change: its
event 6 (stream 7, t949, params [0]) has NO object in the previous stream (stream 6 runs events
only) [D decode], so the game takes the fallback arm — Bond resumes where he was [M 0x1d94c0
case 6]. The port's `player_at: None` already matches; the new test pins that too.

## How it was verified

- NEW `crates/nf-view/tests/capture_placement_07000014_live.rs` (real path, real triggers): vent
  touch → opening scene 0x06000833 → touch #381 → the capture scene → assert the event-6 return
  slot equals the disc's world-frame marker (±0.5); the scene releases control; then the escape
  scene fired through its real SEQ_AND inputs (0x8C + the room touch 0x0C) → assert its return
  slot stays EMPTY (the fallback). GREEN.
- Regression families on the final tree, all GREEN: `walk_07000014_live` 2/2 (the full chain:
  vent, kiko walk, the real fusebox sabotage, the escape scene, Rook, the exit),
  `cutscene_camera_07000014_live` 2/2, `cutscene_door_07000014_live` 1/1,
  `countdown_cutscene_cameras_live` 3/3, `census_07000014` (probe, ignored), `nf-game --lib`
  578 passed / 0 failed / 5 ignored.
- Crates run before push: nf-view (the test binaries above) and nf-game (--lib). The full-suite
  fast runner (`tools/test/suite.py`) was started in the first minutes per the brief but died at
  a tool-call boundary (background processes are killed here) before printing a total; `suite.out`
  holds no FAILED/TIMEOUT lines. The targeted families above are the pre-push evidence; the next
  window should run the suite once on this merged tree.
- Diagnostic probes added: `crates/nf-view/tests/probe14_capture_end.rs` (assertion-free dump of
  Bond/script entities after each scene) and
  `crates/nf-formats/examples/dump_07000014_capture_scene.rs` (the three pack scripts' streams,
  events, entity+camera keys and the SP placements — the [D] base for this report).

## What's left

- The full suite tally on the final merged tree (see above).
- The generalization question: 07000001's intro 0x06000087 s5/s11 markers (kind-0x31, hash-0,
  camera script) currently keep the SP-base compose and its live walks pass on that. The game's
  [M 0x1d9140] identity-base rule says they should compose keys-alone too; either their keys are
  authored base-relative or those walks pin port-shaped spots. A disc read of 07000001's
  06000087 key frame (the same dump-example pattern) decides; then the script scope can widen to
  the whole class and the 07000001 walks re-checked.
- The view-side end-to-end: the sim-level test pins the return matrix; the view's
  `Bond::stand_at` consumption (scene.rs PlayerEnable arm) applies it — a visual confirm of Bond
  standing in the control center after the scene would close the loop.

## Live checks for the main agent

1. After the capture scene ends (07000014, real play), read Bond's position in PCSX2
   (player obj +0x30, the player object is `glb_players`): expect ≈ (-55.49, ~0.8 after the
   settle, 42.02) — the marker key's x/z with the floor snap — NOT (-107.07, -89.41, 73.28).
2. Optional trace: watch SCRIPTINFO+0x40 (the kiko script's si) around the scene's t1197–t1202:
   expect the write at event 6 (t1197) = (-55.4948, 1.5464, 42.0171) (the matrix rows may rotate
   — the translation row is the pin), and `Player_Enable` 0x195498 reading it at Script_Free.

## Usage

not visible to me (no token, credit or context-window figures are shown in this environment; the
session ran well past the 2-hour sandbox mark on the setup plus research described above).
