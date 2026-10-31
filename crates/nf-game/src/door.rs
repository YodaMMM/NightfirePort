// unported-misc-5 — the game's generic list containers are Rust collections here:
// `LList_Init` 0x1cc958 (the door list's init; its other callers `Door_Init` 0x136848,
// `control_init_object_lists` 0x134388, `Coll_ResetHitHeap` 0x1edda0, `Sensor_Init` 0x1b0868,
// `Drone_PreLoad_Init` 0x136888, `Copter_Init` 0x17e4f0, `InitShards` 0x12bed0,
// `Creature_Init` 0x135208) is this module's owned `Vec` — the game's LLISTINFO/LLNODE
// linked list with the same insert-at-head/order semantics, without the manual nodes.
//! Doors (`docs/systems/world-objects.md` "Doors"): `Door_Create` 0x136100, `Door_SetupSwing` 0x135440,
//! `Door_SetState` 0x135618, `Door_Interp` 0x135738, `Door_ReactToHit` 0x135980, `Door_Activate` 0x135be0,
//! `Door_DoMsg` 0x135f18, `Doors_Calc` 0x135fe0, `Door_Update` 0x1363c8 — ported from the disassembly [M] and
//! checked against the live layout in `work/ee_start_05.bin` [L] (5 doors on 07000005: 4 `Door_Locked_3/4`
//! sliders without an anim and lock channel 200, 1 `Door_UNLocked` swing door with a 3-key anim).
//!
//! The text labels 0x2000003 / 0x200000b are ported (`Door_Update`'s unlock note on the lock channel's
//! rising edge, `Door_DoMsg`'s aim hint — door-text-labels, a556ce10).
//!
//! The collide path (fix-door-react-path, G-035): `Door_ReactToHit` is the class-0x1C COLLIDE function
//! (`control_funcs[0x1C·0xC+4]` @0x26f9e0 [ELF, trigger-port-check #3]), dispatched by `Collide_Update`
//! 0x1ea398:59-64 after the control walk + movement pass (docs/atlas/object.md step 7). The contacts it
//! reads are the door's `obj+0xD0` hit list, which `Collide_Update` fills by forwarding each picker's
//! `Collide_PickObj` hits to the hit object (+0x58 = the hitter) [M 0x1ea398:160-233]; the doors are picked
//! because `build_LinkToRoom` 0x1c9680:63-66 puts `obj+0xF0 & 0x20000000` objects on the global `ForcedList`
//! (0x270590), which every picker with `HITTEST+0x98` without bit 0x10 tests [M `Collide_Pick` 0x1eb320:369-375].
//! The player's pick is mode 0x800 [M `Player_Collision` 0x194df0 writes BLData+0x7D4 = 0x800 in the
//! substate arms]: the segment-vs-sphere test — `sqdist(live obj+0x80 sphere centre, closest point on the
//! pick capsule) < (1.5·r_obj + r_pick)²` [M `Collide_PickObj` 0x1eae80:152-168, the ×1.5 at :160].
//! Ported as `Doors::collide_step` (called from the collide caller in `Drones::tick`, before `Doors_Calc`) —
//! the sphere stand-in in the old `react_to_hit` is gone: the pick above IS the game's test, and
//! `react_to_hit` keeps only the game's reversal arms — a closed idle door falls through every
//! branch (kind 0 needs `obj+0xf4 & 0x18`, kind 2 `& 0x10` [M 0x135980:77, 101-104]).

use crate::triggers::Channels;
use crate::trig::sinf;
use crate::world::CollisionWorld;
use nf_formats::placement::{view_rot_trans_matrix, Placement};

pub type V3 = [f32; 3];
/// Row-vector 4×4 (translation in row 3) — the same layout as the game's `MATRIX` at `obj+0x90` [L].
pub type Mat4 = [[f32; 4]; 4];

/// `FRAME_RATE` (0x30d0d0) while the level is being built: the doors are created at load and the fallback speed
/// 1/30 measures 0.03333 in the snapshot [L], although the global reads 60.0 during play (the video rate is set
/// after loading; the auto-close below uses the play value).
const CREATE_FRAME_RATE: f32 = 30.0;
/// `FRAME_RATE_INT` (60 during play) × 6: ticks an open door waits before it closes on its own.
const AUTO_CLOSE: u32 = 360;
/// `Door_Activate`'s wrong-side margin (0x3e99999a).
const MARGIN: f32 = 0.3;
/// `Door_ReactToHit`'s reversal threshold (ABS compares against 0.0002).
const HIT_EPS: f32 = 0.0002;

/// States (`obj+0xF4`): 2 closed, 4 open, 8 opening (pos moves away from closed), 0x10 closing.
pub const CLOSED: u16 = 2;
pub const OPEN: u16 = 4;
pub const OPENING: u16 = 8;
pub const CLOSING: u16 = 0x10;

/// `Txt_BindLabel(0x2000003, 0)` — "Door Unlocked" [D USATxt.dat]: `Door_Update` 0x1363c8's lock-channel
/// rising-edge message, `Text_AddMsg(0, 0, 1, …, 0, 0xb4)` — the type-1 INFO bar, 180 ticks — before the
/// unlock `Door_Activate` [M 0x1363c8 decompile lines 101-107; the level gate 0x7000014/0x700004a is
/// the caller's (Doors does not know the level)].
pub const LABEL_DOOR_UNLOCKED: u32 = 0x2000003;
/// `Txt_BindLabel(0x200000b, 0)` — "Press Action to open door" [D USATxt.dat]: `Door_DoMsg` 0x135f18's
/// aim-ray hint via `Check_Target` 0x1a7810 (once per door, `DoorMsg` 0x30cd04 < 3 per boot, dist < 5.0)
/// [M 0x1a79e8-0x1a7bc8 disasm, 0x135f18].
pub const LABEL_DOOR_HINT: u32 = 0x200000b;

/// One key of a door anim (a 0x19 block payload): 28 bytes — `f32[3]` position at +0, quaternion (x, y, z, w)
/// at +0xC [M: `Spline_Interp` 0x1e1858 addresses keys at `list + 0x1c·k`; L: `Door_UNLocked`'s 3 keys].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Key { pub pos: V3, pub quat: [f32; 4] }

pub fn parse_keys(payload: &[u8]) -> Vec<Key> {
    (0..payload.len() / 28).map(|k| {
        let p = &payload[k * 28..];
        let w = |i: usize| f32::from_le_bytes(p[i..i + 4].try_into().unwrap());
        Key { pos: [w(0), w(4), w(8)], quat: [w(0xc), w(0x10), w(0x14), w(0x18)] }
    }).collect()
}

fn dot3(a: V3, b: V3) -> f32 { a[0] * b[0] + a[1] * b[1] + a[2] * b[2] }
fn sub(a: V3, b: V3) -> V3 { [a[0] - b[0], a[1] - b[1], a[2] - b[2]] }
fn cross3(a: V3, b: V3) -> V3 { [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]] }

/// `Mat_GetDir` (0x1e3398): row 2. `Mat_GetNorm` (0x1e33d8): row 0.
fn row(m: &Mat4, r: usize) -> V3 { [m[r][0], m[r][1], m[r][2]] }

/// Row-vector `v·M` (`RotPreTransVec`'s product with an inverse pose).
fn transform(m: &Mat4, v: V3) -> V3 {
    let mut o = [m[3][0], m[3][1], m[3][2]];
    for k in 0..3 { for c in 0..3 { o[c] += v[k] * m[k][c]; } }
    o
}

/// Inverse of a rigid matrix (transposed rotation, `−t·Rᵀ`) — what `Mat_Inverse` computes for the position +
/// quaternion poses. Anim-less doors never use it (the game inverts whatever `Spline_Interp` left, which is
/// nothing: it returns on a NULL framelist).
fn rigid_inverse(m: &Mat4) -> Mat4 {
    let mut o = [[0.0f32; 4]; 4];
    for i in 0..3 { for j in 0..3 { o[i][j] = m[j][i]; } }
    for c in 0..3 { o[3][c] = -(m[3][0] * o[0][c] + m[3][1] * o[1][c] + m[3][2] * o[2][c]); }
    o[3][3] = 1.0;
    o
}

/// Quaternion → matrix (`Spline_Interp`'s tail): `m00 = 1−(2y·y+2z·z)` and so on, translation in row 3 [L].
pub(crate) fn quat_matrix(q: [f32; 4], t: V3) -> Mat4 {
    let (x, y, z, w) = (q[0], q[1], q[2], q[3]);
    let (yy, zz, xx) = (y + y, z + z, x + x);
    [
        [1.0 - (yy * y + zz * z), yy * x + zz * w, zz * x - yy * w, 0.0],
        [yy * x - zz * w, 1.0 - (xx * x + zz * z), zz * y + xx * w, 0.0],
        [zz * x + yy * w, zz * y - xx * w, 1.0 - (xx * x + yy * y), 0.0],
        [t[0], t[1], t[2], 1.0],
    ]
}

/// `Spline_Interp`'s slerp: dot, shortest path, `PS2Sinf` weights while `1−|dot| > 0.01`, else the unnormalised
/// linear blend.
fn slerp(a: [f32; 4], b: [f32; 4], t: f32) -> [f32; 4] {
    let mut dot = a[0] * b[0] + a[1] * b[1] + a[2] * b[2] + a[3] * b[3];
    let neg = dot < 0.0;
    if neg { dot = -dot; }
    let (w0, w1) = if 0.01 < 1.0 - dot {
        // G-046: Spline_Interp 0x1e1858:285-300 [M] — acosf (0x232c48), wt = angle·t (mul.s), the weights
        // PS2Sinf(wt)·(1/PS2Sinf(angle)) with the EE sub in sinf(angle − wt).
        let angle = crate::trig::acosf_libm(dot);
        let wt = nf_formats::ps2float::mul(angle, t);
        let recip = nf_formats::ps2float::div(1.0, sinf(angle));
        (sinf(crate::trig::ee_sub(angle, wt)) * recip, sinf(wt) * recip)
    } else { (1.0 - t, t) };
    let w0 = if neg { -w0 } else { w0 };
    [w0 * a[0] + w1 * b[0], w0 * a[1] + w1 * b[1], w0 * a[2] + w1 * b[2], w0 * a[3] + w1 * b[3]]
}

/// Catmull-Rom at `t` (`Spline_Interp`'s position formula, in the game's operation order).
fn catmull(p0: V3, p1: V3, p2: V3, p3: V3, t: f32) -> V3 {
    let mut o = [0.0f32; 3];
    for k in 0..3 {
        let (a, b, c, d) = (p0[k], p1[k], p2[k], p3[k]);
        o[k] = (b + b + t * ((c - (t - 1.0) * (t - 1.0) * a)
            + t * (((b * -5.0 + t * b * 3.0 + c * 4.0) - t * c * 3.0) + (t - 1.0) * d))) * 0.5;
    }
    o
}

/// One interpolation of a door anim [M: `KeyFrame_Interp` 0x1e1140, `Spline_Interp` 0x1e1858]. `time` in key
/// units (the caller passes `pos01 · frames`), clamped to `[first, last]`; `k = floor(time)` clamped, `t` the
/// fraction; neighbour keys clamp into range. Keyframe mode lerps the position (clamped to the segment's far
/// end); spline mode runs Catmull-Rom (all four positions equal takes `k`'s directly). Both slerp the rotation
/// between keys `k` and `k+1` (equal quaternions pass through). `None` = a NULL framelist: nothing is written.
pub fn interp(keys: &[Key], first: u16, last: u16, time: f32, spline: bool) -> Option<Mat4> {
    if keys.is_empty() || first >= last { return None; }
    let (lo, hi) = (first as i32, last as i32);
    let time = time.max(lo as f32).min(hi as f32);
    let k = (time as i32).clamp(lo, hi);
    let t = time - k as f32;
    let key = |i: i32| &keys[(i.clamp(lo, hi) - lo) as usize];
    let (pk, pk1) = (key(k), key(k + 1));
    let pos = if spline {
        let (p0, p3) = (key(k - 1).pos, key(k + 2).pos);
        if p0 == pk.pos && pk.pos == pk1.pos && pk1.pos == p3 { pk.pos } else { catmull(p0, pk.pos, pk1.pos, p3, t) }
    } else {
        let mut o = [pk.pos[0] + (pk1.pos[0] - pk.pos[0]) * t, pk.pos[1] + (pk1.pos[1] - pk.pos[1]) * t, pk.pos[2] + (pk1.pos[2] - pk.pos[2]) * t];
        for c in 0..3 {
            let over = if pk1.pos[c] >= pk.pos[c] { pk1.pos[c] < o[c] } else { o[c] < pk1.pos[c] };
            if over { o[c] = pk1.pos[c]; }
        }
        o
    };
    let quat = if pk.quat == pk1.quat { pk.quat } else { slerp(pk.quat, pk1.quat, t) };
    Some(quat_matrix(quat, pos))
}

