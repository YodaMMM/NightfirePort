//! Bond on foot: `Player_Update` (0x193600) with `Player_Move` (0x191dd0), `Player_HandleJump` (0x1916b0),
//! `Player_Collision` (0x194df0), `Player_CollisionHandler` (0x1949a8) and the eye of `Player_PositionCamera`.
//! Spec: docs/systems/player.md.
//!
//! Level carry-over and spawn (fix-level2): `Player_RamSave` (0x195670) / `Player_RamLoad` (0x1957a8),
//! the `player_start` table (`Player_AddNewStartPos` 0x195c00, `Player_Start` 0x190fd0) and the per-level
//! loadout switch of `Player_InitWeapon` (0x1a7ef8) with `Player_Weapon`'s party weapon lock (0x1a24c8).

use crate::ladder::Ladder;
use crate::triggers::Channels;
use crate::wire::{Wire, WireAnims};
use crate::world::{CollisionWorld, V3};

/// `AnimObj+0xCC` [L]: the root height above the feet (07000005, skin 0x05000007), the STANDING value.
pub const ROOT_HEIGHT: f32 = 1.0347183;
/// The CROUCHED stance's root height [L, derived from the PCSX2 trace 05-truck]: riding the truck bed
/// crouched, Bond parks with obj+0x30 = 37.887 over the bed floor 37.280 — pos−floor 0.607, i.e. cc
/// 0.556 after the standing rest offset (0.051: standing live 1.086 with cc 1.0347) — ≈ the capsule
/// radius 0.55 [M 0x194df0: 0x3f0ccccd]. The game's cc is **AnimObject+0xCC** (obj+0xDC+0xCC): the
/// anim/stance writes it and the feet-point math reads it there [M Player_Collision 0x194df0:86,
/// 222-236]. The every-tick copy at Player_Update 0x193600:449-453 / Player_CollisionHandler
/// 0x1949a8:179-183 is `obj+0xC0..0xCC = obj+0x30..0x3C` — the matrix ROW 3 mirroring the position
/// (gated on flags2 bit 4), NOT a root-height write (fields-player correction: the earlier
/// `obj+0xCC = obj+0x3C` reading mistook it). The port models the two stances as constants.
pub const CROUCH_ROOT_HEIGHT: f32 = 0.55;
/// Capsule radius (`HITTEST+0x8C`).
pub const RADIUS: f32 = 0.55;
/// The CLIMB/CREEP hittest arm's radius: `Player_Collision` 0x194df0's substate switch maps the
/// substates 1 (climb) and 7 (creep) to the narrow arm — the radius slot `+0x7CC = 0x3e8ccccd` =
/// 0.275, segment A = pos + up·0.55, B = pos + up·(+0x7CC − cc) [M the jump table @0x2fc0d0 (case
/// 0 = substate 1, case 6 = substate 7) + 0x1950a0]. The WALK substate 0 is NOT in that arm: the
/// switch indexes substate−1, walk 0 → −1 → the unsigned bounds check fails → the DEFAULT arm (r
/// 0.55) [M 0x1953f4]. The port's walk GRAB rides the default arm (`Ladder::capsule_hits_default`);
/// this const stays for the climb/creep arm's shape (`Ladder::capsule_hits`) — the port's climb
/// tick runs no grab scan (the game's `Player_CollLadder` 0x196118 excludes substates 1/2), so the
/// narrow arm's remaining role is documentation + the unit pins of the switch's own geometry.
pub const WALK_PICK_RADIUS: f32 = 0.275;
/// `BL+0x908` on land.
pub const EYE: f32 = 0.7;
const GRAVITY: f32 = -9.8;

/// `obj+0xF6` substates, from the `Player_Update` 0x193600 dispatch [M switch @0x193958; the jump table
/// reads obj+0xF6 and its cases call Player_SSWalk 0x192310 (0), Player_Climb 0x196310 (1),
/// Player_SSGrapple 0x192860 (2), the inline swim (3), Player_SSCrouch 0x1924c0 (4), Player_SSScanMode
/// 0x193098 (5), Player_Wire 0x197658 (6), Player_Creep 0x1969d0 (7), Player_ZeroG 0x192ab0 (8/9), the
/// scripted-cam countdown (10), Player_Zipline 0x197f50 (0xF)]. This session ports 1, 3, 6 and 0xF.
pub const SS_WALK: u8 = 0;
pub const SS_CLIMB: u8 = 1;
pub const SS_GRAPPLE: u8 = 2;
pub const SS_SWIM: u8 = 3;
/// `Player_Wire` 0x197658 — the hand-over-hand wire crossing (wire type 0) [M dispatch case 6].
pub const SS_WIRE: u8 = 6;

/// The crawl substate (`Player_Creep` 0x1969d0; entered by `Player_CollCreepWall` 0x196758 through
/// `Player_Activate`'s class-'+' use scan).
pub const SS_CREEP: u8 = 7;
/// `Player_Zipline` 0x197f50 — the zipline ride (wire type 2) [M dispatch case 0xF].
pub const SS_ZIPLINE: u8 = 0xf;

/// The player's skin hash, per level — `Player_Init` 0x1910f8:94-108's switch on the level id
/// (`GameState+0x12`), the value handed to `AnimObjectNew` [M]: 0x0500009 on 07000001-07000004,
/// 0x050000d on 07000007/07000008, 0x050007c on 0700001b, 0x0500007 everywhere else [D each in its
/// levels' own archives].
pub fn player_skin_hash(level: u32) -> u32 {
    match level {
        0x0700_0001..=0x0700_0004 => 0x0500_0009,
        0x0700_0007 | 0x0700_0008 => 0x0500_000d,
        0x0700_001b => 0x0500_007c,
        _ => 0x0500_0007,
    }
}

/// The zero-g substates (dispatch cases 8/9 share the `Player_ZeroG` 0x192ab0 body [M 0x193600:
/// 238-241]). 9 is the one the game ever sets: the 0700001b spacewalk's script handler answers its
/// 0x4b event with `ChangeSubState(glb_players, 9 | 0)` [M C_CHCHZEROG 0x216388 — 9 when its msg
/// 0x40 query returns nonzero, else 0]; no caller or store writes 8 anywhere in ACTION.ELF. Only
/// substate 9 runs the movement anim set [M 0x192ab0:201 `if (obj+0xF6 == 9)`], so 8 is the anim
/// quiet twin the dispatch still honours.
pub const SS_ZEROG_8: u8 = 8;
pub const SS_ZEROG_9: u8 = 9;

/// `RotMatrix` 0x1e2368 for the zero-g rate vector (x pitch, y yaw, z roll — ZeroG always passes
/// roll 0 [M 0x192ab0:91 `BL+0x38 = 0`]): the row-major rows of Rx(x)·Ry(y)·Rz(z) in the game's
/// row-vector basis (rows = right/up/forward, `Bond::look`'s convention). The sine/cosines go
/// through the port's PS2Sinf/PS2Cosf — `RotMatrix` calls PS2Sinf3/PS2Cosf3 [M 0x1e2368:28-40].
fn rot_matrix(pitch_x: f32, yaw_y: f32, roll_z: f32) -> [[f32; 3]; 3] {
    let (sx, cx) = (crate::trig::sinf(pitch_x), crate::trig::cosf(pitch_x));
    let (sy, cy) = (crate::trig::sinf(yaw_y), crate::trig::cosf(yaw_y));
    let (sz, cz) = (crate::trig::sinf(roll_z), crate::trig::cosf(roll_z));
    [
        [cy * cz, sx * sy * cz + cx * sz, sx * sz - cx * sy * cz],
        [-cy * sz, cx * cz - sx * sy * sz, sx * cz + cx * sy * sz],
        [sy, -sx * cy, cx * cy],
    ]
}

/// `Concat(a, ab)` 0x1e4a90 ⇒ ab := a·ab [M 0x1e4a90:13-49] — Player_ZeroG calls it as
/// `Concat(obj+0x90, R)`, so each tick's look rotation lands in the BODY frame.
fn mat3_mul(a: &[[f32; 3]; 3], b: &[[f32; 3]; 3]) -> [[f32; 3]; 3] {
    let mut out = [[0.0f32; 3]; 3];
    for i in 0..3 {
        for j in 0..3 {
            out[i][j] = a[i][0] * b[0][j] + a[i][1] * b[1][j] + a[i][2] * b[2][j];
        }
    }
    out
}

fn cross3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]]
}

fn norm3(v: [f32; 3]) -> [f32; 3] {
    let l = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if l > 0.0 { [v[0] / l, v[1] / l, v[2] / l] } else { v }
}

/// `Mat_Align2Up(m, up, dir)` 0x1df4b0: rebuild the rotation with the up row = `up` and the
/// forward row over `dir` — row0 = norm(up × dir), row2 = norm(row0 × up) [M 0x1df4b0; the ZeroG
/// caller passes CONSTANT_UP1 = (0,1,0), .data 0x2d6210 written at boot 0x1c7e78].
fn mat_align_up(dir: [f32; 3]) -> [[f32; 3]; 3] {
    let up = [0.0f32, 1.0, 0.0];
    let r0 = norm3(cross3(up, dir));
    [r0, up, norm3(cross3(r0, up))]
}

/// `Quat_MatrixToQuaternion` 0x1e0600 — q = [w, x, y, z]; the trace branch's w = −√/2 sign and the
/// per-diagonal branches verbatim (the decompile's param_2 floats i·4+j = m[i][j]).
fn quat_from_matrix(m: &[[f32; 3]; 3]) -> [f32; 4] {
    let (m00, m11, m22) = (m[0][0], m[1][1], m[2][2]);
    let tr = m00 + m11 + m22;
    if tr >= 0.0 {
        let s = (tr + 1.0).sqrt();
        let k = 0.5 / s;
        [-0.5 * s, (m[2][1] - m[1][2]) * k, (m[0][2] - m[2][0]) * k, (m[1][0] - m[0][1]) * k]
    } else if m00 >= m11 && m00 >= m22 {
        let s = (m00 - (m11 + m22) + 1.0).sqrt();
        let k = 0.5 / s;
        [(m[1][2] - m[2][1]) * k, 0.5 * s, (m[0][1] + m[1][0]) * k, (m[2][0] + m[0][2]) * k]
    } else if m11 >= m22 {
        let s = (m11 - (m22 + m00) + 1.0).sqrt();
        let k = 0.5 / s;
        [(m[2][0] - m[0][2]) * k, (m[0][1] + m[1][0]) * k, 0.5 * s, (m[1][2] + m[2][1]) * k]
    } else {
        let s = (m22 - (m00 + m11) + 1.0).sqrt();
        let k = 0.5 / s;
        [(m[0][1] - m[1][0]) * k, (m[2][0] + m[0][2]) * k, (m[1][2] + m[2][1]) * k, 0.5 * s]
    }
}

/// `Quat_QuaternionToMatrix` 0x1e02a0 — the rotation floats in the game's row-major slots.
pub(crate) fn quat_to_matrix(q: &[f32; 4]) -> [[f32; 3]; 3] {
    let (w, x, y, z) = (q[0], q[1], q[2], q[3]);
    [
        [2.0 * (x * x + w * w) - 1.0, 2.0 * (x * y + z * w), 2.0 * (x * z - y * w)],
        [2.0 * (x * y - z * w), 2.0 * (y * y + w * w) - 1.0, 2.0 * (y * z + x * w)],
        [2.0 * (x * z + y * w), 2.0 * (y * z - x * w), 2.0 * (z * z + w * w) - 1.0],
    ]
}

/// `Mat_Normalize` 0x1df5f8 — orthonormalise the rows: fwd = norm(row2); right = norm(up × fwd);
/// up = norm(fwd × right). Each cross result is normalized, so the VU0 pair's doubled cross terms
/// wash out; a zero row stays zero (the game's degenerate arm).
fn mat_normalize(mut m: [[f32; 3]; 3]) -> [[f32; 3]; 3] {
    m[2] = norm3(m[2]);
    m[1] = norm3(m[1]);
    m[0] = norm3(cross3(m[1], m[2]));
    m[1] = norm3(cross3(m[2], m[0]));
    m
}

/// The per-tick ladder/wire/water context the Sim hands `Bond::tick_ctx` (the game reads the ladder and
/// the wire through `BLData+0x884` and the water surface through the player's cel, `obj+0x20 → cel+0x90`).
pub struct LadderCtx<'a> {
    /// The level's ladder objects (class 0x2D), in placement order.
    pub ladders: &'a [Ladder],
    /// The level's wire objects (class 0x2C), in placement order (`Wire::all`).
    pub wires: &'a [Wire],
    /// The wire anims' root tracks (`wire::WireAnims`); `None` = no anim data (the anims hold their
    /// start pose — no root motion; the clamp still holds Bond on the wire).
    pub wire_anims: Option<&'a WireAnims>,
    /// `cel+0x90` for the cel Bond is in (`Player_InWater` 0x195e50 reads `*(obj+0x20)+0x90`;
    /// `build_alloc_cel` 0x1c93f8 defaults it to −500.0 = no water; `parseentity_fixup_entity` 0x1d00c0
    /// sets it to the entity's bbox max y for flag word 0xC023). `None` = no water cel / below −400.
    pub water: Option<f32>,
    /// The descend control this tick — carried but never read: the game's vertical swim input is the
    /// look pitch (the swim branch rides the look-directed forward [M Player_Move 0x191dd0:149-165])
    /// and the case-3 dispatch reads no crouch/jump input [M Player_Update 0x193600 case 3;
    /// Player_HandleJump 0x1916b0 runs only from the walk]. There is no descend control in the game's
    /// swim; the field stays for the ctx shape (scene.rs feeds it) but nothing keys on it.
    pub sink: bool,
    /// The level's creepwall objects (class 0x2B '+', placement 0x3F), in placement order.
    pub creepwalls: &'a [crate::creepwall::CreepWall],
    /// The head-bump sound gate: `Player_HandleJump` 0x1916b0:196-198 picks the metal clang 0x5FB
    /// when `level_tag+0x50` (a byte the port's loader does not read) is 0xD/0xE, else the flesh
    /// thud 0xA3. The Sim sets it per level once the byte is identified (live check in
    /// docs/systems/sound-coverage.md); default false = the 0xA3 arm.
    pub level_bump_5fb: bool,
    /// The cels Bond's pick capsule straddles this tick — `Collide_StraddleCels` 0x1eac98's BFS from
    /// the HITTEST home cel through the capsule-touching portals [M Collide_Pick 0x1eb320:125-134
    /// admits a candidate object only from a straddled cel's chain]. The Sim fills it from
    /// `CollisionWorld::debug_straddle` with the current substate's pick arm (`Bond::pick_arm`);
    /// EMPTY = no scoping (no cel graph — unit tests, MP remotes), every ladder tested.
    pub straddle: &'a [usize],
    /// The live bots' collision capsules as `(top, bottom, radius)` — the game's bot hittest arms
    /// (`NDrone2_Collision` 0x140e90:91-121: top = pos + up·r, bottom = pos + up·(r − h), r 0.4
    /// default / 0.55 kind 0xc, h = the anim height word +0xa0, default 1.0 [M DefaultInit
    /// 0x14b300:48]) for the actor-actor push of `Collide_Jointy` 0x1ec180:229-281. The Sim fills
    /// the NEAR bots only (`DroneFunc_NearPlayerCollision` 0x14b190: the bot's collision capsule
    /// is ON within 2.0 of the player [M NDrone2_DoCollision 0x149ec8]); EMPTY = no bots (unit
    /// tests, MP remotes) — no actor push runs.
    pub bot_capsules: &'a [(V3, V3, f32)],
}

impl Default for LadderCtx<'_> {
    fn default() -> Self { LadderCtx { ladders: &[], wires: &[], wire_anims: None, water: None, sink: false, creepwalls: &[], level_bump_5fb: false, straddle: &[], bot_capsules: &[] } }
}

/// The current substate's pick arm — the HITTEST the game builds for `Player_Collision` 0x194df0's
/// capsule switch (indexes on substate−1 [M disasm 0x195070, jump table @0x2fc0d0]): returns
/// `(segment A offset, segment B offset, radius)` as upward offsets from the object position, the
/// same numbers `Ladder::capsule_hits*` pin [M 0x1950a0 the climb/creep arm, 0x1953f4 the default
/// arm, 0x194df0 case 3 the crouch arm]. One source of truth for the pick shape; the straddle walk
/// feeds the same capsule to `Collide_StraddleCels` [M 0x1eac98 takes the pick's HITTEST].
pub fn pick_arm(substate: u8, crouched: bool, cc: f32) -> (f32, f32, f32) {
    if substate == SS_CLIMB || substate == SS_CREEP {
        (0.55, WALK_PICK_RADIUS - cc, WALK_PICK_RADIUS) // 0x1950a0: r 0.275, B reads the radius slot
    } else if crouched {
        (0.1 - (cc - 0.55), 0.45 - cc, 0.55) // 0x194df0 case 3 = substate 4 (crouch)
    } else {
        (0.275, 0.55 - cc, 0.55) // 0x1953f4: the walk 0, the swim 3, every other grab-eligible substate
    }
}

/// One tick of player input as game actions: `forward` (action 2), `strafe` (action 1; + = left), `turn` (radians
/// to add to the yaw this tick, mouse or ramped pad), `look` (pitch delta in units of π/2), the jump (7) and crouch (8)
/// presses, aim mode (0x13).
///
/// Weapon switching [M Player_WeaponChange 0x1a6f80]: `weapon_cycle` = ±1 for next/prev weapon (mouse wheel / Q),
/// `gadget_cycle` = ±1 for next/prev gadget (G), `switch_to` = `Some(id)` for a direct selection (number keys).
/// The weapon/gadget split follows the game's own mode 0 (weapons 1..0x47) / mode 1 (gadgets 0x4A..0x5E).
#[derive(Clone, Copy, Debug, Default)]
pub struct Input {
    pub forward: f32,
    pub strafe: f32,
    pub turn: f32,
    pub look: f32,
    pub jump: bool,
    pub crouch: bool,
    pub aiming: bool,
    /// ±1: next/prev weapon (`Player_WeaponChange` mode=0, bounds 1..0x47).
    pub weapon_cycle: i8,
    /// ±1: next/prev gadget (`Player_WeaponChange` mode=1, bounds 0x4A..0x5E).
    pub gadget_cycle: i8,
    /// `Some(id)`: switch directly to weapon `id` (a number key).
    pub switch_to: Option<u8>,
    /// Right mouse held: zoom the scope (`Player_Zoom` 0x1a1a70, mouse-first like the game's aim button).
    pub zoom: bool,
    /// The mouse's scope/aim button (the right button, held): drives the aim bit — the raise, then the zoom —
    /// like the pad's aim button, but never the free-aim cursor window (PLAN.md D-002: the mouse aims
    /// directly). The pad keeps `aiming` (aim) and `zoom` (its own zoom axis).
    pub scope: bool,
    /// The fire action held this tick (action 9; the substates read it directly — the grapple's
    /// reel/hang/release [M Player_SSGrapple 0x192860's Input_Action(9, 1)]). The weapon fire path
    /// gets the same bit through its own parameter.
    pub fire: bool,
    /// The fire-mode button's newly-pressed edge (pad ○, action 12 mask 4) [M `Player_Weapon`
    /// 0x1a24c8: `Input_Action(slot, 0xc, 4)`]. Cycles the mode byte (`BL + w*0xC + 0x1BB`) on
    /// records without a twin link, or switches to the twin variant (weapon_data +0x05) when set.
    /// Keyboard: M (documented in the HUD help).
    pub mode_switch: bool,
    /// One mouse-wheel notch while the aim (right) button is held: +1 zooms in a step, −1 out
    /// (microcam-wheel-zoom). The zoom target steps ×2 / ÷2 within the held weapon's own range
    /// [1.0, weapon_data+0x88] [M Player_Weapon 0x1a24c8:156-187's clamp]; when the aim is not
    /// held the wheel keeps switching weapons (the window routes it to `weapon_cycle` instead).
    /// Local view state — NOT part of the net input packet (the remote path never runs the
    /// zoom pipeline; `Sim::step_net_remotes` ticks `Bond::tick_ctx` only), so the wire format
    /// is unchanged and `unpack_input` always yields 0.
    pub zoom_cycle: i8,
    /// The zero-g vertical thruster: the combined (action 7 − action 8) analog [M Player_ZeroG
    /// 0x192ab0:136-138 — `Input_Action(7,1) − Input_Action(8,1)`, a HELD read, not the walk
    /// edges: jump thrusts up, crouch thrusts down]. The walk/swim paths ignore it. Local view
    /// state, not net-encoded (the `zoom_cycle` precedent — substate 8/9 never runs on the
    /// remote tick path today).
    pub lift: f32,
    /// The action-0xe read this tick (the pad's R2 — the game's "Action"): arms the zero-g
    /// auto-stabiliser's 90-tick window [M Player_ZeroG 0x192ab0:163-166; the game reads flag 4
    /// per tick, so a hold keeps re-arming]. The Sim ORs its use-key and reload edges into it
    /// (both are action 0xe on the pad). Local view state, not net-encoded (the `zoom_cycle`
    /// precedent).
    pub stabilise: bool,
    /// options-census: the Advanced options' Crouch mode (control 0x1000_0125 → PlayerSetting
    /// +0x04): false = Toggle (stand on the crouch/jump press edge — the game's default, byte 1
    /// [M Input_Init 0x17edd8]), true = Hold (stand when the crouch action is not held) [M
    /// Player_SSCrouch 0x1924c0:84-105]. fix-909c: inverted from `crouch_toggle` so the derived
    /// `Default` (every headless/test/replay/shot input) is the game's Toggle, not Hold.
    pub crouch_hold: bool,
    /// The goggles key's newly-pressed edge (action 13, mask 4) [M `HUD_MonitorNightSight`
    /// 0x19e9d8:74 `Input_Action(p, 0xd, 4)`]. With the goggles latch (BL+0x961) clear the
    /// press requests the goggles weapon (`+0x63 = 0x5d`); once the latch is set it cycles the
    /// goggle mode 0 → 1 (night) → 2 (x-ray) → 0 — unless the aim bit (+0x60 bit 0) is up.
    /// Keyboard: N. Local view state, not net-encoded (the `stabilise` precedent).
    pub goggles_action: bool,
}

/// level-0a-office-hatch-10 experiment switch: `NF_WALK_GHOST=1` drops the walk capsule's
/// DISPLACEMENT while keeping its contact signals (the dead-tail hypothesis — see the walk
/// integration block). Off by default; measured before anything lands.
fn walk_ghost() -> bool {
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ON.get_or_init(|| std::env::var("NF_WALK_GHOST").is_ok())
}


