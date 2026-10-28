//! Searchlights (docs/systems/world-objects.md "Searchlights"): `Searchlight_Create` 0x1cb4f8 /
//! `Searchlight_Update` 0x1cb900, ported from the disassembly [M] and checked against the live layout in
//! `work/ee_start_05.bin` [L] (29 parts on 07000005: 7 mounts, 7 heads, 6 beams, 7 coronas, 2 spots).
//! The game's per-part nodes come from the same free/in-use pool machinery as the lights
//! (`DList_MoveFromFree2InUse` 0x1cccb8 is called by `Searchlight_Create` [M]); the port's per-part
//! Vec records replace it.
//!
//! The bullet contact path (U-041, docs/systems/searchlight-bullet.md): a flying bullet's HITTEST picks the
//! head's world sphere each tick and the head's update breaks on a class-5 contact with damage > 0.0 [M].
//!
//! Not ported yet (G-033): the spot's dynamic light (illumination is renderer work) and `Gas_Init` smoke
//! for broken lights (no particle system; its 6 RNG draws per puff are therefore not consumed either).
//! `Control_NextLOD`'s model swap on kill/break (obj+0xD8 ← *(glist+0x40) [M 0x1cb900:138,454 →
//! 0x1349b0:8-12]; the block-0x27 chain, 07000006's heads → OBJECT_SearchLightBroken [D]) is pinned as
//! `Part::lod_next` — drawing the swapped variant is the renderer's side (nf-view resolves
//! `LevelMap::lod_target` from the part's placement).

use crate::drone::{Data, Player, Rng};
use crate::trig::{angle_difference, atan2_approx, sinf, PI, TAU};
use nf_formats::placement::{object_matrix, view_rot_trans_matrix, Placement};
use nf_formats::ps2float::{add_noguard, mul};

pub type V3 = [f32; 3];
pub type Mat4 = [[f32; 4]; 4];

/// The game's degrees→radians constant (`lui 3c8e / ori fa36` at 0x1cb604) — π/180 plus one ulp [M].
pub const DEG2RAD: f32 = f32::from_bits(0x3c8e_fa36);
/// The sweep step 2π/480 (0x3c567751 at 0x1cba68) [M].
const SWEEP: f32 = f32::from_bits(0x3c56_7751);
/// The tracking gain 0.15 (0x3e19999a at 0x1cbb40) [M].
const STEER: f32 = f32::from_bits(0x3e19_999a);
/// `FRAME_RATE` (read from 0x31d0d0) × 15 s: ticks the spot may lose Bond before the head sweeps again [M].
const LOST_LIMIT: f32 = 60.0 * 15.0;

/// One searchlight part object. Kinds [M]: 0 mount, 1 head (the controller), 2 spot (ground circle + light),
/// 3 beam (the visible cone), 4 corona (billboard at the lamp).
#[derive(Clone, Debug)]
pub struct Part {
    /// Index into the main map's placements — the port's handle to the SAME map object whose
    /// celglist the game stores at `obj+0xD8` [M Searchlight_Create 0x1cb4f8:73-76
    /// `*(obj+0xd8) = param_4`; the glist comes from the placement's map-entity id,
    /// parsemap_block_map_data_dynamic 0x1d1b68:79,95-96 `pCelList + id*0x50` -> case 0xDE
    /// 0x1d04f0:342-343 passes it through]. nf-view resolves the index back to the map object
    /// (and its glist) for the draw and the gfx bounds; the part is also linked into the cel's
    /// DYNAMIC object list in the game (`control_link_object_to_cel` 0x1c9680 -> 0x1346b8:16-24),
    /// which is what makes it bullet-pickable while the spot ray's mask 0x18 skips it.
    pub placement: usize,
    pub kind: u8,
    /// `obj+0x30`. A spot is moved −10000 at create until the first ray hit repositions it [M].
    pub pos: V3,
    /// The placement's rotation.y (`instance+0x14`): the sweep's yaw centre.
    pub base_yaw: f32,
    /// `instance+0x34/35/36`: on/off/alarm switch channels.
    pub on: u8, pub off: u8, pub alarm: u8,
    /// `instance+0x38/3c`: half-range and mid angle (radians).
    pub half: f32, pub mid: f32,
    /// `instance+4`: the head's sweep phase / the spot's ticks-not-seen counter.
    pub phase: f32,
    /// `obj+0xFC`: drawn (the game's `View_SetDrawInAllViews` / `…NoViews`).
    pub drawn: bool,
    /// `obj+0x90`: the part's world matrix (heads and mounts via `Mat_Align2Dir` against world up, the beam
    /// against Bond, the spot from the hit normal; coronas only carry the position).
    pub matrix: Mat4,
    /// Heads: `obj+0xF4`. 0 sweep, 1 tracking, 2 broken, 3 off.
    pub state: u8,
    /// Heads: `obj+0x50 / +0x54`.
    pub pitch: f32, pub yaw: f32,
    /// Heads: the linked parts by kind (`instance+0x20 + 4·kind`).
    pub links: [Option<usize>; 5],
    /// The off-channel kill is one-shot (`obj+0xFE |= 1`): the light can never re-light.
    pub killed: bool,
    /// Spots: the light node's flicker seed (`Rand_Rand(0)` at create); `None` only after a kill deletes it.
    pub light: Option<u32>,
    /// Coronas: the alpha byte (`obj+0x106`) and scale (`obj+0xE8`).
    pub alpha: u8, pub scale: f32,
    /// Heads and mounts: the object's gfx box [D parsed bounds; [L ee_start_05.bin head 016dfe70
    /// ~0.5 units]] — kept as parsed data (the tests' fixtures pin it). No logic reads it any more:
    /// the spot ray's own-housing box skip is GONE (the game's mask 0x18 ray tests static geometry
    /// only and its PickObj refusal is dead — see the spot-ray comment), and the bullet path tests
    /// the world sphere below (U-041 ported).
    pub box_min: V3, pub box_max: V3,
    /// The part's glist bounding sphere, LOCAL (`glist+0x10` centre / `+0x1C` radius — the map object's
    /// `centre`/`radius` [D measured on AI_ShadowVolume: centre (0,1.5,0), radius 2.598 = the box half
    /// diagonal]). `Control_BuildWorldSph` 0x133070 poses it: world centre = obj matrix · +0x10, radius
    /// = +0x1C · |obj+0xE8| with obj+0xE8 = 1.0 from control_init_object 0x134630:17 [M] — the sphere
    /// the flying bullet's pick tests (Collide_PickObj 0x1eae80 type 0x201) and what a bullet contact on
    /// the head's `obj+0xD0` list is made of (U-041).
    pub sph_centre: V3, pub sph_radius: f32,
    /// The placement translation BEFORE the kind-2 −10000 move — the psiLight node's static +0x10,
    /// the light's upload position for the part's whole life [M 0x1cb4f8:154-167 writes the node
    /// from obj+0x30 before the move; psiLight_SetLights 0x10e588 uploads node+0x10].
    pub spawn_pos: V3,
    /// The create tick in the sim's own timeline — the game's obj+0xEC stores GameState+52 at
    /// create (control_init_object 0x134630:15), the state-2 smoke's phase term
    /// `(GameState+52 + obj+0xEC) % 7 == 0` [M 0x1cb900:375]. The port creates at frame 0.
    pub created: u32,
    /// Heads: the game ran `Control_NextLOD` on this object — the off-kill [M Searchlight_Update
    /// 0x1cb900:138] and the bullet break [M :454] both call it, and `Control_NextLOD` 0x1349b0:8-12
    /// swaps obj+0xD8 to `*(glist+0x40)` when nonzero. The runtime glist+0x40 is the block-0x27 LOD
    /// chain target (`parsemap_block_LOD` 0x1d1f10:21 stores `pCelList + index·0x50`; the disc's
    /// per-object first-dword index, −1 = none) — 07000006's four heads all resolve object 2
    /// OBJECT_SearchLight1 → object 131 OBJECT_SearchLightBroken [D block 0x27], the dark-lens
    /// variant the off light draws while its housing stays visible. The renderer draws the chain
    /// target's mesh for the part when this is set (nf-view resolves `LevelMap::lod_target` from
    /// the part's placement; the swap is a draw change — the sim only pins the state).
    pub lod_next: bool,
}

/// All searchlight parts of a level, in placement (creation) order.
#[derive(Clone, Debug, Default)]
pub struct Searchlights {
    pub parts: Vec<Part>,
}

