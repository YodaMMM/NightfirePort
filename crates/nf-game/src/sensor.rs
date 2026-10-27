//! Sensors (placement type 0x28, class 0x37): `Sensor_Create` 0x1af010, `Sensor_Update` 0x1af618
//! (dispatched through `control_funcs[0x37]`, docs/atlas/object.md), `Sensor_InCone` 0x1af4f0,
//! `Sensor_SetBeam` 0x1af398, `Sensor_Init` 0x1b0868 (the CameraList 0x319930 init — monitor
//! infra, unported), `Sensor_ApplyDamage` 0x1b08a8 (an empty stub: sensors ignore damage).
//!
//! A sensor is the ceiling/wall security camera: it sweeps its 0x19 key-list pose (the same
//! `Spline_Interp` anim the doors/ss use), and watches a CONE — `Intersect_ConeSphere` 0x1ee100
//! against the player, gated by range and line-of-sight. Detection is PLAYER-ONLY: `Sensor_Update`
//! loads `glb_players[0]` once (disasm `lw $s4, -0x7720(0x2e)` @0x1af644 = 0x2d88e0) and passes it
//! as the test object to both `Sensor_InCone` calls (0x1af97c sweep, 0x1afae0 track) — the only
//! two call sites in the ELF, and no `switch_channels` writer is driven by NPC proximity
//! (docs/systems/script-census.md's "trip-by-anything" plain-words summary is NOT what the binary
//! does). After 31 ticks (0x1e+) of seeing him it alerts
//! (state 1), tracks him with `Vec_Track` 0x1e20a0 at 0.05/tick, and after 2 s of continuous
//! sight writes **1 to its out switch channel** (param 3) — the alarm signal doors/scripts and
//! the drone AI (`DroneFunc_CheckAlarmRaised` 0x1453a8) listen for. Losing him for 61 ticks
//! (FRAME_RATE_INT) returns the head (state 2). The disable channel (param 4) set → state 3
//! (off/destroyed glist), cleared → re-arm.
//!
//! Not ported (all visual/audio, none of it sim-visible): the lens (param 0) and mount (param 7)
//! gfx proxy objects and their `hashtable_set_object_to_entity_gfx` meshes, the destroyed-glist
//! swap (`obj+0xD8`), the draw-flag bits (`obj+0xF0/0xFA`), the monitor-view hide latch
//! (`SENSOR+0x60`, written by `Monitor_Update` 0x1afcc8 — the monitor console type 0x30 is
//! unported), and every sound (the 0x4AA hum, the 0x4DD servo, the 0x4A9 beeps — `Sound_Play3D`
//! enters no RNG draw but nothing in the sim consumes them; ss.rs skips the same way).
// draw hand-off: the lens proxy (param 0, 0x2000360-class) — the SetBeam sight line stretched to
// its hit length (lens obj+0xE8) — is still renderer work; the camera body and the mount (param 7)
// draw from scene.rs's posed slots (`Sensor::matrix` / `mount_matrix`).

use crate::door::{interp, parse_keys, quat_matrix, Key, Mat4};
use crate::trig::sinf;
use crate::triggers::Channels;
use crate::world::{CollisionWorld, V3};
use nf_formats::placement::{view_rot_trans_matrix, Placement};

/// `SENSOR+0x14` — the aim quaternion the track/return states blend.
type Quat = [f32; 4];

