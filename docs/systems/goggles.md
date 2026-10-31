# The night-sight / x-ray goggles (viewer+0x236, BL+0x920)

2026-10-22. The flow controller is `HUD_MonitorNightSight`
0x19e9d8 (820 B), called by `HUD_Update` 0x19a088 **line 19** (right after the crosshair,
before the pane walk), which `Player_Update` 0x193600:76 calls per frame. Port: the machine
is `nf_game::hud_pane::MonitorNightSight` (docs/systems/hud-panes.md), wired into the Sim — `Sim::hud_tick` ticks it (inside `run`, the same tick order),
`HudState::goggles` holds the state, `Sim::goggle_mode()` / `Sim::goggle_battery()` are the
getters for the renderer's consumers.

## State

| game | meaning | evidence |
|------|---------|----------|
| viewer+0x236 (u16) | the mode: 0 off, 1 night sight, 2 x-ray | [M 0x19e9d8:54, :133, :106] |
| BL+0x920 (f32) | the battery, 0..1800 | [M :155-162] |
| BL+0x961 (u8) | the "goggles were on" latch that arms the mode cycle | [M :67, :82; the only writer besides `Player_StandAtNewPosition` 0x190d88:87 which clears it] |
| night pane enable `*(BL+0x7e0)+0x104` (+0x102 zeroed beside it) | the HUD pane enable pair | [M :61-62, disasm 0x19eab4-0x19eac8] |
| x-ray pane enable `*(BL+0x7e0)+0x204` (+0x202) | the second pane block sits at +0x1e0 | [M :146-149, disasm 0x19ebd4-0x19ebf0] |

## The guard ladder [M 0x19e9d8:17-49, each in decompile order]

1. `BL+0x94a == 0` → return. **This byte is NOT the goggles-carried flag** (an earlier
   guess): its writers are `Player_Init` 0x1910f8:150 (=1), `Player_Enable` 0x195498:51 (=2),
   `Player_Disable` 0x195f70:16 (=0), and `Player_Update` 0x193600:79-88 runs the player only
   while it is 1 (values >1 tick down one per frame — a re-enable delay). It is the
   player-update countdown.
2. `player+0xF4 != 1` → return (1 = playing; 2/3 = the death states [M 0x190810:66]).
3. `MPSettings+0x384 != 0` → return — SP only [M :28-34, disasm 0x19ea2c reads 0x2a4920].
4. `player+0xFF == 0x12` → return (the class-0x12 body).
5. `player+0xF6` (the movement substate) must be 0 or 3..=5 [M :34-45].
6. `glb_viewer[idx] == 0` → return.

## The arm path — holding the goggles item [M :50-73, disasm 0x19ea84-0x19eb14]

While the held weapon's id byte (`player+0xDC` weapon block +0x62) is 0x5D (']'):

- gate: `*(short*)(BL+0x7e8 → +0xF4) != 0` → return. BL+0x7e8 is the FIRST-PERSON VIEW MODEL
  object (`Player_InitWeapon` 0x1a7ef8:64 stores it) and +0xF4 is its current anim id —
  `Player_WeaponSelect` 0x1a2cc0 writes 8/3/1 there (the variant-swap/draw/holster states).
  The old hud_pane comment called this "the zoom pane" — wrong reader, since fixed.
- `viewer+0x236 = 1` (mode 1 forced, every frame the item is out and the anim is idle);
- night pane enable = 1 (+0x102 = 0);
- `BL+0x961 = 1`;
- `weapon+0x63 = weapon+0x64`, then if the result is 0x47 ('G') or 0x5d..0x5e → `weapon+0x63
  = 1` [M disasm 0x19ead0-0x19eb14]. **+0x64 is the switch source** — `Player_SetWeaponAnimObj`
  0x1a2e38:197-198 (disasm 0x1a31e8-0x1a31fc) and `Player_WeaponChange` 0x1a6f80:142 write
  `+0x64 = +0x62` at every switch start. So while the goggles item is out, the request slot is
  continuously restored to the weapon the goggles were drawn from (specials clamp to 1): the
  goggles item AUTO-STOWS as soon as its draw ends. The mode is a VIEW state, not item-bound —
  it persists after the stow (nothing in the switch path writes +0x236), and the battery keeps
  draining. Port: `MonitorOutput::armed` → `switch_request = clamp_item_class(saved_weapon)`;
  the port's switch machine now writes `saved_weapon` at every switch start.
