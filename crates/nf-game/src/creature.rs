//! The game's Creature objects — placement kind 0xE0, class byte `obj+0xff == 0x23`.
//!
//! Port of the ACTION.ELF creature family, one function per game address:
//! `Creature_Create` 0x134ab8, `Creature_CalcPoint` 0x134cf0, `Creature_CalcWaypoints`
//! 0x134e58, `Creature_Init` 0x135208, `Creature_Delete` 0x135230, `Creature_Update`
//! 0x135268, `Creature_Hurt` 0x1352d0, `Creature_Move` 0x135340 [M; work/decomp/ACTION.ELF].
//!
//! The dead-data fact this port carries: **placement kind 0xE0 ships on no level** [D —
//! `crates/nf-formats/tests/level_objects_census.rs::creature_kind_0xe0_never_ships` walks
//! every 07* archive in FILES.BIN through the port's placement parser and asserts zero
//! kind-0xE0 rows; re-verified on the slim ISO 2026-11-04]. The only creator of a class-0x23
//! object is `Creature_Create` [M 0x134ab8:63], whose only caller is the kind-0xE0 arm of
//! `parsemap_create_dynamic_objects` 0x1d04f0 (jal 0x1d0650) [M 0x1d04f0:351-357] — so the
//! pool this module fills stays empty on retail data and every real-input walk is untouched
//! by construction. The mechanics are ported so that fact changing (the census test is the
//! tripwire — its panic message already demands exactly this port) finds them in place.
//!
//! What the game's creatures are, from the code [M]: ambient wanderers. `Creature_Create`
//! builds a 0xf0-byte CREATURE instance at obj+0x110 (`Control_CreateObjEx(1.0, 0xf0, ...)`
//! [M 0x134ab8:55-56]; instance = obj+0x110 [M docs/fields/obj-tag.md:33]) holding six
//! waypoints (0x1c each at CREATURE+8: `V3 pos` + `quat` — the `Spline_Interp` key shape),
//! a roam radius, a rate and a mode. `Creature_Move` walks a 4-state machine: state 0
//! recomputes the waypoints (`Creature_CalcWaypoints`), state 1 interpolates the object
//! matrix along them (`Spline_Interp` 0x1e1858 via the port's `door::interp`) until the
//! spline time passes 5.0, states 3→4 set `obj+0xF0 |= 0x20` (the dead/passthrough bit —
//! the same bit every drone death path sets and `Bullet_CollisionHandler` 0x12ce80:124
//! skips on). Nothing in ACTION.ELF ever writes state 3 (exhaustive store sweep of
//! CREATURE+0xde [D — five `sh` sites, all 0/1/4]) and class 0x23 has no collide handler
//! (`control_funcs` 0x26f890 row 0x23 at 0x26FA34 = {Creature_Update, NULL,
//! Creature_Delete} [M+D]) — creatures cannot be hurt or killed; `Creature_Hurt` is the
//! reverse direction: when param 4 is set it deals `Player_Hurt(2.0f, …)` to every class-3
//! (player) object in the obj+0xD0 contact list [M 0x1352d0:11-27].
//!
//! Draw: the game has no creature draw function — drawing is class-agnostic
//! (`View_AddObjects` 0x1e62d8 queues any glist-backed object) and the creature's pose is
//! the matrix `Creature_Move`'s spline rewrites. The port's viewer mesh path for the
//! creature's placement glist is NOT built here: the draw stays a stub (this module ports
//! the sim side; the merge side tracks the missing mesh pose). No retail creature exists to
//! render (census above).
//!
//! Documented port deviations (both no-ops on retail — the pool is empty; both listed in
//! docs/reports/creature-ambient.md for the merge side):
//! 1. `Creature_CalcPoint`'s hit branch lifts the waypoint by `normal·radius` off the ground
//!    [M 0x134dfc-0x134e00] and re-aligns the caller's direction buffer with the surface
//!    (`Mat_Align2Dir` + `Mat_GetNorm` [M 0x134dd8-0x134dfc]). The port's ray API
//!    (`CollisionWorld::segment_surface` world.rs:2161) carries (fraction, point, material) —
//!    no surface normal — so the port lands the waypoint ON the hit and leaves the direction
//!    un-realigned. Bounded by `radius`; the game's own realign result is dead downstream in
//!    the hit branch itself (s2 is never read again [D 0x134dfc-0x134e0c]).
//! 2. The per-cel reads — the mode-1 admission gate `cel+0x3c == 0xC023` [M 0x134ab8:103],
//!    the update gate `cel == 0 || cel+0xa0 != 0` [M 0x135268:10-12] and the mode-1 waypoint
//!    Y-clamp against `cel+0x54`/`cel+0x90` [M 0x13509c-0x1350a4] — need the object's cel
//!    link (`build_LinkToRoom(obj, 0, glb_world)` [M 0x1343c8:38-40]), which the port's
//!    attach has no runtime form of. The gate/clamp values are carried as `Option`s filled
//!    by the attach with `None` (gate folded open, clamp off); the extents field
//!    (`max(|glist+0x38|, |glist+0x28|)` [M 0x134ab8:78-84]) is likewise unmapped
//!    (docs/fields/glist.md covers only +0x10/+0x1C/+0x40/+0x44) and passes 0.0.
//! 3. `Creature_Create`'s own `Creature_CalcWaypoints` call [M 0x134ab8:114-116] is deferred
//!    to the first `Creature_Move` tick (state 0's calc) — the attach runs before the
//!    `CollisionWorld` exists. The waypoint set is identical (waypoint 0 is the create
//!    position, unmoved by then) and the interpolation timeline is unchanged.

