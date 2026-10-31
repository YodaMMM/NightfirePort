//! The HUD pane state machines — the parts of
//! ACTION.ELF's pane functions that are *state and flow* (the drawing stays on the view
//! side; the sprite tables live in `nf-formats::hud`). Evidence map: docs/systems/hud-panes.md.
//!
//! Three machines, all per the decompile:
//!
//! - [`OicwBoot`] — the OICW's COMMAND.COM boot list. `HUD_CreateOICWPane` 0x19cc90 plants
//!   mode 0 / timer 0x96 (SP) or mode 2 / timer 0xfdd5 (MP); `HUD_UpdateOICWPane` 0x19cef8
//!   runs it each frame: the 5 lines rise line by line (one per 30 ticks), flash, then the
//!   pane parks hidden (mode 2) and resets on the next zoom-in.
//! - [`NightSightFx`] — the per-frame pane data of `HUD_UpdateNightSightPane` 0x19d218
//!   (goggle mode 1), `HUD_UpdateXRayPane` 0x1a1410 (mode 2) and the sec-cam wash
//!   `HUD_UpdateSecCamPane` 0x1a1538. Pure getters over the mode + fade state.
//! - [`MonitorNightSight`] — `HUD_MonitorNightSight` 0x19e9d8, the goggles flow controller
//!   `HUD_Update` 0x19a088 calls directly: the mode cycle 0 → 1 (night) → 2 (x-ray) → 0 on the
//!   select action, the goggles-equipped force-to-night path, and the `BL+0x920` fade timer.
//!
//! The game's pane `Rand_Rand` calls (the tint pulse and scanline jitter) draw from the SAME
//! `Rand_Rand__FUi` stream the drones use [M 0x19d218:18,24 / 0x1a1410:15] — under goggles the
//! game's AI stream advances 1-2 draws per frame. The port keeps the HUD off the sim's RNG (the
//! same decision as the redeemer pane in `nf-view::hud`, "a HUD draw must not move the game's
//! draws") and uses a per-frame hash here: an [S] deviation, NOT the game's behaviour (review
//! 2026-10-21 corrected the earlier claim that the HUD had its own stream). Determinism: fixed 60 Hz, no wall clock; `FRAME_RATE_MUL` = 1.0 at 60 Hz
//! [M grenade.rs:171-173's note] folds away.

/// One boot line's draw state. `layer` follows the game's sprite `+0x38` byte (0x1c = the
/// pane's visible layer, 0xff = hidden — `HUD_Update` 0x19a088 hides panes with 0xff).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OicwLine {
    pub text: &'static str,
    pub layer: u8,
    pub x: i16,
    pub y: i16,
}

/// The five boot lines, in the game's order [M `HUD_CreateOICWPane` 0x19cc90's five
/// `sprintf`+`Sprite_SetText` pairs at 0x19cdd4-0x19ce50; the fifth is `DAT_0030ccc8` =
/// `"OK\n"` [D the ACTION.ELF image]].
pub const OICW_LINES: [&str; 5] = ["COMMAND.COM\n", "LOAD BIOS\n", "MEMORY SET\n", "SYSTEM STATUS\n", "OK\n"];

const OICW_VISIBLE_LAYER: u8 = 0x1c;
const OICW_HIDDEN_LAYER: u8 = 0xff;
const OICW_X: i16 = 0x32; // the sprite x the mode-0 loop writes [M 0x19cef8:57]
const OICW_LINE_STEP: i16 = 0x11; // 17 px per line [M :53-58]
const OICW_BASE_Y: i16 = 408; // the loop's 408.0 - visible*17 origin [M :53]

/// The OICW boot state (`OICW_mode` / `OICW_timer`, the game's globals — one per BL).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OicwBoot {
    mode: u8,
    timer: i32,
    mp: bool,
}

impl OicwBoot {
    /// `HUD_CreateOICWPane` 0x19cc90's tail: SP plants mode 0 / timer 0x96 (the boot list
    /// from the top); MP plants mode 2 with timer 0xfdd5 (the value is never read while
    /// mode == 2) [M 0x19cc90:50-62].
    pub fn new(mp: bool) -> Self {
        if mp {
            Self { mode: 2, timer: 0xfdd5, mp }
        } else {
            Self { mode: 0, timer: 0x96, mp }
        }
    }

    pub fn mode(&self) -> u8 {
        self.mode
    }

    /// `HUD_UpdateOICWPane` 0x19cef8, one frame. `pane_enabled` is the HUDINFO entry's
    /// enable byte (the pane only animates while the OICW zoom is up).
    pub fn tick(&mut self, pane_enabled: bool) {
        if self.mp {
            return; // the whole function is gated on MPSettings+0x384 == 0 [M 0x19cef8:15]
        }
        if !pane_enabled {
            if self.mode != 2 {
                self.mode = 0;
                self.timer = 0x96;
            }
            return; // [M :15-19] the disabled pane only resets
        }
        match self.mode {
            1 => {
                self.timer -= 1;
                if self.timer < 1 {
                    self.mode = 2;
                    self.timer = 0x1e;
                }
            }
            0 => {
                self.timer -= 1;
                if self.timer < 1 {
                    self.mode = 1;
                    self.timer = 0x1e;
                }
            }
            _ => {} // mode 2 is steady; the timer is not read [M :52-64]
        }
    }