- the early return skips the battery tail — no drain/recharge while the item is out.

## The input path — the goggles key [M :74-152]

`Input_Action(player, 0xD, 4)` — action 13, the press edge (pad: style-table action 13, e.g.
L2 in style 0 / d-pad-right in styles 2-3 [D styles_gen]; keyboard: N, the port's bind).

- action 0 or the aim bit set (`weapon+0x60 & 1` — the aim-mode bit, action 19's toggle [M
  `Player_Weapon` 0x1a24c8:47-67]; `Player_WeaponChange` refuses switches while it is up) →
  no cycle, fall to the battery tail.
- latch clear (`BL+0x961 == 0`) → `weapon+0x63 = 0x5d`: the press REQUESTS the goggles item
  (the switch machinery draws it; the request itself has **no owned gate** [M :82-84]). Port:
  `MonitorOutput::item_class_override` → `gun.switch_request`.
- latch set → the cycle: 0 → 1 (night pane on), 1 → 2 (night pane off, x-ray pane on), 2 → 0
  (both off) [M :86-152].

## The battery tail [M :154-193]

- mode == 0: `battery += 3·FRAME_RATE_MUL`, **capped at 1800 via the EE's MIN.S** — disasm
  0x19ec68 is word 0x46020029 = COP1.S func 0x29 = `MIN.S f0, f0, f2` (op 0x29 per ps2tek's
  FPU table; Ghidra's `(int)x * (x < 1800) | ... * 0x44e10000` C form is its select expansion,
  the same shape as the min-chains in `AINetwork_DoAStarPath`). No integer truncation.
- mode != 0: `battery -= FRAME_RATE_MUL` (1.0 at the fixed 60 Hz).
- `battery <= 0` with mode 1..2 → battery = 0, mode = 0, both pane enables cleared [M :163-193].
- Init: no writer of BL+0x920 outside this function was found — it starts 0 and charges up
  [S: absence of a writer in the whole-ELF grep of `+ 0x920)`].

## The clears (the cam/pane sites)

- `Player_SetCamMode` 0x195928:15,23,27,40 — every camera-mode change clears viewer+0x236 [M].
- `Camera_Reset` 0x1b9af0:50 — clears it [M].
- `HUD_UpdateOICWPane` 0x19cef8:25 (while the OICW pane is up) and `HUD_UpdateSecCamPane`
  0x1a1538:10 (param_2+0x1c) clear it [M]. The port has no OICW-zoom/sec-cam mode flow yet;
  the port clears the mode on the cut-scene `PlayerDisable` transition (the SetCamMode arm).

## The census — who owns the goggles [D]

**No level grants them.** `Player_InitWeapon` 0x1a7ef8's per-level loadout switch
(0x7000001..0x700001b, 0x700004a) never equips 0x5D/0x5E; the disc's pickup placements (map
block 0x1A, type 0xF0, id = param 1) were scanned across every FILES.BIN archive (99 bins,
2230 map entries, 570 placed pickups, ids 0x00..0x52): **zero hits for 0x5D/0x5E**; the
script route cannot grant weapons (no op reaches `Player_EquipWeapon`; the kind-7 NIS
EntityStart scan: 4985 ops, only 9 create pickups, ids 0x00/0x06/0x2C); `PickupMatrix`
0x2b9100 maxes at 0x52; `Upgrade_Weapon` never returns 0x5D. The ONLY grantor on the disc is
the all-weapons cheat handler `C_CHCHWEAP_Handler` 0x215dc8 (equips 0x4A..0x5E with 999
rounds). The weapon_data records 0x5D/0x5E are minimal (no name label, no models — [L]
`ee_start_05.bin` @ weapon_data 0x2bf150 + the USATxt.dat bank walk: no "Night Sight"/"X-Ray"
string exists; the control-hint texts say only "Vision Mode").