use crate::door::{self, Key};
use crate::trig::{ee_add, atan2f_libm, sinf, HALF_PI};
use crate::world::{CollisionWorld, V3};
use crate::bullet::PlayerHealth;

/// The creature's waypoint (CREATURE+8+i·0x1c): `V3 pos` + quaternion [M 0x134ab8:92-98] —
/// the same 28-byte shape `Spline_Interp` addresses (`door::Key`).
pub type Waypoint = Key;

/// A Creature object (the CREATURE instance at obj+0x110, 0xf0 bytes; the port folds the
/// obj header into the pool entry — fields carry the game offsets in their comments).
#[derive(Clone, Debug)]
pub struct Creature {
    /// obj+0x30 at create; also CREATURE+0xB0, the roam centre the clamp orbits [M 0x134ab8:89-92].
    pub home: V3,
    /// CREATURE+0xDC — placement param 0: 0 = random roam, 1 = list/spline, ≥2 roams like 0
    /// with the keyframe (non-spline) interpolator [M 0x134ab8:66; 0x134e58:88-96].
    pub mode: u16,
    /// CREATURE+0xDE — the Creature_Move state [M 0x135340].
    pub state: u16,
    /// CREATURE+0xC8 — placement param 1 as u32→f32 (both decompile arms are the unsigned
    /// conversion; the `bltz` arm is the halve-convert-double for the top bit) [M
    /// 0x134b54-0x134b84 disasm].
    pub radius: f32,
    /// CREATURE+0xD4 — placement param 2 as u32→f32 × 0.01 [M 0x134ab8:74].
    pub delay: f32,
    /// CREATURE+0xE0 — placement param 3: arms the player-chase waypoint tighten [M
    /// 0x134ab8:75; 0x134e58:100-104].
    pub hurt_flag: u8,
    /// CREATURE+0xE1 — placement param 4: gates Creature_Hurt [M 0x134ab8:76; 0x135268:16-18].
    pub hurt_player: u8,
    /// CREATURE+0xD8 — `max(|glist+0x38|, |glist+0x28|)` [M 0x134ab8:78-84]; the port's
    /// glist map has neither offset (deviation 2) — attach passes 0.0.
    pub extent: f32,
    /// CREATURE+0xC0 — the `Spline_Interp` mode argument: 0 for mode-0 placements, 2 for
    /// mode-1 (spline) [M 0x134ab8:99-110; 0x1353d8 passes it straight through].
    pub spline_mode: u32,
    /// CREATURE+0xCC — the spline time `Creature_Move` advances [M 0x1353c8-0x1353f0].
    pub spline_t: f32,
    /// CREATURE+0xD0 — the per-tick advance: `(Rand_FRand(delay·0.5) + delay·0.5) ·
    /// REC_FRAME_RATE` [M 0x134e58:44-46].
    pub rate: f32,
    /// CREATURE+0xB4 — the mode-1 LList membership at 0x316e90 [M 0x134ab8:106-110;
    /// Creature_Init 0x135208; Creature_Delete 0x135230].
    pub registered: bool,
    /// obj+0xF0 bit 0x20 — set by state 4, the bullet-passthrough bit [M 0x135418-0x135424].
    pub dead: bool,
    /// obj+0xFA bit 0x20 — "moved this tick", set per state-1 tick [M 0x1353ac-0x1353c4].
    pub moved: bool,
    /// obj+0xFE bit 0 — the control walk's delete flag (no retail writer; the port's
    /// delete path mirrors `control_delete_object` → `Creature_Delete`) [M 0x133178:16-17].
    pub delete_me: bool,
    /// The six waypoints [M 0x134e58].
    pub wps: [Waypoint; 6],
    /// obj+0x90 — the pose `Creature_Move`'s spline rewrites [M 0x1353d0].
    pub matrix: [[f32; 4]; 4],
    /// obj+0x30 — the object position (the matrix's translation row) [M docs/fields/obj-tag.md:20].
    pub pos: V3,
    /// The placement's index in the level (the object IS its placement [M — one object per
    /// placement, `parsemap_create_dynamic_objects` 0x1d04f0]).
    pub placement: usize,
    /// Deviation 2: the cel-dependent values, `None` = the attach had no cel data (the
    /// gates fold open / the clamp is off).
    pub cel_3c: Option<i16>,
}