/// One flying bullet's per-tick pick, as `Collide_Update` sees it: the flight segment [from, to] (the
/// HITTEST's +0x20/+0x30 — the game writes A = obj+0x30 and B = obj+0x30 + dir·step with the step
/// clamped to the range and floored at 1.0 [M Bullet_update 0x12e7b0 tail, disasm 0x12efd8-0x12f188]),
/// the contact damage the HITTEST carries (+0x90 = `weapon_data[w]+0x0C` with the id overrides [M],
/// `WeaponData::bullet_contact_damage`), and the pick MODE (+0x94: 0x201 = the segment test, 0x101 =
/// the point test the slow-tick arm takes when the raw step < 0.125 and the weapon id is not one of
/// the always-segment ids 0x2C-0x31/0x62/0x6D [M 0x12e7b0 disasm 0x12f0c8-0x12f134]).
pub struct Shot {
    pub from: V3,
    pub to: V3,
    pub damage: f32,
    /// True = the 0x201 segment pick; false = the 0x101 point pick at `from` with margin 0.25 + the
    /// object's radius.
    pub segment: bool,
}

/// `Searchlight_Create`'s part kinds for a placement list, in creation order (used by create and nf-view).
pub fn part_placements(placements: &[Placement]) -> Vec<usize> {
    placements.iter().enumerate().filter(|(_, p)| p.flags & 0xff == 0xde && p.flags & 0xa000 == 0).map(|(i, _)| i).collect()
}

/// The create-time (p4, p5) → (half, mid), operation for operation [M, 0x1cb5c0–0x1cb680; L 7/7 heads exact]:
/// `lo = float(min)·DEG2RAD`, `hi = float(max)·DEG2RAD` (EE muls), `mid = (hi+lo)·0.5`, `half = (hi−lo)·0.5`
/// (EE add/sub; the halving is exact). Not `(max−min)·π/360` — the constant and the op order both differ.
pub fn half_mid(p4: u32, p5: u32) -> (f32, f32) {
    let (lo, hi) = if p4 <= p5 { (p4 as f32, p5 as f32) } else { (p5 as f32, p4 as f32) };
    let lo = mul(lo, DEG2RAD);
    let hi = mul(hi, DEG2RAD);
    let mid = mul(add_noguard(hi, lo, true), 0.5);
    let half = mul(add_noguard(hi, -lo, true), 0.5);
    (half, mid)
}

/// The state-0 sweep (`Searchlight_Update` 0x1cba60) [M; L five golden heads bit-exact]: pitch and yaw from the
/// phase, then the object's angles normalised to (−π, π] — the live heads with mid > π store the pitch 2π below
/// the raw sum (head 016e03f0: sinf·half + mid = 4.445 stored as −1.838) [L]. `phase` is the value *before* this
/// tick's advance (`+= FRAME_RATE_MUL`); the `fmodf(phase, 480)` call's result is discarded (the store sits in
/// the jal's delay slot) — the phase grows unbounded and `PS2Sinf`'s own wrap keeps precision [M].
pub fn sweep(phase: f32, half: f32, mid: f32, base_yaw: f32) -> (f32, f32) {
    let x = mul(phase, SWEEP);
    let mut pitch = add_noguard(mul(sinf(x), half), mid, true);
    let mut yaw = add_noguard(mul(sinf(add_noguard(x, crate::trig::HALF_PI, true)), half), base_yaw, true);
    while pitch > PI { pitch = add_noguard(pitch, -TAU, true); }
    while pitch <= -PI { pitch = add_noguard(pitch, TAU, true); }
    while yaw > PI { yaw = add_noguard(yaw, -TAU, true); }
    while yaw <= -PI { yaw = add_noguard(yaw, TAU, true); }
    (pitch, yaw)
}

/// `Vec_Spherical_2_Cartesian` (0x1e3948) [M]: `(r·sin(yaw)·cos(p), r·sin(p), r·cos(yaw)·cos(p))`, the sines
/// from the game's `PS2Sinf` and the products EE-chopped.
pub fn spherical(r: f32, yaw: f32, p: f32) -> V3 {
    let (sy, cy, sp, cp) = (sinf(yaw), crate::trig::cosf(yaw), sinf(p), crate::trig::cosf(p));
    [mul(mul(r, sy), cp), mul(r, sp), mul(mul(r, cy), cp)]
}

fn sub3(a: V3, b: V3) -> V3 { [a[0] - b[0], a[1] - b[1], a[2] - b[2]] }
fn dot3(a: V3, b: V3) -> f32 { a[0] * b[0] + a[1] * b[1] + a[2] * b[2] }
fn cross3(a: V3, b: V3) -> V3 { [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]] }

/// `Mat_Align2Dir` (0x1df2a8) [M; cross order pinned against the live beam matrix 016e0130, L]:
/// row 2 = normalize(dir), row 0 = normalize(cross(up, row2)), row 1 = normalize(cross(row2, row0)).
/// The fallback replaces `up` when |up·row2| > 0x3f7fbe77 (0.998993 — `lui at,3f7f; ori at,at,be77`
/// [M 0x1df2a8 disasm 0x1df320-0x1df354], not 0.999). A degenerate vector ZEROES the row — the
/// game's sqrt(x)==0 arms store 0.0 to all three components [M 0x1df2a8 disasm 0x1df2d8-0x1df310 /
/// 0x1df3a8-0x1df3e0: swc1 f3(=0.0) to the vector's slots] (the old [1,0,0]/[0,1,0] fallback, long
/// replaced, snapped the beam basis onto the world axes where the game's own math keeps the
/// (exact-FP) cross direction sound and continuous — the light flipped on its side).
/// Normalization multiplies
/// by the reciprocal [M 0x1df2f4/0x1df3c4: div.s f0,1.0,len; the VMUL], it does not divide.
pub fn align2dir(dir: V3, up: V3, fallback: V3) -> Mat4 {
    let len = dir[0] * dir[0] + dir[1] * dir[1] + dir[2] * dir[2];
    let z = if len == 0.0 { [0.0, 0.0, 0.0] } else { let r = 1.0 / len.sqrt(); [dir[0] * r, dir[1] * r, dir[2] * r] };
    let ulen = up[0] * up[0] + up[1] * up[1] + up[2] * up[2];
    let un = if ulen == 0.0 { [0.0, 0.0, 0.0] } else { let r = 1.0 / ulen.sqrt(); [up[0] * r, up[1] * r, up[2] * r] };
    let u = if dot3(un, z).abs() > f32::from_bits(0x3f7f_be77) { fallback } else { un };
    let r0 = cross3(u, z);
    let r0l = r0[0] * r0[0] + r0[1] * r0[1] + r0[2] * r0[2];
    let r0 = if r0l == 0.0 { [0.0, 0.0, 0.0] } else { let r = 1.0 / r0l.sqrt(); [r0[0] * r, r0[1] * r, r0[2] * r] };
    let r1 = cross3(z, r0);
    let r1l = r1[0] * r1[0] + r1[1] * r1[1] + r1[2] * r1[2];
    let r1 = if r1l == 0.0 { [0.0, 0.0, 0.0] } else { let r = 1.0 / r1l.sqrt(); [r1[0] * r, r1[1] * r, r1[2] * r] };
    [[r0[0], r0[1], r0[2], 0.0], [r1[0], r1[1], r1[2], 0.0], [z[0], z[1], z[2], 0.0], [0.0, 0.0, 0.0, 0.0]]
    // Row 3 stays zeroed as in the live matrix [L]: `Mat_Align2Dir` writes only the rotation rows and
    // `Mat_SetTrans` the translation's x/y/z — the w slot keeps its cleared value. The port's `transform`
    // ignores w.
}