    /// The five lines' draw state for this frame. Layer 0xff = don't draw.
    pub fn lines(&self) -> [OicwLine; 5] {
        let mut out = [OicwLine { text: "", layer: OICW_HIDDEN_LAYER, x: OICW_X, y: OICW_BASE_Y }; 5];
        let visible = match self.mode {
            0 => (5 - (self.timer / 0x1e)).clamp(0, 5) as i16, // [M :53 `5 - OICW_timer/0x1e`]
            1 => 5,
            _ => 0,
        };
        for (k, line) in out.iter_mut().enumerate() {
            let k = k as i16;
            line.text = OICW_LINES[k as usize];
            // y: the block rises — 408 - 17*visible + 17*k [M :53-58, y advances 0x11 per line]
            line.y = OICW_BASE_Y - OICW_LINE_STEP * visible + OICW_LINE_STEP * k;
            line.layer = match self.mode {
                0 => {
                    if k < visible {
                        if visible == 2 && k == 1 && self.timer.rem_euclid(8) < 4 {
                            OICW_HIDDEN_LAYER // the LOAD BIOS blink [M :60-71, timer % 8 < 4]
                        } else {
                            OICW_VISIBLE_LAYER
                        }
                    } else {
                        OICW_HIDDEN_LAYER
                    }
                }
                1 => {
                    if self.timer.rem_euclid(4) < 2 {
                        OICW_HIDDEN_LAYER // the all-lines flash [M :24-45, timer % 4 < 2]
                    } else {
                        OICW_VISIBLE_LAYER
                    }
                }
                _ => OICW_HIDDEN_LAYER, // mode 2 parks the pane [M :52-64]
            };
        }
        out
    }
}

/// The nightsight / x-ray pane data (viewer `+0x236` mode + `BL+0x920` fade). `HUD_Update`
/// 0x19a088 parks both panes (their update fns clear their enable byte) unless the mode
/// matches [M 0x19d218:34-36 / 0x1a1410:15-17].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NightSightFx {
    pub mode: u16,
    /// `BL+0x920`, the shrink/fade timer `MonitorNightSight` drives (0..1800).
    pub fade: f32,
    /// `Scrl_247`, the pane's own tick parity counter [M 0x19d218:41-47].
    pub scrl: u8,
}

impl NightSightFx {
    /// Sprite 0's colour word while mode == 1 (nightsight): `(Rand(0x18) + 7) | 0xff0000`
    /// [M 0x19d218:39-40]. `frame` feeds the pane's private hash (see the module head).
    pub fn night_tint(&self, frame: u32) -> u32 {
        let r = hash_frame(frame, 1) % 0x18;
        (r + 7) | 0xff0000
    }

    /// Sprite 0's colour word while mode == 2 (x-ray): `(Rand(0x18) + 7) | 0xffffff00`, and
    /// the sprite's v counter (+0x32) advances by 1 wrapping at 0x1f [M 0x1a1410:17-30].
    pub fn xray_tint(&self, frame: u32) -> u32 {
        let r = hash_frame(frame, 2) % 0x18;
        (r + 7) | 0xffffff00
    }

    /// The nightsight scanline jitter: on odd `Scrl_247` ticks sprite 0's v re-rolls to
    /// `Rand(h − 1)` [M 0x19d218:43-47]. Returns the new v on jitter ticks.
    pub fn night_jitter_v(&mut self, sprite_h: u16, frame: u32) -> Option<u16> {
        self.scrl = self.scrl.wrapping_add(1);
        if self.scrl & 1 != 0 {
            let h = u32::from(sprite_h).max(2);
            Some((hash_frame(frame, 3) % (h - 1)) as u16)
        } else {
            None
        }
    }

    /// The x-ray v counter step: `+1 & 0x1f` every frame [M 0x1a1410:24-30].
    pub fn xray_next_v(v: u16) -> u16 {
        v.wrapping_add(1) & 0x1f
    }

    /// The brightness step sprite 1's h/v: `max(1, 28 + fade · (200/1800))` [M
    /// 0x19d218:63-79 / 0x1a1410:33-49 — `0.00055555557 * 200.0` and the `<< 0x10 < 1`
    /// zero-clamp to 1]. Both panes compute it identically.
    pub fn step(&self) -> u16 {
        let v = (self.fade * 0.00055555557 * 200.0 + 28.0) as i32;
        let v = if v << 16 < 1 { 1 } else { v };
        v as u16
    }

    /// The sec-cam wash colour word [M `HUD_UpdateSecCamPane` 0x1a1538:12-21]:
    /// `0x7f7f7f78` on the first half of the 32-field display counter, `0x7f7f7f00` on the
    /// second. `field` is `GameState+0x34 & 0x1f`; the port feeds its own frame counter
    /// (`& 0x1f`) — `// [S] unverified` that the two counters agree phase-for-phase (the
    /// values and the split are [M]).
    pub fn sec_cam_tint(field: u32) -> u32 {
        if field & 0x1f < 0x10 {
            0x7f7f7f78
        } else {
            0x7f7f7f00
        }
    }
}

