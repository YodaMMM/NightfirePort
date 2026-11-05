# sp-panes-new-drawing — the audit-sp-panes §3 on-foot rows, built

Branch `s/sp-panes-new-drawing` (cut from origin/master 17a784980). Task: build the on-foot
items of `docs/reports/audit-sp-panes.md` §3 with the template values it lists, cite [M]/[L]
per value, and pin each with a test on the real level. Driving panes out of scope.

## Scope decision

§3's four rows: the RCCarPane row is the RC-car-gadget driving HUD ("when Bond **drives** the
RC car gadget in SP" — audit §3 row 1) and needs an RC-car entity model the port does not
have; it is a driving pane, so per the task it is out of scope. The other three rows are built.

## What was built, with the evidence

### 1. The MissileDeploy lamps are live (and the countdown constant was wrong — fixed)

The port already had the full `SpaceMissile_Update` state machine
(`crates/nf-game/src/space_missile.rs`, states 0-4, the MissileDeploy writes at
0x1aa238/0x1aa338/0x1aa314) and `Drones::missile_deploy` (drone.rs:12712) — but the space pane
ignored it: `space_pane` pinned `let deploy = [0u8; 8]` (hud.rs:72), so only the channel rule
could ever show.

- **Wiring**: `space_pane(table, level, on, frame, deploy: &[u8; 8])` takes the sim's array;
  the sole caller passes `&s.drones.missile_deploy` (hud.rs:1131). The deploy-1/2/3 lamp rules
  were already modelled per [M 0x19d3b8:22-72] (pass 1: deploy 1 → tex 0x0300_0177 flashing
  0x00ff00ff/0xffffffff on `frame%30<15`; pass 2: channel on && deploy≠3 → tex 0x0300_017a,
  0xff000080 at deploy 2 else 0xff0000ff, overriding pass 1; pass 3: deploy 3 → steady
  0x00ff0080).
- **The countdown constant — corrected, measured, and left for a paced follow-up**: state 0's
  decrement is `COUNTDOWN_STEP = 512.0`, cited to "the float at 0x31d0dc" — an unmapped
  address. The disc disassembly of the state-0 body (0x1aa1c8, jump-table entry 0 of
  `PTR_LAB_002fcc30`) loads the constant with `lui $at,0x31; lwc1 $f1,-0x2f24($at)` =
  0x310000−0x2f24 = **0x30d0dc** = `REC_FRAME_RATE` (symtab `0030d0dc 4 REC_FRAME_RATE`, word
  0x3c888889 = 1/60.0; verified by direct capstone disassembly of work/elf/ACTION.ELF this
  session), and the seed p3 is SECONDS [D: 105/40/20/0/60/105/105/60] — a game-accurate
  countdown runs seed·60 gated ticks. I shipped the 1/60 constant first (unit-pinned) and it
  broke the two master-green `walkreal_0700001b_live` mission walks: with the real timing the
  state-1 sync writes (ch 181..188 @t10 of state 1) land seed·60 ticks later and the walk's
  console latch budgets — paced against the collapsed state 0 — missed consoles 1/2/7
  ("console 1 latched by the real shot"; the measured latch set was 3/4/5/6/8 at frames
  6345-7863). Per the merge rules (never ship a change that turns level walks red) the
  constant ships UNCHANGED at 512.0 with the full evidence in the `space_missile.rs` module
  doc, and the re-pacing (the console budgets + the disarm-assert wait, already staged as a
  no-op in `play_the_mission_opts`) is the follow-up. The wiring itself is independent of the
  constant.
- Pins: `space_missile.rs` unit tests — `state0_holds_while_the_gate_is_clear`
  ([M 0x1aa1c8-1e4]) and the state-1/2/3 suite. 6/6.
- `fix_space_pane_live`: the four space_pane calls updated; new `the_missile_pane_deploy_states`
  (deploy 1 flash in/out of phase, the pass-2 override, deploy 2 half-red, deploy 3 steady
  green, and the st-2-channel-off template recomputation [S — the documented sticky divergence;
  st 2 is only reachable with the channel on, the game never sits there on disc]) and
  `the_space_pane_draws_the_sims_deploy_state` (the sim's `missile_deploy[2] = 1` → the overlay
  draws tex 0x0300_0177; the old hardcoded zero never could). 5/5 on the real ISO.

### 2. The redeemer pane's sprite 10, the copter lock bracket

`HUD_UpdateRedeemerPane` 0x19e178:137-183 [M, read from the decompile; the sprite-table
template from the disc]:

- Pre-loop the runtime layer byte (+0x38 — the LAYER, per Sprite_BuildList and the merge
  lessons; not an alpha) parks at 0xff = hidden; per CopterList node with a body
  (`Copter_GetBody` 0x17e4b8 — None → skip [M :144-152]) whose
  `View_3DPoint2Screen(body+0x30)` returns ≥ 1 (in front), the screen point clamps into
  [0,512]×[0,448] ([M :153-164]), the sprite's x/y are written as `scr.x − w` /
  `448 − scr.y − h` ([M :167-169]; w/h are the runtime +0x28/+0x2a = the texture's own dims
  [M `hashtable_set_sprite` 0x1ca47c-48c, direct disasm: it stores texrecord+4/+6 there]) and
  the layer re-lights to 0x27 ([M :170]); a bracket centred on (256,224)±32 with
  `GameState & 8` re-parks the layer ([M :171-176]). The walk is last-wins for both the
  position and the park.