Consequence: in normal SP play the goggles flow is unreachable — the sister test pins the
level-start ownership (no 0x5d) and that the latch-clear request never touches the mode or
the latch [M :56-58]. The request path has no owned gate in the decompile, so whether the
real game arms the view on a goggles-less level after the key press (the switch machinery
drawing an unowned 0x5D) is **open** — see the live checks below.

## Port mapping

| game | port |
|------|------|
| `HUD_MonitorNightSight` tick | `Sim::hud_tick` (first block), machine `HudState::goggles` |
| BL+0x94a | `!self.disabled` (the port's cut-scene/vehicle/gunimp disable) |
| player+0xF4 | 1, or 2 while `drones.death.active` |
| player+0xF6 | `bond.substate` |
| weapon block +0x62 / +0x63 / +0x64 | `gun.weapon` / `gun.switch_request` / `gun.saved_weapon` (now written at every switch start) |
| view-model anim +0xF4 | `gun.switch_state != 0 \|\| gun.fire_state \|\| gun.reload_state != 0` |
| weapon+0x60 bit 0 | `self.aiming` |
| `Input_Action(p, 0xd, 4)` | `Input::goggles_action` (pad action 13 edge via `PadDriver::tick`'s `mask_of`; no keyboard key — the disc grants the goggles nowhere) — stashed to `HudState::goggles_press` in `step_bond`, consumed in `hud_tick` |
| `Player_SetCamMode`/`Camera_Reset` clears | the `nis::Request::PlayerDisable` arm clears mode + pane enables |

Not net-encoded (the `stabilise` precedent — the wire has no lane; MP is gated anyway [M :28]).

Port behaviour today: the switch machine applies the 0x5D request like any other (no owned
check, matching the decompile), so the pad's action-13 key arms night sight on every on-foot
SP level. The keyboard has no bind (unchanged decision, lib.rs).

## Sounds

None. `HUD_MonitorNightSight`, both pane updates and `BodyGlow_Update` call no `Sound_*`
function [M 0x19e9d8, 0x19d218, 0x1a1410, 0x1ac2c8 — no Sound_ call in any]. The 0x5D item
has no models and a 0 holster script [D weapon_data 0x5d; goggles_live.rs]; whatever its
draw script's events sound is the switch machine's ordinary path, nothing goggles-specific.

## What each mode draws (goggles-render)

All gated on viewer+0x236 (`Sim::goggle_mode()`); the panes additionally on their enable byte.

**Night sight (1)**
- Pane 7 (`NightSightSprInfo` 0x2bd7b0, 6 sprites [D; nf-formats tests/goggles_panes.rs]):
  sprite 0 = the 64×64 noise 0x0300004A over 512×448, additive, colour word
  `(Rand(0x18)+7) | 0xff0000` (green, alpha 7..30) and v = `Rand(h−1)` on every other frame
  (`Scrl_247` parity); sprite 1 = 0x03000002 white 0xffffff7f additive; sprite 2 = 0x03000002
  magenta 0xff00ffff SUBTRACTIVE (flags 0x200) — red and blue out: the green image; sprites
  3/4/5 = the battery bar back (0x0300017F row 0), the bar (row 16) at width/uv-width
  `max(1, (int)(battery/1800·200 + 28))`, the icon 0x0300017E at (108, 48)
  [M HUD_UpdateNightSightPane 0x19d218]. The pane's `Rand_Rand` draws come off the shared
  stream inside the tick (`GogglePanes::night_update`, hud_tick) — the game's order.
- The lamp: SP only, every frame, `Light_Create(32.0, 2.5, glb_viewer+0xC0, 0xff,0xff,0xff,
  ch 0, life 0, kind 1, flicker 0)` — a one-tick white light at the eye [M 0x19d218:48-51].
- The crosshair turns 0x208020ff (green) [M HUD_UpdateCrossHair 0x19e6d0:80-85].
- The weapon laser sight's beam object takes gfx 0x2000131 "lazer" instead of 0x2000133
  "watch_lazer" [M Player_LaserPointer 0x1a65d8:151-155] (scene.rs `NIGHT_BEAM_SLOT`). The
  beam's alpha flicker (`(a + 0x3f + FRand(128))/2` here, `a + Rand(9) − 4` in [0x10, 0x30]
  otherwise) is not modelled in either mode (pre-existing, sg5-laser-draw).
- `View_SetupRenderModes_Tweak` 0x1e5888:13-20 quarters the alpha of objects with obj+0xF0 &
  0x20000 in mode 1 (`alpha = obj+0x106 >> 2`; disasm 0x1e588c `lui v1, 2`, 0x1e58b8 `lhu
  +0x236`, 0x1e58d8 `srl s1, 2`) [M]. The bit's only source is the glist flags word
  (`celglist+0x44` → obj+0xF0 by `Control_SetGList` 0x134960 and the creators); no ACTION.ELF
  store ORs it in [M grep of every `+ 0xf0) =` store] and no glist in the 99 FILES.BIN
  archives carries it [D nf-formats tests/night_quarter_census.rs] — on this disc no object
  draws differently. Port: `nf_game::view::tweak_alpha` (unit-tested); no draw site needs it.

**X-ray (2)**
- Pane 15 (`XraySprInfo` 0x2bddb0, 5 sprites): sprite 0 = the 32×32 noise 0x03000098,
  additive, `(Rand(0x18)+7) | 0xffffff00` (white, alpha 7..30), v = `v+1 & 0x1f` per frame;
  sprite 1 = blue 0x0000ffff additive; sprites 2/3/4 = the battery bar (sprite 3 the width)
  [M HUD_UpdateXRayPane 0x1a1410].
- The guards' body glow (`BodyGlow_Update` 0x1ac2c8, per guard, the record every spawn makes
  [M BodyGlow_Create 0x1ac158]): visible while a viewer is in x-ray OR has the OICW zoom pane
  (pane 17's enable byte HUD+0x244 — the AIMS-20 thermal scope; the old "dead/spectator"
  reading was wrong) up; distance owner obj+0x30 → glb_viewer+0xC0, > 10.0 hides (the OICW
  pane forces 0.0); alpha byte `(1 − d·0.1)·count·128/256` on every part; count 0x100, −1 per
  tick while the owner's health ≤ 0, deleted below 1. Parts: the main object 0x200025f
  "lazer_muzzle" (billboard ×6.0 at bone 5; keeps its 0x400 bit → x-ray only) and 15 children
  0x200055a "BoneGlow" on `GlowBones` (each `Mat_Align2Dir(bone − parent, viewer − bone)`,
  scale (1.5, 1, length); bone 0 measures against bone 4). The parts carry obj+0xF0 & 0x2000 →
  `Game_Draw`'s list 2, drawn after `psiClearZ` [M 0x1e6f50:157-176]: over the world — the
  through-walls look. Port: `Scene::glow_targets` (nearest GLOW_GUARDS = 8), the pooled slots
  `GLOW_SLOT_BASE`, pass 6 via `LIST2_NODE`, the alpha in the palette colour slot; the count in
  `BodyGlow::count`, ticked in `run` after `hud_tick`.
