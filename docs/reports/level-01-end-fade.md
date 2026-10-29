# level-01-end-fade — the level-end screen fade

## What the game does (the read)

- `Trigger_Update` 0x1b1a78:162-166, case 10: a LOAD_LEVEL trigger fires `Trigger_Activate` when
  `switch_channels[gate]` is on — no proximity check.
- `Trigger_Activate` 0x1b2318:89-92 (disasm 0x1b24b8-0x1b24ec): `Music_Event(3,1)`;
  `GS_PauseGame(1)`; `ResetMap_LevelToLoad(ext+0x20, 1, 0)`; **`GameFlow_PushState(300.0f, 7,
  0xff)`** (0x43960000 = 300.0f); the trigger spent (type←99); `Player_RamSave`.
- `GameFlow_PushState` 0x1c9f38 (disasm 0x1c9f7c-0x1c9f84): a state-7 push converts to state 8
  through `Camera_SetFade(a0=6, f12=300.0 passthrough, a1=0xff passthrough, a2=1)`; a push aborts
  only when the TARGET viewer's own object is a running mode-1 fade [M 0x1bbdd8 disasm
  0x1bbe08-0x1bbe18: the refuse gate is mode(+0x234) != 0 AND active(+0x235) != 0].
- `Camera_SetFade` 0x1bbdd8 (disasm 0x1bbe20-0x1bbe54): arms the fade object — level 0, rate
  (60/duration)·(1/60)·(colour&0xff)/255 = 1/300 for the exit, colour bytes +0x230..232 =
  0x00,0x00,0x00 (**black** — the push's 0xff), active +0x235 = 1, mode +0x234 = 1.
- `Game_Draw` 0x1e6f50 draws `psiFadeView(R+0x230, G+0x231, B+0x232, A+0x233)` for every viewer
  whose active byte is non-zero — two passes (viewers 0..4 at :188-209, 0..10 at :316-356).
- `Camera_UpdateGlbVars` 0x1bada0:16-27 advances the level by rate·FRAME_RATE_MUL every frame and
  clamps the byte; state 8 pops onto state 3 (`ResetMap_Load`) when the fade ends
  [M 0x1c98c8:173-189] — the next level loads after the fade.
- Length/colour: 300 frames (5 s at 60 Hz) on every trigger exit [M 0x43960000]; the mission-end
  path pushes 60.0f [M 0x182c60:74, 0x42700000] with FadeClr 0xff (black) or 0xff0000ff (red
  death) [M 0x182c60:27,38].

## What the port had and what was missing

The sim side was already on master (mayhew-end-transition: `ExitHold {next, fade_frames, total}`,
both `Trigger_Activate` arms + the mission 60-frame arm, the frozen tick, the deferring drain —
docs/systems/exit-hold.md). The missing piece was the DRAW: `Scene::script_overlay_hud` chained
the overlay sources `view_fade.or(exit_fade_view()).or(spawn_fade_view())`, and 07000001 carries
a PARKED script fade-in — the intro/meet cut-scene's op-0x13 fade ends at alpha 0.0 and is never
cleared (the script-camera hand-back clears only a viewer-0 fade). A parked `Some(alpha 0)`
shadowed the exit fade: the pane never drew and the exit looked like a bare 5-second freeze +
cut (the reported "cuts straight to the next level").

The game's fade object is PER VIEWER: `Camera_SetFade` indexes `(&glb_viewer)[arg0]`
[M 0x1bbdd8]; the exit push arms **viewer 6**'s object [M a0=6], the script op-0x13 fade arms
viewer 4's, `Game_Draw` draws each active viewer's own pane. A parked script pane draws byte 0 —
it cannot cover the exit pane. The fix is the chain order (scene.rs, `script_overlay_hud`):

    let fade = sim.drones.exit_fade_view().or_else(|| sim.drones.view_fade).or_else(|| sim.drones.spawn_fade_view());

## The pin

`crates/nf-view/tests/walkreal_07000001_live.rs::the_exit_fade_starts_on_the_exit_tick_runs_the_games_300_frames_then_the_level_loads`
— on the walked 01 exit (the route body became the shared `walk_to_the_exit` helper;
`stop_at_fire` keeps the walk test's own mission-monitor block byte-identical):

- at the fire tick: `exit_hold = {next 0x07000002, total 300, fade_frames 300}`, `level_exit`
  posted, fade alpha 0.0 [M 0x1bbdd8 level 0];
- +100 ticks: Bond/anchor/clock frozen [M 0x1c98c8:44-47], `fade_frames` 200, alpha 100/300, and
  the DRAWN HUD carries the black pane at that alpha [M 0x1e6f50 psiFadeView];
- the hold clears at exactly 300 with the request intact [M 0x1c98c8:173-189];
- the warm load then spawns Bond at 07000002 start #0 (−27.17, 1.64, 29.80)
  [D docs/levels/07000002.md].

## Other exits

Every LOAD_LEVEL (kind 10) fire rides the level-agnostic post site (the drone control walk's
trigger arm) and the use-key arm — both create the same 300-frame hold; the mission LevelEnd arm
creates the 60-frame one [M 0x182c60:73-74]. The script event-0x12 loadmap intentionally does
NOT hold [M 0x1d94c0:194-199] — pinned by
`mayhew_end_transition_live::the_script_loadmap_exit_does_not_hold`. The other on-foot exits the
walkreal suite reaches (02..0d, 11-16, 1b, 41, 4a) all end through the same trigger path.

## What I ran

- Directly: `NF_SILENT=1 target-cache/release/deps/walkreal_07000001_live-0456f788d7bed7fc
  --test-threads=1` → 5 passed / 0 failed (51.4 s) — the new pin, the main walk and the
  follow-down walk all green.
- The chunked full suite on the final tree (this sandbox kills background jobs between tool
  calls, so `tools/test/chunk_suite.py` with state `/home/z/suite01_state.json`): **interim
  total over 182/904 binaries = 650 passed, 0 failed, 48 ignored.** The single red:
  `mp_stress_explosives_live` rc −9 — the sandbox's 4 GB OOM, recorded identically on previous
  sessions' trees. The remaining ~722 binaries had not been reached when the session deadline
  hit; resume with `python3 tools/test/chunk_suite.py /home/z/suite01_state.json --budget 480`
  (the build is warm). Environmental, from prior sessions (not reached in the interim):
  `probe_pylon_render` needs a GPU (wgpu `adapter: NotFound`).

## What's left

- The remaining ~722 binaries of the chunked suite on this tree (resume command above). Nothing
  red so far.
- Session note: this sandbox's earlier session finished level-16-security-station-use's
  report+push but its finish.sh/ntfy steps were superseded by the driver's next message, and
  capsule-cull-and-los-masks-12 was superseded before its branch was created. This task's branch
  was created fresh from origin/master f40be760.

## Live checks for the main agent

- 07000001, the walked exit: `--bond -30.8,6.4,13.1,0,0` is the stairwell-head anchor; walk Bond
  onto the exit plate (−33.1, 2.9, 15.4) with the contact. EXPECT: the screen ramps to black over
  ~5 s while the world stands frozen, then 07000002 loads (start −27.17, 1.64, 29.80). Before the
  fix the world froze ~5 s with NO fade.
- Mid-fade render spot: `--bond -31.5,3.4,14.5,0,0` during the hold should show the black
  overlay at ~half alpha over the drawn scene.

## Usage

not visible to me