/// `Sensor_Update`'s degrees→half-angle constant (`0.008726647` = π/360, literal in the decompile
/// at 0x1af038/0x1af05c) [M].
const HALF_DEG: f32 = 0.008_726_647;
/// π/2 (the `1.5707964` literal at 0x1af044) [M].
const HALF_PI: f32 = 1.570_796_4;
/// `REC_FRAME_RATE` (0x30d0dc) = 1/FRAME_RATE = 1/60 [M GS_SetRefreshRate 0x17d2b8:16].
const REC: f32 = 1.0 / 60.0;
/// `FRAME_RATE_INT` (0x30d0cc) = 60 [M PS2FrameRate 60 NTSC]: ticks state 1 may lose Bond before
/// returning (the decompile's `FRAME_RATE_INT < obj+0xF6` at 0x1af618:148).
const LOST_LIMIT: i16 = 60;
/// The sweep's alert threshold: `obj+0xF6` counts consecutive in-cone ticks and transitions at
/// `prev > 0x1e` [M 0x1af618:211-216, disasm `slti 0x1f` @0x1af9a4].
const ALERT_SEEN: i16 = 0x1e;
/// `Vec_Track`'s tracking gain (0x3d4ccccd at 0x1afaa8) [M].
const TRACK: f32 = 0.05;
/// State 2's return slerp factor (0x3dcccccd at 0x1afbf0) and its `Quat_IsEqual` epsilon
/// (0x3c23d70a at 0x1afc28) [M].
const RETURN_RATE: f32 = 0.1;
const RETURN_EPS: f32 = 0.01;
/// The detection-time thresholds (the beep acceleration's 2.0/3.0 at 0x1af618:125/128 and the
/// channel write's 2.0 at :154) [M].
const TRIP_TIME: f32 = 2.0;
/// The player's world-sphere radius (`obj+0x8C`): `Control_BuildWorldSph` 0x133070 builds the
/// player's +0x80 sphere from the AnimObject local sphere (glist +0x1C radius × |obj scale
/// +0xE8|) [M 0x133070; docs/fields/glist.md], and the player's is **1.0576** — the fresh-boot
/// EE read recorded in docs/fields/obj-tag-player.md's +0x80 row (sphere (104.826, 24.837,
/// 108.582) r 1.0576 ≈ the capsule) [L]. The old stand-in 0.4 was the port's generic body read.
const TARGET_RADIUS: f32 = 1.0576;

fn sub(a: V3, b: V3) -> V3 { [a[0] - b[0], a[1] - b[1], a[2] - b[2]] }
fn dot3(a: V3, b: V3) -> f32 { a[0] * b[0] + a[1] * b[1] + a[2] * b[2] }
fn cross(a: V3, b: V3) -> V3 { [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]] }
/// `Mat_GetDir` 0x1e3398: matrix bytes +0x20..+0x2C — row 2 of the row-vector MATRIX [M; the
/// port's Mat4 is the same layout, door.rs "translation in row 3"].
fn dir_of(m: &Mat4) -> V3 { [m[2][0], m[2][1], m[2][2]] }

/// `Quat_MatToQuat` 0x1e03f0: the standard Shepperd method, trace table (1, 2, 0)
/// [L 0x305a10 in ee_snapshot.bin]. Quat order (x, y, z, w) — `param_1[3] = w`.
fn mat_to_quat(m: &Mat4) -> Quat {
    let t = m[0][0] + m[1][1] + m[2][2];
    if t >= 0.0 {
        let s = (t + 1.0).sqrt();
        let k = 0.5 / s;
        [(m[1][2] - m[2][1]) * k, (m[2][0] - m[0][2]) * k, (m[0][1] - m[1][0]) * k, s * 0.5]
    } else {
        // Largest-diagonal branch: the game's index table picks (i, j, k) permutations of (1, 2, 0).
        let (i, j, k) = if m[0][0] > m[1][1] && m[0][0] > m[2][2] { (0usize, 1usize, 2usize) }
            else if m[1][1] > m[2][2] { (1, 2, 0) } else { (2, 0, 1) };
        let s = (m[i][i] - (m[j][j] + m[k][k]) + 1.0).sqrt();
        let mut q = [0.0f32; 4];
        q[i] = s * 0.5;
        let f = if s != 0.0 { 0.5 / s } else { 0.0 };
        q[3] = (m[j][k] - m[k][j]) * f;
        q[j] = (m[i][j] + m[j][i]) * f;
        q[k] = (m[i][k] + m[k][i]) * f;
        q
    }
}

/// `Quat_Slerp_Acc` 0x1e4188 / `Vec_Track`'s inner blend — the same slerp `door.rs` uses for the
/// anim keys: shortest path, `PS2Sinf` weights while `1−|dot| > 0.01`, else the linear blend.
fn slerp(a: Quat, b: Quat, t: f32) -> Quat {
    let mut d = a[0] * b[0] + a[1] * b[1] + a[2] * b[2] + a[3] * b[3];
    let neg = d < 0.0;
    if neg { d = -d; }
    let (w0, w1) = if 0.01 < 1.0 - d {
        let ang = d.acos();
        let s = sinf(ang);
        (sinf(ang - t * ang) / s, sinf(t * ang) / s)
    } else { (1.0 - t, t) };
    let w0 = if neg { -w0 } else { w0 };
    [w0 * a[0] + w1 * b[0], w0 * a[1] + w1 * b[1], w0 * a[2] + w1 * b[2], w0 * a[3] + w1 * b[3]]
}