#[derive(Clone, Debug)]
pub struct Bond {
    /// `obj+0x30`.
    pub pos: V3,
    /// `obj+0x40` — the previous position the movement pass copies after the cel change
    /// (`control_movement_object_handler` 0x1336f8:212), so the next `control_handle_cel_change`
    /// 0x133288 relinks on the last tick's move segment. fix-swim-entry.
    pub prev_pos: V3,
    /// `obj+0x54` (0 faces +Z; a larger yaw turns left).
    pub yaw: f32,
    /// `BL+0x8A8`: units of π/2, ±1, + = up.
    pub pitch: f32,
    /// `BL+0x20`: last tick's local velocity (x left, z forward) per tick.
    pub local: [f32; 2],
    /// `BL+0x50`: world velocity, units per second.
    pub vel: V3,
    /// AnimObj flag 8 and the floor's normal y (`BL+0x110`).
    pub grounded: bool,
    pub floor_ny: f32,
    /// `BL+0x940`: grounded in the last 4 ticks (shift register).
    pub grounded_history: u16,
    /// `BL+0x94C` (0 on foot, 1 rising, 2 bumped) and the landing delay `BL+0x952`.
    pub jump: u8,
    pub land_delay: u8,
    /// Substate 4 and the stand / crouch lockout `BL+0x960`.
    pub crouched: bool,
    pub lockout: u32,
    /// `obj+0xCC` (= `obj+0x3C` [M Player_Update 0x193600:449]): the current stance's root height
    /// above the feet. The capsule ends and the floor ray hang off this, so a crouched Bond is a
    /// shorter capsule resting closer to his feet (the truck ride parks him at floor + 0.607).
    pub cc: f32,
    /// `BL+0x90C` (eye drop when crouched) and `BL+0x908`.
    pub crouch_drop: f32,
    pub eye: f32,
    /// `BL+0x928`: ticks falling.
    pub fall_timer: u32,
    /// AnimObj flag 0x20: something touched above.
    pub head_bump: bool,
    /// Damage from the last landing (`Player_HandlePain` type 6), for the caller to apply.
    pub fall_damage: Option<f32>,
    /// `obj+0xF6`: the substate id ([M Player_Update 0x193600 dispatch]; 0 walk, 1 climb, 3 swim here).
    pub substate: u8,
    /// `BL+0x884`: the ladder being climbed (an index into `LadderCtx::ladders`).
    pub ladder: Option<usize>,
    /// `BL+0x8A0` / `BL+0x8A4`: the climb's top/bottom dismount thresholds
    /// (`centre.y + radius + 0.85` / `centre.y − radius + 1.0` [M Player_CollLadder 0x196118,
    /// recomputed per tick in Player_Climb 0x196310]).
    pub climb_top: f32,
    pub climb_bottom: f32,
    /// `BL+0x951`: the regrab lockout in ticks (`Player_Climb` sets `(u8)FRAME_RATE` = 60 on every
    /// dismount; `Player_Update` 0x193840 decrements it; `Player_CollLadder` refuses grabs while set) [M].
    pub regrab: u8,
    /// `BL+0x94D`: the move-mode byte (1 = climbing) [M CollLadder sets 1 on grab].
    pub mode: u8,
    /// `BL+0x942`: the saved substate (the dispatch's case 10 restores it; the wire/creep/climb drops
    /// write it) [M]. The climb writes its own id, matching the game.
    pub saved_sub: u16,
    /// `BL+0x8C0`: the air meter, 0..100 (`Player_MonitorAir` 0x191a70: −5/s with the head below the
    /// surface, +20/s above, drown damage 0.5 (type 7) every 30 ticks below 1.0) [M].
    pub air: f32,
    /// Drown damage from this tick's `Player_MonitorAir`, for the caller to apply.
    pub drown_damage: Option<f32>,
    /// True while the head is below the water surface (`Player_MonitorAir`'s +0x8C8 bit 0x10 path).
    pub underwater: bool,
    /// `BL+0x8C4`: the underwater fade 0..1 (`Player_MonitorAir`: →1 under, →0 surfaced), for the renderer.
    pub water_fade: f32,
    /// `BL+0x884`: the carried creepwall object while the crawl substate (7) runs (`Player_CollCreepWall`
    /// 0x196758 stores the wall; `Player_Creep` 0x1969d0 reads it back every tick).
    pub creep: Option<usize>,
    /// The runtime cel the queries start from (obj+0x20, the movement pass's relink — fix-safe08b):
    /// the `pCelList` cell index in the world's `ScopeGraph`. `None` = the flat (unscoped) world.
    pub cel: Option<usize>,
    /// The drown-damage tick accumulator (the game uses `GameState % 30` [M 0x191a70:136-140]).
    air_tick: u32,
    /// `BL+0x18` → the climb's one-tick local-forward push (`+0.6` top / `−0.05` action [M Player_Climb]),
    /// consumed by the next tick's movement (the game writes the wish `+0x18` after `Player_Move` and the
    /// next tick's move integrates it).
    push_fwd: f32,
    /// `BLData+0x884` narrowed to the wires: the wire being ridden (an index into `LadderCtx::wires`).
    /// The game stores the object pointer; the port indexes the ctx slice like `ladder` [M CollWire
    /// 0x197628 writes +0x884 = the wire object].
    pub wire: Option<usize>,
    /// `BLData+0x8CC`: the wire auto-drop timer — `+= 100/(FRAME_RATE·30)` per tick (paused in the grab
    /// pose, mode 0xC); past 100 the wire drops him [M Player_Wire 0x197748-0x1977a8].
    pub auto_drop: f32,
    /// `oldhash` — the hashcode of the wire anim playing on Bond's AnimObject (the grab pose 0x060005e5,
    /// the idle pair 0x060005ec/ed, the turn pair 0x060005ee/ea, the move pair 0x060000b4/b5, the
    /// zipline 0x06000095/96/94) [M Player_Wire's oldhash tree]. 0 = none.
    pub wire_anim: u32,
    /// The anim's script time in frames (`AnimScriptAdd` starts at 1; `AnimScriptIsStopped` = past the
    /// script's length for these non-looping scripts [D all seven scripts carry looping = false]).
    pub wire_anim_time: f32,
    /// The playing wire anim's rate (`AnimScript+0x98`): 1.0 for the grab pose, ±1.2 for everything
    /// `Player_Wire` itself adds — the tail reads the running rate and re-arms it: negative stays
    /// −1.2, anything else (a fresh script's 0 included) becomes +1.2 [M Player_Wire 0x197658:293-297
    /// `if (rate < 0.0) rate = −1.2 else rate = +1.2`]. A decompile-wide scan for the −1.2 constant
    /// (0xbf99999a) finds no other writer, so from a fresh script the negative arm is unreachable and
    /// +1.2 is the behaviour the port implements.
    pub wire_anim_rate: f32,
    /// The camera mode the grab/drop paths set (`Player_SetCamMode`: 0xB on the wire, 0 on foot) [M
    /// CollWire 0x1975f8, Player_Wire 0x1978a8]. The renderer can adopt it; the headless sim only
    /// records it. This field is the port of the game's cam-mode dispatch: `Camera_Update`
    /// 0x1bb6d8:21 calls the viewer's updator (`Player_PositionCamera` 0x18f5d8) which switches on
    /// BLData+0x950 — this byte — per frame [M].
    pub cam_mode: u16,
    /// `BLData+0x18`: the local velocity's FORWARD lane — the +0x10 quad is (x strafe, y vertical,
    /// z forward) and `Player_Move` writes the z lane fresh every walk tick from the +0x28 smoother
    /// [M Player_Move 0x191dd0:167 `+0x18 = fStack_e8 · mult`], integrated along the matrix rows for
    /// every substate [M Player_Update 0x193600:284-295]. The zipline ride accumulates 0.15/tick into
    /// it [M Player_Zipline 0x197f50:121 `+= FRAME_RATE_MUL·0.15`] and the creep exit pushes 0.05 of
    /// it [M Player_Creep's exit tail]. The port's zipline integration rides this field downward (a
    /// port shape) — the ride itself is unreachable in the shipped game: both live wires are type 0
    /// [D placements probe, grab_wire's census], so the zipline substate is never entered.
    pub fall_speed: f32,
    /// The Q-Claw grapple's runtime state (`BLData+0x140..0x160` narrowed; fix-gadgets). `Some`
    /// while the substate 2 runs [M Player_SetGrapplePoint 0x195da0 → ChangeSubState(2)].
    pub grapple: Option<crate::gadget::GrappleState>,
    /// `BLData+0x95F`: the third-icon interaction context — 0xFF = none. `Player_CollisionHandler`
    /// 0x1949a8:22 resets it every tick and `ThirdIcon_Update` 0x196e80 writes the enabled icon's
    /// type (param 0) while the player is in its room; `Player_HandleJump` 0x1916b0:118-147 grabs the
    /// nearest wire when the jump is pressed with 3 (the cable context). The Sim (scene.rs) owns the
    /// reset + the icon writes.
    pub icon_ctx: u8,
    /// The level's `WldGravity` (0x2d8890..0x2d889c, 16 B [M work/action-symtab.tsv]). The game's
    /// `Player_Init` 0x1910f8 writes the default (0, −9.8, 0) at every spawn (disasm 0x19151c-
    /// 0x191550: `sw zero` → x, `swc1 f0` with 0xc11ccccd → y, `sw zero` → z), then zeroes the
    /// WHOLE vector when the player-start placement's param 3 == 2 (`jal Vec_Zero(&WldGravity)`,
    /// disasm 0x1915f4-0x19161c; the 0xB0 block `Player_Start` 0x190fd0 hands over, p3 at
    /// +0x38). Readers in scope: `Player_Collision` 0x194df0:107/161 (the fall,
    /// `vel += WldGravity·REC_FRAME_RATE`) and `Player_ChangeSubState` 0x190630 case 0xd/0xe
    /// (the death-anim pick, see `death_script`). fix-world-gravity.
    pub world_gravity: [f32; 3],
    /// audit-sound-calls: the jump sounds this tick produced, `(id, pos)` — the launch 0xA3 and
    /// the head bump 0xA3/0x5FB [`Player_HandleJump` 0x1916b0:184, 195-198]. Drained by the Sim
    /// into `drones.audio` (the Sim's sound list) and cleared there.
    pub sfx_out: Vec<(u32, V3)>,
    /// The zero-g orientation: obj+0x90's rotation rows (right/up/forward, the game's row-vector
    /// convention). `Player_ZeroG` composes the look rates body-frame (`m := m·R`, Concat [M
    /// 0x1e4a90]) and the auto-stabiliser slerps it world-up [M 0x192ab0:160-199]; the walk
    /// yaw/pitch scalars build it on entry (ChangeSubState 9 touches no rotation [M 0x190630]).
    pub zero_g_rot: [[f32; 3]; 3],
    /// `BL+0x24`: the vertical thruster's smoother — the (action 7 − action 8)·0.1 target eased
    /// at the game's 0.04 factor [M Player_ZeroG 0x192ab0:136-159]; the third lane the walk's
    /// `local` pair (+0x20/+0x28) doesn't cover.
    pub zero_g_lift: f32,
    /// `BL+0xd0`: the auto-stabiliser's tick window, armed to FRAME_RATE·1.5 = 90 by the
    /// action-0xe read [M Player_ZeroG 0x192ab0:163-168].
    pub stabiliser: u16,
    /// The spawn arrival settle [L, level-0a-office-hatch-5]: the game's placed root (the floor ray
    /// + 1.1 [M 0x190d88:0x190ee0-0x190f18 + 0x1c95a8]) is HELD for four ticks while the fall
    /// velocity accumulates (`vel += WldGravity·REC_FRAME_RATE` each [M 0x194df0:107, the constants
    /// 0x2d8894 = −9.8f / 0x30d0dc = 1/60]), then integrates ONCE — the ee_start_0700000a root
    /// 0x1b78640+0x34 = 0x3f8b67fe = 1.0891111 reproduces BIT-EXACTLY as 1.1f + f32(4·(−9.8f·(1/60))·(1/60))
    /// — and the feet band [M 0x194188:106] catches the dropped root at the game's 1.0891 rest.
    /// Which gate holds the boot's first four ticks (the handler phase clearly does not run: the
    /// grounded tail would zero the accumulating velocity) is not readable from the decompile —
    /// `// [S] unverified`: the port models the observable, armed ONLY at the level-load spawn
    /// (`arm_spawn_settle`), never at the mid-game `Player_Enable` 0x195498 (which does not go
    /// through `Player_StandAtNewPosition` at all — the matrix origin direct [M :57-68] — the
    /// 05 truck hand-off's crouch press is the first post-enable tick and must not be swallowed).
    pub settle_hold: u8,
    /// Port bookkeeping for the game's persistent orientation: `obj+0x90` is built from the walk
    /// yaw/pitch on entering the substates (ChangeSubState 8/9 touches no rotation [M 0x190630]) and
    /// `Player_ZeroG` composes it every tick [M 0x192ab0:136-199] — the game needs no flag; the port
    /// tracks when its scalar `zero_g_rot` mirror is live.
    pub zero_g_active: bool,
    /// The post-integration, pre-push-out position — the game's hittest build point [M
    /// `Player_Collision` 0x194df0 builds BLData+0x760 from obj+0x30 AFTER `Player_Update`'s
    /// integration 0x193600:284-295 (:361); the refine runs on it in the collision pass
    /// (`Collide_Update` 0x1ea398 from `control_movement_object_handler` 0x1336f8:238), and the
    /// +0x750 push-out lands on obj+0x30 only inside `Player_CollisionHandler` 0x1949a8:136-140,
    /// right before the hit dispatch]. The ladder grab's pick tests THIS spot: `Player_CollLadder`
    /// 0x196118 runs no distance test — the grab rides the hit's EXISTENCE, decided by the refine
    /// on the pre-push-out segment. A free-standing ladder's own collision triangles hold the
    /// pushed-out body at exactly RADIUS (0.55) — that spot sits ON the strict refine boundary [M
    /// Intersect_CylGeom 0x109da0:447 `dist < radius`] and never grabs, while the game's pre-push
    /// segment (the tick's inward move included) does.
    pub pre_collide: V3,
    /// `HITTEST+0x50` (BLData+0x790) — the pick-bit block's reference vector [M Intersect_CylGeom
    /// 0x109da0:478-480 loads it for `R = +0x50 − (B + push)`, bit ⟺ `R·axis < 0`, disasm
    /// 0x10a664-0x10a6a0]. No code writes it statically (the full-.text sdc2/sw scan at imm 0x790
    /// found nothing); its only writer is the DeltaP add-back [M Collide_Intersect 0x1ebaa8:353-373
    /// adds the world DeltaP to +0x10/+0x20/+0x30/+0x50/+0x60 per object pass with a hit], so it
    /// is READ by the bit's dot2 [M 0x10a664-0x10a6a0]; its only writer is the DeltaP add-back [M
    /// Collide_Intersect 0x1ebaa8:353-373 adds the world DeltaP to +0x10/+0x20/+0x30/+0x50/+0x60 per
    /// object pass with a hit]. The port feeds the accumulator but gates NO behaviour on it —
    /// `grounded` is `walkable || band` [M Player_FeetOnPoint 0x194188:106 + 0x194df0:104] — so the
    /// open lifetime question (the accumulator reading, 0c3) rides no shipped behaviour; a live
    /// pin of the bit block decides when it returns.
    pub pick_ref: V3,
    /// unported-front-end-menus-3: the `CAM_ORBIT` state (BLData+0xe0..0x104) [M `Camera_Orbit`
    /// 0x1bb828] — `Some` while cam_mode is 1, the GoldenEye-strike orbit (`MP_GoldenEyeUpdate`
    /// 0x187568:396-398 → `Player_SetCamMode` 0x195928 case 1, the game's only mode-1 writer).
    pub orbit: Option<OrbitCam>,
    /// The hang's persisted drift lanes ([f32; 2] = strafe, forward) — the game's `BLData+0x20`
    /// committed-velocity x/z during the grapple's phase 2 [M Player_SSGrapple 0x192860 tail
    /// (LAB_00192a24) commits the +0x10 wish into +0x20 every held tick and Player_Move
    /// 0x191dd0:111-141 eases that block toward the stick's 0.075/tick lanes; the +0x154 rope
    /// block pins y, so the committed move is horizontal only]. Zeroed on the latch
    /// (ChangeSubState 2 zeroes the velocity block [M Player_ChangeSubState 0x190630's
    /// `*(obj+0x50) = 0` arm]) and on the release (the port's pinned no-momentum drop).
    pub grapple_drift: [f32; 2],
}

impl Bond {
    /// Standing at `pos` (`Player_StandAtNewPosition`: the floor point + 1.1) facing `yaw`.
    pub fn new(pos: V3, yaw: f32) -> Bond {
        Bond { pos, yaw, pitch: 0.0, local: [0.0; 2], vel: [0.0; 3], grounded: true, floor_ny: 1.0, grounded_history: 8, jump: 0, land_delay: 0,
            crouched: false, lockout: 0, cc: ROOT_HEIGHT, crouch_drop: 0.0, eye: EYE, fall_timer: 0, head_bump: false, fall_damage: None,
            substate: SS_WALK, ladder: None, climb_top: 0.0, climb_bottom: 0.0, regrab: 0, mode: 0, saved_sub: 0, air: 100.0, drown_damage: None, underwater: false,
            water_fade: 0.0, air_tick: 0, push_fwd: 0.0, wire: None, auto_drop: 0.0, wire_anim: 0, wire_anim_time: 0.0, wire_anim_rate: 1.0, cam_mode: 0, fall_speed: 0.0, grapple: None,
            icon_ctx: 0xff,
            cel: None,
            prev_pos: pos,
            pre_collide: pos,
            pick_ref: [0.0; 3],
            creep: None,
            world_gravity: [0.0, -9.8, 0.0],
            zero_g_rot: [[0.0; 3]; 3],
            zero_g_lift: 0.0,
            stabiliser: 0,
            settle_hold: 0,
            zero_g_active: false,
            grapple_drift: [0.0; 2],
            // audit-sound-calls: the jump launch / head-bump sounds outbox (drained by the Sim
            // into `drones.audio`); `Player_HandleJump` 0x1916b0 plays 0xA3 at launch (:184) and
            // 0xA3/0x5FB at the head bump (:195-198).
            sfx_out: Vec::new(),
            // unported-front-end-menus-3: no orbit until Player_SetCamMode(BL, 1) (0x195928
            // case 1) arms it.
            orbit: None,
        }
    }

    /// The death-anim pick the death branch calls: `Player_ChangeSubState`
    /// 0x190630 case 0xd/0xe [M :68-73] — `WldGravity.y == 0.0` (the exact float test, the
    /// zero-g write is `Vec_Zero`) plays 0x60008e0 (the space death), else 0x6000151 (the
    /// ground death). `Player_HandleDeath` 0x190bb0:15 shares the same test.
    pub fn death_script(&self) -> u32 {
        if self.world_gravity[1] == 0.0 { 0x6000_8e0 } else { 0x6000_151 }
    }

    /// On the floor below `p` (within 10 units), as a level start places him.
    ///
    /// The spawn arrival settle is NOT armed here: the game's `Player_Enable` 0x195498 does not go
    /// through `Player_StandAtNewPosition` at all (it copies the matrix translation into obj+0x30
    /// DIRECTLY — no floor ray, no +1.1 — and zeroes the velocities [M 0x195498:57-68, 81-86]), so
    /// every mid-game re-enable (the 05 truck hand-off's crouch press rides on the first
    /// post-enable tick) must behave like master's placement, or the hold swallows that tick's
    /// input. The settle arms ONLY at the level-load spawn (scene.rs, the `Player_Init`
    /// 0x1910f8:156-181 path), where the ee_start evidence lives. Call `Bond::arm_spawn_settle`
    /// there.
    pub fn stand_at(world: &CollisionWorld, p: V3, yaw: f32) -> Bond {
        let y = world.floor_below([p[0], p[1] + 1.0, p[2]], 10.0).map_or(p[1], |f| f + 1.1);
        Bond::new([p[0], y, p[2]], yaw)
    }

    /// Arms the spawn arrival settle — LEVEL LOAD ONLY (see `stand_at`'s doc): the boot's
    /// `Player_Init` 0x1910f8 → `Player_StandAtNewPosition` 0x190d88 placement is what the game
    /// holds for its first ticks [L ee_start_0700000a root 1.0891111 = 1.1 +
    /// f32(4·(−9.8/60)·(1/60))]. Which game gate holds the boot's first four ticks is not
    /// readable from the decompile — `// [S] unverified` (the observable model, out of pass/fail
    /// as to the gate; the ROOT it produces is the [L] bit-exact pin).
    pub fn arm_spawn_settle(&mut self) {
        self.settle_hold = 4;
    }

    /// `Player_GetHeadPos`: the eye.
    pub fn head(&self) -> V3 { [self.pos[0], self.pos[1] + self.eye - self.crouch_drop, self.pos[2]] }

    /// The player's world-sphere centre (`obj+0x80`, `Control_BuildWorldSph` 0x133070): the local sphere
    /// centre at `M_CONTROL+0x80` through the object matrix — `world = x·row0 + y·row1 + z·row2 + row3`,
    /// i.e. the yaw-rotated local offset added to the position [M 0x133070:33-44; the player takes the
    /// `obj+0xFA & 8` branch which reads M_CONTROL (obj+0xDC), radius `M_CONTROL+0x8C · |obj+0xE8|`].
    /// `M_CONTROL+0x80 = (−0.1347, −0.1267, 0.2402)` local, radius 1.0576 at the 07000005 start `[L]
    /// ee_start_05.bin M_CONTROL 0x16cb8a0+0x80/0x8C`; the computed centre (104.826, 24.837, 108.582)
    /// matches the player object's `+0x80` there `[L] 0x16cefe0+0x80`.
    pub fn sphere_centre(&self) -> V3 {
        Self::sphere_centre_at(self.pos, self.yaw)
    }

    /// The world sphere centre for a feet position + yaw — the same rule as `sphere_centre`, shared
    /// with the bodies that have no `Bond` pose of their own (the MP bots' pickup-take test reads
    /// their obj+0x80 the same way [M `Pickup_Update` 0x18e148:212-214 over every body]).
    pub fn sphere_centre_at(pos: V3, yaw: f32) -> V3 {
        // [L] M_CONTROL+0x80 (ee_start_05.bin, player 0 standing).
        const LOCAL: V3 = [-0.1347, -0.1267, 0.2402];
        let (s, c) = (crate::trig::sinf(yaw), crate::trig::cosf(yaw)); // G-046: the matrix rows are PS2Sinf-built [M 0x133070]
        // The object matrix rows [cos,0,−sin / 0,1,0 / sin,0,cos] (+ translation = pos), as read live
        // [L 0x16cefe0+0x90..0xc8] and matching `Bond::look`'s convention.
        [pos[0] + LOCAL[0] * c + LOCAL[2] * s, pos[1] + LOCAL[1], pos[2] - LOCAL[0] * s + LOCAL[2] * c]
    }

    /// The view direction (pitch applied to the forward axis). In the zero-g substates the whole
    /// orientation lives in the object matrix — ZeroG zeroes the scalar pitch BL+0x8a8 EVERY tick
    /// [M Player_ZeroG 0x192ab0:74] — so the view rides the matrix's forward row.
    pub fn look(&self) -> V3 {
        if self.zero_g_active {
            return self.zero_g_rot[2];
        }
        // G-046: the game's dir is RotMatrixX(view pitch) · Mat_GetDir — PS2Sinf2 lanes, EE products
        // [M RotMatrixX 0x1e46a0:16-24; the pitch arg `BL+0x8a8·π/2` a mul.s].
        let p = nf_formats::ps2float::mul(self.pitch, crate::trig::HALF_PI);
        let (sp, cp) = (crate::trig::sinf(p), crate::trig::cosf(p));
        let (sy, cy) = (crate::trig::sinf(self.yaw), crate::trig::cosf(self.yaw));
        let mul = nf_formats::ps2float::mul;
        [mul(cp, sy), sp, mul(cp, cy)]
    }

    /// The capsule ends (top A, bottom B) for the current stance, off the dynamic root height `cc`
    /// (`Player_Collision` 0x194df0: standing (default arm) `0.275` / `0.55 − cc`; the crouch/swim
    /// case 3 `0.1 − (cc − 0.55)` / `0.45000002 − cc`, stores to BLData+0x760/+0x770 with the radius
    /// slot +0x7cc = 0x3f0ccccd 0.55 [M 0x194df0:235/248 + the tail store :405-408]). The constants
    /// are the game's f32 literals evaluated in the game's order — NOT the algebraic simplifications:
    /// `0.65 − 0.55f32` = 0.1000000238418579 but the game's `0.1 − (0.55 − 0.55)` = 0.1f32, and
    /// `0.45f32 − cc` = −0.10000002384185791 but the game's `0.45000002f32 − cc` (0x3EE66667) =
    /// −0.09999999403953552. The 3e-8 difference is the crouch grounded band: the feet-point ray is
    /// built from the B arm (see the physics_tick comment), so the port's lower B hung the crouch
    /// rest EXACTLY on `Player_FeetOnPoint`'s band edge and the push dither flickered Bond ungrounded
    /// at every hard-floor crouch rest (probe14_ground.rs, the 07000014 crawl-vent start).
    ///
    /// The arms follow the hittest switch's own table [M Player_Collision 0x194df0, second switch
    /// @0x195070 indexed substate−1, jump table @0x2fc0d0]: the crouch substate 4 alone takes the
    /// LOW arm (case 3: A = pos + up·(0.1 − (cc − 0.55)), B = pos + up·(0.45 − cc)); the DEFAULT
    /// arm (A = pos + up·0.275, B = pos + up·(0.55 − cc), r 0.55) covers the walk 0 AND the swim 3
    /// — the underwater path jumps past the gravity straight into this switch [M disasm
    /// 0x194f4c-0x194f70]. The swim pair rode the LOW arm pre-fix (the same misread 750a made for
    /// the swim grab pick): its lowest point sat cc + 0.10 under the root against the game's cc, so
    /// the pond-bed push-out parked the swimmer ~0.1 high (the 02 pond: 3.60/3.70 settles
    /// against the PS2's 3.48/3.17 [L]; docs/systems/swim.md). The grab pick's own swim arm is the
    /// default one [M ladder.rs::capsule_hits_default] — the pair now agrees with it.
    fn capsule(&self) -> (V3, V3) {
        let (a, b) = if self.crouched {
            (0.1 - (self.cc - 0.55), 0.450_000_02 - self.cc)
        } else {
            (0.275, 0.55 - self.cc)
        };
        ([self.pos[0], self.pos[1] + a, self.pos[2]], [self.pos[0], self.pos[1] + b, self.pos[2]])
    }

    /// The capsule ends for the current stance, public for the Sim's wire grab scan (the game's
    /// `Player_Collision` 0x194df0 builds the same segment as the HITTEST).
    pub fn capsule_seg(&self) -> (V3, V3) { self.capsule() }

    /// One 60 Hz tick (no ladders/water context — the pre-ladder signature, still used by the unit tests).
    pub fn tick(&mut self, world: &CollisionWorld, input: &Input) {
        self.tick_ctx(world, &LadderCtx::default(), input);
    }

    /// One 60 Hz tick with the ladder/water context: `Player_Update` 0x193600 — the regrab decay
    /// ([M 0x193840-0x19384c], the `obj+0xF4 == 1` branch that also zeroes BLData+0x10/+0x30 every tick),
    /// the substate dispatch ([M switch @0x193958]), then `Player_MonitorAir` 0x191a70 [M — it runs after
    /// every substate, before the collision pass].
    pub fn tick_ctx(&mut self, world: &CollisionWorld, ctx: &LadderCtx, input: &Input) {
        self.regrab = self.regrab.saturating_sub(1);
        self.fall_damage = None;
        self.drown_damage = None;
        match self.substate {
            SS_CLIMB => self.climb_tick(world, ctx, input),
            SS_GRAPPLE => self.grapple_tick(&input),
            SS_SWIM => self.swim_tick(world, ctx, input),
            SS_WIRE => self.wire_tick(world, ctx, input),
            SS_CREEP => self.creep_tick(world, ctx, input),
            SS_ZIPLINE => self.zipline_tick(world, ctx, input),
            SS_ZEROG_8 | SS_ZEROG_9 => self.zerog_tick(world, ctx, input),
            _ => self.walk_tick(world, ctx, input),
        }
        if !(SS_ZEROG_8..=SS_ZEROG_9).contains(&self.substate) {
            self.zero_g_active = false;
        }
        self.monitor_air(ctx);
    }

    /// `Player_ZeroG` 0x192ab0 — the spacewalk substate (the dispatch runs cases 8 and 9 into the
    /// same body [M 0x193600:238-241]; the script only ever sets 9 [M C_CHCHZEROG 0x216388], and
    /// only 9 plays the movement anim set [M 0x192ab0:201]). The body rides `physics_tick`'s
    /// collision tail with the walk/swim move + gravity replaced.
    fn zerog_tick(&mut self, world: &CollisionWorld, ctx: &LadderCtx, input: &Input) {
        self.physics_tick(world, ctx, input, false, true);
    }

    /// `Player_SSWalk` 0x192310 / `Player_SSCrouch` 0x1924c0 (the walk/crouch substates share this body in
    /// the port; the game splits them). `Player_SSWalk`'s FIRST statement is the water check
    /// [M 0x192310:29-32 `if (Player_InWater()) return;`] — walking into water enters the swim substate
    /// instead of the walk tick. The ladder grab runs last: the game's `Player_CollisionHandler`
    /// 0x1949a8 follows the substate update and `Player_DealWithObjHit` 0x194458 dispatches hit class
    /// 0x2D to `Player_CollLadder` 0x196118.
    fn walk_tick(&mut self, world: &CollisionWorld, ctx: &LadderCtx, input: &Input) {
        if self.enter_water(ctx) { return; }
        self.physics_tick(world, ctx, input, false, false);
        self.ladder_grab(ctx);
    }

    /// `Player_SSGrapple` 0x192860 through `gadget::grapple_tick`: the reel toward the grapple
    /// point, the hang, and the fire's release (the substate 0). The release carries no momentum in
    /// the shipped flow: the grab zeroed all four velocity triples (`Player_ClearInertia` 0x195e08
    /// via `Player_CollWire`) and the reel moves Bond by writing obj+0x30 directly, so the +0x10
    /// triple the game's release arm copies into the +0x20 block [M Player_SSGrapple 0x192860:59-64,
    /// LAB_00192a24] is zero — the drop IS the game's behaviour. The reel ignores gravity [M the
    /// substate body writes obj+0x30 directly].
    ///
    /// The phase-2 HANG drift is ported (level-14-full-walk-2): `Player_Move(1.0)` 0x191dd0 runs at
    /// the top of the substate and eases the stick's wish into the +0x20 block (the 0.075/tick
    /// lanes, the 0.2 ease, the ±0.1 clamp [M 0x191dd0:111-141]); the tail (LAB_00192a24) commits
    /// it every held tick, the engine's move pass applies it, and the +0x154 rope block pins y —
    /// the hang drifts horizontally at the walk's speed, never vertically. The port eases the same
    /// lanes in `grapple_drift`, moves pos by the walk's own axis add [M the walk's application of
    /// the +0x10 quad] and holds y — the rope pin; no push rides the drift (the dead push tail).
    fn grapple_tick(&mut self, input: &Input) {
        let Some(st) = self.grapple.as_mut() else { self.substate = SS_WALK; return };
        let mut pos = self.pos;
        let fire = input.fire;
        let released = crate::gadget::grapple_tick(&mut pos, st, fire);
        if !released && st.phase == 2 {
            let ease = |v: f32, target: f32| -> f32 {
                let nv = v + (target - v) * 0.2;
                if (target - v) * (target - nv) < 0.0 { target } else { nv.clamp(-0.1, 0.1) }
            };
            self.grapple_drift[0] = ease(self.grapple_drift[0], input.strafe * 0.075);
            self.grapple_drift[1] = ease(self.grapple_drift[1], input.forward * 0.075);
            let (s, c) = (crate::trig::sinf(self.yaw), crate::trig::cosf(self.yaw));
            let (vs, vf) = (self.grapple_drift[0], self.grapple_drift[1]);
            pos[0] += c * vs + s * vf;
            pos[2] += -s * vs + c * vf;
        }
        self.pos = pos;
        self.vel = [0.0; 3];
        self.grounded = false;
        if released {
            self.grapple = None;
            self.grapple_drift = [0.0; 2];
            self.substate = SS_WALK;
        }
        // No capsule push on the drift: the game's push tail is dead for the player's own move
        // pass [M Intersect_CylGeom 0x109da0:447 — the DeltaP guard kills every walk contact; the
        // port's walk block keeps the contact only], and the same engine serves the grapple's
        // move — Bond drifts the game's lines on the feet band alone [M Player_FeetOnPoint
        // 0x194188:106], the rope pin holding y.
    }