/// The goggles flow controller (`HUD_MonitorNightSight` 0x19e9d8).
///
/// The port has no goggle item wiring yet (nf-view scene.rs notes goggles are not ported), so
/// the machine is delivered over explicit inputs and outputs; the item-class side (`item+0x63`
/// next-class bookkeeping at 0x19e9d8:56-66) surfaces as [`MonitorOutput::item_class_override`]
/// for the inventory seam (`// [S] unverified` until a task wires the gadget inventory).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct MonitorNightSight {
    /// The viewer's `+0x236` goggle mode: 0 off, 1 nightsight, 2 x-ray.
    pub mode: u16,
    /// `BL+0x920`, the shrink fade (0..1800 while off; drains while a mode is up).
    pub fade: f32,
    /// `BL+0x961`, the "goggles were on" latch that arms the mode cycle.
    pub was_goggles: bool,
    /// The night pane object's enable pair (`+0x102/+0x104` words the transitions set).
    pub night_pane_on: bool,
    /// The x-ray pane object's enable pair (`+0x202/+0x204`).
    pub xray_pane_on: bool,
}

/// What one tick decided, for the callers to apply.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct MonitorOutput {
    /// The goggle mode after this tick.
    pub mode: u16,
    /// The item-class write the game makes at `item+0x63` (`0x5d` — select switches to the
    /// goggles item) when the cycle is not armed [M 0x19e9d8:82-83].
    pub item_class_override: Option<u8>,
    pub night_pane_on: bool,
    pub xray_pane_on: bool,
    /// The goggles-equipped arm fired this tick (mode forced to 1, latch set, night pane
    /// enabled — the game returns right after [M 0x19e9d8:49-72]). The caller applies the
    /// arm path's `item+0x63 = item+0x64` restore (the auto-stow of the goggles item) here.
    pub armed: bool,
}

/// The per-tick inputs (each mirrors the decompile's guard/branch reads).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct MonitorInputs {
    /// `BL+0x94a` — the player-update countdown byte (`Player_Init` 0x1910f8:150 plants 1,
    /// `Player_Enable` 0x195498:51 plants 2, `Player_Disable` 0x195f70:16 plants 0,
    /// `Player_Update` 0x193600:79-88 runs only while it is 1 and ticks >1 down). NOT the
    /// goggles-carried flag — the disc grants the goggles nowhere (see docs/systems/goggles.md).
    pub hud_active: bool,
    /// `player+0xf4` — the on-foot viewer state (must be 1) [M 0x19e9d8:25].
    pub viewer_state: u16,
    /// `MPSettings+0x384` — the flow is SP-only [M :31].
    pub mp: bool,
    /// The held weapon's class byte (`weapon+0x62` high byte; 0x12 blocks) [M :35].
    pub weapon_class: u8,
    /// `player+0xf6` — the gadget sub-state; only 0 and 3..=5 pass [M :37-47].
    pub gadget_sub: i16,
    /// The held weapon is the goggles (class `']'` = 0x5d) [M :49].
    pub goggles_equipped: bool,
    /// `BL+0x7e8` → `+0xf4`: the first-person view model's current anim id (non-0 while a
    /// draw/holster/fire anim plays — `Player_WeaponSelect` 0x1a2cc0 writes 8/3/1 there;
    /// `Player_InitWeapon` 0x1a7ef8:64 stores the view model at BL+0x7e8). The goggles arm
    /// refuses while any view-model anim is up [M 0x19e9d8:51-53]. (The old comment called
    /// this the zoom pane — wrong reader: Player_WeaponSelect is the writer.)
    pub zoom_pane_on: bool,
    /// `Input_Action(player, 0xd, 4)` — the select action this tick [M :69].
    pub toggle_action: bool,
    /// `weapon+0x60` bit 0 — when set, the action does not cycle [M :72-76].
    pub weapon_bit0: bool,
}

impl MonitorNightSight {
    /// One frame of `HUD_MonitorNightSight` 0x19e9d8. Returns `None` when the guards bail
    /// (the game returns without touching anything).
    pub fn tick(&mut self, i: &MonitorInputs) -> Option<MonitorOutput> {
        // The guard ladder [M 0x19e9d8:18-47], in the decompile's order.
        if !i.hud_active || i.viewer_state != 1 || i.mp || i.weapon_class == 0x12 {
            return None;
        }
        if i.gadget_sub != 0 {
            if !(0..=5).contains(&i.gadget_sub) || i.gadget_sub < 3 {
                return None;
            }
        }
        let mut out = MonitorOutput { mode: self.mode, ..Default::default() };
        out.night_pane_on = self.night_pane_on;
        out.xray_pane_on = self.xray_pane_on;
        if i.goggles_equipped {
            // The goggles path: force nightsight while the zoom pane is down [M :45-56].
            if i.zoom_pane_on {
                return Some(out);
            }
            self.mode = 1;
            self.was_goggles = true;
            // The arm writes the night pane's enable pair (+0x104 = 1, +0x102 = 0) [M 0x19e9d8:
            // 55-62] — the state, not only this tick's output (the pane stayed dark after an arm).
            self.night_pane_on = true;
            out.mode = 1;
            out.night_pane_on = true;
            out.armed = true;
            return Some(out);
        }
        if i.toggle_action && !i.weapon_bit0 {
            if !self.was_goggles {
                // The latch is clear: the select action switches to the goggles item [M :56-58].
                out.item_class_override = Some(0x5d);
            } else {
                // The cycle 0 → 1 → 2 → 0 with the pane enables [M :72-101].
                match self.mode {
                    0 => {
                        self.mode = 1;
                        self.night_pane_on = true;
                    }
                    1 => {
                        self.mode = 2;
                        self.night_pane_on = false;
                        self.xray_pane_on = true;
                    }
                    2 => {
                        self.mode = 0;
                        self.night_pane_on = false;
                        self.xray_pane_on = false;
                    }
                    _ => {}
                }
                out.mode = self.mode;
                out.night_pane_on = self.night_pane_on;
                out.xray_pane_on = self.xray_pane_on;
            }
        }
        // The fade timer [M :103-136]: +3 per tick toward 1800 while off, −1 while a mode is
        // up; at ≤ 0 with a mode set, clear the mode and both pane enables.
        if out.mode == 0 {
            // `battery = MIN.S(battery + 3·FRAME_RATE_MUL, 1800)` — the cap tests the GROWN value
            // [M disasm 0x19ec68 = COP1.S MIN.S; docs/systems/goggles.md]. The old form compared
            // the pre-add value and let 1799 grow to 1802.
            self.fade = (self.fade + 3.0).min(1800.0);
        } else {
            self.fade -= 1.0;
            if self.fade <= 0.0 && self.mode != 0 && self.mode <= 2 {
                self.fade = 0.0;
                self.mode = 0;
                self.night_pane_on = false;
                self.xray_pane_on = false;
                out.mode = 0;
                out.night_pane_on = false;
                out.xray_pane_on = false;
            }
        }
        Some(out)
    }