/// `Quat_IsEqual` 0x1e3ae8: every component within `eps`.
fn quat_is_equal(a: Quat, b: Quat, eps: f32) -> bool {
    (a[0] - b[0]).abs() <= eps && (a[1] - b[1]).abs() <= eps && (a[2] - b[2]).abs() <= eps && (a[3] - b[3]).abs() <= eps
}

/// The tracked orientation for `dir`: `Mat_Align2Dir` 0x1df2a8 with the game's constants
/// up (0, 1, 0) [L 0x2d6210] and right fallback (1, 0, 0) [L 0x2d6220]: z = normalize(dir),
/// x = normalize(cross(up, z)) — or, when |up·z| > 0.999, x = normalize(cross(right, z)) [M
/// 0x1df2a8:53-63: the degenerate branch crosses the *fallback right vector* with z and falls
/// through the same normalize block, it does not use (1,0,0) as x] — then y = normalize(cross(z,
/// x)) [M 0x1df2a8:88-115, the same opmula/opmsub cross + normalize]. Row 2 (= `Mat_GetDir`'s
/// axis, what the cone test uses) is exactly `dir`; the roll of rows 0/1 is this basis, ported
/// verbatim (it feeds `Vec_Track` 0x1e20a0:82-84 → the tracked quat → obj+0x90).
fn look_quat(dir: V3) -> Quat {
    let z = dir;
    let x = if dot3(z, [0.0, 1.0, 0.0]).abs() > 0.999 { cross([1.0, 0.0, 0.0], z) } else { cross([0.0, 1.0, 0.0], z) };
    let xl = dot3(x, x).sqrt();
    let x = if xl > 0.0 { [x[0] / xl, x[1] / xl, x[2] / xl] } else { [1.0, 0.0, 0.0] };
    let y = cross(z, x);
    mat_to_quat(&[[x[0], x[1], x[2], 0.0], [y[0], y[1], y[2], 0.0], [z[0], z[1], z[2], 0.0], [0.0; 4]])
}

/// One sensor: the `SENSOR` instance fields (obj+0xE0) plus the obj_tag state the update walks.
#[derive(Clone, Debug)]
pub struct Sensor {
    /// Index into the main map's placements.
    pub placement: usize,
    /// The placement translation (`obj+0x30` / the cone apex at `obj+0xC0`) [M 0x1af010].
    pub pos: V3,
    /// `obj+0x90` — the current pose matrix (swept / tracked).
    pub matrix: Mat4,
    /// `SENSOR+0x14` — the aim quaternion (only written by track/return; NOT by the sweep, which
    /// poses `obj+0x90` directly [M 0x1af618:174]).
    pub quat: Quat,
    /// The 0x19 key list (`SENSOR+0x24` = `m_paths[record+0x44]`).
    pub keys: Vec<Key>,
    /// `SENSOR+0x30` = key count − 1 (0 when the header count is 0) [M 0x1af010:82-88].
    pub limit: u16,
    /// `SENSOR+0x2C` = `(count−1) · REC_FRAME_RATE · (FRAME_RATE / rate)` [M 0x1af010:89-96] —
    /// the sweep's whole duration is the rate word (`extra[7] >> 16`) in ticks.
    pub step: f32,
    /// `SENSOR+0x28` — the sweep playhead.
    pub t: f32,
    /// `SENSOR+0x38` = `PS2Sinf(angle·HALF_DEG + HALF_PI)` — the cone half-angle cosine [M 0x1af010:63-64].
    pub cos_half: f32,
    /// `SENSOR+0x3C` = `PS2Sinf(angle·HALF_DEG)` — the half-angle sine [M 0x1af010:65-66].
    pub sin_half: f32,
    /// `SENSOR+0x40` = (f32)(i32) param 2 — the cone range [M 0x1af010:67-73].
    pub range: f32,
    /// `SENSOR+0x58` = param 3 — the OUT switch channel, 1 after 2 s of sight [M 0x1af618:154-157].
    pub out: u16,
    /// `SENSOR+0x5A` = param 4 — the disable channel: set → state 3, cleared → re-arm [M 0x1af618:107-117, 259-263].
    pub disable: u16,
    /// `SENSOR+0x5E` = param 6 byte — the monitor group id (monitor infra unported; field map).
    pub monitor_id: u8,
    /// `SENSOR+0x5F` = param 5 byte — when 0 the sensor joins the CameraList [M 0x1af010:135-137].
    pub listed: bool,
    /// param 0 — the lens gfx hash (visual; also arms `Sensor_SetBeam`'s ray [M 0x1af398:67]).
    pub lens: u32,
    /// param 7 — the mount gfx hash (visual).
    pub mount: u32,
    /// param 8 u16 — the monitor-view channel (`Monitor_Update`'s write; unported).
    pub monitor_ch: u16,
    /// `obj+0xF4` — the state: 0 sweep, 1 tracking, 2 returning, 3 disabled.
    pub state: u16,
    /// `SENSOR+0x44` — the detection timer (the 2.0 s trip clock).
    pub detect: f32,
    /// `SENSOR+0x48` — the beep/servo timer (only gates unplayed sounds; kept for the field map).
    pub beep: f32,
    /// `obj+0xF6` — the consecutive seen/lost tick counter (both states use it).
    pub seen: i16,
    /// The last detection result (for tests; not a game field).
    pub last_hit: bool,
    /// `SENSOR+0x60` — set while a monitor views this sensor (the unhide) [M 0x1afcc8].
    pub unhidden: bool,
    /// The lens proxy's obj+0xE8 — the beam stretch [M]. `Sensor_Create` 0x1af010:103 seeds it
    /// with the range; `Sensor_SetBeam` 0x1af398 rewrites it every tick: range by default (:72),
    /// then the `Collide_RayIntersect` first-hit distance (HITDATA+0xC, :78) when the mask-8 ray
    /// hits. The game draws the lens glist with the object's own scale (the +0xFA bit-0x1
    /// OnlyScaleZ the create ORs in, 0x1af010:102 — the stretch rides the mesh's length axis).
    pub beam_len: f32,
}