    /// `Player_Update` case 3 [M 0x193958 case 3]: `Player_Move(1.0, …)` — the walk physics with the
    /// multiplier 1.0 — then the inline swim tail: with a cel, Bond's y-velocity is clamped so he cannot
    /// rise above `cel+0x90 − 0.3`, and he leaves the water when grounded and `cel+0x90 <= pos.y − 0.3`
    /// (`BLData+0x130 = 2`, `ChangeSubState(0)`). `Player_Collision` 0x194df0 applies **no gravity** while
    /// Bond's y is below the water surface [M the default case jumps past `WldGravity` to the substate
    /// capsule switch when `cel+0x90 > obj+0x34`].
    fn swim_tick(&mut self, world: &CollisionWorld, ctx: &LadderCtx, input: &Input) {
        self.physics_tick(world, ctx, input, true, false);
        let Some(w) = ctx.water else { self.substate = SS_WALK; return };
        if self.grounded && w <= self.pos[1] - 0.3 {
            // Leave the water [M case 3: `(AnimObj+0x60 & 8) && cel+0x90 <= pos.y − 0.3` → substate 0].
            self.substate = SS_WALK;
            self.ladder = None;
            return;
        }
        // The vertical comes from the look-directed forward (physics_tick's swim branch [M
        // 0x191dd0:149-165]) — NOT from crouch/jump: the game's case 3 reads no such input (the old
        // `±2.4` on jump/crouch was a guess, replaced by the game's rule: dive = look down + forward,
        // surface = look up + forward [M 0x191dd0:149-165]). The surface clamp [M 0x193600:201-220]:
        // `pos.y < surface − 0.3` → the world-Y move is clamped so Bond rises AT MOST to
        // `surface − 0.3`; else any upward move is zeroed.
        if self.pos[1] < w - 0.3 {
            self.vel[1] = self.vel[1].min((w - 0.3 - self.pos[1]) * 60.0);
        } else if self.vel[1] > 0.0 {
            self.vel[1] = 0.0;
        }
        self.pos[1] += self.vel[1] / 60.0;
        // The grab scan runs from the swim substate too — `Player_CollLadder` excludes only substates 1
        // and 2, and `Player_CollisionHandler`'s grab dispatch follows every substate update [M 0x196118].
        self.ladder_grab(ctx);
    }

    /// `Player_CollLadder` 0x196118, fed by the `Player_DealWithObjHit` 0x194458 class-0x2D dispatch over
    /// the level's ladder objects. Guards [M 0x196118:16-17]: substate ∉ {1, 2}, regrab `+0x951 == 0`,
    /// health `+0x894 > 0` (the port keeps health in the Sim's `drones.health` — a dead Bond gets no input,
    /// so the health guard is enforced by the caller). Conditions [M]: `|AngleDiff(ladder.yaw, yaw)| > π/2`,
    /// then the y window: `bottom = cy − r + 1.0 < pos.y ≤ clamped ≤ top = cy + r + 0.85` with `pos.y`
    /// raised to `bottom + 0.02` first. On grab: snap to `centre + fwd(yaw)·0.5`, zero the horizontal
    /// velocity (`obj+0x50/+0x58 = 0`), mode `+0x94D = 1`, `+0x130 = 2`, `ChangeSubState(1)`,
    /// `Player_WeaponNone` (the Sim holsters to fists), `BLData+0x884 = ladder`.
    fn ladder_grab(&mut self, ctx: &LadderCtx) {
        // The game's guards [M 0x196118:16-17]: substate must not be 1 (climb) or 2 (grapple) — swim
        // (3), crouch (4) and walk (0) all grab; plus the regrab lockout (+0x951) and health. The old
        // `!= SS_WALK` gate silently killed the swim-grab call swim_tick already made (the 02 pond
        // hut ladder could not be grabbed from the water).
        if matches!(self.substate, SS_CLIMB | SS_GRAPPLE) || self.regrab != 0 { return; }
        // The pick shape follows the substate's HITTEST arm. The switch indexes on substate−1 [M
        // disasm 0x19506c-0x195098: `lhu 0xf6($s3)`, `addiu −1`, the jump table — so 0x194df0's
        // "case 0" is the CLIMB substate 1 (r 0.275, A = up·0.55, B = up·(0.275 − cc)) and its
        // "case 3" is the CROUCH substate 4 (the LOW short segment A = up·(0.1 − (cc − 0.55)),
        // B = up·(0.45 − cc), r 0.55); the switch's DEFAULT arm (r 0.55, A = up·0.275,
        // B = up·(0.55 − cc)) covers every other substate — INCLUDING the walk 0 and the SWIM 3
        // (both underwater and dry: the underwater path jumps past the gravity straight into this
        // switch [M disasm 0x194f4c-0x194f70: `cel+0x90 > obj+0x34` → `bc1fl 0x195070`]). The
        // underwater ladder softlock (750a): the swim grab ran the CROUCH arm here, whose LOW spine
        // (A = −0.385) misses rungs the game's default arm reaches.
        //
        // ladder-0c: the WALK arm is the DEFAULT arm, pinned by the switch's own jump table [M
        // Player_Collision 0x194df0 second switch @0x195070: the index is substate−1 (`lhu 0xf6($s3)`,
        // `addiu −1`, `sltiu $v1, $a0, 0xf` bounds, the table @0x2fc0d0) — walk 0 → −1 → 0xffffffff
        // fails the unsigned bounds → the default arm @0x1953f4 (r +0x7CC = 0x3f0ccccd = 0.55, A =
        // up·0.275 = 0x3e8ccccd, B = up·(0.55 − cc)); only the substates 1 (climb) and 7 (creep) take
        // the narrow 0.275 arm (@0x1950a0). fix-ladder-water2's "case 0/6 = substate 0 (walk)" mapped
        // the switch on the raw substate — the table maps case 0 → substate 1. The 0.275 arm left the
        // 0700000c yard ladder #164 unclimbable: Bond's walk capsule stops him 0.4985 from its rung
        // plane (the silo wall sits ~0.05 behind the plane [D probes]) and 0.4985 > 0.275 never
        // touches — the game's 0.55 arm does (0.4985 < 0.55), and the dismount escape still clears:
        // the dismount hop −0.05 [M Player_Climb 0x196310:88-96, consumed the same tick by the
        // integration Player_Update 0x193600:284-295] puts the stander 0.55 out, the refine's strict
        // `dist < radius` [M Intersect_CylGeom 0x109da0:447] rejects the exact touch, and the swim
        // escape's drift (0.5 + 0.05 + 0.2 > 0.55) still clears.
        for (i, lad) in ctx.ladders.iter().enumerate() {
            // The straddled-cel scoping [M Collide_Pick 0x1eb320:125-134]: the pick admits only
            // objects linked to a cel the HITTEST straddles — the ladder's link is its centre's cel
            // [M build_LinkToRoom 0x1c9680] and the straddle set is the Sim's portal BFS over the
            // pick capsule. An empty set (no cel graph) scopes nothing.
            if !ctx.straddle.is_empty() {
                match lad.cell {
                    Some(c) if ctx.straddle.contains(&(c as usize)) => {}
                    _ => continue,
                }
            }
            // The pick runs on the PRE-push-out spot [M — see the `pre_collide` field doc]: the
            // game's grab rides the hit existence from the hittest built at the integrated
            // position, before the handler's push-out.
            let pick = self.pre_collide;
            let hit = if self.crouched {
                // The crouched walk is the game's substate 4 (the port folds the crouch into
                // substate 0 + the flag) — the switch's case 3, the LOW arm [M 0x194df0 case 3].
                lad.capsule_hits_crouch(pick, RADIUS, self.cc)
            } else {
                // The walk 0, the swim 3 and every other grab-eligible substate ride the switch's
                // DEFAULT arm [M 0x1953f4 via the jump table @0x2fc0d0].
                lad.capsule_hits_default(pick, RADIUS, self.cc)
            };
            if !hit { continue; }
            let diff = wrap_pi(lad.yaw - self.yaw); // Vec_AngleDifference(player, ladder) [M 0x1e2b78]
            if diff.abs() <= std::f32::consts::FRAC_PI_2 { continue; }
            let (top, bottom) = (lad.top(), lad.bottom());
            self.climb_top = top;
            self.climb_bottom = bottom;
            self.pos[1] = self.pos[1].max(bottom + 0.02);
            if !(bottom < self.pos[1] && self.pos[1] < top) { continue; }
            // G-046: Player_CollLadder 0x196118:31-38 [M]: a = lad.yaw + 3.1415927 (add.s); x = centre −
            // PS2Sinf(a)·0.5, z = centre − PS2Sinf(a + 1.5707964)·0.5 (EE mul.s/sub.s) — the game's own
            // sine arguments, not sin/cos of the raw yaw.
            let a = crate::trig::ee_add(lad.yaw, crate::trig::PI);
            let mul = nf_formats::ps2float::mul;
            self.pos[0] = crate::trig::ee_sub(lad.centre[0], mul(crate::trig::sinf(a), 0.5));
            self.pos[2] = crate::trig::ee_sub(lad.centre[2], mul(crate::trig::sinf(crate::trig::ee_add(a, crate::trig::HALF_PI)), 0.5));
            self.vel[0] = 0.0;
            self.vel[2] = 0.0;
            self.mode = 1;
            self.substate = SS_CLIMB;
            self.ladder = Some(i);
            break;
        }
    }

    /// `Player_Climb` 0x196310 (substate 1) [M]. Order: the yaw clamp+ease toward `ladder.yaw + π`;
    /// `Player_Move(0, …)` (no input movement — the local axes decay); `Vec_Zero(BLData+0x50)` (the world
    /// velocity zeroes every climb tick); the ladder carry `pos += ladder(+0xC0) − ladder(+0x40)` (zero for
    /// the port's static ladders); the input/height logic:
    /// - action (7) pressed with `forward < 0.05` → dismount, `+0x18 −= 0.05` (a slight backward push);
    /// - `pos.y > top` → `+0x18 += 0.6` (a forward lunge onto the platform), dismount;
    /// - `pos.y >= bottom` → climb: `fwd = forward`; a hit above (`AnimObj 0x20`) zeroes an up-push;
    ///   `!(hit below && fwd < 0)` → `+0x14 += fwd·0.02` (0.02 units/tick = 1.2 u/s at full push), stay;
    ///   else dismount;
    /// - `pos.y < bottom` → dismount.
    /// Every dismount: `regrab = (u8)FRAME_RATE` (60), `+0x942 = substate`, the anim carry cleared, and
    /// `ChangeSubState(0)` through the shared tail.
    fn climb_tick(&mut self, world: &CollisionWorld, ctx: &LadderCtx, input: &Input) {
        let li = match self.ladder {
            Some(li) if ctx.ladders.get(li).is_some() => li,
            _ => { self.substate = SS_WALK; self.ladder = None; self.physics_tick(world, ctx, input, false, false); return; }
        };
        let lad = &ctx.ladders[li];
        // The yaw clamp+ease [M 0x196310:18-33]: target = ladder.yaw + π (facing the ladder). The ease
        // uses the PRE-clamp diff (the game overshoots the clamp on the clamp tick — audit #4).
        let target = lad.yaw + std::f32::consts::PI;
        let diff0 = wrap_pi(target - self.yaw);
        let mut diff = diff0;
        if diff > std::f32::consts::FRAC_PI_2 { self.yaw = target - std::f32::consts::FRAC_PI_2; diff = std::f32::consts::FRAC_PI_2; }
        if diff < -std::f32::consts::FRAC_PI_2 { self.yaw = target + std::f32::consts::FRAC_PI_2; diff = -std::f32::consts::FRAC_PI_2; }
        // The ease runs while the AnimObj flag bit 0 is clear — bit 0 is the AIMING flag, managed by
        // `Player_Weapon`/`Player_Aiming` and cleared by `Player_WeaponNone` on the grab [M 0x1a9858:18-19
        // `&= 0xfffe`; 0x1a24c8:45-62] — so the ease pauses while the aim key is held (fix-ladder2).
        if !input.aiming { self.yaw += diff0 * 0.2; }
        // The look during the climb (fix-ladder2) [M 0x196310 + 0x193600 + 0x1a1d28/0x1a92e8]: the turn
        // channel (`bd+0x34`) still integrates into the yaw after `Player_Climb` — `Player_Update`'s
        // `bd+0x30..0x3F` → `obj+0x50..0x5F` integration — and the next tick's clamp above pulls it back
        // into the ±90° window; the pitch offset (`BL+0x8A8`) takes the stick like on foot:
        // `Player_Aiming` runs for every substate ≤ 7 (climb = 1 passes its early-out [M 0x1a1d28:21-30])
        // and `Player_ViewClamping` clamps ±1.0 [M 0x1a92e8:9-16].
        self.yaw += input.turn;
        self.pitch = (self.pitch + input.look).clamp(-1.0, 1.0);
        // No walk movement; the local axes decay toward 0 (Player_Move's smoothing with target 0) [M].
        self.local[0] *= 0.8;
        self.local[1] *= 0.8;
        self.vel = [0.0; 3]; // Vec_Zero(BLData+0x50) [M]
        // The ladder carry [M the +0xC0/−0x40 pair]: static ladders → 0 (no moving-ladder code found).
        // Input: action pressed (jump) with no forward → the action dismount [M 0x196310:45-48, 88-96].
        let fwd_raw = if input.aiming { 0.0 } else { input.forward.clamp(-1.0, 1.0) };
        let action = input.jump;
        let mut dismount_push = 0.0f32; // the +0x18 (local forward) one-tick push
        if action && fwd_raw < 0.05 {
            self.push_fwd = -0.05;
            dismount_push = -0.05;
            self.dismount();
        } else {
            // Recompute the thresholds every tick [M 0x196310:49-53].
            self.climb_top = lad.top();
            self.climb_bottom = lad.bottom();
            if self.climb_top < self.pos[1] {
                // Above the top: the +0.6 forward lunge [M 0x196310:54-62], consumed next tick.
                self.push_fwd = 0.6;
                dismount_push = 0.6;
                self.dismount();
            } else if self.climb_bottom <= self.pos[1] {
                // Mid-ladder: climb [M 0x196310:63-78].
                let mut fwd = fwd_raw;
                // A hit above (AnimObj flag 0x20 — the port's head_bump) blocks an up-push [M].
                if self.head_bump && fwd > 0.0 { fwd = 0.0; }
                // A hit below (flag 8) with a down-push dismounts [M]; the game's flag comes from the
                // CollisionHandler's climb scan: the tick's ladder hit, `hit(+0x40).y − BLData+0x760.y`
                // (the pre-push base) < 0 → flag 8, > 0 → flag 0x20 [M Player_CollisionHandler
                // 0x1949a8:59-79]. The port reads the CEL-SCOPED feet ray instead (the game's player
                // collision tests only the cels the straddle walk keeps from obj+0x20 [M
                // Player_FeetOnPoint 0x194188 → Collide_StraddleCels 0x1eac98]): a climb-down over the
                // cel's bed, so the climb keeps going UNDER the surface to the bottom threshold
                // (centre.y − radius + 1.0 [M 0x196118]) and the dismount lands the swimmer below
                // the grab window instead of dropping him onto the bed right above it (the ladder-02
                // "infinitely latching onto the ladder" — the flat read dismounted the climb on the
                // pond bed mid-column, and the swimmer settled inside the grab column; the ladder's
                // authored mesh was measured to reach the settle spot, so the bed-hit path is the
                // divergence). No cel (the unit-test signature) falls back to the flat water-
                // filtered ray (segment_hit drops the mat-0xE planes).
                let floor_below = world
                    .segment_hit_scoped(self.cel, [self.pos[0], self.pos[1] + 0.5, self.pos[2]], [self.pos[0], self.pos[1] - 1.2, self.pos[2]])
                    .is_some();
                if !floor_below || fwd >= 0.0 {
                    self.vel[1] = fwd * 0.02 * 60.0; // +0x14 += fwd·0.02 per tick → 1.2·fwd u/s
                    self.physics_tick_climb(world);
                    return;
                }
                self.dismount();
            } else {
                // Below the bottom: dismount [M 0x196310:79-86].
                self.dismount();
            }
        }
        // The climb tick still runs the collision pass [M — Player_Collision/Handler run for substate 1;
        // the CollisionHandler's climb branch scans the hit list for the ladder and sets the 0x20/8 flags].
        self.physics_tick_climb(world);
        let _ = dismount_push; // the push is stored at the dismount sites below
    }

    /// The climb dismount common path [M 0x196310 dismount branches]: `regrab = (u8)FRAME_RATE`,
    /// `+0x942 = substate`, `+0x14 = 0`, the anim carry (`AnimObj+0xDC = 0`), `ChangeSubState(0)` (the
    /// port stores the substate directly; the game's dispatch case 10 path is the scripted-cam restore,
    /// not this). `+0x94D` (the mode byte) is NOT touched by the game's dismount — the next grab sets it.
    fn dismount(&mut self) {
        self.regrab = 60;
        self.saved_sub = SS_CLIMB as u16;
        self.substate = SS_WALK;
        self.ladder = None;
        self.local[1] = 0.0; // the dismounts write BLData+0x14 = 0 [M]
    }

    /// The collision pass for the climb substate: the capsule push and the floor ray (the game runs
    /// `Player_Collision` 0x194df0 + `Player_CollisionHandler` 0x1949a8 for every substate), without the
    /// walk-only bookkeeping (jump/land/fall timers).
    fn physics_tick_climb(&mut self, world: &CollisionWorld) {
        // The climb velocity integrates first (the game's wish plumbing lands it in the position before
        // the collision pass).
        for k in 0..3 { self.pos[k] += self.vel[k] / 60.0; }
        // The climb substate's OWN capsule [M Player_Collision 0x194df0 case 0 — the switch indexes
        // on substate−1 (disasm 0x195070), so case 0 IS the climb 1]: radius +0x7CC = 0x3e8ccccd
        // (0.275 — HALF the 0.55 stance radius), A = pos + up·0x3f0ccccd (0.55),
        // B = pos + up·(+0x7CC − cc). The 750a softlock: the climb ran the fat 0.55 stance capsule
        // and WEDGED in the undergarden shaft above 07000002's deep ladder (−50.35, 5.50 [D]) —
        // head_bump every tick ate the up-push [M Player_Climb 0x196310:69-71] and Bond hung on the
        // ladder forever, unable to climb out (a softlock on 02's underwater
        // ladder).
        let a_off = 0.55f32;
        let b_off = 0.275 - self.cc;
        let a = [self.pos[0], self.pos[1] + a_off, self.pos[2]];
        let b = [self.pos[0], self.pos[1] + b_off, self.pos[2]];
        const CLIMB_RADIUS: f32 = 0.275; // 0x3e8ccccd [M 0x194df0 case 0]
        // The pass is the actor's own cel's object lists (fix-08-collision): `Player_Collision`
        // 0x194df0 stores obj+0x20 (the relinked cel) into the BLData+0x740 HITTEST's start-cel
        // field (`*(BLData+0x7c0) = *(obj+0x20)`, the function's first store), and `Collide_Pick`
        // 0x1eb320 walks `Collide_StraddleCels` 0x1eac98's result, testing each straddled cel's
        // OWN list (`*(cel + 0x18)`) — walls hanging on other rooms' cels never block. `self.cel`
        // is that relink (obj+0x20; the Sim refreshes it off the position each tick, the
        // `build_LinkToRoom` 0x1c9680 order); `None` falls back to the flat world.
        let (push, walkable) = world.capsule_push_scoped(self.cel, a, b, CLIMB_RADIUS, false, self.pick_ref);
        // The pick-contact bit is measured but NOT gated into `grounded` (hatch-crouch-0c4):
        // `Player_FeetOnPoint` 0x194188 registers the stand-on contact on the ridden object's
        // `obj+0xD0` list through `Coll_AddHitToList` 0x1ede68 (dist forced 1.0, normal =
        // CONSTANT_UP1, point = obj+0x40 [M 0x194188:138-150]) — the port tracks the ground
        // object as the `ground_move` pair instead, no contact list (a port shape — the game keeps
        // the ridden object's obj+0xD0 list).
        // `Player_FeetOnPoint` 0x194188's tail does ground on `HITTEST+0x9c != 0` alongside the
        // band [M 0x1943a8-0x1943bc], but the port's dot2 model — the bit needs
        // `pick_ref_y < b_pre_y` with `pick_ref` a lifetime push accumulator — rests on HITTEST+0x50
        // being never-reinitialized, and the bit word's per-tick
        // clear is not pinned (the decompile's `*(u16*)(BLData+0x7d8) = 0` store resolves to
        // HITTEST+0x98, base+0xcc raw — `sh $s3, 0x70c($a0)` @0x194ff4 — not the +0x9c bit).
        // As built (0c3) the arm fired on positive-floor levels wherever an up-facing contact
        // met the band's miss and moved EVERY body (failures in mp_world_sync's
        // sub-pen plate walk, escort_follow's stairwell, the four-human digest). The 0c hatch
        // set — including the real walked route — is green WITHOUT the arm, so the grounded
        // rule stays as is until a live check pins the game's bit. See world.rs
        // `capsule_pick_contact_scoped`.
        let pick = world.capsule_pick_contact_scoped(self.cel, a, b, CLIMB_RADIUS, self.pick_ref[1], self.pick_ref);
        let _ = pick;
        self.head_bump = push[1] < -1e-4;
        for k in 0..3 { self.pos[k] += push[k]; }
        // The DeltaP add-back bookkeeping rides the climb tick too [M Collide_Intersect
        // 0x1ebaa8:353-373]; `pick_ref` stays documented-but-unused for grounding (see above).
        for k in 0..3 { self.pick_ref[k] += push[k]; }
        // The floor test reads the same HITTEST (`Player_CollisionHandler` 0x1949a8 →
        // `Player_FeetOnPoint(param_1, BLData+0x740, …)`), so it shares the per-cel scope. The ray
        // and the band are the game's own: to = capsuleB + up·(−(cc+0.6)) [M Player_FeetOnPoint
        // 0x194188:89-98 — HITTEST+0x30 = BLData+0x770, the climb B arm pos + up·(0x7cc − cc) from
        // Player_Collision 0x194df0 case 0 + the tail store :405-408], and HITDATA+0xC =
        // |from − hit| with hit = from + t·dir [M Intersect_RayGeom 0x10a8c8:176-179/252-256/
        // 312-325]; band `hit − 0.1 <= cc` [M 0x194188:106]. See the walk tick's comment.
        let ray = self.cc + 0.6;
        let to = [b[0], b[1] + -ray, b[2]];
        let floor = world.segment_hit_scoped(self.cel, self.pos, to);
        let band = floor.as_ref().map_or(false, |(t, _)| {
            let d_y = to[1] - self.pos[1];
            let hit_y = self.pos[1] + t * d_y;
            let dy = self.pos[1] - hit_y;
            (dy * dy).sqrt() - 0.1 <= self.cc
        });
        // The pick-contact arm rides the climb tick too (the same FeetOnPoint rule — see the
        // walk tick's comment) — NOT gated into grounded: the pick result is measured only (see
        // the pick call above).
        self.grounded = walkable || band;
        // floor_ny is the band-gated BLData+0x110 [M Player_FeetOnPoint 0x194188:85/:106-109;
        // 0x194df0:104] — see the walk tick's comment.
        self.floor_ny = match floor.as_ref() {
            Some((_, n)) if band => n[1],
            _ => 1.0,
        };
    }

    /// `Player_ClearInertia` 0x195e08 [M]: the four velocity triples (`BLData+0x10/+0x20/+0x30/+0x40`)
    /// zero. The port models +0x10 as `vel` and +0x14's push as `push_fwd`; `local` is +0x28's pair.
    /// (Distinct from `Player_ClearAimInertia` 0x1a9790 — BLData+0x128/+0x12C, the aim-mode look
    /// deltas, zeroed by `Player_Weapon` 0x1a24c8:185 and `Player_StandAtNewPosition` 0x190d88:75
    /// [M]; the port applies those deltas directly each tick and stores nothing, so the clear is
    /// a natural no-op — docs/GAPS.md.)
    fn clear_inertia(&mut self) {
        self.vel = [0.0; 3];
        self.local = [0.0; 2];
        self.push_fwd = 0.0;
    }

