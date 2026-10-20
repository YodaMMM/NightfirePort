# The 07000009 script-lift call stations (garage-elevator-buttons-09)

The underground car park's second lift — the one the `SCRIPT_liftSCRIPTPLAYER` pair animates —
takes calls from wall buttons. This doc is the mechanism and its evidence; the port fix is one
line (the use-pick origin), the behaviour was already carried by the script runtime.

## The objects

- `SCRIPT_liftSCRIPTPLAYER` #21/#22 (kind 0xD9, hash 0x0600007E / 0x0600007D, mode 8, mask 4,
  **out 0**, no secondary script; placements near the origin, the art is script-positioned)
  [D placements dump]. Twin scripts for the twin lifts (right/left column): each animates its own
  cab (entity 0x02000506 `SCRIPT_LIFT_IN`, keyed (−2.77, −368.19, 28.08) + the SP's placement
  base x 4.01/0), inner doors, per-floor outer doors (0x020006a8/a9 `SCRIPT_LIFTDOOROUTER_L/Rcarpark`
  for the car park, 0x02000509/0a for the lobby) and lightmaps [D FuseBox 0x31 placements #8-20].
- The call button art is created by the SUB-script 0x0600064F stream 2: `liftswitch_off`
  (0x020006b2) at t0, `liftswithch_on` (0x020006b3) at t7, off again t524, on t537, baked at
  (−4.62, −375.52, 25.91) [D nis dump] — the car-park button (the SP's host-stream base puts
  SP #22's instance at (−4.62, −375.88, 25.91) [L port probe]; SP #21's at (3.06, −375.88, 25.91)).
  The symbol panels (up/down/L/P arrows) come from sub 0x0600064E the same way. The host streams
  13-18 also mount instances on the cab itself (the cab's interior panel).
- The static kind-0x19 art by the EXIT lift — `liftswitch_fake`, `liftsymbol_shutdown`,
  `LIFTDOORSBOTH_FAKE(GARAGE)`, `LIFT_(FAKE)` — is scenery: parsemap has no 0x19 case
  [M 0x1d04f0 case list], no object, no use. Those are the fake-lift dressing, not the buttons.

## The mechanism

1. The mode-8 prime leaves each SP in state 6 with its script PLAYING but HELD: stream 9's
   `Event{kind 3, p0 = 0}` at t1 sets SCRIPTINFO+0x9C = p0 − 1 = −1 [M 0x1d94c0:98-100;
   M SP_LoadScript 0x1dbc08 mode-8 arm → state 6]. The cab sits parked at the lobby deck
   (y −368.19 [D keys; L probe across a full loop]).
2. The use pick: `Player_Activate` 0x1a5170 builds a radius-1.0 sphere at `Player_GetHeadPos`
   + 1× the viewer direction [M 0x1a5170:100-108] and dispatches `SP_Activate` on every hit
   object with `obj+0xE4 ≠ 0 && class == 0` [M 0x1a5170:121-122]. The owner handle is stamped
   only on the TOP-LEVEL script's entities (`SP_LoadScript` passes the SP object as `Script_Load`
   param 5 → SCRIPTINFO+0x80 → `Script_EntityStart` 0x1d8d78:108-110 writes entity+0xE4);
   SUB-script entities get owner 0 [M Script_SubScriptStart 0x1db050:28 passes 0] — so the
   station press lands on the OUTER-DOOR entities that flank the button art (r 1.451 spheres at
   z 26.32 [D glist]), not on the button art itself (r 0.20).
3. `SP_Activate` 0x1dc338:72-85 (mask & 4) sets the SP latch +0x2C = 1 and clears
   SCRIPTINFO+0x9C = 0. `SP_Update` case 6 [M 0x1dbda8:193-216] consumes it: no secondary script
   → the same script resumes in place. Out 0 → no channel write, ever [M 0x1dbda8:221-229].
4. The resumed script plays the whole call: the button art swaps to the ON variant at sub t7
   (the indicator light, [L 2026-10-17 fresh-boot PCSX2: dim → bright white]), the cab descends
   (stream 6 keys t180 → t338, y −368.19 → −375.84), the car-park outer doors slide open
   (streams 7/8 keys t422 → t531) and the script's second self-hold (stream 9 `Wait(0x213)` →
   t539) parks everything with the doors open, waiting for the rider. The next press closes the
   doors (t652) and the cab carries back up (t710 → t869). The lift sounds ride stream 0
   (0x28a t2, 0x28c t124, 0x288 t152, 0x289 t422, 0x28c t710, 0x289 t960…) and stream 1
   (0x287) [D nis dump]. Then the loop rewinds and re-holds at t1.
5. ch 0x39 (the exit lift) and the alarm channels are untouched [L: no alarm, ch 0x39 unchanged;
   M out 0]. The exit lift keeps its own panel activation (elevator-buttons-09).

## The port

- The bug: `player_use_doors` fed the SP arm the RENDER CAMERA as the pick origin, while the
  game builds the sphere at Bond's head. The camera hangs ~2.5 behind Bond; the station
  entities sat outside its reach and the buttons never responded (playtest 1017). Fix: the SP
  arm of `player_activate` now takes Bond's head (`Bond::head()`, the `Player_GetHeadPos`
  analog); the doors/triggers/monitors keep the camera origin their reaches were tuned to
  (the game builds them from the same head ray — unifying them re-tunes every door reach and is
  its own task).
- The sub-script entities (button art, symbols, lightmaps) were already created and posed by the
  script runtime (they live in `Scripts::objects` with `src` = the sub's hash) — the draw side
  needs nothing new; `Scripts::entities()`/`Sim::script_entity` are the getters.
- Tests: `crates/nf-view/tests/garage_lift_calls_live.rs` — the call through the real input path
  (light, ride down, doors, hold at the open doors, second press, ride up, channels unchanged)
  and the sister (no press → parked, no light, doors shut, no channels).