impl Sensor {
    /// The mount proxy's matrix (param 7's glist): `Sensor_SetBeam` 0x1af398:50-54 re-aligns it every
    /// tick with `Mat_Align2Up(mount, (0,1,0), dir(obj+0x90))` [M] — `Mat_Align2Up` 0x1df4b0 writes
    /// right = norm(up x dir), up, forward = norm(right x up) and leaves the translation (the
    /// placement position the proxy was created at, `Sensor_Create` 0x1af010:109) — so the mount
    /// stays upright and turns with the camera's heading.
    pub fn mount_matrix(&self) -> Mat4 {
        let d = [self.matrix[2][0], self.matrix[2][1], self.matrix[2][2]];
        let norm = |v: [f32; 3]| { let l = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt(); if l == 0.0 { v } else { [v[0] / l, v[1] / l, v[2] / l] } };
        let r = norm([d[2], 0.0, -d[0]]); // (0,1,0) x d
        let f = norm([-r[2], 0.0, r[0]]); // r x (0,1,0)
        [[r[0], r[1], r[2], 0.0], [0.0, 1.0, 0.0, 0.0], [f[0], f[1], f[2], 0.0], [self.pos[0], self.pos[1], self.pos[2], 1.0]]
    }

    /// `Sensor_InCone` 0x1af4f0 against the player [M; detection is player-only — disasm
    /// 0x1af644/0x1af97c/0x1afae0 pass `glb_players[0]` and nothing else ever reaches here].
    /// Also runs this tick's `Sensor_SetBeam` (InCone calls it first, 0x1af4f0:54) — the lens
    /// obj+0xE8 refresh below.
    fn in_cone(&mut self, world: &CollisionWorld, player: &crate::drone::Player) -> bool {
        let target_sph = player.sph;
        // `Sensor_SetBeam` 0x1af398's shortcut [M 0x1af398:57-81]: with the lens gfx object
        // (SENSOR+0x4C = param 0 [M Sensor_Create 0x1af010:97-103]) the beam ray pos → pos +
        // dir·range runs `Collide_RayIntersect(start, end, cel, tested, beam_gfx, hits, 0, 8)`
        // and DETECTS when the first hit's object (hit+0x58) is the tested object (0x1af398:79).
        // The port's ray = the shared LOS (static + the dynamic-geo set — the geometry the game's
        // mask-8 walk tests) + the sphere test on the target at the world-sphere radius:
        // wall-first → no detect either way, clear-and-touching → detect either way. The residual
        // coarseness — the game ray-tests the player's collision model, the port the ≈-capsule
        // world sphere r 1.0576 [L obj-tag-player.md +0x80] — is the documented equivalent.
        // The distance variant also carries SetBeam's lens obj+0xE8 write [M 0x1af398:72-78]:
        // range when the ray is clear, the first hit's distance when it is not.
        if self.lens != 0 {
            let dir = dir_of(&self.matrix);
            let end = [self.pos[0] + dir[0] * self.range, self.pos[1] + dir[1] * self.range, self.pos[2] + dir[2] * self.range];
            let hit_t = world.line_of_sight_hit(self.pos, end, crate::world::LOS_PROBE_8);
            // SetBeam's lens obj+0xE8 [M 0x1af398:72-78]: range when clear, the hit distance when not.
            self.beam_len = hit_t.map_or(self.range, |t| t * self.range);
            if hit_t.is_none() && segment_hits_sphere(self.pos, end, target_sph, TARGET_RADIUS) {
                return true;
            }
        }
        // The range gate [M 0x1af4f0:60-62]: `sqdist ≤ (range + target radius)²`.
        let d = sub(target_sph, self.pos);
        let d2 = dot3(d, d);
        let rr = self.range + TARGET_RADIUS;
        if d2 > rr * rr { return false; }
        // `Intersect_ConeSphere` 0x1ee100 (cos = SENSOR+0x38, sin = SENSOR+0x3C, apex = the
        // sensor position, dir = `Mat_GetDir`, sphere = the target's world sphere) [M].
        let dir = dir_of(&self.matrix);
        let proj = dot3(d, dir);
        // The apex-inside auto-hit [M 0x1ee100:36-54]: the cone test only runs when r² < d²; with
        // the apex inside the target sphere the game returns 1 without testing the cone.
        let hit = TARGET_RADIUS * TARGET_RADIUS >= d2 || {
            if proj * proj >= d2 * self.cos_half * self.cos_half && proj > 0.0 {
                true
            } else {
                let t = self.cos_half * proj + self.sin_half * (d2 - proj * proj).abs().sqrt();
                t >= 0.0 && t * t - d2 + TARGET_RADIUS * TARGET_RADIUS >= 0.0
            }
        };
        if !hit { return false; }
        // The line-of-sight gate [M 0x1af4f0:84-87 `Collide_LineOfSight(sensor+0x30, target+0x30,
        // cel, sensor obj, target obj, 10)`] — a thin `Collide_RayIntersect(...) == 0` wrapper [M
        // 0x1edf50], the caller's mask landing in the shared HITTEST+0x98 path (0xA = node-pass
        // drop + material-0x10 drop, objects tested) [M 0x1eb320:149-152, 0x1ebaa8:301-314]. The
        // port's `line_of_sight` is the same shared walk drone vision uses (static + missile cage
        // + dynamic geo); the mask-bit material/node drops are the sight-mask work's documented
        // deltas (world.rs `segment_masked`).
        world.line_of_sight(self.pos, player.pos, crate::world::LOS_SENSOR)
    }