impl Searchlights {
    /// `Searchlight_Create` for every 0xDE placement of the main map, in placement order [M]. Each part draws
    /// `Rand_Rand(480)` for its phase; each spot another `Rand_Rand(0)` for its light's flicker seed — both from
    /// the shared stream, in this order [M, critical]. The linking walk sees the parts created so far (the cel
    /// list, newest first): a spot created before its head is moved −10000 by its own create and can never link
    /// (the measured orphan rule, L: spot 016e8ec0 on 07000005).
    pub fn create(placements: &[Placement], rng: &mut Rng,
                  bounds: impl Fn(&Placement) -> (V3, V3, V3, f32)) -> Searchlights {
        let mut s = Searchlights::default();
        for i in part_placements(placements) {
            let p = &placements[i];
            let param = |k: u32| p.params.iter().find(|(j, _)| *j == k).map(|&(_, v)| v).unwrap_or(0);
            let kind = param(0) as u8;
            let phase = rng.rand(480) as f32;
            let (box_min, box_max, sph_centre, sph_radius) = bounds(p);
            let mut part = Part {
                placement: i, kind, pos: p.translation, base_yaw: p.rotation[1],
                on: param(1) as u8, off: param(2) as u8, alarm: param(3) as u8,
                half: 0.0, mid: 0.0, phase, drawn: kind < 2,
                // Searchlight_Create builds obj+0x90 with CCO(0) → View_RotTransMatrix
                // [M 0x1cb4f8:25, 0x1344e0:26-29].
                matrix: view_rot_trans_matrix(p.rotation, p.translation),
                state: if param(1) != 0 { 3 } else { 0 },
                pitch: 0.0, yaw: p.rotation[1], links: [None; 5], killed: false,
                light: None, alpha: 0xff, scale: 1.0, box_min, box_max,
                sph_centre, sph_radius,
                spawn_pos: p.translation,
                created: 0,
                lod_next: false,
            };
            let (half, mid) = half_mid(param(4), param(5));
            part.half = half;
            part.mid = mid;
            // The linking loop [M, disassembly 0x1cb6c0]: the cel list, newest first (`control_link_object_to_cel`
            // prepends); skip self, skip class ≠ 0x22, link only within 3 units (strict). Both stores are
            // unconditional, so the walk (newest → oldest) leaves the OLDEST match linked. Cels are approximated
            // by the distance here: the parts of one light sit within ~1.5 units and different lights are far
            // apart (07000005's measured links reproduce). The kind-2 branch (the light node and the −10000
            // move) runs AFTER this walk [M, 0x1cb79c], so a spot created after its head links while it is
            // still in place; a spot created before its head is moved away by its own create and the later
            // head's walk never sees it (the measured orphan rule, L: spot 016e8ec0 on 07000005).
            let me = s.parts.len();
            for j in (0..me).rev() {
                let d = sub3(s.parts[j].pos, part.pos);
                if d[0] * d[0] + d[1] * d[1] + d[2] * d[2] >= 9.0 { continue; }
                if part.kind == 1 {
                    part.links[s.parts[j].kind as usize] = Some(j);
                    // A dark head's non-mount parts start dark (`View_SetDrawInNoViews`) [M].
                    if part.on != 0 && s.parts[j].kind != 0 { s.parts[j].drawn = false; }
                } else if s.parts[j].kind == 1 {
                    s.parts[j].links[part.kind as usize] = Some(me);
                    if s.parts[j].on != 0 && part.kind != 0 { part.drawn = false; }
                }
            }
            if kind == 2 {
                // The light node (`psiLight_Create`): radius 500, type 2, the flicker seed `Rand_Rand(0)`
                // (the second draw of this part's create) [M]. Illumination is renderer work (G-033).
                part.light = Some(rng.rand(0));
                // The spot waits below the world for its first ray hit [M].
                part.pos = [part.pos[0] - 10000.0, part.pos[1] - 10000.0, part.pos[2] - 10000.0];
                part.drawn = false;
                part.matrix = object_matrix(p.rotation, part.pos);
            }
            s.parts.push(part);
        }
        s
    }

    /// The off-channel kill [M Searchlight_Update 0x1cb900:92-138]: state 3, the SPOT/BEAM/CORONA
    /// links deleted (`obj+0xFE |= 1` — the movement pass removes the objects, 0x1336f8:88-91) with
    /// their link slots (data+0x28/2c/30) cleared, the spot's psiLight node freed, the on/off/alarm
    /// channel bytes (data+0x34/35/36) zeroed, and `Control_NextLOD` on the head (the model swap).
    /// The MOUNT link (data+0x20) and the head's own draw are untouched — the housing stays [M the
    /// kill walk covers only psVar2+0x14/0x16/0x18]. After the kill the head can never re-light: its
    /// own on-byte is 0 and the state-3 arm reads exactly that byte [M :399].
    fn off_kill(&mut self, head: usize) {
        let links = self.parts[head].links;
        {
            let h = &mut self.parts[head];
            h.state = 3;
            h.killed = true;
            h.links[2] = None;
            h.links[3] = None;
            h.links[4] = None;
            h.on = 0;
            h.off = 0;
            h.alarm = 0;
            h.lod_next = true;
        }
        for p in [links[2], links[3], links[4]].into_iter().flatten() {
            self.parts[p].drawn = false;
            if self.parts[p].kind == 2 { self.parts[p].light = None; }
        }
    }