    /// The item-class clamp the goggles path applies [M 0x19e9d8:60-66]: classes 'G' (0x47)
    /// and 0x5c..0x5e clamp to 1.
    pub fn clamp_item_class(c: u8) -> u8 {
        if c == b'G' || (b'\\' < c && c < b'_') {
            1
        } else {
            c
        }
    }
}

/// `HUD_CreateShrink` 0x1a0bb8's tail (shared by Blood/Sight/NightSight/RCCar/Ronin/Laser and
/// the OICW pane 0x19cc90:23-52): on a sub-fullscreen viewport the pane's sprites re-centre —
/// screen h (`viewer+0x10c`) < 512 halves each sprite's w (`+0x2c`) and shifts x (`+0x24`) by
/// `−def_x/2`; screen h (`viewer+0x110`) < 448 halves h (`+0x2e`) and shifts y (`+0x26`) by
/// `−def_y/2` [M 0x1a0bb8:17-40]. Returns `(w2, x2, h2, y2)` per sprite over the closure's
/// base geometry.
pub fn shrink_to_viewport(
    screen_w: f32,
    screen_h: f32,
    def_x: i16,
    def_y: i16,
    base: impl Fn(usize) -> (i16, i16, i16, i16),
    count: usize,
) -> Vec<(i16, i16, i16, i16)> {
    let hw = screen_w < 512.0;
    let hh = screen_h < 448.0;
    (0..count)
        .map(|k| {
            let (w, x, h, y) = base(k);
            let (w2, x2) = if hw { (w >> 1, x - (def_x >> 1)) } else { (w, x) };
            let (h2, y2) = if hh { (h >> 1, y - (def_y >> 1)) } else { (h, y) };
            (w2, x2, h2, y2)
        })
        .collect()
}

/// `HUD_UpdateCarPane` 0x1a15a0: the RC-car pane's 5 sprites' layer bytes (`+0x38`) swap by
/// the vehicle's cel group — the RC-car cel (`hashtable_hashcode_to_celglist(0x2000194)`,
/// `obj+0xd8`) puts the layers at `[0xff, 0xff, 0x27, 0x27, 0x27]` with sprite 0's colour
/// word `0x0020ff`; anything else `[0x27, 0x27, 0xff, 0xff, 0xff]` with `0x2000ff` [M
/// 0x1a15a0:12-30]. Returns (sprite 0's colour word, the 5 layer bytes).
pub fn car_pane_layers(is_car_cel: bool) -> (u32, [u8; 5]) {
    if is_car_cel {
        (0x0020ff, [0xff, 0xff, 0x27, 0x27, 0x27])
    } else {
        (0x2000ff, [0x27, 0x27, 0xff, 0xff, 0xff])
    }
}

/// The goggles' two pane updates, per frame, in `HUD_Update` 0x19a088's pane-walk order (pane 7
/// `HUD_UpdateNightSightPane` 0x19d218 before pane 15 `HUD_UpdateXRayPane` 0x1a1410), over the
/// SAME shared `Rand_Rand` stream the game draws from [M 0x19d218:18,24; 0x1a1410:15 — the
/// `Rand_Rand__FUi` calls inside the sim tick]. Holds the runtime sprite fields the two updates
/// write; the draw side (nf-view hud.rs) reads them over the `NightSightSprInfo` /
/// `XraySprInfo` templates.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GogglePanes {
    /// Night pane sprite 0's colour word (0xRRGGBBAA): `(Rand(0x18) + 7) | 0xff0000` once the
    /// update runs; the template's 0x007f0020 before [D NightSightSprInfo[0]; M 0x19d218:39-40].
    pub night_colour: u32,
    /// Night pane sprite 0's v (rt+0x32): `Rand(h − 1)` on every other update [M :43-47].
    pub night_v: u16,
    /// `Scrl_247`, the night pane's parity byte [M :41-42].
    pub scrl: u8,
    /// X-ray pane sprite 0's colour word: `(Rand(0x18) + 7) | 0xffffff00` [M 0x1a1410:17-18];
    /// the template's 0x007f0020 before [D XraySprInfo[0]].
    pub xray_colour: u32,
    /// X-ray pane sprite 0's v: `+1 & 0x1f` per update [M :19-22].
    pub xray_v: u16,
    /// The battery bar's width (night sprite 4 / x-ray sprite 3, rt+0x2c and the uv width
    /// rt+0x34): `max(1, (int)(battery · (1/1800) · 200 + 28))` [M 0x19d218:52-62 /
    /// 0x1a1410:23-33]. `None` until an update ran (the template's 256).
    pub night_bar: Option<u16>,
    pub xray_bar: Option<u16>,
}