    /// `Player_CollWire` 0x197130 — the grab. Guards [M 0x19716c-0x1971a8]: the regrab lockout
    /// `+0x951 == 0`, health `+0x894 > 0` (the caller checks — the port keeps health in the Sim, like
    /// the ladder grab), wire type ≠ 1, and the substate ≠ the type's target (6 for type 0, 0xF for
    /// type 2). Builds the grab segment (`radius − 2.0` at the object's pitch), the closest point to
    /// Bond; a zipline refuses grabs within half the segment of the LOWER end. Then `Player_ClearInertia`,
    /// `ChangeSubState(6 or 0xF)`, mode `+0x94D` = 6 (type 0 → 0xC, the grab pose, with anim
    /// 0x060005e5; type 2 → anim 0x06000095), `+0x130 = 2` (the port does not model +0x130), the
    /// auto-drop timer zeroed, the yaw = the nearer of `wire.yaw ± π/2` to Bond's old yaw (type 2 picks
    /// by the segment's slope instead), the hang offset 1.0207 (type 0) / 1.17363 (type 2) below the
    /// closest point, the velocity slots zeroed, `Player_WeaponNone` (the Sim holsters — the caller),
    /// `SetCamMode(0xB)`, and the hand-reach correction: `pos −= R(yaw)·(root(0x060005e5 @last) −
    /// root(@0))` with x −= 0.082 [M 0x1974b8-0x197538, disasm 0x1975b0-0x197618; the track values come
    /// from `WireAnims`]. The caller has already resolved `wi` (the collision dispatch, the jump grab
    /// or the use key).
    pub fn grab_wire(&mut self, wi: usize, ctx: &LadderCtx) {
        let Some(w) = ctx.wires.get(wi) else { return };
        if self.regrab != 0 { return; }
        if w.wire_type == 1 { return; }
        let target = if w.wire_type == 2 { SS_ZIPLINE } else { SS_WIRE };
        if self.substate == target { return; }
        let (a, b) = w.grab_segment();
        if w.wire_type == 2 {
            // A zipline refuses grabs within half the segment of the lower end [M 0x197260-0x19728c:
            // `if (Dist3D(pos, lower) < Dist3D(A, B) · 0.5) return`].
            let low = if a[1] < b[1] { a } else { b };
            if dist3_sq(self.pos, low) < 0.25 * dist3_sq(a, b) { return; }
        }
        let closest = Wire::closest_point(a, b, self.pos);
        self.clear_inertia();
        self.substate = target;
        self.wire = Some(wi);
        self.auto_drop = 0.0;
        // The yaw: the nearer of wire.yaw ± π/2 to Bond's old yaw [M 0x1973f8-0x197430; type 2 flips by
        // the slope — both live wires are type 0 [D placements probe]].
        let old = self.yaw;
        let mut new_yaw = w.yaw - std::f32::consts::FRAC_PI_2;
        if wrap_pi(new_yaw - old).abs() > std::f32::consts::FRAC_PI_2 { new_yaw += std::f32::consts::PI; }
        self.yaw = new_yaw;
        self.pos = [closest[0], closest[1], closest[2]];
        if w.wire_type == 2 {
            self.mode = 6;
            self.pos[1] -= 1.17363;
            self.wire_anim = 0x0600_0095;
        } else {
            self.mode = 0xC;
            self.pos[1] -= 1.0207;
            self.wire_anim = 0x0600_05e5;
        }
        self.wire_anim_time = 1.0;
        self.wire_anim_rate = 1.0;
        self.cam_mode = 0xB;
        // The hand-reach correction [M 0x1974b8-0x197538]: delta = root(last) − root(0) of the grab
        // anim's sequence, x −= 0.082, rotated into world by the NEW yaw and subtracted. The rotation's
        // exact MMI signs are the disassembly's: out.x = l.x·C + l.z·S, out.z = l.z·C − l.x·S
        // [M disasm 0x1975dc-0x197608 — `mul.s f1,f2,f4; mul.s f3,f0,f5; add.s f1,f1,f3` stores x;
        // `mul.s f0,f0,f4; mul.s f2,f2,f5; sub.s f0,f0,f2` stores z], the ±yaw negation gated by a
        // flag bit (`bbit132 $sp,1` @0x1975b8); the magnitude ~0.46 along
        // the facing (the reach), erased by the grab anim's own root motion as it plays.
        if let Some(t) = ctx.wire_anims.and_then(|wa| wa.tracks.get(&0x0600_05e5)) {
            let e = t.at(t.frames as f32);
            let s = t.at(1.0);
            let (dx, dy, dz) = (e[0] - s[0] - 0.082, e[1] - s[1], e[2] - s[2]);
            let (sin, cos) = self.yaw.sin_cos();
            // world = R(yaw)·local (the game's row-vector convention), negated (pos −= …).
            self.pos[0] -= dx * cos + dz * sin;
            self.pos[1] -= dy;
            self.pos[2] -= -dx * sin + dz * cos;
        }
    }

    /// The wire anim state machine helper: advances the playing anim by `rate` frames, applies its root
    /// motion rotated by Bond's yaw (`AnimSeqTick`/`AnimFrameResolve`; the same convention as the
    /// drones' `root_motion`: world = l.x·(cos y, 0, −sin y) + l.z·(sin y, 0, cos y)), and reports
    /// whether the anim has reached its end (`AnimScriptIsStopped` — these scripts never loop [D]).
    /// Missing track or no anim data = the anim holds its start pose with no root motion. In the
    /// shipped game the fallback is unreachable for the seven wire scripts — the level archives
    /// carry them [D — the tracks measured from the archives, wire.rs's RootTrack doc]; the
    /// fallback exists for the no-data signature (`wire_anims: None`, the unit tests).
    fn wire_anim_advance(&mut self, ctx: &LadderCtx) -> bool {
        if self.wire_anim == 0 { return true; }
        let Some(track) = ctx.wire_anims.and_then(|wa| wa.tracks.get(&self.wire_anim)) else {
            return true;
        };
        let len = track.frames as f32;
        if self.wire_anim_time >= len { return true; }
        let prev = self.wire_anim_time.max(1.0);
        let next = (self.wire_anim_time + self.wire_anim_rate).min(len);
        let (a, b) = (track.at(prev), track.at(next));
        let (dx, dy, dz) = (b[0] - a[0], b[1] - a[1], b[2] - a[2]);
        let (sin, cos) = self.yaw.sin_cos();
        self.pos[0] += dx * cos + dz * sin;
        self.pos[1] += dy;
        self.pos[2] += -dx * sin + dz * cos;
        self.wire_anim_time = next;
        self.wire_anim_time >= len
    }

    /// Adds a wire anim (`AnimScriptAdd` + the sway the decompile applies per add): snaps Bond onto
    /// `closest` at hang height with the ±0.0542 hand sway (perpendicular to the facing: the shift is
    /// `±0.0542·(cos yaw, −sin yaw)` [M the PS2Sinf2(−yaw+π/2, −yaw) pairs; the per-add signs follow
    /// the decompile's tree]) and sets `oldhash`/time/rate.
    fn wire_anim_add(&mut self, hash: u32, rate: f32, sway_sign: f32, closest: V3, hang: f32) {
        // G-046: the sway's two sines are the game's own lanes — PS2Sinf2(−yaw + 1.5707964, −yaw) [M
        // Player_Wire 0x197658:194-204, Player_CollWire 0x197130:228] — then `pos −= lane·(−sway_sign·0.0542)`
        // (EE mul.s/sub.s; the per-add immediate ±0.0542 follows the decompile's tree).
        let ny = -self.yaw;
        let lane1 = crate::trig::sinf(crate::trig::ee_add(ny, crate::trig::HALF_PI));
        let lane2 = crate::trig::sinf(ny);
        let mul = nf_formats::ps2float::mul;
        let c = -sway_sign * 0.0542;
        self.pos[0] = crate::trig::ee_sub(closest[0], mul(lane1, c));
        self.pos[1] = closest[1] - hang;
        self.pos[2] = crate::trig::ee_sub(closest[2], mul(lane2, c));
        self.wire_anim = hash;
        self.wire_anim_time = 1.0;
        self.wire_anim_rate = rate;
    }

    /// `Player_Wire` 0x197658 (substate 6) — the hand-over-hand crossing, ported in the decompile's
    /// order. `Music_Event(0xD, 1)` every tick and the switch-channel writes are the Sim's (they cannot
    /// reach `Bond`), keyed off the substate.
    fn wire_tick(&mut self, world: &CollisionWorld, ctx: &LadderCtx, input: &Input) {
        let wi = match self.wire {
            Some(wi) if ctx.wires.get(wi).is_some() => wi,
            _ => { self.substate = SS_WALK; self.wire = None; self.physics_tick(world, ctx, input, false, false); return; }
        };
        let wire = ctx.wires[wi].clone();
        let hang = 1.0207f32;
        // 1. `Vec_Zero(BLData+0x50)` — the world velocity zeroes every tick; the anim root motion moves
        //    Bond instead [M 0x197668].
        self.vel = [0.0; 3];
        // 2. The travel segment (radius − 0.5, pitch forced 0) [M 0x1976dc-0x197730]. Bond's x/z snaps
        //    to an end when he is beyond it, then the closest point recomputes.
        let (a, b) = wire.travel_segment();
        let seg2 = dist2_sq(a, b);
        let d_a_pre = dist2_sq(a, self.pos);
        let d_b_pre = dist2_sq(b, self.pos);
        if seg2 < d_a_pre { self.pos[0] = b[0]; self.pos[2] = b[2]; }
        if seg2 < d_b_pre { self.pos[0] = a[0]; self.pos[2] = a[2]; }
        let closest = Wire::closest_point(a, b, self.pos);
        let d_a = dist2_sq(a, self.pos);
        let d_b = dist2_sq(b, self.pos);
        // 3. The auto-drop timer [M 0x197748-0x197754]: paused while the grab pose (mode 0xC) plays;
        //    100/(FRAME_RATE·30) per tick at the fixed 60 Hz = 1800 ticks to the drop.
        if self.mode != 0xC { self.auto_drop += 100.0 / (60.0 * 30.0); }
        // 4. The drop paths [M 0x197758-0x197834]: the action press (jump, action 7) or the timer past
        //    100 — `+0x942 = substate`, `ChangeSubState(0)`, `SetCamMode(0)`, the timer zeroed,
        //    `+0x951 = 0x1E` (the 30-frame regrab lockout), `AnimObj[99] = AnimObj[100]` (the Sim's
        //    re-equip carry) and the channel 0 (the Sim).
        if input.jump || self.auto_drop > 100.0 {
            self.saved_sub = SS_WIRE as u16;
            self.substate = SS_WALK;
            self.cam_mode = 0;
            self.auto_drop = 0.0;
            self.regrab = 0x1e;
            self.wire = None;
            self.wire_anim = 0;
            return;
        }
        // 5. The strafe axis (`Input_Actionf(slot, 1, 1)`) [M 0x1977c0-0x197834]: mode 7 for positive
        //    (toward B), 8 for negative (toward A), 6 idle — zeroed within 0.5² of the travelled-to end
        //    or past the far one.
        let mut strafe = input.strafe.clamp(-1.0, 1.0);
        if (strafe < 0.0 && (d_a <= 0.5 || seg2 < d_b_pre)) || (strafe > 0.0 && (d_b <= 0.5 || seg2 < d_a_pre)) {
            strafe = 0.0;
        }
        let next_mode = if strafe > 0.0 { 7u8 } else if strafe < 0.0 { 8 } else { 6 };
        // 6. The anim machine. The playback (root motion) runs first: while the current anim plays,
        //    the branches below early-return exactly like the decompile (`AnimScriptIsStopped` gates).
        let stopped = self.wire_anim_advance(ctx);
        if self.mode == 0xC && !stopped {
            // The grab pose still playing: wait (the timer is paused above) [M 0x197838-0x197860].
            return;
        }
        if self.mode == 6 && next_mode == 6 {
            if !stopped { return; }
            // The idle pair alternates [M 0x197870-0x1978d0]: after the grab pose / 5ec / b4 / 5ee the
            // next idle is 5ec, otherwise 5ed. The idle adds return early — no rate change (1.0).
            let hash = if matches!(self.wire_anim, 0x0600_05e5 | 0x0600_05ec | 0x0600_00b4 | 0x0600_05ee) {
                0x0600_05ec
            } else {
                0x0600_05ed
            };
            let sign = if hash == 0x0600_05ec { 1.0 } else { -1.0 };
            self.wire_anim_add(hash, 1.0, sign, closest, hang);
            return;
        }
        // The move/turn tree [M 0x1978e0-0x1979d0]. The decompile gates on `AnimScriptIsStopped`
        // ONLY when the mode byte is NOT 6 — an idle (mode 6) with move input falls straight into the
        // move tree (0x197838's `if (+0x94D == 6)` only wraps the idle branch), so a strafe press
        // during the idle starts the move immediately instead of waiting out the idle cycle.
        if self.mode != 6 && !stopped { return; }
        if next_mode >= 7 {
            let target = if next_mode == 7 { wire.yaw + 3.0 * std::f32::consts::FRAC_PI_2 } else { wire.yaw + std::f32::consts::FRAC_PI_2 };
            let diff = wrap_pi(target - self.yaw);
            if diff.abs() <= 0.0002 || matches!(self.wire_anim, 0x0600_05ee | 0x0600_05ea) {
                // Facing along the wire: the move pair alternates (b4 after b5/5ed, else b5).
                self.yaw = target;
                let (hash, sign) = if self.wire_anim == 0x0600_00b5 || self.wire_anim == 0x0600_05ed {
                    (0x0600_00b4, -1.0)
                } else {
                    (0x0600_00b5, 1.0)
                };
                self.wire_anim_add(hash, 1.2, sign, closest, hang);
            } else if self.wire_anim == 0x0600_00b5 || self.wire_anim == 0x0600_05ed {
                self.wire_anim_add(0x0600_05ea, 1.2, -1.0, closest, hang);
            } else {
                self.wire_anim_add(0x0600_05ee, 1.2, 1.0, closest, hang);
            }
        } else if matches!(self.mode, 7 | 8) && matches!(self.wire_anim, 0x0600_05ee | 0x0600_05ea) {
            // A turn just finished and the input is gone: the yaw fix-up lands on the opposite facing
            // and the mode byte takes 0xA ("turn") before settling to idle next tick [M 0x1979d8-0x197a30].
            let d = wrap_pi(self.yaw - (wire.yaw + std::f32::consts::FRAC_PI_2));
            let base = if d.abs() < 0.0002 { wire.yaw + std::f32::consts::PI } else { wire.yaw };
            self.yaw = base + std::f32::consts::FRAC_PI_2;
            self.mode = 0xA;
            return;
        }
        self.mode = next_mode;
    }

    /// The crawl entry, `Player_CollCreepWall` 0x196758 (the caller resolved the wall — the game's
    /// `Player_Activate` 0x1a5170:66-97 picks the nearest class-'+' object within 1000 when the
    /// third-icon context is 6). Guards [M 0x196758:73-74]: regrab `+0x951 == 0`, health `+0x894 > 0`
    /// (the port's dead Bond gets no input — the caller), and the substate check `1 < (u16)(f6 − 6)`
    /// — the wire (6) and the crawl (7) refuse, everything else passes. Entry [M 0x196758:76-128]:
    /// Bond's xz snaps to the wall's axis (the closest point of pos ± fwd(yaw+π/2)·len), the velocity
    /// slots zero, the floor ray (the game's request 0x62d, from closest.y + 5 down to − 5) puts
    /// `pos.y = hit + 1.0`, `Player_WeaponNone` (the Sim holsters — the caller watches the substate),
    /// `Player_ClearInertia`, `ChangeSubState(7)`, `+0x94D = 0` (the crawl mode byte), cam mode 0xB,
    /// the crawl anim script 0x6000476 (the port renders no body anims here) and `BLData+0x884 = wall`.
    pub fn start_creep(&mut self, wi: usize, world: &CollisionWorld, ctx: &LadderCtx) {
        let Some(w) = ctx.creepwalls.get(wi) else { return };
        if self.regrab != 0 { return; }
        if self.substate == SS_WIRE || self.substate == SS_CREEP { return; }
        let closest = w.closest_point(self.pos);
        self.vel = [0.0; 3];
        self.pos = [closest[0], closest[1], closest[2]];
        // The floor under the wall line: the game's vertical ray from closest.y + 5 [M 0x196758:94-112].
        if let Some(floor) = world.floor_below([closest[0], closest[1] + 4.5, closest[2]], 10.0) {
            self.pos[1] = floor + 1.0;
        }
        self.yaw = w.yaw + std::f32::consts::PI; // [M 0x196758:80 + 102: wall.yaw + π/2 + π/2]
        self.clear_inertia();
        self.substate = SS_CREEP;
        self.mode = 0;
        self.cam_mode = 0xB;
        self.creep = Some(wi);
    }

    /// `Player_Creep` 0x1969d0 (substate 7) [M]; the crawl motion is the port's documented shape:
    /// the game drives Bond along the wall through the crawl anim scripts' root tracks
    /// (AnimScriptAdd 0x600046d/0x6000471/0x600046f by the mode and the stopped state, the
    /// AppendBlend pair 0x600046c/e [M Player_Creep 0x1969d0:172-219]) — the port parses no Bond
    /// body-anim data (the wire tracks are the only player anim archive it reads [M wire.rs]), so
    /// the port moves him along the axis at a constant 0.02/tick (1.2 u/s) with the input. The feel
    /// difference (the game's anim-curve speed vs the constant) is an open feel check;
    /// the extension path is the drones' `root_motion` machinery.
    /// Every tick: the xz snaps to the closest axis point + fwd(yaw)·0.275 [M 0x1969d0:99-125], the
    /// yaw = wall.yaw + π; the 2D distances to the two ends pick the near end (iVar10) and the exits:
    /// within 1.0 of an end WITH the input toward it → `+0x942 = 7`, `ChangeSubState(0)`, cam mode 0,
    /// the exit yaw = crawl yaw ∓ π/2 (toward the open side of that end), regrab = 0x78 (120),
    /// `AnimObj[99] = [100]` (the Sim's weapon carry, as the ladder dismount), `+0x18 += 0.05` (a
    /// forward push out) and the channel cleared [M 0x1969d0:134-156]. The music event (0xD,1) and the
    /// channel set ride the Sim's tick (the port keeps `switch_channels` in the drones).
    fn creep_tick(&mut self, world: &CollisionWorld, ctx: &LadderCtx, input: &Input) {
        let Some(wi) = self.creep else { self.substate = SS_WALK; return };
        let Some(w) = ctx.creepwalls.get(wi) else { self.substate = SS_WALK; self.creep = None; return };
        let crawl_yaw = crate::trig::ee_add(w.yaw, crate::trig::PI); // [M 0x1969d0:118 add.s, stored to obj+0x54]
        self.yaw = crawl_yaw;
        let closest = w.closest_point(self.pos);
        // G-046: pos = closest + PS2Sinf(yaw)·0.275 / PS2Sinf(yaw + 1.5707964)·0.275 [M 0x1969d0:120-125].
        let mul = nf_formats::ps2float::mul;
        self.pos = [
            crate::trig::ee_add(closest[0], mul(crate::trig::sinf(crawl_yaw), 0.275)),
            self.pos[1],
            crate::trig::ee_add(closest[2], mul(crate::trig::sinf(crate::trig::ee_add(crawl_yaw, crate::trig::HALF_PI)), 0.275)),
        ];
        // The ends: A = pos + fwd(yaw+π/2)·len, B = pos − fwd(yaw+π/2)·len [M 0x1969d0:91-105].
        let (a, b) = w.endpoints();
        let d2 = |p: V3, q: V3| (p[0] - q[0]) * (p[0] - q[0]) + (p[2] - q[2]) * (p[2] - q[2]);
        let (d_a, d_b) = (d2(self.pos, a).sqrt(), d2(self.pos, b).sqrt());
        let fwd = input.forward.clamp(-1.0, 1.0);
        let pushing = if fwd.abs() < 0.5 { 0u8 } else if fwd > 0.0 { 2 } else { 1 };
        let at_a = d_a < 1.0;
        let at_b = d_b < 1.0;
        let (exit, nearer_a) = if at_a && pushing == 2 {
            (true, d_a < d_b)
        } else if at_b && pushing == 1 {
            (true, d_a < d_b)
        } else {
            (false, true)
        };
        if exit {
            // [M 0x1969d0:134-156] the shared exit tail; the yaw turn: nearer the A end → −π/2.
            self.saved_sub = SS_CREEP as u16;
            self.substate = SS_WALK;
            self.creep = None;
            self.cam_mode = 0;
            self.yaw = crawl_yaw + if nearer_a { -std::f32::consts::FRAC_PI_2 } else { std::f32::consts::FRAC_PI_2 };
            self.regrab = 0x78;
            self.push_fwd += 0.05;
            return;
        }
        // The crawl motion along the axis is the port's shape (the game's is the crawl anims' root
        // tracks — see the fn doc) [M 0x1969d0:172-219].
        if pushing != 0 {
            let dir = if pushing == 2 { 1.0 } else { -1.0 };
            let (ax_s, ax_c) = (w.yaw + std::f32::consts::FRAC_PI_2).sin_cos();
            self.pos[0] += ax_s * 0.02 * dir;
            self.pos[2] += ax_c * 0.02 * dir;
            self.mode = pushing;
        } else {
            self.mode = 0;
        }
        let _ = world;
    }

    /// `Player_Zipline` 0x197f50 (substate 0xF, wire type 2) [M]. Mode 9 = the arrival pose: when its
    /// anim stops → `+0x942 = substate`, `ChangeSubState(0)`, `SetCamMode(0)`, regrab = (u8)FRAME_RATE
    /// (60), `AnimObj[99] = [100]` (the Sim's carry) and the channel 0. Otherwise: the ride anim
    /// 0x06000096 re-added whenever stopped, the FULL-radius pitch segment, Bond's y lifted 1.17363 for
    /// the lower-end test — within 1.0 of it → mode 9 + anim 0x06000094 — then pos = the closest point
    /// at `closest.y − 1.17363` and the fall-speed accumulator `BLData+0x18 += FRAME_RATE_MUL·0.15`
    /// (the port integrates it as a downward velocity: the game's `Player_Move` writes BLData+0x18
    /// as the local velocity's FORWARD lane [M 0x191dd0:167] and the integration applies the +0x10
    /// quad along the matrix rows [M 0x193600:284-295]; the zipline accumulates it at 0.15/tick
    /// [M 0x197f50:121]). The ride is unreachable in the shipped game — both live wires are type 0
    /// [D placements probe, grab_wire's census] — so the port's downward integration is
    /// dead-in-shipped-play bookkeeping; the clamp projects the fall onto the line, so Bond slides
    /// down a sloped zipline and accelerates.
    fn zipline_tick(&mut self, world: &CollisionWorld, ctx: &LadderCtx, _input: &Input) {
        let wi = match self.wire {
            Some(wi) if ctx.wires.get(wi).is_some() => wi,
            _ => { self.substate = SS_WALK; self.wire = None; self.physics_tick(world, ctx, _input, false, false); return; }
        };
        let wire = ctx.wires[wi].clone();
        if self.mode == 9 {
            if self.wire_anim_advance(ctx) {
                self.saved_sub = SS_ZIPLINE as u16;
                self.substate = SS_WALK;
                self.cam_mode = 0;
                self.regrab = 60;
                self.wire = None;
                self.wire_anim = 0;
            }
            return;
        }
        if self.wire_anim_advance(ctx) && self.wire_anim != 0x0600_0094 {
            self.wire_anim_add(0x0600_0096, 1.0, 0.0, self.pos, 0.0);
        }
        let (a, b) = wire.zipline_segment();
        // The lower-end test runs on the LIFTED position (the decompile lifts Bond's y onto the wire
        // first [M 0x197ff8: `pos.y += 1.17363`], checks `Dist3D(pos, lower) < 1.0`, and only then
        // re-projects and hangs him back 1.17363 below the line).
        let lifted = [self.pos[0], self.pos[1] + 1.17363, self.pos[2]];
        let low = if a[1] < b[1] { a } else { b };
        if dist3_sq(lifted, low) < 1.0 {
            self.mode = 9;
            self.wire_anim_add(0x0600_0094, 1.0, 0.0, self.pos, 0.0);
            return;
        }
        // The fall-speed accumulator `BLData+0x18` [M 0x19804c-0x198054: += FRAME_RATE_MUL·0.15 per
        // tick. The game's `Player_Move` writes BLData+0x18 as the forward lane of the local velocity
        // [M 0x191dd0:167] and the integration applies the +0x10 quad along the matrix rows
        // [M 0x193600:284-295]. The port applies it downward for the zipline ride (unreachable in
        // the shipped game — both live wires are type 0 [D placements probe]). The clamp below
        // projects the ride onto the line, so Bond slides down a sloped zipline and accelerates.
        self.fall_speed += 0.15;
        self.pos[1] -= self.fall_speed / 60.0;
        let lifted = [self.pos[0], self.pos[1] + 1.17363, self.pos[2]];
        let closest = Wire::closest_point(a, b, lifted);
        self.pos = [closest[0], closest[1] - 1.17363, closest[2]];
    }

    /// `Player_InWater` 0x195e50 [M]: with a water cel and `pos.y − 0.3 < surface`, enter the swim
    /// substate — `ChangeSubState(3)`, `+0x94C = 0`, `+0x130 = 2`, `+0x14 = 0`, `regrab = (u8)(DIV·5) = 5`,
    /// `obj+0x50/+0x58 = 0`, the per-tick deltas cleared. Returns whether Bond is swimming (substate 3).
    fn enter_water(&mut self, ctx: &LadderCtx) -> bool {
        let Some(w) = ctx.water else { return false };
        if self.pos[1] - 0.3 >= w { return false; }
        if self.substate != SS_SWIM {
            self.substate = SS_SWIM;
            self.jump = 0;        // +0x94C = 0 [M]
            self.local[1] = 0.0;  // +0x14 = 0 [M]
            self.regrab = 5;      // (u8)(FRAME_RATE_DIV·5) [M]
            self.vel[0] = 0.0;    // obj+0x50 = 0 [M]
            self.vel[2] = 0.0;    // obj+0x58 = 0 [M]
            self.local[0] = 0.0;  // Vec_Zero(BL+0x20): the strafe smoother clears too [M 0x195e50 disasm]
            self.mode = 0;
            self.ladder = None;
        }
        true
    }

    /// `Player_MonitorAir` 0x191a70 [M], run after every substate: the head — the game reads the
    /// anim head bone's world transform (`AnimGetBoneWorldTrans(param_2, bone −5, …)` [M
    /// 0x191a70:50-53]; the port uses the fixed standing eye — it runs no Bond skeleton — the
    /// live check pins the drown threshold) — against `cel+0x90`: below the surface the air `+0x8C0`
    /// drains at 5/s from 100 and the underwater fade `+0x8C4` rises; above it (any cel — the −500
    /// default is far below every head) the air refills at 20/s. At `air < 1.0` every 30 ticks (`GameState % (FRAME_RATE·0.5) == 0`):
    /// `Player_HandlePain(0x40000000 = 2.0, type 7)` [M 0x191a70:136-143, disasm 0x191d9c] — the drown
    /// damage the caller applies. The breath loop sounds (0x60c surfaced / the underwater loop on
    /// `+0x89C`) and the HUD air pane are not ported this pass.
    fn monitor_air(&mut self, ctx: &LadderCtx) {
        // The game's refill branch runs for ANY cel (including the −500 default) when the head is above
        // `cel+0x90` [M 0x191a70:98-111] — air refills on dry land too.
        let surface = ctx.water.unwrap_or(-500.0);
        let head = self.head()[1];
        self.underwater = head < surface && surface > -400.0;
        self.water_fade = if self.underwater { (self.water_fade + (1.0 - self.water_fade) / 60.0).min(1.0) }
            else { (self.water_fade - self.water_fade / 120.0).max(0.0) };
        if self.underwater && !(SS_ZEROG_8..=SS_ZEROG_9).contains(&self.substate) {
            self.air -= 5.0 / 60.0;
            if self.air < 1.0 {
                self.air_tick += 1;
                if self.air_tick >= 30 { self.air_tick = 0; self.drown_damage = Some(2.0); }
            }
        } else {
            // The zero-g substates never drain: `Player_MonitorAir`'s drain enable is false for
            // 7 < substate < 10, so the suit's air refills even under the surface [M 0x191a70:57,
            // :98 the refill].
            self.air = (self.air + 20.0 / 60.0).min(100.0);
            self.air_tick = 0;
        }
        self.air = self.air.clamp(0.0, 100.0);
    }