/// One door: the `obj_tag` fields the port keeps plus the 0x80-byte `DOORINFO` instance at `obj+0xE0`.
#[derive(Clone, Debug)]
pub struct Door {
    /// Index into the main map's placements.
    pub placement: usize,
    /// `obj+0xF6` (parameter 0): 0 swing, 1–2 slide, other = activation marker only.
    pub kind: i16,
    /// `+0x6A` (p1): door group — `Doors_Calc` slaves the group to its first non-busy member.
    pub group: u16,
    /// `+0x66` (p2): the open switch channel — on: open; off and not closed: close at once (no timer).
    pub ch_open: u16,
    /// `+0x68` (p3): the lock channel — off and not closed, activation only plays the hit sound; its rising
    /// edge unlocks the door for good (the door clears the field).
    pub ch_lock: u16,
    /// `obj+0xFD bit 3` (p4): link to a collision portal (block 0x21 — not ported).
    pub straddle: bool,
    /// `+0x60` (p5): entering state 8. `+0x62` (p6): entering 0x10. `+0x64` (p7): the locked/hit feedback.
    pub snd_open: u16, pub snd_close: u16, pub snd_hit: u16,
    /// `+0x70` (p8|p9): bit 0 spline (else keyframe) interpolation, bit 2 "starts open" (+ open channel forced on
    /// at create, `Door_Create` :96-99), bit 4 the busy marker, bit 5 keep open.
    pub flags: u16,
    /// `+0x76` (p10): 1 = activate from the front only, 2 = from behind only (one-shot; 0 = either side).
    pub approach: u8,
    /// `+0x77` (p11): never player-activated.
    pub locked_flag: u8,
    /// `+0xC`: the anim (a 0x19 key list). `None` = no anim: the door never visually moves.
    pub keys: Option<Vec<Key>>,
    /// `+0x6E`: the last key index (the record's count − 1; 0xFFFF without an anim).
    pub frames: u16,
    /// `obj+0xF4`.
    pub state: u16,
    /// `+0x50`: 0..1 along the anim; a swing door's closed position is 0.5.
    pub pos01: f32,
    /// `+0x54`: 1/rate (rate 0 → 1/30 at create [L]); `Door_SetupSwing` re-quantises a swing door to whole ticks.
    pub speed: f32,
    /// `+0x58` / `+0x5C`: the travel direction and the swing sign (`Door_SetupSwing`'s dot product).
    pub dir: f32, pub swing: f32,
    /// `+0x6C`: ticks since the last activation (`Door_ReactToHit` waits it out). `+0x72`: the auto-close timer.
    pub timer: i16, pub open_timer: u16,
    /// `+0x74`: `Door_DoMsg` shows its label once (no text system; the flag still latches).
    pub msg_shown: bool,
    /// `obj+0x90`: the drawn and collided pose. `+0x10` (swing): the inverse closed pose for the side tests.
    pub matrix: Mat4, pub inv_closed: Mat4,
    /// `obj+0x30` (the sound position `obj+0xC0` is the same point) and the sphere `obj+0x80/+0x8C`.
    pub pos: V3, pub sph: (V3, f32),
    /// The gfx sphere centre in the placement's local frame — transformed by the LIVE `matrix` each tick it gives
    /// `obj+0x80` (the moved pass rebuilds the world sphere after `Door_Interp` sets `obj+0xFA & 0x20` [M
    /// 0x135738:28, 0x1336f8:209]), so the auto-close blocking zone follows the open door leaf.
    pub sph_centre: V3,
    /// The object's hashcode (the drawn mesh and the collision shape).
    pub glist: u32,
}

impl Door {
    /// `Door_IsClosed` (0x1367b0) [M]: a kind-1 door (`obj+0xF6`) always reads closed; a grouped
    /// door (ext+0x6A != 0) reads `DoorGroupsStates[g] == 2` (the `Doors_Calc` OR); the rest read
    /// `obj+0xF4 == CLOSED` exactly — OPENING/CLOSING leaves do not block. `group_states` is
    /// `Doors::group_states` (the game's 0xCA-byte `DoorGroupsStates`); a group past the table
    /// reads open (the game would read past its own table — none exists on our levels).
    pub fn is_closed(&self, group_states: &[u16]) -> bool {
        if self.kind == 1 { return true; }
        if self.group != 0 {
            return group_states.get(self.group as usize).copied().unwrap_or(0) == CLOSED;
        }
        self.state == CLOSED
    }

    /// The pose at `pos01` (`Door_Interp`'s write; the create path always takes the spline mode).
    fn pose_at(&self, pos01: f32, spline: bool) -> Option<Mat4> {
        self.keys.as_ref().and_then(|k| interp(k, 0, self.frames, pos01 * self.frames as f32, spline))
    }
    /// `Door_SetupSwing`'s closed pose: `Spline_Interp(0.5 · frames)`.
    fn closed_pose(&self) -> Option<Mat4> { self.pose_at(0.5, true) }

    /// The LIVE world sphere (`obj+0x80`/`obj+0x8C`): the gfx centre transformed by this tick's matrix —
    /// the same sphere the auto-close scan uses.
    pub fn live_sph(&self) -> (V3, f32) { (transform(&self.matrix, self.sph_centre), self.sph.1) }
}

/// All doors of a level, in placement (creation) order.
#[derive(Clone, Debug)]
/// `Door_Init` 0x136848 (60 B, from `ResetMap_GameInit` 0x1d5080:12-19 at 0x1d5184):
/// `LList_Init(TheDoors 0x316f70, 0x80, 0)` + `memset(DoorGroupsStates 0x316ea0, 0, 0xca)` — the
/// per-map empty of the door list and the 101×u16 group-state table. The port's `Doors::create`
/// per level attach (drone.rs `attach_level_id`) rebuilds the list and zeroes `group_states` —
/// this reset; the per-FRAME wipe of `DoorGroupsStates` is `Doors::begin_pass` below.
pub struct Doors {
    pub list: Vec<Door>,
    /// `DoorGroupsStates` (0xca bytes — `Door_Init` 0x136848 zeroes it per map, `Doors_Calc`
    /// 0x135fe0:13 wipes it per frame via `begin_pass`): per group, the OR of the door states.
    pub group_states: [u16; 101],
    /// The previous tick's `switch_channels` (`switch_channels_prev`) for the lock channel's rising edge.
    prev_channels: [u8; 256],
    hold_channels: [u8; 256],
    /// door-text-labels: the 0x2000003 "Door Unlocked" raises this tick — `(label, ticks)` pairs queued by
    /// `update` on the lock channel's rising edge, drained by Sim into the type-1 INFO queue
    /// (`Text_AddMsg(0, 0, 1, Txt_BindLabel(0x2000003, 0), 0, 0xb4)` [M 0x1363c8:101-107]).
    pub labels: Vec<(u32, f32)>,
    /// Per door, this tick's `obj+0xD0` contacts (the collide step's forwards). Cleared by `collide_step`.
    contacts: Vec<Vec<DoorContact>>,
}

impl Default for Doors {
    fn default() -> Doors {
        Doors {
            list: Vec::new(),
            group_states: [0; 101],
            prev_channels: [0; 256],
            hold_channels: [0; 256],
            labels: Vec::new(),
            contacts: Vec::new(),
        }
    }
}

/// The main map's door placements in creation order (create and nf-view).
pub fn door_placements(placements: &[Placement]) -> Vec<usize> {
    placements.iter().enumerate().filter(|(_, p)| p.flags & 0xff == 0xdb && p.flags & 0xa000 == 0).map(|(i, _)| i).collect()
}

/// `Door_Update`'s proximity test [M 0x1363c8:52-68, 121-137]: `Vec_SqDist3D(door obj+0x80, actor obj+0x80) <
/// 2·(r_door + r_actor)²` with the `obj+0x8C` radii. `obj+0x80` is the door's LIVE world sphere (rebuilt by the
/// moved pass after every `Door_Interp`), so a slide door's blocking zone follows the open leaf. The old
/// fixed-placement + 1.0-radius stand-in is DELETED (it closed doors the game leaves open — core-loop §7
/// item 4 / door-port-check #1): `blocked` IS the game's scan, verified line-for-line against
/// 0x1363c8:52-68 (the class-2/3/0x12 walk, the `+0xf0 & 0x10` skip) and :121-137 (the auto-close arm) [M].
fn blocked(door: (V3, f32), a: (V3, f32)) -> bool {
    let d = [door.0[0] - a.0[0], door.0[1] - a.0[1], door.0[2] - a.0[2]];
    let r = door.1 + a.1;
    d[0] * d[0] + d[1] * d[1] + d[2] * d[2] < r * r + r * r
}

/// A body the door's proximity scans see: position, bounding radius, and whether its `obj+0xF0` flags carry bit
/// 0x10 — the scan skips such objects entirely [M 0x136678-0x136684; the 07000001 player (0x890) and drones
/// (0x8B0) have it set [L], the 07000005 player (0x880) does not [L]. The writer of that bit is not ported, so
/// the port models it as clear (the 07000005 evidence; door-port-check #2).
pub type DoorActor = (V3, f32, bool);

/// The scan's candidates: every actor the `obj+0xF0 & 0x10` filter lets through [M 0x136678-0x136684].
fn visible_actors(actors: &[DoorActor]) -> impl Iterator<Item = (V3, f32)> + '_ {
    actors.iter().filter(|a| !a.2).map(|a| (a.0, a.1))
}

/// A sound `Door_SetState` / `Door_Activate` asks for: id and position.
pub type SoundEvent = (u16, V3);

/// One picker in the door collide step (a `Collide_Update` mover whose pick reaches the doors). The
/// fields are the HITTEST pieces the door path reads: the pick capsule `HITTEST+0x20/+0x30` (the
/// per-substate vertical segment at the root [M `Player_Collision` 0x194df0's switch: walk 0/6
/// A = pos+up·0.55, B = pos+up·(0.275−cc), r = 0.275; swim 3 A = pos+up·(0.1−(cc−0.55)),
/// B = pos+up·(0.45−cc), r = 0.55; default A = pos+up·0.275, B = pos+up·(0.55−cc), r = 0.55 —
/// the port's `Ladder::pick_hit` arms]), the pick radius `HITTEST+0x8C`, and the world sphere
/// centre `obj+0x80` the kind-0 reversal transforms [M 0x135980:80].
#[derive(Clone, Copy, Debug)]
pub struct DoorMover {
    pub a: V3,
    pub b: V3,
    pub radius: f32,
    pub centre: V3,
    /// class 3 — the player: the contact selection takes it unconditionally [M 0x135980:65].
    pub player: bool,
    /// The class-2 gate's verdict [M 0x135980:53-60]: `(ext+0x4F8 & 0x600) == 0 && (ext+0x4F8 & 0x100)
    /// != 0 && ext+0xAC > 0.0 && !(obj+0xFE & 1)`. The port maps it to present ∧ non-corpse ∧ health > 0
    /// (no `+0x4F8` in the port; the 0x600 bits are death/stun states, 0x100 active — live checks #3).
    pub valid: bool,
}

/// One `obj+0xD0` contact: the copy `Collide_Update` appends to the hit object, `+0x58` = the hitter,
/// of whose world sphere centre the door's kind-0 reversal reads [M 0x1ea398:176-193, 0x135980:80].
/// The game builds that hit list with the collision LList helpers — `LList_AllocnNodes` 0x1ccbe0
/// allocates the nodes and `LList_Cut` 0x1cca80 moves the filled list onto the object (both called
/// by `Collide_Update` 0x1ea398 / `Collide_Pick` 0x1eb320 [M]); the port's contact Vec append is
/// that same list work.
#[derive(Clone, Copy, Debug)]
struct DoorContact { centre: V3, player: bool, valid: bool }

impl Doors {
    /// The per-door `Door_IsClosed` (0x1367b0) snapshot, indexed by `list` position — what the cel
    /// walk consumes each frame through the portal links (`Vision_Recurse` 0x1e82e0 reads
    /// `Door_IsClosed(portal+0xC)`; fix-door-portal-links).
    pub fn closed_flags(&self) -> Vec<bool> {
        self.list.iter().map(|d| d.is_closed(&self.group_states)).collect()
    }