/// The creature pool (the port's fold of `Creature_Init` 0x135208's `LList_Init(0x316e90,
/// 0xf0, 0)` + the DynamicObjList entries of class 0x23).
#[derive(Default)]
pub struct Creatures {
    /// The creatures, in placement order (the create pass's enumeration).
    pub list: Vec<Creature>,
    /// The mode-1 registry — `Creature_Delete` removes from it [M 0x135230:6-8].
    pub llist: Vec<usize>,
}

impl Creatures {
    pub fn new() -> Creatures {
        Creatures { list: Vec::new(), llist: Vec::new() }
    }

    /// `Creature_Create` 0x134ab8 from a kind-0xE0 placement. `cel_3c` is deviation 2.
    pub fn create(&mut self, placement: usize, pos: V3, rot: V3, param: &impl Fn(u32) -> u32, cel_3c: Option<i16>) -> Option<usize> {
        // [M 0x134ab8:55-56] Control_CreateObjEx(1.0, 0xf0, pos, rot, 0, glist, 0, 1, 0x44) —
        // the port's pool entry replaces the allocation; obj+0xff = 0x23 is the pool membership.
        let mode = param(0) as u16; // [M 0x134ab8:66 — *(u16*)(level_tag+0x2c)]
        // [M 0x134ab8:99-110] the mode branches: 0 → spline mode 0; 1 → the cel+0x3c gate,
        // spline mode 2 and the LList_Add; ≥2 → neither (spline mode 0 via control_init's zero).
        let mut spline_mode = 0u32;
        let mut registered = false;
        if mode == 1 {
            match cel_3c {
                // [M 0x134ab8:103] `*(short*)(cel+0x3c) != -0x3fdd` → control_delete_object, return 0.
                Some(v) if v != -0x3fddi16 => return None,
                _ => {}
            }
            spline_mode = 2;
            registered = true;
        }
        let radius = param(1) as f32; // [M 0x134b54-0x134b84 — the u32→f32 conversion]
        let delay = param(2) as f32 * 0.01; // [M 0x134ab8:74]
        // [M 0x134ab8:92-98] the six waypoint quats = Quat_EulerToQuat(placement rot), copied.
        let q = euler_to_quat(&rot);
        let wps = [0usize; 6].map(|_| Waypoint { pos: [0.0; 3], quat: q });
        let matrix = nf_formats::placement::object_matrix(rot, pos);
        let c = Creature {
            home: pos,
            mode,
            state: 1, // [M 0x134cbc — the create tail sets state 1 (after the deferred calc; deviation 3)]
            radius,
            delay,
            hurt_flag: param(3) as u8,   // [M 0x134ab8:75]
            hurt_player: param(4) as u8, // [M 0x134ab8:76]
            extent: 0.0,                 // deviation 2
            spline_mode,
            spline_t: 0.0,               // [M 0x134e58 — the calc zeroes +0xcc; deferred with it]
            rate: 0.0,
            registered,
            dead: false,
            moved: false,
            delete_me: false,
            wps,
            matrix,
            pos,
            placement,
            cel_3c,
        };
        let i = self.list.len();
        if registered {
            self.llist.push(i); // [M 0x134ab8:108 — LList_Add(0x316e90, creature)]
        }
        self.list.push(c);
        Some(i)
    }