    /// One `Sensor_Update` 0x1af618 tick.
    pub fn update(&mut self, world: &CollisionWorld, player: &crate::drone::Player, ch: &mut Channels, frame: u32) {
        // The preamble [M 0x1af618:96-117]: while the disable channel (SENSOR+0x5A) is set the
        // sensor is pinned to state 3 (the 0x4AA hum stop is unplayed).
        if self.disable != 0 && ch.on(self.disable as u32) {
            self.state = 3;
        }
        match self.state {
            0 => {
                // State 0 clears the out channel every tick [M 0x1af618:169-171] and resets the
                // detection timer [:172].
                if self.out != 0 { ch.set(self.out as u32, 0, frame); }
                self.detect = 0.0;
                // The sweep [M 0x1af618:173-203]: pose at the playhead, advance
                // `t += step · FRAME_RATE_MUL` (1.0 at 60 Hz), wrap past the limit. The servo
                // sounds (0x4DD at the half and at the wrap) are unplayed.
                if self.limit != 0 {
                    if let Some(m) = interp(&self.keys, 0, self.limit, self.t, true) { self.matrix = m; }
                    self.t += self.step;
                    if self.t > self.limit as f32 { self.t = 0.0; }
                }
                // Detection [M 0x1af618:204-222]: hit → count; `prev > 0x1e` → alert (state 1).
                let hit = self.in_cone(world, player);
                self.last_hit = hit;
                if hit {
                    let prev = self.seen;
                    self.seen += 1;
                    if prev > ALERT_SEEN {
                        self.detect = 0.0;
                        self.beep = 0.0;
                        self.state = 1;
                    }
                } else {
                    self.seen = 0;
                }
            }
            1 => {
                // State 1 tracks Bond [M 0x1af618:137-138 `Vec_Track(0.05, players[0]+0x80,
                // obj+0x30, obj+0x90, SENSOR+0x14)`]: slerp the pose toward facing his sphere.
                let qa = mat_to_quat(&self.matrix);
                let d = sub(player.sph, self.pos);
                let len = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
                let dir = if len > 0.0 { [d[0] / len, d[1] / len, d[2] / len] } else { dir_of(&self.matrix) };
                let qb = look_quat(dir);
                self.quat = slerp(qa, qb, TRACK);
                self.matrix = quat_matrix(self.quat, self.pos);
                // Sight bookkeeping [M 0x1af618:139-150]: 61 lost ticks (FRAME_RATE_INT) → return.
                let hit = self.in_cone(world, player);
                self.last_hit = hit;
                if hit {
                    self.seen = 0;
                } else {
                    self.seen += 1;
                }
                if LOST_LIMIT < self.seen {
                    self.state = 2;
                } else {
                    // The trip [M 0x1af618:152-157]: `detect += REC_FRAME_RATE`; at ≥ 2.0 with an
                    // out channel: `switch_channels[out] = 1` + the time stamp — re-written and
                    // re-stamped every tick while the sight holds (a plain array write in the
                    // game; `Channels::set`'s nonzero stamp matches `switch_channels_time`).
                    self.detect += REC;
                    if self.detect >= TRIP_TIME && self.out != 0 {
                        ch.set(self.out as u32, 1, frame);
                    }
                }
            }
            2 => {
                // State 2 returns [M 0x1af618:225-243]: clear the channel, slerp the aim quat
                // toward the sweep pose at the current playhead, state 0 when equal.
                self.beep = 0.0;
                if self.out != 0 { ch.set(self.out as u32, 0, frame); }
                self.seen = 0;
                match interp(&self.keys, 0, self.limit, self.t, true).map(|m| mat_to_quat(&m)) {
                    Some(tq) => {
                        self.quat = slerp(self.quat, tq, RETURN_RATE);
                        self.matrix = quat_matrix(self.quat, self.pos);
                        // SetBeam runs in the return state too [M 0x1af618:238] — the beam stays live.
                        self.beam_len = self.beam_ray(world);
                        if quat_is_equal(self.quat, tq, RETURN_EPS) { self.state = 0; }
                    }
                    // No keys: state 2 runs `Spline_Interp` UNCONDITIONALLY [M 0x1af618:232-234];
                    // with a NULL/empty framelist it writes nothing [M 0x1e1858], so the game's
                    // target quat is uninitialised stack and its `Quat_IsEqual` exit reads garbage
                    // [M 0x1af618:231-242] — the game's own observable behaviour in this case does
                    // not exist. The port's deterministic straight-to-sweep is the documented
                    // equivalent; on disc, 0700000b's two sensors both carry key lists [D
                    // sensor_live: 164 has 5 keys] and no shipped sensor is known to lack one (a
                    // full-level key census is an open live check).
                    None => self.state = 0,
                }
            }
            _ => {
                // State 3 (disabled): the destroyed-glist swap and the lens hide are visual; the
                // re-arm [M 0x1af618:259-263] waits for the disable channel to clear.
                if self.disable != 0 && !ch.on(self.disable as u32) {
                    self.state = 0;
                }
            }
        }
    }

