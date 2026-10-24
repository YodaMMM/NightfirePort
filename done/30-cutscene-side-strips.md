## Your task: `cutscene-side-strips`, cutscenes fill the whole widescreen frame from the cutscene camera

On a 16:9 window, an in-engine cutscene draws its camera only in the 4:3 centre: the side strips outside it keep showing the gameplay camera's view. Repro: `nf-view <iso> 07000004 --bond -6.12,5.44,36.24,-0.402,-0.005 --frame 1694 --shot out.png` — the centre is the cutscene's top-down shot of the courtyard, the left/right strips show the courtyard wall and roofs from Bond's own camera.

1. Find where the port renders the cutscene (script camera) view and where the side strips come from (a second pass, the 4:3 letterbox/pillarbox, the HUD layer) in `crates/nf-view/src/` (`script_camera`, the frame/draw path, the cutscene letterbox bars).
2. Make the cutscene camera drive the whole frame: the strips either show the same cutscene camera (a widened view at the same vertical FOV — the port's widescreen rule for gameplay, `view_aspect`) or plain black, whichever matches how the port already treats widescreen in gameplay; the cutscene's own top/bottom bars stay. Never the gameplay camera during a cutscene.
3. A test that during a script-camera frame no draw uses the gameplay camera, and headless `--shot` renders of the repro frame before/after named in the report with their paths.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/cutscene-side-strips`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.



---
claimed-by: sandbox4 at 2026-10-24T21:11Z

---
## Report (done, 2026-10-24T21:48Z)


## Report body (attached after the cat path failed above)

# cutscene-side-strips — the cut-scene frame builds at the viewer's own aspect

## What was wrong

On a 16:9 window the port drew every in-engine cut-scene inside a hard-coded 4:3 centre band
(`cutscene_frame`'s `GAME = 4.0/3.0` constant, both draw paths) with black side strips, whatever
the AV options' Widescreen row said. The game does not do that: the cut-scene view builds at the
**viewer's own aspect word** —

- `Script_UpdateCamera` 0x1db518:9 multiplies the script camera's view by `viewer+0x114` [M];
- the frustum builder `Vision_Calculate_View_Fustrum` 0x1e7df8:79 reads the same word (`fStack_90 = fVar4 * *(float *)(param_1 + 0x114) * 200.0`) [M];
- `Camera_CalcViewAngles` 0x1baf70:28-32 writes 16/9 (0x3fe38e39) there when the AV options'
  widescreen flag `DAT_002a37c4` is set, 4/3 (0x3faaaaab) otherwise, before the split-screen ±2 [M].

So with the option ON the game's cut-scene is a full-width 16/9 picture; the port shrank it into
the 4:3 centre with dead side strips — the reported "the cut-scene draws only in the 4:3 centre".
The port's own gameplay widescreen treatment is the widened full-window view at the same vertical
FOV, and the cut-scene now matches it, which is the task's primary option ("a widened view at the
same vertical FOV … whichever matches how the port already treats widescreen in gameplay").

## The fix

- `crates/nf-view/src/lib.rs` — `cutscene_frame(win_w, win_h, game_aspect)` takes the game
  viewer's aspect instead of the 4:3 constant (a non-positive value falls back to 4/3). With
  16/9 on a 16:9 window the band is the whole frame (`[0, 0, w, h]`): the script camera drives
  everything, the vertical FOV unchanged, the cut-scene's own top/bottom letterbox bars stay
  (they are HUD quads inside the band). With 4/3 nothing moves: fix-cutscene-letterbox's
  pillar-box shape is unchanged (its own tests keep passing untouched apart from the new
  argument).
- The window path feeds `game_aspect` from the front end's AV row (`FrontEnd::view_aspect`,
  the `0x10000223` Widescreen row's 16/9-or-4/3) and now also feeds that value into
  `Sim::view_aspect` — the field's documented host ("the host feeds it from
  `FrontEnd::view_aspect`", screen-adjust-model), which until now nothing fed, so the gun-tilt
  input and the cut-scene frame use the same option.
- The headless shot path takes the aspect from `Sim::view_aspect`; the new `NF_WIDESCREEN=1`
  render aid sets 16/9 on the headless sim (no front end there), the same way the other
  NF_* render aids work.
- The camera itself never regresses: while a script camera is live both draw paths replace the
  gameplay camera with it (the CLI walk's Bond-head → `script_camera()` replacement; the window
  redraw's per-tick camera chain) and `overlay_for` empties the gameplay HUD. Those are now
  pinned by tests instead of convention.

No gameplay camera ever draws during a script-camera frame: the band only changes WHICH region
and aspect the script camera renders into.

## Verification

- `cargo test --offline --release -p nf-view --test cutscene_side_strips_live` — 3 passed:
  the draw camera at a live-cutscene tick is the script camera and is off Bond's head;
  `overlay_for` has zero vertices/runs during the script camera; the frame follows the
  viewer's row (4/3 → `[160, 0, 960, 720]`, 16/9 → full frame, 16/9 on a 4:3 window →
  letter-boxed `[0, 96, 1024, 576]`, both bands owning the full window height = same vertical FOV).
- `cargo test --offline --release -p nf-view --test fix_cutscene_letterbox_live` — 4 passed:
  the 4:3 pins kept (their `cutscene_frame` calls now pass the default 4/3 explicitly).
- `cargo test --offline --release -p nf-view --test cutscene_draw_live` — 2 passed (the
  neighbouring cut-scene draw tests).
- `cargo test --offline --release -j2 -p nf-view --no-run` — Finished: every nf-view test
  binary compiles against the change; nf-game is untouched by this task.

Headless renders (lavapipe, `--shot`, 1280×720, the task's repro spot
`--bond -6.12,5.44,36.24,-0.402,-0.005`, level 07000004):

- `download/cutscene_before_pillarbox_f900.png` — before, tick 900 (live script camera):
  the 4:3 centre picture with the black side strips (what a widescreen-option user got).
- `download/cutscene_after_optionoff_pillarbox_f900.png` — after, option off: identical
  shape (the fix does not move the option-off game).
- `download/cutscene_after_widescreen_f900.png` — after, `NF_WIDESCREEN=1`: the same
  cut-scene camera driving the whole 16:9 frame at the same vertical FOV, more of the room at
  the sides, the cut-scene's own top/bottom bars still on.
- `download/cutscene_repro_f1694_after.png` — the task's literal repro frame on this tree:
  1694 sits AFTER 07000004's intro script camera (live ~ticks 1–1330 on this walk, measured
  headless), so the frame is honest 16:9 gameplay (HUD, objective banner) — there is no
  cut-scene picture left to strip. The centre-cutscene-plus-gameplay-strips composite the task
  quotes matches a build older than fix-cutscene-letterbox (31a4a775, 2026-10-14), whose
  pillar-box the live frames show on this tree.

## What's left

- The full suite could not be run to completion in this sandbox: the fast runner
  (`tools/test/suite.py`) is killed between tool calls in this environment (three starts —
  plain nohup, setsid, and one restart — all died with 0 bytes of output), and a foreground
  run does not fit the window. Covered instead: the three targeted live-test files above and
  a full compile of every nf-view test binary. The next round should run the fast suite on
  the final tree (`NF_ISO=<abs> nohup python3 tools/test/suite.py > suite.out 2>&1 &`) and
  report the total line.
- The real window experience (option ON, live play) should get the main agent's render check:
  `--bond -6.12,5.44,36.24,-0.402,-0.005` around tick 900 of 07000004 in a 16:9 window with
  the AV Widescreen row ON — expect the cut-scene filling the frame width (bars on), never
  the 4:3 pillar-box.

## Live checks for the main agent

None required this round. Optional confirmation (visual only): with AV Options → Widescreen ON,
play 07000004's intro cut-scene on a 16:9 window — the picture should fill the window width
(top/bottom bars on), and toggling the row OFF should restore the 4:3 pillar-box.

## Usage

`not visible to me` — this session shows no token, credit or context-window figures.