    /// One creature's control tick — `Creature_Update` 0x135268 + `Creature_Move` 0x135340,
    /// and the control walk's delete on `delete_me` (`Creature_Delete` 0x135230).
    pub fn tick_one(&mut self, i: usize, world: &CollisionWorld, player_pos: V3, rng: &mut crate::drone::Rng) {
        // [M 0x135268:10-12] the cel gate (`cel == 0 || cel+0xa0 != 0`) — deviation 2 folds
        // it open; the port's creature is world-cel-linked and always active. The hurt arm
        // [M 0x135268:16-18 → Creature_Hurt 0x1352d0] needs the obj+0xD0 contact list the
        // game's Collide_Update fills (0x1ea398:189 `node[0x16] = moving obj`); the port
        // keeps no contact list for creatures (no retail creature, no filler) — `hurt` is
        // the port of the walk+apply half, driven by the contact set the caller supplies.
        self.list[i].move_tick(world, player_pos, rng);
        // [M 0x135230] Creature_Delete: mode-1 creatures leave the LList.
        if self.list[i].delete_me {
            if self.list[i].registered {
                if let Some(k) = self.llist.iter().position(|&x| x == i) {
                    self.llist.swap_remove(k);
                }
            }
            self.list[i].registered = false;
        }
    }

    /// `Creature_Hurt` 0x1352d0 — the walk+apply half. `contacts` stands in for the obj+0xD0
    /// list's mover class bytes (`true` = class 3, the player) [M 0x1352d0:11-27].
    pub fn hurt(
        &mut self,
        i: usize,
        contacts: &[bool],
        health: &mut PlayerHealth,
        rng: &mut crate::drone::Rng,
        difficulty: u32,
        level: u32,
    ) {
        // [M 0x1352d0:21-24] Player_Hurt__FP7obj_tagfP7_VECTOR(0x40000000, mover, creature_pos)
        // — 2.0f per player-class mover; the fusebox live wire is the in-tree precedent for
        // the Player_Hurt → `PlayerHealth::pain_generic` fold (drone.rs:18774-18785).
        for _ in contacts.iter().filter(|&&p| p) {
            health.pain_generic(rng, 2.0, difficulty, level, 0);
        }
    }
}

impl Creature {
    /// `Creature_Move` 0x135340 — the state machine (per creature).
    fn move_tick(&mut self, world: &CollisionWorld, player_pos: V3, rng: &mut crate::drone::Rng) {
        match self.state {
            // [M 0x13538c-0x1353a8] state 1: the spline beat.
            1 => {
                // [M 0x1353ac-0x1353c4] obj+0xFA |= 0x20.
                self.moved = true;
                // [M 0x1353d0] Spline_Interp(+0xcc, +8, 0, 5, obj+0x90, 0, +0xc0).
                if let Some(m) = door::interp(&self.wps, 0, 5, self.spline_t, self.spline_mode != 0) {
                    self.matrix = m;
                    self.pos = [m[3][0], m[3][1], m[3][2]];
                }
                // [M 0x1353c8-0x1353ec] +0xcc += +0xd0 · FRAME_RATE_MUL (1.0 at the port's
                // fixed 60 Hz [L ee_start_05.bin @0x30d0d8, bullet.rs:50]).
                self.spline_t += self.rate;
                if 5.0 < self.spline_t {
                    self.state = 0; // [M 0x13540c]
                }
            }
            // [M 0x13537c-0x135378] state 0: recalc, state 1. (The game also calc'd at
            // create — deviation 3 folds that into this first tick.)
            0 => {
                self.calc_waypoints(world, player_pos, rng);
                self.state = 1;
            }
            // [M 0x13540c-0x135418] state 3 → 4. Nothing in ACTION.ELF writes 3 [D sweep];
            // the port carries the transition for completeness.
            3 => {
                self.state = 4;
            }
            // [M 0x135418-0x135424] state 4: obj+0xF0 |= 0x20 — the dead/passthrough bit.
            4 => {
                self.dead = true;
            }
            _ => {}
        }
    }