    /// The placement index (the order-chain sort key comes from it).
    pub fn pos(&self) -> V3 { self.pos }

    /// This tick's `Sensor_SetBeam` lens stretch [M 0x1af398:57-79]: the mask-8 ray pos →
    /// pos + dir·range; clear → obj+0xE8 = the range (:62,72), hit → the first hit's distance
    /// (HITDATA+0xC, :78). `Sensor_InCone` runs it every tick in the sweep/track states
    /// (0x1af4f0:54) and `Sensor_Update` runs it again in the return state (0x1af618:238) —
    /// the only state 3 exit is the disable (no SetBeam, the lens display mask cleared 0x1af618:248-250).
    pub fn beam_ray(&self, world: &CollisionWorld) -> f32 {
        let dir = dir_of(&self.matrix);
        let end = [self.pos[0] + dir[0] * self.range, self.pos[1] + dir[1] * self.range, self.pos[2] + dir[2] * self.range];
        world.line_of_sight_hit(self.pos, end, crate::world::LOS_PROBE_8).map_or(self.range, |t| t * self.range)
    }
}

/// The closest-approach segment/sphere test (the same shape as drone.rs's `segment_hits_sphere`).
fn segment_hits_sphere(a: V3, b: V3, c: V3, r: f32) -> bool {
    let ab = sub(b, a);
    let ac = sub(c, a);
    let len2 = dot3(ab, ab);
    let t = if len2 <= 0.0 { 0.0 } else { (dot3(ac, ab) / len2).clamp(0.0, 1.0) };
    let d = [ac[0] - ab[0] * t, ac[1] - ab[1] * t, ac[2] - ab[2] * t];
    dot3(d, d) <= r * r
}