    /// `Door_Create` over the main map's placements. `path_blocks` is the level's 0x19 payloads (its
    /// `path_blocks()`); `object` resolves a placement to `(glist hashcode, gfx sphere centre, radius)`.
    pub fn create(
        placements: &[Placement],
        path_blocks: &[&[u8]],
        mut object: impl FnMut(&Placement) -> Option<(u32, V3, f32)>,
    ) -> Doors {
        let mut list = Vec::new();
        for &pi in door_placements(placements).iter() {
            let p = &placements[pi];
            let par = |k: u32| p.params.iter().find(|(i, _)| *i == k).map_or(0, |&(_, v)| v);
            // The runtime record `parsemap_block_map_data_dynamic` (0x1d1b68) builds on its stack: the record's
            // +0x40 (count) → +0x02, +0x42 (rate) → +0x04, and `m_paths[record+0x44]` → +0x08 — that is,
            // extra[7]'s halves and extra[8]'s low half.
            let count = (p.extra[7] & 0xffff) as u16;
            let rate = (p.extra[7] >> 16) as u16;
            let index = (p.extra[8] & 0xffff) as usize;
            let keys = if count != 0 { path_blocks.get(index).map(|b| parse_keys(b)) } else { None };
            let (glist, centre, radius) = object(p).unwrap_or((0, p.translation, 0.0));
            // Door_Create's pre-spline seed matrix: CCO(0) → View_RotTransMatrix; the swing/sliding
            // key paths overwrite obj+0x90 from Spline_Interp afterwards [M 0x136100:21,83-91].
            let m = view_rot_trans_matrix(p.rotation, p.translation);
            let flags = (par(8) | par(9)) as u16;
            let mut d = Door {
                placement: pi,
                kind: par(0) as i16,
                group: par(1) as u16,
                ch_open: par(2) as u16,
                ch_lock: par(3) as u16,
                straddle: par(4) != 0,
                snd_open: par(5) as u16, snd_close: par(6) as u16, snd_hit: par(7) as u16,
                flags,
                approach: par(10) as u8,
                locked_flag: par(11) as u8,
                keys, frames: count.wrapping_sub(1),
                state: if flags & 4 != 0 { OPEN } else { CLOSED },
                pos01: 0.0, speed: if rate != 0 { 1.0 / rate as f32 } else { 1.0 / CREATE_FRAME_RATE },
                dir: 0.0, swing: 0.0, timer: 0, open_timer: 0, msg_shown: false,
                matrix: m, inv_closed: [[0.0; 4]; 4],
                pos: p.translation, sph: (transform(&m, centre), radius), sph_centre: centre, glist,
            };
            if d.kind == 0 {
                // `Door_SetupSwing`: the swing covers half the range, so the speed re-quantises to whole ticks
                // (`0.5 / (int)(0.5 / 2s)`; 1/30 → 1/14 [L]); closed sits at 0.5 (1.0 for a starts-open door);
                // the inverse closed pose and the swing sign come from the anim's ends.
                d.speed = 0.5 / ((0.5 / (d.speed + d.speed)) as i32 as f32);
                d.pos01 = if d.state & 4 != 0 { 1.0 } else { 0.5 };
                if let Some(closed) = d.closed_pose() {
                    d.inv_closed = rigid_inverse(&closed);
                    if let Some(open_a) = d.pose_at(0.0, true) {
                        d.swing = if dot3(row(&open_a, 2), row(&closed, 0)) < 0.0 { 1.0 } else { -1.0 };
                    }
                }
                if let Some(m) = d.pose_at(d.pos01, true) { d.matrix = m; }
            } else {
                d.pos01 = if d.state & 4 != 0 { 1.0 } else { 0.0 };
                if let Some(m) = d.pose_at(d.pos01, true) { d.matrix = m; }
                if let Some(c) = d.pose_at(0.0, true) { d.inv_closed = rigid_inverse(&c); }
            }
            list.push(d);
        }
        let n = list.len();
        Doors {
            list,
            group_states: [0; 101],
            prev_channels: [0; 256],
            hold_channels: [0; 256],
            labels: Vec::new(),
            contacts: (0..n).map(|_| Vec::new()).collect(),
        }
    }

    /// A door that starts open sets its open channel at create (`Door_Create`'s tail).
    pub fn create_channels(&self, channels: &mut Channels, frame: u32) {
        for d in &self.list {
            if d.state & 4 != 0 && d.ch_open != 0 { channels.set(d.ch_open as u32, 1, frame); }
        }
    }

    /// `Door_SetState`: the entering sounds, the open channel mirrors "in motion" (timestamped every motion
    /// transition, not only its rising edge), the auto-close timer resets.
    fn set_state(&mut self, i: usize, new: u16, channels: &mut Channels, frame: u32, sounds: &mut Vec<SoundEvent>) {
        let (old, pos, ch, so, sc) = {
            let d = &self.list[i];
            (d.state, d.pos, d.ch_open, d.snd_open, d.snd_close)
        };
        if new & 8 != 0 && old & 8 == 0 && so != 0xffff { sounds.push((so, pos)); }
        if new & 0x10 != 0 && old & 0x10 == 0 && sc != 0xffff { sounds.push((sc, pos)); }
        if ch != 0 {
            // `Door_SetState` (0x135618:26): the channel mirrors the OLD state's "in motion" bits (0xc), timestamped
            // every motion transition. The game reads `+0xf4 & 0xc` BEFORE writing the new state [M]. channel-persistence:
            // this goes through `Channels::set` — the game's write lands in `switch_channels` and stamps
            // `switch_channels_time` exactly when the mirrored bits are nonzero (set's nonzero stamp), and the port's
            // writer census sees it like every other write.
            let moving = (old & 0xc) != 0;
            channels.set(ch as u32, moving as u8, frame);
        }
        let d = &mut self.list[i];
        d.state = new;
        d.open_timer = 0;
    }

    /// `Door_Interp`: pose at the current `pos01`, advance, snap at the ends, re-pose. Without an anim the
    /// state machine still runs but nothing is posed (`Spline_Interp` returns on a NULL framelist).
    fn interp(&mut self, i: usize, advance: bool, channels: &mut Channels, frame: u32, sounds: &mut Vec<SoundEvent>) {
        let (spline, frames) = (self.list[i].flags & 1 != 0, self.list[i].frames);
        let _ = frames;
        if let Some(m) = self.list[i].pose_at(self.list[i].pos01, spline) { self.list[i].matrix = m; }
        if !advance { return; }
        let (speed, dir) = (self.list[i].speed, self.list[i].dir);
        let f = self.list[i].pos01 + speed * dir;     // FRAME_RATE_MUL is 1.0 during play [L]
        self.list[i].pos01 = f;
        let mut posed = false;
        let (kind, state) = (self.list[i].kind, self.list[i].state);
        if kind == 0 {
            // The game's ends are UNCONDITIONAL range clamps [M Door_Interp 0x135738:33-61]: pos01
            // += speed·dir, then `pos01 < 0 → 0` / `pos01 > 1 → 1.0` with the state switch,
            // whatever the state — the old port gate (`state & 8 == 0 ||` in range) skipped the
            // clamp for a leaf whose dir pointed outward while not OPENING, and pos01 ran away
            // unbounded (the 06 guardhouse log: pos01 9.0). The 0.5 close-snap keeps its CLOSING
            // gate [M 0x135738:40-43].
            if (0.0..=1.0).contains(&f) {
                if state & 0x10 != 0 && (0.5 - self.list[i].pos01).abs() <= speed {
                    self.list[i].pos01 = 0.5;
                    self.set_state(i, CLOSED, channels, frame, sounds);
                    posed = true;
                }
            } else {
                self.list[i].pos01 = f.max(0.0).min(1.0);
                self.set_state(i, OPEN, channels, frame, sounds);   // fully swung to an end
                posed = true;
            }
        } else if (0..=2).contains(&kind) {
            if f < 0.0 {
                self.list[i].pos01 = 0.0;
                self.set_state(i, CLOSED, channels, frame, sounds);
                posed = true;
            }
            if self.list[i].pos01 > 1.0 {
                self.list[i].pos01 = 1.0;
                self.set_state(i, OPEN, channels, frame, sounds);
                posed = true;
            }
        }
        if posed {
            // The re-pose at the snapped position is always the spline mode (`Door_Interp`'s tail).
            if let Some(m) = self.list[i].pose_at(self.list[i].pos01, true) { self.list[i].matrix = m; }
        }
    }

    /// `Door_IsLocked` 0x136810 [M :4-17]: a null obj (the port's rows always exist) or an
    /// unwired lock channel (`ext+0x68 == 0`) is not locked; a wired channel reads
    /// `switch_channels[ch] == 0` — locked = the channel is OFF. The game's callers are the
    /// AI pathing links (`AINetwork_EmitPath` 0x158cf8:233, `AINetwork_Emitter_GetNodeAtDistance`
    /// 0x159268:177, `AINetwork_DoAStarPath` 0x1571b0:269 — a link between two door-type
    /// MAPNODEs (+4 & 2) is rejected while locked); the port's `nav::Network` route queries
    /// do not apply the gate yet (drones open doors by contact), so this is the door-side
    /// fn the pathfinder wiring will call [S].
    pub fn is_locked(&self, i: usize, channels: &Channels) -> bool {
        let d = &self.list[i];
        d.ch_lock != 0 && !channels.on(u32::from(d.ch_lock)) // [M 0x136810:10-16]
    }

    /// `Door_DoMsg` 0x135f18 [M]: the aim-ray door hint. The game's order — the `+0x74` latch sets
    /// BEFORE any check (one attempt per door ever, a locked door burns it), then the lock-channel
    /// gate (a wired channel that is OFF fails), then the state gate (`== 4` OPEN fails; grouped
    /// doors read `DoorGroupsStates[g]`) — returns `Txt_BindLabel(0x200000b, 0)` or 0.
    /// `mp` is `MPSettings+0x388 != 0` [M 0x135f18:13]: no hints in a match.
    pub fn do_msg(&mut self, i: usize, mp: bool, channels: &Channels) -> Option<u32> {
        if mp { return None; }
        if self.list[i].msg_shown { return None; }
        self.list[i].msg_shown = true;
        if self.is_locked(i, channels) { return None; }
        let d = &self.list[i];
        let state = if d.group != 0 {
            self.group_states.get(d.group as usize).copied().unwrap_or(0)
        } else {
            d.state
        };
        if state != OPEN { Some(LABEL_DOOR_HINT) } else { None }
    }

    /// The aim ray's door pick (`Check_Target` 0x1a7810's `Collide_RayIntersect` arm) [M disasm
    /// 0x1a79ec-0x1a7a20: a0 = HITDATA+0x58, the hit object; class `obj+0xff` == 0x1c]. The game's
    /// ray is a true MESH ray (docs/systems/door-aim-ray.md): `Collide_RayIntersect` 0x1ec690
    /// builds a mask-0x201 HITTEST and sorts per-object hits by fraction [M :95-112, :114-160];
    /// `Collide_Intersect` 0x1ebaa8 transforms A/B into the object space of the LIVE `obj+0x90`
    /// and calls `Intersect_RayGeom` [M :157-161, :229], which skips every triangle whose plane
    /// denominator is non-negative (`n·dir >= 0`) and accepts only a fraction in [0, 1]
    /// [M 0x10a8c8:168-172]. The port tests the door's own collision triangles
    /// (`world.shape(glist)` — the same data `set_dynamic` poses for the door's solidity) at the
    /// live pose with that front-face rule. The deleted stand-in tested the LIVE WORLD SPHERE
    /// (obj+0x80/obj+0x8C) with the entry fraction clamped to 0 inside the sphere — reporting the
    /// door for ANY ray from inside it, aiming away too, burning `Door_DoMsg`'s one-shot +0x74
    /// latch [S stand-in deleted]. Returns `(door index, hit fraction)` of the nearest touch, or
    /// None; `wall_t` still caps by the caller's static-segment fraction (the game's ray is
    /// wall-occluded by the same walk).
    pub fn pick_target(&self, world: &CollisionWorld, eye: V3, dir: V3, wall_t: f32) -> Option<(usize, f32)> {
        let l = (dir[0] * dir[0] + dir[1] * dir[1] + dir[2] * dir[2]).sqrt();
        if l < 1e-9 { return None; }
        let d = [dir[0] / l, dir[1] / l, dir[2] / l];
        let far = [eye[0] + d[0] * 50.0, eye[1] + d[1] * 50.0, eye[2] + d[2] * 50.0]; // the 50-unit Check_Target ray [M 0x1a7810:114-116]
        let mut best: Option<(f32, usize)> = None;
        for (i, door) in self.list.iter().enumerate() {
            // The game's ray tests only glist-backed objects (the use pick's admission) [M 0x1eae80:88-90].
            if door.glist == 0 { continue; }
            let Some(tris) = world.shape(door.glist) else { continue; };
            // The game tests in OBJECT space: the inverse of the live `obj+0x90`, `RotPreTransVec`
            // for A and B [M 0x1ebaa8:157-161, 193-229]. Row-vector v·M⁻¹, the rigid inverse.
            let inv = rigid_inverse(&door.matrix);
            let a = transform(&inv, eye);
            let seg = sub(transform(&inv, far), a);
            for t in tris {
                let (e1, e2) = (sub(t[1], t[0]), sub(t[2], t[0]));
                // One-sided, like the game [M Intersect_RayGeom 0x10a8c8:168: `n·dir >= 0` skips
                // the triangle; the winding IS the facing [D — world.rs's stored-normal identity]].
                if dot3(cross3(e1, e2), seg) >= 0.0 { continue; }
                let p = cross3(seg, e2);
                let det = dot3(e1, p);
                if det.abs() < 1e-12 { continue; }
                let inv_det = 1.0 / det;
                let s = sub(a, t[0]);
                let u = dot3(s, p) * inv_det;
                if !(0.0..=1.0).contains(&u) { continue; }
                let q = cross3(s, e1);
                let v = dot3(seg, q) * inv_det;
                if v < 0.0 || u + v > 1.0 { continue; }
                let tt = dot3(e2, q) * inv_det;
                if (0.0..=1.0).contains(&tt) && best.is_none_or(|(bt, _)| tt < bt) { best = Some((tt, i)); }
            }
        }
        best.map(|(tf, i)| (i, tf))
    }

