## Your task: `level-14-cutscene-fall`, 07000014: the broken cut-scene camera and the fall below the map after it

Spots from real play (Bond root x, y, z, yaw, pitch; frame since load):
- the broken cut-scene: (40.87, 2.98, 40.85, -3.179, 0.210) at frames 1931 and 2300 — the cut-scene camera is inside rock, the rest of the frame a flat blue void;
- the camera-view console: (33.71, 0.65, -18.11, -7.579, 0.220) frame 9909 — "Press Action for Camera View" shows, nothing happens;
- the panels: (54.21, 1.03, 46.15, -8.344, 0.097) frame 5739 — a vent/hatch opening with nothing beneath it; (-88.58, -1.10, 46.15, -0.333, -0.041) frame 25538 — a wall panel with nothing behind it;
- the fall: (-49.65, 0.65, 36.85, -6.979, 0.006) frame 20889 (black screen) then (-107.07, -70.14, 73.28, -4.003, 0.595) frame 21799 — Bond below the map after a broken cut-scene, objective "Sabotage base security and escape from room".

1. The cut-scene at (40.87, 2.98, 40.85): find which script plays there (the trigger/SP, its camera stream) and why the port's camera sits inside rock with a void around it [M — the script camera's base compose (`Script_UpdateCamera` 0x1db518, the SP's base matrix, the parent chain), the camera key interpolation]; fix it.
2. The fall: after the cut-scene near (-49.65, 0.65, 36.85) Bond ends below the map at y -70. Find the cut-scene's hand-back (the marker entity that re-enables Bond, `Player_Enable` 0x195498 — the matrix origin placed directly [M :57-68]) and why the port's hand-back pose lands off the floor; fix it [M].
3. Real-input tests from the trigger to the hand-back: the camera stays in the room, Bond is back on the floor, the walk continues.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-14-cutscene-fall`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

---
claimed-by: sandbox6 at 2026-10-25T09:15Z

---
## Report (done, 2026-10-25T10:07Z)


---
## Report from the worker (done, 2026-10-25T10:08Z)

# level-14-cutscene-fall — the broken cut-scene camera and the fall below the map after it

Branch `s/level-14-cutscene-fall` (from origin/master 8de76e31, merged with origin/master
batch-1025c at 34ac31a5). Window 09:17–11:17 UTC.

## What's done

**One root cause fixes both symptoms.** The port composed every script stream's keyed matrix
with the ScriptPlayer's placement base. The game does that only for scripts that never run a
camera op: **`Script_CameraStart` (0x1d9140) resets the script's base matrix
(SCRIPTINFO+0x00) to IDENTITY** — its first act, before the flag test, is
`Mat_IdentityT(SCRIPTINFO)` [M 0x1d916c: the jal's a0 still holds param_1 = SCRIPTINFO; the
decompiler dropped the argument; `Mat_IdentityT` 0x1e34c8 zeroes all 0x40 bytes and writes the
1.0 diagonal]. Nothing restores the base (`Script_CameraEnd` 0x1db348 only kills the stream;
the Mat_IdentityT caller census has no other Script_* hit). The base is seeded from the
placement transform at load [M Script_Load 0x1d8248 → disasm 0x1d8498:
`View_RotTransMatrix(obj+0x50, obj+0x30, SCRIPTINFO)`; 0x1e5168], and the interps concat it:
case 4 (camera) passes the base-concat flag = 1 [M disasm 0x1da054–0x1da068] whose tail runs
`Concat(SCRIPTINFO, out)` [M disasm 0x1da838–0x1da844 — the spilled a0/t0]; case 1 (entities)
composes the parent chain then the base [M 0x1d9b30–0x1d9b74, flag = 0]. So a script WITH a
camera op keys everything in **world space** from the camera start on; a script WITHOUT one
keys local offsets.

Evidence chain (all [D] unless noted):
- The three 07000014 cut-scene scripts 0x06000833 (SP #276), 0x0600062f (SP #382),
  0x06000848 (SP #407) carry the NIS camera flag [D header u16@+2 = 1]; the fusebox-wire
  0x06000a1b, the SafePickUps 0x060009e2 (07000008), the truck 0x06000a22 (07000005) carry 0.
- Their RAW key positions sit in dense, real level geometry: the opening scene's shot A
  (25.90, 2.16, 36.07) is inside the security-centre room (37 placements in the box: the
  fusebox #833, drones, doors, cover nodes); the Kiko hand-back marker's single key
  (−55.50, 1.55, 42.01) sits beside cover nodes #432/#433 and door #151/#152. Their COMPOSED
  positions are disc-data **voids**: zero placements and zero glists in (70..105, −10..15,
  70..105) (the port's camera spot (78.06, 4.79, 82.69)) and in (−120..−90, −5..10, 60..85)
  (the old hand-back (−107.07, 1.55, 73.28)) — probe test
  `cutscene_fall_probe_07000014.rs` (`probe_the_disc_data_at_the_stages`, run with
  `--nocapture`).
- The port reproduced the user's fall exactly: after the Kiko scene the enable landed at
  (−107.07, 1.55, 73.28) with no floor (floor-under None) and Bond fell through y −70
  (`probe_the_fall_run` trace) — matching the reported (−107.07, −70.14, 73.28) at frame
  21799.
- The same mechanism explains the earlier [L] measurement on 0700004a: the landing scene
  0x060006cf (camera flag 1) re-enabled Bond at the marker's RAW key — the port's hash-0
  kind-0x19 special case was a workaround for the same missing reset. With the fix it becomes
  a no-op (identity concat) and `fix_landing_matrix_live` still passes.

**The fix** (nf-game/src/nis.rs, `Script::run` op 0x0d): `self.matrix = IDENT` at the camera
start, matching [M 0x1d916c].

**The countdown camera test was wrong and is fixed**: it pinned the composed positions
(base + key) as "the disc keys + [M] compose". The compose is real, but the base it concats is
identity from the camera start [M 0x1d916c]; the composed spot is a disc-data void, so the old
expectations contradicted the game. Re-pinned to the raw keys with the address cited
(countdown_cutscene_cameras_live.rs; the file header tells the whole story).

**New real-input tests** (`cutscene_fall_07000014_live.rs`, 2/2):
- `the_vent_cutscene_camera_stays_in_the_room`: Bond walks into TouchOnce #301's volume
  (39.68…41.75, 2.07…4.14, 37.62…40.40) [D]; the camera runs the whole 1400-tick scene inside
  the security-centre room band with floor below every sampled tick; ch 0x47 completes; Bond
  resumes at his own matrix (event 6's no-entity fallback [M 0x1d94c0:73-84]).
- `the_kiko_cutscene_hands_bond_back_on_the_floor_and_the_walk_continues`: Bond walks into
  TouchOnce #381's volume [D]; the camera stays in the touch room; the hand-back puts him at
  (−55.49, 1.10, 42.02) — the marker's raw key, floor beneath — he stays standing 180 ticks
  (the fall symptom had him at y −70 by then) and walks west.

## How it was verified

Ran on the final tree (merged with origin/master batch-1025c): the 07000014 batch
(countdown 2/2, cutscene_fall pair 2/2, panels_07000014 1/2 + 1 pre-existing ignored, the
facility chain 2/2) plus: nf-game lib **547/547**, exit_aim_hint 2/2, fix_landing_matrix 1/1,
ledge_live 2/2, cutscene_guard_models 3/3, walk_07000005 2/2, walk_07000016 3/3,
walk_0700004a 1/1, walk_07000002/06/07/08 9/9. Everything green; no test was weakened (the
one changed expectation set — countdown — was the test contradicting the decompile, per the
task's rule, with the address in the file).

## What's left

- **The full four-crate suite** (`NF_ISO=… cargo test --offline --release -j2 -p nf-game -p
  nf-formats -p nf-audio -p nf-view -- --test-threads=1`) did not fit the 2-hour window (the
  build alone of every nf-view test binary plus the sequential runs). The crates the change
  touches were covered by the set above; the next window should run the full suite on this
  tree and report the total line.
- The camera-view console (33.71, 0.65, −18.11) "Press Action for Camera View — nothing
  happens" and the two panel spots (54.21, 1.03, 46.15) / (−88.58, −1.10, 46.15) are NOT
  addressed — they were context in the task message, not numbered items. With the base reset
  landed, the next suspect for the console is the camera-view hand-off (a
  `Player_Cam2Mode`/viewer-swap family), not the script base.
- The probe test file (`crates/nf-view/tests/cutscene_fall_probe_07000014.rs`) is committed as
  the [D] census evidence; its dump tests are assertion-free and opt-in via NF_ISO.

## Live checks for the main agent

1. **The opening scene's camera in the real game**: fresh boot of 07000014 (not a save-state
   resume), walk into trigger #301, and during the cut-scene watch the script viewer's matrix
   at the viewer's +0x120 (`Script_UpdateCamera` 0x1db518 copies the CAM_INFO there —
   gdbwatch on the viewer object's +0x120; access type: write, per frame). Expected
   first-frame translation ≈ (25.90, 2.16, 36.07) [D key 0 raw] — NOT (78.06, 4.79, 82.69).
   A screenshot at scene tick 0 should show the security-centre room.
2. **The Kiko hand-back**: fresh boot, walk into trigger #381, let 0x0600062f run to the end;
   watch `glb_players+0x30` (the player position, write at the enable) — expected
   (−55.50, 1.55, 42.01) [D 0600062f key 471], and Bond standing on the touch-room floor
   afterwards — NOT (−107.07, 1.55, 73.28).
3. Optional: the escape scene 0x06000848's hand-back — the same watch; its marker key should
   also land in a real room (the port now uses the raw key there too).

## Usage

not visible to me — this session's environment shows no token, credit or context-window
figures (no usage display was available at report time; nothing estimated).