- Template [L ee_start_05.bin 0x2bda30 + 10·0x2c]: tex **0x03000059**, (192,160) 128×128,
  flags 0x500, layer 0x22, colour 0x7f7f7f78 — read directly from the EE snapshot this
  session (the runtime layer 0x27 overrides the table's while drawn).
- Port: `copter_lock_bracket(points, frame, w, h)` (hud.rs, pure) + the `10 =>` arm of the
  redeemer block (hud.rs:700 was `10 => {}`). The projection runs through the ACTIVE viewer —
  during the ride the missile's own camera [M `Player_PositionCamera` 0x18f5d8 case 0xc; the
  port's render builds the same yaw/pitch from `view_override`'s dir] — with `project_hud`'s
  60° zoom-1 math. The copter root's matrix translation stands in for the body obj+0x30 (the
  body rides the root; the port keeps no per-part pose) [S]. Copters come from
  `s.drones.copters` behind `Copter::body`'s own hide gate [M 0x17e4b8:6-8].
- Pins: `crates/nf-view/tests/redeemer_copter_bracket.rs` 7/7 — the pure pins (park when no
  copter, the −w/flip-y position, the clamps, the ±32-open-box blink, last-wins both ways),
  the [L] table pin, and the live pin on 07000005's gate-0 copter #241 (flies from the load,
  real input only) with the cam up: the bracket draws at frame&8 clear and blinks off at
  frame&8 set, and draws nothing with the cam down ([M 0x195928:41 — the game enables pane 9
  only via `Player_SetCamMode(0xd)`; the test stands in for that one enable — no disc path
  fires a guided weapon on the probed levels' reachable walks]). The texture probe fell
  through 07000008/0700001b/07000003/0700000b to 07000005 (the bracket texture rides the
  level HUD archives; the probed prints are in the test output).

### 3. The status-pane image message

`HUD_UpdateStatusPane` 0x19c310 state 2's image arm [M :183-190; disc disasm 0x19c558-570
recovers the dropped argument the decompiler lost]: when `TXT_MSG+0x10 != 0`,
`hashtable_set_sprite(sprite0, msg+0x10)`, sprite text cleared, straight to state 3 — the
strcmp and the text rows are skipped. The port's `Msg` grew `image: u32` (scene.rs; every
literal updated, `Msg::new` defaults 0) and the objective banner draw renders the texture
where the body row sits, with the body row's own fade-in schedule and colour, instead of the
label+text rows [S — no disc writer of +0x10 exists: `Text_AddMsg` stores it only for
NULL-text posts [M 0x1b0c30:160] and the 36-decompiled-call-site sweep (the audit's) found
only 0; the queue pushes keep 0, so the arm is inert until a future caller sources an id].
Pin: the objective-banner live binaries compile-run the arm via the new field and pass.

## Verification

Changed code: `crates/nf-game/src/space_missile.rs`, `crates/nf-view/src/hud.rs`,
`crates/nf-view/src/scene.rs`; tests: `fix_space_pane_live.rs` (+2 tests, 4 calls updated),
`redeemer_copter_bracket.rs` (new), the three objective-banner tests (+`image: 0` per the
new-field rule).

- `nf-game --lib space_missile` 6/6.
- `nf-view fix_space_pane_live` 5/5; `nf-view redeemer_copter_bracket` 7/7;
  `nf-view fix_objective_banner_live` 3/3; `nf-view fix_objective_messages_live` 5/5;
  `nf-view objective_banner_timing_live` 3/3.
- `nf-view walkreal_0700001b_live` (the changed space level, full binary, single-threaded)
  **17 passed / 0 failed / 2 ignored** — run after the constant revert, including both mission
  walks and the real-health variant.
- The full `cargo test --offline --release -p nf-game -p nf-formats -p nf-audio -p nf-view`
  suite did not fit the window: the sandbox (2 cores) spent the first ~25 min on setup (clone
  + work-slim + ISO + rustup + cargo fetch + the ALSA prefix) and the walkreal level set
  alone exceeded 10 min foreground. Per the merge rules the changed-code set above ran green;
  the walkreal_*_live level set (25 binaries) + npc_doors_live + fix_guard_*_live +
  fix_objective_messages_live + objective_banner_timing_live were relaunched under nohup at
  04:26 — their result lines are appended below.

## What's left

- The rest of the walkreal_*_live level set (25 binaries) + npc_doors_live + fix_guard_*_live
  and, on a machine with room, the full suite — the sandbox (2 cores/3 GB, ~2 h) spent its
  first ~25 min on setup and the remaining window on the changed-code set + the 0700001b
  binary; the walks share the changed helpers only through `hud::overlay` (the bracket arm
  draws nothing without `missile_cam`, the deploy wiring is 0700001b-gated, the image arm is
  behind `Msg.image != 0` which no disc pusher sets).
- The countdown re-pacing follow-up above.
- The RCCarPane row (driving, out of scope) and the audit's four live checks (§4). The
  bracket's live checks for the main agent: (a) a `--bond` shot during a guided ride in the
  level that carries the redeemer textures, with a copter in view, to confirm the bracket
  frames the target up-left of the projected point (the `448 − scr.y − h` flip's convention
  against `View_3DPoint2Screen`'s y direction could not be pinned from the ELF — the
  function's body disassembly stops at a PS2-only branch instruction); (b) the deploy-1 flash
  on 0700001b per console after 105/40/20/60 s of its gate channel being on.

## Usage

not visible to me