    /// `Creature_CalcWaypoints` 0x134e58.
    fn calc_waypoints(&mut self, world: &CollisionWorld, player_pos: V3, rng: &mut crate::drone::Rng) {
        // [M 0x134e58:16-24] waypoint 0 = the current object position; its quat = wp_5's
        // (Quat_Copy(+0x14, +0xa0) — +0xa0 is wp_5's quat: 0x14 + 5·0x1c).
        self.wps[0].pos = self.pos;
        self.wps[0].quat = self.wps[5].quat;
        self.spline_t = 0.0; // [M 0x134e58:28]
        // [M 0x134e58:44-46] the rate draw.
        self.rate = (rng.frand(self.delay * 0.5) + self.delay * 0.5) * (1.0 / 60.0);
        // [M 0x134e58:30-32] Mat_GetDir(obj+0x90) — matrix row 2 [M 0x1e3398:6-10].
        let mut dir = [self.matrix[2][0], self.matrix[2][1], self.matrix[2][2]];
        let mut prev = self.wps[0].pos;
        for i in 1..6usize {
            // [M 0x134e58:100-128] the chase check — i ≥ 2 only, needs the hurt flag and a
            // player within (extent·4 + 1.1)²: the waypoint direction becomes the player and
            // the euler scale drops to 0.2.
            let mut scale = 1.0f32;
            if i >= 2 && self.hurt_flag != 0 {
                let dx = player_pos[0] - self.pos[0];
                let dy = player_pos[1] - self.pos[1];
                let dz = player_pos[2] - self.pos[2];
                let sq = dx * dx + dy * dy + dz * dz;
                let reach = self.extent * 4.0 + 1.1;
                if sq < reach * reach {
                    let l = sq.sqrt();
                    if l > 0.0 {
                        dir = [dx / l, dy / l, dz / l];
                    }
                    scale = 0.2;
                }
            }
            let (off, euler) = if self.mode == 1 {
                // [M 0x135014-0x135044] mode 1: off = radius; euler = {Rand_FRandHalf(0.2),
                // Rand_FRandHalf(π/2), 0}.
                (self.radius, [scale * rng.frand_half(0.2), scale * rng.frand_half(HALF_PI), 0.0])
            } else {
                // [M 0x135084-0x1350b0] modes 0 and ≥2: off = Rand_FRand(radius·0.5) +
                // radius·0.5; euler = {s·Rand_FRandHalf(π), s·Rand_FRandHalf(π), 0}.
                let half = self.radius * 0.5;
                (rng.frand(half) + half, [scale * rng.frand_half(HALF_PI), scale * rng.frand_half(HALF_PI), 0.0])
            };
            let mut wp = self.wps[i].pos;
            calc_point(&mut wp, prev, &mut dir, &euler, off, self.radius, world);
            // [M 0x1350a8-0x1350d8] the roam clamp: |wp − home|² > radius² → home + radius·dir.
            let dx = wp[0] - self.home[0];
            let dy = wp[1] - self.home[1];
            let dz = wp[2] - self.home[2];
            let sq = dx * dx + dy * dy + dz * dz;
            if self.radius * self.radius < sq {
                let l = sq.sqrt();
                let k = self.radius / l;
                wp = [self.home[0] + dx * k, self.home[1] + dy * k, self.home[2] + dz * k];
            }
            // [M 0x13509c-0x1350a4] mode 1's Y clamp against the cel bounds — deviation 2
            // (no cel data: the clamp is off).
            self.wps[i].pos = wp;
            prev = wp;
        }
        // [M 0x135130-0x13518c] mode 1's orientation pass: each waypoint's quat faces the
        // step from the previous waypoint (cartesian_to_spherical_acc + EulerToQuat).
        if self.spline_mode != 0 {
            for i in 1..6usize {
                let dx = self.wps[i].pos[0] - self.wps[i - 1].pos[0];
                let dy = self.wps[i].pos[1] - self.wps[i - 1].pos[1];
                let dz = self.wps[i].pos[2] - self.wps[i - 1].pos[2];
                // [M 0x1e3a70:8-12] out = {atan2(−y, √(x²+z²)), atan2(x, z), 0}.
                let pitch = atan2f_libm(-dy, (dx * dx + dz * dz).sqrt());
                let yaw = atan2f_libm(dx, dz);
                self.wps[i].quat = euler_to_quat(&[pitch, yaw, 0.0]);
            }
        }
    }
}