    /// `Door_Update` for one door, in the control list's order. `actors`: the objects the proximity tests see —
    /// the game scans classes 2 (the player), 3 and 0x12 (drones) by bounding sphere, skipping every candidate
    /// whose `obj+0xF0` carries 0x10 [M 0x136678-0x136684]; **`actors[0]` is the activator** (`glb_players` in
    /// every call the port makes). Returns the sounds to play this tick.
    pub fn update(
        &mut self, i: usize, channels: &mut Channels, actors: &[DoorActor], frame: u32,
    ) -> Vec<SoundEvent> {
        let mut sounds = Vec::new();
        if self.list[i].timer != 0 { self.list[i].timer -= 1; }
        if self.list[i].flags & 0x10 != 0 { return sounds; }   // busy: `Door_Activate`'s marker, cleared by `calc`
        // The live `obj+0x80` sphere: the pose matrix from the LAST interp (the game's moved pass rebuilds the
        // sphere after the walk — one tick behind the visual, matching this read order) [M 0x1336f8:209].
        let door_sph = (transform(&self.list[i].matrix, self.list[i].sph_centre), self.list[i].sph.1);
        // The open channel: on opens (unless already open); off and not closed: close at once, timer reset.
        let (state, ch_open, ch_lock, keep_open) = {
            let d = &self.list[i];
            (d.state, d.ch_open, d.ch_lock, d.flags & 0x20 != 0)
        };
        if state & 6 != 0 && ch_open != 0 {
            if channels.on(ch_open as u32) {
                if state & 4 == 0 { self.activate(i, actors, false, channels, frame, &mut sounds); }
            } else if state & 2 == 0 {
                if !visible_actors(actors).any(|a| blocked(door_sph, a)) {
                    self.activate(i, actors, false, channels, frame, &mut sounds);
                }
                self.list[i].open_timer = 0;
            }
        }
        // The lock channel's rising edge: the game's message block FIRST, then unlock for good. The
        // message: level-gated (0x7000014/0x700004a — the caller's gate; Doors doesn't know the level)
        // `Text_AddMsg(0, 0, 1, Txt_BindLabel(0x2000003, 0), 0, 0xb4)` — 180 ticks [M 0x1363c8:101-107];
        // then `Door_Activate(obj, glb_players, 0)`, `+0x68 = 0`, busy marker [M :105-107].
        if ch_lock != 0 {
            let was = self.prev_channels[ch_lock as usize] != 0;
            let now = channels.on(ch_lock as u32);
            if !was && now {
                self.labels.push((LABEL_DOOR_UNLOCKED, 180.0));
                self.activate(i, actors, false, channels, frame, &mut sounds);
                let d = &mut self.list[i];
                d.ch_lock = 0;
                d.flags |= 0x10;
            }
        }
        // The auto-close: an open door without the keep-open flag, after `FRAME_RATE_INT·6` quiet ticks
        // [M 0x13660c-0x136700]. The timer ALWAYS increments while open; at 360 the proximity scan runs (skipping
        // `obj+0xF0 & 0x10` candidates), and if nobody is blocking, `Door_Activate` closes it. The timer ALWAYS
        // resets to 0 at 360 (whether or not the close fired). The scan centres on the LIVE `obj+0x80` sphere with
        // the real `obj+0x8C` radius — the open leaf, not the frame (core-loop §7 item 4).
        if self.list[i].state & 4 != 0 && !keep_open {
            self.list[i].open_timer = self.list[i].open_timer.wrapping_add(1);
            if self.list[i].open_timer as u32 >= AUTO_CLOSE {
                if !visible_actors(actors).any(|a| blocked(door_sph, a)) {
                    self.activate(i, actors, false, channels, frame, &mut sounds);
                }
                self.list[i].open_timer = 0;
            }
        }
        // Motion. `Door_ReactToHit` (0x135980) is the COLLIDE function (control_funcs[0x1C*0xC+4]), called
        // from `Collide_Update` (0x1ea398:59-64) via the proxy list (obj+0xD0) — NOT from `Door_Update`.
        // The port's proxy-list path (U-041) is not ported; until then, the anti-crush does not fire from
        // `update` — matching the disassembly's call structure. (A-11b: removed the `react_to_hit` call that
        // was reversing CLOSING→OPENING when Bond stood near the hinge, preventing the close.)
        if self.list[i].state & 0x18 != 0 {
            // NF_DOORTRACE=1 per-tick diagnostic (level-0b-waiting-room-doors): a moving door's
            // (index, state, pos01, dir, speed, flags) each tick, so a stalled swing shows its
            // last-advancing tick and the gate that stopped it.
            if std::env::var_os("NF_DOORTRACE").is_some() {
                let d = &self.list[i];
                eprintln!("[doortrace] #{} f{} state {:#x} pos01 {:.4} dir {:.1} speed {:.4} flags {:#x}",
                    i, frame, d.state, d.pos01, d.dir, d.speed, d.flags);
            }
            self.interp(i, true, channels, frame, &mut sounds);
        }
        let d = &self.list[i];
        if d.group != 0 && (d.group as usize) < self.group_states.len() {
            self.group_states[d.group as usize] |= d.state;
        }
        sounds
    }

    /// `Door_Activate` [M: disassembly 0x135be0–0x135f14]. `from_player` honours the locked flag.
    pub fn activate(
        &mut self, i: usize, actors: &[DoorActor], from_player: bool, channels: &mut Channels, frame: u32,
        sounds: &mut Vec<SoundEvent>,
    ) -> bool {
        let _ = frame;
        let (hit_snd, pos) = (self.list[i].snd_hit, self.list[i].pos);
        let mut hit = || { if hit_snd != 0xffff { sounds.push((hit_snd, pos)); } };
        if from_player && self.list[i].locked_flag != 0 { return false; }
        {
            let d = &self.list[i];
            if d.ch_lock != 0 && !channels.on(d.ch_lock as u32) && d.state & 0x12 != 0 { hit(); return false; }
            if d.flags & 0x10 != 0 { return false; }
        }
        // The approach side (one-shot): the activator's offset along the door's facing (`Mat_GetDir`).
        if self.list[i].approach != 0 {
            let dot = actors.first().map_or(0.0, |a| dot3(sub(a.0, self.list[i].pos), row(&self.list[i].matrix, 2)));
            if self.list[i].approach == 1 && dot < 0.0 { hit(); return false; }
            if self.list[i].approach == 2 {
                if dot > 0.0 { hit(); return false; }
                self.list[i].approach = 0;
            } else {
                self.list[i].approach = 0;
            }
        }
        let mut acted = false;
        let (kind, state) = (self.list[i].kind, self.list[i].state);
        match kind {
            0 => {
                // Swing: closed or closing swings away from the activator's side; open or opening closes toward
                // 0.5 — unless the activator stands in the swing path (the ±0.3 margin against the radius).
                if state & 0xc == 0 {
                    self.swing_from_side(i, actors, channels, frame, sounds);
                    acted = true;
                } else {
                    let act = actors.first().map_or(pos, |a| a.0);
                    let (dir, swing, radius) = (self.list[i].dir, self.list[i].swing, self.list[i].sph.1);
                    let local = transform(&self.list[i].inv_closed, act);
                    let m = if dir * swing < 0.0 { local[2] + MARGIN } else { local[2] - MARGIN };
                    let f = m * dir * swing;
                    if f < 0.0 && f.abs() < radius {
                        // In the doorway: the busy marker only (the door waits the two ticks out).
                    } else if state & 0x12 != 0 {
                        self.swing_from_side(i, actors, channels, frame, sounds);
                        acted = true;
                    } else if state & 0xc != 0 {
                        self.list[i].dir = if self.list[i].pos01 > 0.5 { -1.0 } else { 1.0 };
                        self.set_state(i, CLOSING, channels, frame, sounds);
                        acted = true;
                    }
                }
            }
            1..=2 => {
                // Slide: closed or closing opens; open or opening closes (a plain reverse).
                if state & 0x12 != 0 {
                    self.list[i].dir = 1.0;
                    self.set_state(i, OPENING, channels, frame, sounds);
                    acted = true;
                } else if state & 0xc != 0 {
                    self.list[i].dir = -1.0;
                    self.set_state(i, CLOSING, channels, frame, sounds);
                    acted = true;
                }
            }
            _ => {}
        }
        let d = &mut self.list[i];
        d.timer = 2;
        d.flags |= 0x10;
        acted
    }

    /// The swing door's `side·swing` opening: `dir = sign((inv_closed · activator).z) · swing`, state 8.
    fn swing_from_side(&mut self, i: usize, actors: &[DoorActor], channels: &mut Channels, frame: u32, sounds: &mut Vec<SoundEvent>) {
        let act = actors.first().map_or(self.list[i].pos, |a| a.0);
        let local = transform(&self.list[i].inv_closed, act);
        let side = if local[2] < 0.0 { -1.0 } else { 1.0 };
        self.list[i].dir = side * self.list[i].swing;
        self.set_state(i, OPENING, channels, frame, sounds);
    }

    /// The door collide step: `Collide_Update` 0x1ea398's pick + forward, then the class-0x1C dispatch
    /// `Door_ReactToHit` 0x135980. The caller (the collide caller in `Drones::tick`) runs it after the
    /// control walk, before `Doors_Calc` — the game's order (atlas/object.md steps 6-7, `Doors_Calc`
    /// at 0x1336f8:23) — so a reversal's busy marker clears in the same tick's calc, exactly as the
    /// game's next-frame `Doors_Calc` would, and the 2-tick timer (`ext+0x6C`) still gates it.
    ///
    /// Pick [M `Collide_PickObj` 0x1eae80:100-110 — the test is the door's own `obj+0x8C` radius
    /// ALONE: the sphere centre+radius `obj+0x80..0x8c` are copied onto the hittest, the closest
    /// point on the A..B segment is taken (`vecutil_calculate_closest_point_on_line`), and the pick
    /// fires only while `SqDist3D(centre, closest) < r_door²` (`r_door² <= d²` returns). No 1.5
    /// inflation and no pick-radius term — the old `1.5·r_door + r_pick` made every E-activator a
    /// permanent contact, so `Door_ReactToHit`'s anti-crush (0x135980:85-97) reversed the swing
    /// every ~3 ticks and the leaf never cleared the doorway (0700000b doors #3/#0, the walkreal
    /// stand-in's cause). Mode 0x800 = the player's `BLData+0x7D4`, written by `Player_Collision`
    /// 0x194df0's substate arms; for every mover × every door (the ForcedList is global —
    /// `Collide_Pick` 0x1eb320:369-375 — no cel filter), the LIVE world sphere (`obj+0x80`, rebuilt
    /// by the moved pass after `Door_Interp` [M 0x1336f8; the port's `transform(matrix,
    /// sph_centre)`]). On hit, the contact forwards to the door's `obj+0xD0` (+0x58 = the hitter)
    /// [M `Collide_Update` 0x1ea398:160-233]. Returns the `Door_SetState` sounds.
    pub fn collide_step(&mut self, movers: &[DoorMover], channels: &mut Channels, frame: u32) -> Vec<SoundEvent> {
        let mut sounds = Vec::new();
        if self.contacts.len() != self.list.len() { self.contacts.resize_with(self.list.len(), Vec::new); }
        for h in self.contacts.iter_mut() { h.clear(); }   // the clear pass: every +0xD0 to the pool [M 0x1ea398:25-53]
        for m in movers {
            let ab = [m.b[0] - m.a[0], m.b[1] - m.a[1], m.b[2] - m.a[2]];
            for i in 0..self.list.len() {
                let c = transform(&self.list[i].matrix, self.list[i].sph_centre);
                let r = self.list[i].sph.1;   // obj+0x8C alone [M 0x1eae80:105-110]
                let ab2 = ab[0] * ab[0] + ab[1] * ab[1] + ab[2] * ab[2];
                let t = if ab2 > 0.0 {
                    (((c[0] - m.a[0]) * ab[0] + (c[1] - m.a[1]) * ab[1] + (c[2] - m.a[2]) * ab[2]) / ab2).clamp(0.0, 1.0)
                } else { 0.0 };
                let dx = m.a[0] + ab[0] * t - c[0];
                let dy = m.a[1] + ab[1] * t - c[1];
                let dz = m.a[2] + ab[2] * t - c[2];
                if dx * dx + dy * dy + dz * dz < r * r {
                    self.contacts[i].push(DoorContact { centre: m.centre, player: m.player, valid: m.valid });
                }
            }
        }
        // The dispatch [M 0x1ea398:59-64]: `control_funcs[class·0xC+4](obj)` — doors run `Door_ReactToHit`.
        for i in 0..self.list.len() {
            if !self.contacts[i].is_empty() { self.react_to_hit(i, channels, frame, &mut sounds); }
        }
        sounds
    }