- The x-ray-only glists: glist flags 0x400 are submitted to a view only while its +0x236 == 2
  [M View_AddObjects 0x1e62d8:38-47, View_AddForcedObjects 0x1e5fb8:79-84, the cel/object
  sub-entry tests in View_DrawCelGlist 0x1e6538 / View_DrawObjectGlist 0x1e6780] — the port
  had read the bit as "mirror-only". Created ones on the disc [D nf-formats
  tests/xray_only_census.rs]: 07000003 "HOTOBJECT" (kind 0x20 breakable), 0700000a "laserbeam"
  ×10 and 0700000b "laserbeam-large" ×8 (kind 0xeb triggers), 07000014 "laserbeam" ×8 (0xea) —
  tripwire beams only the goggles show. Port: one slot each (`XRAY_SLOT_BASE` + placement
  index) at obj+0x90 — `View_RotTransMatrix` for the 0x20 breakable and the trigger family
  (`object_matrix_for_class`); a breakable hides when dead, a trigger when deleted or its
  lock channel fired with the hide byte set (`Trigger_Update` 0x1b1a78:47-55's |= 0x30).
  The keyed triggers move: `Trigger_Create` 0x1b1720:56-87 stores the 0x19 key list
  (`m_paths[extra[8]]`), last = count − 1 and the step last/rate (`extra[7]` = count | rate<<16),
  and every `Trigger_Update` after the lock return poses obj+0x90 with `Spline_Interp(t, keys,
  0, last, obj+0x90, 0, 2)`, then `t += step`, `t = 0` once `(float)last < t` [M 0x1b1a78:60-70]
  — one pass every `rate` ticks; a Touch updates inside `Trigger_Calc`'s ProcessTouch window
  [M 0x1b1900:50-71], so it moves too. On the disc [D night_quarter_census]: 0700000a 3 of the
  10 beams (#606/#614/#625, rate 240), 0700000b all 8 (rates 150-420), 07000014 all 8 (rate
  480), each a 3-key list whose key 0 is the placement pose. Port: `triggers::Motion` — the
  volume (`set_tris`) and the x-ray draw follow the pose. The "IR FootPrint Left/Right" pair
  (0x2000567/0x2000568, flags 0x721) sits on 0x2000-skipped placements in every archive and
  has no creator in ACTION.ELF (neither hash nor name in the decompile) — unspawned.
- Breath: `Env_Breath` 0x17be00 spawns its x-ray puff variant (gas +0xF0 |= 0x400 → drawn
  only in mode 2) whenever its env gate holds — `GlobalEnv == 0 || GlobalEnv+0x4b != 0 ||
  cel+0x3c & 0x10000000`, where +0x4b is the Env_Create drop type (0 = snow). So off the
  snow levels the guards breathe ONLY in x-ray; on 07000005-08 (drop type 0) the plain puff
  shows in every mode, the x-ray one only in underwater cels. Wired from the guard pass
  (`Sim::guard_breath`, docs/systems/env-breath-ripple.md); `Gas::xray_only` gates the draw.
- The guards' laser ray (`AnimObjectDraw` 0x1c4490:259-311 [M]): a drawn anim object whose
  held byte (obj+0xDC → +0x62) is 0x18 gets `AnimGetBoneWorldTrans(obj, 8, 3)` — datum 8+3 —
  as the start, start + (−100, 0, 0)·datum rows as the far end, `Collide_RayIntersect(…, 8)`
  clamps it to the first hit, and `View_DrawGlist(gfx, start, spherical(start − end), (1, 1,
  |start − end|))` draws gfx 0x2000131 when the DRAWING viewer's +0x236 == 1, else 0x2000133.
  Carriers [D goggles_draws_live]: weapon 0x19 on 0700000b/11/12/13 (10/10/10/4 guards, skins
  0x5000004/6/8, datum 11 on bone 23); the 0x19 → 0x18 swap is the captains' (Attack 0x164210 /
  AimStand 0x166178 / AimStandFire 0x166328) — 2 on 0b, 3 on 12. Port: `Scene::guard_beams`,
  pooled `GUARD_BEAM_BASE` slots (GUARD_BEAMS = 4 rays × the two sprites).