/// Every sensor of the level, in placement order (`Sensor_Create` order — `parsemap_create_dynamic_objects`
/// 0x1d04f0 case 0x28, no RNG draw [M 0x1af010 has none]).
#[derive(Clone, Debug, Default)]
pub struct Sensors { pub list: Vec<Sensor> }

impl Sensors {
    /// `Sensor_Create` over the main map's 0x28 placements [M]. Params (level_tag → `Placement`):
    /// p0 lens hash, p1 cone angle (degrees), p2 range, p3 out channel, p4 disable channel,
    /// p5 skip-CameraList byte, p6 monitor id byte, p7 mount hash, p8 monitor channel
    /// [M 0x1af010 + docs/fields/level-tag.md's sensor row].
    pub fn create(placements: &[Placement], path_blocks: &[&[u8]]) -> Sensors {
        let mut list = Vec::new();
        for (i, p) in placements.iter().enumerate() {
            if p.flags & 0xff != 0x28 { continue; }
            // The spawn gate every creator shares (bit 0x8000 static cel, 0x2000 template).
            if p.flags & 0xa000 != 0 { continue; }
            let param = |k: u32| p.params.iter().find(|(j, _)| *j == k).map_or(0, |&(_, v)| v);
            let count = (p.extra[7] & 0xffff) as usize;
            let limit = if count == 0 { 0 } else { count - 1 };
            let index = (p.extra[8] & 0xffff) as usize;
            let keys = path_blocks.get(index).map(|payload| parse_keys(payload)).unwrap_or_default();
            let rate = (p.extra[7] >> 16) as f32;
            // `SENSOR+0x2C` in the game's op order [M 0x1af010:89-96]: (count−1) · (1/60) · (60/rate).
            let step = if rate > 0.0 { limit as f32 * REC * (60.0 / rate) } else { 0.0 };
            let angle = param(1) as f32;
            // Sensor_Create builds obj+0x90 with CCO(0) → View_RotTransMatrix
            // [M 0x1af010:54, 0x1344e0:26-29].
            let matrix = view_rot_trans_matrix(p.rotation, p.translation);
            list.push(Sensor {
                placement: i,
                pos: p.translation,
                matrix,
                quat: mat_to_quat(&matrix),
                keys,
                limit: limit as u16,
                step,
                t: 0.0,
                cos_half: sinf(angle * HALF_DEG + HALF_PI),
                sin_half: sinf(angle * HALF_DEG),
                range: param(2) as i32 as f32,
                out: param(3) as u16,
                disable: param(4) as u16,
                monitor_id: param(6) as u8,
                listed: param(5) as u8 == 0,
                lens: param(0),
                mount: param(7),
                monitor_ch: param(8) as u16,
                state: 0,
                detect: 0.0,
                beam_len: param(2) as i32 as f32,
                beep: 0.0,
                seen: 0,
                last_hit: false,
                unhidden: false,
            });
        }
        Sensors { list }
    }

    /// One sensor's update (the control-list walk calls per object, like `Doors::update`).
    pub fn update(&mut self, index: usize, world: &CollisionWorld, player: &crate::drone::Player, ch: &mut Channels, frame: u32) {
        if let Some(s) = self.list.get_mut(index) {
            s.update(world, player, ch, frame);
        }
    }