    /// `Searchlight_Update` for one head, in the control list's order (called from `Drones::tick`). Returns the
    /// `Sound_Play3D` events (id at position): 0xE5 the relay clunk when a channel re-lights the head, 0xE6 the
    /// glass break when a bullet kills it [M]. `shots` = this tick's flying player bullets (the class-5 contacts
    /// `Collide_Update` distributes onto the head's `obj+0xD0` list the same frame [M 0x1ea398:160-215]).
    pub fn update(&mut self, d: &Data, player: &Player, channels: &mut crate::triggers::Channels, head: usize,
                  shots: &[Shot], rng: &mut Rng, frame: u32) -> Vec<(u32, V3)> {
        let mut sounds = Vec::new();
        let (pos, base_yaw, half, mid, on, off) = {
            let h = &self.parts[head];
            (h.pos, h.base_yaw, h.half, h.mid, h.on, h.off)
        };
        // The kill check, any state [M Searchlight_Update 0x1cb900:92-138]: state 3, the SPOT/BEAM/CORONA
        // links deleted (`obj+0xFE |= 1` — the movement pass removes the objects, 0x1336f8:88-91) with
        // their link slots (data+0x28/2c/30) cleared, the spot's psiLight node freed, the on/off/alarm
        // channel bytes (data+0x34/35/36) zeroed, and `Control_NextLOD` on the head (the model swap).
        // The MOUNT link (data+0x20) and the head's own draw are untouched — the housing stays [M the
        // kill walk covers only psVar2+0x14/0x16/0x18]. After the kill the head can never re-light: its
        // own on-byte is 0 and the state-3 arm reads exactly that byte [M :399].
        if off != 0 && channels.on(off as u32) {
            self.off_kill(head);
            return sounds;
        }
        let state = self.parts[head].state;
        let mut parts_now = false; // run the spot/beam/corona updates this tick (states 0/1 and the 3→0 turn-on)
        match state {
            // State 0: the sweep [M].
            0 => {
                let h = &mut self.parts[head];
                let (pitch, yaw) = sweep(h.phase, half, mid, base_yaw);
                h.pitch = pitch;
                h.yaw = yaw;
                h.phase += 1.0; // FRAME_RATE_MUL; the game's fmodf result is discarded (store in the delay slot)
                parts_now = true;
            }
            // State 1: hold the alarm channel and steer to Bond [M, disassembly 0x1cbb00].
            1 => {
                channels.set(self.parts[head].alarm as u32, 1, frame);
                let h = &mut self.parts[head];
                let dx = player.pos[0] - pos[0];
                let dy = player.pos[1] - pos[1];
                let dz = player.pos[2] - pos[2];
                // G-046: the game's steer targets are the PS2 libm atan2f [M Searchlight_Update 0x1cb900:150
                // atan2f(dx, dz); :166 atan2f((py − sy) − 1.0, SQRT(dx² + dz²)) — the products/sums EE, the √
                // sqrt.s] — not host atan2, and not ATAN2_APPROX.
                let t_yaw = crate::trig::atan2f_libm(dx, dz);
                let t_pitch = -crate::trig::atan2f_libm(
                    crate::trig::ee_sub(dy, 1.0),
                    nf_formats::ps2float::sqrt(crate::trig::ee_add(
                        nf_formats::ps2float::mul(dx, dx),
                        nf_formats::ps2float::mul(dz, dz),
                    )),
                );
                h.yaw += angle_difference(h.yaw, t_yaw) * STEER;
                h.pitch += angle_difference(h.pitch, t_pitch) * STEER;
                // Clamped to the sweep cone: yaw within base ± half, pitch within mid ± half.
                let dyaw = angle_difference(h.yaw, base_yaw);
                if dyaw > half { h.yaw = base_yaw - half; } else if dyaw < -half { h.yaw = base_yaw + half; }
                let dpitch = angle_difference(h.pitch, mid);
                if dpitch > half { h.pitch = mid - half; } else if dpitch < -half { h.pitch = mid + half; }
                parts_now = true;
            }
            // State 2: broken — the head droops, nothing else updates [M]. The game's state-2 arm returns on
            // every path before the contact walk [M 0x1cb900:373-393]; the walk's own state-2 gate below makes
            // that fall-through a no-op here.
            2 => {
                let h = &mut self.parts[head];
                if h.pitch < 0.8 { h.pitch += 0.05; }
                // Gas_Init smoke every 7th drawn frame is not ported (no particle system); its RNG draws are
                // not consumed either (G-033).
                let _ = rng;
            }
            // State 3: off — the on-channel re-lights it with the relay clunk [M].
            _ => {
                if on != 0 && channels.on(on as u32) {
                    let links = self.parts[head].links;
                    self.parts[head].state = 0;
                    for p in links.into_iter().flatten() { self.parts[p].drawn = true; }
                    sounds.push((0xe5, pos));
                    parts_now = true; // the turn-on falls straight into the part updates [M]
                }
            }
        }
        if !parts_now {
            // The contact walk still runs in state 3 (with or without the relight) [M 0x1cb900:399-406].
            self.bullet_check(head, shots, &mut sounds);
            return sounds;
        }
        // The mount follows the head's yaw [M; L: mount 016dffd0's yaw = its head's].
        if let Some(m) = self.parts[head].links[0] {
            let yaw = self.parts[head].yaw;
            self.parts[m].yaw = yaw;
        }
        // The head's and mount's matrices: the framework's `Mat_Align2Dir` of the aim (the mount's pitch is 0)
        // against world up — [L] pinned on the snapshot's head 016dfe70, whose row 2 is the aim direction and
        // row 0 ⊥ with a zero y component.
        {
            let (yaw, pitch) = (self.parts[head].yaw, self.parts[head].pitch);
            let mut m = align2dir(spherical(1.0, yaw, -pitch), [0.0, 1.0, 0.0], [0.0, 1.0, 0.0]);
            m[3] = [pos[0], pos[1], pos[2], 0.0];
            self.parts[head].matrix = m;
        }
        if let Some(m) = self.parts[head].links[0] {
            let (p, yaw) = (self.parts[m].pos, self.parts[m].yaw);
            let mut mm = align2dir(spherical(1.0, yaw, 0.0), [0.0, 1.0, 0.0], [0.0, 1.0, 0.0]);
            mm[3] = [p[0], p[1], p[2], 0.0];
            self.parts[m].matrix = mm;
        }
        // The spot [M]: a 100-unit ray along the aim; on a hit the spot sits on the surface, oriented by the
        // hit normal. No hit: hidden. The ray is `Collide_RayIntersect(pos, dir, head cel, head obj, NULL,
        // hits, 0, mask 0x18)` [M 0x1cb900:201-208, disasm 0x1cbdb0-0x1cbe2c]. Mask 0x18's bit 0x10 makes
        // Collide_Pick SKIP the per-cel dynamic-object walk and the ForcedList [M 0x1eb320:314-328,
        // 370-374; disasm 0x1eb480-0x1eb49c] — the ray tests ONLY the map-static nodes (cel+0x18) and the
        // cel's intrinsic geometry; the param_3 == param_2 source refusal lives only in Collide_PickObj
        // [M 0x1eae80:57-59] and is DEAD on this path (PickObj runs for dynamic objects only). The
        // searchlight parts are dynamic objects [M 0x1cb4f8:73-76 obj+0xD8 = the placement's glist;
        // control_link_object_to_cel 0x1c9680 -> 0x1346b8:16-24], so the game's ray never tests the
        // parts' own meshes — and nothing excludes the level's static housing geometry, which the ray
        // legitimately hits (the port's static world measures the same first hit ~0.68 units below every
        // 07000005 head, [L probe]). The old own-housing box skip stands in for a rule the game does not
        // have (its PickObj refusal is unreachable under mask 0x10); REMOVED — the port's raw static
        // segment IS the game's ray. The port's ray-vs-player-sphere test on top of the static segment is
        // the port's own embellishment (the comment's old warrant — "flags 0x18 do not exclude the
        // player's class" — missed that mask 0x10 skips the whole dynamic walk, Bond included): the game's
        // spot lands on static geometry and `Vec_Dist3D(spot, player) <= 2.0` [M 0x1cb900:251] arms off
        // the ground spot; a PCSX2 render check decides whether the spot may rest on Bond (open live
        // check). The arming predicate below is untouched.
        if let Some(sp) = self.parts[head].links[2] {
            let (yaw, pitch) = (self.parts[head].yaw, self.parts[head].pitch);
            let dir = spherical(100.0, yaw, -pitch);
            let end = [pos[0] + dir[0], pos[1] + dir[1], pos[2] + dir[2]];
            // The player's pick sphere: centre `obj+0x80` (the `Player.sph` the port fills from
            // `Bond::sphere_centre`), radius `M_CONTROL+0x8C` = 1.0576 [L ee_start_05.bin].
            const PLAYER_SPH_R: f32 = 1.0576;
            let (mut t_hit, mut n_hit) = match d.world.segment_surface(pos, end, false) {
                Some((t, n, _)) => (Some(t), Some(n)),
                None => (None, None),
            };
            {
                // The ray is `pos + dir·t` with |dir| = 100 (spherical's length), t ∈ [0, 1] on the segment.
                let oc = [pos[0] - player.sph[0], pos[1] - player.sph[1], pos[2] - player.sph[2]];
                let a = dir[0] * dir[0] + dir[1] * dir[1] + dir[2] * dir[2];
                let b = oc[0] * dir[0] + oc[1] * dir[1] + oc[2] * dir[2];
                let c = oc[0] * oc[0] + oc[1] * oc[1] + oc[2] * oc[2] - PLAYER_SPH_R * PLAYER_SPH_R;
                let disc = b * b - a * c;
                if disc >= 0.0 {
                    let t = (-b - disc.sqrt()) / a;
                    if t > 0.0 && t_hit.is_none_or(|tw| t < tw) {
                        let p = [pos[0] + dir[0] * t, pos[1] + dir[1] * t, pos[2] + dir[2] * t];
                        let inv = 1.0 / PLAYER_SPH_R;
                        n_hit = Some([(p[0] - player.sph[0]) * inv, (p[1] - player.sph[1]) * inv, (p[2] - player.sph[2]) * inv]);
                        t_hit = Some(t);
                    }
                }
            }
            if let (Some(t), Some(n)) = (t_hit, n_hit) {
                let hit = [pos[0] + (end[0] - pos[0]) * t, pos[1] + (end[1] - pos[1]) * t, pos[2] + (end[2] - pos[2]) * t];
                let spot = &mut self.parts[sp];
                spot.pos = [hit[0] + n[0] * 0.01, hit[1] + n[1] * 0.01, hit[2] + n[2] * 0.01];
                spot.drawn = true;
                // The game's spot orientation [M 0x1cb900 hit branch, disasm 0x1cbe78–0x1cbea8]: the hit
                // normal (HITDATA +0x10/14/18) becomes the object's ANGLES — pitch(+0x50) =
                // π/2 − ATAN2_APPROX(n.y, |n.xz|), yaw(+0x54) = ATAN2_APPROX(n.x, n.z), roll(+0x58) = 0 —
                // and the framework rebuilds obj+0x90 with `View_RotTransMatrix` (Control_GetObjMatrix
                // 0x134210 → 0x1e5168; the spot's +0xFA bits 0x20/0x4 leave the RotTrans path live).
                // aivolume::matrix is that builder (verified against the live volume matrix). The old
                // reading put atan2(n.z, |n.xz|) in the pitch slot of the PLACEMENT-Euler object_matrix
                // with the yaw in the roll slot — on wall/slope/Bond-sphere normals the disc's plane
                // folded onto its edge.
                let pitch = crate::trig::HALF_PI - atan2_approx(n[1], (n[0] * n[0] + n[2] * n[2]).sqrt());
                let yaw = atan2_approx(n[0], n[2]);
                spot.matrix = crate::aivolume::matrix([pitch, yaw, 0.0], spot.pos);
            } else {
                self.parts[sp].drawn = false;
            }
            // Bond inside the circle arms the light; while tracking, the spot counts the ticks he is not [M].
            let sp_pos = self.parts[sp].pos;
            let dx = sp_pos[0] - player.pos[0];
            let dy = sp_pos[1] - player.pos[1];
            let dz = sp_pos[2] - player.pos[2];
            let dist = (dx * dx + dy * dy + dz * dz).sqrt();
            if self.parts[head].state == 0 && dist <= 2.0 {
                self.parts[head].state = 1;
                self.parts[sp].phase = 0.0;
            } else if self.parts[head].state == 1 {
                if dist <= 2.0 {
                    self.parts[sp].phase = 0.0;
                } else {
                    let lost = self.parts[sp].phase + 1.0;
                    self.parts[sp].phase = lost;
                    if lost >= LOST_LIMIT { self.parts[head].state = 0; }
                }
            }
        }
        // The beam [M]: aligned to the aim, up = Bond − head (pinned against the live beam matrix, L).
        if let Some(b) = self.parts[head].links[3] {
            let (yaw, pitch) = (self.parts[head].yaw, self.parts[head].pitch);
            let up = sub3(player.pos, pos);
            let mut m = align2dir(spherical(1.0, yaw, -pitch), up, up);
            m[3] = [pos[0], pos[1], pos[2], 0.0];
            let beam = &mut self.parts[b];
            beam.matrix = m;
            beam.drawn = true;
        }
        // The corona [M, disassembly 0x1cc03c]: a billboard 0.4 along the aim, visible when the head is drawn
        // and the aim points within ~37° of Bond (dot > 0.2), fading with the angle and growing when in sight.
        if let Some(c) = self.parts[head].links[4] {
            let (yaw, pitch) = (self.parts[head].yaw, self.parts[head].pitch);
            let off = spherical(0.4, yaw, -pitch);
            let corona_pos = [pos[0] + off[0], pos[1] + off[1], pos[2] + off[2]];
            let to = sub3(player.eye, pos);
            let len = to[0] * to[0] + to[1] * to[1] + to[2] * to[2];
            let dot = dot3(off, to) / len.sqrt();
            let head_drawn = self.parts[head].drawn;
            if !head_drawn || dot <= 0.2 {
                self.parts[c].drawn = false;
            } else {
                let a = (dot - 0.2) * 4.0;
                let scale = a * 4.0;
                let alpha = (a * 255.0) as u8;
                let mut scale = scale;
                if alpha != 0 && d.world.line_of_sight(player.eye, corona_pos, crate::world::LOS_PROBE_8) { scale *= 1.5; }
                let corona = &mut self.parts[c];
                corona.pos = corona_pos;
                corona.drawn = true;
                corona.alpha = alpha;
                corona.scale = scale;
                corona.matrix = object_matrix([0.0; 3], corona_pos);
            }
        }
        // The tail's contact walk (states 0/1/3) [M 0x1cb900:406-462].
        self.bullet_check(head, shots, &mut sounds);
        sounds
    }