Render aid: `nf-view <iso> <level> --frame N --goggles 1|2 --shot out.png` presses the real
goggles key in the shot's second half until the mode reads 1/2.

## Tests

`crates/nf-view/tests/goggles_live.rs` (NF_ISO opt-in), all through `step_bond`:
1. the key cycles 1 → 2 → 0 → 1 with the latch set, after the real request path armed it;
2. the battery drains −1/tick in modes 1/2, recharges +3/tick capped 1800, and at 0 the mode
   drops with both panes cleared;
3. sister: the level start owns no goggles [D census] and the latch-clear request writes
   `switch_request = 0x5d` but never touches the mode or the latch [M :56-58];
4. the arm path auto-stows the item to the pre-goggles weapon and the mode + latch persist.

`crates/nf-view/tests/goggles_render_live.rs` (NF_ISO opt-in, the key pressed through
`step_bond`): pane 7's fields + the viewer lamp + the HUD quads, parked again at mode 0
(sister); the x-ray glow pool at 3.0 from a guard (the alpha formula, every part posed on the
guard), empty at 12.0 and in night sight (sisters); a dead guard's count-down; 0700000a's ten
beams drawn only in mode 2 (mode 0/1 sisters); the per-level x-ray slot census (03 = 1,
14 = 8, 05 = 0); the laser sight's 0x2000131/0x2000133 swap. Unit tests: `hud_pane`
goggle_pane_tests (the pane updates' shared-stream draws, the crosshair colour, the battery cap
on the grown value, the arm's pane enable), `body_glow` (the count-down, the child matrix, the
parent pick). nf-formats: `goggles_panes` (both SprInfo tables, numbers) and
`xray_only_census` (the 0x400 placements + the assets in all 21 SP archives).