/// `Creature_CalcPoint` 0x134cf0 [M — the parameter map is the disasm's: a0 = the euler
/// `VECTOR*` (RotMatrix's input), a1 = the previous waypoint (the ray start), a2 = the out
/// waypoint, a3 = the direction `VECTOR*` (transformed in place by the euler matrix),
/// f12 = off, f13 = radius, t0 = the object]. The ray (mask 0x20d [M 0x134db4-0x134db8]) is
/// the port's unscoped static walk (`segment_hit`) — the game scopes it by the object's cel
/// [M 0x134d78 `lw a2,32(s0)`], which the port does not link (deviation 2).
fn calc_point(out: &mut V3, prev: V3, dir: &mut V3, euler: &V3, off: f32, radius: f32, world: &CollisionWorld) {
    // [M 0x134d38] RotMatrix(euler) — the X·Y·Z composition the port already carries as
    // `object_matrix`'s rotation (placement.rs:69; the translation row is zeroed here).
    let m = nf_formats::placement::object_matrix(*euler, [0.0; 3]);
    // [M 0x134d40-0x134d6c] the direction is transformed in place by the rotation.
    let d0 = *dir;
    *dir = [
        m[0][0] * d0[0] + m[1][0] * d0[1] + m[2][0] * d0[2],
        m[0][1] * d0[0] + m[1][1] * d0[1] + m[2][1] * d0[2],
        m[0][2] * d0[0] + m[1][2] * d0[1] + m[2][2] * d0[2],
    ];
    // [M 0x134d44 + 0x134d70] to = prev + dir·(off + radius).
    let reach = off + radius;
    let to = [prev[0] + dir[0] * reach, prev[1] + dir[1] * reach, prev[2] + dir[2] * reach];
    match world.segment_hit(prev, to) {
        // [M 0x134dbc hit path] out = hit + normal·radius — deviation 1 (no normal in the
        // port's ray result): the waypoint lands on the hit.
        Some((_t, point)) => {
            *out = point;
        }
        // [M 0x134e14-0x134e24 miss path] out = to − dir·radius (= prev + dir·off).
        None => {
            *out = [to[0] - dir[0] * radius, to[1] - dir[1] * radius, to[2] - dir[2] * radius];
        }
    }
}

/// `Quat_EulerToQuat` 0x1e3d78 [M — the two PS2Sinf3 calls and the four products, lanes
/// literal: cy·sx·cz − sy·cx·sz, cy·sx·sz + sy·cx·cz, cy·cx·sz − sy·sx·cz,
/// cy·cx·cz + sy·sx·cz where c/s·(x|y|z) = sinf(±/2 + π/2)/sinf(·/2)].
pub fn euler_to_quat(e: &V3) -> [f32; 4] {
    let cos_x = sinf(ee_add(e[0] * 0.5, HALF_PI));
    let sin_x = sinf(e[0] * 0.5);
    let cos_y = sinf(ee_add(e[1] * 0.5, HALF_PI));
    let sin_y = sinf(e[1] * 0.5);
    let cos_z = sinf(ee_add(e[2] * 0.5, HALF_PI));
    let sin_z = sinf(e[2] * 0.5);
    [
        cos_y * sin_x * cos_z - sin_y * cos_x * sin_z,
        cos_y * sin_x * sin_z + sin_y * cos_x * cos_z,
        cos_y * cos_x * sin_z - sin_y * sin_x * cos_z,
        cos_y * cos_x * cos_z + sin_y * sin_x * sin_z,
    ]
}