impl Default for GogglePanes {
    fn default() -> Self {
        Self { night_colour: 0x007f_0020, night_v: 0, scrl: 0, xray_colour: 0x007f_0020, xray_v: 0, night_bar: None, xray_bar: None }
    }
}

/// What the night pane update asks of the caller this frame.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct NightPaneOut {
    /// SP only: `Light_Create(radius 32.0, intensity 2.5, glb_viewer+0xc0, 0xff,0xff,0xff,
    /// channel 0, life 0, kind 1, flicker 0)` — a one-tick white light at the viewer, every
    /// frame night sight is up [M 0x19d218:48-51; the arg order of the muzzle-flash sites].
    /// The caller creates it (its own `Rand_Rand(0)` draw follows the pane's draws).
    pub light: bool,
}

impl GogglePanes {
    /// `HUD_UpdateNightSightPane` 0x19d218, one frame. `enable` is the pane's +0x1c byte
    /// (`MonitorNightSight::night_pane_on`); a mode other than 1 clears it and draws nothing
    /// [M :33-36, :66-68]. `sprite_h` is sprite 0's height word the jitter range reads (rt+0x2a;
    /// the port passes the template's v extent, the redeemer pane's idiom — [S] the exact
    /// runtime word). `mp` skips the light [M :48].
    pub fn night_update(&mut self, enable: &mut bool, mode: u16, battery: f32, sprite_h: u16, mp: bool, rng: &mut crate::drone::Rng) -> NightPaneOut {
        let mut out = NightPaneOut::default();
        if !*enable { return out; }
        if mode != 1 { *enable = false; return out; }
        self.night_colour = (rng.rand(0x18) + 7) | 0x00ff_0000;
        let odd = self.scrl & 1 != 0;
        self.scrl = self.scrl.wrapping_add(1);
        if odd {
            self.night_v = rng.rand(u32::from(sprite_h).wrapping_sub(1)) as u16;
        }
        out.light = !mp;
        self.night_bar = Some(NightSightFx { mode, fade: battery, scrl: 0 }.step());
        out
    }

    /// `HUD_UpdateXRayPane` 0x1a1410, one frame (`enable` = `MonitorNightSight::xray_pane_on`;
    /// mode != 2 clears it [M :13-16, :36-38]).
    pub fn xray_update(&mut self, enable: &mut bool, mode: u16, battery: f32, rng: &mut crate::drone::Rng) {
        if !*enable { return; }
        if mode != 2 { *enable = false; return; }
        self.xray_colour = (rng.rand(0x18) + 7) | 0xffff_ff00;
        self.xray_v = NightSightFx::xray_next_v(self.xray_v);
        self.xray_bar = Some(NightSightFx { mode, fade: battery, scrl: 0 }.step());
    }
}

/// `HUD_UpdateCrossHair` 0x19e6d0:80-85: the crosshair's colour word — 0x208020ff (green)
/// while night sight is up, else 0xff0000ff (red) [M].
pub fn crosshair_colour(mode: u16) -> u32 {
    if mode == 1 { 0x2080_20ff } else { 0xff00_00ff }
}