    /// `Door_ReactToHit` [M 0x135980] over this tick's contacts (`obj+0xD0`). Guards [M :44]: `ext+0x6C == 0`
    /// (the 2-tick lockout `Door_Update`'s head decrements — 0x1363c8:22-23). Selection [M :47-72]: walk the
    /// contacts in list order; class 3 (the player) qualifies outright, class 2 (a drone) only through the
    /// `+0x4F8`/`+0xAC`/`+0xFE` gate; the first qualifier's `obj+0x80` is the hitter. Then:
    /// kind 1: nothing; kind 0 animating (`obj+0xF4 & 0x18`): the hitter's centre in the door's live space
    /// (`f = local.z · ext+0x5C`, negated past half travel `ext+0x50 > 0.5`) — `f < −0.0002` while CLOSING
    /// reverses to OPENING, `f > +0.0002` while OPENING reverses to CLOSING (the anti-crush), other cases
    /// return early (no tail); kind 2 while CLOSING (`& 0x10`) reverses to OPENING. Kind 0 idle falls through
    /// to the tail. Tail [M :109-111]: `Door_Interp(…, 0)` re-pose, `ext+0x6C = 2`, busy `ext+0x70 |= 0x10`.
    fn react_to_hit(&mut self, i: usize, channels: &mut Channels, frame: u32, sounds: &mut Vec<SoundEvent>) {
        if self.list[i].timer != 0 { return; }
        let Some(hit) = self.contacts[i].iter().find(|c| c.player || c.valid).map(|c| c.centre) else { return };
        let (kind, state, dir) = { let d = &self.list[i]; (d.kind, d.state, d.dir) };
        if kind == 1 { return; }
        if kind < 2 {
            if kind == 0 && state & 0x18 != 0 {
                let local = transform(&rigid_inverse(&self.list[i].matrix), hit);
                let mut f = local[2] * self.list[i].swing;
                if self.list[i].pos01 > 0.5 { f = -f; }
                if state & 0x10 == 0 || f >= -HIT_EPS {
                    if state & 8 == 0 { return; }
                    if f <= HIT_EPS { return; }
                    self.list[i].dir = -dir;
                    self.set_state(i, CLOSING, channels, frame, sounds);
                } else {
                    self.list[i].dir = -dir;
                    self.set_state(i, OPENING, channels, frame, sounds);
                }
            }
        } else if kind == 2 {
            if state & 0x10 == 0 { return; }
            self.list[i].dir = -dir;
            self.set_state(i, OPENING, channels, frame, sounds);
            self.interp(i, false, channels, frame, sounds);   // the kind-2 arm's own Door_Interp(0) [M :107]
        }
        // The tail [M :109-111]: Door_Interp(obj, ext, 0), timer = 2, busy.
        self.interp(i, false, channels, frame, sounds);
        let d = &mut self.list[i];
        d.timer = 2;
        d.flags |= 0x10;
    }

    /// `Doors_Calc`'s head memset [M 0x135fe0:13 — the per-frame wipe the decompile opens with]:
    /// the game runs `Doors_Calc` at the top of `control_movement_object_handler` 0x1336f8 (before
    /// the object walk), so `DoorGroupsStates` starts each frame empty and `Door_Update`'s ORs
    /// (0x1363c8:170-171) accumulate only this frame's states — `Door_IsClosed` 0x1367b0's
    /// `== CLOSED (2)` read never sees a stale OR. The port runs `calc` at the tick's tail
    /// (drone.rs), so the wipe rides `begin_pass` — call it before the object pass each tick.
    pub fn begin_pass(&mut self) {
        self.group_states = [0; 101];
    }

    /// `Doors_Calc` (after every door's update): a group's members copy the first non-busy door's direction,
    /// position, state and timer, re-pose, and every busy marker clears. (The game's per-frame
    /// `DoorGroupsStates` memset is `begin_pass` above — the decompile's 0x135fe0:13.)
    pub fn calc(&mut self) {
        for master in 0..self.list.len() {
            let group = self.list[master].group;
            if group == 0 { continue; }
            let (dir, pos01, state, open_timer) = {
                let m = &self.list[master];
                (m.dir, m.pos01, m.state, m.open_timer)
            };
            for other in 0..self.list.len() {
                if other == master { continue; }
                let spline;
                {
                    let d = &mut self.list[other];
                    if d.group != group || d.flags & 0x10 != 0 { continue; }
                    d.dir = dir;
                    d.pos01 = pos01;
                    d.state = state;
                    d.open_timer = open_timer;
                    d.flags |= 0x10;
                    spline = d.flags & 1 != 0;
                }
                if let Some(m) = self.list[other].pose_at(pos01, spline) { self.list[other].matrix = m; }
            }
        }
        for d in self.list.iter_mut() { d.flags &= !0x10; }
    }

    /// `Player_Activate`'s door part (0x1a5310): the use sphere — radius 1 at one unit ahead of the head —
    /// activates every door it meets (the game walks the whole hit list; walls do not stop it). `head` is also
    /// the activator's position for the side tests.
    pub fn player_use(&mut self, head: V3, look: V3, channels: &mut Channels, frame: u32, _world: &CollisionWorld) -> (Vec<SoundEvent>, usize) {
        // `Player_Activate` (0x1a5170:100-133): the pick is `Collide_SphereIntersect(1.0, head +
        // Mat_GetDir(viewer)·1.0, player cel, player, body obj, &list, 0, 0, 0, 0)`, and the walk
        // dispatches `Door_Activate(obj, player, 1)` on EVERY class-0x1c node — no distance gate
        // [M 0x1a5170:113-137]. The candidate that puts a node in the list is SPHERE-VS-SPHERE,
        // not a mesh test:
        //   - `Collide_SphereIntersect` 0x1ec940 fills the HITTEST (mask +0x94 = 0x101 — param 8
        //     is 0 — the pick centre at +0x60, the radius at +0x6c) and calls `Collide_Pick`
        //     0x1eb320; the post-pick loop then runs `Collide_Intersect` per node but uses its
        //     return ONLY for the `param_10 & 0x10` early-out (param_10 is 0 here) — a failed
        //     refinement NEVER removes the node [M 0x1ec940's do/while after `Collide_Pick`].
        //   - `Collide_Pick` 0x1eb320 walks the straddled cels; for mask 0x101 its linked-object
        //     arm tests `|pick_centre − obj+0x80|² < (pick_radius + obj+0x8C)²` (the AABB variant
        //     only when obj+0x3c & 0x8000000), and its object walk calls `Collide_PickObj`
        //     0x1eae80, whose 0x100/0x101 arm tail is the same sphere test from the pick centre
        //     [M 0x1eae80: `fVar11 = SqMagnitude(centre − obj+0x80); fVar12 = hittest+0x6c +
        //     obj+0x8c; hit iff fVar11 < fVar12²`] — the glist gate (`obj+0xd8 != 0`) only
        //     ADMITS the object to the pick, it does not shape it.
        //   - `obj+0x80`/`obj+0x8C` is the object's world sphere: the glist bounds centre/radius
        //     posed by the object matrix (`Control_BuildWorldSph` 0x133070: obj+0x80 = glist+0x10
        //     × the matrix rows, obj+0x8C = glist+0x1c · |obj+0xe8|; the moved pass rebuilds it
        //     [M 0x1336f8:209]) — the port's `live_sph()`.
        // So the game's E reach is the use sphere touching the door's live world sphere, and the
        // leaf mesh never gates the activation. (The 820b leaf-mesh pick was a misread of the
        // chain — its own T1 "2.3-out must not open" pin rested on that misread; the master
        // merge's full suite failed the three real-path E tests because the mesh, a thin plane,
        // sits 1.2-3.0 from those use points while the door's world sphere is touched. See
        // docs/systems/door-use.md.)
        // A door with NO resolved glist (glist 0 — no obj+0xd8) is E-DEAD, as in the game: the use
        // pick admits a 0x101-mask object only via `obj+0xd8 != 0 && *(obj+0xd8+8) != 0` [M
        // Collide_PickObj 0x1eae80:88-90 — Door_Create 0x136100 ORs obj+0xfa |= 0xc4, so the
        // obj+0xfa&8 arm is clear for doors], and the game never creates such a door at all —
        // `parsemap_create_dynamic_objects` runs only when the placement's glist resolved [M
        // parsemap_block_map_data_dynamic 0x1d1b68:93-95]. The old placement-distance < 2.5
        // stand-in here is DELETED (door-glist0-use-standin): the ISO-gated census
        // (`nf-game/tests/door_glist_census_live.rs`, 21 SP levels, 443 doors) shows every created
        // door resolves a glist, and the glist-0 records the create fallback can still produce
        // (a broken by_hash map) are skipped below, matching the game's non-admission.
        let mut sounds = Vec::new();
        let use_point = [head[0] + look[0], head[1] + look[1], head[2] + look[2]];
        let mut hits = 0;
        for i in 0..self.list.len() {
            let d = &self.list[i];
            // The game's admission gate: no obj+0xd8 (glist 0) → the pick never admits the door
            // [M 0x1eae80:88-90]. Also drops the pre-sphere stand-in's reach for good.
            if d.glist == 0 { continue; }
            let dx = use_point[0] - d.pos[0];
            let dy = use_point[1] - d.pos[1];
            let dz = use_point[2] - d.pos[2];
            // Broad phase: the placement's bounding sphere can hardly exceed ~3 — 6 clears the
            // scan for far doors (the game's `Collide_Pick` does the same by cel proximity).
            if dx * dx + dy * dy + dz * dz > 36.0 { continue; }
            // The candidate: the use sphere (r 1.0 at head + viewer dir) vs the door's live
            // world sphere (obj+0x80 centre, obj+0x8C radius) [M Collide_PickObj 0x1eae80's
            // 0x100/0x101 arm; Collide_Pick 0x1eb320's linked-object arm]. Strict `<`: the
            // game returns (no node) when `radius² <= sqdist` [M both sites].
            let (c, r) = d.live_sph();
            let sx = use_point[0] - c[0];
            let sy = use_point[1] - c[1];
            let sz = use_point[2] - c[2];
            let reach = 1.0 + r;
            if sx * sx + sy * sy + sz * sz < reach * reach {
                self.activate(i, &[(head, 0.0, false)], true, channels, frame, &mut sounds);
                hits += 1;
            }
        }
        // The straddle walk-in (walkreal-c): the kind-2 slide doors carry the placement's straddle
        // param (par 4 [D] — every 07000015/16 kind-2 door has it) and the game arms a per-door
        // flag from it at create (`Door_Create` 0x136100 reads the +0x3c param into obj+0xfd bit 8
        // [M]). The player opens them by WALKING INTO the threshold — the 07000015 top-band door
        // [32] (25.1, 41.6, 70.1, unlocked) is E-dead in the port yet must open for the level's
        // only free west crossing [D flow]. The port had the flag parsed but never consumed.
        // No game straddle test exists to pin — the opener census (standins-door-933) closes the
        // question: `Door_Activate` 0x135be0 has exactly three callers (Player_Activate
        // 0x1a5170:100-129 the E press, Door_Update 0x1363c8's channel/close arms,
        // NDrone2_OpenDoor 0x14aa38 the route doors) and `Door_SetState` 0x135618 exactly three
        // (Door_Interp 0x135738, Door_ReactToHit 0x135980, Door_Activate) [M, symbol census over
        // the whole ACTION.ELF decompile], and `Door_ReactToHit` reverses only MOVING leaves (kind
        // 0 needs `obj+0xf4 & 0x18`, kind 2 `& 0x10` [M 0x135980:77, 101-104]) — no contact or
        // proximity opens a closed idle door. A closed straddle door activates here (from_player —
        // the lock guards hold) when the player's feet stand inside its threshold (xz 1.4, the
        // leaf's half-width plus the capsule; y within the door band) — SHIM PARAMETERS, not game
        // claims.
        // CORRECTION (level-15-complete): the "every 07000015 kind-2 door carries it" and
        // "door[32] is E-dead" claims above are DISPROVEN by the disc — none of 07000015's
        // corridor doors [29]/[32]/[49]/[50]/[51]/[52] carries a par-4 (parsed `straddle=false`,
        // probe walkreal_probe15), and door[32] opens with the plain E path once the player
        // stands in reach: the walkreal-c stop was the ROUTE (the console placement map54 #145
        // walls the south-east corner; the nav graph's own lane reaches the doorway from the
        // north — walkreal_07000015_live now walks it green). The arm stays for the
        // 07000016-arrival-door shape as a LABELLED PLAYABILITY SHIM and changes no E-path
        // behaviour — the game's opening surface is provably closed to contact: `Door_Activate`
        // 0x135be0 has exactly three callers (Player_Activate 0x1a5170:100-129 the E press,
        // Door_Update 0x1363c8's channel/close arms, NDrone2_OpenDoor 0x14aa38 the route doors),
        // and `Door_ReactToHit` 0x135980:44-111 reverses only moving leaves (a closed idle door
        // falls through every branch) [M] — live checks L-019/L-141 (docs/live/backlog.md) are
        // the only remaining witness for the 16 arrival door.
        for i in 0..self.list.len() {
            let d = &self.list[i];
            if !d.straddle || d.kind == 0 || d.state & 0xc != 0 { continue; }
            let dx = head[0] - d.pos[0];
            let dz = head[2] - d.pos[2];
            if dx * dx + dz * dz > 1.96 { continue; }
            if (head[1] - d.pos[1]).abs() > 2.2 { continue; }
            self.activate(i, &[(head, 0.0, false)], true, channels, frame, &mut sounds);
            hits += 1;
        }
        (sounds, hits)
    }

