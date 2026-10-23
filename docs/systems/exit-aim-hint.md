# The exit aim hint (`Check_Target`'s class-0x1D trigger arm) + `Trigger_Locked`

## Game rule [M]
`Check_Target` 0x1a7810 runs every 6th tick (`GameState+0x52 % 6`). It decrements the `LoadMsg`
global (0x30cd08, .sdata, next to `DoorMsg` 0x30cd04) on EVERY invocation (:78-79). When the 50-unit
view ray's nearest hit object is class 0x1d (a trigger, `obj+0xff`), and `Trigger_Type` 0x1b2fa8
(the discrete's ext+0x38, stored from placement param +0x2c by `Trigger_Create` 0x1b1720:70) is 10 —
a LOAD_LEVEL exit (`Trigger_LoadLevelCreate` 0x1b2950 forces the param to 10 and shifts the key
gate into the ext+0x26 slot) — and hit dist < 5.0, `LoadMsg == 0`, and the level is not 0x7000009
(:193-195; 09's panels carry their own hint), it re-arms `LoadMsg = FRAME_RATE_INT * 0x14` = 60
whether or not the hint shows (:196). If `Trigger_Locked` 0x1b2fc8 returns 0, it posts
`Txt_BindLabel(0x2000009, 0)` = "Press Action to open door" [D USATxt bank 2 index 9] for 0xb4 =
180 ticks through `Text_AddMsg` (:201-207).

`Trigger_Locked` 0x1b2fc8: type 10 locks on the key gate (ext+0x26) non-zero with its
`switch_channels` entry OFF (:14-22); every other type locks on the lock channel (ext+0x24)
non-zero with its channel ON (:24-29). `Trigger_Activate` 0x1b2318:76-90 inlines the same type-10
test on the load path.

## Port
- `Trigger::kind` documents as ext+0x38 (`Trigger_Type`'s field) and `Trigger::locked(&Channels)`
  (crates/nf-game/src/triggers.rs) implements both arms; `Trigger_Activate`'s two inline key-gate
  checks now call it.
- `Sim::pick_trigger_target` (crates/nf-view/src/scene.rs) picks the ray's nearest trigger AABB,
  wall-occluded, skipping spent (`gone`) and mesh-less triggers [S stand-in shape — the game's ray
  hits the shared collision hitlist; the port's `world.segment` returns no object id].
- The 0x1d arm sits beside the door arm in the `Check_Target` block (scene.rs): `HudState.load_msg`
  = `LoadMsg` (decrement per 6th-frame invocation, re-arm to 60, gate at 0); the hint pushes the
  raw label 0x2000009 into `info_labels` (bound in `hud_tick` like the door hints). The two aim
  arms yield to each other and to the drone pick — the game's ONE hitlist dispatches on the nearest
  object only [M 0x1a79e8-0x1a7bc8].
- 01's exit 0xE8 #18 has no collision [D placements, exit_aim_01_live] so it can never hint there;
  05's zip-wire exit carries real collision ([M] `Player_Activate` 0x1a5170 route c reaches it).

## Tests (crates/nf-view/tests/exit_aim_hint_live.rs)
- `aiming_at_a_load_level_exit_raises_the_hint_once_and_arms_the_cooldown` — real path on
  07000005: aim at the exit → label 0x2000009 for 180 ticks; `load_msg` == 60 and no second raise
  inside the cooldown window. Fails on the earlier stand-in (no trigger arm at all).
- `trigger_locked_arms_match_the_decompile` — both `Trigger_Locked` arms on/off pairs [M 0x1b2fc8].