    /// One 60 Hz physics tick (the port's `Player_Move` + `Player_Collision` walk body). `swimming`
    /// selects the swim-substate differences: no `Player_HandleJump` (case 3 doesn't call it [M]) and no
    /// gravity (`Player_Collision`'s water path skips `WldGravity` [M]).
    fn physics_tick(&mut self, world: &CollisionWorld, ctx: &LadderCtx, input: &Input, swimming: bool, zero_g: bool) {
        let prev = self.pos;
        // The clip's move vector = pos − pre_move_pos, the WHOLE positional delta this tick
        // applies — the walk's matrix·(vs, vf) add, the swim dir add, zero-g's thruster add AND
        // the gravity integration — computed AT THE COLLISION BLOCK below (after those adds),
        // from the pre-move capture here. (An earlier draft captured it after the walk's x/z add
        // — move_v then never carried the walk's horizontal move, the clip saw (0, gravity, 0)
        // every walked tick and never fired on any wall press: measured in
        // level-0a-walk-clip-flip, the west-hall press walked through the wall.)
        let pre_move_pos = prev;
        self.fall_damage = None;
        // The spawn arrival settle (the `settle_hold` field doc): the boot holds the placed root
        // while the fall velocity accumulates; the release tick integrates the held velocity ONCE
        // in the game's op order and the push/band tail below re-grounds him. NARROWED
        // (level-0a-office-hatch-7): the hold gates ONLY the pieces the observable pins — the
        // gravity add stays unconditional, the vel→pos integration is skipped (the root holds),
        // and the grounded tail's velocity zeroing is held off — everything else RUNS: the crouch
        // read, the jump/look/move reads and the capsule PUSH resolution (the game's player never
        // accumulates wall overlap: the broad whole-tick skip ejected the parked 09 body-armour
        // tester into the pickup and left the 01 seam read inside the wall's contact — both
        // master-green tests failed on the held ticks' unresolved geometry). The handler's ground
        // phase still cannot ground him in the game's hold or its tail would zero the accumulating
        // velocity — the port keeps the zeroing gated, not the tick skipped.
        let (settle_held, settle_release) = if self.settle_hold > 0 {
            self.vel[1] += self.world_gravity[1] / 60.0; // [M 0x194df0:107]
            self.settle_hold -= 1;
            (self.settle_hold > 0, self.settle_hold == 0)
        } else { (false, false) };
        // `Player_SSWalk` / `Player_SSCrouch`: the lockout runs down (*deviation*: the transition animation that may
        // end it early is not played).
        self.lockout = self.lockout.saturating_sub(1);
        // Case 3 calls `Player_Move(1.0, …)` [M 0x193600:181-182] — the swim multiplier is ALWAYS 1.0
        // (the 0.75 is `Player_SSCrouch`'s own call argument 0x1924c0:58, not a crouched flag read).
        let k = if swimming { 1.0 } else if self.crouched { 0.75 } else { 1.0 };
        // `Player_Move`: aim mode zeroes forward (and the pad's turn action, which the caller leaves out of `turn`:
        // with a mouse the view turns past the aim box's edge, D-002).
        let (fwd, turn) = if input.aiming { (0.0, input.turn) } else { (input.forward.clamp(-1.0, 1.0), input.turn) };
        let strafe = input.strafe.clamp(-1.0, 1.0);
        let target_f = if fwd > 0.0 { 0.1 * fwd * fwd + 0.005 } else if fwd < 0.0 { -0.075 * fwd * fwd - 0.005 } else { 0.0 };
        let target_s = 0.075 * strafe;
        let axis = |last: f32, target: f32| -> f32 {
            let mut v = last + (target - last) * 0.2;
            if (target - last) * (target - v) < 0.0 { v = target; }
            if target.abs() <= 0.0002 && v.abs() < 0.01 { v = 0.0; }
            v.clamp(-0.1, 0.1)
        };
        let (vs, vf0) = if zero_g {
            // Zero-g eases its own smoothers below (the 0.04 factor, the ±0.05 cap [M 0x192ab0:
            // 92-159]) — the walk's `Player_Move` ease (0.2, the ±0.1 cap, the dead-zone) must
            // not touch the shared BL+0x20/+0x28 slots this tick.
            (0.0, 0.0)
        } else {
            (axis(self.local[0], target_s) * k, axis(self.local[1], target_f) * k)
        };
        // `Player_HandleJump`: crouch (toggle) and jump. Case 3 (swim) doesn't call it [M].
        let headroom = |w: &CollisionWorld| w.line_of_sight(self.pos, [prev[0], prev[1] + 0.95, prev[2]], crate::world::LOS_PLAYER_HEADROOM);
        if swimming {
        } else if zero_g {
            // No `Player_HandleJump` here: the jump/crouch actions ARE the vertical thruster in
            // zero-g (the held analog pair) [M 0x192ab0:136-138]; the crouch toggle, the jump grab
            // and the headroom checks are walk-only [M 0x193600 dispatch — HandleJump is
            // SSWalk's; case 8/9 reads none of it].
        } else if !self.crouched && self.jump == 0 && input.crouch && self.lockout == 0 {
            self.crouched = true;
            self.cc = CROUCH_ROOT_HEIGHT;
            self.lockout = 60;
        } else if self.crouched && self.lockout == 0 && !self.head_bump && headroom(world)
            // options-census: the stand condition follows the Crouch option [M Player_SSCrouch
            // 0x1924c0:84-105] — Toggle (byte +0x04 != 0) stands on a press edge of action 8
            // (crouch) or 7 (jump); Hold stands when the crouch action is NOT held.
            && (if !input.crouch_hold { input.crouch || input.jump } else { !input.crouch }) {
            self.crouched = false;
            self.cc = ROOT_HEIGHT;
            self.lockout = 60;
        } else if !self.crouched && self.jump == 0 && input.jump && self.grounded_history != 0 && !self.head_bump && headroom(world) {
            // `Player_HandleJump` 0x1916b0:118-147 — the jump grab: with the third-icon context 3 (the
            // wire icon, `BLData+0x95F`), the press scans the level's wires for the NEAREST class-0x2C
            // within 1000 units (no capsule test) and grabs it instead of jumping [M — the game's scan
            // is `control_first_object` over class ',' with `Vec_Dist3D`]. A refused grab (the regrab
            // lockout inside `grab_wire`) still eats the jump — the game returns from HandleJump
            // regardless [M 0x1916b0:146-149]. No context (0xFF) or other types → the plain jump.
            if self.icon_ctx == 3 {
                if let Some(wi) = ctx.wires.iter().enumerate()
                    .filter(|(_, w)| dist3_sq(w.centre, self.pos) < 1000.0 * 1000.0)
                    .min_by(|(_, a), (_, b)| dist3_sq(a.centre, self.pos).partial_cmp(&dist3_sq(b.centre, self.pos)).unwrap_or(std::cmp::Ordering::Equal))
                    .map(|(i, _)| i) {
                    self.grab_wire(wi, ctx);
                    return;
                }
            }
            self.vel = [0.0, -GRAVITY * 0.4, 0.0];
            self.jump = 1;
            self.land_delay = 4;
            self.grounded = false;
            // [M Player_HandleJump 0x1916b0:184]: the launch plays 0xA3 at the player.
            self.sfx_out.push((0xa3, self.pos));
        }
        // The matrix turns first, then the position moves along the new axes. The climb's one-tick
        // local-forward push (Player_Climb's ±0x18 write) rides along once.
        let vf = vf0 + self.push_fwd;
        self.push_fwd = 0.0;
        if zero_g {
            // ===== Player_ZeroG 0x192ab0 (the spacewalk body) =====
            // First tick of the spell builds the orientation from the walk yaw/pitch:
            // ChangeSubState 9 touches no rotation [M 0x190630:59-63]; the walk matrix is
            // RotMatrix(−pitch·π/2, yaw, 0) [M 0x1e2368 — Bond::look's rows].
            if !self.zero_g_active {
                self.zero_g_rot =
                    rot_matrix(-self.pitch * std::f32::consts::FRAC_PI_2, self.yaw, 0.0);
                self.zero_g_active = true;
                self.stabiliser = 0;
            }
            // :71 `Vec_Zero(BLData+0x50)` — the world velocity zeroes EVERY tick: no gravity, no
            // momentum in `vel`; the whole move is the local thruster quad through the object
            // matrix (the caller's `pos += M·(+0x10 quad)` [M 0x193600:284-295]). :72-74 zero
            // BL+0x957 / +0x130 (the weapon-raise re-trigger) / +0x8a8 — the scalar view pitch is
            // dead here (`Bond::look` returns the matrix's forward row instead).
            self.vel = [0.0; 3];
            // The look rates [M :76-90]: the pitch action → the x angle, the turn action → the y
            // angle, roll 0 — `RotMatrix` then `Concat(obj, R)` ⇒ m := m·R [M 0x1e4a90:13-49],
            // the BODY frame (a pitched yaw drifts the up — what the auto-stabiliser repairs).
            // The port's `turn`/`look` are already per-tick deltas (mouse-direct, pad-ramped,
            // D-002); the game's own pad constant is 1.2566371·REC_FRAME_RATE rad/tick, `look`
            // riding in π/2 units.
            let r = rot_matrix(-input.look * std::f32::consts::FRAC_PI_2, turn, 0.0);
            self.zero_g_rot = mat3_mul(&self.zero_g_rot, &r);
            // The thruster smoothers [M :92-159]: targets strafe = action 1·0.1, forward =
            // action 2·0.1, vertical = (action 7 − action 8)·0.1; each eases
            // `v += (target − v)·FRAME_RATE_MUL·0.04` (FRAME_RATE_MUL = 1.0 at 60 Hz [M .sdata
            // 0x30d0d8]) with an overshoot snap and a ±FRAME_RATE_MUL·0.05 clamp — and NO
            // Player_Move dead-zone (the walk's `target≈0 → v=0` snap is 0x191dd0's; ZeroG has
            // none). The smoothers are BL+0x20/+0x28 — walk's own `local` slots — and +0x24, and
            // nothing clears them on the way in or out [M ChangeSubState 0x190630]: releasing the
            // thrust decays at 0.96^n — the drift inertia.
            let ease = |last: f32, target: f32| -> f32 {
                let mut v = last + (target - last) * 0.04;
                if (target - last) * (target - v) < 0.0 { v = target; }
                v.clamp(-0.05, 0.05)
            };
            self.local[0] = ease(self.local[0], input.strafe * 0.1);
            self.zero_g_lift = ease(self.zero_g_lift, input.lift * 0.1);
            self.local[1] = ease(self.local[1], input.forward * 0.1);
            // The auto-stabilisers [M :163-199]: the action-0xe read ("Press Action to activate
            // spacesuit auto-stabilizers") arms BL+0xd0 = FRAME_RATE·1.5 = 90; while it counts
            // down, the orientation slerps toward `Mat_Align2Up(m, (0,1,0), fwd − (0,1,0)·
            // (fwd·(0,1,0)))` — world up, same heading (the ±0.995 degenerate arms swap in
            // ∓up [M :177-184]) — at Quat_Slerp_Acc t = REC_FRAME_RATE·3.0 = 0.05 per tick [M
            // :196], then `Mat_Normalize` [M :199].
            if input.stabilise { self.stabiliser = 90; }
            if self.stabiliser != 0 {
                self.stabiliser -= 1;
                let fwd = self.zero_g_rot[2];
                let d = fwd[1];
                let dir = if d > 0.995 {
                    [-self.zero_g_rot[1][0], -self.zero_g_rot[1][1], -self.zero_g_rot[1][2]]
                } else if d < -0.995 {
                    self.zero_g_rot[1]
                } else {
                    [fwd[0], fwd[1] - d, fwd[2]]
                };
                let target = mat_align_up(dir);
                let q = crate::anim::slerp(
                    quat_from_matrix(&self.zero_g_rot),
                    quat_from_matrix(&target),
                    0.05,
                );
                self.zero_g_rot = mat_normalize(quat_to_matrix(&q));
            }
            // The move [M 0x193600:284-295]: pos += local.x·row0 + local.y·row1 + local.z·row2 —
            // the +0x10 quad's lanes (x strafe, y the vertical thruster, z forward) against the
            // matrix's right/up/forward rows. Pitched forward + forward thrust FLIES along the
            // look.
            let v = [self.local[0], self.zero_g_lift, self.local[1]];
            for k in 0..3 {
                self.pos[k] += v[0] * self.zero_g_rot[0][k]
                    + v[1] * self.zero_g_rot[1][k]
                    + v[2] * self.zero_g_rot[2][k];
            }
            // The port keeps obj+0x54's heading in `yaw` for the readers outside the substate
            // (sphere_centre, the death pick) — a port-read mirror: the game leaves obj+0x54 stale
            // in the substate (Player_ZeroG 0x192ab0 stores no +0x54 [M full-file scan]) and only
            // the matrix is authoritative in zero-g.
            self.yaw = self.zero_g_rot[2][0].atan2(self.zero_g_rot[2][2]);
        } else {
        self.yaw += turn;
        self.pitch = (self.pitch + input.look).clamp(-1.0, 1.0);
        let (s, c) = (crate::trig::sinf(self.yaw), crate::trig::cosf(self.yaw)); // G-046: View_RotTransMatrix 0x1e5168:33-36 / RotMatrixX 0x1e46a0 PS2Sinf2 lanes [M]
        if swimming {
            // `Player_Move` 0x191dd0:149-165 [M]: in swim the forward speed rides the LOOK direction
            // (`RotMatrixX(obj+0x50 − BL+0x8a8·π/2)` + `Mat_GetDir`, obj+0x50 zeroed on entry — the
            // view pitch IS BL+0x8a8), so looking down + forward DIVES and looking up + forward
            // SURFACES; strafe stays on the horizontal local X (the game overwrites the X lane with
            // the strafe speed after the dir add). `Bond::look` is the same convention (pitch +1 = up
            // = the game's dir.y = sin(0x8a8·π/2)). No crouch/jump input exists in swim [M case 3 of
            // 0x193600 reads none; Player_HandleJump 0x1916b0 is only called from SSWalk].
            let ld = self.look();
            self.pos[0] += c * vs + ld[0] * vf;
            self.pos[2] += -s * vs + ld[2] * vf;
            // The world-Y component rides as this tick's vertical velocity; the swim tail clamps it
            // against `surface − 0.3` before integrating (case 3 tail [M 0x193600:201-220]).
            self.vel[1] = ld[1] * vf * 60.0;
        } else {
            self.pos[0] += c * vs + s * vf;
            self.pos[2] += -s * vs + c * vf;
        }
self.local = [vs / k, vf / k];
        }
        // `Player_Collision`: gravity unless standing on a gentle floor (jumping keeps the velocity).
        // The water path skips `WldGravity` entirely [M 0x194df0 default case]. The vertical velocity
        // integration is also the swim tail's job alone (it clamps first) — skip it here when swimming.
        // The settle's release tick skips the gate entirely: the held velocity integrates as it stands.
        if settle_release {
        } else if settle_held {
            // The settle added this tick's gravity above; the grounded arm's zeroing stays held
            // off — the accumulating velocity IS the observable ([L ee_start root 1.0891]).
        } else if swimming {
        } else if zero_g {
            // `vel` (BL+0x50) is zeroed every zero-g tick [M 0x192ab0:71] — WldGravity has no
            // reader left and nothing integrates; the tick's whole move was the matrix·thruster
            // add above.
        } else if !self.grounded || self.floor_ny < 0.5 {
            // The fall rides the level's `WldGravity` [M Player_Collision 0x194df0:107/161,
            // `vel += WldGravity · REC_FRAME_RATE`] — −9.8/60 per tick on the default levels,
            // 0 on the p3==2 (spacewalk) starts (fix-world-gravity).
            self.vel[1] += self.world_gravity[1] / 60.0;
        } else if self.jump != 1 {
            self.vel = [0.0; 3];
        }
        self.vel[1] = self.vel[1].clamp(-45.0, 45.0);
        if settle_release {
            // The ONE integration of the held velocity, in the game's op order (the VU mul then the
            // add [M 0x194df0:189-197]) — the shape the ee_start root reproduces bit-exactly. The
            // x/z velocities are zero through the hold.
            self.pos[1] += nf_formats::ps2float::mul(self.vel[1], crate::ssys::REC_FRAME_RATE);
            self.vel = [0.0; 3];
        } else if settle_held {
            // The root holds: no vel→pos integration on the held ticks ([L ee_start_0700000a —
            // the placed 1.1 root reads 1.1 through the boot's hold]).
        } else {
            for k in 0..3 { if !(swimming && k == 1) { self.pos[k] += self.vel[k] / 60.0; } }
        }
        // `Player_CollisionHandler`: the floor ray first, then the capsule's push. Both read the
        // one BLData+0x740 HITTEST, whose start cel is the actor's own obj+0x20 [M
        // `Player_Collision` 0x194df0's first store `*(BLData+0x7c0) = *(obj+0x20)`;
        // `Collide_Pick` 0x1eb320 tests each `Collide_StraddleCels` cel's own +0x18 list] — the
        // per-cel attribution (fix-08-collision): walls hanging on other rooms' cels never block.
        // `self.cel` is the Sim's per-tick relink (obj+0x20); `None` falls back to the flat world.
        let (a, b) = self.capsule();
        // The game's push-tail q-span guard rides the SWIM path only: it is [M]-evidenced there
        // (Intersect_CylGeom 0x109da0's `dot(q − A, B − A) > 0` gate + the 02 fin wedge the PS2
        // rounds [L]); the port's walk/NPC paths still depend on the unguarded through-push
        // (escort_follow_07000001_live regressed with it on) — a known port divergence.
        // THE WALK'S COLLISION RESPONSE — the game's move-clip model (the walk-model flip,
        // level-0a-walk-clip-flip): the tick's move CLIPPED at the capsule's contacts (the r-skin
        // radial retraction — see `world::capsule_tri_clip`), the decoded push tail dead — its
        // reference HITTEST+0x50 is the lifetime DeltaP accumulator [M Intersect_CylGeom
        // 0x109da0:447's guard; M Collide_Intersect 0x1ebaa8:353-373; L ee_start_* boots (0,0,0),
        // ee_snapshot drifts ~15 m], so no walk contact ever passes the guard and no push lands.
        // The pick_ref feed below takes the clip's retraction (the [L] accumulator match).
        let move_v = [self.pos[0] - pre_move_pos[0], self.pos[1] - pre_move_pos[1], self.pos[2] - pre_move_pos[2]];
        let (mut push, mut walkable) = world.capsule_clip_scoped(self.cel, a, b, RADIUS, swimming, move_v);
        // level-0a-office-hatch-10 EXPERIMENT (`NF_WALK_GHOST=1`): the game's walk push tail is
        // dead [M Intersect_CylGeom 0x109da0:447 — the guard is |q − HITTEST+0x50| < radius with
        // +0x50 the lifetime DeltaP accumulator [M Collide_Intersect 0x1ebaa8:353-373]; [L
        // ee_snapshot] its live value drifts ~15 m from the owner, and [L ee_start_*] it boots
        // (0,0,0) — so no walk contact passes the guard and no push lands]. The port keeps the
        // CONTACT here (walkable feeds grounded) but drops the DISPLACEMENT: Bond walks the game's
        // lines on the feet band alone [M Player_FeetOnPoint 0x194188:106]. Default OFF —
        // measured against the full walkreal battery before it lands.
        if walk_ghost() { push = [0.0; 3]; }
        // The floor ray: from = obj+0x30, to = capsuleB + up·(−(cc+0.6)) [M Player_FeetOnPoint
        // 0x194188:89-98 loads HITTEST+0x30 = BLData+0x770 — the B arm Player_Collision 0x194df0
        // stores at :405-408 — adds the matrix up scaled −(cc+0.6) (:92, one negation of the f32
        // sum), and passes obj+0x30 as the from]. Upright the up is (0,1,0), so the multiplies are
        // exact and the endpoint is B.y − (cc+0.6); the port's old `pos − (cc+0.6)` skipped the B
        // hop and shortened the crouch ray to 1.15 where the game's is exactly −1.25 (probe14_ground
        // d_y 0xbfa00000).
        let ray = self.cc + 0.6;
        let to = [b[0], b[1] + -ray, b[2]];
        let mut floor = world.segment_hit_scoped(self.cel, self.pos, to);
        // The game's pass never lands over a void: `build_FindCel` 0x1c8cb0 relinks obj+0x20 by
        // tie-break rays against each candidate's own chain geometry, so the actor's cel always
        // carries the geometry under him. The port's buckets are coarser than the game's cels (a
        // port-structure fact, covered by the redo below):
        // over 07000002's ladder head the scoped pass sees no floor at all (the garden-floor rim
        // sits in a bucket the straddle walk doesn't keep) and Bond fell down the shaft he had
        // just climbed (fix_ladder_live); on 07000009 the car-park stairwell steps hung outside
        // the straddle set too (walkreal_07000009_live) — a wall push in the set does not put a
        // floor under Bond, so the redo fires on the missing floor alone. On the ground the
        // scoped pass always finds the room's own floor, so the redo never fires there: across
        // the fix-08-collision slab band the floor stays in the room's bucket while the exterior
        // wall stays out of it.
        let mut pick = world.capsule_pick_contact_scoped(self.cel, a, b, RADIUS, self.pick_ref[1], self.pick_ref);
        if floor.is_none() {
            let (fp, fw) = world.capsule_clip(a, b, RADIUS, swimming, move_v);
            push = if walk_ghost() { [0.0; 3] } else { fp };
            walkable = fw;
            pick = world.capsule_pick_contact(a, b, RADIUS, self.pick_ref[1], self.pick_ref);
            floor = world.segment_hit(self.pos, to);
        }
        let _ = pick;
        // The game's pick-contact bit block [M Intersect_CylGeom 0x109da0 disasm 0x10a5e0-0x10a734]:
        // dot1 = dot(face normal, axis) > 0.7 (0x10a650-0x10a674, up-facing faces only), dot2 =
        // dot(HITTEST+0x50 − (B + push), axis) < 0 (0x10a66c-0x10a6a0), dot3 = dot(normalize(R),
        // axis) < 0.7 (0x10a6a8-0x10a734). The port's dot2 MODEL — HITTEST+0x50 as a lifetime push
        // accumulator, so the arm reduces to `pick_ref_y < b_pre_y` — that reading (the 0c3
        // attempt) rests on the bit word's unpinned per-tick clear; the pick result is MEASURED here but
        // does NOT gate `grounded` (hatch-crouch-0c4): the 0c hatch set, the real walked route and
        // the 0700004a descent are all green on master's `walkable || band` rule alone. Feeding
        // the accumulator stays [M Collide_Intersect 0x1ebaa8:353-373] for when the arm returns.
        // The contact-distance band [M Player_FeetOnPoint 0x194188:106 `*(float *)(hit + 0xc) - 0.1
        // <= cc`] reads HITDATA+0xC, which Intersect_RayGeom 0x10a8c8 fills with the ABSOLUTE
        // distance |from − hit| — hit = from + t·dir (:176-179 the dir·t product and the from add;
        // :252-256 the store to HITDATA+0x40), dir = to − from recomputed f32 [M Collide_Intersect
        // 0x1ebaa8's 0x201 branch], and the sqrt-of-squares store :312-325 — NOT t·raylen, which is
        // what the port multiplied here. For the upright ray the x/z deltas are exactly 0 (the
        // endpoint's x/z equal pos), so the VU's pairwise-add collapse leaves the single y term.
        // With the game's own B arm and ray the crouched rest grounds through the push dither with
        // no epsilon: the rest contact (0.09999999403953552 + 0.55f32 below the root) lands 1 ulp
        // under the band's f32 edge (0x3f266667, where `hit − 0.1f32` rounds to exactly cc), while
        // the port's old 0.45f32 arm sat exactly ON the edge and the t·ray multiply tipped it over
        // (probe14_ground.rs: the rest y dithered ±3 ulps and grounded flickered) [L: the crouched
        // Bond crawls out of the 07000014 vent on PCSX2].
        let band = floor.as_ref().map_or(false, |(t, _)| {
            let d_y = to[1] - self.pos[1];
            let hit_y = self.pos[1] + t * d_y;
            let dy = self.pos[1] - hit_y;
            (dy * dy).sqrt() - 0.1 <= self.cc
        });
        // The pick arm is NOT gated into grounded — the pick result is measured only (see the pick
        // bit block above).
        self.grounded = walkable || band;
        // `BLData+0x110` — the port's floor_ny — is the band's OTHER output, and it is band-GATED:
        // FeetOnPoint writes `*param_3 = 1.0f` first [M 0x194188:85] and overwrites it with the
        // band-passed hit's +0x14 only inside the band [M :106-109]; the gravity gate reads that
        // value (`flag8 == 0 || BL+0x110 < 0.5` → gravity [M Player_Collision 0x194df0:104]). The
        // port's old floor_ny took the hit normal REGARDLESS of the band, so the B-hop ray's far
        // hits — geometry past the band that master's `pos − (cc+0.6)` ray never reached — fed
        // steep normals into the gravity gate on ticks Bond was otherwise grounded: on the
        // 07000005 upper ledge he rested walkable-grounded over a ny 0.45 slope ~1.85 below the
        // root, floor_ny went 0.45, the < 0.5 gravity branch fired every tick and he slid off the
        // ledge (interact_07000005_live::the_ledge_plate_plays_the_sound_trigger; master's short
        // ray missed that geometry — floor=None — and floor_ny stayed 1.0). In the game the band
        // fails there, so BL+0x110 stays 1.0 and the gate never opens [M 0x194df0:104].
        self.floor_ny = match floor.as_ref() {
            Some((_, n)) if band => n[1],
            _ => 1.0,
        };
        self.head_bump = push[1] < -1e-4;
        // The DeltaP add-back [M Collide_Intersect 0x1ebaa8:353-373]: every applied push
        // accumulates into HITTEST+0x50 (+0x10 feeds the handler's obj+0x30 add at
        // 0x1949a8:136-140 — the port's `pos += push` — and +0x50 feeds the pick bit's dot2).
        // Under the walk clip the applied displacement is the CLIP's retraction (the tick's move
        // clipped at the contact — capsule_clip_scoped's correction), so the accumulator feeds
        // from that same correction: the [L 2026-10-26 fresh boot] west-hall wall press reads
        // +0.1/contact-tick into HITTEST+0x50 (0.27723 -> 3.27709 over 30 contact ticks, y/z
        // unchanged) — exactly the per-tick clipped-into-the-wall move the clip model retracts —
        // and the accumulator holds EXACTLY constant on the push-free hall walk (frames 20-60+),
        // where the clip applies nothing. the_pick_ref_accumulator_tracks_the_game_ref pins both.
        for k in 0..3 { self.pick_ref[k] += push[k]; }
        // The actor-actor capsule push [M Collide_Jointy 0x1ec180:229-281 — the player's kind-0x800
        // query resolving against a live bot's own kind-0x800 capsule hittest]: the closest points
        // between the two spines by two rounds of alternating projection (:234-239), the overlap
        // = (r_mine + r_theirs) − dist (:251-252), and the push = −(their closest − my closest)·
        // overlap·fVar12 applied to my capsule arms and the +0x10 accumulator — the accumulator
        // `Player_CollisionHandler` 0x1949a8:136-140 adds to obj+0x30 (the port's `push` →
        // `pos += push` below). SP mode: `MPSettings+0x384 == 0` [M C_SBNFMAP_Handler 0x20b8d8:26
        // zeroes it; the MP menus set it 0x20ec00:149] makes fVar12 = 1.0 (the FULL overlap on
        // the query's owner) and returns 0 for a class-3 other (:231-233) — the PLAYER (class 3
        // [M Player_Init 0x1910f8:76]) is pushed off the BOT (class 2 [M NDrone2_CreateObj
        // 0x14d7c8:14]) and the bot never pushes itself. The class-3-vs-3 and class-2-vs-2 gates
        // (:213-227) don't fire on a mixed pair. The bot's capsule is ON only NEAR the player
        // [M DroneFunc_NearPlayerCollision 0x14b190 — `2.0 <= dist` fails the DoCollision gate
        // 0x149ec8]: the Sim passes only those (see `LadderCtx::bot_capsules`). Dead-centre the
        // between-spines delta is vertical — Bond rides tangent ON the carrier and slides off as
        // it walks (the delta tilts); beside its spine the push is lateral. The Jointy push does
        // NOT ride the DeltaP accumulator (Jointy writes +0x10 only [M :253-266]), so this sits
        // after the add-back and `pick_ref` keeps the static pass's value.
        // [S] unverified — the PORT CHOICE gate: the push rides only while Bond is AIRBORNE
        // (the drop-onto case the task pins). The game's own per-bot ON/OFF gates (+0x4f8 bit
        // 0x20/0x200, +0x570 bit 2 — NDrone2_DoCollision 0x149ec8) are not modelled per bot
        // yet, and master's green walks path Bond THROUGH ally tangent zones (the 01 armour
        // walk, the 0a west doorway): the grounded-side push against allies is a live-check
        // item, not shipped here. The sleeper drop (grounded false through the whole contact)
        // keeps its [M] push either way.
        if !ctx.bot_capsules.is_empty() && !self.grounded {
            let (ja, jb) = self.capsule();
            if std::env::var("NF_PUSH_DEBUG").is_ok() {
                eprintln!("[pushdbg] n={} ja={:?} jb={:?} bots={:?}", ctx.bot_capsules.len(), ja, jb, ctx.bot_capsules);
            }
            for &(ta, tb, tr) in ctx.bot_capsules {
                let mut p = closest_on_seg(ta, tb, ja);
                let mut q = closest_on_seg(ja, jb, p);
                p = closest_on_seg(ta, tb, q);
                let d = [p[0] - q[0], p[1] - q[1], p[2] - q[2]];
                let dist_sq = d[0] * d[0] + d[1] * d[1] + d[2] * d[2];
                let rr = RADIUS + tr;
                if dist_sq >= rr * rr { continue; }
                let dist = dist_sq.sqrt().max(1e-6);
                let overlap = (rr - dist) / dist; // fVar12 = 1.0 in SP [M :229-233]
                push[0] -= d[0] * overlap;
                push[1] -= d[1] * overlap;
                push[2] -= d[2] * overlap;
            }
        }
        // The pre-push-out snapshot: the game's ladder-grab pick sees the segment built HERE (the
        // hittest from the integrated obj+0x30 [M 0x194df0:361]; the +0x750 push lands in the
        // handler 0x1949a8:136-140, after the refine) — see the `pre_collide` field doc.
        self.pre_collide = self.pos;
        for k in 0..3 { self.pos[k] += push[k]; }
        self.grounded_history = (self.grounded_history >> 1) | if self.grounded { 8 } else { 0 };
        // Fall damage: ticks falling with no ground for 4 ticks; on landing, over 60 → (timer − 60) if ≥ 10.
        if swimming {
        } else if self.grounded_history == 0 && self.vel[1] < 0.0 { self.fall_timer += 1; }
        else if self.grounded {
            if self.fall_timer > 60 && self.fall_timer - 60 >= 10 { self.fall_damage = Some((self.fall_timer - 60) as f32); }
            self.fall_timer = 0;
        }
        // Jump: a head bump ends the rise; landing after the delay. The zeroing arms stay held
        // off on the settle's held ticks (the accumulating velocity is the observable).
        if !settle_held && self.jump == 1 && self.head_bump {
            self.vel = [0.0; 3];
            self.jump = 2;
            // [M Player_HandleJump 0x1916b0:195-198]: the bump zeroes the velocity and plays
            // 0x5FB on levels whose `level_tag+0x50` byte is 0xD/0xE, else 0xA3.
            self.sfx_out.push((if ctx.level_bump_5fb { 0x5fb } else { 0xa3 }, self.pos));
        }
        self.land_delay = self.land_delay.saturating_sub(1);
        if !settle_held && self.jump != 0 && self.land_delay == 0 && self.grounded && self.vel[1] <= 0.0 { self.vel = [0.0; 3]; self.jump = 0; }
        if !settle_held && self.grounded && self.jump == 0 { self.vel[1] = 0.0; }
        // `Player_PositionCamera` 0x18f5d8 (its viewer write is `Camera_Set` 0x1bbbf8 — the
        // `View_RotTransMatrix` 0x1e5168 matrix into viewer+0x120, the rot copy to +0xE0, the
        // pos copy to +0xC0, the byte +0x2B cleared [M 0x1bbbf8:5-16]): the port carries the
        // same role as this eye/crouch-drop pair the view builds its camera from.
        // `Player_PositionCamera`: the crouch drop and the eye ease in.
        self.crouch_drop = if self.crouched { (self.crouch_drop + 0.0225).min(0.45) } else { (self.crouch_drop - 0.0225).max(0.0) };
        self.eye += (EYE - self.eye) * 0.1;
    }
}