    /// The sensor of a placement index.
    pub fn of_placement(&self, placement: usize) -> Option<usize> {
        self.list.iter().position(|s| s.placement == placement)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::CollisionWorld;

    fn sensor_aiming(dir: [f32; 3]) -> Sensor {
        let mut m: Mat4 = [[0.0; 4]; 4];
        m[2] = [dir[0], dir[1], dir[2], 0.0];
        Sensor {
            placement: 0,
            pos: [0.0; 3],
            matrix: m,
            quat: [0.0, 0.0, 0.0, 1.0],
            keys: Vec::new(),
            limit: 0,
            step: 0.0,
            t: 0.0,
            cos_half: 0.991_445,
            sin_half: 0.130_526,
            range: 20.0,
            out: 0,
            disable: 0,
            monitor_id: 0,
            listed: false,
            lens: 0,
            mount: 0,
            monitor_ch: 0,
            state: 0,
            detect: 0.0,
            beep: 0.0,
            seen: 0,
            last_hit: false,
            unhidden: false,
            beam_len: 0.0,
        }
    }

    fn player_at(sph: [f32; 3]) -> crate::drone::Player {
        let mut p = crate::drone::Player::default();
        p.sph = sph;
        p.pos = sph;
        p.target = sph;
        p
    }

    /// Mat_Align2Dir's degenerate branch [M 0x1df2a8:53-63]: with dir straight up (|up·z| > 0.999)
    /// the game crosses the FALLBACK RIGHT vector with z and normalizes — x = (0,0,1), y =
    /// (1,0,0) — not the old port's x = (1,0,0), y = (0,0,−1). Fails on the pre-task tree.
    #[test]
    fn look_quat_degenerate_branch_matches_mat_align2dir() {
        let q = look_quat([0.0, 1.0, 0.0]);
        let m = quat_matrix(q, [0.0; 3]);
        let want = [[0.0, 0.0, 1.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        for (r, w) in m.iter().take(3).zip(want) {
            for k in 0..3 { assert!((r[k] - w[k]).abs() < 1e-4, "row {r:?} vs {w:?}"); }
        }
    }

    /// Sister: the non-degenerate branch keeps the up-referenced basis — dir (1,0,0): x =
    /// cross(up, z) = (0,0,−1) [M 0x1df2a8:60-63 normal branch].
    #[test]
    fn look_quat_normal_branch_unchanged() {
        let q = look_quat([1.0, 0.0, 0.0]);
        let m = quat_matrix(q, [0.0; 3]);
        let want = [[0.0, 0.0, -1.0], [0.0, 1.0, 0.0], [1.0, 0.0, 0.0]];
        for (r, w) in m.iter().take(3).zip(want) {
            for k in 0..3 { assert!((r[k] - w[k]).abs() < 1e-4, "row {r:?} vs {w:?}"); }
        }
    }

    /// The apex-inside auto-hit [M Intersect_ConeSphere 0x1ee100:36-54]: with the apex inside the
    /// target sphere (d² ≤ r²) the game returns 1 without testing the cone — the sensor pointing
    /// AWAY from a target hugging it still detects. Fails on the pre-task tree.
    /// The range gate's edge [M 0x1af4f0:60-62: `sqdist <= (range + obj+0x8C)^2`, the player's
    /// +0x8C = 1.0576 L]: range 5 on the view axis detects at 6.0 and refuses at 6.5.
    #[test]
    fn range_gate_edge_is_range_plus_the_player_radius() {
        let mut s = sensor_aiming([0.0, 0.0, 1.0]);
        s.range = 5.0;
        let world = CollisionWorld::empty();
        assert!(s.in_cone(&world, &player_at([0.0, 0.0, 6.0])), "6.0 <= 5 + 1.0576: inside the gate");
        assert!(!s.in_cone(&world, &player_at([0.0, 0.0, 6.5])), "sister: 6.5 > 6.0576 is refused");
    }

    #[test]
    fn in_cone_hits_when_the_apex_is_inside_the_target_sphere() {
        let mut s = sensor_aiming([-1.0, 0.0, 0.0]);
        let world = CollisionWorld::empty();
        // The sphere centre 0.2 off the apex, BEHIND the view axis (dir −x): outside the cone, but
        // inside the 1.0576 world sphere.
        let behind = player_at([0.2, 0.0, 0.0]);
        assert!(s.in_cone(&world, &behind), "apex inside the sphere detects [M 0x1ee100:36-54]");
    }

    /// Sister: the same behind-the-axis target pushed OUTSIDE the sphere (d² > r²) misses — the
    /// auto-hit does not leak past the game's r² < d² guard [M 0x1ee100:37].
    #[test]
    fn in_cone_misses_behind_the_axis_outside_the_sphere() {
        let mut s = sensor_aiming([-1.0, 0.0, 0.0]);
        let world = CollisionWorld::empty();
        let behind = player_at([1.5, 0.0, 0.0]);
        assert!(!s.in_cone(&world, &behind), "behind the axis, outside the sphere → no detect [M 0x1ee100:37]");
    }
}