`crates/nf-view/tests/goggles_draws_live.rs` (NF_ISO opt-in): the snow-level plain breath in mode 0 (spawn alpha 128, static 0.05, at the head; dead guards make none), the x-ray-only breath off the snow levels (none in mode 0; alpha 200, growing, drawn only in mode 2), the 0x19 carrier/captain census, a captain's real 0x19 → 0x18 swap and its ray (datum 11 start, −X row, ≤ 100, sprite swap in mode 1; a 0x19 holder draws none), the keyed tripwires (step last/rate, key 0 = the placement, the volume and the x-ray draw follow, the wrap; 0a 3 / 0b 8 / 14 8). nf-formats `night_quarter_census` (no glist carries 0x20000; the 0xDF drop types; the keyed trigger placements). Unit tests: env breath variants/origin, `view::tweak_alpha`, `triggers` motion + lock hold.

## Live checks (open)

1. Fresh boot (no save-state resume), a level WITHOUT the goggles (e.g. 07000005), press the
   goggles key once: watch viewer+0x236 (expect 0 until/unless the item equips) and the
   weapon block +0x62/+0x63/+0x64 — does the game switch to 0x5D unowned (the decompile's
   request path has no owned gate), and does the mode arm? Expected from [M]: +0x63 = 0x5d,
   the switch applies, the arm forces mode 1 for the frames the item is out, the item
   auto-stows to +0x64's value (clamped {0x47,0x5d,0x5e} → 1), mode persists.
2. Same boot, with the all-weapons cheat entered so the item is owned: hold 0x5D — expect
   mode 1 pinned while out, BL+0x920 frozen (the early return), then after switching away
   the battery drains −1/frame and the mode drops at 0 (1800 frames later).
3. `gdbwatch` on viewer+0x236 during an OICW zoom / a cut-scene / the sec-cam: expect the
   clears at 0x19cef8:25 / 0x195928 / 0x1a1538:10.