    /// The straddle walk-in, EVERY tick (fix-casings-16). The E-path arm above only runs while the
    /// use key is held, but the game's straddle doors open with no E at all — on 07000016 the
    /// arrival walkway's first door (glist 0x803f00a0, kind 2, straddle, ch_open 0,
    /// ch_lock 0, flags 0x3 → closed at create [M Door_Create 0x136100: state 4 only when
    /// doorinfo+0x4c|+0x50 bit 2]) blocks the only path, and neither `Door_Update` (a ch_open==0
    /// door skips every channel arm [M 0x1363c8: `(state & 6) == 0` / `uVar2 == 0 → LAB_00136550`])
    /// nor `Door_ReactToHit` (0x135980 reverses only MOVING leaves: kind 0 needs `state & 0x18`,
    /// kind 2 needs `state & 0x10`) ever opens a closed idle one [M 0x135980:77, 101-104]. The
    /// port's collide path IS rebuilt (fix-door-react-path: `collide_step`, called from the
    /// collide caller in `Drones::tick`) and faithfully re-opens nothing. The old "the game opens
    /// it through the collide-contact path the port has not rebuilt (U-041)" premise is corrected
    /// twice over: U-041 is the searchlight head's bullet child list (docs/UNKNOWNS.md), not the
    /// doors; and the door opener census [M — `Door_Activate` 0x135be0's three callers,
    /// `Door_SetState` 0x135618's three, see the `player_use` walk-in note above] leaves the
    /// player's E press as the only game route through a closed idle ch_open-0 door. Until the
    /// live checks settle what the player actually experiences there (L-019/L-141,
    /// docs/live/backlog.md), this shim drives
    /// the SAME activation from `Door_Update`'s per-tick actor list, with the game's own proximity
    /// metric — `Vec_SqDist3D(door+0x80, actor+0x80) < (door+0x8C + actor+0x8C)² · 2`
    /// [M 0x1363c8 @0x136678-0x136684, the scan `Door_Update` itself runs for the auto-close] —
    /// so the open lands where the game's contact would. The lock guards hold: `activate`
    /// (from_player) rejects a locked door (the 320 thud) and the `state & 0xc` skip keeps a
    /// moving leaf alone. Swing doors (kind 0) stay E/ReactToHit-only, as in the E arm.
    ///
    /// 102j2-fix-casings-16b — the gate narrowed to the game's scan domain. `Door_Update`'s
    /// proximity scan exists ONLY as the auto-close: both of its arms sit behind an OPEN-door
    /// test — `(state & 2) == 0` inside the `(state & 6) != 0` branch [M 0x1363c8 @0x136678,
    /// the ch_open-went-off arm] and `uVar7 & 4` [M 0x1363c8 @0x136610, the open_timer arm] —
    /// and its only action is one `Door_Activate` to close. The scan never touches a moving
    /// leaf: a CLOSING (0x10) door is left to finish. The old `state & 0xc != 0` skip read only
    /// bits 2–3 (OPEN|OPENING), so a CLOSING straddle door passed the gate and the walk-in
    /// re-`activate`d it every tick — the port's slide `activate` turns 0x10 into OPENING
    /// [M Door_Activate 0x135be0: `state & 0x12` → SetState(8)], and a leaf barely off its open
    /// end snaps straight back to OPEN. That broke 07000001's
    /// `door_e_opens_then_e_closes_on_07000001` (after the second E the state read 0x4, not
    /// 0x10). The shim is the scan's open-side complement: it acts on a CLOSED IDLE door
    /// only (`state == CLOSED`), never on a moving one. Two more narrowing arms, same
    /// principle: a ch_open door is `Door_Update`'s own channel arm's job [M 0x1363c8: channel
    /// on → `Door_Activate`], so the shim covers only the channel-less doors (16's arrival
    /// door, ch_open 0 [M 0x136100 + glist 0x803f00a0]); and the lock pre-check is silent —
    /// the game's per-tick scan plays no lock sound (the ch_lock thud is `Door_Activate`'s
    /// E-path arm [M 0x135be0]), so the walk-in skips before `activate` instead of thudding
    /// every tick.
    pub fn straddle_walkin(&mut self, actors: &[DoorActor], channels: &mut Channels, frame: u32) -> Vec<SoundEvent> {
        let mut sounds = Vec::new();
        for i in 0..self.list.len() {
            let (straddle, kind, state) = { let d = &self.list[i]; (d.straddle, d.kind, d.state) };
            if !straddle || kind == 0 || state != CLOSED { continue; }
            let (ch_open, ch_lock, locked) =
                { let d = &self.list[i]; (d.ch_open, d.ch_lock, d.locked_flag) };
            if ch_open != 0 { continue; }
            if locked != 0 || (ch_lock != 0 && !channels.on(ch_lock as u32)) { continue; }
            let (sc, sr) = self.list[i].sph;
            let near = actors.iter().filter(|a| !a.2).any(|a| {
                let dx = sc[0] - a.0[0]; let dy = sc[1] - a.0[1]; let dz = sc[2] - a.0[2];
                let th = (sr + a.1) * (sr + a.1) * 2.0;
                dx * dx + dy * dy + dz * dz < th
            });
            if !near { continue; }
            self.activate(i, actors, true, channels, frame, &mut sounds);
        }
        sounds
    }

    /// Snapshot the channels for this tick's lock-edge test (`switch_channels_prev`). The game
    /// copies BOTH latches right after `Trigger_Calc`, BEFORE the object walk — two amemcpys at
    /// the head of `control_movement_object_handler` [M 0x1336f8: `amemcpy(0x26fd90, 0x26fe90,
    /// 0x100); amemcpy(0x26fe90, 0x26fc90, 0x100)` — `prev ← hold; hold ← channels`]: `prev`
    /// lags `channels` by TWO snapshots, so a channel pulsed anywhere since the last snapshot
    /// (a trigger or script mid-walk, or the player's own laser burn in `Player_Update`'s
    /// class-4 slot [M Player_LaserPointer 0x1a65d8 → `Lock_Update` 0x1ae3d8:194]) still shows a
    /// rising edge in `Door_Update`'s `prev == 0 && now != 0` test (0x136550-0x136608). The old
    /// single-copy read made any pre-snapshot write (the Q-Decryptor keypad burn on 07000012)
    /// invisible to the edge — the door stayed locked forever.
    pub fn end_tick(&mut self, channels: &Channels) {
        self.prev_channels = self.hold_channels;
        for (k, c) in self.hold_channels.iter_mut().enumerate() { *c = channels.v.get(k).copied().unwrap_or(0); }
    }

    /// `NF_DOORLOG=1` per-tick diagnostic (Task A-8-2): for every door, prints placement, kind,
    /// ch_open, ch_lock, flags (with the keep-open and spline bits decoded), the glist hash, whether
    /// that hash is registered in the collision world's `shapes` table (the bullet-collision check —
    /// `0xffffffff` means the placement's object hashcode is `u32::MAX` and the door has no collision),
    /// the current state, `pos01`, the auto-close `open_timer`, and the bounding sphere.
    ///
    /// `world` is the collision world built by `CollisionWorld::build` (so `has_shape` can be tested);
    /// pass `None` to skip the `in_shapes` column. Called from `Drones::tick` after `end_tick`.
    pub fn log_doors(&self, world: Option<&crate::world::CollisionWorld>) {
        if std::env::var_os("NF_DOORLOG").is_none() { return; }
        eprintln!("=== NF_DOORLOG frame: {} doors ===", self.list.len());
        eprintln!("idx placement kind grp ch_open ch_lock flags     appr locked glist        in_shapes state  pos01  open_timer sph.pos                      sph.r");
        for (i, d) in self.list.iter().enumerate() {
            let in_shapes = world.map_or(false, |w| w.has_shape(d.glist));
            eprintln!(
                "{i:<3} {:<9} {:<4} {:<3} {:<7} {:<7} {:#06x} {:<4} {:<6} {:#010x} {:<9} {:#04x}  {:5.3} {:<10} ({:7.2},{:7.2},{:7.2}) {:5.3}",
                d.placement, d.kind, d.group, d.ch_open, d.ch_lock, d.flags,
                d.approach, d.locked_flag, d.glist, in_shapes,
                d.state, d.pos01, d.open_timer,
                d.sph.0[0], d.sph.0[1], d.sph.0[2], d.sph.1
            );
            let keep = d.flags & 0x20 != 0;
            let spline = d.flags & 1 != 0;
            eprintln!("    bits: keep_open={} spline={} busy={} starts_open={}", keep, spline, d.flags & 0x10 != 0, d.flags & 4 != 0);
            eprintln!("    matrix row3 ({:.3},{:.3},{:.3}) keys {}", d.matrix[3][0], d.matrix[3][1], d.matrix[3][2], if d.keys.is_some() { "yes" } else { "NONE" });
        }
    }