/// The level's kind-0xE0 creatures — `Drones::attach_level_id`'s case 0xE0 (the game's
/// `parsemap_create_dynamic_objects` 0x1d04f0:351-357 arm; the census is the dead-data
/// proof that this creates nothing on retail).
pub fn attach(level: &nf_formats::level::Level) -> Creatures {
    let mut out = Creatures::new();
    for m in 0..level.maps.len() {
        for (i, p) in level.maps[m].placements.iter().enumerate() {
            // [M 0x1d1b68:93 — flags & 0xa000 placements never reach the dispatcher].
            if p.flags & 0xa000 != 0 || p.flags & 0xff != 0xe0 {
                continue;
            }
            let param = |k: u32| p.params.iter().find(|(j, _)| *j == k).map(|&(_, v)| v).unwrap_or(0);
            // Deviation 2: the cel+0x3c gate value — the port has no cel link at attach.
            let _ = &param;
            out.create(i, p.translation, p.rotation, &param, None);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn placement(mode: u32, radius: u32, delay: u32, hurt: u32, gate: u32) -> impl Fn(u32) -> u32 {
        move |k: u32| match k {
            0 => mode,
            1 => radius,
            2 => delay,
            3 => hurt,
            4 => gate,
            _ => 0,
        }
    }

    fn hp() -> PlayerHealth {
        PlayerHealth { health: 100.0, armour: 0.0, flash: 0.0, dir_bits: 0, flashbang: 0.0, flash_total: 0.0, flash_alpha: 0.0, hit_push: 0.0 }
    }

    /// The create map: params → fields [M 0x134ab8:63-116].
    #[test]
    fn create_maps_the_placement_params() {
        let mut pool = Creatures::new();
        let pos = [1.0, 2.0, 3.0];
        let rot = [0.0, 1.5707964, 0.0];
        let i = pool.create(7, pos, rot, &placement(0, 400, 200, 1, 1), None).unwrap();
        let c = &pool.list[i];
        assert_eq!(c.home, pos);
        assert_eq!(c.mode, 0);
        assert_eq!(c.radius, 400.0);
        assert!((c.delay - 2.0).abs() < 1e-6); // param 2 × 0.01 [M 0x134ab8:74]
        assert_eq!(c.hurt_flag, 1);
        assert_eq!(c.hurt_player, 1);
        assert_eq!(c.spline_mode, 0);
        assert!(!c.registered);
        assert_eq!(c.state, 1);
        assert_eq!(c.placement, 7);
        // The six waypoint quats are the placement rotation's quat [M 0x134ab8:92-98].
        let q = euler_to_quat(&rot);
        for w in &c.wps {
            assert_eq!(w.quat, q);
        }
    }

    /// Mode 1 registers on the LList [M 0x134ab8:106-110] and the cel gate rejects
    /// non-0xC023 cels [M 0x134ab8:103].
    #[test]
    fn mode_one_registers_and_the_cel_gate_rejects() {
        let mut pool = Creatures::new();
        let i = pool.create(0, [0.0; 3], [0.0; 3], &placement(1, 100, 0, 0, 0), Some(-0x3fddi16)).unwrap();
        assert!(pool.list[i].registered);
        assert_eq!(pool.llist, vec![i]);
        assert_eq!(pool.list[i].spline_mode, 2);
        assert!(pool.create(1, [0.0; 3], [0.0; 3], &placement(1, 100, 0, 0, 0), Some(0x1234)).is_none());
    }

    /// The state machine [M 0x135340]: 0 → calc → 1 → the spline advances → t > 5 → 0;
    /// 3 → 4 → dead.
    #[test]
    fn move_state_machine_loops_and_dies() {
        let mut pool = Creatures::new();
        let mut rng = crate::drone::Rng::new(1);
        let world = CollisionWorld::empty();
        let i = pool.create(0, [0.0; 3], [0.0; 3], &placement(0, 100, 6000, 0, 0), None).unwrap();
        pool.list[i].state = 0;
        pool.tick_one(i, &world, [1e6; 3], &mut rng);
        assert_eq!(pool.list[i].state, 1); // the calc tick
        let mut t = 0.0f32;
        for _ in 0..10000 {
            pool.list[i].moved = false;
            pool.tick_one(i, &world, [1e6; 3], &mut rng);
            t += pool.list[i].rate;
            if pool.list[i].state == 0 {
                break;
            }
            assert!(pool.list[i].moved); // every state-1 tick sets obj+0xFA bit 0x20
        }
        assert!(t > 5.0);
        assert_eq!(pool.list[i].state, 0);
        pool.list[i].state = 3;
        pool.tick_one(i, &world, [1e6; 3], &mut rng);
        assert_eq!(pool.list[i].state, 4);
        pool.tick_one(i, &world, [1e6; 3], &mut rng);
        assert!(pool.list[i].dead); // obj+0xF0 bit 0x20
    }

    /// Creature_Hurt: 2.0f per player-class contact [M 0x1352d0:21-24], through the game's
    /// `Player_HandlePain` damage factor (level 0x01 group, Normal → ×0.7 [M bullet.rs]).
    #[test]
    fn hurt_drops_two_per_player_contact() {
        let mut pool = Creatures::new();
        let i = pool.create(0, [0.0; 3], [0.0; 3], &placement(0, 100, 0, 0, 1), None).unwrap();
        let mut health = hp();
        let mut rng = crate::drone::Rng::new(1);
        pool.hurt(i, &[true, false, true], &mut health, &mut rng, 2, 0x0700_0001);
        let expect = 100.0 - 2.0 * (2.0 * crate::bullet::damage_factor(0x0700_0001, 2));
        assert!((health.health - expect).abs() < 1e-4, "health {} != {}", health.health, expect);
        assert!((expect - 97.2).abs() < 1e-4, "the factor assumption moved: {}", expect);
    }

    /// The waypoints stay inside the roam radius [M 0x1350a8-0x1350d8] and waypoint 0 is
    /// the object position [M 0x134e58:16-24].
    #[test]
    fn waypoints_clamp_to_the_radius() {
        let mut pool = Creatures::new();
        let mut rng = crate::drone::Rng::new(7);
        let world = CollisionWorld::empty();
        let i = pool.create(0, [10.0, 20.0, 30.0], [0.0; 3], &placement(0, 50, 6000, 1, 0), None).unwrap();
        pool.list[i].state = 0;
        // The chase check runs against the player position (i ≥ 2, hurt flag set) — a far
        // player keeps the roam branch.
        pool.tick_one(i, &world, [1e6; 3], &mut rng);
        let c = &pool.list[i];
        assert_eq!(c.wps[0].pos, [10.0, 20.0, 30.0]);
        for w in &c.wps[1..] {
            let dx = w.pos[0] - 10.0;
            let dy = w.pos[1] - 20.0;
            let dz = w.pos[2] - 30.0;
            assert!(dx * dx + dy * dy + dz * dz <= 50.0 * 50.0 + 1e-3, "waypoint {:?} outside radius", w.pos);
        }
        assert!(c.rate > 0.0);
    }

    /// The delete path: mode-1 creatures leave the LList on delete_me [M 0x135230:6-8].
    #[test]
    fn delete_unregisters_mode_one() {
        let mut pool = Creatures::new();
        let mut rng = crate::drone::Rng::new(1);
        let world = CollisionWorld::empty();
        let i = pool.create(0, [0.0; 3], [0.0; 3], &placement(1, 100, 0, 0, 0), Some(-0x3fddi16)).unwrap();
        assert_eq!(pool.llist, vec![i]);
        pool.list[i].delete_me = true;
        pool.tick_one(i, &world, [0.0; 3], &mut rng);
        assert!(pool.llist.is_empty());
        assert!(!pool.list[i].registered);
    }
}