/// `vecutil_calculate_closest_point_on_line` 0x1e1cb8's form [M — the Jointy spine pair
/// 0x1ec180:234-239 calls it three times]: the closest point to `p` on the segment a→b.
fn closest_on_seg(a: V3, b: V3, p: V3) -> V3 {
    let ab = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let ap = [p[0] - a[0], p[1] - a[1], p[2] - a[2]];
    let len_sq = ab[0] * ab[0] + ab[1] * ab[1] + ab[2] * ab[2];
    if len_sq < 1e-12 { return a; }
    let t = ((ap[0] * ab[0] + ap[1] * ab[1] + ap[2] * ab[2]) / len_sq).clamp(0.0, 1.0);
    [a[0] + ab[0] * t, a[1] + ab[1] * t, a[2] + ab[2] * t]
}

/// `Vec_AngleDifference` 0x1e2b78 [M]: `wrap(b − a)` into (−π, π].
fn wrap_pi(d: f32) -> f32 {
    let mut d = d % std::f32::consts::TAU;
    if d >= std::f32::consts::PI { d -= std::f32::consts::TAU; }
    if d < -std::f32::consts::PI { d += std::f32::consts::TAU; }
    d
}

/// `Vec_SqDist2D` 0x1e2ef8 [M]: the x/z squared distance (the wire clamp is 2D).
fn dist2_sq(a: V3, b: V3) -> f32 {
    let (dx, dz) = (a[0] - b[0], a[2] - b[2]);
    dx * dx + dz * dz
}