    /// `NF_DOORLOG=1` throttled diagnostic (Task A-9-2): every 60 ticks (≈1 second of
    /// game time at 60 fps) prints, per door, the four values that decide whether the
    /// auto-close path can fire — `state`, `pos01`, `open_timer` — plus the `keep_open`
    /// bit and whether **any** actor in `actors` is currently blocking the proximity
    /// check `dist² < (r1+r2)²·2` (the same `blocked()` test `Door_Update`'s 360-tick
    /// branch and `Door_ReactToHit`'s anti-crush use).
    ///
    /// Why this exists: the auto-close path increments `open_timer` every tick, then at
    /// 360 ticks checks `!actors.iter().any(|a| blocked(sph, *a))` — if any actor is
    /// blocking it skips the close **but still resets `open_timer` to 0**. So a Bond
    /// standing in or near the doorway (within `(door_r + 0.4)·√2` of the sphere centre)
    /// keeps the timer cycling 0→360→0 forever and the door never closes. The unit
    /// tests don't catch this because they park the actor at `[0,1,5]` (door 07000005)
    /// or `[1000,1000,1000]` (door_research_07000001) — far outside the blocking radius.
    /// This diagnostic, run against the live sim, makes the failure visible.
    ///
    /// `actors` is the same list `Drones::tick` builds for `doors.update` (player first
    /// with radius 0.4, then every live non-corpse drone with radius 0.4). The closest
    /// actor's distance² and the door's threshold `(r1+r2)²·2` are printed alongside
    /// so the margin can be read directly. Called from `Drones::tick` after `end_tick`.
    pub fn log_doors_throttled(
        &self,
        world: Option<&crate::world::CollisionWorld>,
        actors: &[DoorActor],
        frame: u32,
    ) {
        if std::env::var_os("NF_DOORLOG").is_none() { return; }
        if frame == 0 || frame % 60 != 0 { return; }
        let _ = world;
        eprintln!("=== NF_DOORLOG @ frame {frame} ({} doors; {} actors) ===", self.list.len(), actors.len());
        eprintln!("idx placement state  pos01  open_timer keep_open blocked blocked_by                     sph.pos                       sph.r   closest_actor (d² / thresh)");
        for (i, d) in self.list.iter().enumerate() {
            let keep = d.flags & 0x20 != 0;
            // The live blocking sphere and the same `blocked()` test the auto-close path runs.
            let live = (transform(&d.matrix, d.sph_centre), d.sph.1);
            let (is_blocked, by) = match visible_actors(actors).find(|a| blocked(live, *a)) {
                Some(a) => (true, format!("({:7.2},{:7.2},{:7.2}) r={:.2}", a.0[0], a.0[1], a.0[2], a.1)),
                None => (false, String::from("-")),
            };
            // The closest actor and its squared distance, plus the threshold the test
            // compares against (`(r1+r2)²·2`); lets you read the margin directly.
            let closest = visible_actors(actors).min_by(|a, b| {
                let da = sub(live.0, a.0); let db = sub(live.0, b.0);
                (da[0]*da[0] + da[1]*da[1] + da[2]*da[2])
                    .partial_cmp(&(db[0]*db[0] + db[1]*db[1] + db[2]*db[2])).unwrap_or(std::cmp::Ordering::Equal)
            });
            let closest_s = closest.map(|a| {
                let dd = sub(d.sph.0, a.0);
                let dist2 = dd[0]*dd[0] + dd[1]*dd[1] + dd[2]*dd[2];
                let r = d.sph.1 + a.1;
                let thresh = r * r + r * r;
                format!("({:7.2},{:7.2},{:7.2}) d²={:6.2} thresh={:6.2}{}",
                    a.0[0], a.0[1], a.0[2], dist2, thresh, if dist2 < thresh { " [BLOCKS]" } else { "" })
            }).unwrap_or_default();
            eprintln!(
                "{i:<3} {:<9} {:#04x}  {:5.3} {:<10} {:<8} {:<7} {:<30} ({:7.2},{:7.2},{:7.2}) {:5.3}  {}",
                d.placement, d.state, d.pos01, d.open_timer, keep, is_blocked, by,
                d.sph.0[0], d.sph.0[1], d.sph.0[2], d.sph.1, closest_s
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn channels() -> Channels { Channels::default() }

    /// Two synthetic swing keys: closed at 0.5 (identity) and the ends ±90° about Y.
    fn swing_keys() -> Vec<Key> {
        let q = |a: f32| [0.0, (a / 2.0).sin(), 0.0, (a / 2.0).cos()];
        vec![
            Key { pos: [0.0, 0.0, 0.0], quat: q(-std::f32::consts::FRAC_PI_2) },
            Key { pos: [0.0, 0.0, 0.0], quat: q(0.0) },
            Key { pos: [0.0, 0.0, 0.0], quat: q(std::f32::consts::FRAC_PI_2) },
        ]
    }

    #[test]
    fn slerp_ends_are_the_keys() {
        // t = 0 and t = 1 give the keys back (the slerp weights are exactly 1 and 0 at the ends).
        let a = [0.0, 0.3826834, 0.0, -0.9238795];
        let b = [0.0, 0.9238795, 0.0, -0.3826834];
        for (t, q) in [(0.0, a), (1.0, b)] {
            let s = slerp(a, b, t);
            for k in 0..4 { assert!((s[k] - q[k]).abs() < 1e-6, "{s:?} at {t}"); }
        }
    }

    #[test]
    fn catmull_midpoint_hits_the_middle_key() {
        // Evenly spaced keys: the spline at t = 0.5 of the middle segment is the average of the neighbours.
        let k = |x: f32| Key { pos: [x, 0.0, 0.0], quat: [0.0, 0.0, 0.0, 1.0] };
        let m = catmull(k(0.0).pos, k(1.0).pos, k(2.0).pos, k(3.0).pos, 0.5);
        assert!((m[0] - 1.5).abs() < 1e-5, "{m:?}");
    }

    #[test]
    fn interp_equal_positions_takes_the_key() {
        let keys = swing_keys();
        let m = interp(&keys, 0, 2, 1.0, true).unwrap();   // the closed key
        assert!((m[0][0] - 1.0).abs() < 1e-6 && (m[3][0] - 0.0).abs() < 1e-6);
    }

    #[test]
    fn keyframe_mode_clamps_to_the_far_end() {
        let keys = vec![
            Key { pos: [0.0, 0.0, 0.0], quat: [0.0, 0.0, 0.0, 1.0] },
            Key { pos: [2.0, 0.0, 0.0], quat: [0.0, 0.0, 0.0, 1.0] },
        ];
        let m = interp(&keys, 0, 1, 0.25, false).unwrap();
        assert!((m[3][0] - 0.5).abs() < 1e-6, "{m:?}");
        assert!(interp(&[], 0, 1, 0.0, true).is_none(), "a NULL framelist writes nothing");
    }

    #[test]
    fn swing_quantises_to_whole_ticks() {
        // 1/30 → 0.5/(int)(0.5/(2/30)) = 0.5/7 = 1/14 [L: the snapshot's speed].
        let s = 1.0 / CREATE_FRAME_RATE;
        assert!((0.5 / ((0.5 / (s + s)) as i32 as f32) - 1.0 / 14.0).abs() < 1e-7);
    }

    /// One swing door built like `Doors::create` would (keys at the world origin).
    fn swing_door() -> Doors {
        let keys = swing_keys();
        let mut doors = Doors::default();
        doors.list.push(Door {
            placement: 0, kind: 0, group: 0, ch_open: 0, ch_lock: 0, straddle: false,
            snd_open: 285, snd_close: 284, snd_hit: 320, flags: 1, approach: 0, locked_flag: 0,
            keys: Some(keys), frames: 2, state: CLOSED, pos01: 0.5, speed: 1.0 / 14.0,
            dir: 0.0, swing: -1.0, timer: 0, open_timer: 0, msg_shown: false,
            matrix: quat_matrix([0.0, 0.0, 0.0, 1.0], [0.0, 0.0, 0.0]),
            inv_closed: [[1.0, 0.0, 0.0, 0.0], [0.0, 1.0, 0.0, 0.0], [0.0, 0.0, 1.0, 0.0], [0.0, 0.0, 0.0, 1.0]],
            pos: [0.0, 0.0, 0.0], sph: ([0.0, 1.1, 0.0], 1.28), sph_centre: [0.0, 1.1, 0.0], glist: 7,
        });
        doors
    }

    #[test]
    fn swing_opens_away_from_the_player_and_auto_closes() {
        let mut doors = swing_door();
        let mut ch = channels();
        let mut sounds = Vec::new();
        // A player in front (+Z) of the closed door at the origin: it swings to the −dir side (away).
        let actors = [([0.0, 1.0, 2.0], 0.4f32, false)];
        assert!(doors.activate(0, &actors, true, &mut ch, 0, &mut sounds));
        let d = &doors.list[0];
        assert_eq!(d.state, OPENING);
        assert_eq!(sounds, vec![(285, [0.0, 0.0, 0.0])], "the opening sound");
        // local.z of (0,1,2) in the closed (identity) frame is +2 → side +1 → dir = +swing = −1.
        assert_eq!(d.dir, -1.0);
        // The busy marker clears in `calc`; then the door travels 0.5 → 0 in 7 ticks and ends OPEN. The player
        // stays where he was: near enough that the opening door's anti-crush sees him (local.z = 2, past the
        // margin), far enough that his sphere (r = 0.4) does not block the door's (r = 1.28).
        doors.calc();
        let near = [([0.0, 1.0, 2.0], 0.4f32, false)];
        for _ in 0..8 { sounds.extend(doors.update(0, &mut ch, &near, 0)); doors.calc(); }
        assert_eq!(doors.list[0].state, OPEN, "reached the swung end");
        // Sister: the timer branch needs the state, not this run — an open door with nobody near (the player
        // walked off) closes after the 360-tick timer (`FRAME_RATE_INT·6`) from whenever it reached OPEN.
        let far = [([0.0, 1.0, 5.0], 0.4f32, false)];
        let mut guard = 0;
        while doors.list[0].state == OPEN && guard < 600 { sounds.extend(doors.update(0, &mut ch, &far, 0)); doors.calc(); guard += 1; }
        assert_eq!(doors.list[0].state, CLOSING, "the 6-second timer closed it (after {guard} ticks)");
        let closing = sounds.clone();
        assert!(closing.contains(&(284, [0.0, 0.0, 0.0])), "the closing sound: {sounds:?}");
    }

    #[test]
    fn a_locked_slide_door_only_thuds() {
        let mut doors = Doors::default();
        doors.list.push(Door {
            placement: 0, kind: 2, group: 0, ch_open: 0, ch_lock: 200, straddle: false,
            snd_open: 0xffff, snd_close: 0xffff, snd_hit: 320, flags: 0, approach: 0, locked_flag: 0,
            keys: None, frames: 0xffff, state: CLOSED, pos01: 0.0, speed: 1.0 / 30.0,
            dir: 0.0, swing: 0.0, timer: 0, open_timer: 0, msg_shown: false,
            matrix: [[1.0, 0.0, 0.0, 0.0], [0.0, 1.0, 0.0, 0.0], [0.0, 0.0, 1.0, 0.0], [0.0, 0.0, 0.0, 1.0]],
            inv_closed: [[0.0; 4]; 4], pos: [10.0, 0.0, 0.0], sph: ([10.0, 1.1, 0.0], 1.28), sph_centre: [10.0, 1.1, 0.0], glist: 8,
        });
        let mut ch = channels();
        let mut sounds = Vec::new();
        // Channel 200 off and the door closed: the locked feedback only.
        assert!(!doors.activate(0, &[], true, &mut ch, 0, &mut sounds));
        assert_eq!(sounds, vec![(320, [10.0, 0.0, 0.0])]);
        // The rising edge of channel 200 unlocks it (activates, clears the lock field). The edge is against
        // the channels at the end of the last tick (`switch_channels_prev`), so the set stays after it.
        let mut ch2 = channels();
        ch2.set(200, 1, 1);
        let mut doors2 = doors.clone();
        doors2.update(0, &mut ch2, &[], 1);
        assert_eq!(doors2.list[0].ch_lock, 0, "unlocked for good");
        // Sister: without the edge (the channel was already on last tick) the door stays locked.
        let mut doors3 = doors.clone();
        doors3.prev_channels[200] = 1;
        doors3.update(0, &mut ch2, &[], 1);
        assert_eq!(doors3.list[0].ch_lock, 200);
    }

    /// The lock channel's rising edge, full law [M 0x1363c8:96-107]: the label (0x2000003,
    /// 180 ticks — the level-gate half lives in the caller), then Door_Activate, then `+0x68 = 0`
    /// and the `+0x70 |= 0x10` once-only latch — a second tick with the channel held must not
    /// repeat any of it. Sister of `a_locked_slide_door_only_thuds`, which pins the ch_lock halves.
    #[test]
    fn a_lock_channel_rising_edge_labels_once_and_latches() {
        let mut doors = Doors::default();
        doors.list.push(Door {
            placement: 0, kind: 2, group: 0, ch_open: 0, ch_lock: 0x12, straddle: false,
            snd_open: 0xffff, snd_close: 0xffff, snd_hit: 320, flags: 0, approach: 0, locked_flag: 0,
            keys: None, frames: 0xffff, state: CLOSED, pos01: 0.0, speed: 1.0 / 30.0,
            dir: 0.0, swing: 0.0, timer: 0, open_timer: 0, msg_shown: false,
            matrix: [[1.0, 0.0, 0.0, 0.0], [0.0, 1.0, 0.0, 0.0], [0.0, 0.0, 1.0, 0.0], [0.0, 0.0, 0.0, 1.0]],
            inv_closed: [[0.0; 4]; 4], pos: [10.0, 0.0, 0.0], sph: ([10.0, 1.1, 0.0], 1.28), sph_centre: [10.0, 1.1, 0.0], glist: 8,
        });
        let mut ch = channels();
        ch.set(0x12, 1, 1); // 07000011: Lock #177's burn / TouchOnce #283's out channel
        let mut sounds: Vec<(u16, [f32; 3])> = Vec::new();
        doors.update(0, &mut ch, &[], 1);
        assert_eq!(doors.labels, vec![(LABEL_DOOR_UNLOCKED, 180.0)], "the unlock label, 180 ticks [M 0x1363c8:101-103]");
        assert_eq!(doors.list[0].ch_lock, 0, "the lock channel consumed [M 0x1363c8:106]");
        assert_ne!(doors.list[0].flags & 0x10, 0, "the once-only latch [M 0x1363c8:107]");
        let labels_after_edge = doors.labels.len();
        doors.update(0, &mut ch, &[], 2);
        assert_eq!(doors.labels.len(), labels_after_edge, "held channel: the latch stops a second label [M 0x1363c8:96-98]");
        assert_eq!(doors.list[0].ch_lock, 0);
    }

    /// A glist-0 door (the create fallback's `unwrap_or((0, translation, 0.0))` record) is E-DEAD:
    /// the game's use pick never admits an object without `obj+0xd8` [M Collide_PickObj 0x1eae80:88-90]
    /// and never even creates such a door [M parsemap_block_map_data_dynamic 0x1d1b68:93-95]. This
    /// test FAILED on the deleted placement-distance < 2.5 stand-in — at 2.0 out it activated.
    #[test]
    fn a_glist0_door_is_e_dead_at_the_old_stand_in_reach() {
        let mut doors = Doors::default();
        doors.list.push(Door {
            placement: 0, kind: 2, group: 0, ch_open: 0, ch_lock: 0, straddle: false,
            snd_open: 285, snd_close: 284, snd_hit: 320, flags: 0, approach: 0, locked_flag: 0,
            keys: None, frames: 0xffff, state: CLOSED, pos01: 0.0, speed: 1.0 / 30.0,
            dir: 0.0, swing: 0.0, timer: 0, open_timer: 0, msg_shown: false,
            matrix: [[1.0, 0.0, 0.0, 0.0], [0.0, 1.0, 0.0, 0.0], [0.0, 0.0, 1.0, 0.0], [0.0, 0.0, 0.0, 1.0]],
            // The create fallback's sphere: the placement translation, radius 0.0.
            inv_closed: [[0.0; 4]; 4], pos: [10.0, 0.0, 0.0], sph: ([10.0, 0.0, 0.0], 0.0), sph_centre: [10.0, 0.0, 0.0], glist: 0,
        });
        let mut ch = channels();
        // The use point 2.0 out (head + look): inside the old stand-in's 2.5, so the deleted arm
        // would have activated here.
        let (_, hits) = doors.player_use([12.0, 0.0, 0.0], [0.0, 0.0, 0.0], &mut ch, 0, &CollisionWorld::empty());
        assert_eq!(hits, 0, "glist-0 door must not answer the use pick (game non-admission [M 0x1eae80:88-90])");
        assert_eq!(doors.list[0].state, CLOSED, "still closed");
        // Point blank: 0.5 out — the use sphere touches the door's zero-radius fallback point, and
        // the game STILL does not admit it (no obj+0xd8). The stand-in also fired here (0.25 < 6.25).
        let mut doors2 = doors.clone();
        let (_, hits2) = doors2.player_use([10.5, 0.0, 0.0], [0.0, 0.0, 0.0], &mut ch, 0, &CollisionWorld::empty());
        assert_eq!(hits2, 0, "point blank, still non-admitted");
        assert_eq!(doors2.list[0].state, CLOSED);
    }

    /// Sister: the SAME record with its glist resolved answers the use pick through the live-sphere
    /// rule — the deletion did not touch the neighbouring glist!=0 behaviour.
    #[test]
    fn the_same_door_with_a_glist_still_opens_on_the_sphere_rule() {
        let mut doors = Doors::default();
        doors.list.push(Door {
            placement: 0, kind: 2, group: 0, ch_open: 0, ch_lock: 0, straddle: false,
            snd_open: 285, snd_close: 284, snd_hit: 320, flags: 0, approach: 0, locked_flag: 0,
            keys: None, frames: 0xffff, state: CLOSED, pos01: 0.0, speed: 1.0 / 30.0,
            dir: 0.0, swing: 0.0, timer: 0, open_timer: 0, msg_shown: false,
            matrix: [[1.0, 0.0, 0.0, 0.0], [0.0, 1.0, 0.0, 0.0], [0.0, 0.0, 1.0, 0.0], [0.0, 0.0, 0.0, 1.0]],
            inv_closed: [[0.0; 4]; 4], pos: [10.0, 0.0, 0.0], sph: ([10.0, 1.1, 0.0], 1.28), sph_centre: [10.0, 1.1, 0.0], glist: 8,
        });
        let mut ch = channels();
        // The use point 1.0 from the live sphere centre [10, 1.1, 0]: reach = 1.0 + 1.28 = 2.28 → hit.
        let (_, hits) = doors.player_use([10.0, 1.1, 0.0], [0.0, 0.0, 1.0], &mut ch, 0, &CollisionWorld::empty());
        assert_eq!(hits, 1, "the resolved door answers the use sphere");
        assert_eq!(doors.list[0].state, OPENING, "and starts opening");
    }

    #[test]
    fn group_members_follow_the_master() {
        let mut doors = Doors::default();
        for k in 0..2 {
            doors.list.push(Door {
                placement: k, kind: 2, group: 5, ch_open: 0, ch_lock: 0, straddle: false,
                snd_open: 0xffff, snd_close: 0xffff, snd_hit: 0xffff, flags: 0, approach: 0, locked_flag: 0,
                keys: None, frames: 0xffff, state: CLOSED, pos01: 0.0, speed: 1.0 / 30.0,
                dir: 0.0, swing: 0.0, timer: 0, open_timer: 0, msg_shown: false,
                matrix: [[1.0, 0.0, 0.0, 0.0], [0.0, 1.0, 0.0, 0.0], [0.0, 0.0, 1.0, 0.0], [0.0, 0.0, 0.0, 1.0]],
                inv_closed: [[0.0; 4]; 4], pos: [k as f32, 0.0, 0.0], sph: ([k as f32, 1.1, 0.0], 1.28), sph_centre: [k as f32, 1.1, 0.0], glist: 9,
            });
        }
        let mut ch = channels();
        let mut sounds = Vec::new();
        doors.activate(0, &[], true, &mut ch, 0, &mut sounds);
        doors.calc();
        assert_eq!(doors.list[1].state, OPENING, "the group member follows");
        assert_eq!(doors.list[1].pos01, doors.list[0].pos01);
        assert_eq!(doors.list[0].flags & 0x10, 0, "calc clears the busy markers");
    }

    /// A kind-2 slide door closing on the player's pick capsule reverses to OPENING through the
    /// collide step [M Door_ReactToHit 0x135980 kind-2 arm, fed by Collide_Update's obj+0xD0 forward].
    fn slide_door(state: u16, dir: f32) -> Doors {
        let mut doors = Doors::default();
        doors.list.push(Door {
            placement: 0, kind: 2, group: 0, ch_open: 0, ch_lock: 0, straddle: false,
            snd_open: 0xffff, snd_close: 0xffff, snd_hit: 0xffff, flags: 0, approach: 0, locked_flag: 0,
            keys: None, frames: 0xffff, state, pos01: if state & 4 != 0 { 1.0 } else { 0.0 },
            speed: 1.0 / 30.0, dir, swing: 0.0, timer: 0, open_timer: 0, msg_shown: false,
            matrix: [[1.0, 0.0, 0.0, 0.0], [0.0, 1.0, 0.0, 0.0], [0.0, 0.0, 1.0, 0.0], [0.0, 0.0, 0.0, 1.0]],
            inv_closed: [[0.0; 4]; 4], pos: [10.0, 0.0, 0.0], sph: ([10.0, 1.1, 0.0], 1.28), sph_centre: [10.0, 1.1, 0.0], glist: 8,
        });
        doors
    }

    /// The standing walk pick capsule at the door's threshold (the 0x194df0 case-0/6 arm).
    fn player_at(x: f32, z: f32) -> crate::door::DoorMover {
        let p = [x, 0.0, z];
        crate::door::DoorMover { a: [p[0], p[1] + 0.55, p[2]], b: [p[0], p[1] + 0.275 - 1.0347, p[2]], radius: 0.275, centre: p, player: true, valid: false }
    }

    #[test]
    fn the_collide_step_reverses_a_closing_slide_door_on_the_player() {
        let mut doors = slide_door(CLOSING, -1.0);
        let mut ch = channels();
        // Bond in the doorway: the pick capsule crosses the door's live sphere ×1.5.
        let evs = doors.collide_step(&[player_at(10.0, 0.0)], &mut ch, 0);
        let d = &doors.list[0];
        assert_eq!(d.state, OPENING, "the kind-2 reversal: CLOSING → OPENING [M 0x135980]");
        assert_eq!(d.dir, 1.0, "dir flipped");
        assert_eq!(d.timer, 2, "the 2-tick tail [M 0x135980:110]");
        assert_eq!(d.flags & 0x10, 0x10, "the busy marker [M 0x135980:111]");
        // Sister: an open door ignores the contact (the kind-2 arm needs CLOSING).
        let mut open = slide_door(OPEN, 1.0);
        let evs2 = open.collide_step(&[player_at(10.0, 0.0)], &mut ch, 0);
        assert_eq!(open.list[0].state, OPEN, "an open slide door does not reverse");
        assert!(evs.is_empty() && evs2.is_empty(), "no sounds (0xffff ids)");
        // And a far player makes no contact at all.
        let mut away = slide_door(CLOSING, -1.0);
        away.collide_step(&[player_at(30.0, 20.0)], &mut ch, 0);
        assert_eq!(away.list[0].state, CLOSING, "no contact, no reversal");
    }

    #[test]
    fn the_drone_gate_selects_only_a_valid_guard() {
        let mut ch = channels();
        // A dead guard's contact (invalid) does not reverse; a live one behind it does [M 0x135980:53-60].
        let mut doors = slide_door(CLOSING, -1.0);
        let dead = crate::door::DoorMover { a: [10.0, 0.0, 0.0], b: [10.0, 0.0, 0.0], radius: 0.4, centre: [10.0, 0.9, 0.0], player: false, valid: false };
        let alive = crate::door::DoorMover { a: [10.0, 0.0, 0.0], b: [10.0, 0.0, 0.0], radius: 0.4, centre: [10.0, 0.9, 0.0], player: false, valid: true };
        doors.collide_step(&[dead], &mut ch, 0);
        assert_eq!(doors.list[0].state, CLOSING, "an invalid (dead) drone contact never reverses");
        doors.collide_step(&[dead, alive], &mut ch, 0);
        assert_eq!(doors.list[0].state, OPENING, "the first valid drone in list order reverses it");
        // The 2-tick lockout: an immediate re-run does nothing (timer != 0) [M 0x135980:44].
        let mut doors2 = slide_door(CLOSING, -1.0);
        doors2.collide_step(&[alive], &mut ch, 0);
        assert_eq!(doors2.list[0].state, OPENING);
        doors2.collide_step(&[alive], &mut ch, 0);
        assert_eq!(doors2.list[0].state, OPENING, "still OPENING (the reversal fired once; the timer gates the tail)");
        assert_eq!(doors2.list[0].timer, 2, "the tail did not re-fire");
    }

    #[test]
    fn kind1_ignores_contacts_and_kind0_reverses_by_the_swing_plane() {
        let mut ch = channels();
        // Kind 1: contacts but nothing happens [M 0x135980:75].
        let mut k1 = slide_door(CLOSING, -1.0);
        k1.list[0].kind = 1;
        k1.collide_step(&[player_at(10.0, 0.0)], &mut ch, 0);
        assert_eq!(k1.list[0].state, CLOSING);
        assert_eq!(k1.list[0].timer, 0, "kind 1 never reaches the tail");
        // Kind 0 swing, CLOSING (pos01 0.6 heading down, dir −1 — Door_Activate's close path), the
        // hitter where the leaf closes into (local.z·swing = +2, negated past half travel → f < −eps)
        // → OPENING [M 0x135980:95-98].
        let mut doors = swing_door();
        doors.list[0].state = CLOSING;
        doors.list[0].pos01 = 0.6;
        doors.list[0].dir = -1.0;
        doors.collide_step(&[player_at(0.0, -2.0)], &mut ch, 0);
        assert_eq!(doors.list[0].state, OPENING, "the swing anti-crush reopens");
        assert_eq!(doors.list[0].dir, 1.0, "dir flipped");
        // Sister: the same door with nobody near finishes the close in update (state → CLOSED).
        let mut alone = swing_door();
        alone.list[0].state = CLOSING;
        alone.list[0].pos01 = 0.6;
        alone.list[0].dir = -1.0;
        let far = [([100.0, 1.0, 100.0], 0.4f32, false)];
        for _ in 0..12 { alone.update(0, &mut ch, &far, 0); alone.calc(); }
        assert_eq!(alone.list[0].state, CLOSED, "a door with nobody in it closes");
    }
}

#[cfg(test)]
mod world_helper_tests {
    use super::*;

    /// `Door_IsLocked` 0x136810 [M :10-16]: a wired lock channel reads `switch_channels[ch]`,
    /// locked = OFF; channel 0 (unwired) is never locked.
    #[test]
    fn door_is_locked_reads_the_lock_channel() {
        let mk = |ch_lock: u16| Door {
            placement: 0, kind: 0, group: 0, ch_open: 0, ch_lock, straddle: false,
            snd_open: 0, snd_close: 0, snd_hit: 0, flags: 0, approach: 0, locked_flag: 0,
            keys: None, frames: 0, state: CLOSED, pos01: 0.0, speed: 1.0, dir: 0.0, swing: 0.0,
            timer: 0, open_timer: 0, msg_shown: false, matrix: [[0.0; 4]; 4],
            inv_closed: [[0.0; 4]; 4], pos: [0.0; 3], sph: ([0.0; 3], 0.0),
            sph_centre: [0.0; 3], glist: 0,
        };
        let mut doors = Doors::default();
        doors.list.push(mk(0x5A)); // a wired lock channel
        doors.list.push(mk(0)); // unwired
        let mut ch = Channels::default();
        assert!(doors.is_locked(0, &ch), "channel OFF -> locked [M 0x136810:14-16]");
        assert!(!doors.is_locked(1, &ch), "unwired -> never locked [M 0x136810:12-13]");
        // sister: the same channel turned ON unlocks
        ch.set(0x5A, 1, 0);
        assert!(!doors.is_locked(0, &ch), "channel ON -> unlocked [M 0x136810:14-16]");
    }
}