/// The HUD pane's private per-frame hash (the redeemer pane's idiom, nf-view hud.rs):
/// deterministic, never the sim RNG.
fn hash_frame(frame: u32, salt: u32) -> u32 {
    let mut x = frame.wrapping_mul(0x9e37_79b9) ^ salt.wrapping_mul(0x85eb_ca6b);
    x ^= x >> 15;
    x = x.wrapping_mul(0x2c1b_3c6d);
    x ^ (x >> 12)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// [M 0x19cc90] SP plants mode 0 / timer 0x96; MP plants mode 2 (the steady, hidden state).
    #[test]
    fn oicw_boot_plant() {
        let sp = OicwBoot::new(false);
        assert_eq!(sp.mode, 0);
        let mp = OicwBoot::new(true);
        assert_eq!(mp.mode, 2);
        // MP: every layer hidden, and tick never animates.
        let mut mp = mp;
        mp.tick(true);
        assert!(mp.lines().iter().all(|l| l.layer == 0xff));
    }

    /// [M 0x19cef8 mode 0] the list reveals one line per 30 ticks, top first, block rising.
    #[test]
    fn oicw_boot_reveals_one_line_per_30_ticks() {
        let mut b = OicwBoot::new(false);
        let mut visible = |ticks: u32| {
            for _ in 0..ticks {
                b.tick(true);
            }
            b.lines().iter().filter(|l| l.layer == 0x1c).count()
        };
        assert_eq!(visible(1), 1); // timer 0x96 → 0x95: 5 - 4 = 1
        assert_eq!(visible(28), 1); // timer 121: 121/30 = 4 → 1
        assert_eq!(visible(2), 2); // timer 119: 119/30 = 3 → 2
        assert_eq!(visible(30), 3); // timer 89
        assert_eq!(visible(30), 4); // timer 59
        assert_eq!(visible(30), 5); // timer 29
    }

    /// [M 0x19cef8 :53-58] line y: 408 - 17*visible + 17*k (the block rises).
    #[test]
    fn oicw_boot_line_positions() {
        let mut b = OicwBoot::new(false);
        b.tick(true);
        let ls = b.lines();
        assert_eq!(ls[0].y, 408 - 17); // visible 1: y = 391
        assert_eq!(ls[0].x, 0x32);
        assert_eq!(ls[0].text, "COMMAND.COM\n");
        for _ in 0..30 {
            b.tick(true);
        }
        let ls = b.lines();
        assert_eq!(ls[0].y, 408 - 34); // visible 2: y = 374
        assert_eq!(ls[1].y, 408 - 17);
        assert_eq!(ls[1].text, "LOAD BIOS\n");
    }

    /// [M 0x19cef8 :60-71] while exactly 2 lines show, LOAD BIOS blinks on timer % 8 < 4.
    #[test]
    fn oicw_boot_load_bios_blinks() {
        let mut b = OicwBoot::new(false);
        for _ in 0..150 - 119 {
            b.tick(true);
        }
        assert_eq!(b.lines().iter().filter(|l| l.layer == 0x1c).count(), 2);
        let blink_on = b.lines()[1].layer; // timer 119: 119 % 8 = 7 → visible
        assert_eq!(blink_on, 0x1c);
        for _ in 0..4 {
            b.tick(true);
        } // timer 115: 115 % 8 = 3 → hidden
        assert_eq!(b.lines()[1].layer, 0xff);
    }

    /// [M 0x19cef8 :80-84 + :24-64] boot (150 ticks) → mode 1 flash (30 ticks) → mode 2 park;
    /// disabled pane resets to mode 0 / timer 0x96 unless parked.
    #[test]
    fn oicw_boot_flash_then_park_then_reset() {
        let mut b = OicwBoot::new(false);
        for _ in 0..150 {
            b.tick(true);
        }
        assert_eq!(b.mode, 1);
        for _ in 0..30 {
            b.tick(true);
        }
        assert_eq!(b.mode, 2);
        let mut b2 = OicwBoot::new(false);
        for _ in 0..150 + 1 {
            b2.tick(true);
        } // timer 0x1d: 29 % 4 = 1 < 2 → hidden
        assert_eq!(b2.lines().iter().filter(|l| l.layer == 0xff).count(), 5);
        b2.tick(true); // timer 28: 28 % 4 = 0 < 2 → still hidden
        assert_eq!(b2.lines().iter().filter(|l| l.layer == 0x1c).count(), 0);
        // Parked pane does not reset when disabled...
        b.tick(false);
        assert_eq!(b.mode, 2);
        // ...but an SP pane caught mid-boot does.
        b2.tick(false);
        assert_eq!(b2.mode, 0);
        b2.tick(true); // timer 0x95: visible 1
        assert_eq!(b2.lines()[0].y, 391);
    }

    /// [M 0x19d218 :63-79] the brightness step: max(1, 28 + fade/1800*200).
    #[test]
    fn nightsight_step_matches_the_decompile_arithmetic() {
        let fx = NightSightFx { mode: 1, fade: 0.0, scrl: 0 };
        assert_eq!(fx.step(), 28);
        let fx = NightSightFx { mode: 1, fade: 1800.0, scrl: 0 };
        assert_eq!(fx.step(), 228); // 28 + 200
        let fx = NightSightFx { mode: 1, fade: 900.0, scrl: 0 };
        assert_eq!(fx.step(), 128); // 28 + 100
    }

    /// [M 0x19d218 :39-47 + 0x1a1410 :17-30] the tint bases and the x-ray v wrap.
    #[test]
    fn nightsight_tints_and_xray_v() {
        let fx = NightSightFx { mode: 1, fade: 10.0, scrl: 0 };
        for f in 0..32u32 {
            let t = fx.night_tint(f);
            assert_eq!(t >> 16, 0xff); // green base 0x00ff00xx
            assert_eq!((t >> 8) & 0xff, 0);
            assert!((t & 0xff) >= 7 && (t & 0xff) < 7 + 0x18);
            let t = fx.xray_tint(f);
            assert_eq!(t >> 8, 0xffffff); // blue base 0xffffffxx
            assert!((t & 0xff) >= 7 && (t & 0xff) < 7 + 0x18);
        }
        assert_eq!(NightSightFx::xray_next_v(0x1e), 0x1f);
        assert_eq!(NightSightFx::xray_next_v(0x1f), 0); // wraps at 0x1f
        // Scanline jitter: odd scrl ticks roll, even ones do not.
        let mut fx = fx;
        let a = fx.night_jitter_v(480, 7);
        let b = fx.night_jitter_v(480, 7);
        assert!(a.is_some() != b.is_some());
        if let Some(v) = a { assert!(v < 479); }
    }

    /// [M 0x1a1538 :12-21] the sec-cam wash alternates on the 32-field counter.
    #[test]
    fn sec_cam_tint_alternates() {
        assert_eq!(NightSightFx::sec_cam_tint(0), 0x7f7f7f78);
        assert_eq!(NightSightFx::sec_cam_tint(15), 0x7f7f7f78);
        assert_eq!(NightSightFx::sec_cam_tint(16), 0x7f7f7f00);
        assert_eq!(NightSightFx::sec_cam_tint(31), 0x7f7f7f00);
    }

    fn inputs() -> MonitorInputs {
        MonitorInputs {
            hud_active: true,
            viewer_state: 1,
            mp: false,
            weapon_class: 0x30,
            gadget_sub: 0,
            goggles_equipped: false,
            zoom_pane_on: false,
            toggle_action: false,
            weapon_bit0: false,
        }
    }

    /// [M 0x19e9d8 :18-47] the guard ladder.
    #[test]
    fn monitor_guards_bail() {
        let mut m = MonitorNightSight::default();
        let mut i = inputs();
        i.hud_active = false;
        assert_eq!(m.tick(&i), None);
        let mut i = inputs();
        i.viewer_state = 2;
        assert_eq!(m.tick(&i), None);
        let mut i = inputs();
        i.mp = true;
        assert_eq!(m.tick(&i), None);
        let mut i = inputs();
        i.weapon_class = 0x12;
        assert_eq!(m.tick(&i), None);
        let mut i = inputs();
        i.gadget_sub = 1; // 0 and 3..=5 only
        assert_eq!(m.tick(&i), None);
        let mut i = inputs();
        i.gadget_sub = 4;
        assert!(m.tick(&i).is_some());
    }

    /// [M 0x19e9d8 :72-101] the cycle: action cycles 0 → 1 → 2 → 0 with the pane enables.
    #[test]
    fn monitor_cycles_night_xray_off() {
        let mut m = MonitorNightSight::default();
        let mut i = inputs();
        i.toggle_action = true;
        // Latch clear: the first action overrides the item class to 0x5d, no cycle.
        let out = m.tick(&i).unwrap();
        assert_eq!(out.item_class_override, Some(0x5d));
        assert_eq!(m.mode, 0);
        // Latch set (goggles were on): the action cycles.
        m.was_goggles = true;
        let out = m.tick(&i).unwrap();
        assert_eq!(out.mode, 1);
        assert!(out.night_pane_on && !out.xray_pane_on);
        let out = m.tick(&i).unwrap();
        assert_eq!(out.mode, 2);
        assert!(!out.night_pane_on && out.xray_pane_on);
        let out = m.tick(&i).unwrap();
        assert_eq!(out.mode, 0);
        assert!(!out.night_pane_on && !out.xray_pane_on);
        // weapon_bit0 set: the action does not cycle [M :72-76].
        i.weapon_bit0 = true;
        m.mode = 0;
        let out = m.tick(&i).unwrap();
        assert_eq!(out.mode, 0);
    }

    /// [M 0x19e9d8 :45-66 + :103-136] the goggles force-to-night and the fade integration.
    #[test]
    fn monitor_goggles_force_and_fade() {
        let mut m = MonitorNightSight::default();
        let mut i = inputs();
        i.goggles_equipped = true;
        let out = m.tick(&i).unwrap();
        assert_eq!(out.mode, 1);
        assert!(m.was_goggles);
        // Zoom pane up: the goggles path returns without changes [M :51-53].
        i.zoom_pane_on = true;
        m.mode = 0;
        let out = m.tick(&i).unwrap();
        assert_eq!(out.mode, 0);
        i.zoom_pane_on = false;
        // Fade: grows +3 toward 1800 while off [M :103-106]...
        let mut m = MonitorNightSight { mode: 0, fade: 0.0, was_goggles: false, night_pane_on: false, xray_pane_on: false };
        for _ in 0..10 {
            m.tick(&inputs());
        }
        assert_eq!(m.fade, 30.0);
        // ...drains −1 while a mode is up, and resets the mode at ≤ 0 [M :107-136].
        m.mode = 2;
        m.fade = 3.0;
        let out = m.tick(&inputs()).unwrap();
        assert_eq!(out.mode, 2); // 3 → 2, still up
        let out = m.tick(&inputs()).unwrap();
        assert_eq!(out.mode, 2); // 2 → 1, still up
        let out = m.tick(&inputs()).unwrap(); // 1 → 0: ≤ 0 resets
        assert_eq!(out.mode, 0);
        assert_eq!(m.fade, 0.0);
        assert!(!m.night_pane_on && !m.xray_pane_on);
    }

    /// [M 0x1a0bb8 :17-40] the sub-viewport re-centre: w/x on h < 512, h/y on h < 448.
    #[test]
    fn shrink_recenters_sub_fullscreen() {
        let base = |k: usize| match k {
            0 => (0x100i16, 0x80i16, 0x40i16, 0x20i16),
            _ => (0x40, 0x10, 0x20, 0x30),
        };
        // Fullscreen: untouched.
        let out = shrink_to_viewport(640.0, 448.0, 0x40, 0x20, base, 2);
        assert_eq!(out[0], (0x100, 0x80, 0x40, 0x20));
        // Narrow (640 < 512? no — width guard is screen_w < 512): 512-wide halves.
        let out = shrink_to_viewport(511.0, 448.0, 0x40, 0x20, base, 2);
        assert_eq!(out[0], (0x80, 0x80 - 0x20, 0x40, 0x20));
        // Short: h/y halve/shift.
        let out = shrink_to_viewport(640.0, 447.0, 0x40, 0x20, base, 2);
        assert_eq!(out[0], (0x100, 0x80, 0x20, 0x20 - 0x10));
        // Both.
        let out = shrink_to_viewport(511.0, 447.0, 0x40, 0x20, base, 2);
        assert_eq!(out[0], (0x80, 0x80 - 0x20, 0x20, 0x20 - 0x10));
    }

    /// [M 0x1a15a0 :12-30] the RC-car layer swap (sister: the two sets differ everywhere).
    #[test]
    fn car_pane_layer_swap() {
        let (c, layers) = car_pane_layers(true);
        assert_eq!(c, 0x0020ff);
        assert_eq!(layers, [0xff, 0xff, 0x27, 0x27, 0x27]);
        let (c, layers) = car_pane_layers(false);
        assert_eq!(c, 0x2000ff);
        assert_eq!(layers, [0x27, 0x27, 0xff, 0xff, 0xff]);
    }

    /// [M 0x19e9d8 :60-66] the item-class clamp.
    #[test]
    fn monitor_clamps_item_class() {
        assert_eq!(MonitorNightSight::clamp_item_class(b'G'), 1);
        assert_eq!(MonitorNightSight::clamp_item_class(0x5d), 1); // '\\' < 0x5d < '_'
        assert_eq!(MonitorNightSight::clamp_item_class(0x30), 0x30);
        assert_eq!(MonitorNightSight::clamp_item_class(b'A'), b'A');
    }
}