/// `Vec_Dist3D` squared [M 0x1e2e58's pattern].
fn dist3_sq(a: V3, b: V3) -> f32 {
    let d = [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
    d[0] * d[0] + d[1] * d[1] + d[2] * d[2]
}

// ── Level carry-over and spawn (fix-level2) ────────────────────────────────────────────────────────

/// The mission-tally slice of the game's `RamSave` staging buffer: the pair of copies
/// `Player_RamSave` 0x195670:10-11 makes — `RamSave+0x000 ← PlrMissionStats[0]` (0x38 bytes, the
/// SP slot-0 record) and `RamSave+0xA04 ← BondMoments` (0x14) [M]. `Player_RamLoad` 0x1957a8:15-16
/// mirrors both back, inside the same `IsWarmReset != 0` gate as the rest of the body [M]. This is
/// why the pause SCORE tab's counters survive The Exchange's 07000005 → 07000006 hop in the game:
/// the exit trigger saves the tally, the warm load restores it, and `Mission_Init` only wipes it
/// when the loaded map IS the mission's base map [M 0x182270:59-66].
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct TallySave {
    /// The slot-0 `PlrMissionStats` record (0x2bcc20, 0x38 bytes) [M 0x195670:10].
    pub stats: crate::progress::PlrMissionStats,
    /// `BondMoments` 0x2bcde0 (0x14, the ten 0x65..0x6E done flags) [M 0x195670:11].
    pub bond_moments: [i16; 10],
}

/// `Player_RamSave` (0x195670) → the global `RamSave` (0x2b9440, 0xD50 bytes) [M]. The port carries the
/// fields the port models; the game also saves `SSysItems` (0x28) — still unmodeled as a block (the
/// inventory fields cover the modeled part). `PlrMissionStats` (0x38) and `BondMoments` (0x14) ride
/// `tally` since 987-mission-stats-carry [M 0x195670:10-11].
#[derive(Clone, Debug, Default)]
pub struct RamSave {
    /// `RamSave+0x5FC ← BLData+0x894` [M 0x195670:16].
    pub health: f32,
    /// `RamSave+0x600 ← BLData+0x8B0` [M 0x195670:17].
    pub armour: f32,
    /// `RamSave+0xA18 ← M_CONTROL+0x63` (the switch-request byte) [M 0x195670:18-19].
    pub weapon: u8,
    /// `RamSave+0x038 ← BLData+0x170`, 0x42 = 33 s16 spare-ammo rounds [M 0x195670:6].
    pub carried: Vec<i16>,
    /// `RamSave+0x07C ← BLData+0x1B4`, 114 weapon entries × 12 B: clip s16 at +4, owned u8 at +6
    /// [M 0x195670:7; atlas player.md].
    pub clip: Vec<i16>,
    pub owned: Vec<bool>,
    /// `RamSave+0xA1A ← BLData+0x70C`, 0x33 key-ring bytes [M 0x195670:8].
    pub keys: Vec<bool>,
    /// `RamSave+0xA4D ← switch_channels`, 0x100 [M 0x195670:13].
    pub channels: Vec<u8>,
    /// `RamSave+0x604 ← switch_channels_time`, 0x400 [M 0x195670:12]. The game also saves `_hold` and
    /// `_prev` (0x2b9f8d/0x2ba08d); the port models no hold array and keeps `_prev` per-consumer
    /// (trigger-port-check #10) [M gap noted in the WORKLOG].
    pub ch_time: Vec<u32>,
    /// The mission layer's globals (`MissionState` 0x30cb9c, `BaseMap` 0x30cba0, `ThisOrderNum`
    /// 0x30cba4, `FailLabel` 0x30cba8 and the per-objective states). The game keeps them in `.data`
    /// across `ResetMap_LevelToLoad` — only `Mission_Init` 0x182270 touches them, and it resets the
    /// current and future parts IN PLACE, leaving previous parts alone [M 0x182270:68-129]. The port
    /// re-creates `Mission` per load, so the carry rides the RamSave (the payload shape is
    /// `LS_MakeMission`/`LS_LoadMission` 0x1cd720/0x1cd880; map-tower2).
    pub mission: Option<crate::mission::MissionSave>,
    /// The mission tally — `Player_RamSave` 0x195670:10-11's `PlrMissionStats[0]` (0x38) +
    /// `BondMoments` (0x14) copies [M]. `None` = a capture that predates the tally (or the test
    /// helpers' plain `capture`): `restore` leaves the fresh record alone, the cold-load shape.
    pub tally: Option<TallySave>,
}

/// `ContinueHealthBoostEasy` / `ContinueHealthBoostMedium` / `ContinueHealthBoostHard`
/// (0x30cc34 / 0x30cc38 / 0x30cc3c) [L ee_start_05.bin 0x30cc34..3c — all three 50.0 on the
/// retail build].
pub const CONTINUE_HEALTH_BOOST: [f32; 3] = [50.0; 3];

/// `Player_RamLoad` 0x1957a8:21-33's boost pick: the difficulty global `GameState+0x40`
/// (0x2a37a8 — the port's `Progress::difficulty` mirror) selects the table entry — 1 → Easy,
/// 2 → Medium, anything else (0/3/4) → Hard [M]. All three are 50.0 on retail [L], so the pick
/// is only observable against a non-retail table.
pub fn continue_health_boost(difficulty: u32) -> f32 {
    match difficulty { 1 => CONTINUE_HEALTH_BOOST[0], 2 => CONTINUE_HEALTH_BOOST[1], _ => CONTINUE_HEALTH_BOOST[2] }
}

impl RamSave {
    /// The save `Trigger_Activate` makes on the way out of a level, after the success/fallback channel
    /// write [M 0x1b2318:97-116].
    pub fn capture(health: &crate::bullet::PlayerHealth, weapon: u8, inv: &crate::pickup::Inventory, ch: &Channels) -> RamSave {
        RamSave {
            health: health.health,
            armour: health.armour,
            weapon,
            carried: inv.carried.clone(),
            clip: inv.clip.clone(),
            owned: inv.owned.clone(),
            keys: inv.keys.clone(),
            channels: ch.v.clone(),
            ch_time: ch.time.clone(),
            mission: None,
            tally: None,
        }
    }

    /// Attaches the mission tally — the game's `Player_RamSave` copies it unconditionally on every
    /// save [M 0x195670:10-11]. The port keeps it a builder step so the plain `capture` signature
    /// (and its many test call sites) stays; the game-path captures attach it.
    pub fn with_tally(mut self, p: &crate::progress::Progress) -> RamSave {
        self.tally = Some(TallySave { stats: p.stats[0], bond_moments: p.bond_moments });
        self
    }

    /// Attaches the mission globals snapshot — the game persists them in `.data` across a level load
    /// while the port re-creates `Mission` (map-tower2; see the `mission` field).
    pub fn with_mission(mut self, m: &crate::mission::Mission) -> RamSave {
        self.mission = Some(m.snapshot());
        self
    }

    /// `Player_RamLoad`'s boost arm (0x1957a8:21-34), the second argument non-zero [M]: the saved
    /// health is floored at the difficulty's `ContinueHealthBoost*` — a MAX, written back into the
    /// buffer (`swc1 $f12, 0x5fc` at 0x1958e4) before `Player_SetHealth` (jal 0x1960b8 at 0x1958e8)
    /// applies it; armour is untouched (restored separately from the save's +0x1536 [M :36]). The
    /// game's call sites: the fail/death menu's Restart row 0 passes 1 [M `P_ENDMISSION_Handler`
    /// 0x216618:21]; `Player_InitWeapon` 0x1a7ef8 passes 0 on all EIGHT of its `Player_RamLoad`
    /// calls [M :158-305] — the trigger-exit warm load never boosts. The port's boost-1 call site
    /// is the restart serve (`nf_view::fail_restart_save`, the handler's position in the reload
    /// flow); the trigger-exit capture never calls this.
    pub fn continue_boost(&mut self, difficulty: u32) {
        self.health = self.health.max(continue_health_boost(difficulty));
    }

    /// Attaches a pre-built mission snapshot — `fail_restart_save`'s serve carries the states in the
    /// shape the game's `Mission_Init(level, 1)` leaves the `.data` in (previous parts kept,
    /// current + future reset [M 0x182270:68-129]) because the port applies the carried snapshot
    /// AFTER its `Mission_Init` latch (drone.rs), while the game's `Player_RamLoad` never touches
    /// the states.
    pub fn with_mission_snapshot(mut self, m: crate::mission::MissionSave) -> RamSave {
        self.mission = Some(m);
        self
    }

    /// `Player_RamLoad` (0x1957a8), `IsWarmReset != 0` arm [M]: the whole body is gated on the warm flag,
    /// so a cold load (`save = None`) restores nothing. Restores health (`Player_SetHealth`), armour
    /// (+0x8B0), the saved weapon id (+0x934) and the four channel arrays. The `boost` argument is the
    /// buffer-side max `RamSave::continue_boost` (0x1957a8:21-34): the fail-menu Restart's serve applies
    /// it before the reload reads the buffer — exactly the game's write-back into RamSave+0x5FC
    /// (`swc1 $f12, 0x5fc` 0x1958e4) before `Player_SetHealth` (jal 0x1960b8 0x1958e8) — so `restore`
    /// itself applies whatever the buffer holds.
    pub fn restore(&self, health: &mut crate::bullet::PlayerHealth, inv: &mut crate::pickup::Inventory, ch: &mut Channels) -> u8 {
        health.health = self.health;
        health.armour = self.armour;
        let fit = |dst: &mut Vec<i16>, src: &[i16]| { dst.resize(src.len(), 0); dst.copy_from_slice(src); };
        fit(&mut inv.carried, &self.carried);
        fit(&mut inv.clip, &self.clip);
        inv.owned.resize(self.owned.len(), false);
        for (d, &s) in inv.owned.iter_mut().zip(&self.owned) { *d = s; }
        inv.keys.resize(self.keys.len(), false);
        for (d, &s) in inv.keys.iter_mut().zip(&self.keys) { *d = s; }
        ch.v.resize(256, 0);
        ch.v.copy_from_slice(&self.channels);
        ch.time.resize(256, 0);
        ch.time.copy_from_slice(&self.ch_time);
        self.weapon
    }
}

/// One registered player start (`player_start` entry, 0xE0 bytes): position, yaw, the gate switch
/// channel — placement type 0x24's parameter 0 [M `parsemap_create_dynamic_objects` 0x1d04f0 case 0x24 →
/// `Player_AddNewStartPos` 0x195c00: param block +0x2C lands at entry+0x50].
#[derive(Clone, Copy, Debug)]
pub struct LevelStart {
    pub pos: V3,
    pub yaw: f32,
    pub gate: u16,
    /// The placement's param 3 (level_tag+0x38) — the zero-g selector `Player_Init` 0x1910f8
    /// switches on (== 2 → `Vec_Zero(&WldGravity)`; else the default (0, −9.8, 0)) [M].
    pub p3: u32,
    /// The registration kind — `Player_AddNewStartPos`'s u16 third argument, stored at entry+0x22
    /// [M 0x195c00:33]: type-0x24 placements pass 1, type-0x2D pass 0 [M 0x1d04f0 case 0x24 →
    /// disasm 0x1d0a00 `addiu a2,$zero,1`; case 0x2d → disasm 0x1d0e98 `move a2,$zero`].
    /// `Player_Start` 0x190fd0 only ever picks kind-1 entries (disasm 0x191028/0x1910b0:
    /// `bne v0, 1` skips the entry); the kind-0 records are written but never read by the game's
    /// on-foot runtime — no other function touches `player_start` in the decompile.
    pub kind: u16,
}

/// The registration: the main map's placements, in parse order, through
/// `parsemap_create_dynamic_objects` 0x1d04f0's start-pos arms [M]. The game parses the main map's
/// dynamic block with `n != 0` (`ResetMap_Load` 0x1d6360:150 — `parsemap_block_map_data_dynamic
/// (0x30cfe8, DynamicPtr, 1)`; `n != 0` = the main parse, the port's `main_map` convention), only
/// records with `(flags & 0xa000) == 0` reach the dispatch [M 0x1d1b68:93], and both arms sit
/// behind the MP gate `MPSettings._384_4_ == 0` [M disasm 0x1d09e0-0x1d09e8 / 0x1d0e78-0x1d0e80 —
/// the port's single-player path is that case]:
/// - case 0x24 → `Player_AddNewStartPos(pos, rot, 1, level_tag)` [M 0x1d04f0:91-98, disasm 0x1d0a00];
/// - case 0x2d → the same with 0 [M 0x1d04f0:154-161, disasm 0x1d0e98].
///
/// The game also skips placements whose object lookup failed (`beqz a3` before the call
/// [M 0x1d1c90]) — every 0x24/0x2d placement on the disc's main maps resolves in the port's
/// assembly (checked by start_slot_0x2d_live's sweep), so the port keeps the data-level rule.
/// Yaw = the placement's rotation[1], the pinned convention (party07-start-rules); gate = param 0,
/// p3 = param 3 (level_starts' parse, unchanged).
/// `Player_ResetStartPos` 0x195bf0 (16 B, `ResetMap_GameInit` 0x1d5080:18 — the first init
/// action): zeroes the game's `player_start_positions_index` / `player_start_position` globals
/// [M 0x195bf0:6-7] before the map parse refills them (`Player_AddNewStartPos` 0x195c00:20,65).
/// The port's equivalent is this per-load rebuild — fresh `Vec` each scene build, one
/// `Player_Start` per load — so the two globals' reset is implicit in the rebuild.
pub fn starts_from_placements(placements: &[nf_formats::placement::Placement]) -> Vec<LevelStart> {
    placements.iter()
        .filter(|p| p.flags & 0xa000 == 0)
        .filter_map(|p| match p.flags & 0xff {
            0x24 => Some((1u16, p)),
            0x2d => Some((0u16, p)),
            _ => None,
        })
        .map(|(kind, p)| LevelStart {
            pos: p.translation,
            yaw: p.rotation[1],
            gate: p.params.iter().find(|(i, _)| *i == 0).map_or(0, |&(_, v)| v as u16),
            p3: p.params.iter().find(|(i, _)| *i == 3).map_or(0, |&(_, v)| v),
            kind,
        })
        .collect()
}

/// `Player_Start` (0x190fd0) [M]: pass 1 — the first kind-1 entry whose gate is 0 or whose channel is
/// on; pass 2 (fallback) — the first kind-1 entry (the game zeroes its gate so a later `Player_Start`
/// in the same level cannot re-pick it; the port runs one `Player_Start` per load). Kind-0 entries
/// (the 0x2D set) never win either pass [M disasm 0x191028 and 0x1910b0: both loops `bne v0, 1` on
/// entry+0x22]. Index order = placement order [M 0x195c00 appends]. Returns the entry index, or None
/// with no kind-1 entry (the MP maps' human spawns come from `MP_Start` 0x183a50's `MP_GetSpawnPoint`,
/// not `player_start` [M 0x183a50:118-128]).
pub fn pick_start(starts: &[LevelStart], ch: &Channels) -> Option<usize> {
    if let Some(i) = starts.iter().position(|s| s.kind == 1 && (s.gate == 0 || ch.on(s.gate as u32))) {
        return Some(i);
    }
    starts.iter().position(|s| s.kind == 1)
}

/// `Player_InitAmmoWeapons` (0x1a17f8) with the boot flag `DAT_002a373c == 0`: carried = 0, clip = 0,
/// owned = false for every weapon [M :20-45]. The [L] boot snapshot proves this arm ran: owned after
/// `Player_InitWeapon` = exactly the equip set {1, 2, 0x4A, 0x4E, 0x50, 0x54} — the +0x04 byte is 1 on
/// every probed weapon, so the non-zero arm (owned = +0x04) would have owned 23 (ee_start_05.bin;
/// WORKLOG fix-level2). `Player_CheckWeaponsLoaded` (0x1a97c8) can only revoke owned — a no-op from
/// all-false. The equip calls that follow carry the `loaded` refusal of `Player_EquipWeapon`
/// [M 0x1a8a58:20].
pub fn init_ammo_weapons(inv: &mut crate::pickup::Inventory, w: &nf_formats::weapon::WeaponData, loaded: impl Fn(u8) -> bool) {
    for c in inv.carried.iter_mut() { *c = 0; }
    for id in 0..inv.clip.len().min(0x72) {
        inv.clip[id] = 0;
        inv.owned[id] = false;
    }
    let _ = (w, loaded);
}

/// `Player_EquipWeapon` is `Inventory::equip_weapon` [M 0x1a8a58]; the game looks the model up in the
/// hashtable and refuses when missing — the port's `loaded` closure stands in.
fn equip(inv: &mut crate::pickup::Inventory, w: &nf_formats::weapon::WeaponData, id: u16, rounds: i16, loaded: impl Fn(u8) -> bool) {
    inv.equip_weapon(w, id as usize, rounds, loaded(id as u8));
}

/// `Player_InitWeapon` (0x1a7ef8) — the per-level loadout switch on `GameState+0x0C`, the whole
/// jump table (`sltiu v1,a0,74` 001a8268: the level ids 0x0700001..0x070004a; anything else is the
/// default arm). `save = Some(…)` = a warm transition (`IsWarmReset != 0`, the only way a
/// `LOAD_LEVEL`/script-event-0x12 transition loads [M 0x1d3800, 0x1b24cc]); `None` = cold
/// (`Player_RamLoad`'s whole body is skipped [M 0x1957a8:9]).
///
/// Returns the switch request the game writes to `M_CONTROL+0x63` (the draw machine then holsters and
/// draws it). Cases [M 0x1a7ef8:148-317]:
/// - 0x0700001 (fresh, and the fallback of 02/03/04): P2K 48 rounds, taser 999, **0x5b 999** (the
///   60 Hz pick — 0x59 is the 50 Hz one, disasm `001a8244 movz s4,v0,a0`); request = `Upgrade_Weapon(6)`
///   = 2 [L UpgradedHandguns 0x2bee08 = [2,4,6,8], Upgrades 0].
/// - 0x0700002/3/4: `Player_RamLoad`; request = the saved weapon; if the P2K is not owned → the fresh
///   01 kit (`switchD_001a8284_caseD_7000001`).
/// - 0x0700005 (fresh, and the fallback of 06): P2K 48 rounds, taser 999, laser 0, watch 0, + the common
///   tail; request = 2.
/// - 0x0700006: `Player_RamLoad`; request = the saved weapon; if the P2K is not owned → the fresh 05 case.
/// - 0x0700007: `Player_RamLoad`, `Player_InitAmmoWeapons`, armour = 0, P2K 48 / taser 999 / laser 0 /
///   watch 0, request = **1 (fists)** — the party disguise.
/// - 0x0700008: `Player_RamLoad`; request = the saved weapon; if the P2K is not owned → the 07 case.
/// - 0x0700009 (fresh, and the fallback of 0a/0b): P2K 48 / taser 999 / decryptor 0x56 0 / Q-Worm 0x58 0 /
///   laser 0x50 0 / dart gun 0x43 999 / 0x5b 999; request = 0x43.
/// - 0x070000c (fresh, and the fallback of 0d): P2K 48 / sniper 0x1e 999 / taser 999 / 0x50 / 0x54 /
///   0x58 at 0; request = 0x1e.
/// - 0x070000d: `Player_RamLoad`; request = the saved weapon; the continuation gate reads
///   `owned[Upgrade_Weapon(6)]` = owned[2] like every RamLoad case (the shared `iVar8*0xC + BL+0x1BA`
///   read — the disasm's `lbu 442(v1)` with v1 = s0*12+BL, s0 = 2 [M 001a82a0-1a82b4]); when unowned →
///   the fresh 0c kit.
/// - 0x0700011 (fresh, and the fallback of 12/13/4a): **0x10 with 7 rounds** and the laser 0x50 at 0 —
///   no P2K; request = 0x10.
/// - 0x0700012/13/4a: `Player_RamLoad`; request = the saved weapon; the gate reads `owned[0x10]`
///   (`iVar13 + 0x27a` = BL+0x1BA + 0x10·0xC); when unowned → the fresh 11 kit.
/// - 0x0700014 (fresh, and the fallback of 15/16/17): P2K 48, **crossbow 0x11 12 rounds**, frag 0x34 5,
///   mine 0x37 5, taser 999, laser 0x50 0, camera 0x54 0, decryptor 0x56 0; request = **0x11** — the
///   Delta Repeater starts in hand [L the 2026-10-14 measurement].
/// - 0x0700015/16/17: `Player_RamLoad`; request = the saved weapon; if the P2K is not owned → the fresh
///   14 kit (`switchD_001a8284_caseD_7000014`).
/// - 0x0700018/19/1a/1b (fresh): `Player_InitAmmoWeapons`, **0x33 999 only**, armour = 50.0
///   (`0x42480000` [M]); request = 0x33.
/// - default (ids outside the jump table): the give-everything list (equips 6/0x12/0x18/0x46/0x37/
///   0x36/0x35/0x34/0x3a/0x52/0x2c/0x2e/0x30/0x4a + 0x50@0 + 0x1e + EquipAmmo(0x1f,10) + 0x24 +
///   EquipAmmo(0x25,999) + 0xe/0x10 + 0x54@0 + 0x1c/0x15/10/0xd + 0x56@0/0x58@0 + 0x41/0x59/0x3b +
///   0x1a + EquipAmmo(0x1b,999) + 0x33/0x11/2/0x2a/0x42/0x43/0x45, then 0x16 999), request = 2.
///
/// Common tail [M 0x1a7ef8:318-325]: equip laser 0x4E 999, equip fists 1 with 0 rounds,
/// `Player_CheckWeaponsLoaded` (0x1a97c8 — a selectable weapon whose first-person skin +0xDC is missing
/// from the hashtable loses owned), the request falling to 1 when its base is unowned
/// (`owned[weapon_data[req].+0x02] == 0 → request = 1`, disasm 001a89c4-001a89fc), and the re-save.
pub fn init_level_loadout(
    level: u32,
    save: Option<&RamSave>,
    health: &mut crate::bullet::PlayerHealth,
    inv: &mut crate::pickup::Inventory,
    ch: &mut Channels,
    w: &nf_formats::weapon::WeaponData,
    loaded: impl Fn(u8) -> bool + Copy,
) -> u8 {
    // `Upgrade_Weapon(6)` (0x1a8f30 case 6 → `UpgradedHandguns[Upgrades]`, slot 0 = 2 [L]).
    const HANDGUN: u16 = 2;
    let mut req = HANDGUN as u8;
    // A cold load (`save = None`, `IsWarmReset == 0`) skips `Player_RamLoad`'s whole body [M
    // 0x1957a8:9], so the inventory `Player_InitWeapon` reads behind it is the game's boot state —
    // the one `Player_InitAmmoWeapons` (0x1a17f8) zeroed (the [L] boot snapshot proves the zero arm
    // ran: owned = exactly the equip set, WORKLOG fix-level2). fix-perf-level: the port's Sim used to
    // carry the viewer's legacy default (P2K 7/41, `Gun::new(2,7,41)` + the sim's built-in owned
    // flags), so case 0x7000006/8's `p2k_owned` read TRUE on a cold load and the request became the
    // uninitialized saved weapon 0 — the draw machine holstered into weapon 0 and Bond stood unarmed
    // (nf-view 07000006 --frame 30: "switch request 0", no weapon). Clear first on
    // a cold load; cases 5/7 and the default re-clear or replace anyway, so only 6/8 change.
    if save.is_none() { init_ammo_weapons(inv, w, loaded); }
    // `Player_RamLoad` (0x1957a8): the saved health/armour/weapon/inventory/channels.
    let saved_weapon = save.map(|s| s.restore(health, inv, ch)).unwrap_or(0);
    let p2k_owned = |inv: &crate::pickup::Inventory| inv.owned.get(HANDGUN as usize).copied().unwrap_or(false);
    let up = |id: u16| crate::pickup::Inventory::upgrade_weapon(id, 0, 0);
    // [M 0x1a7ef8 case 0x7000001 → LAB_001a8980]: P2K 48, taser 999, then the shared tail-label equip
    // of uVar11 = 0x5b (the 60 Hz pick) with 999; held = s0 = 2.
    let fresh_01 = |inv: &mut crate::pickup::Inventory, health: &mut crate::bullet::PlayerHealth| {
        init_ammo_weapons(inv, w, loaded);
        equip(inv, w, HANDGUN, 0x30, loaded);
        equip(inv, w, up(0x4a), 999, loaded);
        equip(inv, w, 0x5b, 999, loaded);
        let _ = health;
    };
    let fresh_05 = |inv: &mut crate::pickup::Inventory, health: &mut crate::bullet::PlayerHealth| {
        init_ammo_weapons(inv, w, loaded);
        equip(inv, w, HANDGUN, 0x30, loaded);
        equip(inv, w, up(0x4a), 999, loaded);
        equip(inv, w, 0x50, 0, loaded);
        equip(inv, w, 0x54, 0, loaded);
        let _ = health;
    };
    let party = |inv: &mut crate::pickup::Inventory, health: &mut crate::bullet::PlayerHealth| {
        init_ammo_weapons(inv, w, loaded);
        *armour_in(health) = 0.0;
        equip(inv, w, HANDGUN, 0x30, loaded);
        equip(inv, w, up(0x4a), 999, loaded);
        equip(inv, w, 0x50, 0, loaded);
        equip(inv, w, 0x54, 0, loaded);
    };
    // [M 0x1a7ef8 case 0x7000009]: the Nightfire loadout — `Upgrade_Weapon(6)` (the handgun,
    // 48), `Upgrade_Weapon(0x4a)` (the Stunner, 999), `Upgrade_Weapon(0x56)` (the Decryptor, 0),
    // **0x58 (the Q-Worm, 0)**, 0x50 (the Grapple, 0), `Upgrade_Weapon(0x43)` (the dart gun, 999)
    // and 0x5b (the Shaver, 999), selecting the dart gun. The upgrade level is the boot `Upgrades`
    // (0), so every `Upgrade_Weapon` is its base record here.
    let fresh_09 = |inv: &mut crate::pickup::Inventory| {
        init_ammo_weapons(inv, w, loaded);
        equip(inv, w, up(6), 0x30, loaded);
        equip(inv, w, up(0x4a), 999, loaded);
        equip(inv, w, up(0x56), 0, loaded);
        equip(inv, w, 0x58, 0, loaded);
        equip(inv, w, 0x50, 0, loaded);
        equip(inv, w, up(0x43), 999, loaded);
        equip(inv, w, 0x5b, 999, loaded);
    };
    // [M 0x1a7ef8 case 0x700000c]: the power station — P2K 48, `Upgrade_Weapon(0x1e)` (the sniper,
    // 999), `Upgrade_Weapon(0x4a)` (the Stunner, 999) and the gadgets 0x50/0x54/0x58 at 0, selecting
    // the sniper (`+0x63 = uVar3`).
    let fresh_0c = |inv: &mut crate::pickup::Inventory| {
        init_ammo_weapons(inv, w, loaded);
        equip(inv, w, HANDGUN, 0x30, loaded);
        equip(inv, w, up(0x1e), 999, loaded);
        equip(inv, w, up(0x4a), 999, loaded);
        equip(inv, w, 0x50, 0, loaded);
        equip(inv, w, 0x54, 0, loaded);
        equip(inv, w, 0x58, 0, loaded);
    };
    // [M 0x1a7ef8 case 0x7000011]: 0x10 with 7 rounds and the Grapple 0x50 at 0 — no handgun, no
    // tail-request change: held = 0x10.
    let fresh_11 = |inv: &mut crate::pickup::Inventory| {
        init_ammo_weapons(inv, w, loaded);
        equip(inv, w, 0x10, 7, loaded);
        equip(inv, w, 0x50, 0, loaded);
    };
    // [M 0x1a7ef8 case 0x7000014]: the evil-base kit — P2K 48, the Delta Repeater 0x11 with 12
    // rounds, frag 0x34 5, mine 0x37 5, Stunner 999, Grapple/camera/decryptor at 0; held = 0x11
    // (the crossbow starts in hand [L the 2026-10-14 measurement]).
    let fresh_14 = |inv: &mut crate::pickup::Inventory| {
        init_ammo_weapons(inv, w, loaded);
        equip(inv, w, HANDGUN, 0x30, loaded);
        equip(inv, w, 0x11, 0xc, loaded);
        equip(inv, w, 0x34, 5, loaded);
        equip(inv, w, 0x37, 5, loaded);
        equip(inv, w, up(0x4a), 999, loaded);
        equip(inv, w, 0x50, 0, loaded);
        equip(inv, w, 0x54, 0, loaded);
        equip(inv, w, up(0x56), 0, loaded);
    };
    match level {
        0x0700_0001 => { fresh_01(inv, health); req = HANDGUN as u8; }
        // [M 0x1a7ef8 cases 0x7000002/3/4]: RamLoad keeps the carry and re-selects the saved weapon;
        // without the P2K the fresh 01 kit re-runs (switchD_001a8284_caseD_7000001).
        0x0700_0002 | 0x0700_0003 | 0x0700_0004 => {
            req = saved_weapon;
            if !p2k_owned(inv) { fresh_01(inv, health); req = HANDGUN as u8; }
        }
        0x0700_0005 => fresh_05(inv, health),
        0x0700_0006 => {
            req = saved_weapon;
            if !p2k_owned(inv) { fresh_05(inv, health); req = HANDGUN as u8; }
        }
        0x0700_0007 => { party(inv, health); req = 1; }
        0x0700_0008 => {
            req = saved_weapon;
            if !p2k_owned(inv) { party(inv, health); req = 1; }
        }
        0x0700_0009 => { fresh_09(inv); req = up(0x43) as u8; }
        // [M 0x1a7ef8 cases 0x700000a/0x700000b]: `Player_RamLoad` keeps the carry and re-selects the
        // saved weapon; when the handgun record is unowned the 0x0700009 loadout re-runs (the island's
        // Q-Worm arrives here — without it there was no grant at all).
        0x0700_000a | 0x0700_000b => {
            req = saved_weapon;
            if !p2k_owned(inv) { fresh_09(inv); req = up(0x43) as u8; }
        }
        0x0700_000c => { fresh_0c(inv); req = up(0x1e) as u8; }
        // [M 0x1a7ef8 case 0x700000d]: RamLoad keeps the carry and re-selects the saved weapon; the
        // continuation gate is owned[2] like every RamLoad case (the shared `iVar8*0xC + 0x1BA` read,
        // not the sniper's slot); when unowned the 0c loadout re-runs.
        0x0700_000d => {
            req = saved_weapon;
            if !p2k_owned(inv) { fresh_0c(inv); req = up(0x1e) as u8; }
        }
        0x0700_0011 => { fresh_11(inv); req = 0x10; }
        // [M 0x1a7ef8 cases 0x7000012/13/0x700004a]: RamLoad keeps the carry; the gate reads
        // owned[0x10] (`iVar13 + 0x27a`); when unowned the 11 kit re-runs.
        0x0700_0012 | 0x0700_0013 | 0x0700_004a => {
            req = saved_weapon;
            if !inv.owned.get(0x10).copied().unwrap_or(false) { fresh_11(inv); req = 0x10; }
        }
        0x0700_0014 => { fresh_14(inv); req = 0x11; }
        // [M 0x1a7ef8 cases 0x7000015/16/17]: RamLoad keeps the carry and re-selects the saved
        // weapon; when the P2K is unowned the fresh 14 kit re-runs (switchD_001a8284_caseD_7000014).
        0x0700_0015 | 0x0700_0016 | 0x0700_0017 => {
            req = saved_weapon;
            if !p2k_owned(inv) { fresh_14(inv); req = 0x11; }
        }
        // [M 0x1a7ef8 cases 0x7000018/19/1a/1b]: the space-station kit — 0x33 999 only, armour 50.0,
        // held = 0x33.
        0x0700_0018 | 0x0700_0019 | 0x0700_001a | 0x0700_001b => {
            init_ammo_weapons(inv, w, loaded);
            equip(inv, w, 0x33, 999, loaded);
            *armour_in(health) = 50.0; // `BLData+0x8B0 = 0x42480000` [M 0x1a7ef8 case 0x7000018]
            req = 0x33;
        }
        // [M 0x1a7ef8 default] (the 001a86f0 arm, also the jump-table's out-of-range target): the
        // give-everything list, then the shared tail-label equip of 0x16 999 (disasm 001a897c
        // `addiu a1,zero,22`), holding the P2K.
        _ => {
            init_ammo_weapons(inv, w, loaded);
            for (id, rounds) in [(6u16, 999i16), (0x12, 999), (0x18, 999), (0x46, 999), (0x37, 999),
                (0x36, 999), (0x35, 999), (0x34, 999), (0x3a, 999), (0x52, 999), (0x2c, 999),
                (0x2e, 999), (0x30, 999), (0x4a, 999), (0x50, 0), (0x1e, 999), (0x24, 999),
                (0xe, 999), (0x10, 999), (0x54, 0), (0x1c, 999), (0x15, 999), (10, 999), (0xd, 999),
                (0x56, 0), (0x58, 0), (0x41, 999), (0x59, 999), (0x3b, 999), (0x1a, 999),
                (0x33, 999), (0x11, 999), (2, 999), (0x2a, 999), (0x42, 999), (0x43, 999), (0x45, 999)] {
                equip(inv, w, id, rounds, loaded);
            }
            inv.equip_ammo(w, 0x1f, 10);
            inv.equip_ammo(w, 0x25, 999);
            inv.equip_ammo(w, 0x1b, 999);
            equip(inv, w, 0x16, 999, loaded);
            req = HANDGUN as u8;
        }
    }
    // Common tail [M 0x1a7ef8:318-319].
    equip(inv, w, 0x4e, 999, loaded);
    equip(inv, w, 1, 0, loaded);
    // `Player_CheckWeaponsLoaded` (0x1a97c8): a selectable weapon whose first-person skin (+0xDC)
    // misses the hashtable loses owned — the RamLoad restore can carry owned across a level boundary
    // the skins did not cross.
    for id in 0..inv.owned.len().min(0x72) {
        if inv.owned[id] && !loaded(id as u8) && w.selectable(id as u8) { inv.owned[id] = false; }
    }
    // [M 0x1a7ef8:321-324, disasm 001a89c4-001a89fc]: an unowned request falls to the fists (1) —
    // `owned[weapon_data[req].base(+0x02)] == 0`.
    if !inv.owned.get(w.base(req as usize) as usize).copied().unwrap_or(false) { req = 1; }
    req
}

/// The armour lives in `PlayerHealth` next to health; a tiny accessor keeps `party`'s closure signature
/// flat (the game writes `BLData+0x8B0` [M 0x1a7ef8:180]).
fn armour_in(h: &mut crate::bullet::PlayerHealth) -> &mut f32 { &mut h.armour }

/// The party weapon lock (`Player_Weapon` 0x1a24c8:242-275) [M]: on 0x07000007 with switch channel 0x48
/// OFF (`if (switch_channels[0x48] == '\0') bVar4 = false`), the weapon-cycle actions route to
/// `Player_WeaponChange(±1, mode 2)` — the fists-only cycle. The port zeroes the cycle request (the
/// mode-2 cycle from fists re-picks fists and plays the empty-switch click 0x1DB — no switch, sound
/// aside) and, as the only way a player could otherwise reach a gun, refuses direct selections that are
/// not the fists [S: the number-key path's lock is not verified in the decompile; the observable game
/// behaviour is "no weapons at the party"].
pub const WEAPON_LOCK_LEVEL: u32 = 0x0700_0007;
/// The re-arm channel: TouchOnce placements 411/412 on 07000007 write it [D nf-extract triggers];
/// `Trigger_Activate` case 1 latches it [M 0x1b2318:41-49].
pub const WEAPON_LOCK_CHANNEL: u32 = 0x48;

pub fn weapon_lock_input(level: u32, ch48_on: bool, input: &mut Input) {
    // [M 0x1a24c8:243-246]: `if (GameState == 0x7000007) { if (switch_channels[0x48] == 0) bVar4 = false; }`
    // — the lock is active while the channel is OFF.
    if level == WEAPON_LOCK_LEVEL && !ch48_on {
        input.weapon_cycle = 0;
        if input.switch_to.is_some_and(|w| w != 1) { input.switch_to = None; }
    }
}

/// `Player_Weapon`'s rising-edge re-arm [M 0x1a24c8:247-250]: channel 0x48 on while `switch_channels_prev`
/// still reads off → the switch request becomes `Upgrade_Weapon(6)` = 2 (the P2K auto-draws).
pub fn weapon_lock_auto_draw(level: u32, on: bool, prev: bool) -> bool {
    level == WEAPON_LOCK_LEVEL && on && !prev
}

/// A hint marker (placement kind 0x33, `Hint_Create` 0x1b3248 → class 0x43): the one-shot text
/// markers the game scatters next to usable things ("Hint: Use Watch Laser" at the 07000008 safe).
/// `Hint_Create` builds a 0x10-byte instance off the placement's params:
/// `inst+0x00` = param 1 (**the sound id** — `Sound_Play(100.0, sound, 0, 0)` on fire; 0 = silent),
/// `inst+0x04` = param 0 (**the text label** — `Txt_BindLabel(param 0)`; the safe's is 0x1000336
/// [D USATxt.dat entry 1822]), `inst+0x08` = param 3 (**the on-channel** — the hint only fires
/// while the channel is 0 (never armed) or on), `inst+0x0C` = param 4 (**the off-channel** — when
/// it is non-zero and ON the hint stays silent) [M 0x1b3248:33-37, 0x1b3380:17-22].
///
/// `Hint_Update` 0x1b3380 fires it once: with a class-3 object (the player) in the marker's
/// obj+0xD0 chain (the same straddle-cel occupancy the third icons arm through — the marker's
/// world sphere is its map object's bounds, `Control_BuildWorldSph` 0x133070 via
/// `build_LinkToRoom` 0x1c9680:40, resolved like `wire::icons`) it plays the sound, posts
/// `Text_AddMsg(BL+0x94e, 0, 1, label, 0, 0xb4)` — TXTMSG **type 1 = the INFO pane**, life 0xB4 =
/// 180 ticks — and sets obj+0xFE bit 0, the delete flag [M 0x1b3380:24-42; obj-tag.md +0xFE bit 0
/// = delete me]. `Hint_Create`'s stage gate is the GameState+0x40 compare [M 0x1b3248:12-21:
/// `param2 != 0 && param2 < GameState+0x40` → refuse; `param2 == 5` → only state 1; `6` → only 2;
/// `7` → only 3], and it is 0 on every hint marker found on the disc so far [D census] — the
/// gate's non-zero arms never fire in the shipped data.
///
/// The label id rides in the struct; the message text binds at drain (`USATxt.dat`, the scene's
/// `lang`), so the tests can pin the game's ids without disc text.
#[derive(Clone, Debug)]
pub struct Hint {
    /// The placement translation (the marker object's position).
    pub pos: V3,
    /// `obj+0x80..0x8C`: the world-sphere centre through the placement's yaw matrix (the same
    /// resolution as `wire::icons` [M Control_BuildWorldSph 0x133070]).
    pub centre: V3,
    /// `obj+0x8C`: the world-sphere radius (the object's glist bounds — the trigger volume the
    /// player's pick fills the obj+0xD0 chain through).
    pub radius: f32,
    /// Param 0: the hint's text label (`Txt_BindLabel` id).
    pub label: u32,
    /// Param 1: the sound id played on fire (0 = none).
    pub sound: u32,
    /// Param 3: the on-channel (0 = always armed).
    pub on_channel: u32,
    /// Param 4: the off-channel (when non-zero and ON the hint never fires).
    pub off_channel: u32,
    /// Param 2: the difficulty stage gate (`Hint_Create` 0x1b3248:12-25): 0 = always; 1..=4 =
    /// a difficulty CEILING (the hint is refused when `stage < difficulty`, so stage 1 shows
    /// on Operative only and stage 3 everywhere); 5/6/7 = exactly Operative/Agent/00 Agent [M].
    pub stage: u32,
}

/// `Hint_Create`'s stage gate, verbatim [M 0x1b3248:12-25]: `stage != 0 && stage < difficulty`
/// refuses, then 5/6/7 refuse unless the difficulty is exactly 1/2/3 (the DAT_002a37cc master
/// gate is a bootup-set 1 — always on in the shipped game, not difficulty-dependent).
pub fn hint_stage_gates(stage: u32, difficulty: u32) -> bool {
    if stage != 0 && stage < difficulty { return false; }
    if stage == 5 && difficulty != 1 { return false; }
    if stage == 6 && difficulty != 2 { return false; }
    if stage == 7 && difficulty != 3 { return false; }
    true
}

/// Every hint marker of a level (placement kind 0x33, `parsemap_create_dynamic_objects` 0x1d04f0
/// case 0x33 → `Hint_Create`), in placement order, resolved against its map object exactly like
/// `wire::icons` (an unresolved placement never creates an object, so it is skipped).
pub fn hints(level: &nf_formats::level::Level, difficulty: u32) -> Vec<Hint> {
    let mut out = Vec::new();
    for (mi, m) in level.maps.iter().enumerate() {
        for p in &m.placements {
            if p.flags & 0xff != 0x33 { continue; }
            let stage = p.params.iter().find(|(j, _)| *j == 2).map_or(0, |&(_, v)| v);
            if !hint_stage_gates(stage, difficulty) { continue; }
            let obj = if p.by_hash() { level.by_hash.get(&p.hashcode).copied() }
                else if (p.index as usize) < m.objects.len() { Some((mi, p.index as usize)) } else { None };
            let obj = obj.and_then(|(om, oi)| level.maps.get(om).and_then(|map| map.objects.get(oi)));
            let Some(o) = obj else { continue };
            let yaw = p.rotation[1];
            let (s, c) = yaw.sin_cos();
            let centre = [
                p.translation[0] + o.centre[0] * c + o.centre[2] * s,
                p.translation[1] + o.centre[1],
                p.translation[2] - o.centre[0] * s + o.centre[2] * c,
            ];
            let scale = p.scale();
            let s_max = scale.iter().fold(1.0f32, |a, &v| a * v.max(1e-6).abs());
            out.push(Hint {
                pos: p.translation,
                centre,
                radius: o.radius * s_max,
                label: param_of(p, 0),
                sound: param_of(p, 1),
                on_channel: param_of(p, 3),
                off_channel: param_of(p, 4),
                stage,
            });
        }
    }
    out
}

/// The placement's param k (the word at +0x2C + k·4); 0 when the record omits it (the same
/// default `wire::icons` reads through [M 0x1d04f0's param-buffer reads]).
fn param_of(p: &nf_formats::placement::Placement, k: u32) -> u32 {
    p.params.iter().find(|(i, _)| *i == k).map_or(0, |&(_, v)| v)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// fix-swim: the grab gate [M `Player_CollLadder` 0x196118:16-17] excludes ONLY substates 1
    /// (climb) and 2 (grapple) — swim (3) and crouch (4) grab. Pre-fix the port's `substate !=
    /// SS_WALK` gate made swim_tick's grab call dead code (the 02 pond hut ladder could not be
    /// grabbed from the water). No NF_ISO needed: a hand-built ladder context.
    #[test]
    fn the_ladder_grab_gate_excludes_only_climb_and_grapple() {
        let world = CollisionWorld::from_triangles(Vec::new());
        let lad = Ladder::simple([10.0, 3.0, 20.0], 2.0, 0.0);
        let ctx = LadderCtx { ladders: &[lad.clone()], wires: &[], wire_anims: None, water: Some(2.0), sink: false, creepwalls: &[], level_bump_5fb: false, straddle: &[], bot_capsules: &[] };
        // SWIM grabs: one tick in the water context attaches (pre-fix it stayed SS_SWIM).
        let mut b = Bond::new([10.0, 3.0, 20.5], lad.yaw + std::f32::consts::PI);
        b.substate = SS_SWIM;
        b.tick_ctx(&world, &ctx, &Input { forward: 1.0, ..Default::default() });
        assert_eq!(b.substate, SS_CLIMB, "the swim grab never attached (substate {})", b.substate);
        assert_eq!(b.ladder, Some(0), "the grab did not latch the ladder");
        assert!((b.pos[2] - (lad.centre[2] + 0.5)).abs() < 1e-3, "the grab snap left the axis: {:?}", b.pos);
        // Sister: the crouch stance (4) grabs too — the game's gate passes it.
        let mut b4 = Bond::new([10.0, 3.0, 20.5], lad.yaw + std::f32::consts::PI);
        b4.substate = 4; // SS_CROUCH (no named const in this port; the dispatch default arm)
        b4.tick_ctx(&world, &ctx, &Input { forward: 1.0, ..Default::default() });
        assert_eq!(b4.substate, SS_CLIMB, "the crouch grab never attached (substate {})", b4.substate);
        // Sister: the water grab's y-snap holds the swimmer at the rungs (never BELOW bottom+0.02).
        assert!(b.pos[1] >= lad.bottom() + 0.02 - 1e-4, "the grab dropped the swimmer below the rungs: {}", b.pos[1]);
    }

    /// ladder-mesh-refine: the straddled-cel scoping [M `Collide_Pick` 0x1eb320:125-134 admits a
    /// candidate object only from a straddled cel's chain; the ladder's link is its centre's cel
    /// [M `build_LinkToRoom` 0x1c9680 → build_FindCel → control_link_object_to_cel]]. A ladder
    /// whose cell is NOT in the pick capsule's straddle set is never admitted — even with the
    /// capsule dead on the rungs (the old unscoped pick grabbed through the cel wall, the
    /// 07000008 through-the-wall zipline arm). Sister: the same shape with the
    /// ladder's OWN cell in the set grabs, and an EMPTY set (no cel graph — unit tests, MP
    /// remotes) scopes nothing.
    #[test]
    fn the_ladder_pick_only_admits_ladders_of_the_straddled_cels() {
        let world = CollisionWorld::from_triangles(Vec::new());
        let mut lad = Ladder::simple([10.0, 3.0, 20.0], 2.0, 0.0);
        lad.cell = Some(3);
        let grab = |straddle: &[usize]| {
            let mut b = Bond::new([10.0, 3.0, 20.5], lad.yaw + std::f32::consts::PI);
            b.substate = SS_SWIM;
            b.tick_ctx(&world, &LadderCtx { ladders: &[lad.clone()], wires: &[], wire_anims: None, water: Some(2.0), sink: false, creepwalls: &[], level_bump_5fb: false, straddle, bot_capsules: &[] }, &Input { forward: 1.0, ..Default::default() });
            b.substate
        };
        // The ladder's cel NOT in the straddle set: no grab (the pick never admitted it).
        assert_eq!(grab(&[5]), SS_SWIM, "a ladder of a non-straddled cel grabbed — the scoping is off");
        // Sister that fails without the scoping: the same capsule dead on the rungs grabs the
        // moment the ladder's own cel is in the set.
        assert_eq!(grab(&[3]), SS_CLIMB, "the ladder's own cel in the set must grab");
        // The unresolved link (cell None) is the game's unlinked object: never admitted.
        let mut unlinked = lad.clone();
        unlinked.cell = None;
        let mut b2 = Bond::new([10.0, 3.0, 20.5], unlinked.yaw + std::f32::consts::PI);
        b2.substate = SS_SWIM;
        b2.tick_ctx(&world, &LadderCtx { ladders: &[unlinked], wires: &[], wire_anims: None, water: Some(2.0), sink: false, creepwalls: &[], level_bump_5fb: false, straddle: &[3], bot_capsules: &[] }, &Input { forward: 1.0, ..Default::default() });
        assert_eq!(b2.substate, SS_SWIM, "an unlinked ladder (cell None) must not grab while scoping is active");
        // The empty set scopes nothing (no cel graph): the grab goes through.
        assert_eq!(grab(&[]), SS_CLIMB, "the no-graph path must keep the unscoped pick");
    }

    /// fix-swim: the swim stroke rides the LOOK direction [M `Player_Move` 0x191dd0:149-165 —
    /// `RotMatrixX(obj+0x50 − BL+0x8a8·π/2)` + `Mat_GetDir`] and the case-3 tail clamps the rise at
    /// `surface − 0.3` [M 0x193600:201-220]. No crouch/jump input exists in swim [M case 3 reads
    /// none] — the pre-fix stand-in (planar-only motion, a made-up ±2.4/tick rise on crouch/jump)
    /// is gone; the assertions below pin the game's rule.
    #[test]
    fn swim_moves_along_the_look_direction_and_the_surface_clamps() {
        let world = CollisionWorld::from_triangles(Vec::new());
        let ctx = LadderCtx { ladders: &[], wires: &[], wire_anims: None, water: Some(3.0), sink: false, creepwalls: &[], level_bump_5fb: false, straddle: &[], bot_capsules: &[] };
        let mut b = Bond::new([0.0, 2.7, 0.0], 0.0); // below surface − 0.3? 2.7 < 2.7 is false — at the ceiling
        b.substate = SS_SWIM;
        // Look down-ish (pitch −0.5: dir = (0, −sin π/4, cos π/4)), swim forward: the position must
        // DROP and move along the look dir (the dive).
        b.pitch = -0.5;
        let y0 = b.pos[1];
        for _ in 0..60 { b.tick_ctx(&world, &ctx, &Input { forward: 1.0, ..Default::default() }); }
        assert!(b.pos[1] < y0 - 0.5, "looking down + forward did not dive (y {} -> {})", y0, b.pos[1]);
        assert!(b.pos[2] > 0.5, "the dive did not move forward along the look dir (z {})", b.pos[2]);
        // Look up hard, swim forward: the rise stops AT surface − 0.3 (the clamp), never past it.
        b.pitch = 1.0;
        for _ in 0..240 { b.tick_ctx(&world, &ctx, &Input { forward: 1.0, ..Default::default() }); }
        assert!(b.pos[1] <= 3.0 - 0.3 + 1e-3, "the swim rose past surface − 0.3: {}", b.pos[1]);
        assert!(b.pos[1] > y0 - 0.5, "looking up + forward did not surface (y {})", b.pos[1]);
        // Sister: crouch/jump do NOTHING in swim (the game reads no such input in case 3). Neutral
        // look so the look-dir vertical is out of the picture.
        b.pitch = 0.0;
        let y_at = b.pos[1];
        for _ in 0..30 { b.tick_ctx(&world, &ctx, &Input { jump: true, crouch: true, ..Default::default() }); }
        assert!((b.pos[1] - y_at).abs() < 1e-3, "crouch/jump moved the swimmer (y {} -> {})", y_at, b.pos[1]);
    }

    /// swim-height-02: the hittest capsule arms follow the switch's own table [M
    /// `Player_Collision` 0x194df0 second switch @0x195070, indexed substate−1, jump table
    /// @0x2fc0d0]: the SWIM 3 rides the DEFAULT arm (A = pos + up·0.275, B = pos + up·(0.55 − cc),
    /// r 0.55 — lowest point exactly cc under the root), and only the crouch substate 4 takes the
    /// LOW arm (case 3: A = pos + up·(0.1 − (cc − 0.55)), B = pos + up·(0.45 − cc)). Pre-fix the
    /// port gave the swim the LOW arm, hanging the lowest point cc + 0.10 under the root — the
    /// pond-bed push-out parked the swimmer ~0.1 high (the 02-pond 3.60/3.70 settles vs the PS2's
    /// 3.48/3.17 [L]). The grab pick's swim arm is the default one [M ladder.rs
    /// `capsule_hits_default`] — the pair must agree.
    #[test]
    fn swim_capsule_is_the_switch_default_arm_and_crouch_keeps_the_low_arm() {
        let mut b = Bond::new([0.0, 3.0, 0.0], 0.0);
        b.substate = SS_SWIM;
        let (a, c) = b.capsule();
        assert!((a[1] - 3.275).abs() < 1e-6, "the swim capsule's A rode the wrong arm: {}", a[1]);
        assert!((c[1] - (3.0 + 0.55 - b.cc)).abs() < 1e-6, "the swim capsule's B rode the wrong arm: {}", c[1]);
        // Sister: the crouch stance keeps the LOW arm (case 3 [M 0x195070 case 3]).
        let mut b4 = Bond::new([0.0, 3.0, 0.0], 0.0);
        b4.crouched = true;
        let (a4, c4) = b4.capsule();
        assert!((a4[1] - (3.0 + 0.1 - (b4.cc - 0.55))).abs() < 1e-6, "the crouch A left the LOW arm: {}", a4[1]);
        assert!((c4[1] - (3.0 + 0.450_000_02 - b4.cc)).abs() < 1e-6, "the crouch B left the LOW arm: {}", c4[1]);
        // Sister: the LOW arm's lowest point hangs exactly 0.1 deeper than the default's
        // (cc + 0.10 vs cc) — the pre-fix excess that rode the bed.
        let low = b4.cc - c4[1] - 0.55;
        let def = b.cc - c[1] - 0.55;
        assert!((low - def - 0.1).abs() < 1e-4, "the arm depth gap moved: {low} vs {def}");
    }

    /// fix-ladder2: the climb look [M Player_Climb 0x196310 + Player_Update 0x193600 + ViewClamping
    /// 0x1a92e8]. The turn input moves the yaw inside the eased ±90° window around `ladder.yaw + π`,
    /// and the pitch offset takes the stick (free, clamped ±1). Before the fix neither input reached
    /// the climb substate.
    #[test]
    fn climb_look_turns_within_the_ladder_window_and_pitch_is_free() {
        let world = CollisionWorld::from_triangles(Vec::new());
        let lad = Ladder::simple([10.0, 3.0, 20.0], 2.0, 0.0);
        let ctx = LadderCtx { ladders: &[lad.clone()], wires: &[], wire_anims: None, water: None, sink: false, creepwalls: &[], level_bump_5fb: false, straddle: &[], bot_capsules: &[] };
        let mut b = Bond::new([10.0, 3.0, 20.0 + 0.5], lad.yaw + std::f32::consts::PI);
        b.substate = SS_CLIMB;
        b.ladder = Some(0);
        b.climb_top = lad.top();
        b.climb_bottom = lad.bottom();
        // Turn left 0.05 rad/tick for a second: the yaw leaves the facing but stays within ±90°.
        let turn = Input { turn: 0.05, ..Default::default() };
        for _ in 0..60 { b.tick_ctx(&world, &ctx, &turn); }
        let target = lad.yaw + std::f32::consts::PI;
        let dev = wrap_pi(target - b.yaw).abs();
        // The equilibrium under a held turn is turn/0.2 (the ease opposes the turn [M 0x196310]) — 0.25.
        assert!(dev > 0.1, "the turn input never moved the yaw during the climb (deviation {dev}) — the camera is stuck");
        assert!(dev <= std::f32::consts::FRAC_PI_2 + 1e-3, "the yaw left the ±90° ladder window: {dev}");
        // Pitch: free, clamped ±1 [M ViewClamping 0x1a92e8].
        let look = Input { look: 0.05, ..Default::default() };
        for _ in 0..60 { b.tick_ctx(&world, &ctx, &look); }
        assert!(b.pitch > 0.3, "the pitch offset did not move during the climb ({})", b.pitch);
        for _ in 0..240 { b.tick_ctx(&world, &ctx, &look); }
        assert!(b.pitch <= 1.0 + 1e-4 && b.pitch >= -1.0 - 1e-4, "the pitch offset passed the ViewClamping clamp: {}", b.pitch);
        // Sister: without turn input the eased yaw converges to the ladder facing (the pre-existing ease).
        let mut b2 = Bond::new([10.0, 3.0, 20.0 + 0.5], lad.yaw + std::f32::consts::PI + 0.8);
        b2.substate = SS_CLIMB; b2.ladder = Some(0);
        b2.climb_top = lad.top(); b2.climb_bottom = lad.bottom();
        let idle = Input::default();
        for _ in 0..120 { b2.tick_ctx(&world, &ctx, &idle); }
        assert!(wrap_pi(target - b2.yaw).abs() < 0.1, "the ease no longer converges to the ladder facing");
    }

    /// `Bond::sphere_centre` vs the live 07000005 start [L ee_start_05.bin]: player obj 0x16cefe0, pos
    /// (104.954, 24.963, 108.825), yaw from the matrix (cos −0.5446, sin 0.8387 → forward (0.8387, 0,
    /// −0.5446)), `obj+0x80` = (104.826, 24.837, 108.582). `Control_BuildWorldSph` 0x133070 takes the
    /// centre from `M_CONTROL+0x80` through the object matrix [M].
    #[test]
    fn sphere_centre_matches_the_game_snapshot() {
        // yaw: sin = −0.8387, cos = −0.5446 — forward = Bond::look() = (sin, 0, cos) = matrix row 2 [L]
        // (rows [cos,0,−sin / 0,1,0 / sin,0,cos] read at 0x16cefe0+0x90..0xc8).
        let yaw = (-0.8387f32).atan2(-0.5446);
        let b = Bond::new([104.954, 24.963, 108.825], yaw);
        assert!((b.look()[0] - (-0.8387)).abs() < 1e-3 && (b.look()[2] - (-0.5446)).abs() < 1e-3, "forward matches the matrix row 2");
        let c = b.sphere_centre();
        for (k, want) in [(0, 104.826), (1, 24.837), (2, 108.582)] {
            assert!((c[k] - want).abs() < 2e-3, "axis {k}: got {} want {want}", c[k]);
        }
        // Sister: a different yaw moves the centre (the offset rotates with the body).
        let b2 = Bond::new([104.954, 24.963, 108.825], yaw + std::f32::consts::FRAC_PI_2);
        assert!((b2.sphere_centre()[0] - c[0]).abs() > 0.05, "the centre rotates with yaw");
    }

    fn room() -> CollisionWorld {
        let q = |a: V3, b: V3, c: V3, d: V3| [[a, b, c], [a, c, d]];
        let mut t = Vec::new();
        // All quads wound with the stored plane normal facing the occupant (floors UP — the disc's
        // convention, 7,246 up / 0 down; the wall −z at the walker; `Intersect_RayGeom` 0x10a8c8
        // skips dot(n,dir) >= 0).
        t.extend(q([-20.0, 0.0, -20.0], [-20.0, 0.0, 20.0], [20.0, 0.0, 20.0], [20.0, 0.0, -20.0]));
        // A wall at z = 5 and a 0.3 step at x ≥ 3 (top at y 0.3).
        t.extend(q([-20.0, 0.0, 5.0], [-20.0, 4.0, 5.0], [20.0, 4.0, 5.0], [20.0, 0.0, 5.0]));
        t.extend(q([3.0, 0.3, -20.0], [3.0, 0.3, 4.0], [20.0, 0.3, 4.0], [20.0, 0.3, -20.0]));
        t.extend(q([3.0, 0.0, -20.0], [3.0, 0.0, 4.0], [3.0, 0.3, 4.0], [3.0, 0.3, -20.0]));
        CollisionWorld::from_triangles(t)
    }

    #[test]
    fn walks_at_the_game_speed_stops_at_walls_and_climbs_steps() {
        let w = room();
        let mut b = Bond::stand_at(&w, [0.0, 0.0, 0.0], 0.0);
        let fwd = Input { forward: 1.0, ..Default::default() };
        for _ in 0..30 { b.tick(&w, &fwd); }
        // 0.1 a² + 0.005 = 0.105, clamped to 0.1 per tick once accelerated.
        assert!((b.local[1] - 0.1).abs() < 1e-6, "{:?}", b.local);
        for _ in 0..120 { b.tick(&w, &fwd); }
        assert!(b.pos[2] < 5.0 - RADIUS + 0.01 && b.pos[2] > 4.0, "held by the wall: {:?}", b.pos);
        assert!(b.grounded && (b.pos[1] - (ROOT_HEIGHT + 0.05)).abs() < 0.07, "standing: {:?}", b.pos);
        // Left (+x at yaw 0) onto the step: the capsule rides up.
        let left = Input { strafe: 1.0, ..Default::default() };
        let mut b = Bond::stand_at(&w, [0.0, 0.0, 0.0], 0.0);
        for _ in 0..120 { b.tick(&w, &left); }
        assert!(b.pos[0] > 4.0 && b.pos[1] > 0.3 + ROOT_HEIGHT - 0.05, "on the step: {:?}", b.pos);
        // Sister: stopping takes ~11 ticks, not one.
        for _ in 0..5 { b.tick(&w, &Input::default()); }
        assert!(b.local[0] != 0.0);
    }

    #[test]
    fn zero_g_thrusts_along_the_look_and_glides_when_the_thruster_stops() {
        let w = room();
        let mut b = Bond::stand_at(&w, [0.0, 0.0, -14.0], 0.0);
        b.substate = SS_ZEROG_9;
        b.pitch = 0.5; // the entry matrix takes the walk pitch [M ChangeSubState 9: no rotation]
        let entered_look = b.look();
        let start = b.pos;
        for _ in 0..60 { b.tick(&w, &Input { forward: 1.0, ..Default::default() }); }
        let moved = [b.pos[0] - start[0], b.pos[1] - start[1], b.pos[2] - start[2]];
        let n = (moved[0] * moved[0] + moved[1] * moved[1] + moved[2] * moved[2]).sqrt();
        assert!(n > 1.0, "moved along the look: {:?}", b.pos);
        // The move rides the matrix [M 0x193600:284-295]: parallel to the entry look (up +
        // forward at pitch 0.5), never straight up or straight ahead.
        let c = cross3(moved, entered_look);
        assert!(c[0].abs() < 1e-3 && c[1].abs() < 1e-3 && c[2].abs() < 1e-3,
            "parallel to the look: {:?} vs {:?}", moved, entered_look);
        assert!(moved[1] > 0.0 && moved[2] > 0.0);
        // The ±0.05 smoother cap [M 0x192ab0:110-112].
        let p0 = b.pos;
        b.tick(&w, &Input { forward: 1.0, ..Default::default() });
        let step = dist3_sq(b.pos, p0).sqrt();
        assert!(step <= 0.05 + 1e-4, "smoother cap: {}", step);
        // Inertia [M :92-135 — the 0.04 ease, no Player_Move dead-zone]: 30 released ticks later
        // Bond is STILL gliding (the walk ease snaps inside ~11); and decaying, not holding.
        for _ in 0..30 { b.tick(&w, &Input::default()); }
        let p1 = b.pos;
        b.tick(&w, &Input::default());
        let step2 = dist3_sq(b.pos, p1).sqrt();
        assert!(step2 > 0.01, "still gliding: {}", step2);
        assert!(step2 < step, "decaying: {} < {}", step2, step);
    }

    #[test]
    fn zero_g_vertical_thruster_rides_the_body_up_and_vel_stays_zero() {
        let w = room();
        let mut b = Bond::stand_at(&w, [0.0, 0.0, -14.0], 0.0);
        b.substate = SS_ZEROG_9;
        // Level attitude: the lift thruster rides the body's UP row [M 0x193600:284-295, the y
        // lane against row1] — straight up at pitch 0. The rise saturates at the ±0.05 smoother
        // cap [M 0x192ab0:110-112].
        for _ in 0..40 { b.tick(&w, &Input { lift: 1.0, ..Default::default() }); }
        assert!(b.pos[1] > 2.0, "risen: {:?}", b.pos);
        assert!(b.pos[0].abs() < 1e-4 && (b.pos[2] + 14.0).abs() < 1e-4,
            "no horizontal drift: {:?}", b.pos);
        // BL+0x50 zeroes every tick [M 0x192ab0:71] — hanging in the air, nothing integrates.
        assert_eq!(b.vel, [0.0; 3]);
        // Released, the lift smoother's own inertia keeps the drift decaying [M :92-159 — the
        // 0.04 ease, no dead-zone]: it settles, and then Bond HANGS (no gravity pulls him).
        for _ in 0..120 { b.tick(&w, &Input::default()); }
        let p = b.pos;
        b.tick(&w, &Input::default());
        assert!(dist3_sq(b.pos, p) < 1.0e-6, "settled: {:?} vs {:?}", b.pos, p);
        assert_eq!(b.vel, [0.0; 3]);
        // Pitched down, the same lift pushes along the TILTED body up (forward + up), not world
        // up — the matrix basis, not a scalar vertical.
        let mut b = Bond::stand_at(&w, [0.0, 0.0, -14.0], 0.0);
        b.substate = SS_ZEROG_9;
        b.pitch = -0.5;
        let start = b.pos;
        for _ in 0..40 { b.tick(&w, &Input { lift: 1.0, ..Default::default() }); }
        let moved = [b.pos[0] - start[0], b.pos[1] - start[1], b.pos[2] - start[2]];
        assert!(moved[2] > 0.5 && moved[1] > 0.5 && moved[0].abs() < 1e-3,
            "body-up with pitch: {:?}", moved);
    }

    #[test]
    fn zero_g_stabiliser_slerps_world_up_for_its_window_then_stops() {
        let w = room();
        let mut b = Bond::stand_at(&w, [0.0, 0.0, -14.0], 0.0);
        b.substate = SS_ZEROG_9;
        b.pitch = 0.5;
        b.tick(&w, &Input::default()); // the entry matrix is live now
        let heading = b.yaw;
        let f0 = b.look()[1];
        // Sister: without the Action read the attitude holds [M :163-168 — the window only arms
        // on it].
        for _ in 0..30 { b.tick(&w, &Input::default()); }
        assert!((b.look()[1] - f0).abs() < 1e-5,
            "no drift without the press: {}", b.look()[1]);
        // One press arms 90 ticks [M :165]; the slerp parameter is REC_FRAME_RATE·3.0 = 0.05 per
        // tick [M :196], so the pitch angle decays ×0.95 exactly (same-axis slerp) and the
        // heading is preserved [M Mat_Align2Up 0x1df4b0].
        b.tick(&w, &Input { stabilise: true, ..Default::default() });
        for k in 1..=89 {
            b.tick(&w, &Input::default());
            let want = (0.5f32 * std::f32::consts::FRAC_PI_2 * 0.95f32.powi(k as i32 + 1)).sin();
            assert!((b.look()[1] - want).abs() < 0.003,
                "slerp tick {}: {} vs {}", k, b.look()[1], want);
            assert!((b.yaw - heading).abs() < 1e-4, "heading kept: {}", b.yaw);
        }
        let fy = b.look()[1];
        for _ in 0..20 { b.tick(&w, &Input::default()); }
        assert!((b.look()[1] - fy).abs() < 1e-5,
            "the 90-tick window expired: {} vs {}", b.look()[1], fy);
    }

    #[test]
    fn zero_g_refills_air_even_underwater_like_the_game() {
        let w = room();
        let ctx = LadderCtx { water: Some(2.0), ..Default::default() }; // the head sits under 2.0
        let mut b = Bond::stand_at(&w, [0.0, 0.0, -14.0], 0.0);
        b.air = 50.0;
        b.substate = SS_ZEROG_9;
        b.tick_ctx(&w, &ctx, &Input::default());
        assert!((b.air - (50.0 + 20.0 / 60.0)).abs() < 1e-4, "refill in zero-g: {}", b.air);
        // Sister: the walk substate still drains — the gate is the substate, not the suit [M
        // Player_MonitorAir 0x191a70:57, :98].
        let mut b = Bond::stand_at(&w, [0.0, 0.0, -14.0], 0.0);
        b.air = 50.0;
        b.tick_ctx(&w, &ctx, &Input::default());
        assert!((b.air - (50.0 - 5.0 / 60.0)).abs() < 1e-4, "walk drains: {}", b.air);
    }

    #[test]
    fn walk_ignores_the_zero_g_lift_and_stabilise_bits() {
        let w = room();
        let mut b = Bond::stand_at(&w, [0.0, 0.0, -14.0], 0.0);
        for _ in 0..30 { b.tick(&w, &Input { lift: 1.0, stabilise: true, ..Default::default() }); }
        assert_eq!(b.substate, SS_WALK);
        assert!(b.grounded && b.pos[1] < 1.2, "no lift on foot: {:?}", b.pos);
    }

    #[test]
    fn jumps_and_falls_with_gravity() {
        let w = room();
        let mut b = Bond::stand_at(&w, [0.0, 0.0, 0.0], 0.0);
        // The spawn arrival settle runs first (the `settle_hold` doc): the boot holds the placed
        // root four ticks, then drops it into the band — the jump test measures from the REST.
        b.arm_spawn_settle();
        for _ in 0..8 { b.tick(&w, &Input::default()); }
        let y0 = b.pos[1];
        b.tick(&w, &Input { jump: true, ..Default::default() });
        let mut top = y0;
        for _ in 0..60 { b.tick(&w, &Input::default()); top = top.max(b.pos[1]); }
        // 3.92 m/s up against 9.8: ~0.78 high.
        assert!((top - y0 - 0.78).abs() < 0.08, "apex {}", top - y0);
        assert!(b.jump == 0 && (b.pos[1] - y0).abs() < 0.1);
        // Off a ledge: no floor → falls.
        let mut b = Bond::new([0.0, 30.0, 0.0], 0.0);
        for _ in 0..30 { b.tick(&w, &Input::default()); }
        assert!(b.pos[1] < 30.0 - 1.0 && !b.grounded);
    }
}

/// unported-front-end-menus-3: the game's `CAM_ORBIT` (BLData+0xe0, 0x24 bytes) — the orbit
/// camera state `Camera_Orbit` 0x1bb828 advances [M]. `Player_PositionCamera` 0x18f5d8:135-144
/// (cam_mode 1) refreshes `look` from the head bone each tick (`AnimGetBoneWorldTrans(obj, −5,
/// 0, BLData+0xe0, 0)`) and stores the obj at BLData+0x100; `Player_SetCamMode` 0x195928:17-24
/// initializes max = 1.5, radius = 0.15, target = 1.5, phase = 0 (case 1: +0xf0 = 0x3fc00000,
/// +0xf4 = 0x3e19999a, +0xf8 = 0x3fc00000, +0xfc = 0).
#[derive(Clone, Debug, PartialEq)]
pub struct OrbitCam {
    /// `CAM_ORBIT+0x00..+0x0c`: the look-at point (the holder's head-bone position).
    pub look: V3,
    /// `CAM_ORBIT+0x10`: the maximum orbit radius (init 1.5 [M 0x195928:18]).
    pub max: f32,
    /// `CAM_ORBIT+0x14`: the current orbit radius (init 0.15 [M 0x195928:19]).
    pub radius: f32,
    /// `CAM_ORBIT+0x18`: the target radius the current one eases toward (init 1.5 [M 0x195928:20]).
    pub target: f32,
    /// `CAM_ORBIT+0x1c`: the orbit phase, radians (init 0 [M 0x195928:21]).
    pub phase: f32,
}

impl Bond {
    /// unported-front-end-menus-3: `Player_SetCamMode(BL, 1)` 0x195928 case 1 [M] — the only
    /// mode-1 writer in the game is `MP_GoldenEyeUpdate` 0x187568:396-398 (the GoldenEye
    /// strike's orbit on its class-3 holder). Sets the port's `cam_mode` and stamps the
    /// CAM_ORBIT init the game's case-1 arm writes (HUD_Reset + viewer+0x236 = 0 are HUD
    /// state the port does not draw).
    pub fn set_cam_mode_orbit(&mut self) {
        self.cam_mode = 1;
        self.orbit = Some(OrbitCam {
            look: self.head(),
            max: 1.5,
            radius: 0.15,
            target: 1.5,
            phase: 0.0,
        });
    }

    /// unported-front-end-menus-3: `Camera_Orbit` 0x1bb828 [M] — one per-frame orbit step,
    /// run by the game from `Camera_Update` 0x1bb6d8:21 → `Player_PositionCamera` 0x18f5d8
    /// case 1 while cam_mode is 1. Order per the decompile: the sines and the eye use the
    /// CURRENT phase and radius (:29-52), then the phase advances (:45), then the eye's
    /// sphere pull sets the target and the radius eases toward it without overshoot
    /// (:63-82). `sphere_hit` is the eye sphere's closest hit distance — the game's
    /// `Collide_SphereIntersect(1.5, viewer+0x150, cel, obj, …)` HITDATA+0xc (:63-70; None =
    /// no hit → target = max). Returns `(eye, look, zoom)` — the eye computed as x = look.x +
    /// sin(φ)·r, z = look.z + cos(φ)·r, y = look.y + (0.75 when r < 0.75, else min(r, max))
    /// [M :35-52]; the game `Mat_LookAt`s eye → look into the view matrix — the draw side's
    /// job here, so the port hands it both endpoints.
    pub fn orbit_camera(&mut self, sphere_hit: Option<f32>) -> Option<([f32; 3], [f32; 3], f32)> {
        let o = self.orbit.as_mut()?;
        // Eye from the CURRENT phase + radius: x = look.x + sin(φ)·r, z = look.z + cos(φ)·r [M
        // 0x1bb828:52-58 (PS2Sinf2(phase+π/2, phase) → cos φ @fStack_50 / sin φ @fStack_4c)].
        // y = look.y + 0.75, or look.y + min(r, max) when that sits at/above the 0.75 floor [M
        // 0x1bb828:40-52 (fStack_5c = look.y + 0.75; if it ≤ look.y + r → min(look.y + r,
        // look.y + +0x10))].
        let (s, c) = (crate::trig::sinf(o.phase), crate::trig::cosf(o.phase));
        let eye_y = if 0.75 <= o.radius {
            (o.look[1] + o.radius).min(o.look[1] + o.max)
        } else {
            o.look[1] + 0.75
        };
        let eye = [o.look[0] + s * o.radius, eye_y, o.look[2] + c * o.radius];
        // Phase advance [M 0x1bb828:45: +0x1c += REC_FRAME_RATE · 0.7].
        o.phase += crate::grenade::REC_FRAME_RATE * 0.7;
        // Sphere pull [M 0x1bb828:72-78]: no hit → target = max (+0x10); hit → target = the
        // hit's HITDATA+0xc floored at 0.1 (0x3dcccccd).
        o.target = match sphere_hit {
            None => o.max,
            Some(h) => h.max(0.1),
        };
        // Radius ease, no overshoot [M 0x1bb828:79-82: radius += (target − radius) ·
        // REC_FRAME_RATE · 5, clamped at the target from both sides].
        let was = o.radius;
        o.radius += (o.target - was) * crate::grenade::REC_FRAME_RATE * 5.0;
        if (o.target >= was && o.radius > o.target) || (o.target < was && o.radius < o.target) {
            o.radius = o.target;
        }
        Some((eye, o.look, 1.0)) // zoom 1.0: Player_PositionCamera case 1 sets viewer+0x108 = 0x3f800000 [M 0x18f5d8:136]
    }
}