    /// The head's contact walk (U-041 ported) [M `Searchlight_Update` 0x1cb900:406-462]. The game's
    /// `Collide_Update` distributes each flying bullet's pick onto the hit object's `obj+0xD0` list as a
    /// 0x60-byte node {next +4, damage f32 +8, param +0xC, hit data +0x10.., attacker obj +0x58} [M
    /// 0x1ea398:160-215]; the head's tail then walks its own list: every entry whose attacker's class byte
    /// (`obj+0xFF`) is 5 (a bullet) and whose damage float is > 0.0 breaks the light [M 0x1cb900:411-417].
    /// The break: state 2 [M :418]; the spot's psiLight node freed (`psiLight_Delete` + the pool return)
    /// [M :419-433]; the spot, beam and corona objects marked deleted (`obj+0xFE |= 1`) and their link
    /// slots cleared — the MOUNT (`links[0]`) untouched [M :438-453]; `Control_NextLOD` (the damaged
    /// model swap — visual, not ported) [M :454]; `Sound_Play3D(0xE6)` at the head [M :455-456]; the walk
    /// continues, so a second qualifying contact re-runs the block (state 2 again, another 0xE6) [M :457].
    fn bullet_check(&mut self, head: usize, shots: &[Shot], sounds: &mut Vec<(u32, V3)>) {
        // The broken head never reads contacts: the game's state-2 arm returns on every path before the
        // walk [M 0x1cb900:373-393].
        if self.parts[head].state == 2 { return; }
        if shots.is_empty() { return; }
        let pos = self.parts[head].pos;
        // The head's world sphere (obj+0x80/0x8C): `Control_BuildWorldSph` 0x133070 poses the glist's
        // local bounds by the object's matrix — centre = matrix · sph_centre, radius = sph_radius ·
        // |obj+0xE8| (obj+0xE8 = 1.0 from control_init_object 0x134630:17 [M]). The head must be PICKABLE:
        // the game's object walk requires a glist (obj+0xD8 with glist+8 ≠ 0, or obj+0xFA & 8) [M
        // 0x1eae80:88-90] — Searchlight_Create sets obj+0xD8 = the part's glist [M 0x1cb4f8:73-76].
        // Only the HEAD object's contacts matter: the mount is its own class-0x22 object whose update
        // returns before the walk (`if (*psVar2 != 1) return` [M 0x1cb900:89]), so its contacts — like the
        // head's from a non-bullet attacker — are never read (the old box test on the head AND the
        // mount is long replaced by this sphere pick).
        let (centre, radius) = {
            let h = &self.parts[head];
            let m = &h.matrix;
            let c = [m[0][0] * h.sph_centre[0] + m[0][1] * h.sph_centre[1] + m[0][2] * h.sph_centre[2] + m[3][0],
                     m[1][0] * h.sph_centre[0] + m[1][1] * h.sph_centre[1] + m[1][2] * h.sph_centre[2] + m[3][1],
                     m[2][0] * h.sph_centre[0] + m[2][1] * h.sph_centre[1] + m[2][2] * h.sph_centre[2] + m[3][2]];
            (c, h.sph_radius)
        };
        // The bullet's pick [M Collide_PickObj 0x1eae80:92-141, the type-0x201 arm reached with
        // hittest+0x94 = 0x201 from Bullet_update's flying tick]: the closest point on the flight segment
        // to the sphere centre (`vecutil_calculate_closest_point_on_line` 0x1df7e0 clamps t to [0,1]),
        // hit iff its squared distance < the radius squared. The slow-tick arm (hittest+0x94 = 0x101,
        // raw step < 0.125) tests the POINT hittest+0x60 = the pre-move position with the margin
        // hittest+0x6C (0.25 — the bullet obj's own 0.1 world radius floored at 0.25 [M 0x133070:23-31])
        // + the object's radius [M 0x1eae80:226-228, disasm 001eb028-001eb068]. A hit leaves one contact
        // per bullet per tick [M 0x1ea398's per-node redistribution; the node's +0x58 carries the
        // attacker], with the HITTEST's damage float at +8.
        let mut contacts: Vec<f32> = Vec::new();
        for s in shots {
            let hit = if s.segment {
                seg_sphere_hit(s.from, s.to, centre, radius)
            } else {
                let d = [s.from[0] - centre[0], s.from[1] - centre[1], s.from[2] - centre[2]];
                let d2 = d[0] * d[0] + d[1] * d[1] + d[2] * d[2];
                let m = 0.25 + radius;
                d2 < m * m
            };
            if hit { contacts.push(s.damage); }
        }
        // The walk [M 0x1cb900:406-462]: entries in list order, class-5 attackers only (the port's
        // contacts are all class-5 player bullets), damage > 0.0 breaks. Each qualifying entry re-runs
        // the break block — after the first break the links are already cleared, so later entries only
        // re-write state 2 and re-play 0xE6 [M the goto at :457 continues the loop].
        for &damage in &contacts {
            if damage <= 0.0 { continue; }
            let links = self.parts[head].links;
            self.parts[head].state = 2;
            // links[2]: free the light node, delete the spot object, clear the slot [M :419-439].
            if let Some(sp) = links[2] {
                self.parts[sp].drawn = false; // the game's obj+0xFE |= 1: the movement pass deletes it
                self.parts[sp].light = None;  // the psiLight node freed (psiLight_Delete) [M :430-433]
                self.parts[head].links[2] = None;
            }
            // links[3] (the beam) and links[4] (the corona): delete + clear [M :440-453].
            if let Some(b) = links[3] { self.parts[b].drawn = false; self.parts[head].links[3] = None; }
            if let Some(c) = links[4] { self.parts[c].drawn = false; self.parts[head].links[4] = None; }
            // The break swaps the head's model the same way the off-kill does [M Control_NextLOD at
            // 0x1cb900:454; the chain is the block-0x27 target, 07000006: OBJECT_SearchLightBroken [D]].
            self.parts[head].lod_next = true;
            // Control_NextLOD (the damaged model) is visual — see the module doc.
            sounds.push((0xe6, pos));
        }
    }
}

