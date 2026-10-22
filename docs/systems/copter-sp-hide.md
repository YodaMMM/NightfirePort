# The copter's ScriptPlayer and the SP +0x36 disabled byte

## The game's rule

Every helicopter placement (type 0x2F) owns a ScriptPlayer created at level load:

- `Copter_Create` 0x17d340:66-70 builds the SP with `SP_Create(pos, pos, record+0x3C, record+0x40,
  8, 0, 0, Copter_Register 0x17e530)` [M]. `SP_Create` 0x1dc7e0 fills a synthetic level record —
  scripts at +0x2C/+0x30 (SP+0x1C/+0x20), the mode byte at BOTH +0x34/+0x38 (SP+0x26/+0x28, so
  modes A and B are both 8) — and tails into `SP_CreateScriptPlayer` 0x1db9a8: the mode-8 arm of
  `SP_LoadScript` 0x1dbc08 loads the script and `Script_Play`s it looping (state 6) [M 0x1dbc08's
  mode dispatch; L work/ee_start_05.bin: the pair SPs read modes 8,8, state 6].
- `Copter_Create` 0x17d340:72-74 then stores the SP at copter+0x2C and — **the setter** —
  `if (*(ushort *)(copter+0x70) != 0 && switch_channels[copter+0x70] == 0) SP_HideObj(sp, 1);`
  [M]. copter+0x70 is the placement's param 6 (the gate channel). `SP_HideObj` 0x1dcbf0:10-14
  writes `SP+0x36 = param_2` and cascades `Script_HideObj(script, param_2)` — which sets the
  obj+0xF0 bit 0x10 (hide) on every kind-1..3 stream object of the CURRENT script
  (`SCRIPTINFO+0x2a` selects it) [M 0x1db808].
- `Copter_Update` 0x17dbb8 head: while copter+0x70 is non-zero, a channel that is off returns at
  once; on the first tick the channel reads ON, `SP_HideObj(sp, 0)` clears SP+0x36 (and the
  stream objects' hide bit) and `copter+0x70 = 0` — the unhide happens exactly once, and the
  copter keeps updating even if the channel turns off later [M].
- `SP_Update` 0x1dbda8:57-59 returns while SP+0x36 is set — the SP and its script freeze (the
  primed pose holds; the entities exist but are hidden).

## The evidence on 07000005

- The level carries TWO type-0x2F copter placements, both with scripts 0x06000046/0x06000047
  [D params 4/5; the census in crates/nf-view/tests/interact_07000005_live.rs]: #241 with gate
  param 0 (the ambient fly-by) and #242 with gate param 66 (0x42, the
  SWITCH_Helicopterstart(SC3) plate's channel).
- Their SPs are the snapshot's "twin" pair: one reads SP+0x36 = 1 (0x16ddb40 — #242, hidden at
  level start because ch 0x42 is off), the other +0x36 = 0 (0x16df360 — #241, gate 0 → no hide
  test → plays from load) [L work/ee_start_05.bin; docs/fields/script-player.md].

## The port

- `Copter::create` (crates/nf-game/src/copter.rs) builds the copter records; the SP itself is
  created by `Scripts::spawn_copter_sp` (crates/nf-game/src/nis.rs) at the copter-create site in
  `Drones` init: modes [8, 8], mask 0 → state 6, `hidden = gate != 0 && !ch.on(gate)` — the
  SP_HideObj beat [M 0x17d340:73-74].
- The unhide rides `Copter::tick`'s gate edge: the tick zeroes the gate field and reports
  `unhide`; the caller runs `Scripts::unhide_sp` [M 0x17dbb8 head].
- The `Player.hidden` field IS SP+0x36: `Player::update` returns while it is set [M 0x1dbda8:57-59];
  the prime (the create-time load beat) runs for hidden players too — the game primes first and
  hides second [M 0x1dc7e0 → 0x1db9a8 → 0x17d340:73-74].
- The hidden players' current-script stream objects are surfaced through
  `Scripts::hidden_stream_objects`; the nf-view entity getters (`script_entity_k`, `script_anim`,
  `entities`) skip them — the port-side `Script_HideObj` bit [M 0x1db808]. The draw code needs no
  change: the copter's frozen model simply resolves to nothing while #242 waits on ch 0x42.

## Deviations

- The gate channel's writer on 07000005 is the SWITCH_Helicopterstart plate (a placement
  trigger); the tests drive ch 0x42 through `Channels::set_channel` as the documented harness
  stand-in for walking to the plate.
- The copter's own path timing stays the module's documented [S] deviation (Copter_Move's exact
  per-path advance is not modelled); this task only moves the SP freeze/unfreeze onto the game's
  rule.
