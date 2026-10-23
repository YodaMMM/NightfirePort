# The night-sight / x-ray goggles (viewer+0x236, BL+0x920)

Task `goggles-nightsight-xray` (2026-10-23). The flow controller is `HUD_MonitorNightSight`
0x19e9d8 (820 B), called by `HUD_Update` 0x19a088 **line 19** (right after the crosshair,
before the pane walk), which `Player_Update` 0x193600:76 calls per frame. Port: the machine
is `nf_game::hud_pane::MonitorNightSight` (delivered by the unported-hud-1 census task); this
task wired it into the Sim — `Sim::hud_tick` ticks it (inside `run`, the same tick order),
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

1. `BL+0x94a == 0` → return. **This byte is NOT the goggles-carried flag** (the task brief's
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
  The old hud_pane comment called this "the zoom pane" — wrong reader, fixed in this task.
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

## Port mapping (this task)

| game | port |
|------|------|
| `HUD_MonitorNightSight` tick | `Sim::hud_tick` (first block), machine `HudState::goggles` |
| BL+0x94a | `!self.disabled` (the port's cut-scene/vehicle/gunimp disable) |
| player+0xF4 | 1, or 2 while `drones.death.active` |
| player+0xF6 | `bond.substate` |
| weapon block +0x62 / +0x63 / +0x64 | `gun.weapon` / `gun.switch_request` / `gun.saved_weapon` (now written at every switch start) |
| view-model anim +0xF4 | `gun.switch_state != 0 \|\| gun.fire_state \|\| gun.reload_state != 0` |
| weapon+0x60 bit 0 | `self.aiming` |
| `Input_Action(p, 0xd, 4)` | `Input::goggles_action` (pad action 13 edge via `PadDriver::tick`'s `mask_of`; keyboard N in lib.rs) — stashed to `HudState::goggles_press` in `step_bond`, consumed in `hud_tick` |
| `Player_SetCamMode`/`Camera_Reset` clears | the `nis::Request::PlayerDisable` arm clears mode + pane enables |

Not net-encoded (the `stabilise` precedent — the wire has no lane; MP is gated anyway [M :28]).

## Consumers (for the renderer tasks — the main agent draws these)

The panes' data getters exist in `hud_pane.rs` (`NightSightFx`: the tint/jitter/step per
`HUD_UpdateNightSightPane` 0x19d218 / `HUD_UpdateXRayPane` 0x1a1410). Mode readers elsewhere:
the crosshair 0x19e6d0:80, `Player_LaserPointer` 0x1a65d8:151, `Env_Breath` 0x17be00:52,
`AnimObjectDraw` 0x1c4490:296 (the night-mode character lighting), `BodyGlow_Update` 0x1ac2c8
(mode 2 shows the guards' glow) — none ported yet; they read `Sim::goggle_mode()`.

## Tests

`crates/nf-view/tests/goggles_live.rs` (NF_ISO opt-in), all through `step_bond`:
1. the key cycles 1 → 2 → 0 → 1 with the latch set, after the real request path armed it;
2. the battery drains −1/tick in modes 1/2, recharges +3/tick capped 1800, and at 0 the mode
   drops with both panes cleared;
3. sister: the level start owns no goggles [D census] and the latch-clear request writes
   `switch_request = 0x5d` but never touches the mode or the latch [M :56-58];
4. the arm path auto-stows the item to the pre-goggles weapon and the mode + latch persist.

## Live checks for the main agent

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