#[cfg(test)]
mod goggle_pane_tests {
    use super::*;
    use crate::drone::Rng;

    /// [M 0x19d218:33-62] mode 1: the green tint word, the parity jitter, the SP light, the bar —
    /// and the draws come off the shared stream (1 per frame + 1 on odd parity frames).
    #[test]
    fn night_update_writes_the_game_fields_from_the_shared_stream() {
        let mut p = GogglePanes::default();
        let mut rng = Rng::new(7);
        let mut on = true;
        let out = p.night_update(&mut on, 1, 1800.0, 255, false, &mut rng);
        assert!(on && out.light);
        assert_eq!(p.night_colour >> 8, 0x00ff00, "0x00ff00AA: green, alpha 7..30");
        assert!((7..7 + 0x18).contains(&(p.night_colour & 0xff)));
        assert_eq!(p.night_bar, Some(228));
        assert_eq!(rng.draws, 1, "parity 0: tint only");
        p.night_update(&mut on, 1, 900.0, 255, false, &mut rng);
        assert_eq!(rng.draws, 3, "parity 1: tint + the v jitter");
        assert!(p.night_v < 254);
        assert_eq!(p.night_bar, Some(128));
        // MP: no light, the bar still written [M :48].
        let out = p.night_update(&mut on, 1, 0.0, 255, true, &mut rng);
        assert!(!out.light);
        assert_eq!(p.night_bar, Some(28));
        // Sister (can fail): any other mode clears the enable and draws nothing.
        let before = rng.draws;
        let out = p.night_update(&mut on, 2, 1800.0, 255, false, &mut rng);
        assert!(!on && !out.light);
        assert_eq!(rng.draws, before);
        // A disabled pane does nothing at all, mode 1 included.
        let out = p.night_update(&mut on, 1, 1800.0, 255, false, &mut rng);
        assert!(!out.light);
        assert_eq!(rng.draws, before);
    }