/// The game's type-0x201 pick primitive [M `Collide_PickObj` 0x1eae80:92-141]: the closest point on the
/// segment `a → b` to the sphere centre (`vecutil_calculate_closest_point_on_line` 0x1df7e0 clamps t to
/// [0,1] — an endpoint when the sphere is past either end), hit iff its squared distance is under the
/// radius squared (strict — `c.olt.s` on the squared compare).
fn seg_sphere_hit(a: V3, b: V3, centre: V3, radius: f32) -> bool {
    let d = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let dd = d[0] * d[0] + d[1] * d[1] + d[2] * d[2];
    let p = [centre[0] - a[0], centre[1] - a[1], centre[2] - a[2]];
    let t = if dd < 1e-20 { 0.0 } else { ((p[0] * d[0] + p[1] * d[1] + p[2] * d[2]) / dd).clamp(0.0, 1.0) };
    let q = [p[0] - d[0] * t, p[1] - d[1] * t, p[2] - d[2] * t];
    let q2 = q[0] * q[0] + q[1] * q[1] + q[2] * q[2];
    q2 < radius * radius
}

/// A segment against an axis-aligned box (both endpoints transformed into the box's frame by the caller).
fn seg_box(from: V3, to: V3, min: V3, max: V3) -> bool {
    let d = [to[0] - from[0], to[1] - from[1], to[2] - from[2]];
    let mut t0 = 0.0f32;
    let mut t1 = 1.0f32;
    for k in 0..3 {
        if d[k].abs() < 1e-12 {
            if from[k] < min[k] || from[k] > max[k] { return false; }
        } else {
            let mut a = (min[k] - from[k]) / d[k];
            let mut b = (max[k] - from[k]) / d[k];
            if a > b { std::mem::swap(&mut a, &mut b); }
            t0 = t0.max(a);
            t1 = t1.min(b);
            if t0 > t1 { return false; }
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn half_mid_matches_the_live_heads() {
        // [L] The seven heads of 07000005: (p4, p5) → stored (half, mid) bits in ee_start_05.bin.
        let golden: [(u32, u32, u32, u32); 7] = [
            (358, 345, 0x3de8_56a0, 0x40c4_508c),
            (0, 35, 0x3e9c_61ab, 0x3e9c_61ab),
            (250, 320, 0x3f1c_61ac, 0x409f_2c8d),
            (250, 320, 0x3f1c_61ac, 0x409f_2c8d),
            (0, 40, 0x3eb2_b8c3, 0x3eb2_b8c3),
            (0, 40, 0x3eb2_b8c3, 0x3eb2_b8c3),
            (355, 345, 0x3db2_b8c0, 0x40c3_7a15),
        ];
        for &(p4, p5, half, mid) in &golden {
            let (h, m) = half_mid(p4, p5);
            assert_eq!((h.to_bits(), m.to_bits()), (half, mid), "p4 {p4} p5 {p5}");
        }
        // Sister: the naive (max−min)·π/360 disagrees in the last bits (three of the seven heads).
        let naive = |p4: u32, p5: u32| ((p4.max(p5) - p4.min(p5)) as f32 * std::f32::consts::PI / 360.0).to_bits();
        assert!(golden.iter().any(|&(p4, p5, h, _)| naive(p4, p5) != h), "sister");
    }

    #[test]
    fn sweep_matches_the_live_heads() {
        // [L] Golden sweep vectors (world-objects.md): the stored phase is post-advance, so compute with −1.
        let golden: [(u32, u32, u32, u32, u32, u32); 5] = [
            (0x439f_0000, 0x3e9c_61ab, 0x3e9c_61ab, 0x3f49_0fdb, 0x3d41_00a8, 0x3f1f_5694),
            (0x43a0_8000, 0x3f1c_61ac, 0x409f_2c8d, 0x3f49_0fdb, 0xbfeb_4450, 0x3ef5_be04),
            (0x4383_0000, 0x3f1c_61ac, 0x409f_2c8d, 0xbf37_3094, 0xbfbcc_698, 0xbfa6_d999),
            (0x438a_8000, 0x3eb2_b8c3, 0x3eb2_b8c3, 0x3f25_514d, 0x3e43_2adf, 0x3eab_6499),
            (0x4160_0000, 0x3eb2_b8c3, 0x3eb2_b8c3, 0xbee8_5696, 0x3ed0_fcf8, 0xbde0_cab0),
        ];
        for &(phase, half, mid, base, pitch, yaw) in &golden {
            let (p, y) = sweep(f32::from_bits(phase) - 1.0, f32::from_bits(half), f32::from_bits(mid), f32::from_bits(base));
            assert_eq!((p.to_bits(), y.to_bits()), (pitch, yaw), "phase {phase:#x}");
        }
        // Sister: swapping the yaw's half-range for the pitch's disagrees (head 2: half ≠ mid).
        let (p, y) = sweep(f32::from_bits(0x43a0_8000) - 1.0, f32::from_bits(0x409f_2c8d), f32::from_bits(0x3f1c_61ac), f32::from_bits(0x3f49_0fdb));
        assert_ne!((p.to_bits(), y.to_bits()), (0xbfeb_4450, 0x3ef5_be04), "sister");
    }

    #[test]
    fn align2dir_matches_the_live_beam() {
        // [L] Head 016dfe70 (yaw/pitch) aiming at Bond at the level start: the beam 016e0130's matrix bits.
        // The cross ORDER is what this pins (the mirrored order is ~2.0 off on row 0); the components match
        // within 2 ulp — the game's `Vec_Spherical_2_Cartesian` (PS2Sinf + FPU ops) and the VU0 MAC's single
        // rounding are not modelled to the last bit for this visual-only matrix (see docs/systems/render.md's
        // stance on last-ulp visual differences).
        let (yaw, pitch) = (f32::from_bits(0x3f1f_5694), f32::from_bits(0x3d41_00a8));
        let head = [f32::from_bits(0x4278_ab6a), f32::from_bits(0x4209_7269), f32::from_bits(0x42d6_0361)];
        let player = [f32::from_bits(0x42d1_e873), f32::from_bits(0x41c7_b4d7), f32::from_bits(0x42d9_a68a)];
        let up = sub3(player, head);
        let mut m = align2dir(spherical(1.0, yaw, -pitch), up, up);
        m[3] = [head[0], head[1], head[2], 0.0];
        let golden: [[u32; 4]; 4] = [
            [0xbe5e_bc78, 0xbf78_9016, 0x3dcc_3e74, 0],
            [0x3f48_8620, 0xbe70_3cbc, 0xbf13_5fb4, 0],
            [0x3f15_1500, 0xbd40_ee5d, 0x3f4f_c322, 0],
            [0x4278_ab6a, 0x4209_7269, 0x42d6_0361, 0],
        ];
        for r in 0..4 {
            for c in 0..4 {
                let (got, want) = (m[r][c].to_bits(), golden[r][c]);
                let ulp = got.abs_diff(want);
                assert!(ulp <= 4, "row {r} col {c}: {got:#x} vs {want:#x} ({ulp} ulp)");
            }
        }
        // Sister: the mirrored cross order (z × up) is far off row 0.
        let z = spherical(1.0, yaw, -pitch);
        let r0 = cross3(z, up);
        assert!((r0[0] - f32::from_bits(0xbe5e_bc78)).abs() > 0.1, "sister");
    }

    #[test]
    fn seg_box_hits_and_misses() {
        let (min, max) = ([-1.0, -1.0, -1.0], [1.0, 1.0, 1.0]);
        assert!(seg_box([-5.0, 0.0, 0.0], [5.0, 0.0, 0.0], min, max));
        assert!(!seg_box([-5.0, 2.0, 0.0], [5.0, 2.0, 0.0], min, max));
        assert!(seg_box([0.5, 0.5, 0.5], [3.0, 3.0, 3.0], min, max), "starts inside");
        assert!(!seg_box([2.0, 0.0, 0.0], [3.0, 0.0, 0.0], min, max), "past it");
    }

    #[test]
    fn seg_sphere_hits_and_misses() {
        let (c, r) = ([0.0, 0.0, 0.0], 1.0);
        assert!(seg_sphere_hit([-5.0, 0.0, 0.0], [5.0, 0.0, 0.0], c, r), "through the centre");
        assert!(seg_sphere_hit([-5.0, 0.99, 0.0], [5.0, 0.99, 0.0], c, r), "grazes inside");
        assert!(!seg_sphere_hit([-5.0, 1.01, 0.0], [5.0, 1.01, 0.0], c, r), "grazes outside");
        assert!(seg_sphere_hit([-0.5, 0.0, 0.0], [3.0, 0.0, 0.0], c, r), "starts inside, exits through +x");
        assert!(!seg_sphere_hit([1.5, 0.0, 0.0], [3.0, 0.0, 0.0], c, r), "the clamped endpoint (t=0) at 1.5 is outside");
        // Sister: the unclamped infinite-line answer would differ — the segment ends before the sphere.
        assert!(!seg_sphere_hit([5.0, 0.5, 0.0], [6.0, 0.5, 0.0], c, r), "behind the segment");
    }

    /// The contact walk [M 0x1cb900:406-462]: a class-5 contact with damage > 0.0 on the head's world
    /// sphere breaks the light like the game — state 2, the spot/beam/corona deleted (links[2..4]
    /// cleared), the MOUNT untouched [M :438-453], the glass 0xE6 at the head [M :455-456]. (The
    /// pre-U-041 port tested the head's AND the mount's gfx boxes and hid every link including the
    /// mount — both differences this test pins against the sphere rule.)
    #[test]
    fn a_damaging_bullet_contact_breaks_the_head_like_the_game() {
        let mut sl = Searchlights::default();
        // Head at the origin, sphere radius 1 centred on it; a mount 1.5 below (its own box); spot/beam/corona linked.
        let part = |kind: u8, pos: V3| Part {
            placement: 0, kind, pos, base_yaw: 0.0, on: 1, off: 0, alarm: 55, half: 0.0, mid: 0.0,
            phase: 0.0, drawn: true, matrix: [[1.0, 0.0, 0.0, 0.0], [0.0, 1.0, 0.0, 0.0], [0.0, 0.0, 1.0, 0.0], [pos[0], pos[1], pos[2], 0.0]],
            state: 0, pitch: 0.0, yaw: 0.0, links: [None; 5], killed: false, light: None,
            alpha: 255, scale: 1.0, box_min: [-0.5; 3], box_max: [0.5; 3],
            sph_centre: [0.0; 3], sph_radius: 1.0, spawn_pos: pos, created: 0, lod_next: false,
        };
        sl.parts = vec![part(0, [0.0, -1.5, 0.0]), part(1, [0.0, 0.0, 0.0]), part(2, [0.0, -10000.0, 0.0]), part(3, [0.0, 0.0, 0.0]), part(4, [0.0, 0.4, 0.0])];
        sl.parts[1].links = [Some(0), None, Some(2), Some(3), Some(4)];
        sl.parts[2].light = Some(7);
        // The shot crosses the head's sphere but NOT its gfx box: the sphere's radius 1 covers the box
        // half-diagonal only up to ~0.866 per axis, so the aim 0.95 off-axis is inside the sphere, outside
        // the box — the game's pick (the sphere) contacts; the pre-U-041 box test would have missed it.
        let shot = Shot { from: [-3.0, 0.0, 0.95], to: [3.0, 0.0, 0.95], damage: 3.5, segment: true };
        let mut sounds = Vec::new();
        sl.bullet_check(1, &[shot], &mut sounds);
        assert_eq!(sl.parts[1].state, 2, "the head breaks (state 2) [M 0x1cb900:418]");
        assert_eq!(sounds, vec![(0xe6, [0.0, 0.0, 0.0])], "the glass sound at the head [M 0x1cb900:455-456]");
        assert!(!sl.parts[2].drawn && !sl.parts[3].drawn && !sl.parts[4].drawn, "spot/beam/corona deleted [M 0x1cb900:438-453]");
        assert_eq!(sl.parts[1].links[2..4], [None, None], "links[2..4] cleared");
        assert_eq!(sl.parts[2].light, None, "the psiLight node freed [M 0x1cb900:430-433]");
        // The mount survives: drawn, still linked [M 0x1cb900:438-453 never touches links[0]].
        assert!(sl.parts[0].drawn, "the mount is NOT deleted — the game keeps links[0]");
        assert_eq!(sl.parts[1].links[0], Some(0), "the mount link stays");
        assert!(sl.parts[1].lod_next, "the break runs Control_NextLOD on the head [M 0x1cb900:454]");
    }

    /// The off-channel kill [M Searchlight_Update 0x1cb900:92-138]: state 3, the spot/beam/corona
    /// objects deleted (obj+0xFE |= 1 — the movement pass removes them, 0x1336f8:88-91) with the link
    /// slots cleared, the spot's psiLight node freed, the on/off/alarm channel bytes zeroed, and
    /// `Control_NextLOD` on the head [M :138]. The MOUNT survives drawn and linked (the walk covers
    /// only psVar2+0x14/0x16/0x18) — the housing stays. A later ON-channel pulse must NOT re-light:
    /// the head's own on-byte was zeroed [M :399 reads psVar2+0x1a].
    #[test]
    fn the_off_channel_kill_matches_the_game_field_for_field() {
        let mut sl = Searchlights::default();
        let part = |kind: u8, pos: V3| Part {
            placement: 0, kind, pos, base_yaw: 0.0, on: 64, off: 57, alarm: 55, half: 0.0, mid: 0.0,
            phase: 0.0, drawn: kind < 2, matrix: [[1.0, 0.0, 0.0, 0.0], [0.0, 1.0, 0.0, 0.0], [0.0, 0.0, 1.0, 0.0], [pos[0], pos[1], pos[2], 0.0]],
            state: 0, pitch: 0.0, yaw: 0.0, links: [None; 5], killed: false, light: None,
            alpha: 255, scale: 1.0, box_min: [-0.5; 3], box_max: [0.5; 3],
            sph_centre: [0.0; 3], sph_radius: 1.0, spawn_pos: pos, created: 0, lod_next: false,
        };
        sl.parts = vec![part(0, [0.0, -0.1, 0.0]), part(1, [0.0, 0.0, 0.0]), part(2, [0.0, -10000.0, 0.0]), part(3, [0.0, 0.0, 0.0]), part(4, [0.0, 0.4, 0.0])];
        sl.parts[1].links = [Some(0), None, Some(2), Some(3), Some(4)];
        sl.parts[2].light = Some(7);
        sl.off_kill(1);
        let h = &sl.parts[1];
        assert_eq!(h.state, 3, "state 3 [M 0x1cb900:94]");
        assert!(h.killed, "one-shot");
        assert_eq!((h.on, h.off, h.alarm), (0, 0, 0), "data+0x34/35/36 zeroed [M :134-137]");
        assert_eq!(h.links[0], Some(0), "the mount link survives [M the walk never touches data+0x20]");
        assert_eq!(h.links[1..5], [None, None, None, None], "spot/beam/corona links cleared [M :114-133]");
        assert!(h.lod_next, "Control_NextLOD ran on the head [M :138]");
        assert!(h.drawn, "the head stays drawn — the housing is the swapped model");
        assert!(sl.parts[0].drawn, "the mount stays drawn [M the kill never marks it]");
        assert!(!sl.parts[2].drawn && !sl.parts[3].drawn && !sl.parts[4].drawn, "spot/beam/corona deleted [M :113-133]");
        assert_eq!(sl.parts[2].light, None, "the psiLight node freed [M :105-110]");
        assert_eq!(sl.parts[1].state, 3, "no re-light off a cleared on-byte [M :399]");
    }

    /// Sister of the break test: a shot through the MOUNT's volume but outside the head's sphere does NOT
    /// break the light — the mount is its own class-0x22 object whose update returns before the contact
    /// walk [M 0x1cb900:89], so nobody reads its contacts. (The pre-U-041 port walked the mount's box
    /// and broke here — the U-041 symptom this sister guards.)
    #[test]
    fn a_mount_only_contact_does_not_break_the_head() {
        let mut sl = Searchlights::default();
        let part = |kind: u8, pos: V3| Part {
            placement: 0, kind, pos, base_yaw: 0.0, on: 1, off: 0, alarm: 55, half: 0.0, mid: 0.0,
            phase: 0.0, drawn: true, matrix: [[1.0, 0.0, 0.0, 0.0], [0.0, 1.0, 0.0, 0.0], [0.0, 0.0, 1.0, 0.0], [pos[0], pos[1], pos[2], 0.0]],
            state: 0, pitch: 0.0, yaw: 0.0, links: [None; 5], killed: false, light: None,
            alpha: 255, scale: 1.0, box_min: [-0.5; 3], box_max: [0.5; 3],
            sph_centre: [0.0; 3], sph_radius: 1.0, spawn_pos: pos, created: 0, lod_next: false,
        };
        sl.parts = vec![part(0, [0.0, -3.0, 0.0]), part(1, [0.0, 0.0, 0.0])];
        sl.parts[1].links = [Some(0), None, None, None, None];
        // Through the mount's box (its centre), 3 units below the head's sphere centre.
        let shot = Shot { from: [-3.0, -3.0, 0.0], to: [3.0, -3.0, 0.0], damage: 3.5, segment: true };
        let mut sounds = Vec::new();
        sl.bullet_check(1, &[shot], &mut sounds);
        assert!(sounds.is_empty(), "no glass sound — the mount's contact is unread");
        assert_eq!(sl.parts[1].state, 0, "the head keeps sweeping");
        assert!(sl.parts[0].drawn && sl.parts[1].drawn, "nothing hidden");
    }

    /// The break gate is the CONTACT DAMAGE, not the geometry [M 0x1cb900:417 `if (0.0 < damage)`]: a
    /// class-5 contact with damage 0.0 (the throwables' forced value — `Bullet_update` 0x12e7b0's
    /// override switch, ids 0x2B/0x34-0x37/0x3A-0x40) hits the sphere and does not break.
    #[test]
    fn the_break_gate_is_the_contact_damage_float() {
        let mut sl = Searchlights::default();
        let part = |kind: u8| Part {
            placement: 0, kind, pos: [0.0; 3], base_yaw: 0.0, on: 1, off: 0, alarm: 55, half: 0.0, mid: 0.0,
            phase: 0.0, drawn: true, matrix: [[1.0, 0.0, 0.0, 0.0], [0.0, 1.0, 0.0, 0.0], [0.0, 0.0, 1.0, 0.0], [0.0, 0.0, 0.0, 0.0]],
            state: 0, pitch: 0.0, yaw: 0.0, links: [None; 5], killed: false, light: None,
            alpha: 255, scale: 1.0, box_min: [-0.5; 3], box_max: [0.5; 3],
            sph_centre: [0.0; 3], sph_radius: 1.0, spawn_pos: [0.0; 3], created: 0, lod_next: false,
        };
        sl.parts = vec![part(1)];
        let through = Shot { from: [-3.0, 0.0, 0.0], to: [3.0, 0.0, 0.0], damage: 0.0, segment: true };
        let mut sounds = Vec::new();
        sl.bullet_check(0, &[through], &mut sounds);
        assert!(sounds.is_empty() && sl.parts[0].state == 0, "damage 0.0 never breaks [M 0x1cb900:417]");
        let grazing = Shot { from: [-3.0, 0.0, 0.0], to: [3.0, 0.0, 0.0], damage: 0.25, segment: true };
        sl.bullet_check(0, &[grazing], &mut sounds);
        assert_eq!(sl.parts[0].state, 2, "any positive damage breaks");
        assert_eq!(sounds.len(), 1, "one contact, one 0xE6");
    }

    /// A broken head ignores further contacts: the game's state-2 arm returns on every path before the
    /// walk [M 0x1cb900:373-393] — no second 0xE6, no state churn.
    #[test]
    fn a_broken_head_ignores_further_contacts() {
        let mut sl = Searchlights::default();
        let part = |kind: u8| Part {
            placement: 0, kind, pos: [0.0; 3], base_yaw: 0.0, on: 1, off: 0, alarm: 55, half: 0.0, mid: 0.0,
            phase: 0.0, drawn: true, matrix: [[1.0, 0.0, 0.0, 0.0], [0.0, 1.0, 0.0, 0.0], [0.0, 0.0, 1.0, 0.0], [0.0, 0.0, 0.0, 0.0]],
            state: 2, pitch: 0.4, yaw: 0.0, links: [None; 5], killed: false, light: None,
            alpha: 255, scale: 1.0, box_min: [-0.5; 3], box_max: [0.5; 3],
            sph_centre: [0.0; 3], sph_radius: 1.0, spawn_pos: [0.0; 3], created: 0, lod_next: false,
        };
        sl.parts = vec![part(1)];
        let shot = Shot { from: [-3.0, 0.0, 0.0], to: [3.0, 0.0, 0.0], damage: 3.5, segment: true };
        let mut sounds = Vec::new();
        sl.bullet_check(0, &[shot], &mut sounds);
        assert!(sounds.is_empty(), "the broken head never re-plays 0xE6 [M 0x1cb900:373-393]");
        assert_eq!(sl.parts[0].state, 2, "stays broken");
    }

    /// The slow-tick arm (hittest+0x94 = 0x101) tests the POINT with the 0.25 + radius margin [M
    /// Bullet_update 0x12e7b0's raw-step < 0.125 arm; Collide_PickObj 0x1eae80:226-228]: a hovering slow
    /// round within the margin contacts even though its floored 1-unit segment points away and misses the
    /// segment test. The segment-mode sister with the same start misses (the pick mode is what differs).
    #[test]
    fn the_slow_point_pick_uses_the_point_with_the_margin() {
        let mut sl = Searchlights::default();
        let part = |kind: u8| Part {
            placement: 0, kind, pos: [0.0; 3], base_yaw: 0.0, on: 1, off: 0, alarm: 55, half: 0.0, mid: 0.0,
            phase: 0.0, drawn: true, matrix: [[1.0, 0.0, 0.0, 0.0], [0.0, 1.0, 0.0, 0.0], [0.0, 0.0, 1.0, 0.0], [0.0, 0.0, 0.0, 0.0]],
            state: 0, pitch: 0.0, yaw: 0.0, links: [None; 5], killed: false, light: None,
            alpha: 255, scale: 1.0, box_min: [-0.5; 3], box_max: [0.5; 3],
            sph_centre: [0.0; 3], sph_radius: 1.0, spawn_pos: [0.0; 3], created: 0, lod_next: false,
        };
        sl.parts = vec![part(1)];
        // 1.2 from the centre: inside the 0.25 + 1.0 point margin, outside the 1.0 sphere.
        let point = Shot { from: [1.2, 0.0, 0.0], to: [2.2, 0.0, 0.0], damage: 3.5, segment: false };
        let mut sounds = Vec::new();
        sl.bullet_check(0, &[point], &mut sounds);
        assert_eq!(sl.parts[0].state, 2, "the point pick reaches 1.2 < 1.25");
        // Sister: the same start in segment mode misses (the closest point on the segment is the start,
        // 1.2 > the radius).
        sl.parts[0].state = 0;
        let seg = Shot { from: [1.2, 0.0, 0.0], to: [2.2, 0.0, 0.0], damage: 3.5, segment: true };
        let mut sounds2 = Vec::new();
        sl.bullet_check(0, &[seg], &mut sounds2);
        assert!(sounds2.is_empty() && sl.parts[0].state == 0, "the segment pick misses (1.2 > 1.0)");
    }

    /// The spot disc's orientation [M 0x1cb900 hit branch + 0x134210/0x1e5168]: the disc mesh lies in
    /// its local XZ plane (bbox y = 0 [D 'Searchlight1']), so its world normal is the matrix's ROW 1 —
    /// it must equal the hit normal. The old placement-Euler reading folded it onto its edge on
    /// vertical normals.
    #[test]
    fn the_spot_disc_faces_the_hit_normal() {
        let nrm = |n: V3| {
            let pitch = crate::trig::HALF_PI - atan2_approx(n[1], (n[0] * n[0] + n[2] * n[2]).sqrt());
            let yaw = atan2_approx(n[0], n[2]);
            crate::aivolume::matrix([pitch, yaw, 0.0], [0.0; 3])
        };
        for n in [[0.0, 1.0, 0.0], [-1.0, 0.0, 0.0], [0.0, 0.0, -1.0], [0.6, 0.8, 0.0], [-0.7, 0.0, -0.7141428]] {
            let m = nrm(n);
            let row1 = [m[1][0], m[1][1], m[1][2]];
            let d = (row1[0] - n[0]).abs() + (row1[1] - n[1]).abs() + (row1[2] - n[2]).abs();
            assert!(d < 0.01, "disc normal {row1:?} != hit normal {n:?}");
        }
        // The flat ground pose is the identity rotation (pitch 0, yaw 0, roll 0).
        let m = nrm([0.0, 1.0, 0.0]);
        assert!((m[0][0] - 1.0).abs() < 1e-6 && (m[2][2] - 1.0).abs() < 1e-6 && m[0][2].abs() < 1e-6);
        // Sister: the old reading (atan2(n.z, |n.xz|) in the placement-Euler pitch slot, the yaw in the
        // roll slot) does NOT face the wall normal — row 1 misses (−1,0,0) by far.
        let n = [-1.0f32, 0.0, 0.0];
        let a = atan2_approx(n[2], (n[0] * n[0] + n[2] * n[2]).sqrt());
        let old = object_matrix([a, crate::trig::HALF_PI - a, atan2_approx(n[0], n[2])], [0.0; 3]);
        let d = (old[1][0] - n[0]).abs() + (old[1][1] - n[1]).abs() + (old[1][2] - n[2]).abs();
        assert!(d > 0.5, "sister: the old reading should miss the wall normal ({d})");
    }
}