    /// [M 0x1a1410:13-38] mode 2: the white tint word, the v ramp wrapping at 0x1f, the bar.
    #[test]
    fn xray_update_ramps_v_and_tints_white() {
        let mut p = GogglePanes::default();
        let mut rng = Rng::new(3);
        let mut on = true;
        for k in 1..=0x21u16 {
            p.xray_update(&mut on, 2, 450.0, &mut rng);
            assert_eq!(p.xray_v, k & 0x1f);
        }
        assert_eq!(p.xray_colour >> 8, 0xffffff);
        assert_eq!(p.xray_bar, Some(78));
        assert_eq!(rng.draws, 0x21);
        // Sister: mode 1 clears the x-ray enable.
        p.xray_update(&mut on, 1, 450.0, &mut rng);
        assert!(!on);
        assert_eq!(rng.draws, 0x21);
    }

    /// [M 0x19e6d0:80-85] the crosshair turns green only in night sight.
    #[test]
    fn crosshair_colour_follows_the_mode() {
        assert_eq!(crosshair_colour(1), 0x2080_20ff);
        assert_eq!(crosshair_colour(0), 0xff00_00ff);
        assert_ne!(crosshair_colour(2), crosshair_colour(1), "sister: x-ray keeps the red cross");
    }

    /// [M 0x19e9d8:50-62] the arm path writes the night pane's enable byte into the state.
    #[test]
    fn the_arm_enables_the_night_pane() {
        let mut m = MonitorNightSight::default();
        let i = MonitorInputs { hud_active: true, viewer_state: 1, goggles_equipped: true, ..Default::default() };
        m.tick(&i);
        assert!(m.night_pane_on && m.mode == 1 && m.was_goggles);
        // Sister (can fail): a view-model anim up blocks the whole arm.
        let mut m = MonitorNightSight::default();
        m.tick(&MonitorInputs { zoom_pane_on: true, ..i });
        assert!(!m.night_pane_on && m.mode == 0 && !m.was_goggles);
    }

    /// [M disasm 0x19ec68 MIN.S] the battery's +3 caps on the GROWN value: 1799 → 1800, not 1802.
    #[test]
    fn battery_cap_tests_the_grown_value() {
        let mut m = MonitorNightSight { fade: 1799.0, ..Default::default() };
        let i = MonitorInputs { hud_active: true, viewer_state: 1, ..Default::default() };
        m.tick(&i);
        assert_eq!(m.fade, 1800.0);
        m.fade = 1795.0;
        m.tick(&i);
        assert_eq!(m.fade, 1798.0, "sister: below the cap the +3 lands exactly");
    }
}
