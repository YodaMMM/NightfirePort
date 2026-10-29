//! `nf-view`: level viewer for 007: Nightfire (PS2), reading the player's own disc image.
//!
//! ```text
//! nf-view [<iso>] <level> [--all] [--seq K] [--frame F] [--fire] [--reinforcements] [--shot out.png [--cam x y z yaw pitch] [--pull T1,T2…]]
//! ```
//! `<level>` is a FILES.BIN name (`07000005`, `.bin` optional); without `<iso>` the image path comes from `NF_ISO`.
//! `--all` also draws dynamic entities at their placement transforms (not only 0x8000 cels). `--shot` renders one
//! 1280×720 frame headless to a PNG and exits (used to check output without a window).
//!
//! Controls: click to capture the mouse; mouse = look (raw deltas → yaw/pitch, no acceleration), WASD = move,
//! Space / Ctrl = up / down, Shift = fast, [ / ] = browse sequences ([ from the first returns to the game's start
//! animations), P = pause, F = firing demo for drones without AI, R = reload, F3 = show held-back drones (spawner reinforcements, drones waiting
//! for a switch), K = turn all switch channels on, F12 = screenshot.
//! Any other key skips a cut-scene while one is playing (the game's own skip: `SP_Update` state 2's
//! `Input_Action(-1, 0x19)` → `Script_FFwd` [M 0x1dbda8:129-141]; a death cut-scene can't be skipped).
//! The guard AI (nf-game) runs at 60 Hz with the camera as Bond; `--frame N` runs N ticks before a `--shot`, pressing
//! the trigger on the `--pull` ticks.
//! (nf-view-shot.png), E = use (doors), Esc = release mouse, Esc again = quit.

pub mod gpu;
#[cfg(feature = "gamepad")]
pub mod gamepad;
pub mod dev;
pub mod devwin;
pub mod hud;
pub mod credits;
pub mod menu;
pub mod menu_engine;
pub mod mp_lobby;
pub mod menu_music;
pub mod settings;
pub mod stdio_guard;
pub mod movie;
pub mod mpnet;
pub mod mpfx;
pub mod desync;
pub mod clipboard;
pub mod scene;
pub mod replay;
pub mod bugreport;
pub mod panel;
pub mod weapon_assets;
pub mod woman;
pub mod loading;
pub mod async_load;
/// The launcher (`NightfirePort` binary): pick the disc image once, then PLAY.
pub mod launcher;
/// The weapon matrix: every on-foot level × every weapon, measured headless.
pub mod matrix;

use anyhow::{bail, Context, Result};
use glam::{Mat4, Vec3};
use nf_formats::{disc, iso9660::IsoImage, level::Level};
use std::collections::HashSet;
use std::sync::Arc;

/// ── Load-phase timing (level-load-time probe) ───────────────────────────────────────────────────
/// `load_impl` and `Scene::build` mark their phases here; the probe test reads them back.
/// Collection is always on but costs one `Instant::now` + a Vec push per phase (a few dozen per
/// load — noise against multi-second loads); it never feeds the Sim (no wall-clock in game state).
pub(crate) fn phase_mark(name: &'static str, t: std::time::Instant) {
    LOAD_PHASES.with(|p| p.borrow_mut().push((name, t.elapsed().as_secs_f64())));
}
/// Clears the phase log for the calling thread (the probe calls it before each timed load).
pub fn phases_reset() { LOAD_PHASES.with(|p| p.borrow_mut().clear()); }
/// Takes the phase log the calling thread accumulated since the last reset/clear.
pub fn phases_take() -> Vec<(&'static str, f64)> {
    LOAD_PHASES.with(|p| std::mem::take(&mut *p.borrow_mut()))
}
thread_local! {
    static LOAD_PHASES: std::cell::RefCell<Vec<(&'static str, f64)>> = const { std::cell::RefCell::new(Vec::new()) };
}
use winit::application::ApplicationHandler;
use winit::event::{DeviceEvent, ElementState, MouseButton, MouseScrollDelta, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{CursorGrabMode, Window, WindowId};

/// Radians per mouse count. Mouse deltas go straight into yaw/pitch (docs/systems/input.md §8).
const MOUSE_SENSITIVITY: f32 = 0.0025;

/// The game's vertical field of view in play (60°, docs/systems/player.md).
const GAME_FOV: f32 = std::f32::consts::FRAC_PI_3;

/// The viewer at one tick (pos = the eye, yaw/pitch the look angles, fov half-angle radians). Pub for
/// the headless interpolation tests (fix-interp-ghosting) to build tick cameras the way the loop does.
#[derive(Clone, Copy)]
pub struct Camera { pub pos: Vec3, pub yaw: f32, pub pitch: f32, pub fov: f32 }

/// Render interpolation (G-043): logic runs at a fixed 60 Hz; frames between two ticks draw `t` of the way from
/// the previous tick's camera and palette to the latest one (one tick behind, the standard fixed-step scheme).
/// Jumps larger than `SNAP` (cut-scene cuts, teleports, hide/show) snap instead of blending.
const SNAP: f32 = 4.0;
fn lerp_cam(a: &Camera, b: &Camera, t: f32) -> Camera {
    if a.pos.distance(b.pos) > SNAP || (a.fov - b.fov).abs() > 0.2 { return *b; }
    Camera { pos: a.pos.lerp(b.pos, t), yaw: lerp_yaw(a.yaw, b.yaw, t), pitch: a.pitch + (b.pitch - a.pitch) * t, fov: a.fov + (b.fov - a.fov) * t }
}

/// The yaw half of `lerp_cam`: the short way round the ±π wrap (07000002's corridor turns must not
/// sweep the long way). Extracted with the tick scheduler so the fps-determinism test pins it.
pub fn lerp_yaw(a: f32, b: f32, t: f32) -> f32 {
    let mut dy = b - a;
    while dy > std::f32::consts::PI { dy -= std::f32::consts::TAU; }
    while dy < -std::f32::consts::PI { dy += std::f32::consts::TAU; }
    a + dy * t
}

/// The render loop's tick scheduler (`RedrawRequested`, the `anim_time += dt * 60` block), extracted
/// verbatim so the headless fps-determinism test drives the same arithmetic the winit loop runs:
/// a render frame advances the 60 Hz clock by `dt * 60` and runs the whole ticks that appeared,
/// capped at `max_ticks` (fix-perf-level's drop cap — capped ticks are dropped, not deferred).
/// Logic stays fixed-60 Hz at any frame rate: faster frames just run 0 ticks more often.
///
/// This is the game's frame gate under another name: `main` 0x109080 → `CreateDB` 0x10b000:52 →
/// `sceGsSyncV` 0x225930 → `VSync` 0x23c058 (mask interrupts, clear INTC stat bit 4, busy-wait
/// the next vsync) / `VSync2` 0x23c0e8 (the same plus the kernel flag via `SetVSyncFlag`
/// 0x23bf90, syscall 0x73) — one sim tick per vertical blank [M 0023c058/0023c0e8 decomp +
/// caller scan]. The host render loop's `dt * 60` accumulation with whole-tick runs is the
/// port's vsync wait (determinism rule: the tick count, not the wall clock, drives Sim).
pub fn ticks_for_frame(anim_time: &mut f32, dt: f32, max_ticks: u32) -> u32 {
    let before = *anim_time as u32;
    *anim_time += dt * 60.0;
    (*anim_time as u32).saturating_sub(before).min(max_ticks)
}
/// fix-interp-camera-bracket: one presented frame's interpolation bracket. The camera bracket is
/// advanced FIRST (on the frames that rebuild the palette), then the draw camera and the blend
/// fraction are taken from that same bracket — so the camera and the palette always interpolate
/// between the same two ticks. The loop used to do it the other way round: `draw_cam` was computed
/// from the PREVIOUS bracket while `pal_prev`/`pal_cur` were blended from the new one, so every
/// frame that ran a tick drew the camera one whole tick behind the world. Measured on 0700004a's
/// lift (drawn eye y, one tick = -0.0303 u): -22.72629, -22.76416, -22.77174, -22.77932,
/// -22.75659 - i.e. 1.25, 0.25, 0.25 then a 0.75-tick BACKWARD jump, 21 px at 1 m, every fourth
/// frame. Pub for the headless interpolation tests (fix-interp-camera-bracket).
pub fn draw_bracket(cam_prev: &mut Option<Camera>, cam_cur: &mut Option<Camera>, cam_now: Camera,
                    advanced: bool, fract: f32, interp: bool) -> (Camera, f32) {
    if advanced || cam_cur.is_none() {
        *cam_prev = *cam_cur;
        *cam_cur = Some(cam_now);
    }
    let alpha = if interp { fract } else { 1.0 };
    let draw = match (*cam_prev, *cam_cur) {
        (Some(a), Some(b)) if interp => lerp_cam(&a, &b, alpha),
        _ => cam_now,
    };
    (draw, alpha)
}
/// The palette half of the G-043 interpolation: the draw loop blends the previous tick's skin
/// palette into the latest one while `lerp_cam` blends the camera by angle. fix-interp-ghosting:
/// world matrices blend as ROTATION (slerp of the rotation part [M `Quat_Slerp_Acc` 0x1e4188] — a
/// component-wise lerp of a rotation is not a rotation, it shears and shrinks mid-way, the
/// arm/gun smear) + component lerp of the translation. Matrices whose 3×3 is not a
/// rotation (the collapse/FAR hides, the camera-facing billboards) keep the component lerp, where
/// slerp has no defined meaning. Pub for the headless interpolation tests (fix-interp-ghosting) to
/// drive the exact path the `RedrawRequested` handler runs.
pub fn lerp_palette(a: &[nf_formats::anim::Mat4], b: &[nf_formats::anim::Mat4], t: f32) -> Vec<nf_formats::anim::Mat4> {
    if a.len() != b.len() { return b.to_vec(); }
    a.iter().zip(b).map(|(x, y)| lerp_entry(x, y, t)).collect()
}

/// One palette entry of `lerp_palette`.
fn lerp_entry(x: &nf_formats::anim::Mat4, y: &nf_formats::anim::Mat4, t: f32) -> nf_formats::anim::Mat4 {
    // perf-240: an entry that did not change between the two ticks blends to itself — skip the
    // quaternion round trip (the bulk of the palette: props, doors at rest, collapsed slots).
    if x == y { return *y; }
    let d = (0..3).map(|k| (x[3][k] - y[3][k]).powi(2)).sum::<f32>().sqrt();
    if d > SNAP { return *y; }
    blend_pose(x, y, t)
}

/// perf-240: the per-frame blend touches only the entries that differ between the two ticks.
/// `out` holds the newer tick's palette (set once per tick, `moving` = the differing indices);
/// each presented frame rewrites just those (and `extra`, the view-model entries the camera
/// re-attach overwrote last frame). Equal to `lerp_palette(prev, cur, t)` entry for entry.
pub fn lerp_palette_moving(out: &mut [nf_formats::anim::Mat4], prev: &[nf_formats::anim::Mat4], cur: &[nf_formats::anim::Mat4], moving: &[u32], extra: &[usize], t: f32) {
    for &i in moving { let i = i as usize; out[i] = lerp_entry(&prev[i], &cur[i], t); }
    for &i in extra { if i < out.len() { out[i] = lerp_entry(&prev[i], &cur[i], t); } }
}

/// The rotation blend of one palette entry: slerp of the rotation part ([M `Quat_Slerp_Acc`
/// 0x1e4188], the game's own skin-pose blend) + lerp of the translation; entries whose 3×3 is not
/// orthonormal (collapse/FAR hides, scaled billboards) fall back to the component lerp.
fn blend_pose(x: &nf_formats::anim::Mat4, y: &nf_formats::anim::Mat4, t: f32) -> nf_formats::anim::Mat4 {
    match (rot_quat(x), rot_quat(y)) {
        (Some(qx), Some(qy)) => {
            let qt = nf_game::anim::slerp(qx, qy, t);
            let mut m = quat_rows(qt);
            for k in 0..4 { m[3][k] = x[3][k] + (y[3][k] - x[3][k]) * t; }
            m
        }
        _ => {
            let mut m = *y;
            for r in 0..4 { for c in 0..4 { m[r][c] = x[r][c] + (y[r][c] - x[r][c]) * t; } }
            m
        }
    }
}

/// The quaternion of a palette entry's 3×3 when it is a rotation (rows orthonormal within 1e-3 —
/// collapse/FAR hides and the scaled billboard coronas fail the gate): Shepperd's method on the
/// port's row-vector matrices (stored rows = the images of the local axes, i.e. the transposed
/// column-convention form the textbook formulas assume).
fn rot_quat(m: &nf_formats::anim::Mat4) -> Option<[f32; 4]> {
    for i in 0..3 {
        let n = (m[i][0] * m[i][0] + m[i][1] * m[i][1] + m[i][2] * m[i][2]).sqrt();
        if (n - 1.0).abs() > 1e-3 { return None; }
    }
    for i in 0..3 { for j in (i + 1)..3 {
        let d = m[i][0] * m[j][0] + m[i][1] * m[j][1] + m[i][2] * m[j][2];
        if d.abs() > 1e-3 { return None; }
    } }
    // Proper rotations only: a reflection (det −1) has no quaternion — Shepperd on one would
    // silently return an unrelated rotation. Scaled billboards with scale 1 can be improper.
    let det = m[0][0] * (m[1][1] * m[2][2] - m[1][2] * m[2][1])
            + m[0][1] * (m[1][2] * m[2][0] - m[1][0] * m[2][2])
            + m[0][2] * (m[1][0] * m[2][1] - m[1][1] * m[2][0]);
    if det < 0.5 { return None; }
    let tr = m[0][0] + m[1][1] + m[2][2];
    let (x, y, z, w);
    if tr > 0.0 {
        let s = (tr + 1.0).sqrt() * 2.0;
        w = 0.25 * s;
        x = (m[1][2] - m[2][1]) / s; y = (m[2][0] - m[0][2]) / s; z = (m[0][1] - m[1][0]) / s;
    } else if m[0][0] > m[1][1] && m[0][0] > m[2][2] {
        let s = (1.0 + m[0][0] - m[1][1] - m[2][2]).sqrt() * 2.0;
        w = (m[1][2] - m[2][1]) / s; x = 0.25 * s;
        y = (m[0][1] + m[1][0]) / s; z = (m[0][2] + m[2][0]) / s;
    } else if m[1][1] > m[2][2] {
        let s = (1.0 + m[1][1] - m[0][0] - m[2][2]).sqrt() * 2.0;
        w = (m[2][0] - m[0][2]) / s; x = (m[0][1] + m[1][0]) / s; y = 0.25 * s;
        z = (m[1][2] + m[2][1]) / s;
    } else {
        let s = (1.0 + m[2][2] - m[0][0] - m[1][1]).sqrt() * 2.0;
        w = (m[0][1] - m[1][0]) / s; x = (m[0][2] + m[2][0]) / s; y = (m[1][2] + m[2][1]) / s;
        z = 0.25 * s;
    }
    Some([x, y, z, w])
}

/// The palette-entry matrix of a quaternion: stored rows = the images of the local axes (the
/// row-vector convention's transposed form; round-trips `rot_quat` exactly).
fn quat_rows(q: [f32; 4]) -> nf_formats::anim::Mat4 {
    let (x, y, z, w) = (q[0], q[1], q[2], q[3]);
    [
        [1.0 - 2.0 * (y * y + z * z), 2.0 * (x * y + z * w), 2.0 * (x * z - y * w), 0.0],
        [2.0 * (x * y - z * w), 1.0 - 2.0 * (x * x + z * z), 2.0 * (y * z + x * w), 0.0],
        [2.0 * (x * z + y * w), 2.0 * (y * z - x * w), 1.0 - 2.0 * (x * x + y * y), 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ]
}

/// The tick camera as a view → world matrix (C⁻¹ = R·T(eye)): rows (left, up, fwd) + the eye —
/// the exact basis the first-person gun is posed from (`Player_PositionGun` 0x1a3b40 builds the
/// gun's axes as gl = up×gf = the viewer LEFT, gu = gf×gl = up, gf = the turned forward — det +1,
/// a proper rotation, and at yaw 0 it is the identity). So the gun's world matrix decomposes as
/// local pose · C⁻¹ and the camera factor cancels in camera space. (cam_axes' `right = f×Y` rows
/// would be det −1 — a reflection — which no quaternion represents.)
fn cam_view_to_world(cam: &Camera) -> nf_formats::anim::Mat4 {
    let f = cam.dir();
    let l = -cam.right(); // the viewer's left = −(f×Y) [M 0x1a3b40's gl], from the yaw (no ±90° degeneracy)
    let u = f.cross(l); // the game's gu = gf×gl [M 0x1a3b40] (cam_axes' up = right×f, same vector)
    [[l.x, l.y, l.z, 0.0], [u.x, u.y, u.z, 0.0], [f.x, f.y, f.z, 0.0], [cam.pos.x, cam.pos.y, cam.pos.z, 1.0]]
}

/// world → view: C = T(−eye)·Rᵀ (v_view = (v_world − eye)·Rᵀ, rows of the result = right/up/fwd
/// dots). `cam_view_to_world` is its exact inverse for orthonormal rows.
fn cam_world_to_view(cam: &Camera) -> nf_formats::anim::Mat4 {
    let mut m = cam_view_to_world(cam);
    // transpose the rotation, negate the translation: T(−p)·Rᵀ
    let (p, r) = ([m[3][0], m[3][1], m[3][2]], rot_transpose3(&m));
    let mut c = r;
    let dot = |row: usize| -(p[0] * c[0][row] + p[1] * c[1][row] + p[2] * c[2][row]);
    c[3] = [dot(0), dot(1), dot(2), 1.0];
    c
}

/// Transpose the 3×3 block, zeroing the translation row.
fn rot_transpose3(m: &nf_formats::anim::Mat4) -> nf_formats::anim::Mat4 {
    let mut r = *m;
    for i in 0..3 { for j in 0..3 { r[i][j] = m[j][i]; } }
    r[3] = [0.0; 4];
    r
}

/// The palette entries of the ACTIVE first-person view model (the object matrix, its bones and the
/// attachment slots hung on the view model character — the rigid gun parts, the muzzle flash).
fn view_model_entries(scene: &scene::Scene, l: &scene::SkinLayout) -> Option<Vec<usize>> {
    let s = scene.sim.as_ref()?;
    // fix-sentinel-camera-play: the ride draws no view model (the game's substate 10 cleared the
    // anim set [M `Player_ChangeSubState(10)`; L PCSX2 "no view model"]) — frame_state hides the
    // arms' palette; the reattach skips too.
    if s.missile_cam.is_some() { return None; }
    // fix-wallhug-prompt: the creep's third-person camera hides the view model too [M
    // `Player_PositionCamera` 0x18f5d8 case 0xb: `View_SetDrawInNoViews(BLData+0x7e8)`; the entry
    // holsters the weapon, `Player_WeaponNone` 0x196758:114].
    if s.bond.substate == nf_game::player::SS_CREEP { return None; }
    // nis-event9-cnis: the event-9 cut-scene orbit hides the view model with the same call [M
    // `Player_PositionCamera` 0x18f5d8 case 2: `View_SetDrawInNoViews(BLData+0x7e8)`, :148].
    if s.bond.cam_mode == 2 { return None; }
    let ci = *scene.view_models.get(&s.gun.weapon)?;
    let base = *l.char_base.get(ci)? as usize;
    let nb = scene.characters.get(ci)?.skeleton.bones as usize;
    let mut v: Vec<usize> = (base..=base + nb).collect();
    let att = l.attach_base as usize;
    for (k, a) in scene.attachments.iter().enumerate() {
        if a.character as usize == ci { v.push(att + k); }
    }
    Some(v)
}

/// fix-interp-ghosting: re-attach the first-person view model's palette entries (the hands and gun,
/// posed in world space from each tick's camera [M `Player_PositionGun` 0x1a3b40]) to the
/// interpolated draw camera, blending them in CAMERA space: for each entry in `vm_entries`, convert
/// both ticks' world matrices camera-relative with that tick's camera (`cam_prev` / `cam_cur`),
/// blend the relative poses at `alpha`, and re-attach the result to `draw_cam` — the same scheme
/// `lerp_cam` uses for the view itself, so mid-turn the gun cannot swim against the view. Entries
/// whose relative pose jumps more than `SNAP` snap to the newer tick (draw/holster switches), like
/// `lerp_palette`'s world-space snap. `out` is the blended palette (from `lerp_palette`), `prev` /
/// `cur` the two ticks' palettes. Pub for the headless interpolation tests to drive the exact path
/// the `RedrawRequested` handler runs.
pub fn reattach_view_model(out: &mut [nf_formats::anim::Mat4], prev: &[nf_formats::anim::Mat4], cur: &[nf_formats::anim::Mat4], alpha: f32,
                           cam_prev: &Camera, cam_cur: &Camera, draw_cam: &Camera, vm_entries: &[usize]) {
    let (vp, vc, vd) = (cam_world_to_view(cam_prev), cam_world_to_view(cam_cur), cam_view_to_world(draw_cam));
    for &k in vm_entries {
        let (Some(mp), Some(mc)) = (prev.get(k), cur.get(k)) else { continue };
        let Some(op) = out.get_mut(k) else { continue };
        let rel_p = nf_formats::anim::mul(mp, &vp);
        let rel_c = nf_formats::anim::mul(mc, &vc);
        // The camera-space snap mirrors `lerp_palette`'s world-space one: a relative-pose jump
        // (draw/holster switches, scoped hide) snaps to the newer tick instead of sweeping.
        let d = (0..3).map(|k2| (rel_p[3][k2] - rel_c[3][k2]).powi(2)).sum::<f32>().sqrt();
        if d > SNAP { *op = *mc; continue; }
        // Non-rotation entries (the collapse/FAR hides) keep the world blend `lerp_palette` wrote.
        if rot_quat(&rel_p).is_none() || rot_quat(&rel_c).is_none() { continue; }
        *op = nf_formats::anim::mul(&blend_pose(&rel_p, &rel_c, alpha), &vd);
    }
}

/// The far "hidden" translation every collapsed slot carries (`[0, -10000, 0]`).
fn collapsed(m: &nf_formats::anim::Mat4) -> bool { m[3][1] <= -9999.0 }

/// The gadget beams stretch from the
/// first-person watch to the hit point. They were blended in WORLD space while the watch is
/// re-attached to the interpolated camera, so mid-turn the beam's start left the watch. Blend them
/// in CAMERA space like the view model (component-wise: the beam's z axis is scaled by its length,
/// so it is not a rotation) and re-attach to the draw camera. A beam appearing or vanishing (the
/// collapsed slot) keeps the plain blend's snap.
pub fn reattach_beams(out: &mut [nf_formats::anim::Mat4], prev: &[nf_formats::anim::Mat4], cur: &[nf_formats::anim::Mat4], alpha: f32,
                      cam_prev: &Camera, cam_cur: &Camera, draw_cam: &Camera, entries: &[usize]) {
    let (vp, vc, vd) = (cam_world_to_view(cam_prev), cam_world_to_view(cam_cur), cam_view_to_world(draw_cam));
    for &k in entries {
        let (Some(mp), Some(mc)) = (prev.get(k), cur.get(k)) else { continue };
        if collapsed(mp) || collapsed(mc) { continue; }
        let Some(op) = out.get_mut(k) else { continue };
        let rel_p = nf_formats::anim::mul(mp, &vp);
        let rel_c = nf_formats::anim::mul(mc, &vc);
        let mut r = rel_c;
        for i in 0..4 { for j in 0..4 { r[i][j] = rel_p[i][j] + (rel_c[i][j] - rel_p[i][j]) * alpha; } }
        *op = nf_formats::anim::mul(&r, &vd);
    }
}


impl Camera {
    /// The screen-right axis from the yaw alone — `dir() × Y` normalised for |pitch| < 90°, and still defined at
    /// ±90° (the game's clamp limit), where the cross product degenerates and the view flipped upside down.
    fn right(&self) -> Vec3 { Vec3::new(-self.yaw.cos(), 0.0, self.yaw.sin()) }
    fn dir(&self) -> Vec3 { Vec3::new(self.pitch.cos() * self.yaw.sin(), self.pitch.sin(), self.pitch.cos() * self.yaw.cos()) }
    /// Right-handed view, wgpu clip depth 0..1, 60° vertical field of view (column-major for the shader).
    /// (full view-projection, the same with the eye at the origin for sky objects that follow the viewer).
    fn view_proj(&self, aspect: f32) -> ([[f32; 4]; 4], [[f32; 4]; 4]) {
        let (near, far, f) = (0.05f32, 4000.0f32, 1.0 / (self.fov * 0.5).tan());
        let proj = Mat4::from_cols_array_2d(&[[f / aspect, 0.0, 0.0, 0.0], [0.0, f, 0.0, 0.0], [0.0, 0.0, far / (near - far), -1.0], [0.0, 0.0, near * far / (near - far), 0.0]]);
        let fw = self.dir().normalize();
        // The right axis from the yaw alone (the game builds its view from yaw/pitch angles, not a
        // look-at): `fw × Y` degenerates at ±90° pitch (the game's own clamp limit, `Player_ViewClamping`
        // 0x1a92e8 ±1.0) and flipped the screen upside down there. For |pitch| < 90°
        // this is the same vector `fw × Y` normalised.
        let s = self.right();
        let u = s.cross(fw);
        let view = |e: Vec3| Mat4::from_cols_array_2d(&[[s.x, u.x, -fw.x, 0.0], [s.y, u.y, -fw.y, 0.0], [s.z, u.z, -fw.z, 0.0], [-s.dot(e), -u.dot(e), fw.dot(e), 1.0]]);
        ((proj * view(self.pos)).to_cols_array_2d(), (proj * view(Vec3::ZERO)).to_cols_array_2d())
    }
}

fn start_camera(scene: &scene::Scene) -> Camera {
    match scene.start {
        Some((p, yaw)) => Camera { pos: Vec3::new(p[0], p[1] + 1.6, p[2]), yaw, pitch: 0.0, fov: GAME_FOV },
        None => {
            let (lo, hi) = scene.bounds;
            Camera { pos: Vec3::new((lo[0] + hi[0]) / 2.0, hi[1], lo[2]), yaw: 0.0, pitch: -0.4, fov: GAME_FOV }
        }
    }
}

/// The `--bug-replay` frame: Bond's first-person view after the replay (head, yaw, pitch × π/2 — the HUD's own
/// camera convention), else the level's start camera.
pub(crate) fn bug_replay_shot(scene: &scene::Scene, out: &std::path::Path) -> Result<()> {
    let cam = match scene.sim.as_ref() {
        Some(s) => { let h = s.bond.head(); Camera { pos: Vec3::new(h[0], h[1], h[2]), yaw: s.bond.yaw, pitch: s.bond.pitch * std::f32::consts::FRAC_PI_2, fov: GAME_FOV } }
        None => start_camera(scene),
    };
    shot(scene, &cam, &out.to_string_lossy(), None, 0.0, scene::ViewOpts::default(), false, false, None, None)
}

/// Builds the sound driver + output from the scene's bank/music, and wires the music logic. The scene's bank,
/// music and tables are consumed (taken out) — the same path `main` uses at startup [M]. Public so the live
/// tests drive the exact runtime driver (fix-weapon-sounds).
///
/// The PS2 teardown/init it replaces [M grep ACTION.ELF decomp]: `Sound_Kill` 0x1ddfd0 +
/// `Sound_Ready` 0x1dceb8 call `SFXReset` 0x116a18 → `SFXReset2` 0x116a38 (batch op 0x0f,
/// ES_FadeReset — the IOP ends with no active items/voices, fade back to 10000), and
/// `Sound_Init` 0x1ddf88 calls `SFXInitialise` 0x1157b0 once per boot (bind the SFX.IRX RPC
/// 0x40013575, replay SetLanguage/Mode/Volume, op 0x00). Dropping the old `Sound` and
/// building a fresh `Driver` here (`Driver::new` inits `fade: 10000`) is the same state
/// transition. `SFXSuspendFileAccess` 0x1165d0 / `SFXUnSuspendFileAccess` 0x1166f8 (ops
/// 0x24/0x25 around `SP_LoadScript`/`Script_Free` loadable loads — IOP CD lockout) and
/// `SFXSetFilePath` 0x116c50 (op 0x14; dead — `Sound_Init` passes 0) have no host
/// equivalent: the port reads banks/streams from memory. `SFXSetNumberOfListeners` 0x116ec0
/// is cited at `nf_audio::sfx::Driver::set_listener`.
pub fn build_sound(scene: &mut scene::Scene) -> Option<nf_audio::sfx::Sound> {
    let sound = scene.sound_bank.take().map(|(slot, bank, sbf)| {
        let mut driver = nf_audio::sfx::Driver::new();
        driver.load_bank(slot, bank, sbf);
        // The spare banks: every other SBINFO slot (fix-weapon-sounds). The level's own stays primary, so
        // ids it carries resolve exactly as the one-bank driver did [M `Sound_Ready` 0x1dceb8].
        for (slot, bank, sbf) in std::mem::take(&mut scene.sound_spares) { driver.load_spare_bank(slot, bank, sbf); }
        if let Some((lut, bin)) = scene.streams.take() { driver.load_streams(lut, std::sync::Arc::new(bin)); }
        let output = nf_audio::Output::open();
        // Diagnostic sink note (unported-misc-10): the game's printf output sinks into
        // `deci2Putchar` 0x23cc90 — `_putchar` (OBJECT 0x2e7a58) swapped in by `scePrintf`
        // 0x23d570, emitting the boot banner "NFSV = 0x%\n" from `PS2FreshBoot` 0x120ab0 [M
        // chain in docs/systems/ps2-sdk-glue.md]. The port's equivalent is this eprintln!
        // diagnostic path (no DECI2 console exists).
        if output.is_none() { eprintln!("audio: no output device; running silent"); }
        let music_files = scene.music_files.take().map(|(t, smf, ssd)| (t, smf, std::sync::Arc::new(ssd)));
        nf_audio::sfx::Sound { driver, output, captured: None, music_files, music: None, music_pending: Vec::new() }
    });
    if let (Some(sim), Some((mm, vars))) = (scene.sim.as_mut(), scene.music_tables.take()) {
        if sim.drones.music.table.is_empty() {
            wire_music(sim, mm, vars);
        }
    }
    sound
}

/// `Sound_Ready`'s music wiring [M docs/systems/music.md]: the MapMusic/MapDroneData tables into the
/// sim's music logic + the attack gate, then `Music_Event(0, level)` / `Music_Event(1, 1)` — the start
/// pulse the first `UpdateMusicalEvents` consumes. fix-party-mission: the tests' `load` path now wires
/// the music too (the cut-scene music jumps were test-invisible before); `build_sound` reuses this.
fn wire_music(sim: &mut scene::Sim, mm: Vec<nf_formats::music::MapMusic>, vars: Vec<nf_formats::music::MusicVars>) {
    // The load path's sound reboot: `ResetMap_Load` 0x1d6360:24/:131 runs `Sound_Kill` 0x1ddfd0
    // (`SFXReset` when `Sound_Ready` had set SoundInit [M 0x1dceb8:107], SoundInit = 0) then
    // `Sound_Init` 0x1ddf88 (`DList_Init` pool, `SFXInitialise(0)`, `memset(&EventList,0,0x100)` =
    // the 64-u32 music event board) on EVERY load. The port takes the same job by rebuilding the
    // audio world here (build_sound's fresh driver/banks/streams, and this fresh engine with its
    // zeroed event board) — docs/systems/sound-lifecycle.md.
    sim.drones.attack_gate.vars = vars.iter().find(|v| v.level == sim.level).copied()
        .or(Some(nf_formats::music::MusicVars { level: sim.level, count: 2, floor: 0, window: 8, distance: 25.0 }));
    sim.drones.music = nf_game::music::Music::new(mm);
    sim.drones.music.event(0, sim.level);
    sim.drones.music.event(1, 1);
}

pub fn load(iso: &str, archive: &str, all: bool) -> Result<scene::Scene> {
    load_inner(iso, archive, all, false)
}

/// mp-core: the `--mp` load — the same parse, plus the match core (the placement registrations,
/// `MP_Init`/`MP_Start` and the human spawn at the policy's point) inside `load_inner`.
pub fn load_mp(iso: &str, archive: &str, all: bool) -> Result<scene::Scene> {
    load_inner(iso, archive, all, true)
}

fn load_inner(iso: &str, archive: &str, all: bool, mp: bool) -> Result<scene::Scene> {
    load_impl(iso, archive, all, None, false, &[], mp, None, 1, 0)
}

/// net-match-host: the ONLINE match load — the MP core from the negotiated settings (the host's
/// choices + seed, equal on every machine) instead of the `--mp` default. The level build draws
/// from the settings' seed (see `Scene::build`'s Rng note).
pub fn load_mp_carry(iso: &str, archive: &str, all: bool, save: Option<&nf_game::player::RamSave>, settings: nf_game::mp::MpSettings) -> Result<scene::Scene> {
    load_impl(iso, archive, all, save, false, &[], true, Some(settings), 1, 0)
}

/// mp-join-in-progress: the late joiner's headless fast-forward. Loads the
/// level from the host's handover settings (the same `load_mp_carry` path a
/// match start uses), replays the host's recorded run log tick by tick with
/// the net tick's driver shape (`Sim::step_net_remotes` + the local
/// `Bond::tick_ctx` + `set_slot` — the movement core every machine ticks),
/// and returns the scene plus the state hash at the last replayed tick. The
/// app compares that hash against the handover's `snapshot_hash` before
/// `NetCore::confirm_late_join` — a mismatch refuses the join
/// (`NetCore::late_join_verify_failed`). The returned scene IS the joiner's
/// live sim from the snapshot onward: it keeps ticking through the announced
/// join tick (the spawn via `Sim::mp_join_spawn`, its own slot's inputs) as
/// normal net ticks. Wall-clock measurement of the catch-up belongs to the
/// CALLER (the determinism rule keeps timers out of the Sim).
/// Render-check aid (NF_MP_FACE_OTHER=1): every human turns to the nearest other one at the
/// online match start - the same on every machine (the yaw is in the lockstep hash), and in the
/// late joiner's replay. Returns whether it ran.
fn face_other_aid(sim: &mut scene::Sim) -> bool {
    duel_aid(sim);
    if std::env::var_os("NF_MP_FACE_OTHER").is_none() { return false; }
    let mut all: Vec<(usize, [f32; 3])> = vec![(sim.mp_local_slot, sim.bond.pos)];
    for (s, b) in sim.mp_bonds.iter().enumerate() { if let Some(b) = b { all.push((s, b.pos)); } }
    let face = |me: [f32; 3]| all.iter().map(|(_, p)| *p).filter(|p| *p != me)
        .min_by(|a, b| { let d = |p: &[f32; 3]| (p[0] - me[0]).powi(2) + (p[2] - me[2]).powi(2); d(a).total_cmp(&d(b)) })
        .map(|o| (o[0] - me[0]).atan2(o[2] - me[2]));
    if let Some(y) = face(sim.bond.pos) { sim.bond.yaw = y; }
    for b in sim.mp_bonds.iter_mut().flatten() { if let Some(y) = face(b.pos) { b.yaw = y; } }
    // NF_MP_FACE_TURN1=radians: slot 1 turns away by that much (a side view of its body) — the same
    // on every machine (the env rides both windows).
    if let Some(d) = std::env::var("NF_MP_FACE_TURN1").ok().and_then(|v| v.parse::<f32>().ok()) {
        if sim.mp_local_slot == 1 { sim.bond.yaw += d; } else if let Some(b) = sim.mp_bonds.get_mut(1).and_then(|b| b.as_mut()) { b.yaw += d; }
    }
    // NF_MP_FACE_TURN="<slot>:<radians>[:<pitch>][,…]" (mp-remote-fx render aid): those slots then turn by
    // that much — e.g. 1:3.14159 faces slot 1 away, so its rounds land on the wall the host looks at.
    // Every machine applies it (the yaw is hashed).
    if let Ok(v) = std::env::var("NF_MP_FACE_TURN") {
        // "<slot>:<yaw>[:<pitch>]" — the optional pitch is SET (negative = down).
        for part in v.split(',') {
            let f: Vec<&str> = part.split(':').map(str::trim).collect();
            let (Some(sl), Some(dy)) = (f.first().and_then(|x| x.parse::<usize>().ok()), f.get(1).and_then(|x| x.parse::<f32>().ok())) else { continue };
            let pitch = f.get(2).and_then(|x| x.parse::<f32>().ok());
            let b = if sl == sim.mp_local_slot { Some(&mut sim.bond) } else { sim.mp_bonds.get_mut(sl).and_then(|b| b.as_mut()) };
            if let Some(b) = b { b.yaw += dy; if let Some(p) = pitch { b.pitch = p; } }
        }
    }
    true
}

/// Test aid (NF_MP_DUEL=<dist>, mp-combat-weapons): at the online match start slot 1's body is
/// moved onto the first spot `dist` from slot 0 (16 headings) on the same floor with a clear line
/// between the eyes — identically on every machine (the positions are in the lockstep hash, every
/// window runs it with the same env) — so the two-window weapon check fires across open ground.
/// Combine with NF_MP_FACE_OTHER=1.
fn duel_aid(sim: &mut scene::Sim) {
    let Some(dist) = std::env::var("NF_MP_DUEL").ok().and_then(|v| v.parse::<f32>().ok()) else { return };
    let local = sim.mp_local_slot;
    let body = |sim: &scene::Sim, s: usize| if s == local { Some(sim.bond.clone()) } else { sim.mp_bonds[s].clone() };
    let (Some(a), Some(_)) = (body(sim, 0), body(sim, 1)) else { return };
    for k in 0..16 {
        let ang = k as f32 * std::f32::consts::TAU / 16.0;
        let spot = [a.pos[0] + dist * ang.sin(), a.pos[1], a.pos[2] + dist * ang.cos()];
        let b = nf_game::player::Bond::stand_at(&sim.world, spot, 0.0);
        if (b.pos[1] - a.pos[1]).abs() > 0.2 || sim.world.segment(a.head(), b.head()).is_some() { continue; }
        let bp = b.pos;
        if local == 1 { sim.bond = b; } else { sim.mp_bonds[1] = Some(b); }
        if let Some(m) = sim.mp.as_mut() { m.set_slot(1, bp); }
        eprintln!("[net] duel aid: slot 1 placed {dist} from slot 0 at {bp:?}");
        return;
    }
    eprintln!("[net] duel aid: no clear spot {dist} from slot 0");
}

/// The online match start's sim setup as slot 0's machine (`swap_in_scene`'s net block: slot 0's
/// private generator, the facing aid). The late joiner's replay and the tests start from it.
pub fn net_slot0_setup(sim: &mut scene::Sim) {
    sim.net_rng = Some(nf_game::drone::Rng::new(0x5eed_0b0b));
    face_other_aid(sim);
}

/// One online tick as slot 0's machine runs it in the live loop: the local slot's full
/// `step_bond` on its session input, the remotes, the online combat, the slots' positions, the
/// state hash. The hashed state is the same on every machine whatever its local slot.
pub fn net_slot0_tick(sim: &mut scene::Sim, row: &[nf_net::Input; nf_net::MAX_AGENTS]) -> u64 {
    net_tick(sim, row, 0)
}

/// One online tick as the machine playing `local` runs it (the live loop's shape, the join
/// handling left out): the local slot's full `step_bond` on its session input, the remotes, the
/// online combat, the slots' positions, the state hash. Every machine must reach the same hash.
pub fn net_tick(sim: &mut scene::Sim, row: &[nf_net::Input; nf_net::MAX_AGENTS], local: usize) -> u64 {
    crate::scene::fold_component_hashes(&net_tick_comps(sim, row, local))
}

/// GAPS G-053 item 4: every active slot's match record takes its body's post-move
/// position, written in ONE place for the live loop and `net_tick` (docs/systems/
/// net-slot-positions.md §3). The two sites drifted once — net-match-host's live loop
/// wrote `for sl in 0..4` while the tick path wrote every slot — and a 5th+ human's
/// record then froze on the other machines while its own machine held the post-move
/// position (both `net_state_hash` folds read the record: the bodies' and the match's).
/// One shared writer keeps that class of drift impossible; the order is the slots
/// ascending with the local slot last, the shape every machine runs.
pub fn write_net_slot_positions(sim: &mut scene::Sim, local: usize) {
    if let Some(m) = sim.mp.as_mut() {
        for sl in 0..nf_net::MAX_AGENTS {
            if sl == local { continue; }
            if let Some(b) = &sim.mp_bonds[sl] { m.set_slot(sl, b.pos); }
        }
        m.set_slot(local, sim.bond.pos);
    }
}

/// net-desync-detail: `net_tick` with the named component hashes returned — the desync watcher's
/// per-tick feed (the combined hash the session compares is their fixed-order fold).
pub fn net_tick_comps(sim: &mut scene::Sim, row: &[nf_net::Input; nf_net::MAX_AGENTS], local: usize) -> [u64; crate::scene::NET_HASH_COMPONENTS] {
    let li = row[local];
    // mp-world-sync: the row rides into `step_bond` (the remotes move inside it, before the local
    // body; the call below is then a no-op).
    sim.net_row = Some(*row);
    sim.step_bond(mpnet::to_player_input(&li), li.trigger, li.reload, li.use_key);
    let inputs = std::array::from_fn(|i| mpnet::to_player_input(&row[i]));
    sim.step_net_remotes(&inputs, local);
    sim.step_net_combat(row, local);
    write_net_slot_positions(sim, local);
    sim.net_component_hashes(local)
}

pub fn mp_late_join_catchup(
    iso: &str,
    settings: nf_game::mp::MpSettings,
    log: &[[nf_net::Input; nf_net::MAX_AGENTS]],
    replay_ticks: u32,
) -> Result<(scene::Scene, u64), String> {
    mp_late_join_catchup_joins(iso, settings, log, replay_ticks, &[])
}

/// `mp_late_join_catchup` with the match's earlier late joins (slot, join tick — the host's
/// JOIN_BEGIN history): each body spawns before its tick exactly as the live machines spawned it
/// (`Sim::mp_join_spawn`; a reconnect into a standing body creates nothing). Without them a
/// second late join — or a late joiner's reconnect — could never verify (mp-online-failures).
pub fn mp_late_join_catchup_joins(
    iso: &str,
    settings: nf_game::mp::MpSettings,
    log: &[[nf_net::Input; nf_net::MAX_AGENTS]],
    replay_ticks: u32,
    joins: &[(u8, u32)],
) -> Result<(scene::Scene, u64), String> {
    let arena = format!("{:08x}.bin", settings.level_code);
    let mut scene = load_mp_carry(iso, &arena, false, None, settings)
        .map_err(|e| format!("late join: the arena load failed: {e}"))?;
    let sim = scene.sim.as_mut().ok_or("late join: the scene has no sim")?;
    // The replay IS slot 0's machine (the host's view): the same load-time setup the live match
    // start runs (`swap_in_scene` - slot 0's private generator, the facing aid) and the same tick
    // as the live loop (the local slot's full `step_bond` on its session input, the remotes, the
    // online combat, the slot positions, the hash). The old cut-down step (movement only) could
    // never reproduce the host's hash: the live tick's AI/bots/match rules and the shared stream
    // are all hashed (the three-window late-join check refused every joiner at the last tick).
    net_slot0_setup(sim);
    let mut prev = 0u64;
    for (i, row) in log.iter().enumerate() {
        let tick = i as u32;
        if tick > replay_ticks {
            break;
        }
        for (slot, jt) in joins {
            if *jt == tick { sim.mp_join_spawn(*slot as usize); }
        }
        prev = net_slot0_tick(sim, row);
    }
    Ok((scene, prev))
}

/// `--dev`: the same load with the dev view-model merge (folded 190-fix-dev2). Normal play never
/// passes `dev = true`, so no extra archive is opened without the flag.
pub fn load_dev(iso: &str, archive: &str, all: bool) -> Result<scene::Scene> {
    load_impl(iso, archive, all, None, true, &[], false, None, 1, 0)
}

/// `load` pre-merged for the given weapon ids (`--give` at startup, the dev menu's lazy give):
/// each id's first-person skin (+0xDC), meshes, skeleton, `07F` sequences and view scripts merge
/// from their owning archive before the scene builds, so the weapon draws exactly as it does on a
/// level that carries it (`weapon_assets::merge_weapon_assets`).
pub fn load_weapons(iso: &str, archive: &str, all: bool, wids: &[u8]) -> Result<scene::Scene> {
    load_impl(iso, archive, all, None, false, wids, false, None, 1, 0)
}

/// The sim catch-up cap (`NF_MAXTICKS`, default 4) — see the frame loop.
static MAX_TICKS: std::sync::OnceLock<u32> = std::sync::OnceLock::new();

/// net-consolidate: the state-hash cadence moved into the session's own
/// `hash_every` (nf_net `session_cfg`, mpnet.rs) — the app submits every tick
/// and the session sends + piggybacks on that schedule.
#[allow(dead_code)]
const NET_HASH_EVERY: u32 = 15;

/// `load` with the level-transition carry-over (fix-level2). `Some(save)` = a warm transition:
/// `Player_Start` (0x190fd0) picks the type-0x24 start whose gate channel the previous level's exit
/// wrote, and `Player_InitWeapon` (0x1a7ef8) runs the per-level loadout switch over `Player_RamLoad`
/// (0x1957a8). `None` = a cold load (the game's boot path; `Player_RamLoad`'s whole body is skipped).
pub fn load_with_carry(iso: &str, archive: &str, all: bool, save: Option<&nf_game::player::RamSave>) -> Result<scene::Scene> {
    load_impl(iso, archive, all, save, false, &[], false, None, 1, 0)
}

/// The carry-over load pre-merged for the given weapon ids (the dev menu's lazy give: the same
/// warm reload the level transition runs, plus the given weapons' assets).
pub fn load_with_carry_weapons(iso: &str, archive: &str, all: bool, save: Option<&nf_game::player::RamSave>, wids: &[u8]) -> Result<scene::Scene> {
    load_impl(iso, archive, all, save, false, wids, false, None, 1, 0)
}

/// `--dev` level transition: the carry-over load with the dev view-model merge.
pub fn load_with_carry_dev(iso: &str, archive: &str, all: bool, save: Option<&nf_game::player::RamSave>) -> Result<scene::Scene> {
    load_impl(iso, archive, all, save, true, &[], false, None, 1, 0)
}

/// difficulty-effects: the front-end's launch path — the mission loads with the difficulty the
/// player picked on the wheel (GameState+0x28 = 0x2a3790; every create-time gate reads it
/// [M C_SBNFDFCTY 0x216ca8:15, 0x1c7e78:25 boot default 2]). The plain `load*` wrappers keep 1
/// (the old direct-load replay parity).
pub fn load_with_carry_difficulty(iso: &str, archive: &str, all: bool, save: Option<&nf_game::player::RamSave>, difficulty: u32) -> Result<scene::Scene> {
    load_impl(iso, archive, all, save, false, &[], false, None, difficulty, 0)
}

/// pickup-medal-gate: the campaign load with the `bonus[0]` word in play — the game's `bonus[]`
/// global (`LS_LoadBonus` 0x1ce488 fills it from the save's BUNS block; nothing resets it per map
/// [M Menu_SetBonus 0x1ff898's writers]) that `PlrStats_HasGoldMedal` 0x199a40 reads for
/// `Pickup_Create`'s kind-6 gate [M 0x18dda0:93-95]. The front end passes the active profile's
/// word; the plain `load*` wrappers keep 0 (the fresh-boot BSS state [L ee_start_05 0x2e1b88]).
pub fn load_with_carry_medals(iso: &str, archive: &str, all: bool, save: Option<&nf_game::player::RamSave>, difficulty: u32, bonus: u64) -> Result<scene::Scene> {
    load_impl(iso, archive, all, save, false, &[], false, None, difficulty, bonus)
}

/// difficulty-effects: the warm reload with a weapon-asset set at the selected difficulty.
pub fn load_with_carry_weapons_difficulty(iso: &str, archive: &str, all: bool, save: Option<&nf_game::player::RamSave>, wids: &[u8], difficulty: u32) -> Result<scene::Scene> {
    load_impl(iso, archive, all, save, false, wids, false, None, difficulty, 0)
}

/// The fail/death menu's Restart serve — the reload loop's `level_exit_restart` branch
/// (`fix-restart-health`). `P_ENDMISSION_Handler` 0x216618:19-21 = `ResetMap_LevelToLoad(current,
/// 1, 1)` + `Player_RamLoad(Bond, 1)`; `Player_RamLoad` 0x1957a8 floors the health at the
/// difficulty's `ContinueHealthBoost*` — 50.0 on all three [L ACTION.ELF 0x30cc34-3c] — and the
/// disasm writes the max BACK INTO the buffer (`swc1 $f12, 0x5fc` at 0x1958e4) before
/// `Player_SetHealth` (jal 0x1960b8 at 0x1958e8, $f12 = the floored value, $a0 = BLData). The
/// boost arg 1 is the port's `RamSave::continue_boost` on this serve (fix-death-retry-boost),
/// with the difficulty global's port mirror `GameState+0x40` = `Progress::difficulty`
/// [M 0x1957a8:23]. The reload's `Player_InitWeapon` then re-runs the level's entry arm over
/// that buffer and re-saves it [M 0x1a7ef8:325]. So the serve is the ENTRY checkpoint — the
/// buffer as of this level's load — with the boost floor: after a COLD load the buffer holds the
/// fresh load's health 100 (`Player_Init` 0x1910f8:128-138 sets it; the `Player_RamLoad` body is
/// gated on `IsWarmReset` [M 0x1957a8:9], so nothing overwrites it; the unconditional re-save
/// captures it) — a restart after death starts at full health. `entry = None` = no load
/// has completed yet (the game's BSS zeros — unreachable in play, since every load ends in the
/// 0x1a7ef8:325 re-save): the shapes are zeroed at the live sizes and the boost keeps the health
/// at the floor (max(0, 50.0) = 50.0 [M 0x1957a8:21-34]).
pub fn fail_restart_save(entry: Option<&nf_game::player::RamSave>, sim: &scene::Sim) -> nf_game::player::RamSave {
    let mut s = entry.cloned().unwrap_or_else(|| {
        let mut z = nf_game::player::RamSave::capture(
            &sim.drones.health, sim.gun.switch_request, &sim.drones.inventory, &sim.drones.channels);
        z.health = 0.0; z.armour = 0.0;
        for v in z.carried.iter_mut() { *v = 0; }
        for v in z.clip.iter_mut() { *v = 0; }
        for o in z.owned.iter_mut() { *o = false; }
        for k in z.keys.iter_mut() { *k = false; }
        for c in z.channels.iter_mut() { *c = 0; }
        for t in z.ch_time.iter_mut() { *t = 0; }
        z
    });
    // The handler's `Player_RamLoad(Bond, 1)` [M 0x216618:21 → 0x1957a8:21-34]: the difficulty's
    // `ContinueHealthBoost*` max, written into the serve (the game writes it into the resident
    // buffer at 0x1958e4) — `continue_boost` is the game-logic port, not a viewer constant.
    s.continue_boost(sim.drones.progress.difficulty);
    // fix-objective-messages: the mission record states ride the game's `.data` and the restart
    // resets only the player (`P_ENDMISSION_Handler` 0x216618:19-21 — no mission reinit), so the
    // LIVE mission states ride the serve (the reloaded level's fresh `Mission` must not re-announce
    // the earlier parts' objectives).
    // The payload must be the shape the game's `Mission_Init(level, 1)` LEAVES the `.data` in when
    // the reload runs it: previous parts keep their states, current + future parts reset (enable →
    // 1, else 0) [M 0x182270:68-129]. The port applies the carried snapshot AFTER its
    // `Mission_Init` latch (drone.rs), so the dying run's dirt (the monitor's 2/3 on the fired
    // condition channels, the fail states) must be reset HERE — the game's RamLoad does not touch
    // the states, Mission_Init's reset is the last writer.
    let mut ms = sim.drones.mission.snapshot();
    {
        let mut off = 0usize;
        for l in nf_game::mission::LEVELS.iter() {
            let n = l.objectives.len();
            if l.mission == sim.drones.mission.base_map && l.part >= sim.drones.mission.order {
                for (j, o) in l.objectives.iter().enumerate() {
                    ms.states[off + j] = if o.enable != 0 { 1 } else { 0 }; // [M 0x182270:98-105]
                }
            }
            off += n;
        }
    }
    s = s.with_mission_snapshot(ms);
    s
}

/// The entry checkpoint a later fail-Restart restores: the buffer as of level ENTRY. The game
/// re-saves it with the POST-load state on EVERY load — `Player_InitWeapon`'s unconditional
/// `Player_RamSave` [M 0x1a7ef8:325] — cold loads included: `Player_Init`'s health 100 / armour 0
/// [M 0x1910f8:128-138] stand (the `Player_RamLoad` body is skipped when `IsWarmReset == 0` [M
/// 0x1957a8:9]) and the per-level arm's loadout/armour are what a restart gets back. On a warm
/// load the restored state is captured unchanged (health = the carried exit health [M 0x1957a8
/// SetHealth call], the arm's re-equips included).
pub fn entry_checkpoint_save(sim: &scene::Sim) -> nf_game::player::RamSave {
    let mut s = nf_game::player::RamSave::capture(
        &sim.drones.health, sim.gun.switch_request, &sim.drones.inventory, &sim.drones.channels)
        .with_mission(&sim.drones.mission);
    // 987-mission-stats-carry: the game's re-save tail copies the RESIDENT tally — which on a warm
    // load already holds the `Player_RamLoad` 0x1957a8:15-16 restore, because `Player_RamLoad` runs
    // inside `Player_InitWeapon` BEFORE the re-save [M 0x1a7ef8 arms → 0x1a8a00 → :325]. The port
    // applies the tally at the `Mission_Init` latch (the first tick), so a pre-latch entry capture
    // reads it from the carry; post-latch (or a cold load — carry `None`, the record zeroed by the
    // part-0 wipe) the live record is the truth.
    s.tally = sim.drones.tally_carry.or(Some(nf_game::player::TallySave {
        stats: sim.drones.progress.stats[0], bond_moments: sim.drones.progress.bond_moments,
    }));
    s
}

/// The warm level-change save — the capture `App::start_async_load`'s campaign arm (and the
/// weapon-assets warm reload) hands the worker, i.e. `Trigger_Activate`'s `Player_RamSave`
/// 0x195670: the player state + the mission globals + the mission tally (`PlrMissionStats[0]`
/// 0x38 + `BondMoments` 0x14 [M 0x195670:10-11] — the pause SCORE counters' ride across the
/// level change, 987-mission-stats-carry).
pub fn warm_exit_save(sim: &scene::Sim) -> nf_game::player::RamSave {
    nf_game::player::RamSave::capture(
        &sim.drones.health, sim.gun.switch_request, &sim.drones.inventory, &sim.drones.channels)
        .with_mission(&sim.drones.mission)
        .with_tally(&sim.drones.progress)
}

fn load_impl(iso_path: &str, archive: &str, all: bool, save: Option<&nf_game::player::RamSave>, dev: bool, give: &[u8], mp: bool, mp_settings: Option<nf_game::mp::MpSettings>, difficulty: u32, bonus: u64) -> Result<scene::Scene> {
    use nf_formats::{anim::Sequence, character, levelbin::LevelBin};
    use std::collections::HashMap;
    let mut t = std::time::Instant::now();
    let iso = IsoImage::open(iso_path)?;
    let (entries, bin) = disc::files_bin_entries(&iso)?;
    phase_mark("01 iso open + FILES.BIN", t); t = std::time::Instant::now();
    let read = |name: &str| -> Result<Option<Vec<u8>>> {
        let Some(fe) = entries.iter().find(|e| e.name.eq_ignore_ascii_case(name)) else { return Ok(None) };
        Ok(Some(iso.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64)?))
    };
    let data = read(archive)?.with_context(|| format!("no {archive} in FILES.BIN"))?;
    phase_mark("02 archive read", t); t = std::time::Instant::now();
    let mut level = Level::parse(&data)?;
    // ── The session object table (`models-weapons-items`) ───────────────────────────────────────────
    // `hashtable_delete_hashtype` is called from the loadable-pack path ONLY (`LoadableLoad`
    // 0x1d69e8 / `LoadableReload` 0x1d6ab0 — the cut-scene packs) in all of ACTION.ELF [M decompile
    // grep]: the map-object hashtable accumulates every archive loaded in the session, so a level's
    // weapon-model meshes resolve even when its own archive doesn't hold them. Measured [D]: Tower-2's
    // guards (07000011-13) carry weapons 0x10/0x1c/0x1d/0x1a, whose muzzle-flash meshes 0x02000261 /
    // 0x0200044a and the SMG's held mesh 0x0200025c are on NO archive from 07000011 on — the game
    // resolves them through the session table filled by the earlier story archives. The story order is
    // the case order of `ResetMap_LevelCode2Img` (0x1d5258): the level codes ascend in mission order
    // [docs/systems/levels.md §1], so the session before a level is exactly the archives with smaller
    // codes. The port merges every weapon-model hash the level archive misses from the latest earlier
    // story archive that holds it. Appending maps never shifts an existing (map, object) index, and
    // `by_hash` is first-wins, so every local resolution is unchanged.
    let lb = LevelBin::parse(&data)?;
    let mut skeletons: HashMap<u8, character::Skeleton> = lb.entries.iter().filter(|e| e.kind == 6)
        .map(|e| character::parse_skeleton(lb.data(&data, e)).map(|s| (s.id as u8, s)))
        .collect::<std::result::Result<_, _>>()?;
    // The level's animations: LoaderLoad rewrites the hashcode's 0x00F00000 nibble to 0xF for loader types 0/3,
    // so 070000NN.bin's sequences are in 07F000NN.bin (docs/formats/level-bin.md). `LoaderLoadAnims` 0x1d6b48
    // is the game's thin tail call into `LoaderLoad` 0x1d4a00 for this same animation load [M 0x1d6b48];
    // this block is the port's LoaderLoadAnims.
    let anim_name = format!("{}F{}", &archive[..2], &archive[3..]);
    // `AnimPreLoadInit` (0x1c63b8, from `ResetMap_GameInit` 0x1d5080 / `LoadableReload` 0x1d6ab0):
    // every level load starts the anim layer fresh — the tag/script timestamps, skin-scale globals
    // and the pSkeletons+0x1fc scratch it resets have no port counterparts (nothing persists across
    // loads), so these fresh skeleton/sequence/script maps ARE the reset.
    // The same ResetMap_GameInit (0x1d5080:37) calls `AnimCacheInit` 0x1c6390 — `jr ra; nop` [M
    // disasm 0x1c6390]: the shipped build's anim cache was emptied; there is nothing to init, so
    // the port models no cache gate (the family's Enable 0x1c6398 / Disable 0x1c63a0 likewise).
    // `ResetMap_Load` 0x1d6360 also calls `AnimSeqGetHdrBuf` 0x1c7998 (jal 0x1d67c4, a0=1) before
    // this loader loop — `jr ra; or v0,zero,zero`, always 0, and the caller never reads v0 [M
    // disasm 0x1d67c4-0x1d67d8]: a retired header-buffer hook; the port's load needs no hook.
    // `ResetMap_GameInit` also calls `Sound_SetLogSfxFile` 0x1de368 right before `Mission_Init`
    // [M 0x1d5080:43] — the shipped function is a bare `ret` [M 0x1de368], so the port preserves
    // the call as nothing (docs/systems/sound-lifecycle.md).
    let mut sequences: HashMap<u8, Vec<Sequence>> = HashMap::new();
    if let Some(anim) = read(&anim_name)? {
        let alb = LevelBin::parse(&anim)?;
        for e in alb.entries.iter().filter(|e| e.kind == 4 && e.length >= 0x40) {
            let s = Sequence::parse(alb.data(&anim, e))?;
            sequences.entry(s.skeleton).or_default().push(s);
        }
        // The game frees each seq's +0x84/+0x88 frame buffers with `AnimSeqDelete` (0x1c72e0) when its
        // script dies; the port's `Sequence` owns its decoded channels and drops with these maps.
    }
    phase_mark("03 level+levelbin parse, skeletons, 07F sequences", t); t = std::time::Instant::now();
    // The level's own sleeve, before any merge lends foreign ones (see the substitution below).
    let own_sleeve = SLEEVES.iter().copied().find(|h| level.by_hash.contains_key(h));
    merge_weapon_models(&iso, &read, &archive[..8], &mut level)?;
    phase_mark("04 weapon-models merge", t); t = std::time::Instant::now();

    // The dev view models (folded 190-fix-dev2): each level archive ships only its own weapons'
    // first-person skins (+0xDC), so the dev panel's "no 1st-person model here" covers most weapons.
    // With `--dev` the missing FP skins (and their textures, skeletons, sequences and the weapons'
    // anim scripts) merge from their holding archives — the same session-table mechanism the
    // weapon-model merge mirrors — and `Scene::build` then gives every weapon a view model. Dev
    // mode only: normal play loads no extra archive for skins.
    // The level's own kind-5 anim scripts, parsed once and shared with the DroneContext below.
    let mut level_scripts: HashMap<u32, nf_formats::anim::Script> = lb.entries.iter().filter(|e| e.kind == 5)
        .filter_map(|e| nf_formats::anim::Script::parse(lb.data(&data, e)).ok()).map(|x| (x.hashcode, x)).collect();
    // The weapon-asset merge (fix-weapon-assets): `--dev` pulls every missing fp skin as before
    // (folded 190-fix-dev2); a normal load pulls exactly the `--give` / dev-menu weapons it is
    // asked for. The gameplay-skeleton guard keeps the appended 07F sequences off every skeleton
    // a drone or character placement animates (the per-skeleton lists are order-sensitive gameplay
    // data — the loadable-pack sentinel rule); the fp hands/gun skeletons never carry those.
    let gameplay_skeletons: std::collections::HashSet<u8> = if dev || !give.is_empty() {
        let mut gs = std::collections::HashSet::new();
        if let Some(m) = level.main_map() {
            for p in level.maps[m].placements.iter() {
                let Some(&(_, skin)) = p.params.iter().find(|(i, _)| *i == 0) else { continue };
                if skin >> 24 != 5 { continue; }
                if let Some(sk) = level.skins.get(&skin) { gs.insert(sk.skeleton); }
            }
        }
        gs
    } else { Default::default() };
    let weapon_wanted = if dev { weapon_assets::Wanted::All }
        else if !give.is_empty() { weapon_assets::Wanted::Ids(give.to_vec()) }
        else { weapon_assets::Wanted::None };
    let mut weapons_table: Option<nf_formats::weapon::WeaponData> = None;
    // qworm-not-on-0c: the skin set `Player_InitWeapon`'s availability gate (0x1a7ef8 tail →
    // `Player_CheckWeaponsLoaded` 0x1a97c8) must see — the game's hashtable at loadout time. The
    // game's hashtable holds what its containers registered: this level's own archive skins plus
    // the session-accumulated ones the g-021 merge models below. The dev/`--give` asset merge's
    // foreign skins are a PORT viewing aid (they must not leak into the loadout, or the kit's
    // gadget lines — e.g. 0c's own EquipWeapon(0x58, 0) [M 0x1a8568] — own the Q-Worm on levels
    // whose session hashtable misses it). Snapshots: before the asset merge, after it, after g-021.
    let loadout_skins_pre_assets: std::collections::HashSet<u32> = level.skins.keys().copied().collect();
    // cutscene-handback-sp-path: the level's cut-scene packs' ANIM SET. The game loads the pack a
    // trigger names before its cut-scene: `LoadableLoad` 0x1d69e8 = AnimCacheDisable +
    // `hashtable_delete_hashtype(0x4000000)` (the 0x04-top-byte sequences wipe) + `LoaderLoad(2,
    // pack)` in a loop [M 0x1d69e8; LoaderProcess 0x1d4880 case 3 → AnimLoadFile 0x1bdf40:43-47
    // registers first-wins, `hashtable_additem` only on a miss]. The pack sequences are where the
    // cut-scene anims' root tracks live [D 06000501's windows 0x04000584/0x04000474 in 07900002.bin,
    // 06000504's 0x04000477/0x04000551 in 07800002.bin, 060006d5's 0x040006f0/0x04000601 in
    // 0790004a.bin — none of them in the level's 07F pack], so the runtime `Data`'s
    // `anim_obj_matrix`/`anim_length` (drone.rs) resolve kind-2 cut-scene streams ONLY with these
    // entries merged — without them the SP-path event-6 snapshot poses at the create identity.
    // Merge rule = the census's `AnimWorld::for_level` (nis_handback_census.rs), the level's own
    // entries first-wins (`or_insert` on every table); a pack sequence additionally skips any
    // hashcode a bucket already holds, so an appended entry never shifts an existing bucket index
    // or flips a `by_hash`/`seqs` resolution (Scene builds both after this). [S] unverified port
    // choice where a hash would collide: the game wipes sequences and holds the PACK's during the
    // scene (the 0x1d69e8 delete), the port keeps the union with the level's copy — no collision
    // exists on the pinned rows [D]. Skins ride in AFTER the loadout snapshot above (the
    // availability gate sees the level's own set only) and BEFORE the cut-scene-actor sweep below
    // (a pack skin satisfies its ScriptPlayer placement locally — the pack IS the game's source
    // for those actors). `share_datums` re-runs because the game's AnimPostLoadInit 0x1be1a8 runs
    // per skin-file load (each pack's registration included).
    let nn = &archive[3..8];
    for pfx in ["078", "079", "07A"] {
        let pack = format!("{pfx}{nn}.bin");
        let Some(fe) = entries.iter().find(|e| e.name.eq_ignore_ascii_case(&pack)) else { continue };
        let Ok(pd) = iso.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64) else { continue };
        let Ok(plb) = LevelBin::parse(&pd) else { continue };
        for e in plb.entries.iter().filter(|e| e.kind == 6) {
            if let Ok(s) = character::parse_skeleton(plb.data(&pd, e)) { skeletons.entry(s.id as u8).or_insert(s); }
        }
        for e in plb.entries.iter().filter(|e| e.kind == 5) {
            if let Ok(s) = nf_formats::anim::Script::parse(plb.data(&pd, e)) { level_scripts.entry(s.hashcode).or_insert(s); }
        }
        for e in plb.entries.iter().filter(|e| e.kind == 4 && e.length >= 0x40) {
            if let Ok(s) = Sequence::parse(plb.data(&pd, e)) {
                if sequences.values().any(|v| v.iter().any(|q| q.hashcode == s.hashcode)) { continue; }
                // The pack sequences ride the fix-cutscenes loader's SENTINEL key
                // (`0x80 | skeleton`), NOT the gameplay bucket: the game's `LoadableLoad`
                // 0x1d69e8 wipes the 0x4000000 hashtable and streams the pack's kind-4 rows,
                // so they REPLACE the cache while the scene plays and NEVER join the level's
                // per-skeleton sequence list — that list is order-sensitive gameplay data
                // (`frame_of`'s `k % len` idle cycle, the skeleton-filtered `seq_index`). The
                // sentinel key still reaches the by-hash map `Scene::seqs` (flattened over
                // every bucket), the only reader a cut-scene's `Script_AnimStart` resolution
                // and the kind-2 `anim_obj_matrix` pose use.
                sequences.entry(s.skeleton | 0x80).or_default().push(s);
            }
        }
        for e in plb.entries.iter().filter(|e| e.kind == 3) {
            if let Ok(s) = character::parse_skin(plb.data(&pd, e), |id| skeletons.get(&id).map(|k| k.bones)) {
                level.skins.entry(s.hashcode).or_insert(s);
            }
        }
    }
    nf_formats::level::share_datums(&mut level.skins);
    if weapon_wanted != weapon_assets::Wanted::None {
        // weapon_data once for the whole load (the EE initialiser run the merge needs is the same
        // one the DroneContext consumes below; `--dev` used to run it twice).
        let elf = iso.read_file(iso.find("/ACTION.ELF").context("no ACTION.ELF on the disc")?)?;
        weapons_table = match nf_formats::weapon::WeaponData::from_action_elf(&elf) {
            Ok(mut w) => { w.apply_player_init_override(u32::from_str_radix(&archive[..8], 16).unwrap_or(0)); Some(w) }
            Err(e) => { eprintln!("weapon_data: {e} — drones drawn unarmed"); None }
        };
        if let Some(wd) = weapons_table.as_ref() {
            let index = weapon_assets::DiscIndex::for_iso(&iso, iso_path);
            let t = std::time::Instant::now();
            let have_scripts: std::collections::HashSet<u32> = level_scripts.keys().copied().collect();
            let st = weapon_assets::merge_weapon_assets(&iso, &read, &mut level, &mut skeletons, &mut sequences,
                &mut level_scripts, &have_scripts, &gameplay_skeletons, wd, &weapon_wanted, &index, dev,
                &casing_meshes(&elf))?;
            eprintln!("weapon assets: merged {} archive(s), {} skin(s), {} map(s), {} sequence(s), {} script(s) in {:.2}s",
                st.archives, st.skins, st.maps, st.sequences, st.scripts, t.elapsed().as_secs_f32());
            // weapon-model-coverage: the honest final tally — the session mesh merge's per-stage
            // "still missing" count is before this merge ran. The residue is the set on NO archive
            // of the disc (the shipped game cannot resolve it either, e.g. the two casing meshes
            // 0x02000446/0x02000443 [D census]).
            let still: Vec<u32> = weapon_assets::meshes_for_wanted(&level, wd, &weapon_wanted, &casing_meshes(&elf))
                .into_iter().filter(|h| !level.by_hash.contains_key(h)).collect();
            if !still.is_empty() {
                eprintln!("weapon models: {} hash(es) unresolved after the merge, on no archive of the disc: {}",
                    still.len(), still.iter().map(|h| format!("{h:08x}")).collect::<Vec<_>>().join(" "));
            }
        }
    }
    // The weapon_data the DroneContext consumes: the hoisted build when the merge ran, else built
    // here (a normal load without --give never opens another archive).
    let weapons_table = match weapons_table {
        Some(w) => Some(w),
        None => match nf_formats::weapon::WeaponData::from_action_elf(&{
            iso.read_file(iso.find("/ACTION.ELF").context("no ACTION.ELF on the disc")?)?
        }) {
            Ok(mut w) => { w.apply_player_init_override(u32::from_str_radix(&archive[..8], 16).unwrap_or(0)); Some(w) }
            Err(e) => { eprintln!("weapon_data: {e} — drones drawn unarmed"); None }
        },
    };
    let loadout_skins_post_assets: std::collections::HashSet<u32> = level.skins.keys().copied().collect();
    phase_mark("05 weapon_data (ACTION.ELF)", t); t = std::time::Instant::now();

    // G-021 (models-characters): drone placements whose skin (placement param 0) is not in their
    // level's archive — 33 of them on 2 levels [D scan]. TWO mechanisms, read off the decompile:
    //
    // 1. `AnimObjectNew` (0x1be910 disasm 0x1be958-0x1be9b0) SUBSTITUTES 0x05000090 before the
    //    hashtable get: on GameState+0xc == 0x0700000c → 0x05000087 (0700000c's own skin), on
    //    0x07000014 → 0x05000093 (07000014's own). The port mirrors it at load: the placement hash
    //    0x05000090 resolves to the substitute skin's data — exactly the object the game builds.
    // 2. 07000014's 0x050000b1 (11 placements) has NO substitution in `AnimObjectNew` (the
    //    disassembly covers only 0x05000090) — but the RENDER request is never the raw hash:
    //    `NDrone2_DefaultInit` REWRITES the param-0 0x050000b1 to 0x0500008c in Drone+0xe0 on
    //    GameState+12 == 0x07000014, before the kind switch, the captain block and the
    //    `AnimObjectNew` handoff [M 0x14b300:82-95, :563-565] (found by cite-14-skinless-drones;
    //    the earlier reading here concluded the placements draw skinless). 0x0500008c is in
    //    07000014's own archive [D]; the raw 0x050000b1 lives in 07000015/16's archives [D] and
    //    never registers on a fresh story run (the skin table accumulates every archive loaded in
    //    the session — `AnimLoadFile` 0x1bdf40 registers per map; nothing frees between levels —
    //    0x1d8248 is `Script_Load`, the old mislabel). `CommonAnimFile` (the LoaderLoad type-4
    //    name source, the variable at 0x30d860 [M 0x1d4a00 case 0: `param_2 = CommonAnimFile` —
    //    the only reference in ACTION.ELF, no writer]) is 0 at every level-start snapshot [L
    //    ee_start_05.bin / ee_start_07000001.bin / ee_start_0700000a.bin @0x30d860], so the common
    //    file is not the source here either. The port still merges the holding archive for skins a
    //    placement DOES request: its maps (the skin's meshes are glist objects of that archive),
    //    object hashcodes (first wins), skins and skeletons — and its 07F sequences ONLY for
    //    skeletons the level has no sequences for (a NEW bucket; the per-skeleton lists are
    //    order-sensitive gameplay data — the WORKLOG's warning).
    {
        let mm = level.main_map();
        // The substitution [M 0x1be910]: 0x05000090 → the level's own variant, before anything
        // else looks the hash up. The compared word is `GameState+0xC` @0x2a3774 (the current
        // level id; the "+0x12" in older notes was the decompiler's decimal offset 12).
        match u32::from_str_radix(&archive[..8], 16).unwrap_or(0) {
            0x0700_000c | 0x0700_0014 => {
                let level_id = u32::from_str_radix(&archive[..8], 16).unwrap_or(0);
                let sub = if level_id == 0x0700_000c { 0x0500_0087u32 } else { 0x0500_0093 };
                if let Some(mut sk) = level.skins.get(&sub).cloned() {
                    // The port keys its skin tables by the placement's hash (the sim's `dr.skin`,
                    // the draw caches, the 0x800 ambient tables): the substituted entry keeps the
                    // placement's identity while carrying the variant's data.
                    sk.hashcode = 0x0500_0090;
                    level.skins.entry(0x0500_0090).or_insert(sk);
                }
            }
            _ => {}
        }
        // bond-hitbox-pose: the PLAYER's skin on every level. `Player_Init` 0x1910f8:94-108 resolves
        // the level's player-skin hash off the level id and hands it to `AnimObjectNew` — the
        // hashtable always resolves it (a skinless Bond is never shipped). The disc carries each
        // level's own player skin in its own archive — 0x05000009 in 01-04, 0x050000d in 05-08,
        // 0x0500007c in 16/1b/43, 0x05000007 in the other story archives [D nf-extract bin-list
        // census, kind-3 heads; all 0x05000007 carriers share sha1 04709295…, the record is
        // identical everywhere] — except 07000048 (the front-end container, no player skin at all).
        // There the session table's earlier registration stands [M AnimLoadFile 0x1bdf40 registers
        // per container and nothing frees skins — the g-021 session model below]. The port: when
        // the own archive lacks the level's player skin, merge that ONE skin record from a carrier
        // (session carriers first — 07000048 or an earlier story level — else the first carrier
        // [S the front-end chain's container order is not observable from here]; the record is
        // byte-identical across carriers [D], so the source cannot change the table's content).
        {
            let cur = u32::from_str_radix(&archive[..8], 16).unwrap_or(0);
            let want = nf_game::damage::player_skin_hash(cur);
            if !level.skins.contains_key(&want) {
                static PLAYER_SKIN_CARRIERS: std::sync::OnceLock<HashMap<u32, Vec<String>>> = std::sync::OnceLock::new();
                let carriers = PLAYER_SKIN_CARRIERS.get_or_init(|| {
                    let mut map: HashMap<u32, Vec<String>> = HashMap::new();
                    for e in entries.iter().filter(|e| e.name.len() == 12 && e.name.ends_with(".bin")) {
                        let Ok(d) = iso.read_bytes(bin.lba as u64 * 2048 + e.offset as u64, e.size as u64) else { continue };
                        let Ok(l) = LevelBin::parse(&d) else { continue };
                        for pe in l.entries.iter().filter(|pe| pe.kind == 3) {
                            let raw = l.data(&d, pe);
                            if raw.len() < 4 { continue }
                            let h = u32::from_le_bytes(raw[0..4].try_into().unwrap());
                            if (h & 0xff00_0000) == 0x0500_0000 { map.entry(h).or_default().push(e.name.clone()); }
                        }
                    }
                    map
                });
                let story_pos = |id: u32| nf_game::mission::LEVELS.iter().position(|l| l.level == id);
                let cur_pos = story_pos(cur);
                let session = |name: &str| -> bool {
                    if name.eq_ignore_ascii_case("07000048.bin") { return true; }
                    let Some(id) = name.get(..8).and_then(|s| u32::from_str_radix(s, 16).ok()) else { return false };
                    match (story_pos(id), cur_pos) {
                        (Some(p), Some(c)) => p < c,
                        (Some(_), None) => true,
                        (None, _) => false,
                    }
                };
                if let Some(list) = carriers.get(&want) {
                    let found = list.iter().find(|n| session(n)).or_else(|| list.first()).cloned();
                    if let Some(archive_name) = found {
                        if let Some(fd) = read(&archive_name)? {
                            let foreign = Level::parse(&fd)?;
                            if let Some(sk) = foreign.skins.get(&want) {
                                level.skins.insert(want, sk.clone());
                            } else {
                                eprintln!("player skin: carrier {archive_name} parsed without {want:08x}");
                            }
                        }
                    }
                } else {
                    eprintln!("player skin: {want:08x} is on no archive of the disc — Bond's hit boxes will be empty here");
                }
            }
        }
        let mut wanted: Vec<u32> = Vec::new();
        if let Some(m) = mm {
            for p in level.maps[m].placements.iter().filter(|p| p.flags & 0xff == 0x0f) {
                if let Some(&v) = p.params.iter().find(|(k, _)| *k == 0).map(|(_, v)| v) {
                    if v != 0 && !level.skins.contains_key(&v) && !wanted.contains(&v) { wanted.push(v); }
                }
            }
        }
        // The cut-scene actors: every ScriptPlayer placement's script's `Anim` skins.
        if let Some(m) = mm {
            for p in level.maps[m].placements.iter().filter(|p| p.flags & 0xff == 0xd9) {
                let Some(&v) = p.params.iter().find(|(k, _)| *k == 0).map(|(_, v)| v) else { continue };
                let Some(nis) = level.nis.get(&v) else { continue };
                for st in nis.streams.iter() {
                    for cmd in st.decode() {
                        if let nf_formats::nis::Cmd::Anim { skin, .. } = cmd {
                            if skin != 0 && !level.skins.contains_key(&skin) && !wanted.contains(&skin) { wanted.push(skin); }
                        }
                    }
                }
            }
        if !wanted.is_empty() {
            // G-021 (drone-skins-g021): the merge source must be a container the SESSION has
            // already loaded. The skin table holds what the session's containers registered:
            // `LoaderLoad` 0x1d4a00 opens a hash-named .bin container, walks its sub-file
            // directory, and `LoaderProcess` 0x1d4880 case 3 hands each type-3 sub-file to
            // `AnimLoadFile` 0x1bdf40, which registers it first-wins (`hashtable_additem` only
            // when `hashtable_getitem` 0x1ca5b0 misses); NOTHING frees skins afterwards — the
            // only table deletions are `LoadableLoad`/`LoadableReload` 0x1d69e8/0x1d6ab0's
            // hashtype-4 wipe (0x1ca520 compares the hash top byte == 0x04, sequences) and
            // `Script_FreeFromHashTable` 0x1daec0's single scripts. A fresh story session
            // before level L therefore holds the front-end container (boot, 0x07000048) plus
            // every earlier story level's container (mission::LEVELS order). A container the
            // session has not loaded is NOT a source: 07000014's raw 0x050000b1 is carried only
            // by 07000015/16's archives [D scan of all 99 containers — the only carriers:
            // 07000016 entry 139 @0x8594a0 len 0x860, 07000015 entry 102 @0x609e60 len 0x860],
            // which the story loads AFTER 14, so the table never registers it. The placements
            // still resolve: `NDrone2_DefaultInit` REWRITES the param-0 0x050000b1 to the
            // own-archive 0x0500008c in Drone+0xe0 before any read [M 0x14b300:82-95, :563-565;
            // cite-14-skinless-drones], and the captains swap on top of that (0x14b300:536-555:
            // kind 6 → 0x05000093, kind 7 → 0x050000b2 — the sim's `captain_setup`). A skin a
            // placement DOES request with no session carrier (07000001's script lane) just
            // misses — `AnimObjectNew` 0x1be910:44-46 returns 0, `NDrone2_DefaultInit`
            // 0x14b300:563-565 discards the return and the placement draws no character.
            static SKIN_ARCHIVES: std::sync::OnceLock<HashMap<u32, Vec<String>>> = std::sync::OnceLock::new();
            let index = SKIN_ARCHIVES.get_or_init(|| {
                let mut map: HashMap<u32, Vec<String>> = HashMap::new();
                for e in entries.iter().filter(|e| e.name.len() == 12 && e.name.ends_with(".bin")) {
                    let Ok(d) = iso.read_bytes(bin.lba as u64 * 2048 + e.offset as u64, e.size as u64) else { continue };
                    let Ok(l) = LevelBin::parse(&d) else { continue };
                    for pe in l.entries.iter().filter(|pe| pe.kind == 3) {
                        let raw = l.data(&d, pe);
                        if raw.len() < 4 { continue }
                        let h = u32::from_le_bytes(raw[0..4].try_into().unwrap());
                        if (h & 0xff00_0000) == 0x0500_0000 { map.entry(h).or_default().push(e.name.clone()); }
                    }
                }
                map
            });
            let story_pos = |id: u32| nf_game::mission::LEVELS.iter().position(|l| l.level == id);
            let cur = u32::from_str_radix(&archive[..8], 16).unwrap_or(0);
            let cur_pos = story_pos(cur);
            // The session gate: the front-end container (boot) and every story level BEFORE the
            // current one. A story container loaded later, and the MP/bonus containers the SP
            // story never opens, are not sources. Game side: the hashtable is rebuilt on every
            // level load [M ResetMap_Load 0x1d6360:23,130 hashtable_initialise] and each loaded
            // archive registers under its hash ids [M AnimLoadFile 0x1bdf40:43-47
            // hashtable_additem], the chain bringing the shared 0x7000500 container then the
            // level's own data [M 0x1d6360:28-36] — this gate is the port's substitute for that
            // per-load chain. The non-story branch is a port-dev case: the shipped game's loads
            // are all story / MP / bonus / front-end (0x7000048) walks [M 0x1d6360] and a skin
            // hash the chain never brings in just misses — the placement draws no character [M
            // AnimObjectNew 0x1be910:44-46]. Dev/packs loads (port-only) keep the story
            // containers eligible.
            let session_container = |name: &str| -> bool {
                if name.eq_ignore_ascii_case("07000048.bin") { return true; }
                let Some(id) = name.get(..8).and_then(|s| u32::from_str_radix(s, 16).ok()) else { return false };
                match (story_pos(id), cur_pos) {
                    (Some(p), Some(c)) => p < c,
                    (Some(_), None) => true,
                    (None, _) => false,
                }
            };
            let mut merged: Vec<String> = Vec::new();
            for want in wanted.iter() {
                let Some(carriers) = index.get(want) else { eprintln!("g-021: skin {want:08x} is on no archive of the disc"); continue };
                let Some(archive_name) = carriers.iter().find(|n| session_container(n)) else {
                    eprintln!("g-021: skin {want:08x} is carried only by {} — no container the session has loaded; the table never registers it. On 07000014 the 0x050000b1 placements still resolve through the Drone+0xe0 rewrite [M 0x14b300:82-95, :563-565]; a skin a placement DOES request with no session carrier just misses (AnimObjectNew 0x1be910:44-46) and draws no character",
                        carriers.join("/"));
                    continue;
                };
                if merged.contains(archive_name) { continue; }
                merged.push(archive_name.clone());
                let Some(fd) = read(archive_name)? else { continue };
                let mut foreign = Level::parse(&fd)?;
                let flb = LevelBin::parse(&fd)?;
                let map_base = level.maps.len();
                // `LevelMap` is not `Clone`: move the maps out of the parsed foreign level.
                level.maps.append(&mut foreign.maps);
                let foreign_by_hash = std::mem::take(&mut foreign.by_hash);
                for (h, (m, o)) in foreign_by_hash.iter() {
                    level.by_hash.entry(*h).or_insert((map_base + m, *o));
                }
                for (&h, sk) in foreign.skins.iter() {
                    level.skins.entry(h).or_insert_with(|| sk.clone());
                }
                for e in flb.entries.iter().filter(|e| e.kind == 6) {
                    let sk = character::parse_skeleton(flb.data(&fd, e))?;
                    if skeletons.contains_key(&(sk.id as u8)) { continue; }
                    if let Some(anim) = read(&format!("{}F{}", &archive_name[..2], &archive_name[3..]))? {
                        let alb = LevelBin::parse(&anim)?;
                        for pe in alb.entries.iter().filter(|pe| pe.kind == 4 && pe.length >= 0x40) {
                            let s = Sequence::parse(alb.data(&anim, pe))?;
                            if s.skeleton == sk.id as u8 { sequences.entry(s.skeleton | 0x80).or_default().push(s); }
                        }
                    }
                    skeletons.insert(sk.id as u8, sk);
                }
                eprintln!("g-021: merged {} for skins {:?} ({} maps, {} skins)", archive_name,
                    wanted.iter().filter(|w| level.skins.contains_key(w)).map(|w| format!("{w:08x}")).collect::<Vec<_>>(),
                    foreign.maps.len(), foreign.skins.len());
            }
        }
    }
    }
    // The sleeve substitution (fix-missing-drone-skins): `AnimProcessSkinData` 0x1bd500:159-166
    // reroutes every 0x10-flagged MAIN entry through `AnimSleeveGetEntity(0)` 0x1c6590 — the first
    // present entity of the fixed SleeveEnts list @0x2c6ed8 — before the celglist lookup; none
    // present → celglist 0 (nothing drawn). The port applies it once here, after the session
    // merges above, so every downstream reader (character_instances' mesh walk, the scene build,
    // the sim) sees exactly the hashes the game's runtime skin records carry.
    // `AnimSleeveGetEntity(0)` picks the FIRST present entity of SleeveEnts, and the game's level only
    // ever holds its own Bond's sleeve. The session/give merges can lend other levels' sleeves earlier
    // in that list (they ride in on borrowed skins), which then won over the level's own: the 08
    // launcher drew another level's grey glove instead of the tuxedo cuff and bare hand (PCSX2 side
    // by side, 2026-10-17). A level with its own sleeve drops the borrowed ones before the pick.
    // qworm-not-on-0c: the loadout's availability set = the level's own skins + the game-accurate
    // session additions (the g-021 merge and the 0x05000090 substitution above) — NOT the dev/`--give`
    // asset merge's foreign skins (the viewing aid) [M 0x1a97c8's hashtable read; the game's
    // hashtable only holds what its containers registered, D the archive skin scan]. On 0700000c
    // this keeps the kit's EquipWeapon(0x58, 0) refused: the Q-Worm fp skin 0x050005b is carried
    // only by 07000009/0a/0b (+ the MP/bonus 41/46 containers), not by 0700000c.bin [D scan].
    let loadout_skins: std::collections::HashSet<u32> = {
        let mut s = loadout_skins_pre_assets.clone();
        for (&h, _) in level.skins.iter() {
            if !loadout_skins_post_assets.contains(&h) { s.insert(h); }
        }
        s
    };
    if let Some(own) = own_sleeve {
        for h in SLEEVES { if h != own { level.by_hash.remove(&h); } }
    }
    {
        let present = |h: u32| level.by_hash.contains_key(&h);
        for sk in level.skins.values_mut() { sk.substitute_sleeves(present); }
    }
    // The loadable cut-scene packs (fix-cutscenes): `SP_LoadScript` (0x1dbc08) checks `isLoadable(hash)`
    // (0x1d69a0 — the table filled by `LoaderProcess` 0x1d4880 case 0xC from the level archive's type-0xC
    // entries, one record per entry: u32 script hashcode at +0, u8 flag at +4) and, when it hits, streams the
    // pack file named with the CURRENT LEVEL's id and the (flag+7) nibble at bits 20-23 (`LoadableLoad`
    // 0x1d69e8: `GameState+0xC & 0xff0fffff | ((flag+7) << 20)`; `LoaderLoad` 0x1d4a00 case 0 sprintf's that
    // into "%08x.bin"; flag 1 → nibble 8, flag 2 → nibble 9, flag 3 → nibble A). Measured on the disc
    // [D]: 07000007's records (0x06000072, 2) and (0x06000073, 1) → 07900007.bin / 07800007.bin, which hold
    // exactly those type-7 scripts plus their kind-4 sequences; 07000008's (0x06000077, 1) / (0x06000078, 2)
    // → 07800008.bin / 07900008.bin likewise. These scripts exist nowhere else — the party's Drake fly-through
    // (0x06000072), the party's hand-back cut-scene (0x06000073) and the gondola level's intro (0x06000077)
    // are pack-only. Without the packs the port's ScriptPlayers hit the missing-script arm of `Player::load`
    // (state 5, deleted) on their first trigger and the cut-scene never plays. The
    // pack's kind-7 entries go into `level.nis` (the same lib `Scripts::new` reads) and kind-4 into
    // `sequences`, mirroring `parsemap_parsemap` (LoaderProcess case 2 → the map parser walks the entries:
    // kind 7 → `Script_Load`, kinds 3/4/5 → `AnimLoadFile`).
    let level_id = u32::from_str_radix(&archive[..8], 16).unwrap_or(0);
    for e in lb.entries.iter().filter(|e| e.kind == 0xC) {
        let r = lb.data(&data, e);
        if r.len() < 5 { continue; }
        let (hash, flag) = (u32::from_le_bytes([r[0], r[1], r[2], r[3]]), r[4]);
        if flag == 0 || flag > 3 { continue; }
        let pack_name = format!("{:08x}.bin", (level_id & 0xff0f_ffff) | ((flag as u32 + 7) << 20));
        let Some(pack) = read(&pack_name)? else { eprintln!("loadable pack {pack_name} not on the disc (script {hash:08x})"); continue };
        let plb = LevelBin::parse(&pack)?;
        for pe in plb.entries.iter() {
            match pe.kind {
                7 => { let h = pe.hash(); match nf_formats::nis::Nis::parse(h, plb.data(&pack, pe)) { Ok(n) => { level.nis.insert(h, n); }, Err(err) => eprintln!("pack {pack_name} script {h:08x}: {err}") } }
                // Pack sequences go under `0x80 | skeleton` — a key no skin's skeleton ever uses. The game
                // DELETES the animation hashtable before streaming a pack (`LoadableLoad` 0x1d69e8's
                // `hashtable_delete_hashtype(0x4000000)`, then `AnimLoadFile` for the pack's kind-4 entries):
                // the cut-scene's sequences REPLACE the cache while it plays and never join the level's
                // per-skeleton sequence list. In the port that list is order-sensitive gameplay data —
                // `frame_of`'s `k % len` idle cycle and the skeleton-filtered `seq_index` drone lookups —
                // and appending to it changed guards' animations and broke live tests (07000001's
                // `dropped_guns_dim_to_the_cel_ambient` "the guard died"). (`LoadableLoad` 0x1d69e8's
                // first call `AnimCacheDisable` 0x1c63a0 — like `LoadableReload` 0x1d6ab0's
                // `AnimCacheEnable` 0x1c6398 — is an EMPTY function in ACTION.ELF, so the port needs
                // no anim-cache gate around this stream, only the 0x4000000 flush modelled here.)
                // The sentinel key still reaches
                // the by-hash map `Scene::seqs` (built by flattening every bucket), which is the only thing
                // a cut-scene's `Script_AnimStart` resolution reads (`script_frame`'s hash → `seqs.get`).
                4 if pe.length >= 0x40 => { let s = Sequence::parse(plb.data(&pack, pe))?; sequences.entry(s.skeleton | 0x80).or_default().push(s); }
                _ => {}
            }
        }
        eprintln!("loadable pack {pack_name}: {} scripts, {} sequences (for {hash:08x})", plb.entries.iter().filter(|e| e.kind == 7).count(), plb.entries.iter().filter(|e| e.kind == 4).count());
    }
    // Drone start animations: ACTION.ELF's tables (read through its symbols) and the level's scripts (type 5).
    phase_mark("06 skin merge + loadable packs", t); t = std::time::Instant::now();
    let elf = iso.read_file(iso.find("/ACTION.ELF").context("no ACTION.ELF on the disc")?)?;
    // mp-character-data: the bot slots' character word BEFORE the scene build — `BOT_init`
    // 0x1249e8:160 writes the bot record's character byte (+0x10) into
    // `MPSettings+slot*0x30+0x24` (and reads the body's skin from `MP_skins[rec[0x10]]`,
    // 0x1249e8:100). The port's `MpMatch::create_bots` repeats the write when the match starts;
    // the scene build parks the bodies earlier, so it happens here off the same records.
    // mp-body-look: the bots sit at `human_count + i` with the records cycled (`MpMatch::create_bots`,
    // mp-16-slots); this mirror still wrote slots 4.. from the first 4 records, so with 2 humans
    // the bot bodies (slots 2, 3) drew character 0 (Bond) instead of their records' characters
    // (Snow Guard 6 / Black Ops 7) — render before_bot2_f200.png.
    let mp_settings = mp_settings.map(|mut st| {
        let humans = st.human_count as usize;
        let bots = (st.bot_count as usize)
            .min(nf_game::mp::MP_MAX_SLOTS.saturating_sub(humans))
            .min(st.players.len().saturating_sub(humans));
        for i in 0..bots {
            st.players[humans + i].skin = nf_game::mp::MpMatch::bot_record(&st, i).character() as u32;
        }
        st
    });
    let drones = scene::DroneContext {
        tables: nf_formats::drone::DroneAnimData::from_action_elf(&elf)?,
        // The level's own kind-5 scripts plus the weapon-asset merge's foreign view scripts (the
        // merged FP skins' idle/fire/reload) — the merge inserts missing hashes only, so a hash
        // the level has stayed local.
        scripts: level_scripts,
        level_id: u32::from_str_radix(&archive[..8], 16).unwrap_or(0),
        // weapon_data is built at boot by its static initialiser; run it over the disc's ELF (docs/formats/weapon-data.md).
        props: nf_formats::drone::PropertyTables::from_action_elf(&elf).ok(),
        // Built once above (the merge consumes the same table); `Player_Init` (0x1910f8)'s override
        // of weapon_data[1]+0xDC (the fists skin) is already applied.
        weapons: weapons_table,
        tuning: {
            let mut t = nf_formats::tuning::Tuning::from_action_elf(&elf).unwrap_or_default();
            nf_game::tuning::read_tuning_vars(&mut t, u32::from_str_radix(&archive[..8], 16).unwrap_or(0));
            t
        },
        hitboxes: nf_formats::hitbox::CollisionBones::from_action_elf(&elf).ok(),
        sfx: nf_formats::sfx::SfxTable::from_action_elf(&elf).ok(),
        surfaces: nf_formats::sfx::SurfaceSounds::from_action_elf(&elf).map_err(|e| eprintln!("EffectInfo: {e}")).ok(),
        voices: nf_formats::sfx::voice_lines(&elf).map_err(|e| eprintln!("Drone_SfxInfo: {e}")).ok(),
        hud: nf_formats::hud::HudTables::from_action_elf(&elf).map_err(|e| eprintln!("HUD tables: {e}")).ok(),
        // mp-character-data: the MP body tables, read from the same ELF [D]. Parsed on every load
        // (cheap — 29 × 0x10 + 16 × 0x28 bytes); only the MP body build consumes them.
        mp_skins: nf_formats::mpanim::MpSkins::from_action_elf(&elf).map_err(|e| eprintln!("MP_skins: {e}")).ok(),
        mp_anims: nf_formats::mpanim::MpAnimSets::from_action_elf(&elf).map_err(|e| eprintln!("MP anim sets: {e}")).ok(),
        // net-match-host: the online match builds the level from the host's seed (equal on every
        // machine — the session's settings blob carries it); 0 keeps the pre-net constant.
        mp_seed: mp_settings.as_ref().map_or(0, |st| st.seed),
        mp_body_skins: std::array::from_fn(|slot| mp_settings.as_ref().map_or(0, |st| nf_game::mp::slot_skin(st, slot))),
        // pickup-medal-gate: the `bonus[0]` word the kind-6 gate reads [M 0x199a40 → 0x18dda0:93-95].
        bonus,
    };
    let mut scene = scene::Scene::build(&level, !all, &skeletons, sequences, Some(&drones), difficulty, mp_settings.as_ref());
    phase_mark("07 scene build (Scene::build)", t); t = std::time::Instant::now();
    // The MP radar's textures (hud::MP_TEXTURES) — MP scenes only: the SP pane list has no radar.
    if mp {
        for h in hud::MP_TEXTURES.into_iter().chain([hud::HEADER_FONT_TEX]) {
            if scene.hud_textures.iter().any(|(k, _)| *k == h) { continue; }
            if let Some(img) = level.maps.iter().enumerate().find_map(|(mi, m)| m.textures.iter().find(|t| t.hashcode == h).map(|t| scene::TextureImage::from_texture(mi, t))) {
                scene.hud_textures.push((h, img));
            } else {
                eprintln!("mp hud: texture {h:#010x} not in the arena archive");
            }
        }
    }
    // The pause/fail menus' burning woman [M `psiDecompressWoman` 0x1202a8]: the archive's kind-16 "woman" run-length
    // mask filled from the fire texture 0x030000da, drawn into 0x030000db's texels through 0x030000da's CLUT (the
    // disc's 0x030000db is a placeholder). The HUD texture starts on the first decoded frame.
    {
        let tex = |h: u32| level.maps.iter().flat_map(|m| m.textures.iter()).find(|t| t.hashcode == h);
        if let (Some(e), Some(fire)) = (lb.entries.iter().find(|e| e.kind == 16), tex(0x0300_00da)) {
            let mut w = woman::Woman::new(lb.data(&data, e).to_vec(), fire.frame(0).to_vec(), fire.palette_rgba8());
            w.tick();
            if let Some((_, t)) = scene.hud_textures.iter_mut().find(|(h, _)| *h == 0x0300_00db) {
                if t.width as usize == woman::W && t.height as usize == woman::H { t.rgba = w.rgba(); }
            }
            scene.woman = Some(w);
        }
    }
    // The HUD's language strings (`fix-hud`): `Txt_LoadLanguage` 0x1caad0 loads `FNames[SelLang]` — the
    // NTSC-U default is `USATxt.dat` [D FILES.BIN; FNames 0x244680]. Parsed per its on-disc layout.
    if let Some(s) = scene.sim.as_mut() {
        match read("USATxt.dat") {
            Ok(Some(bytes)) => match nf_formats::hud::Lang::parse(&bytes) {
                Ok(lang) => s.lang = Some(lang),
                Err(e) => eprintln!("USATxt.dat: {e} — HUD labels/messages blank"),
            },
            Ok(None) => eprintln!("no USATxt.dat in FILES.BIN — HUD labels/messages blank"),
            Err(e) => eprintln!("USATxt.dat: {e}"),
        }
    }
    if std::env::var("NF_DRAWS").is_ok() {
        let mut c: std::collections::BTreeMap<String, (usize, u32)> = Default::default();
        for m in &scene.materials { let e = c.entry(format!("{:?} {:?}", m.layer, m.draw)).or_default(); e.0 += 1; e.1 += m.index_count / 3; }
        for (k, (n, t)) in c { eprintln!("{k}: {n} materials, {t} triangles"); }
        let mut names: std::collections::BTreeMap<String, usize> = Default::default();
        for inst in level.instances(!all).0 { let o = &level.maps[inst.map].objects[inst.object]; if o.flags & 1 != 0 { *names.entry(format!("{} flags {:#x}", o.name.trim(), o.flags)).or_default() += 1; } }
        for (k, n) in names { eprintln!("list 1: {k} x{n}"); }
        let mut fl: std::collections::BTreeMap<String, usize> = Default::default();
        for inst in level.instances(!all).0 { let o = &level.maps[inst.map].objects[inst.object]; if o.flags & 0xc != 0 { *fl.entry(format!("{} flags {:#x}", o.name.trim(), o.flags)).or_default() += 1; } }
        for (k, n) in fl { eprintln!("flag 4/8: {k} x{n}"); }
        for t in level.maps.iter().flat_map(|m| &m.textures).filter(|t| t.frames > 1) { eprintln!("animated texture {:#010x}: {}x{} {} frames, rate {}", t.hashcode, t.width, t.height, t.frames, t.rate); }
        let mut va: std::collections::BTreeMap<u8, usize> = Default::default();
        let mut low: std::collections::BTreeMap<String, usize> = Default::default();
        let mut pr: std::collections::BTreeMap<(u16, bool), usize> = Default::default();
        for inst in level.instances(!all).0 {
            let o = &level.maps[inst.map].objects[inst.object];
            if o.flags & 1 != 0 { continue; }
            for l in &o.gfx.leaves { for b in &l.batches { *pr.entry((b.prim, b.verts.iter().any(|v| v.rgba[3] < 0x7c))).or_default() += 1; for v in &b.verts { *va.entry(v.rgba[3]).or_default() += 1; if v.rgba[3] < 0x7c { *low.entry(o.name.trim().to_string()).or_default() += 1; } } } }
        }
        eprintln!("list-0 batch prims (prim, has low alpha): {pr:?}");
        if let (Some(w), Some(sim)) = (drones.weapons.as_ref(), scene.sim.as_ref()) {
            let r = &w.records[2];
            for o in [0xa0usize, 0xa4, 0xa8, 0xac, 0xb0, 0xb4, 0xb8, 0xbc] {
                let h = u32::from_le_bytes(r[o..o + 4].try_into().unwrap());
                let Some(sc) = sim.scripts.get(&h) else { continue };
                let ops: Vec<String> = sc.commands.iter().filter(|c| c.op == 1 || c.op == 2).map(|c| format!("op{} {:x?}", c.op, c.args)).collect();
                eprintln!("P2K +{o:#x} script {h:08x} len {}: {}", sc.length, ops.join(", "));
            }
        }
        if let Ok(name) = std::env::var("NF_LEAVES") {
            for inst in level.instances(!all).0 {
                let o = &level.maps[inst.map].objects[inst.object];
                if o.name.trim() != name { continue; }
                for (li, l) in o.gfx.leaves.iter().enumerate() {
                    for b in &l.batches {
                        let (lo, hi) = b.verts.iter().fold((255u8, 0u8), |(lo, hi), v| (lo.min(v.rgba[3]), hi.max(v.rgba[3])));
                        let p0 = b.verts.first().map(|v| v.pos);
                        eprintln!("leaf {li} gs {:x?} tex {:?} verts {} alpha {lo}..{hi} rgb0 {:?} p0 {p0:?}", l.gs, b.texture, b.verts.len(), b.verts.first().map(|v| v.rgba));
                    }
                }
                break;
            }
        }
        for (k, n) in low.iter().take(40) { eprintln!("low alpha: {k} x{n}"); }
        let anim = level.maps.iter().flat_map(|m| &m.textures).filter(|t| t.frames > 1).count();
        eprintln!("animated textures: {anim}");
    }
    // The level's sound bank (`Sound_Ready`): its hash's slot in SBINFO.SBI, then SB_<slot>.{SHF,SFX,SBF}.
    phase_mark("08 woman + lang", t); t = std::time::Instant::now();
    let mut scene = scene;
    let level_id = u32::from_str_radix(&archive[..8], 16).unwrap_or(0);
    scene.sound_bank = (|| -> Option<(usize, nf_formats::soundbank::Bank, Vec<u8>)> {
        let hash = nf_game::sound::level_bank(level_id)?;
        let get = |p: &str| iso.find(p).and_then(|e| iso.read_file(e).ok());
        let slot = nf_formats::soundbank::bank_slots(&get("PS2/SBINFO.SBI")?).ok()?.iter().position(|&h| h == hash)?;
        let f = |ext: &str| get(&nf_formats::soundbank::bank_path("ENGLISH", slot, ext));
        let bank = nf_formats::soundbank::Bank::parse(&f("SHF")?, &f("SFX")?).map_err(|e| eprintln!("sound bank: {e}")).ok()?;
        Some((slot, bank, f("SBF")?))
    })();
    // fix-weapon-sounds: the spare banks — every other SBINFO slot with files on the disc, so a weapon
    // given on a level whose own bank lacks a sound still plays it from a bank that holds it. Only ids
    // the level's own bank lacks ever resolve to a spare (the driver's primary-first lookup), so every
    // level-carried sound is bit-identical to the one-bank behaviour [M `Sound_Ready` 0x1dceb8; S the
    // extension — the real game loads one bank per level].
    scene.sound_spares = (|| -> Vec<(usize, nf_formats::soundbank::Bank, Vec<u8>)> {
        let get = |p: &str| iso.find(p).and_then(|e| iso.read_file(e).ok());
        let Some(sbi) = get("PS2/SBINFO.SBI") else { return Vec::new() };
        let Some(slots) = nf_formats::soundbank::bank_slots(&sbi).ok() else { return Vec::new() };
        let own = scene.sound_bank.as_ref().map(|(s, _, _)| *s);
        let mut spares = Vec::new();
        for (slot, &hash) in slots.iter().enumerate() {
            if hash == u32::MAX || Some(slot) == own { continue; }
            let f = |ext: &str| get(&nf_formats::soundbank::bank_path("ENGLISH", slot, ext));
            let (Some(shf), Some(sfx), Some(sbf)) = (f("SHF"), f("SFX"), f("SBF")) else { continue };
            match nf_formats::soundbank::Bank::parse(&shf, &sfx) {
                Ok(bank) => spares.push((slot, bank, sbf)),
                Err(e) => eprintln!("sound bank slot {slot}: {e}"),
            }
        }
        spares
    })();
    // Streamed effects (guards' speech): STREAMS.LUT / STREAMS.BIN.
    scene.streams = (|| {
        let get = |p: &str| iso.find(p).and_then(|e| iso.read_file(e).ok());
        Some((nf_formats::soundbank::stream_lut(&get("PS2/ENGLISH/STREAMS/STREAMS.LUT")?).ok()?, get("PS2/ENGLISH/STREAMS/STREAMS.BIN")?))
    })();
    // Music (`Sound_Ready` → `Music_Event(0, level)`; docs/systems/music.md): the MapMusic / MapDroneData tables, and
    // the level's track through MFXINFO.MXI to PS2/MUSIC/MFX_<n>.{SMF,SSD}.
    phase_mark("09 sound banks + streams", t); t = std::time::Instant::now();
    scene.music_tables = nf_formats::music::tables(&elf).map_err(|e| eprintln!("music tables: {e}")).ok();
    // The music logic rides every load (the game's `Sound_Ready` runs before the first frame whether or
    // not an output device exists); the audio driver takes the tables later in `build_sound`.
    if let (Some(sim), Some((mm, vars))) = (scene.sim.as_mut(), scene.music_tables.as_ref()) {
        if sim.drones.music.table.is_empty() {
            wire_music(sim, mm.clone(), vars.clone());
        }
    }
    scene.music_files = (|| {
        let track = scene.music_tables.as_ref()?.0.iter().find(|m| m.level == level_id && m.track != 0)?.track;
        let get = |p: &str| iso.find(p).and_then(|e| iso.read_file(e).ok());
        let n = nf_formats::music::track_file(&get("PS2/MFXINFO.MXI")?, track).ok()??;
        let smf = nf_formats::music::Smf::parse(&get(&nf_formats::music::music_path(n, "SMF"))?).ok()?;
        Some((track, smf, get(&nf_formats::music::music_path(n, "SSD"))?))
    })();
    if let Some((t, smf, ssd)) = &scene.music_files { eprintln!("music: track {t}, {} sections, {} markers, {:.1} s", smf.starts.len(), smf.markers.len(), ssd.len() as f64 * 7.0 / 8.0 / 32000.0); }
    match &scene.sound_bank { Some((slot, b, _)) => eprintln!("sound bank slot {slot}: {} samples, {} effects", b.samples.len(), b.effects.len()), None => eprintln!("no sound bank") }
    eprintln!("{archive} (+ {anim_name}): {}", scene.stats);
    // ── Player_Start (0x190fd0) + Player_InitWeapon (0x1a7ef8) ─────────────────────────────────────
    // The game registers BOTH start sets from the main map's placements in parse order — type 0x24
    // as kind 1, type 0x2D as kind 0 (`Player_AddNewStartPos` 0x195c00 via
    // `parsemap_create_dynamic_objects` 0x1d04f0 case 0x24/0x2d [M disasm 0x1d0a00/0x1d0e98]) — and
    // `Player_Start` picks the first kind-1 entry whose gate is 0 or on with the channels as the
    // previous level's exit left them (they survive the warm reset, `Init_SwitchChannels` runs only
    // when `IsWarmReset == 0` [M 0x1d5080]); the kind-0 entries never win either pass [M 0x190fd0
    // disasm 0x191028/0x1910b0]. Then the per-level loadout switch. The port used to stand Bond at
    // the first "Player1" placement with the [L] 07000005 loadout at EVERY level.
    if let Some(sim) = scene.sim.as_mut() {
        // The mission globals ride the RamSave (the game keeps them in `.data` across the load; the
        // port restores them at the `Mission_Init` latch — map-tower2).
        sim.drones.mission_carry = save.and_then(|s| s.mission.clone());
        // 987-mission-stats-carry: the mission tally rides the same RamSave (the game's
        // `Player_RamSave` 0x195670:10-11 / `Player_RamLoad` 0x1957a8:15-16 pair).
        sim.drones.tally_carry = save.and_then(|s| s.tally);
        let starts: Vec<nf_game::player::LevelStart> = level_starts(&level);
        if !starts.is_empty() {
            // The gate scan reads the carried channels on a warm transition (the save mirrors the live
            // globals — `Player_RamSave` snapshots them at the exit [M 0x195670:13-15]).
            let scan: nf_game::triggers::Channels = match save {
                Some(s) => nf_game::triggers::Channels { v: s.channels.clone(), time: s.ch_time.clone(), writes: None, script_channels: Default::default() },
                None => nf_game::triggers::Channels::default(),
            };
            if let Some(i) = nf_game::player::pick_start(&starts, &scan) {
                let s = starts[i];
                if let Some((old, _)) = scene.start { eprintln!("start: default {:?} → start {} gate {:#x} at {:?}", old, i, s.gate, s.pos); }
                // `Player_Init(0, pos, rot, params)` creates the player object at the entry; the port
                // stands him on the floor below (the level start convention, `Bond::stand_at`).
                sim.bond = nf_game::player::Bond::stand_at(&sim.world, s.pos, s.yaw);
                // The spawn arrival settle — the LEVEL-START path only (`arm_spawn_settle` doc):
                // the boot holds the placed root four ticks then drops it into the band [L
                // ee_start_0700000a root 1.0891111 bit-exact]; the gate is [S] unverified. The
                // mid-game `Player_Enable` 0x195498 does NOT go through `Player_StandAtNewPosition`
                // (the matrix origin direct [M :57-68]) and must not carry the hold — the 05 truck
                // hand-off's crouch press is the first post-enable tick.
                sim.bond.arm_spawn_settle();
                // `Player_Init` 0x1910f8:152-172 [M]: the default WldGravity write (0, −9.8, 0),
                // then p3 == 2 → `Vec_Zero(&WldGravity)` (the spacewalk starts; disasm 0x19151c-
                // 0x191550 the default, 0x191618 the zero). fix-world-gravity.
                sim.bond.world_gravity = if s.p3 == 2 { [0.0; 3] } else { [0.0, -9.8, 0.0] };
                // p3 == 2 also STARTS substate 8: the same `level start == 2` branch calls
                // `Player_StandAtNewPosition(.., 8)` [M Player_Init 0x1910f8 :178-183 — the
                // spacewalk spawn enters Player_ZeroG's dispatch case straight away; the daily
                // audit 2026-10-17: the port kept the walk substate].
                if s.p3 == 2 { sim.bond.substate = nf_game::player::SS_ZEROG_8; }
                // p3 == 3 → `Player_StandAtNewPosition(.., 4)` (the crouch substate) and
                // `BLData+0x90C = 0.45` (0x3ee66666, the full crouch eye drop) [M Player_Init 0x1910f8
                // :172-176]. 07000014 starts this way inside its crawl vent [L: PCSX2 substate 4,
                // BL+0x90C 0.45 at the start]; standing there put the eye above the duct roof.
                if s.p3 == 3 {
                    sim.bond.crouched = true;
                    sim.bond.cc = nf_game::player::CROUCH_ROOT_HEIGHT;
                    sim.bond.crouch_drop = 0.45;
                }
                scene.start = Some((s.pos, s.yaw));
            }
        } else if let Some((pos, yaw)) = scene.start {
            // No type-0x24 starts parsed: the game creates NO player here — both Player_Start
            // passes require a kind-1 record and there is no third fallback [M 0x190fd0:11-32].
            // The port's Bond is structural (every scene carries a sim), so the fallback stands
            // at the "Player1" placement — the game's own start-object name — for the loads
            // without 0x24 starts (the front end, dev maps); MP arenas take the 0x25/0x26 spawn
            // path below [M parsemap 0x1d04f0 case 0x25/0x26].
            sim.bond = nf_game::player::Bond::stand_at(&sim.world, pos, yaw);
        }
        // mp-core: the MP arenas have no type-0x24 starts — `--mp` runs the match core instead:
        // the 0x25/0x26 registrations (`parsemap_create_dynamic_objects` 0x1d04f0 case 0x25/0x26 —
        // the team and the MP object type are the placement's param 0, the block+0x2c slot [M
        // 0x1d1b68]), `MP_Init` 0x183438 / `MP_Start` 0x183a50, then the human spawn at the policy's
        // point (the `Player_Init` 0x1910f8 port — the yaw is the placement rotation[1], the same
        // convention `level_starts` pins).
        if mp {
            // net-match-host: the online path hands the negotiated settings here (the host's
            // choices + seed); local `--mp` keeps the deathmatch default.
            let local_default = mp_settings.is_none();
            let mut settings = mp_settings.unwrap_or_else(|| nf_game::mp::MpSettings::deathmatch(level_id));
            // Render aid (NF_MP_BOTS=n, local `--mp` only): n bots in the default match, so a
            // headless `--mp --shot` shows a full 16-participant HUD (n is clamped to the 16 cap).
            if local_default {
                if let Some(n) = std::env::var("NF_MP_BOTS").ok().and_then(|v| v.parse::<u32>().ok()) {
                    settings.bot_count = n.min(nf_game::mp::MP_MAX_SLOTS as u32 - settings.human_count.max(1) as u32);
                }
                // mp-hud-real render aids: NF_MP_SCENARIO=<hex scenario word> (e.g. 20000002 Team
                // Arena; `MpMatch::init` derives the team/bit30 flags from it) and
                // NF_MP_TIME=<seconds> (the time limit — the PCSX2 comparison's 600).
                if let Some(sc) = std::env::var("NF_MP_SCENARIO").ok().and_then(|v| u32::from_str_radix(v.trim_start_matches("0x"), 16).ok()) {
                    settings.scenario = sc;
                    settings.gametype_mask = sc;
                }
                if let Some(t) = std::env::var("NF_MP_TIME").ok().and_then(|v| v.parse::<i32>().ok()) {
                    settings.time_limit = t;
                }
            }
            // mp-lobby-settings: one line with every lobby word the match runs on (the two-window
            // settings check diffs the host's line against each client's).
            {
                let st = &settings;
                let bots: Vec<String> = st.bot_records.iter().map(|r| format!("c{}/s{}/t{}/hp{}", r.character(), r.aggression(), r.team(), r.max_hp())).collect();
                let hum: Vec<String> = (0..st.human_count as usize).map(|i| format!("{}:t{}/c{}", st.players[i].codename, st.players[i].team, st.players[i].skin)).collect();
                eprintln!("[mp-settings] level {:#x} scen {:#x} ws {} frag {} time {} ff {} radar {} cap(0x1BC) {} loc(0x1C8) {} spawn(0x1C0) {} guns(0x1B8) {} expl(0x1D4) {} grapple(0x1D0) {} vehicles(0x1CC) {} humans [{}] bots {} [{}] seed {:#x}",
                    st.level_code, st.scenario, st.weapon_set, st.frag_limit, st.time_limit, st.friendly_fire, st.radar, st.capture_reward,
                    st.location_damage, st.spawn_policy, st.gun_emplacements, st.explosive_scenery, st.grapple, st.vehicles,
                    hum.join(" "), st.bot_count, bots.join(" "), st.seed);
            }
            let mut m = nf_game::mp::MpMatch::init(settings);
            // Render aid (mp-hud-real): NF_MP_KILLMSG=<victim slot> queues the kill message the
            // local player gets for that victim (the MP info pane's draw, headless).
            if local_default {
                if let Some(v) = std::env::var("NF_MP_KILLMSG").ok().and_then(|v| v.parse::<usize>().ok()) {
                    m.kill_msgs.push((0, v.min(nf_game::mp::MP_MAX_SLOTS - 1)));
                }
            }
            let map = level.main_map().and_then(|mi| level.maps.get(mi));
            let mut spawn_count = 0usize;
            if let Some(map) = map {
                let param = |p: &nf_formats::placement::Placement, k: u32| -> u32 {
                    p.params.iter().find(|(i, _)| *i == k).map_or(0, |&(_, v)| v)
                };
                for p in &map.placements {
                    match p.flags & 0xff {
                        0x25 => {
                            let team = param(p, 0) as u16;
                            let before = m.spawns.count;
                            m.register_spawn_point(p.translation, p.rotation, team, &sim.world);
                            if m.spawns.count > before { spawn_count += 1; }
                        }
                        0x26 => {
                            let ty = param(p, 0);
                            let cfg: [u32; 7] = std::array::from_fn(|k| param(p, k as u32));
                            // mp-scenarios-online (agent6): the placement model's bounds — the
                            // celglist the game's object keeps at obj+0xD8 [M MP_CreateObject
                            // 0x185de8:118] (centre/radius = +0x10/+0x1C, box = +0x20/+0x30 [D
                            // MapObject]) — give the flags/terminals/items their contact sphere and
                            // the hill its box.
                            let bounds = level.by_hash.get(&p.hashcode).map(|&(mi, oi)| {
                                let o = &level.maps[mi].objects[oi];
                                nf_game::mp::MpObjBounds { centre: o.centre, radius: o.radius, min: o.min, max: o.max, sphere_off: [0.0; 3] }
                            });
                            m.register_mp_object_bounds(p.translation, p.rotation, ty, cfg, p.hashcode, bounds);
                        }
                        _ => {}
                    }
                }
            }
            m.start(&mut sim.drones.rng);
            // mp-host-match-logic: the mode objects build here — `MP_Start` 0x183a50:77-114 creates
            // them (the demo/prot/blueprint/GE draws — Deathmatch/CTF draw nothing) BEFORE the
            // humans' spawn picks below, the game's draw order. The headless rigs call
            // `create_mode_objects` themselves; the live host must too, or no scenario object ever
            // exists (`update_objects` runs on an empty `objects_rt`).
            m.create_mode_objects(&mut sim.drones.rng);
            // mp-goldeneye-strike: the strike's flight length is the SP script 0x0600003e's own
            // length — the arena's type-7 entry (`SP_Create` 0x1dc7e0's script, `SP_Update`
            // 0x1dbda8 plays it to its end → the game's landed test `SP_GetState` 0x1dc900 == 2
            // [M 0x187568:96]; 299 ticks on 07000023 [D]). 0 = the arena has no strike script:
            // the ScriptPlayer never plays and the strike never lands [M Script_Load 0x1d8248's
            // failed-lookup arm].
            m.objects_rt.ge_flight_ticks = level.nis.get(&0x0600_003e).map_or(0, |n| n.length);
            // Every human's pick: `MP_Start` calls `MP_GetSpawnPoint(team, NULL)` BEFORE
            // `Player_Init` creates the obj [M 0x183a50:120-123] — no slots exist yet, so every used
            // point is a 9999-distance candidate (policy-dependent pick, empty slots). net-match-host:
            // ALL human slots spawn here in slot order with the SAME draws on every machine, so the
            // online match's bodies agree; slot 0's bond takes `sim.bond` (the pre-net behaviour) and
            // the rest park in `Sim::mp_bonds` until each machine swaps its own slot in
            // (`Sim::swap_local_slot`).
            // The game's human loop is slots 0..player_count [M MP_Start 0x183a50:120-128 —
            // MP_GetSpawnPoint then Player_Init per slot], the shipped count topping out at the
            // 4 pad slots its menus offer; the bots clamp at 4 [M 0x183a50:135-137]. The port
            // extends the same walk to every human slot of the 16-cap settings (the documented
            // mp-16-slots port extension) — the game's loop is the
            // shape this walk ports.
            let humans = (m.settings.human_count as usize)
                .clamp(1, nf_game::mp::MP_MAX_SLOTS)
                .min(sim.mp_bonds.len());
            let mut first_spawn = 0usize;
            for slot in 0..humans {
                let sp = m.get_spawn_point(m.settings.players[slot].team as u16, slot, &mut sim.drones.rng) as usize;
                let sp_ent = m.spawns.points[sp];
                let b = nf_game::player::Bond::stand_at(&sim.world, [sp_ent.pos[0], sp_ent.pos[1], sp_ent.pos[2]], sp_ent.facing[1]);
                if slot == 0 {
                    first_spawn = sp;
                    sim.bond = b;
                    m.set_slot(0, sim.bond.pos);
                } else {
                    m.set_slot(slot, b.pos);
                    sim.mp_bonds[slot] = Some(b);
                }
            }
            let sp = first_spawn;
            // The bots' half of `MP_Start` (0x183a50:134-160): after every human's create, one
            // `BOT_init` per enabled record in slot order — the game's order is the humans FIRST,
            // then the bots, so the spawn picks' RNG draws keep the game's sequence.
            // mp-lobby-settings: the bots start with the Weapon Set's gun [M BOTWEAP_InitWeapon
            // 0x129e48:44-48].
            m.bot_start = Some(nf_game::mp::equip::bot_start_weapon(&nf_game::mp::equip::weapon_row(&m.settings), &sim.weapons));
            m.create_bots(&mut sim.drones.rng);
            // The bots stand on the floor like the humans' `Bond::stand_at`: the game spawns
            // BOT_init at the raw SpawnPoints entry [M MP_Start 0x183a50:139-160 → BOT_init
            // 0x1249e8 → Drone_Create 0x136be0] and lets its collision gravity settle the drone
            // onto the floor over the first ticks [M Drone_CollisionHandler 0x140968:176-267
            // applies WldGravity]. The port's bot loop runs no settle physics, so `stand_at`
            // pre-settles to the same resting pose in one step (unsettled, the bots
            // floated and fired level over every player).
            for b in m.bots.iter_mut() {
                let feet = nf_game::player::Bond::stand_at(&sim.world, b.brain.pos, b.brain.yaw).pos;
                b.brain.pos = [feet[0], feet[1] + scene::BOT_EYE, feet[2]];
                m.game.slots[b.slot].pos = Some(b.brain.pos);
            }
            eprintln!("mp: bots created: {}", m.bots.len());
            // mp-scenarios-online (agent6): `MP_Start` 0x183a50:166-167 — Assassination picks its
            // first target and assassin (`MP_assassinReset(0)`) once every player and bot exists.
            m.start_assassin(&mut sim.drones.rng);
            // The SP mission machine stays off for the match [M 0x182c60:16-18 — Drones::mp_master].
            sim.drones.mp_master = m.settings.master;
            // mp-world-sync (agent 9): an MP match creates no mounted gun unless the Enviro-Mods
            // word `MPSettings+0x1B8` is set — `GunImp_Create` 0x131f20:52-58 returns 0 for
            // `MPSettings+0x180 != 0 && +0x1B8 == 0` [M]; the boot value is 0 [M bootup 0x1c7e78,
            // L the EE snapshots; menu.rs mp_envmods] and the match settings carry no such word yet,
            // so the arenas' 0x34 placements stay uncreated (a mount was one machine's alone).
            sim.drones.props.gunimps.clear();
            sim.mp = Some(m);
            eprintln!("mp: arena {level_id:08x} — {spawn_count} spawn points registered, {humans} human(s), Bond at spawn {sp} ({:.2}, {:.2}, {:.2}) yaw {:.3}",
                sim.bond.pos[0], sim.bond.pos[1], sim.bond.pos[2], sim.bond.yaw);
        }
        // `Player_CheckWeaponsLoaded`'s model check (0x1a97c8): the hashtable holds the level's loaded
        // skins; the port's equivalent is the level skin table [M — the fp model hash is a skin hash].
        // qworm-not-on-0c: `loadout_skins` (own + game-accurate session pulls) not `sim.skins` —
        // the dev/`--give` merge's foreign skins must not own kit gadgets the game's hashtable misses.
        let loaded: Vec<bool> = (0..0x72u32)
            .map(|w| loadout_skins.contains(&sim.weapons.first_person(w as usize))).collect();
        let loaded = |w: u8| loaded.get(w as usize).copied().unwrap_or(false);
        let req = if sim.mp.is_some() {
            // mp-lobby-settings: an MP match's `Player_InitWeapon` takes its MP arm — `MP_EquipPlayer`
            // and nothing else [M 0x1a7ef8:143-145]; the per-level switch below it (whose default
            // arm handed the arenas the give-everything kit) never runs in MP.
            sim.mp_weapon_loaded = (0..0x72u8).map(loaded).collect();
            // mp-respawn-fists-draw: the equip leaves the fists 0x47 held with the spawn weapon
            // pending [M 0x1855f8:50-53 via 0x1a7ef8:143-145] — the draw runs on the first
            // step's switch machinery. It returns the equipped id for the gate seed below
            // (gun.weapon is 0x47 right after the equip).
            let equipped = sim.mp_equip_local();
            // mp-shot-gate: every human slot's shared fire gate, seeded with the same equip
            // (docs/systems/mp-shot-gate.md) — the rate/ammo/holding check every machine applies
            // to every slot's shot description.
            let humans = (sim.mp.as_ref().unwrap().settings.human_count as usize).clamp(1, nf_net::MAX_AGENTS);
            sim.mp_combat.gate_setup(humans, &sim.mp.as_ref().unwrap().settings, &sim.weapons, loaded);
            equipped
        } else {
            nf_game::player::init_level_loadout(level_id, save, &mut sim.drones.health,
                &mut sim.drones.inventory, &mut sim.drones.channels, &sim.weapons, loaded)
        };
        // The switch request is the game's `M_CONTROL+0x63`; the draw machine holsters and draws it
        // (`Player_WeaponSelect` 0x1a2cc0). 0x47 (none) draws nothing.
        sim.gun.switch_request = req;
        eprintln!("loadout: level {level_id:08x} switch request {req} ({} owned weapons, health {:.0}, armour {:.0})",
            sim.drones.inventory.owned.iter().filter(|&&o| o).count(), sim.drones.health.health, sim.drones.health.armour);
    }
    phase_mark("10 music + starts + loadout", t);
    Ok(scene)
}

/// The level's player starts: the main map's type-0x24 placements in placement order, with the gate
/// (parameter 0) [M `parsemap_create_dynamic_objects` 0x1d04f0 case 0x24 → `Player_AddNewStartPos`
/// 0x195c00 → `Player_Start` 0x190fd0]. Yaw = the placement's rotation[1], the same convention the
/// The story order (`ResetMap_LevelCode2Img` 0x1d5258's case order — docs/systems/levels.md §1: the
/// level codes ascend in mission order), the session the game accumulates before each level.
const STORY_ORDER: [u32; 20] = [0x0700_0001, 0x0700_0002, 0x0700_0003, 0x0700_0004, 0x0700_0005, 0x0700_0006,
    0x0700_0007, 0x0700_0008, 0x0700_0009, 0x0700_000a, 0x0700_000b, 0x0700_000c, 0x0700_000d, 0x0700_0011,
    0x0700_0012, 0x0700_0013, 0x0700_0014, 0x0700_0015, 0x0700_0016, 0x0700_001b];

/// The view-model sleeve meshes (`Scene::build`'s SLEEVES — `AnimSleeveGetEntity(0)`'s per-level set).
const SLEEVES: [u32; 8] = [0x0200_09d0, 0x0200_0900, 0x0200_01f7, 0x0200_0929, 0x0200_09e6, 0x0200_09ff, 0x0200_0a14, 0x0200_0a13];

/// The weapon-model hashes the level archive misses and the session can supply (`merge_weapon_models`).
/// fix-weapon-fx-levels adds two FX sets to the list:
/// - the first-person muzzle-flash mesh `+0x48` (`Player_MuzzleFlash` 0x1a4298 gates on `+0x48 != 0`,
///   `Draw_MuzzleFlash` 0x1a3fe0 attaches it to the view model [M]; `Scene::build` looks it up in
///   `by_hash`, scene.rs) — the disc census [D]: every record's `+0x48` resolves in every story
///   archive EXCEPT 0x02000ac7 (records 0x32/0x33/0x6f), which lives only in 07000014/15/16/1b;
/// - the shell-casing meshes (`ammo_data` entry +4, `Bullet_init_casing` 0x12fde0 attaches
///   `ammo_data[weapon+0x90]+4` [M]; `Casings::eject` reads the parsed table) — ammo 9's default
///   shell 0x02000548 is absent from 07000005-0d/14-16/1b [D census], so the shotgun's casings
///   simulated there were never drawn.
/// The game never hits either gap (a level only offers its own weapons — 0x32/0x33 ride
/// 07000014-16/1b, the shotgun rides 07000011-13/MP, whose archives hold their meshes); the dev
/// give path did, the same way 100a's skins did.
fn wanted_weapon_models(level: &Level, wd: &nf_formats::weapon::WeaponData, casing_meshes: &[u32]) -> Vec<u32> {
    let mut needed: Vec<u32> = Vec::new();
    let mut want = |h: u32, needed: &mut Vec<u32>| {
        if h != 0 && h != u32::MAX && !level.by_hash.contains_key(&h) && !needed.contains(&h) { needed.push(h); }
    };
    for r in &wd.records {
        // fix-silencer-visual: the view-entity mesh (+0x7C, `AnimDatumSetEntity(vm, 0, weapon+0x7C,
        // 0xff)` per tick on the 0x800 records — `Player_WeaponFiring` 0x1a5460:96-110) rides the
        // give/merge path too, or a foreign-level give of a silenced twin misses the tube.
        for off in [0x80usize, 0x4c, 0x48, 0x5c, 0x7c] {
            want(u32::from_le_bytes(r[off..off + 4].try_into().unwrap()), &mut needed);
        }
    }
    for h in SLEEVES { want(h, &mut needed); }
    for &h in casing_meshes { want(h, &mut needed); }
    needed
}

/// The shell-casing meshes (`AmmoData.casing_mesh`, ACTION.ELF `ammo_data`+4 per 0xc-byte record;
/// `Bullet_init_casing` 0x12fde0's mesh argument [M]). Empty when the ELF lacks the symbol.
fn casing_meshes(elf: &[u8]) -> Vec<u32> {
    nf_formats::sfx::AmmoData::from_action_elf(elf).map(|a| a.casing_mesh).unwrap_or_default()
}

/// Clones the foreign maps that hold any of `hashes` into `level` (appended; `by_hash` first-wins, so a
/// hash the level resolves locally keeps its (map, object)). Returns how many maps moved.
fn take_maps_holding(level: &mut Level, foreign: &Level, hashes: &[u32]) -> usize {
    let mut take: std::collections::HashSet<usize> = Default::default();
    for (mi, m) in foreign.maps.iter().enumerate() {
        if m.objects.iter().any(|o| hashes.contains(&o.hashcode)) { take.insert(mi); }
    }
    if take.is_empty() { return 0; }
    let base = level.maps.len();
    let mut moved = 0;
    for (mi, m) in foreign.maps.iter().enumerate() {
        if !take.contains(&mi) { continue; }
        let idx = base + moved;
        for (oi, o) in m.objects.iter().enumerate() {
            level.by_hash.entry(o.hashcode).or_insert((idx, oi));
        }
        // level-load-time: the caller's parsed-archive cache hands shared `&Level`s now, so the
        // taken maps CLONE out (bit-identical data — load_scene_digest pins it) instead of moving.
        level.maps.push(m.clone());
        moved += 1;
    }
    moved
}

/// level-load-time: the parsed-foreign-archive cache behind `merge_weapon_models`. The session mesh
/// merge used to re-parse every earlier story archive on EVERY load (5-7 `Level::parse` per level,
/// ~6.7 s of the 27-level probe total). Entries are immutable after parse; a hit's taken maps clone
/// out bit-identically. Bounded FIFO of 4 entries (the parsed maps can be tens of MB each — keep
/// the resident set small); the scan order (STORY_ORDER descending per load) re-visits the same
/// archives across consecutive levels, so the small window still hits.
fn foreign_level_cache() -> &'static std::sync::Mutex<(std::collections::VecDeque<String>, std::collections::HashMap<String, std::sync::Arc<Level>>)> {
    static C: std::sync::OnceLock<std::sync::Mutex<(std::collections::VecDeque<String>, std::collections::HashMap<String, std::sync::Arc<Level>>)>> = std::sync::OnceLock::new();
    C.get_or_init(|| std::sync::Mutex::new((Default::default(), Default::default())))
}

/// The session object table (`models-weapons-items`, see the caller's comment): every weapon-model mesh
/// (`weapon_data` +0x80 held guns, +0x4C muzzle flashes, +0x5C projectile bodies, and the view-model
/// sleeves) the level archive misses is merged from the latest earlier story archive that holds it.
pub fn merge_weapon_models<R: std::io::Read + std::io::Seek>(iso: &IsoImage<R>,
    read: &impl Fn(&str) -> Result<Option<Vec<u8>>>, prefix: &str, level: &mut Level) -> Result<()> {
    let elf = iso.read_file(iso.find("/ACTION.ELF").context("no ACTION.ELF on the disc")?)?;
    let Ok(wd) = nf_formats::weapon::WeaponData::from_action_elf(&elf) else { return Ok(()) };
    let level_id = u32::from_str_radix(prefix, 16).unwrap_or(0);
    let mut needed = wanted_weapon_models(level, &wd, &casing_meshes(&elf));
    if needed.is_empty() { return Ok(()); }
    for code in STORY_ORDER.iter().rev().filter(|&&c| c < level_id) {
        if needed.is_empty() { break; }
        let name = format!("{code:08x}.bin");
        let foreign: std::sync::Arc<Level> = {
            let mut c = foreign_level_cache().lock().unwrap();
            if let Some(l) = c.1.get(&name) { l.clone() } else {
                drop(c);
                let fd = match read(&name) { Ok(Some(d)) => d, _ => continue };
                let Ok(l) = Level::parse(&fd) else { continue };
                let l = std::sync::Arc::new(l);
                let mut c = foreign_level_cache().lock().unwrap();
                if !c.1.contains_key(&name) {
                    c.0.push_back(name.clone());
                    c.1.insert(name.clone(), l.clone());
                    while c.1.len() > 4 { if let Some(old) = c.0.pop_front() { c.1.remove(&old); } }
                }
                l
            }
        };
        let moved = take_maps_holding(level, &foreign, &needed);
        if moved > 0 {
            needed.retain(|h| !level.by_hash.contains_key(h));
            eprintln!("session mesh merge: {moved} map(s) from {name} ({} weapon-model hashes still missing)", needed.len());
        }
    }
    Ok(())
}

/// The registration + pick: `Player_AddNewStartPos` 0x195c00 → `Player_Start` 0x190fd0. The game
/// registers BOTH start sets from the main map's placements in parse order — type 0x24 as kind 1,
/// type 0x2D as kind 0 (`parsemap_create_dynamic_objects` 0x1d04f0 case 0x24/0x2d [M disasm
/// 0x1d0a00/0x1d0e98]) — and picks the first kind-1 entry whose gate is open, else the first
/// kind-1 [M 0x190fd0; the kind-0 records never win either pass]. The mapping lives in
/// `nf_game::player::starts_from_placements` (the same fn the tests call); yaw = the placement's
/// rotation[1] — the game copies the rotation triple into the start record [M parsemap 0x1d04f0
/// case 0x24] and Player_Init writes its [1] into the object's euler Y (obj+0x54 = *(param_3+4),
/// the parallel spawn record's rotation[1]) [M Player_Init 0x1910f8:76-81]; the MP human/bot
/// spawns ride the same array (param_3 = sp*0x30 + 0x2a7360 [M 0x183a50:120-128]). scene.rs's
/// "Player1" fallback reads the same field.
pub fn level_starts(level: &Level) -> Vec<nf_game::player::LevelStart> {
    let Some(m) = level.main_map() else { return Vec::new() };
    nf_game::player::starts_from_placements(&level.maps[m].placements)
}

/// The level archive for a hashcode, `LoaderLoad` (0x1d4a00) style [M]: `sprintf("%s%8.8x.bin", …,
/// code & 0xff0fffff | …)` — the loader type's nibble (0x00F00000) is masked OFF for the level itself
/// (loader type 1; type 0/3 force 0xF00000 for the anim archives). `ResetMap_LevelToLoad` (0x1d3800)
/// stores e.g. 0x07100006 for a `LOAD_LEVEL` to 0x07000006 (param_3 == 0 remap), and the level load
/// then masks it back to 07000006.bin.
pub fn level_archive_name(code: u32) -> String { format!("{:08x}.bin", code & 0xff0f_ffff) }

/// `NF_FPSLOG` (the port's own measuring aid): accumulates whole-frame times and prints, once a
/// second, the frame count, the average and the worst frame in ms.
/// The on-screen frame counter: presented frames per half-second window, shown as frames/s.
struct FpsMeter { start: Option<std::time::Instant>, frames: u32, shown: f32 }

impl Default for FpsMeter { fn default() -> Self { FpsMeter { start: None, frames: 0, shown: 0.0 } } }

impl FpsMeter {
    fn frame(&mut self) {
        let now = std::time::Instant::now();
        let start = *self.start.get_or_insert(now);
        self.frames += 1;
        let el = (now - start).as_secs_f32();
        if el >= 0.5 { self.shown = self.frames as f32 / el; self.frames = 0; self.start = Some(now); }
    }
}

/// mp-perf-16 (`NF_FPSLOG`): this frame's tick-loop time and its CPU time up to the submit /
/// the present call, folded into the once-a-second `[fps]` line (avg / worst of each).
static PAL_REBUILDS: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
static FRAME_PARTS: std::sync::Mutex<[f64; 12]> = std::sync::Mutex::new([0.0; 12]);
fn frame_part(i: usize, d: std::time::Duration) {
    if let Ok(mut p) = FRAME_PARTS.lock() {
        let ms = d.as_secs_f64() * 1000.0;
        p[i * 2] += ms;
        p[i * 2 + 1] = p[i * 2 + 1].max(ms);
    }
}

fn fps_log(frame: std::time::Duration) {
    use std::sync::Mutex;
    static ACC: Mutex<Option<(std::time::Instant, u32, f64, f64)>> = Mutex::new(None);
    let Ok(mut acc) = ACC.lock() else { return };
    let ms = frame.as_secs_f64() * 1000.0;
    let a = acc.get_or_insert((std::time::Instant::now(), 0, 0.0, 0.0));
    a.1 += 1; a.2 += ms; a.3 = a.3.max(ms);
    if a.0.elapsed().as_secs_f64() >= 1.0 {
        let parts = FRAME_PARTS.lock().map(|mut p| std::mem::take(&mut *p)).unwrap_or_default();
        let n = a.1.max(1) as f64;
        { let __m = format!("[fps] {} frames, avg {:.2} ms, worst {:.2} ms | ticks avg {:.2} worst {:.2} | to submit avg {:.2} worst {:.2} | submit+present avg {:.2} worst {:.2} | palette build avg {:.2} worst {:.2} | blend+upload avg {:.2} worst {:.2} | render encode avg {:.2} worst {:.2} | tick frames {}", a.1, a.2 / a.1 as f64, a.3,
            parts[0] / n, parts[1], parts[2] / n, parts[3], parts[4] / n, parts[5], parts[6] / n, parts[7], parts[8] / n, parts[9], parts[10] / n, parts[11], PAL_REBUILDS.swap(0, std::sync::atomic::Ordering::Relaxed)); std::thread::spawn(move || eprintln!("{__m}")); }
        *acc = None;
    }
}

/// Reads one FILES.BIN entry by name (fix-menus: the front end's USATxt.dat when the map has no Sim).
fn read_files_bin(iso_path: &str, name: &str) -> Option<Vec<u8>> {
    let iso = IsoImage::open(iso_path).ok()?;
    let (entries, bin) = disc::files_bin_entries(&iso).ok()?;
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case(name))?;
    iso.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).ok()
}

/// frontend-music: the credits music's files — `SFXStartMusic(0x27, 0)` [M P_CREDITS 0x2152b0:94]
/// plays MFX track 0x27 through the same MFXINFO.MXI → MFX_<n>.{SMF,SSD} path the level music
/// uses [M docs/systems/music.md; the front end has no MapMusic entry — the loader runs from the
/// command, not from `Sound_Ready`'s table].
fn menu_music_files(iso_path: &str, track: i32) -> Option<(i32, nf_formats::music::Smf, std::sync::Arc<Vec<u8>>)> {
    let iso = IsoImage::open(iso_path).ok()?;
    let get = |p: &str| iso.find(p).and_then(|e| iso.read_file(e).ok());
    let n = nf_formats::music::track_file(&get("PS2/MFXINFO.MXI")?, track).ok()??;
    let smf = nf_formats::music::Smf::parse(&get(&nf_formats::music::music_path(n, "SMF"))?).ok()?;
    Some((track, smf, std::sync::Arc::new(get(&nf_formats::music::music_path(n, "SSD"))?)))
}

/// The keyboard mapping of the game's cut-scene skip action (`Input_Action(-1, 0x19, 1)`, read by
/// `SP_Update` state 2 [M 0x1dbda8:129-141] and GameFlow case 6 [M 0x1c98c8:145-150]). Action 0x19
/// is bound to raw pad mask 0x840 — R1 (0x800) or D-pad DOWN (0x040), pad 0 only [M
/// psiInput_MapInputs 0x10cd80:415-417] — so a movement key (W/A/S/D, Space, the mouse) NEVER skips
/// a scene (a W press must not skip the 07000007 Kimiko scene). The port's
/// two keys are the game's own two bindings: ArrowDown is the mask's D-pad-down half, R the R1
/// half. The viewer's dedicated keys (Esc, F1/F2/F11/F12) keep their own functions;
/// `--skip-cutscenes` is the dev override and bypasses this.
pub fn cutscene_skip_key(k: winit::keyboard::KeyCode) -> bool {
    matches!(k, winit::keyboard::KeyCode::ArrowDown | winit::keyboard::KeyCode::KeyR)
}

/// The dev menu's type-to-filter lane: a physical key → the ASCII char it types into the filter
/// (lowercase; the HUD font has only ASCII glyphs anyway, `dev::fit`).
fn dev_filter_char(k: winit::keyboard::KeyCode) -> Option<char> {
    use winit::keyboard::KeyCode as K;
    Some(match k {
        K::KeyA => 'a', K::KeyB => 'b', K::KeyC => 'c', K::KeyD => 'd', K::KeyE => 'e', K::KeyF => 'f',
        K::KeyG => 'g', K::KeyH => 'h', K::KeyI => 'i', K::KeyJ => 'j', K::KeyK => 'k', K::KeyL => 'l',
        K::KeyM => 'm', K::KeyN => 'n', K::KeyO => 'o', K::KeyP => 'p', K::KeyQ => 'q', K::KeyR => 'r',
        K::KeyS => 's', K::KeyT => 't', K::KeyU => 'u', K::KeyV => 'v', K::KeyW => 'w', K::KeyX => 'x',
        K::KeyY => 'y', K::KeyZ => 'z',
        K::Digit1 => '1', K::Digit2 => '2', K::Digit3 => '3', K::Digit4 => '4', K::Digit5 => '5',
        K::Digit6 => '6', K::Digit7 => '7', K::Digit8 => '8', K::Digit9 => '9', K::Digit0 => '0',
        K::Space => ' ', K::Minus => '-', K::Period => '.', K::Comma => ',',
        _ => return None,
    })
}

/// The Character-layer vertices posed after `time` 60 Hz ticks: start animations (`None`) or sequence `choice`.
/// (`Scene::posed_vertices` also applies the per-character ambient multiplier — the 0x800 TWEAK dim.)
fn posed_vertices(scene: &scene::Scene, choice: Option<usize>, time: f32, opts: scene::ViewOpts) -> Vec<scene::Vertex> {
    scene.posed_vertices(choice, time, opts)
}

/// The destroyed-glist swap's draw side, per frame: the destroyed Destroy leaves' material centres
/// as f32 bits (the draw filter's match key). The game's `Destroy_Smash` 0x12bb60 swaps the dead
/// leaf's object onto the destroyed glist entry — the block-0x27 LOD pointer or the parsemap
/// fallback slot [M 0x12bbb8 asm + 0x1d1b68]; the C5 leaves' fallback slot is undrawable [D
/// lod_word −1] and the doorway ends open [L], so the dead leaf's static materials stop drawing.
fn dead_destroy_centres(scene: &scene::Scene) -> Vec<[u32; 3]> {
    let Some(sim) = scene.sim.as_ref() else { return Vec::new() };
    scene.destroy_centres.iter()
        // The pair carries the leaf's PLACEMENT index, not its slot in `props.destroy`.
        .filter(|(_, pl)| sim.drones.props.destroy.iter().any(|d| d.placement == *pl && d.dead))
        .map(|(c, _)| c.map(f32::to_bits))
        .collect()
}

/// The hit-flash tweak's draw side: the live Destroy leaves whose colour pair is below white
/// (`Renderer::destroy_tint`).
fn destroy_tints(scene: &scene::Scene) -> Vec<([u32; 3], [u8; 2])> {
    let Some(sim) = scene.sim.as_ref() else { return Vec::new() };
    scene.destroy_centres.iter()
        .filter_map(|(c, pl)| sim.drones.props.destroy.iter().find(|d| d.placement == *pl)
            .filter(|d| !d.dead && d.colour != [0xff, 0xff])
            .map(|d| (c.map(f32::to_bits), d.colour)))
        .collect()
}

/// The snow quads facing the camera, plus the emitter particles (fix-world-fx: the same billboard
/// pass — `Emitter_Draw` 0x17b708 path A draws both through `psiDrawParticleList`).
fn snow_for(scene: &scene::Scene, cam: &Camera) -> Vec<scene::Vertex> {
    let Some(sim) = scene.sim.as_ref() else { return Vec::new() };
    let f = cam.dir();
    let right = cam.right();
    let up = right.cross(f).normalize_or_zero();
    let mut out = sim.snow_quads(right.to_array(), up.to_array());
    out.extend(sim.particle_quads(right.to_array(), up.to_array()));
    out
}

/// One frame of sound (`Sound_UpdateListeners` → `Sound_UpdateSounds` → the batch → `SFXUpdate`): the listener is
/// Bond's eye and view (glb_viewer[0]; the cut-scene camera's listener call is dropped by the EE batcher [M]).
///
/// The game's viewer is a 0x240-byte `viewer_tag` built by `build_alloc_viewer` 0x1c9378 (malloc +
/// the +0x28 viewer index) and initialised by `Camera_Reset` 0x1b9af0 (272 B) [M]: zero pos/rot
/// vectors (+0xc0..+0xcc / +0xe0..+0xec from the zero constant at 0x2d6200 [D]), the view MATRIX
/// at +0x120 from zeros, +0x1a0 identity matrix, +0x2c StackInfo (cap 12), the dirty byte +0x2b,
/// +0x108 = 1.0f (the FOV/zoom factor) and the camera radii +0x1ec/+0x1f0 = 4.0f [M
/// 0x1b9af0:13-62]. The port's single `Viewer` struct here + the per-frame camera construction is
/// that init.
/// `pub` so the headless tests drive the exact function the winit loop runs (the fix-gpu-skinning precedent).
pub fn sound_tick(s: &mut nf_audio::sfx::Sound, sim: &mut scene::Sim) {
    let (pos, f) = (sim.bond.head(), sim.bond.look());
    // Row 0 of the view matrix: up × forward, the screen-left axis (snapshot: dir (−0.839, 0, −0.545) → (−0.545, 0, 0.839)).
    let norm = { let l = (f[0] * f[0] + f[2] * f[2]).sqrt().max(1e-6); [f[2] / l, 0.0, -f[0] / l] };
    sim.drones.map_sounds.update(&mut sim.drones.audio, pos);
    // `UpdateMusicalEvents` moved into the sim's tick (it runs every frame with or without an output
    // device [M 0x114e70]); the IOP's now-playing section feeds it through `Drones::iop_section`.
    sim.drones.iop_section = s.music.as_ref().map_or(0, |p| p.now_playing);
    // The EE's SFXPaused mirror (the pause page's op 9 set it on the edge below; the game's
    // UpdateMusicalEvents reads the global every frame [M 0x114e70:61-64]).
    sim.drones.music.sfx_paused = s.driver.paused();
    let mut fade_down = None;
    for c in std::mem::take(&mut sim.drones.music.out) {
        match c {
            nf_game::music::Cmd::Start { track, section } => s.music_pending.push((0, track, section)),
            nf_game::music::Cmd::Jump { section, instant } => s.music_pending.push((1, section, instant as i32)),
            nf_game::music::Cmd::FadeDown { music } => fade_down = Some(music),
        }
    }
    // The IOP's op-0x19 pair each frame (`SFXUpdate` emits it when it changed): `NisPlaying` pauses the
    // `PauseInNis` sounds (`ES_HandleNisPauseSfx`), `NisQuitten` stops the ducker [M sound.md §6 step 3, §8].
    let (nis_playing, nis_quitten) = sim.drones.music.nis_state();
    s.driver.set_nis_playing(nis_playing);
    s.driver.set_nis_quitten(nis_quitten);
    let driver = &s.driver;
    sim.drones.audio.update(sim.sfx.as_ref(), (pos, norm, [0.0, 1.0, 0.0], f), &|h| driver.is_playing(h));
    let cmds = sim.drones.audio.take_commands();
    let (mpfx_log, listener, frame) = (std::env::var_os("NF_MPFX_LOG").is_some(), pos, sim.drones.frame);
    // `Sound_UpdateListeners` 0x1dd278 (fix-room-sound): every frame, from the active viewer's cel —
    // `SFXSetEnvironment(cel s16 +0x98, underwater = (cel+0x3c & 0x10000000) || env >= 20)` [M disasm
    // 0x1dd4b0-0x1dd4fc]; a viewer with no cel sends nothing and the IOP keeps its previous targets.
    // The port's single viewer is Bond's eye (as below); the glb_viewer[4] script-camera override is
    // not modelled — the script-camera listener call is dropped by the EE batcher.
    let env_send = sim.cel_at(pos).map(|i| {
        let c = &sim.cels[i];
        (c.env as i32, (c.underwater || c.env >= 20) as i32)
    });
    s.batch(|drv, out| {
        // `Sound_UpdateListeners`→`SFXSetEnvironment(room env, underwater)` [M sound.md §0, 0x1dd278].
        if let Some((reverb, underwater)) = env_send { drv.set_environment(reverb, underwater); }
        for c in cmds {
            match c {
                nf_game::sound::Cmd::Listener { pos, norm, dir, .. } => {
                    // The pan probe (level-0a-office-hatch-8): the listener the sound is mixed against.
                    if mpfx_log { eprintln!("[snd] f{frame} listener {pos:?} norm {norm:?} dir {dir:?}"); }
                    drv.set_listener(pos, dir, norm);
                }
                nf_game::sound::Cmd::Start { handle, id, pos, radii } => {
                    // mp-remote-fx's log: every driver start with its position (NF_MPFX_LOG).
                    if mpfx_log {
                        let d = pos.map(|p| ((p[0] - listener[0]).powi(2) + (p[1] - listener[1]).powi(2) + (p[2] - listener[2]).powi(2)).sqrt());
                        eprintln!("[snd] f{frame} start {id:#x} handle {handle} at {pos:?} dist {d:?}");
                    }
                    drv.start(out, id, pos, handle, radii.map(|(i, o)| (i as i32, o as i32)));
                }
                nf_game::sound::Cmd::Position { handle, pos } => drv.set_position(handle, None, pos),
                nf_game::sound::Cmd::Remove { handle } => drv.remove(handle, None),
                // `SFXPreCache` (RPC cmd 0x26 [M 0x117118]): the cut-scene line's item armed
                // `frames + 5` ticks early — the stream decodes now, off the play tick
                // [M 0x1c1b38 case 1].
                nf_game::sound::Cmd::PreCache { id, frames } => drv.pre_cache(out, id, frames),
            }
        }
        // `SFXFadeDown(fadeMusic)` (the music end-sting / level-transition fade) [M sound.md §5.5].
        if let Some(music) = fade_down { drv.fade_down(music); }
    });
}

async fn device(instance: &wgpu::Instance, surface: Option<&wgpu::Surface<'_>>) -> Result<(wgpu::Adapter, wgpu::Device, wgpu::Queue)> {
    let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions { compatible_surface: surface, ..Default::default() }).await?;
    // Player bug reports carry the GPU and its driver (bugreport.rs).
    let _ = bugreport::GPU_INFO.set({ let i = adapter.get_info(); format!("{} ({:?}; driver {} {})", i.name, i.backend, i.driver, i.driver_info) });
    // batch-draws: the bindless texture array + per-draw first_instance the batched world pass needs
    // (gpu.rs `Batch`); requested only where the adapter has them — the per-draw path is the fallback.
    let want = wgpu::Features::TEXTURE_BINDING_ARRAY
        | wgpu::Features::SAMPLED_TEXTURE_AND_STORAGE_BUFFER_ARRAY_NON_UNIFORM_INDEXING
        | wgpu::Features::INDIRECT_FIRST_INSTANCE;
    let desc = wgpu::DeviceDescriptor { required_features: adapter.features() & want, required_limits: adapter.limits(), ..Default::default() };
    let (device, queue) = adapter.request_device(&desc).await?;
    Ok((adapter, device, queue))
}

/// The screen-space HUD overlay. The game hides the whole sprite pass while a script camera runs:
/// `Script_CameraStart` (0x1d9140) → `Camera_Enable` (0x1b9920) clears `viewer+0x2a` (the sprite-draw flag)
/// for every gameplay viewer, and `Game_Draw` (0x1e6f50) skips `Sprite_BuildList`/`View_DrawSprites` when it
/// is clear — health iris, ammo, crosshair, radar, messages and the damage glow (`psiFadeView`, in the same
/// gated block) all disappear together. Snapshot-verified: the 07000001 opening cut-scene has viewer[0]/[5]
/// +0x2a = 0 while `ScriptCam`@0x30d100 is non-zero [L ee_start_07000001.bin]. The port's condition is the
/// one it already drives the cut-scene camera from: a live script camera object.
pub fn overlay_for(scene: &scene::Scene, aspect: f32) -> gpu::HudFrame {
    let cutscene = scene.sim.as_ref().is_some_and(|s| s.script_camera().is_some());
    if cutscene { gpu::HudFrame::default() } else { hud::overlay(scene, aspect) }
}

/// The cut-scene frame (fix-cutscene-letterbox): while a script camera runs the port shows the
/// game's own picture — the projection narrowed to the console's 4:3 aspect and the render
/// confined to the band that fits the window (pillar-boxed black on a wider window); the game's
/// letterbox bars ride INSIDE the band (`hud::script_overlay`, `gpu::render`'s viewport). The
/// game rendered every cut-scene at the console's 4:3 frame — the port's widescreen projection
/// showed more at the sides than the game's frame (PCSX2 compare, frame 400:
/// Dominique waiting in the left doorway visible in the port, outside the game's frame).
/// Gameplay keeps the window's full widescreen.
///
/// cutscene-side-strips: the game's cut-scene view builds at the VIEWER's own aspect —
/// `Script_UpdateCamera` 0x1db518:9 multiplies the view by `viewer+0x114` (through
/// `Vision_Calculate_View_Fustrum` 0x1e7df8:79), and `Camera_CalcViewAngles` 0x1baf70:28-32
/// writes 16/9 (0x3fe38e39) there with the AV options' widescreen flag (`DAT_002a37c4`) set,
/// 4/3 (0x3faaaaab) otherwise. So `game_aspect` is that viewer+0x114 value (the port's
/// `FrontEnd::view_aspect` / `Sim::view_aspect` mirror): with the widescreen option ON the
/// cut-scene camera drives the WHOLE frame at 16/9 (same vertical FOV, widened horizontally —
/// the port's own gameplay widescreen treatment); OFF keeps the 4:3 band the option-off game
/// shows, pillar-boxed black on a wider window. Either way the frame is the script camera
/// only — never the gameplay camera. Returns (projection aspect, the band [x, y, w, h] in
/// target pixels).
pub fn cutscene_frame(win_w: f32, win_h: f32, game_aspect: f32) -> (f32, [f32; 4]) {
    let game = if game_aspect > 0.0 { game_aspect } else { 4.0 / 3.0 };
    let (bw, bh) = if win_w / win_h > game { (win_h * game, win_h) } else { (win_w, win_w / game) };
    (game, [(win_w - bw) * 0.5, (win_h - bh) * 0.5, bw, bh])
}

/// The HUD frame the headless `--shot` render draws (fix-cutscene-letterbox): the gameplay
/// overlay (`overlay_for` — empty while a script camera runs) PLUS the script player's overlay —
/// the fade, the letterbox bars and the op-0x14 sprites (`Scene::script_overlay_hud`). The shot
/// path used to draw `overlay_for` alone, so the Kimiko-scene compare frames showed no letterbox
/// bars; the window path always appended the script overlay (App redraw) and now both do.
/// mp-screens: the spot shot's movie gates, mirrored from the redraw's `movie_up` decision
/// (the window's per-frame composition). Inputs: the front-end's movie states (state-6 briefing,
/// the intro pair, the attract movie, whether the ONE player holds a looping page backdrop, the
/// briefing's `movie_pending`, a player at all). Returns `(want_frame, blank_rule)`:
/// `blank_rule` decides the overlay — `Blank::Always` (the state-6 briefing blits over the empty
/// backdrop whether or not a frame has landed yet), `Blank::WithFrame` (the intro/attract FMV
/// owns the screen once a frame exists), `Blank::Never` (the looping page backdrop draws UNDER
/// the overlay — the pages' controls paint over their backdrop [M movies.md §Presentation]) or
/// no movie at all. The frame is pulled only when a movie is up (an extra `present()` would step
/// the page loop). Pure so the tests pin the window parity without a GPU.
pub fn spot_movie_gates(state6: bool, intro_up: bool, attract_up: bool, looping: bool, movie_pending: bool, has_movie: bool) -> (bool, Blank) {
    let flow_up = state6 && !intro_up && has_movie;
    if flow_up { return (true, Blank::Always); }
    if intro_up || attract_up { return (true, Blank::WithFrame); }
    if looping && !movie_pending { return (true, Blank::Never); }
    (false, Blank::Never)
}

/// The overlay rule a movie-up shot uses (see `spot_movie_gates`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Blank { Always, WithFrame, Never }

/// Whether the scene's script overlay (the view fade / letterbox / flash — the 3D view's) draws
/// this frame. Not while a movie frame owns the screen (the game's movie loop draws nothing of the
/// scene over it), and not on the front-end level 0x07000048: `ResetMap_Load` runs `Player_Start`
/// → `Player_Init` there too [M 0x1d6360:154], arming the 45-tick entry fade [M 0x1910f8:45], but
/// the game's front end ticks it away under the menus while the port's front-end scene never
/// ticks — the fade sat at full black OVER the title and every backdrop-less page, and (once
/// doublecross-transitions chained it into this overlay) over every boot/briefing movie (the
/// audio played with no picture).
pub fn scene_overlay_up(scene: &scene::Scene, movie_up: bool) -> bool {
    !movie_up && scene.sim.as_ref().map_or(true, |s| s.level & 0xff00_ffff != menu::FRONTEND_LEVEL)
}

/// `movie_up`: a movie frame is on screen (see `scene_overlay_up`).
pub fn shot_hud(scene: &scene::Scene, aspect: f32, menu: Option<&gpu::HudFrame>, movie_up: bool) -> gpu::HudFrame {
    let mut f = menu.map_or_else(|| overlay_for(scene, aspect), |m| m.clone());
    if scene_overlay_up(scene, movie_up) { f.append(&scene.script_overlay_hud(aspect)); }
    f
}

/// The camera's screen axes for the corona billboards (same basis as the snow quads).
fn cam_axes(cam: &Camera) -> ([f32; 3], [f32; 3]) {
    let f = cam.dir();
    let right = cam.right();
    let up = right.cross(f).normalize_or_zero();
    (right.to_array(), up.to_array())
}

/// fix-menus2: the front-end overlay for `--shot`. On the front-end boot the menu overlay replaces the
/// gameplay HUD (the task: the shot path must draw the menus). `--page <start|main|codename|
/// difficulty|missions|dossier>` jumps to a page for the per-page renders, `--cursor N` picks the row,
/// `--unlock-all` is the `menu_unlock_everything` dev global [M 0x30d2a7, 0x20b8d8:18]. The page is
/// drawn with the game's font (ACTION.ELF's FONTSerpLight) and the USATxt.dat strings.
/// fix-menu-movies2: returns `(hud, movie)` — with the `movies` feature and an image that has movie
/// data, the page's backdrop is its own looping FMV instead of the dark plate, decoded as a still
/// (closed-GOP chunks in order until the frame is covered — fast; `--movie-frame N` picks the
/// frame) and drawn on the ONE
/// movie plane (gpu.rs), exactly like the live loop. `--intro [N]` renders the P_INTRO movie the
/// same way, fullscreen with no overlay (the intro page has no controls [M 0x209e88]).
fn front_end_menu(args: &[String], scene: &scene::Scene, front_end: bool, iso_path: &str) -> (Option<gpu::HudFrame>, Option<(u32, u32, Vec<u8>)>) {
    let page_arg = args.iter().position(|a| a == "--page");
    if !front_end && page_arg.is_none() { return (None, None); }
    let Some(tables) = scene.hud.as_ref() else { eprintln!("[front end] no HUD tables (the game font) — the menu overlay needs ACTION.ELF"); return (None, None); };
    let mut fe = menu::FrontEnd::new();
    if args.iter().any(|a| a == "--unlock-all") { fe.unlock_all = true; }
    // Render aid: NF_VIEW_H=<px> renders the page as a window that tall would (the online lobby's
    // constant-size text: more rows on a tall window, a scroll bar on a short one).
    if let Some(v) = std::env::var("NF_VIEW_H").ok().and_then(|v| v.parse::<f32>().ok()) { fe.view_h = v; }
    if let Some(i) = args.iter().position(|a| a == "--cursor") {
        fe.cursor = args.get(i + 1).and_then(|v| v.parse().ok()).unwrap_or(0);
    }
    if let Some(i) = page_arg {
        let name = args.get(i + 1).map(String::as_str).unwrap_or("start");
        fe.page = match name {
            "start" | "title" => menu::Page::Start,
            "main" => menu::Page::Main,
            "codename" => menu::Page::Codename,
            "difficulty" => menu::Page::Difficulty,
            "missions" | "single" => menu::Page::MissionSelect,
            "dossier" => menu::Page::Dossier,
            // fix-menus-real (additive hook): the dossier's Records table renders headless too.
            "records" => menu::Page::Records,
            // dossier-pages-data (additive hooks): the Dossier wheel's rows 1-3 render headless too.
            "rewards" => menu::Page::Rewards,
            "gadgets" => menu::Page::Gadgets,
            "weapons" => menu::Page::Weapons,
            "cnselect" => menu::Page::CnSelect,
            // Render aid: the 007 Bonus page with every tier of 07000005's reward earned (the
            // fields bonus_enter builds, all lines standing).
            "nfbonus" => {
                let i = nf_game::progress::REWARDS_TABLE.iter().position(|e| e.hash == 0x0700_0005).unwrap_or(0);
                let lines = [0usize, 1, 2, 3].map(|t| { let (label, texture) = nf_game::progress::REWARD_TEXT[i][t]; menu::RewardLine { has: true, label, texture } });
                let text_labels = [0usize, 3, 2, 1].iter().map(|&t| lines[t].label).collect();
                fe.bonus = Some(menu::BonusFields { level: 0x0700_0005, lines, texture: Some(lines[0].texture), text_labels, has_text: true });
                menu::Page::NfBonus
            }
            "cnmenu" => { fe.cn_name = "Default".into(); menu::Page::CnMenu }
            "advanced" => menu::Page::CnAdvanced,
            "mpoptions" => menu::Page::CnMp,
            "avoptions" => menu::Page::CnAv,
            "controls" => menu::Page::CnControls,
            // codename-entry: the name-entry grid renders headless too (cn_open seeds the proposal).
            "cnname" => { fe.cn_open(menu::PAGE_CN_SELECT); menu::Page::CnName }
            "cncheat" => { fe.cn_open(menu::PAGE_CN_MENU); menu::Page::CnName }
            "mpscenario" => menu::Page::MpScenario,
            "mpmap" => menu::Page::MpMap,
            "mpjoin" => menu::Page::MpJoin,
            "mpscenopts" => menu::Page::MpOptions,
            "mpbots" => menu::Page::MpBots,
            "mpbotchoose" => menu::Page::MpBotChoose,
            "mprules" => menu::Page::MpRules,
            "mpplayermods" => menu::Page::MpPlayerMods,
            "mpenviromods" => menu::Page::MpEnviroMods,
            "mpbotsetup" => menu::Page::MpBotSetup,
            // The setup step / match summary (agent 1 joined; NF_SETUP_STATE=2|3|4 picks team/character/handicap,
            // NF_CONFIRM_TEAM=1 a Team Arena with 3 bots split across the teams).
            "mpsetup" | "mpconfirm" => {
                fe.mp_join[0].joined = true; fe.mp_join[0].state = 5; fe.mp_join[0].skin = 0;
                fe.mp_level = 0x0700_0024;
                if std::env::var_os("NF_CONFIRM_TEAM").is_some() {
                    fe.mp_scenario = menu::MP_SCENARIO_ROWS.iter().map(|r| r.3).find(|s| s & 0x2000_0000 != 0).unwrap_or(0x2000_0001);
                    fe.mp_join[0].team = 1;
                    for (k, b) in fe.mp_bots.iter_mut().take(3).enumerate() { b.enabled = true; b.team_mi6 = k == 0; }
                } else { fe.mp_scenario = 1; }
                fe.mp_setup_state = std::env::var("NF_SETUP_STATE").ok().and_then(|v| v.parse().ok()).unwrap_or(2);
                if name == "mpsetup" { menu::Page::MpSetup } else { menu::Page::MpConfirm }
            }
            // The online one screen as the host sees it (a joined player in slot 2 for the render).
            "mplobby" => {
                fe.mp_hosting = true;
                fe.mp_one.players[0].joined = true; fe.mp_one.players[0].ready = true; fe.mp_one.players[0].codename = "Bond".into();
                fe.mp_one.players[1].joined = true; fe.mp_one.players[1].codename = "Agent 2".into(); fe.mp_one.players[1].ping_ms = 23;
                // NF_LOBBY_PLAYERS=n: n joined players for the big-lobby render.
                if let Some(n) = std::env::var("NF_LOBBY_PLAYERS").ok().and_then(|v| v.parse::<usize>().ok()) {
                    for i in 2..n.min(fe.mp_one.players.len()) {
                        let p = &mut fe.mp_one.players[i];
                        p.joined = true; p.ready = i % 3 != 0; p.codename = format!("Agent {}", i + 1); p.ping_ms = 20 + i as u32 * 7;
                    }
                }
                menu::Page::MpOnlineLobby
            }
            // fix-pause-menu: the in-game pages render headless too (the pause page over the
            // paused frame, the results per outcome). `--cursor N` picks the row, `--pause-tab N`
            // picks the pause tab, `--confirm` arms the pause confirm.
            "pause" => menu::Page::Pause,
            "results" | "failed" => { fe.outcome = menu::Outcome::Failed; menu::Page::Results }
            "complete" => { fe.outcome = menu::Outcome::Complete; menu::Page::Results }
            // Render aid (mp-hud-real): the debriefing of a 1-human + 3-bot draw — the PCSX2
            // comparison's match (codename "Agent"; bots Snow Guard / Black Ops / Yakuza =
            // characters 6/7/8, 0 kills, 7 deaths each). NF_DEBRIEF_N=n: n participants.
            "mpdebrief" => {
                let n = std::env::var("NF_DEBRIEF_N").ok().and_then(|v| v.parse::<u32>().ok()).unwrap_or(4).clamp(1, nf_game::mp::MP_MAX_SLOTS as u32);
                let mut st = nf_game::mp::MpSettings::deathmatch(0x0700_0023);
                st.human_count = 1;
                st.bot_count = n - 1;
                st.total_players = n;
                st.players[0].codename = "Agent".into();
                for k in 1..n as usize { st.players[k].skin = [6, 7, 8, 9][(k - 1) % 4]; }
                let mut m = nf_game::mp::MpMatch::init(st);
                m.start(&mut nf_game::drone::Rng::new(1));
                for k in 1..n as usize { m.game.slots[k].deaths = 7; }
                fe.open_mp_debriefing(&m);
                menu::Page::MpDebriefing
            }
            other => { eprintln!("unknown --page '{other}' (start|main|codename|difficulty|missions|dossier|pause|results|complete)"); return (None, None); }
        };
    }
    // The MP wheels keep their own row counters: `--cursor N` picks the row there too.
    if fe.page == menu::Page::MpScenario { fe.mp_scenario_row = fe.cursor; }
    if fe.page == menu::Page::MpMap { fe.mp_map_row = fe.cursor; }
    if fe.page == menu::Page::MpOptions { fe.mp_options_row = fe.cursor; }
    if fe.page == menu::Page::MpBots { fe.mp_bots_row = fe.cursor; }
    if fe.page == menu::Page::MpBotChoose { fe.mp_bot_char = fe.cursor; }
    // Join Game: `--cursor` is agent 1's panel state (0 press to join, 1 choose codename, 5 joined).
    if fe.page == menu::Page::MpJoin { fe.mp_join[0].state = fe.cursor as u32; fe.mp_join[0].joined = fe.cursor > 0; fe.cursor = 0; }
    if let Some(i) = args.iter().position(|a| a == "--pause-tab") {
        fe.pause_tab = args.get(i + 1).and_then(|v| v.parse().ok()).unwrap_or(0);
    }
    if args.iter().any(|a| a == "--confirm") {
        fe.confirming_quit = fe.cursor == 2;
        fe.confirming_restart = fe.cursor == 1;
        fe.confirming_default_profile = fe.page == menu::Page::Codename;
    }
    // The pause page is a MISSION page: give the flow a mission level so open_pause's guard matches
    // (the game opens it over a running mission [M Player_Update 0x193600:73]).
    if fe.page == menu::Page::Pause && fe.flow.current_level == 0 {
        fe.flow.current_level = 0x0700_0005;
        fe.mission = 1;
        fe.open_pause();
    }
    if fe.page == menu::Page::Pause {
        if let Some(sim) = scene.sim.as_ref() { fe.pause_objectives = pause_objective_rows(sim); fe.stats = pause_stats(sim); }
        // objective-hints: pre-open the ✕ hint box for the render (the row the cursor stands on) —
        // the same state `FrontEnd::action`'s Confirm sets on the OBJECTIVES tab [M 0x204408
        // 0x205b98-0x205c34: control 1's activation reads control 8's row and opens the ok-only
        // box; 0xffffffff → 0x31d]. After the fill above (the rows must exist).
        if args.iter().any(|a| a == "--pause-hint") && fe.pause_tab == 1 && !fe.pause_objectives.is_empty() {
            let sel = fe.pause_objectives_sel.min(fe.pause_objectives.len() - 1);
            let hint = fe.pause_objectives[sel].hint;
            fe.pause_hint = Some(if hint == 0xffff_ffff { 0x31d } else { hint });
        }
    }
    // Tick through the boot load and let the page's timers run (the flash cycle etc.).
    for _ in 0..3 { fe.flow.main_tick(); let _ = fe.flow.load_tick(); }
    fe.press_start_on = true;
    let lang = read_files_bin(&scene_iso_name(), "USATxt.dat").and_then(|b| nf_formats::hud::Lang::parse(&b).ok())
        .or_else(|| scene.sim.as_ref().and_then(|s| s.lang.clone()));
    let intro = args.iter().any(|a| a == "--intro");
    #[allow(unused_mut)] // the movies feature assigns it
    let mut movie = None;
    #[cfg(feature = "movies")]
    {
        // --movie-frame defaults past the first GOP: frame 0 of the aux FMVs is the fade-in from
        // black (fix-menus-real's live check; the movie reports pin decodes on frame 12), so a bare
        // --page main --shot must not show it.
        let frame_n: usize = args.iter().position(|a| a == "--movie-frame").and_then(|i| args.get(i + 1)).and_then(|v| v.parse().ok()).unwrap_or(12);
        let want = if intro {
            Some(menu::INTRO_MOVIES[args.iter().position(|a| a == "--intro").and_then(|i| args.get(i + 1)).and_then(|v| v.parse::<usize>().ok()).unwrap_or(0).min(menu::INTRO_MOVIES.len() - 1)])
        } else {
            let aux = menu::aux_movie_for(fe.page_id());
            (aux != 0).then_some(aux)
        };
        if let Some(code) = want {
            // The image the movies are on: NF_MOVIE_ISO over the ISO argument (the live loop's
            // App::movie_iso_path precedence) — not the bare NF_ISO env, which may point at the
            // slim on-foot image whose PSS files are 0-byte stubs [L probe_iso_movies]. (Both
            // sides of the fix-menus-real2 merge landed this precedence; movie_iso_name keeps the
            // ISO argument as the no-NF_MOVIE_ISO fallback, which the env-only form dropped.)
            match movie::movie_still(&movie_iso_name(iso_path), code, frame_n) {
                Ok(img) => movie = Some((img.width, img.height, img.rgba)),
                Err(e) => eprintln!("[front end] movie {code:08X}: {e} — the dark plate stands in"),
            }
        }
    }
    if intro {
        // The intro FMV owns the screen — no controls, no plate [M 0x209e88]; the movie plane is
        // the whole picture. Without movie data the headless plate stands in.
        let hud = if movie.is_some() { None }
            else { Some(menu::menu_overlay(&fe, &tables.font, &tables.font_header, lang.as_ref(), &scene.hud_textures, 1280.0 / 720.0)) };
        return (hud, movie);
    }
    fe.backdrop_live = movie.is_some();
    (Some(menu::menu_overlay(&fe, &tables.font, &tables.font_header, lang.as_ref(), &scene.hud_textures, 1280.0 / 720.0)), movie)
}

/// The ISO path for `read_files_bin` (the NF_ISO env or the default image name).
fn scene_iso_name() -> String {
    std::env::var("NF_ISO").unwrap_or_else(|_| "Nightfire-onfoot.iso".to_string())
}

/// The image the headless movie stills read: NF_MOVIE_ISO (a full retail image) over the ISO
/// argument the binary was given — the live loop's `App::movie_iso_path` precedence. The old
/// still path read the bare `NF_ISO` env instead of the ISO argument, so
/// `nf-view <retail.iso> 07000048 --page main` reported the slim image's 0-byte PSS stubs and
/// never saw the retail bytes [fix-menu-backdrop].
pub fn movie_iso_name(iso_path: &str) -> String {
    std::env::var("NF_MOVIE_ISO").unwrap_or_else(|_| iso_path.to_string())
}

/// Renders one frame offscreen and writes it as a PNG. `menu` = a front-end overlay frame (fix-menus2:
/// `--shot` on the front-end boot draws the menu pages too, for comparison with PCSX2).
/// The `--mp-look slot[,dist,height,angle]` camera: `dist` (2.5) from that slot's body, `angle` (0 = in
/// front) around its yaw, `height` (1.2) above its feet, looking at its chest.
fn mp_look_camera(scene: &scene::Scene, v: &[f32]) -> Option<Camera> {
    let slot = v.first().copied().unwrap_or(1.0) as usize;
    let (dist, height, angle) = (v.get(1).copied().unwrap_or(2.5), v.get(2).copied().unwrap_or(1.2), v.get(3).copied().unwrap_or(0.0));
    let s = scene.sim.as_ref()?;
    let st = s.mp_body_state(slot)?;
    let (p, yaw) = (st.0, st.1 + angle);
    let feet_y = p[1] - s.mp_body_cc(slot);
    let eye = [p[0] + yaw.sin() * dist, feet_y + height, p[2] + yaw.cos() * dist];
    let target = [p[0], feet_y + 1.0, p[2]];
    let d = [target[0] - eye[0], target[1] - eye[1], target[2] - eye[2]];
    Some(Camera { pos: Vec3::new(eye[0], eye[1], eye[2]), yaw: d[0].atan2(d[2]), pitch: (d[1] / (d[0] * d[0] + d[2] * d[2]).sqrt()).atan(), fov: GAME_FOV })
}

/// `--mp-drive` (see the `--shot` path): scripted slot-1 inputs through the online tick, captures at
/// the `--mp-shots` ticks with the `--mp-look` camera (fixed to the body's yaw at the first capture
/// unless `--mp-look-follow`).
fn mp_drive_shots(scene: &mut scene::Scene, args: &[String], spec: &str, out: &str, ticks: u32, opts: scene::ViewOpts) -> Result<()> {
    struct Seg { a: u32, b: u32, f: f32, s: f32, t: f32, l: f32, c: bool, j: bool, k: bool }
    let mut segs = Vec::new();
    for part in spec.split(';').map(str::trim).filter(|p| !p.is_empty()) {
        let (range, keys) = part.split_once(':').unwrap_or((part, ""));
        let (a, b) = range.split_once('-').context("--mp-drive: a segment needs a-b")?;
        let mut sg = Seg { a: a.trim().parse()?, b: b.trim().parse()?, f: 0.0, s: 0.0, t: 0.0, l: 0.0, c: false, j: false, k: false };
        for kv in keys.split(',').map(str::trim).filter(|k| !k.is_empty()) {
            let (k, v) = kv.split_once('=').unwrap_or((kv, "1"));
            let v: f32 = v.trim().parse()?;
            match k.trim() { "f" => sg.f = v, "s" => sg.s = v, "t" => sg.t = v, "l" => sg.l = v, "c" => sg.c = v != 0.0, "j" => sg.j = v != 0.0, "k" => sg.k = v != 0.0, o => anyhow::bail!("--mp-drive: unknown key {o:?}") }
        }
        segs.push(sg);
    }
    let shots: Vec<u32> = match args.iter().position(|a| a == "--mp-shots") {
        Some(i) => args.get(i + 1).context("--mp-shots needs ticks")?.split(',').map(|t| t.trim().parse()).collect::<Result<_, _>>()?,
        None => vec![ticks],
    };
    let look: Vec<f32> = match args.iter().position(|a| a == "--mp-look") {
        Some(i) => args.get(i + 1).context("--mp-look needs slot[,dist,height,angle]")?.split(',').map(|t| t.trim().parse()).collect::<Result<_, _>>()?,
        None => vec![1.0],
    };
    let follow = args.iter().any(|a| a == "--mp-look-follow");
    let stem = out.trim_end_matches(".png");
    if let Some(sim) = scene.sim.as_mut() { net_slot0_setup(sim); }
    let mut fixed: Option<Camera> = None;
    for t in 1..=ticks {
        let mut row = [nf_net::input::NEUTRAL; nf_net::MAX_AGENTS];
        let held = scene.sim.as_ref().map_or(2, |s| s.gun.weapon);
        row[0].held = held;
        row[1].held = 2;
        for sg in segs.iter().filter(|sg| (sg.a..=sg.b).contains(&t)) {
            row[1].forward = sg.f;
            row[1].strafe = sg.s;
            row[1].turn = sg.t;
            row[1].look = sg.l;
            row[1].crouch = sg.c;
            row[1].jump = sg.j;
            if sg.k && t == sg.a {
                if let Some(s) = scene.sim.as_mut() {
                    s.mp_hp[1] = 0.0;
                    s.mp_dead_ticks[1] = 1;
                    if let Some(m) = s.mp.as_mut() { m.player_killed(1, Some(0), false, false, &mut s.drones.rng); }
                }
            }
        }
        net_slot0_tick(scene.sim.as_mut().context("no sim")?, &row);
        if shots.contains(&t) {
            let cam = if follow { mp_look_camera(scene, &look) } else { fixed.or_else(|| mp_look_camera(scene, &look)) };
            let Some(cam) = cam else { eprintln!("--mp-shots: tick {t}: slot has no drawn body"); continue };
            if !follow && fixed.is_none() { fixed = Some(cam); }
            // A fixed camera keeps its spot but re-aims at the body, which may have walked.
            let cam = if follow { cam } else {
                let s = scene.sim.as_ref().context("no sim")?;
                match s.mp_body_state(look.first().copied().unwrap_or(1.0) as usize) {
                    Some((p, _, _)) => {
                        let slot = look.first().copied().unwrap_or(1.0) as usize;
                        let target = [p[0], p[1] - s.mp_body_cc(slot) + 1.0, p[2]];
                        let d = [target[0] - cam.pos.x, target[1] - cam.pos.y, target[2] - cam.pos.z];
                        Camera { pos: cam.pos, yaw: d[0].atan2(d[2]), pitch: (d[1] / (d[0] * d[0] + d[2] * d[2]).sqrt()).atan(), fov: GAME_FOV }
                    }
                    None => cam,
                }
            };
            if let Some(s) = scene.sim.as_mut() { s.first_person = false; }
            if let Some(s) = scene.sim.as_ref() {
                let a = &s.mp_anim[1];
                eprintln!("--mp-shots t{t}: slot 1 layers {:?} phase {:.3} aim {:?} crouch {} dead {} root y {:.3} cc {:.3}",
                    a.layers.iter().map(|l| (l.kind, (l.time * 10.0).round() / 10.0, (l.weight() * 100.0).round() / 100.0)).collect::<Vec<_>>(), a.phase, a.aim, a.crouch, a.dead,
                    s.mp_body_state(1).map_or(0.0, |b| b.0[1]), s.mp_body_cc(1));
            }
            let path = format!("{stem}_t{t:04}.png");
            shot(scene, &cam, &path, None, t as f32, opts, false, false, None, None)?;
        }
    }
    Ok(())
}

fn shot(scene: &scene::Scene, cam: &Camera, out: &str, choice: Option<usize>, time: f32, mut opts: scene::ViewOpts, notex: bool, nofog: bool, menu: Option<&gpu::HudFrame>, movie: Option<&(u32, u32, Vec<u8>)>) -> Result<()> {
    let (w, h) = (1280u32, 720u32);
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
    let (_adapter, device, queue) = pollster::block_on(device(&instance, None))?;
    let format = wgpu::TextureFormat::Rgba8Unorm;
    // fix-sentinel-camera-play: the headless renders show the ride too — a `--frame N` shot taken
    // during the guided ride draws the rocket's forward view [M `Player_PositionCamera` 0x18f5d8
    // case 0xc], not Bond's.
    let cam = &scene.sim.as_ref().and_then(|s| s.view_override()).map(|(p, d)| Camera {
        pos: Vec3::new(p[0], p[1], p[2]), yaw: d[0].atan2(d[2]), pitch: d[1].clamp(-1.0, 1.0).asin(), fov: GAME_FOV,
    }).unwrap_or(*cam);
    let (opts_right, opts_up) = cam_axes(cam);
    opts.right = opts_right;
    opts.up = opts_up;
    let mut r = gpu::Renderer::new(&device, &queue, format, scene, notex, nofog);
    // fix-cutscene-letterbox: a live script camera frames the shot at the game's own 4:3 aspect
    // and pillar-boxes the render into the band; gameplay keeps the window's widescreen.
    let cut = scene.sim.as_ref().is_some_and(|s| s.script_camera().is_some());
    // cutscene-side-strips: the shot frames at the viewer's own aspect (the sim's view_aspect;
    // the window path feeds it from the front end's AV widescreen row) — see cutscene_frame.
    let game_aspect = scene.sim.as_ref().map_or(4.0 / 3.0, |s| s.view_aspect);
    let (proj_aspect, band) = if cut { cutscene_frame(w as f32, h as f32, game_aspect) } else { (w as f32 / h as f32, [0.0; 4]) };
    let (vp_shot, _) = cam.view_proj(proj_aspect);
    // The doors' `Door_IsClosed` snapshot for the walk's portal gate (fix-door-portal-links).
    let door_closed = scene.sim.as_ref().map(|s| s.drones.doors.closed_flags());
    // Render aid (NF_NO_PORTAL_CULL): draw every cel — the dev panel's portal-culling OFF — for shots from a `--bond`
    // spot the eye's cel walk doesn't resolve.
    let world = if std::env::var_os("NF_NO_PORTAL_CULL").is_some() { None } else { scene.sim.as_ref().map(|s| &s.world) };
    let vis_shot = r.cel_visibility(cam.pos.to_array(), vp_shot, world, door_closed.as_deref());
    // The destroyed-glist swap's draw side: the dead Destroy leaves' centres (see Scene::destroy_centres).
    r.destroy_dead = dead_destroy_centres(scene);
    r.destroy_tint = destroy_tints(scene);
    // Objects are never room-culled (see the live path).
    r.update_skin(&queue, scene, choice, time, opts, None);
    r.set_char_light_centres(scene);
    r.lights = scene.frame_lights();
    r.screen_aspect = w as f32 / h as f32;
    // `NF_BENCH=N`: time the per-frame CPU side N times (posing + vertex build, and the upload) for profiling.
    if let Some(n) = std::env::var("NF_BENCH").ok().and_then(|v| v.parse::<u32>().ok()) {
        let t0 = std::time::Instant::now();
        for _ in 0..n { let _ = scene.animate(choice, time, opts); }
        let t1 = std::time::Instant::now();
        let mut v = Vec::new();
        for _ in 0..n { v = posed_vertices(scene, choice, time, opts); }
        let t2 = std::time::Instant::now();
        let _ = &v;
        queue.submit([]); let _ = device.poll(wgpu::PollType::wait_indefinitely());
        let t3 = std::time::Instant::now();
        let ms = |a: std::time::Instant, b: std::time::Instant| (b - a).as_secs_f64() * 1000.0 / n as f64;
        eprintln!("[bench] {} char vertices: animate {:.2} ms, posed_vertices (incl. animate) {:.2} ms, upload {:.2} ms", v.len(), ms(t0, t1), ms(t1, t2), ms(t2, t3));
        let t4 = std::time::Instant::now();
        for _ in 0..n { r.update_skin(&queue, scene, choice, time, opts, None); }
        queue.submit([]); let _ = device.poll(wgpu::PollType::wait_indefinitely());
        eprintln!("[bench] GPU path: palette build + upload {:.2} ms", ms(t4, std::time::Instant::now()));
    }
    let target = device.create_texture(&wgpu::TextureDescriptor {
        label: None, size: wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 }, mip_level_count: 1, sample_count: 1,
        dimension: wgpu::TextureDimension::D2, format, usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC, view_formats: &[],
    });
    let view = target.create_view(&Default::default());
    let depth = gpu::depth_view(&device, w, h);
    let (vp, vp_rot) = cam.view_proj(proj_aspect);
    let snow = snow_for(scene, cam);
    let hud_owned = shot_hud(scene, proj_aspect, menu, movie.is_some());
    let hud = &hud_owned;
    let movie_arg = movie.map(|(w, h, rgba)| (*w, *h, rgba.as_slice()));
    r.set_shards(&queue, &scene.shard_draws());
    let mut enc = r.render(&device, &queue, &view, &depth, vp, vp_rot, cam.pos.to_array(), time as u32, &snow, hud, movie_arg, if cut { Some(band) } else { None });
    let row = (4 * w).next_multiple_of(256);
    let buf = device.create_buffer(&wgpu::BufferDescriptor { label: None, size: (row * h) as u64, usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ, mapped_at_creation: false });
    enc.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo { texture: &target, mip_level: 0, origin: wgpu::Origin3d::ZERO, aspect: wgpu::TextureAspect::All },
        wgpu::TexelCopyBufferInfo { buffer: &buf, layout: wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(row), rows_per_image: Some(h) } },
        wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 });
    queue.submit([enc.finish()]);
    buf.map_async(wgpu::MapMode::Read, .., |r| r.expect("map screenshot buffer"));
    device.poll(wgpu::PollType::wait_indefinitely())?;
    let data = buf.get_mapped_range(..)?;
    let mut rgba = Vec::with_capacity((4 * w * h) as usize);
    for y in 0..h as usize { rgba.extend_from_slice(&data[y * row as usize..][..4 * w as usize]); }
    std::fs::write(out, nf_formats::png::encode_rgba(w, h, &rgba))?;
    eprintln!("wrote {out}");
    Ok(())
}

struct Gfx {
    window: Arc<Window>,
    instance: wgpu::Instance,
    /// Minimised (0×0): nothing is configured or drawn until the window has a size again.
    minimized: bool,
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    depth: wgpu::TextureView,
    renderer: gpu::Renderer,
    /// The adapter (the dev window configures its own surface against it).
    adapter: wgpu::Adapter,
}

struct App {
    /// The sound driver and output (none without a device or bank).
    sound: Option<nf_audio::sfx::Sound>,
    scene: scene::Scene,
    cam: Camera,
    gfx: Option<Gfx>,
    keys: HashSet<KeyCode>,
    grabbed: bool,
    /// Fire button held (left mouse while grabbed) and aim mode (a pad's L1; never set by the mouse, D-002).
    trigger: bool,
    aiming: bool,
    last: std::time::Instant,
    /// Animation: `None` = the game's start animations; `Some(k)` = the k-th sequence per skeleton (`[` / `]`);
    /// time in 60 Hz ticks; paused (`P`).
    choice: Option<usize>,
    anim_time: f32,
    paused: bool,
    /// F3 = show held-back drones (reinforcements, drones waiting for a switch), F = make armed drones fire.
    opts: scene::ViewOpts,
    /// N: the free camera instead of Bond. Keys pressed since the last game tick (jump, crouch), and the mouse turn
    /// (radians) and look (units of π/2) not yet given to a tick.
    noclip: bool,
    just: HashSet<KeyCode>,
    turn: f32,
    look: f32,
    /// Pending weapon/gadget cycle: ±1 for next/prev (mouse wheel = weapons, G = gadgets), or `Some(id)` for a
    /// direct selection (number keys). Consumed by the next game tick.
    weapon_cycle: i8,
    gadget_cycle: i8,
    /// microcam-wheel-zoom: pending wheel zoom step (±1) — the wheel while the aim (right) button
    /// is held on a weapon that can zoom. Consumed by the next game tick, like `weapon_cycle`.
    zoom_cycle: i8,
    weapon_switch_to: Option<u8>,
    /// The external dev panel (http://127.0.0.1:7007, `--dev`) and the level its catalog describes.
    panel: Option<panel::Handle>,
    panel_level: u32,
    panel_frames: u32,
    /// Right mouse held: scope zoom (`Player_Zoom`).
    zoom: bool,
    /// Dev mode (WORKLOG 0083): `None` when `--dev` is off.
    dev: Option<Box<dev::Dev>>,
    /// Debug render overrides (Bug 1): `--no-tex` ignores textures (isolate vertex colour), `--no-fog` skips fog.
    notex: bool,
    nofog: bool,
    /// The ISO path (for level transitions: `LOAD_LEVEL` → reload the next archive).
    iso: String,
    /// A pending level transition: the next level's hashcode, set by `LOAD_LEVEL`; consumed by the main loop.
    pending_level: Option<u32>,
    /// difficulty-effects: the selected difficulty, the port's `GameState+0x28` (0x2a3790) —
    /// boot default 2 [M 0x1c7e78:25], the difficulty page's confirm writes it [M 0x216ca8:15],
    /// and every load/adopt reads it.
    selected_difficulty: u32,
    /// fix-weapon-assets: a dev-menu/panel give of a weapon whose assets the level archive lacks —
    /// the tick loop answers it with a warm same-level reload (`load_with_carry_weapons`) that
    /// merges the weapon's assets, then re-places Bond and re-issues the switch request.
    pending_weapon_assets: Option<u8>,
    /// fix-restart-target: the ENTRY checkpoint — the RamSave buffer 0x2b9440 the game keeps
    /// across level changes. The fail/death menu's Restart re-serves it (`P_ENDMISSION_Handler`
    /// 0x216618:21's `Player_RamLoad(Bond, 1)`) instead of capturing the dying state — the entry
    /// checkpoint's channels predate the fail, so the reload cannot re-fail (the 07000007 loop).
    /// fix-restart-health: re-captured with the POST-load state after EVERY load
    /// (`entry_checkpoint_save` — the game's `Player_InitWeapon` re-save [M 0x1a7ef8:325]; cold
    /// loads included, whose capture holds `Player_Init`'s health 100). The old model kept the
    /// save the level was loaded with — `None` after a cold load — and the restart served the
    /// zeroed buffer: health 50.0, half health.
    entry_save: Option<nf_game::player::RamSave>,
    /// The `--all` flag (draw dynamic entities at their placements, not only 0x8000 cels) — reused on level reload.
    all: bool,
    /// Input recorder (B-5): `None` unless `--record <file>` was given. Each tick's input is pushed right before
    /// `step_bond`; `finish()` writes the file on exit. Costs nothing when `None`.
    recorder: Option<replay::Recorder>,
    /// Input replay (B-5): `None` unless `--replay <file>` was given. When set, the tick loop reads the next input
    /// from `replay_ticks[idx]` instead of the keyboard/mouse, so the recorded session plays back bit-for-bit.
    /// `replay_idx == replay_ticks.len()` ⇒ the recording ended (the sim pauses).
    replay: Option<replay::Replay>,
    replay_idx: usize,
    /// The party weapon lock's rising-edge memory (`switch_channels_prev[0x48]`, `Player_Weapon`
    /// 0x1a24c8:247): the previous tick's channel state. Reset on every level load — the game's `_prev`
    /// is restored from the RamSave, whose value for the lock channel is 0 in every pre-party level.
    prev_lock_ch: bool,
    /// Render interpolation state: the cameras and palettes of the last two ticks, and whether a tick ran this frame.
    ticked: bool,
    /// This frame's 60 Hz tick count (the burning woman advances once per tick while a paused page shows her).
    frame_ticks: u32,
    /// The paused menus' 60 Hz accumulator (seconds) for the animations that run while the game is paused.
    menu_clock: f32,
    /// fe-60hz: the front end's own 60 Hz tick accumulator (seconds).
    fe_clock: f32,
    /// The sim reported the mission's own end this frame (see `Drones::mission_end`).
    pending_mission_end: bool,
    /// mission-results-page: the finished mission's progress layer, snapshotted at
    /// `demo_results` — the results sub-pages' data source after the sim is gone (the game's
    /// PlrMissionStats/ScoringTable are BSS globals the pages read until the next Mission_Init
    /// [M 0x209938:36, 0x182270:65]). Consumed by the `menu_after` hook through
    /// `FrontEnd::results_subpages_enter`.
    results_progress: Option<nf_game::progress::Progress>,
    /// mp-match-end: the sim's MP machine reached the debriefing handover (`MP_Update` 0x184bd0
    /// case 4 → `MpGame::request_debriefing`): the finished match's debriefing board and its
    /// settings (`Sim::take_mp_debrief_request`), drained by `App::mp_results` after the tick.
    pending_mp_debrief: Option<(nf_game::mp::debrief::MpDebriefing, nf_game::mp::MpSettings)>,
    /// Dev `--set-channel <ch>`: a switch channel raised 2 s into the level (channel 100 = mission
    /// success — what scratchpad complete.py writes in PCSX2), for end-of-mission checks.
    force_channel: Option<u8>,
    cam_prev: Option<Camera>,
    cam_cur: Option<Camera>,
    pal_prev: Option<(Vec<nf_formats::anim::Mat4>, Vec<[f32; 4]>)>,
    pal_cur: Option<(Vec<nf_formats::anim::Mat4>, Vec<[f32; 4]>)>,
    /// perf-240: the presented-frame palette (seeded per tick) and the entries that move between the two ticks.
    pal_blend: Vec<nf_formats::anim::Mat4>,
    pal_moving: Vec<u32>,
    /// The gadget-beam palette entries (`Scene::beam_entries`), refreshed when the palette is
    /// rebuilt.
    beam_entries: Vec<usize>,
    /// `--fps N`: a frame-rate cap (vsync off); None = vsync.
    fps_cap: Option<f32>,
    /// The always-on frame counter (top right): frames actually presented, counted over half-second
    /// windows — a port readout, never in the Sim.
    fps_meter: FpsMeter,
    frame_start: std::time::Instant,
    /// The front end (fix-menus): Some while a menu page is up (title / main / mission select /
    /// difficulty / options / pause / results). None = playing a mission directly.
    fe: Option<menu::FrontEnd>,
    /// The Advanced options' Auto Aim (control 0x1000011b; the profile's +0x01 — default ON [M
    /// Player_AutoAim 0x1a7c98:41 reads it]). Mirrored from the front end while it is up and kept
    /// across missions (the front end closes during play); applied to the sim every frame. The
    /// dev T key flips the same option.
    auto_aim_opt: bool,
    /// options-census: the Controls page's Y-axis row (0x1000_00c9) — the look delta negation
    /// [M Input_Update 0x17ee88:50-52 negates the two analog look channels when set].
    look_inverted: bool,
    /// options-census: the Advanced page's Crouch row (0x1000_0125) — Toggle (true, the game's
    /// default [M Input_Init 0x17edd8]) or Hold; feeds the sim's stand branch [M 0x1924c0:84-105].
    crouch_toggle_opt: bool,
    /// fix-909c: the last option values the front end held (the game's profile globals outlive
    /// the menus [M PlayerSetting 0x2a38c8 is read during play]). The front end is closed during
    /// play (`fe` None), so the drain reads this copy then — without it every row fell back to
    /// its default on the next level load and Auto Aim snapped back to the mirror.
    opts_seen: Option<std::collections::HashMap<u32, i32>>,
    /// net-consolidate: the ONE session layer (nf-net) while an online match runs (`--net-host` /
    /// `--net-join`, or adopted from the lobby pages' `fe.mp_session` when their match starts).
    /// None = local play, no session, no gate.
    net: Option<mpnet::SessionProxy>,
    /// The lobby's session (the same core `fe.mp_session` wraps): created when the online pages
    /// open, adopted into `net` when the match starts.
    net_lobby: Option<mpnet::SessionProxy>,
    /// This machine's slot in the online match (the host is 0; a client's arrives in WELCOME).
    net_local_slot: usize,
    /// mp-join-in-progress: the announced join (slot, tick) — the app spawns
    /// the slot's agent at that tick (`Sim::mp_join_spawn`) on every machine.
    net_join_pending: Option<(u8, u32)>,
    /// mp-join-in-progress: the joiner's completed handover (the settings,
    /// the snapshot tick + hash and the recorded log) — the app replays it
    /// (`mp_late_join_catchup`) and confirms or refuses. The draw code reads
    /// this for the joining-in-progress page.
    net_late_handover: Option<(Box<nf_game::mp::MpSettings>, u32, u64, Vec<[nf_net::Input; nf_net::MAX_AGENTS]>)>,
    /// The next sim tick of the online match (the lockstep counter; render interpolation is
    /// unchanged — `ticked` still marks the frames that advanced it).
    net_tick: u32,
    /// The online match's negotiated settings (the host's choices + seed): set when the session
    /// reports `MatchStarted`, consumed by the next level load (`load_mp_carry`). The local menu
    /// confirm path rides the same slot (its `mp_match_settings` was set but never read before —
    /// the menu's own choices now reach the match core through it too).
    pending_mp_settings: Option<nf_game::mp::MpSettings>,
    /// The online match is armed but its level has not been swapped in yet: no lockstep tick runs
    /// (the host ran 3 match ticks on its pre-match scene, then the load restarted the sim at
    /// frame 1 with `net_tick` still at 3 — every later hash compared different frames; the
    /// two-window check, 2026-10-18). Cleared, with `net_tick = 0`, by `swap_in_scene`.
    net_await_load: bool,
    /// The lobby path adopted the session and its menu handover already requested the arena —
    /// the MatchStarted event must not request it again (one load per match).
    net_menu_load: bool,
    /// mp-join-in-progress: this machine is the late joiner of that slot, still ticking the
    /// replayed slot-0 view until the announced join tick.
    net_late_self: Option<usize>,
    /// Online rematch: how this machine got into the running match — (hosted, its port / the
    /// address it dialed) — handed to the debriefing so Continue returns to the online lobby.
    net_rejoin: Option<(bool, String)>,
    /// The finished online match's session, kept open while its debriefing is up.
    net_finished: Option<mpnet::SessionProxy>,
    /// net-match-end: the host's end wait — the finished match's session stays LIVE (polled,
    /// relaying inputs) until every human peer has acked its own final tick + debriefing, or
    /// departed, or the wall deadline passes. Only then does the session move to `net_finished`
    /// (docs/systems/net-match-end.md §4) — port-only: the shipped game has no online [D
    /// action-symtab.tsv census: zero socket/TCP/UPnP symbols].
    net_end_wait: Option<NetEndWait>,
    /// The online match's wall clock (set when its level swaps in): the lockstep runs to
    /// `elapsed · 60` ticks, catching up after a stall instead of dropping the stalled ticks.
    net_clock: Option<std::time::Instant>,
    /// net-internet: the online tick budget's stall smoothing (`nf_net::pace::TickPacer`).
    net_pacer: nf_net::pace::TickPacer,
    /// net-desync-resync: this machine is resyncing after a state-hash mismatch —
    /// it asked the host to re-serve the match state and replays the host's log
    /// headless (`mp_late_join_catchup_joins`) until the re-announce arms its
    /// slot again. The draw code renders the "Resyncing…" note from the getter
    /// (`App::net_resyncing`); cleared when the machine plays its own slot again
    /// (the late-join slot swap) or on `net_fail`. NOT in the Sim — the session
    /// policy (the 60 s window, the 8 s watch) lives in nf-net's logical clock.
    net_resyncing: bool,
    /// 700v6: the local fire pipeline spawned a round on the last tick (rides the next input).
    net_shot_pending: Option<scene::NetShot>,
    /// net-desync-detail: this machine's desync watcher — the component-hash history, the
    /// diagnosis exchange and the report it builds when the session reports a desync.
    desync_watch: desync::Watcher,
    /// net-desync-detail: the desync report line the draw code can show on the fail box (the
    /// watcher's diagnosis + where the zip went); `menu::FrontEnd::desync_report` mirrors it.
    desync_report: Option<String>,
    /// net-desync-detail: the desync tick being diagnosed (the exchange window — the report or
    /// the fail box follows).
    desync_wait: Option<(u32, std::time::Instant)>,
    /// The `--net-host` demo arm: the settings waiting for the first client to join (the lobby
    /// pages are separate; the flag path starts the match as soon as someone connects).
    net_armed: Option<nf_game::mp::MpSettings>,
    /// The current level's hashcode (the archive name), for the pause page's restart / the results.
    level_code: u32,
    /// Port dev aid (not game behaviour): the `--bond x,y,z[,yaw,pitch]` spot,
    /// re-applied once when the level's opening cut-scene hands Bond back (its hand-back moves
    /// him to the scene's end pose) — so a check starts where it says. Cleared after the first
    /// re-apply, or after 900 ticks when the level has no opening scene.
    dev_bond: Option<[f32; 5]>,
    /// `dev_bond`'s latch: an opening cut-scene (sim.disabled) has been seen.
    dev_bond_cut: bool,
    /// frontend-music: the front-end page the music transitions last saw (the manager's msg-0x44
    /// page change re-runs the new page's 0x4c arm [M 0x21c858:345-346]); None before the first.
    fe_music_page: Option<menu::Page>,
    /// frontend-music: the host's SFXStart handle counter (the game's RPC passes the EE node
    /// handle; the port's driver dedupes/tracks by it).
    menu_handle: u32,
    /// frontend-music (menu-audio-state): the game's music statics, which are .bss bytes that
    /// outlive every menu manager [M 0x1dceb8's `DAT_002e1b11`; L `MusicMasterVolume`
    /// 0x30c958]. Stashed from the front end on every manager drop (`stash_music_statics`) and
    /// restored into every fresh one (`inherit_music_statics`).
    music_bank_flip: bool,
    music_loaded_bank: u32,
    music_master: i32,
    /// create-save (profile-session): the codename session the game keeps in globals OUTSIDE
    /// every menu manager — the save slot `DAT_002e1ae0` + the working codename buffer
    /// `DAT_002e1ae8` [M C_KEYBOARD 0x213a98:53-57 writes them; P_NFRESULTS 0x208e78:30/145/194
    /// reads them after fresh managers] — plus the port's card dir. Stashed from the front end
    /// on every manager drop (`stash_profile_session`) and inherited into every fresh one
    /// (`inherit_profile_session`), the music statics' pattern.
    profile_session: menu::ProfileSession,
    /// The game's strings (USATxt.dat) for the front end when the loaded map has no Sim to hold them.
    lang: Option<nf_formats::hud::Lang>,
    /// The gamepad backend (fix-input), behind the `gamepad` cargo feature. `None` without a pad
    /// device; with one, its Input merges into the keyboard's each tick (the game's own mapping,
    /// curves and ramps — docs/systems/input.md §8).
    #[cfg(feature = "gamepad")]
    pad: Option<gamepad::Backend>,
    /// fix-menu-movies2: the ONE playing movie, the port's `psiStartBackgroundMovie` [M 0x1081a8].
    /// Every movie rides it — the GameFlow state-6 briefing/outro (fix-movie-playback), the
    /// P_INTRO pair and the page's looping backdrop (fix-menu-movies); the host's `movie_tick`
    /// decides which one the game would be playing and `present()` feeds the ONE plane in
    /// gpu.rs. Cleared by the skip, the end of a non-looping stream, or a code change.
    movie: Option<movie::Player>,
    /// `--dev`: the dev menu in its own native window beside the game (devwin.rs; a native window, not a browser
    /// page). None without `--dev`, or when the window couldn't open (the F2 overlay is the fallback).
    devwin: Option<devwin::DevWindow>,
    /// The sound hold's mirror (fix-pause-audio): true while a menu page is up or the viewer pause
    /// holds the sim. The diff in `RedrawRequested` applies it to `sound` on the edge — the game's
    /// pause page runs `SFXPause`+`SFXPauseAllStreams` on open and the unpause pair on close
    /// [M 0x21b908:121-124, 0x21c2b0:11-22].
    audio_paused: bool,
    /// The loading screens' pictures (07000500.bin, crate::loading), read on the first level change.
    loading_art: Option<std::collections::HashMap<u32, (u32, u32, Vec<u8>)>>,
    /// The loading screen's hint, picked ONCE per load (level, label) — `ResetMap_GenLoadScreen`
    /// 0x1d5de0 builds the screen a single time per load [M]; cleared by `start_async_load`.
    loading_hint: loading::HintPick,
    /// The pointer in the menu's 640×480 space (PC mouse support).
    menu_pointer: Option<(f32, f32)>,
    /// async-level-load: the level load running on a worker thread — `Some` from the frame the
    /// transition spawns it until the scene is swapped in. While it runs, the window thread
    /// redraws the loading screen with the dot animation each frame (`poll_async_load`); the
    /// game does the same on one thread: the load blocks the main thread while `VBlankInt`
    /// 0x124088 steps the dots every 11 vblanks.
    async_load: Option<LevelLoad>,
}

/// A running level load plus what its swap needs (async-level-load).
struct LevelLoad {
    job: async_load::Job<Result<scene::Scene, String>>,
    /// The level code the load serves (`pending_level`'s value at spawn): the loading screen's
    /// picture and the failure log read it.
    next: u32,
    /// drive-ptp: the DRIVING return (`Boot_LoadPTPData` 0x1c8560), taken from the OLD sim at
    /// spawn and applied to the NEW one at the swap (see `swap_in_scene`).
    ptp_return: Option<nf_game::ptp::DriveReturn>,
    /// The finished build, parked while the loading screen's minimum time runs (`LOADING_HOLD`).
    ready: Option<Option<Result<scene::Scene, String>>>,
}

/// The loading screen's minimum time on screen. The PS2's disc load kept it up ~11.5 s on
/// 07000001 → 07000002 (PS2LSActive 2 → 0, measured in PCSX2 2026-10-15) — long enough to read
/// the objective and the hint while the dots sweep (183.5 ms a step, the port's 11-vblank cadence
/// exactly). The port builds a level in 1-2 s, so the screen flashed by; the
/// port holds it a few seconds — a documented PC-port choice (the game's disc timing is
/// the [L] above; the port's 1-2 s builds need their own floor). Pub for the
/// flow tests (pause-results-flows): the headless suite pins the 5 s = 300-tick window.
pub const LOADING_HOLD: f64 = 5.0;

/// microcam-wheel-zoom: where a wheel notch goes. While the aim (right) button is held on a weapon
/// that can zoom — the game's own scope flag +0x68 & 0x40 with a magnification above 1 [M
/// weapon_data 0x2bf150; the sim's zoom block gates on the same two] — the wheel steps the zoom.
/// The game steps its zoom with action 6 in aim mode (D-pad Up = −1 / Down = +1 [M
/// docs/systems/input.md §3 style-0 table, measured]; the consumer is `Player_Zoom` 0x1a1a70:19,
/// `Input_Actionf(player, 6, 1)` [M]) — a wheel notch is the PC surrogate for the game's own
/// zoom-step action (the wheel, not key repeats). Otherwise it
/// keeps switching weapons (the pre-feature behaviour).
pub fn wheel_steps_zoom(zoom_held: bool, weapon_flags: u32, weapon_zoom: f32) -> bool {
    zoom_held && weapon_flags & 0x40 != 0 && weapon_zoom > 1.0
}

/// net-match-end: the host's end wait (docs/systems/net-match-end.md §4) — port protocol, the
/// disc has no online [D action-symtab.tsv census: zero socket/TCP/UPnP symbols]. The finished
/// match's session stays live while the host's debriefing is
/// up, so a client still draining its last ticks keeps its relay; the wait ends when every human
/// peer has acked its own debriefing or departed (a dead peer cannot hold it — the deadline).
struct NetEndWait {
    /// The human slots that must ack (snapshot at the handover; `slot_human` shrinks on a
    /// departure [M nf-net Session::drop_slot], the snapshot keeps them countable).
    peers: Vec<usize>,
    /// Peers whose ack arrived (their board hash matched the host's own fold — a mismatch logs
    /// and shows on the debriefing's notice, the ack still counts).
    acked: Vec<usize>,
    /// Peers seen departing (ClientLeft / dropped by the session's timeout).
    gone: Vec<usize>,
    /// The host's own board fold, to compare the acks against.
    board_hash: u64,
    /// Wall deadline — a dead peer can't hold the debriefing forever.
    deadline: std::time::Instant,
    /// Whether any ack's fold disagreed with the host's own board.
    board_mismatch: bool,
}

impl NetEndWait {
    fn done(&self) -> bool {
        self.peers.iter().all(|p| self.acked.contains(p) || self.gone.contains(p))
    }
}

impl App {
    /// net-consolidate: an online match starting adopts the lobby's session as the lockstep
    /// gate. `has_match` is exact: match_start is Some only after the host's arm / the client's
    /// tunnel payload, and a cancelled lobby's core is closed. Called after a menu action AND
    /// every frame — the client's handover comes from the lobby poll (movie_tick takes its
    /// level request), and adopting only in menu_after left the joiner loading the arena
    /// offline while the host waited on it (the two-window menu check: "player 2 timed out").
    fn adopt_lobby_session(&mut self) {
        // mp-online-failures: a late join the player cancelled (Back on the join page while the
        // handover ran) closed the adopted core — drop it, the menus own the online pages again.
        if self.net.as_ref().is_some_and(|p| p.core_is_closed())
            && self.fe.as_ref().is_some_and(|fe| matches!(fe.page, menu::Page::MpMode | menu::Page::MpNetJoin | menu::Page::MpHost))
        {
            self.net = None;
            self.net_late_self = None;
            self.net_join_pending = None;
            self.net_late_handover = None;
        }
        let kind = self.net_lobby.as_ref().map_or(mpnet::LobbyAdopt::No, |p| p.adopt_kind());
        if kind != mpnet::LobbyAdopt::No && self.net.is_none() {
            if let Some(p) = self.net_lobby.take() {
                if let Some(fe) = self.fe.as_ref() {
                    self.net_rejoin = Some(if fe.mp_hosting { (true, fe.mp_net_port.to_string()) } else { (false, fe.mp_net_addr.trim().to_string()) });
                }
                self.net_local_slot = p.0.borrow().local_slot();
                self.net_tick = 0;
                self.net_resyncing = false; // net-desync-resync: a fresh adoption starts clean
                self.net = Some(p);
                if kind == mpnet::LobbyAdopt::LateJoin {
                    // mp-online-failures: the menu dial reached a running match — no arena load
                    // and no load hold: the handover's replay builds the scene (the
                    // LateJoinHandover arm below), exactly the flag path's late join.
                    self.net_await_load = false;
                    self.net_menu_load = false;
                    eprintln!("[net] adopted the lobby's session for a join in progress");
                } else {
                    self.net_await_load = true;
                    self.net_menu_load = true;
                    eprintln!("[net] adopted the lobby's session for the match");
                }
            }
        }
    }

    /// The online pages need their session: the proxy is created lazily on page entry (the
    /// listen/dial rows re-bind its unbound placeholder core); `net_lobby` holds the same Rc so
    /// the app can adopt it when the match starts. Not while a session is adopted (a late join
    /// in progress keeps the join page up — a fresh proxy would orphan it).
    fn ensure_lobby_session(&mut self) {
        let Some(fe) = self.fe.as_mut() else { return };
        if matches!(fe.page, menu::Page::MpHost | menu::Page::MpNetJoin) && self.net_lobby.is_none() && self.net.is_none() {
            if let Ok(p) = mpnet::SessionProxy::host(0) {
                // net-p2p: the app's sessions probe internet reachability (UPnP / NAT-PMP /
                // STUN) on listen and dial; NF_NO_REACH=1 keeps them LAN-only, NF_STUN
                // ("host:port,host:port") replaces the public STUN list. Port-only surface —
                // the shipped game has no online at all [D action-symtab.tsv census: zero
                // socket/TCP/UPnP symbols]; docs/systems/net-p2p.md §1 owns the design.
                // net-cloud-lobby: the cloud meeting point rides along (nf-lobbyd overrides).
                enable_net_services(&p);
                fe.mp_session = Some(Box::new(p.clone()));
                self.net_lobby = Some(p);
            }
        }
    }

    /// mp-online-failures: this machine leaves the running online match on its own (the pause
    /// page's QUIT): LEAVE goes out now — the others continue on NEUTRAL for this slot (a client)
    /// or return to their online pages at once (the host) — instead of the session lingering
    /// with keepalives and every other machine stalled on this one's missing inputs forever.
    fn net_leave_match(&mut self) {
        if let Some(p) = self.net.take() { p.0.borrow_mut().close(); }
        self.net_lobby = None;
        self.net_menu_load = false;
        self.net_late_self = None;
        self.net_armed = None;
        self.net_join_pending = None;
        self.net_late_handover = None;
        self.net_await_load = false;
        self.net_rejoin = None;
        eprintln!("[net] left the online match");
    }

    /// Test aid (NF_MENU_SCRIPT="ms:step,ms:step,..."): menu presses for the two-window MP checks,
    /// no desktop input. Each `ms` is the delay after the PREVIOUS step (the first counts from the
    /// first frame). Steps: up/down/left/right/confirm/back/side/extra (`down*30` repeats), shot
    /// (the F12 capture), type=TEXT (the join page's typing), page=Name (wait until that page is up
    /// and no load runs — `page=None` is "in a level, no menu"), sleep (nothing). A wait holds the
    /// script, so a slow boot or load never lands presses on the wrong page.
    fn menu_script_tick(&mut self) {
        use std::sync::atomic::Ordering::Relaxed;
        static LAST: std::sync::OnceLock<std::sync::Mutex<std::time::Instant>> = std::sync::OnceLock::new();
        static NEXT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
        let Ok(spec) = std::env::var("NF_MENU_SCRIPT") else { return };
        let last = LAST.get_or_init(|| std::sync::Mutex::new(std::time::Instant::now()));
        let steps: Vec<(u64, &str)> = spec.split(',').filter_map(|s| s.split_once(':')).filter_map(|(a, b)| Some((a.trim().parse().ok()?, b.trim()))).collect();
        while let Some(&(delay, step)) = steps.get(NEXT.load(Relaxed)) {
            let since = last.lock().map(|l| l.elapsed().as_millis() as u64).unwrap_or(0);
            if since < delay { break; }
            if let Some(want) = step.strip_prefix("page=") {
                let now = format!("{:?}", self.fe.as_ref().map(|f| f.page)).replace("Some(", "").replace(')', "");
                if now != want || self.async_load.is_some() || self.pending_level.is_some() { break; }
            } else if self.async_load.is_some() {
                break; // no menu input while a level load runs
            }
            NEXT.fetch_add(1, Relaxed);
            if let Ok(mut l) = last.lock() { *l = std::time::Instant::now(); }
            use menu::Action as MA;
            // "down*30" repeats a press.
            let (step, times) = match step.split_once('*') { Some((s, n)) => (s, n.parse().unwrap_or(1)), None => (step, 1usize) };
            let a = match step { "up" => Some(MA::Up), "down" => Some(MA::Down), "left" => Some(MA::Left), "right" => Some(MA::Right),
                "confirm" => Some(MA::Confirm), "back" => Some(MA::Back), "side" => Some(MA::Side), "extra" => Some(MA::Extra), _ => None };
            if let Some(a) = a { for _ in 0..times { self.menu_input(a); } }
            else if step == "shot" { let m = self.spot_shot(); eprintln!("[spot] {m}"); }
            // "bksp": the address/port pages' Backspace (the host's port field starts at 27420).
            else if step == "bksp" { for _ in 0..times { if let Some(fe) = self.fe.as_mut() { fe.mp_net_backspace(); } } }
            // "lobby": one line with this machine's one-screen lobby model (the two-window
            // settings check compares the host's and the client's lines) — a port test aid of
            // the NF_ENU_SCRIPT dev rig, not a game surface (no counterpart in the shipped
            // game's flow).
            else if step == "lobby" {
                if let Some(fe) = self.fe.as_ref() { eprintln!("[lobby] {}", fe.mp_one.debug_line()); }
            }
            // "row=Enviro(3)": real Up/Down presses until the lobby focus sits on that row id
            // (the row list shifts with joins and bots, so fixed press counts drift) — same
            // NF_ENU_SCRIPT dev-rig test aid, port-only.
            else if let Some(want) = step.strip_prefix("row=") {
                for _ in 0..80 {
                    let Some(fe) = self.fe.as_mut() else { break };
                    let rows = fe.mp_one.rows();
                    let Some(at) = rows.iter().position(|r| format!("{:?}", r.id) == want) else { break };
                    let cur = fe.mp_one.focus;
                    if cur == at { break; }
                    self.menu_input(if cur < at { MA::Down } else { MA::Up });
                }
            }
            else if let Some(text) = step.strip_prefix("type=") {
                if let Some(fe) = self.fe.as_mut() { for c in text.chars() { fe.mp_net_type(c); } }
            }
            eprintln!("[menu-script] +{delay} {step} -> page {:?} net_tick {}", self.fe.as_ref().map(|f| f.page), self.net_tick);
        }
    }

    /// One menu action from the keyboard or the mouse (PC mouse support, 2026-10-15): the movie skip, the pause page's
    /// Back = resume, the page's own handling, then the level request / close the page asked for.
    fn menu_input(&mut self, a: menu::Action) {
        if self.fe.is_none() { return; }
        use menu::Action as MA;
        let fe = self.fe.as_mut().unwrap();
        // State 6 (the briefings/outros): only the skip action matters [M GameFlow_Main 0x1c98c8
        // case 6:145-149 — it reads Input_Action(-1, 0x19, 1) and psiMovieFinished and nothing
        // else; the menu pages are not up during a movie]. The game's skip button is action
        // 0x19 = R1/D-pad-down, mask 0x840, pad 0 [M psiInput_MapInputs 0x10cd80:416]; the port
        // binds those buttons' PC keys (ArrowDown/R) to the skip AT ONCE — case 6 has no time
        // gate (App::app_key's front-end arm). The Confirm keys (Enter/Space) are NOT the game's
        // skip button: the pad's confirm buttons (X/Start) never reach case 6, so binding them
        // to the skip is the PC adapter's own convenience, kept behind the 1.0 s guard below so
        // the press that picked the mission cannot skip its own briefing (a
        // double-tap) [S unverified as a game claim — the game needs no guard because
        // its skip key is disjoint from its pick keys]. The front-end intro run is NOT state 6:
        // it reaches `fe.action`, which honours P_ESTHERO's 180-tick grace and the boot logos'
        // no-skip [M 0x215a50:6] (fe-audit: this intercept used to swallow the intro's skip).
        if fe.flow.get_state() == menu::ST_MOVIE {
            // The Confirm guard (the game's own 0x19 keys skip ungated in app_key):
            let shown = self.movie.as_ref().map_or(f32::MAX, |p| p.shown_for());
            if a == MA::Confirm && shown >= 1.0 { fe.flow.skip_movie(); }
            return;
        }
        if fe.page == menu::Page::Pause && a == MA::Back {
            // profile-session: the pause front end drops here — the session stands [M the
            // globals outlive MenuManager_Delete; the next manager inherits].
            self.stash_profile_session();
            self.fe = None; // resume [S: the game's resume is its own row; Back resumes too]
            if self.scene.sim.is_some() { self.grab(true); }
            return;
        }
        fe.action(a);
        self.menu_after();
    }

    /// After a menu action (key, click or hover): the page's sounds, the level it requested and its close.
    fn menu_after(&mut self) {
        // mission-results-page: the results sub-pages' 0x4c builds — the game delivers the 0x44
        // page change to the opened page, whose enter arm runs against the progress layer (the
        // Stats page's GetScore(0) [M P_NFSTATS 0x209938:31-38]; the bonus page's
        // Menu_GetLevelBonuses + the DAT_002e1c2c text gate [M P_NFBONUS 0x209670:14-18]). The
        // port builds here, after the action, against the mission-end snapshot.
        if self.results_progress.is_some() {
            let sub = matches!(self.fe.as_ref().map(|fe| fe.page),
                Some(menu::Page::Stats) | Some(menu::Page::NfBonus));
            if sub {
                let lang = self.lang.clone();
                if let (Some(fe), Some(p)) = (self.fe.as_mut(), self.results_progress.as_mut()) {
                    fe.results_subpages_enter(p, lang.as_ref());
                }
            }
        }
        // profile-session: every action may move the session (a create, a codename load, a
        // results save) — keep the standing copy current so any manager drop hands it on.
        self.stash_profile_session();
        let Some(fe) = self.fe.as_mut() else { return };
        // difficulty-effects: the difficulty page's confirm writes GameState+0x28 at pick time
        // [M C_SBNFDFCTY 0x216ca8:15] — the game keeps it across loads; the port stores it on
        // the App and every following load adopts it (Scene::build's create-time gates).
        if let Some(d) = fe.requested_difficulty.take() {
            self.selected_difficulty = d;
        }
        // settings-persist: the option pages' Confirm/Back raised the flag — write the option
        // file (crates/nf-view/src/settings.rs). The headless render path never runs this, so
        // a render can't touch the player's settings.
        if fe.settings_save_pending {
            fe.settings_save_pending = false;
            if let Some(p) = settings::default_path() {
                if let Err(e) = settings::save_opts(&p, &fe.opts) { eprintln!("[settings] {e}"); }
            }
        }
        // menu-sfx: the front end's sound events play straight through the driver's 2-D start —
        // `Menu_PlaySound`'s `Sound_Play(100.0f, id, 0, 0)` tail [M 0x1f3278] batched like the EE's
        // `SFXStart`. The loaded bank(s) decide audibility exactly as the game's lookup does (a
        // miss returns without starting): the front-end map boots bank 0xC, a paused mission keeps
        // its own bank, which carries the pause blips [D SB_4/SB_5; docs/systems/menu-sfx.md §2-3].
        let menu_sfx: Vec<_> = fe.drain_sfx().collect();
        let menu_rm: Vec<u32> = fe.drain_sfx_removes().collect();
        if !menu_sfx.is_empty() || !menu_rm.is_empty() {
            if let Some(snd) = self.sound.as_mut() {
                snd.batch(|drv, out| {
                    // The 0x4c order: the removes land before the same frame's starts [M
                    // 0x20a408:17-22] — the restart replaces the loop instead of stacking it.
                    // menu-music-stack: the pair's removes belong to the music channel too — the sfx channel's
                    // remove landed after the music channel's start and killed it (half a second of menu music,
                    // then silence at the main menu).
                    for id in &menu_rm { if menu_music::menu_sfx_reaches_driver(*id) { drv.remove(0, Some(*id)); } }
                    for ev in &menu_sfx { if menu_music::menu_sfx_reaches_driver(ev.id) { drv.start(out, ev.id, None, 0, None); } }
                });
            }
        }
        // net-consolidate: the online pages need their session (`ensure_lobby_session`).
        self.ensure_lobby_session();
        // mp-online-failures: the pause page's QUIT out of a running online match (the quit
        // requests the front-end map) leaves the session — LEAVE now, not a lingering socket.
        let quitting_online = self.net.is_some()
            && self.fe.as_ref().is_some_and(|fe| fe.requested_level == Some(menu::FRONTEND_LEVEL));
        if quitting_online { self.net_leave_match(); }
        let Some(fe) = self.fe.as_mut() else { return };
        let closed = fe.closed;
        if let Some(code) = fe.requested_level.take() {
            // The menu picked a level: through the same reload path a LOAD_LEVEL uses. A cold pick
            // (the pause page's RESTART [M 0x204408:139]) loads fresh, not with the current state.
            if !fe.flow.warm_reset { if let Some(sim) = self.scene.sim.as_mut() { sim.drones.level_exit_cold = true; } }
            // death-loop-07: the results page's REPLAY row carries the `Player_RamLoad(Bond, 1)`
            // half [M P_ENDMISSION_Handler 0x216618:19-21] — mark the serve so the reload comes
            // off the ENTRY checkpoint (the restart arm), not the dying sim's live capture: the
            // carried fail channel re-failed the mission the tick it came back — the 07000007
            // replay loop. Consumed with the request, one pick only.
            let replay_pick = std::mem::take(&mut fe.requested_restart);
            if replay_pick { if let Some(sim) = self.scene.sim.as_mut() { sim.drones.level_exit_restart = true; } }
            self.pending_level = Some(code);
            // net-match-host: the confirm page's settings ride with the level (`mp_confirm_start`
            // builds them [M mp-menu-flow]; before this they were set but never read — the match
            // core now gets the menu's own scenario/map/limits/players).
            if let Some(st) = fe.mp_match_settings.take() { self.pending_mp_settings = Some(st); }
            // net-consolidate: an online match starting (the host's confirm page armed the
            // session, or the client took the tunnel's match payload) — adopt the lobby's
            // session as the lockstep gate (also run every frame: the client's start arrives
            // on a lobby tick, not an action — `adopt_lobby_session`).
            self.adopt_lobby_session();
        }
        // The pause page's Resume row (cursor 0) closes the menu [M C_GCPAUSE_Handler
        // 0x204408 — the game's resume is its own row]: the host consumes the flag and
        // leaves the menu (the sound hold lifts through the diff below).
        if closed {
            self.close_front_end();
        }
    }

    /// The front end closes (a briefing resolved to a level, the pause's pick, the resume):
    /// the menu-music pair dies with it. The game's exit path plays the menus fade and lets the
    /// level's own `SFXLoadSoundBank` reset the items [M C_SBNFMAP 0x20b8d8 case 0x4b —
    /// SFXFadeDown(1), no explicit pair remove]; the port's banks swap without that reset, so
    /// the close removes the pair explicitly through the driver (all instances of the id — the
    /// IOP's KillSFX semantics [M IRX 0x1f88 → KillSFX 0x1ea4]). Without it the looped items
    /// played on through gameplay and the pause (the menu music continued after the menu
    /// closed).
    fn close_front_end(&mut self) {
        let rm: Vec<u32> = self.fe.as_mut().map(|fe| fe.drain_sfx_removes().collect()).unwrap_or_default();
        if let Some(snd) = self.sound.as_mut() {
            snd.batch(|drv, _out| {
                for id in &rm { drv.remove(0, Some(*id)); }
                for id in [menu::SFX_MENU_OPEN_MOVE, menu::SFX_MENU_OPEN_SELECT] { drv.remove(0, Some(id)); }
            });
        }
        self.stash_music_statics();
        self.stash_profile_session();
        self.fe = None;
        if self.scene.sim.is_some() { self.grab(true); }
    }

    /// frontend-music (menu-audio-state): the music statics are .bss bytes that OUTLIVE every
    /// menu manager — `DAT_002e1b11` (the bank alternation) persists across front-end loads and
    /// rides the PTP data across boots [M 0x1dceb8:94-101, Boot_LoadPTPData 0x1c8560:61], and
    /// `MusicMasterVolume` @ 0x30c958 outlives the managers the same way [L ACTION.ELF .data].
    /// The port keeps them per-`FrontEnd`, so every drop of a manager stashes them here and
    /// every create of one (the gameplay pause's `get_or_insert_with`, `demo_results`) restores
    /// them — without it a fresh pause front end re-played the boot's jingle on the
    /// quit-to-front-end load instead of the alternation's other one.
    fn stash_music_statics(&mut self) {
        if let Some(fe) = self.fe.as_ref() {
            self.music_bank_flip = fe.music.bank_flip;
            self.music_loaded_bank = fe.music.loaded_bank;
            self.music_master = fe.music.music_master;
        }
    }

    /// The create side of [`App::stash_music_statics`]: a fresh front end inherits the statics.
    fn inherit_music_statics(fe: &mut menu::FrontEnd, bank_flip: bool, loaded_bank: u32, master: i32) {
        fe.music.bank_flip = bank_flip;
        fe.music.loaded_bank = loaded_bank;
        fe.music.music_master = master;
    }

    /// create-save (profile-session): the stash side — the manager drop leaves the codename
    /// session standing, exactly as the game's globals survive MenuManager_Delete [M
    /// 0x2e1ae0/0x2e1ae8 .data]. Only a front end WITH a card updates the stash: a dirless
    /// one (the headless render paths never reach the App) must not clobber it.
    fn stash_profile_session(&mut self) {
        if let Some(fe) = self.fe.as_ref() {
            if fe.profile_dir.is_some() {
                self.profile_session = menu::ProfileSession::from_fe(fe);
            }
        }
    }

    /// The create side of [`App::stash_profile_session`]: a fresh front end inherits the
    /// standing session — the card dir + its enumeration [M Menu_EnumSaves 0x1f7b78] and the
    /// codename in play (the pause's `get_or_insert_with`, the results/debriefing handovers).
    fn inherit_profile_session(&self, fe: &mut menu::FrontEnd) {
        self.profile_session.apply_to(fe);
    }

    // ── net-match-host: the online session's app surface ────────────────────────────────────────
    /// Host: listen on `port` (slot 0 is this machine's agent). The lobby pages are separate;
    /// the flag path (`--net-host`) arms the demo match for the first join.
    pub fn net_host(&mut self, port: u16) -> Result<(), String> {
        // net-p2p: an ephemeral placeholder, reach on, then the real port — so the mapping /
        // STUN probe starts for it (the console prints the join code once known).
        let p = mpnet::SessionProxy::host(0)?;
        enable_net_services(&p);
        p.0.borrow_mut().rebind_host(port)?;
        // net-lobby-list: the flag host advertises too (the lobby page's host sets it each tick).
        { let mut q = p.clone(); menu::MpSession::set_lobby(&mut q, "Bond's game", self.net_armed.as_ref().map_or(0x0700_0023, |s| s.level_code), 1, 1); }
        self.net = Some(p);
        self.net_lobby = None;
        self.net_local_slot = 0;
        self.net_tick = 0;
        self.net_rejoin = Some((true, port.to_string()));
        Ok(())
    }

    /// Client: dial `addr` ("host:port"); the slot arrives in the session's WELCOME.
    pub fn net_join(&mut self, addr: &str) -> Result<(), String> {
        // net-p2p: `addr` may be a join code too.
        let s = mpnet::SessionProxy::host(0)?;
        enable_net_services(&s);
        s.0.borrow_mut().dial(addr)?;
        s.0.borrow_mut().set_codename("Bond");
        self.net = Some(s);
        self.net_lobby = None;
        self.net_local_slot = 0; // corrected when the WELCOME lands
        self.net_tick = 0;
        self.net_rejoin = Some((false, addr.trim().to_string()));
        Ok(())
    }

    /// net-desync-detail: send the watcher's next exchange packet (`PT_HASHCOMPS` or
    /// `PT_HASHDETAIL`).
    fn send_desync_action(&mut self, a: desync::Action) {
        let Some(net) = self.net.as_ref() else { return };
        let mut core = net.0.borrow_mut();
        match a {
            desync::Action::SendComps(tick, comps) => core.send_hash_comps(tick, comps),
            desync::Action::SendDetail(tick, component, items) => core.send_hash_detail(tick, component, items),
        }
    }

    /// net-desync-detail: the diagnosis exchange completed — build the desync report zip (the
    /// bug-report format, `kind: desync`), offer the same "send report" path as F10 (the issue
    /// form pre-filled + the zip shown in the file manager), and carry the diagnosis into the
    /// fail box. The report holds the first diverging component and slot, the match settings +
    /// seed, the full input log since the match start (`--bug-replay` replays it) and the
    /// component hash history.
    fn finish_desync_report(&mut self) {
        let Some(diag) = self.desync_watch.diagnosis.take() else { return };
        self.desync_wait = None;
        // Gather everything the report needs (owned) before touching `self` again.
        let built = (|| {
            let sim = self.scene.sim.as_ref()?;
            let m = sim.mp.as_ref()?;
            let d = &sim.drones;
            let channels = d.triggers.len() + d.doors.list.len() * 2 + d.props.destroy.len()
                + d.pickups.len() + d.sss.list.len() + sim.hurts.len();
            let counts = [d.pickups.len(), d.doors.list.len(), d.triggers.len(), channels, d.props.destroy.len(), d.breaks.list.len(), d.sss.list.len(), sim.hurts.len()];
            let settings_payload = mpnet::encode_match_start(&mpnet::mp_to_match(&m.settings), m.settings.seed);
            let (base, log) = self.net.as_ref().map(|n| n.0.borrow().run_log()).unwrap_or((0, Vec::new()));
            let hist = self.desync_watch.hist.to_text();
            Some((sim.level, desync::desync_report_files(&diag, sim.level, &settings_payload, base, &log, counts, &hist)))
        })();
        let Some((level, files)) = built else {
            self.net_fail(&format!("State desync at tick {}\n({})\nReturning to the lobby", diag.tick, diag.ui_line()));
            return;
        };
        let id = files.iter().find(|(f, _)| f == "report.txt")
            .and_then(|(_, b)| String::from_utf8(b.clone()).ok())
            .and_then(|r| r.lines().find_map(|l| l.strip_prefix("report_id: ").map(str::to_string)))
            .unwrap_or_default();
        let url = bugreport::issue_url(&format!("[Desync] {level:08x}: {}", diag.ui_line()), &[
            ("version", bugreport::version_line()),
            ("report_id", id.clone()),
            ("what_happened", format!("The match desynced.\n\n{}\n\n(What were you doing just before?)", diag.ui_line())),
            ("details", bugreport::scrub(&format!("{}\n{}", bugreport::os_line(), diag.ui_line()))),
        ]);
        let line = match bugreport::save(&id, files, url.clone()) {
            Ok(saved) => {
                // The same "send report" path as F10: the form opens pre-filled, the zip shows.
                self.grab(false);
                let opened = bugreport::open_url(&saved.url);
                bugreport::reveal(&saved.zip);
                match opened {
                    Ok(()) => format!("{} — report {} saved: {} (finish it in the browser and drag the zip in)", diag.ui_line(), saved.id, saved.zip.display()),
                    Err(e) => format!("{} — report saved: {} (the browser didn't open: {e})", diag.ui_line(), saved.zip.display()),
                }
            }
            Err(e) => format!("{} — the report failed: {e}", diag.ui_line()),
        };
        eprintln!("[net] {line}");
        self.desync_report = Some(line.clone());
        self.net_fail(&format!("State desync at tick {}\n{line}\nReturning to the lobby", diag.tick));
    }

    /// A desync / a lost host / a refused join: the game's message box (the front end's option
    /// box with the port-only string) and the lobby page underneath it; the session is closed.
    ///
    /// mp-online-failures: it lands on the ONLINE page this machine came from (the host's port
    /// page / the client's address page, `net_rejoin`) — the couch Join Game page it used to
    /// open had no way back online. Out of a running match the front end is rebuilt the
    /// debriefing's way (`mp_results`: the music statics, the profile session, the runner flow,
    /// the front-end map requested under the fade) so the arena unloads instead of staying up
    /// frozen behind the page.
    /// net-desync-resync: the draw code's "Resyncing..." read — true while this
    /// machine is recovering from a state-hash mismatch (it asked the host to
    /// re-serve the match state; the machine replays the host's log headless
    /// and rejoins at the re-announce). The draw code renders the note; the
    /// text lives beside it so draw and tests share one string.
    pub fn net_resyncing(&self) -> bool {
        self.net_resyncing
    }

    /// The resync note's text (one string for the draw code and the tests).
    pub const NET_RESYNC_TEXT: &str = "Resyncing\u{2026}";

    fn net_fail(&mut self, msg: &str) {
        eprintln!("[net] {msg}");
        if let Some(se) = self.net.as_mut() { se.0.borrow_mut().close(); }
        self.net = None;
        self.net_end_wait = None;
        self.net_lobby = None;
        self.net_menu_load = false;
        self.net_late_self = None;
        self.net_armed = None;
        self.net_join_pending = None;
        self.net_late_handover = None;
        self.net_await_load = false;
        self.net_resyncing = false; // net-desync-resync: the note dies with the match
        self.desync_wait = None;
        let rejoin = self.net_rejoin.take();
        let in_match = self.scene.sim.is_some() && self.level_code != 0 && self.level_code != menu::FRONTEND_LEVEL;
        let mut fe = if in_match {
            let mut fe = menu::FrontEnd::new();
            if let Some(old) = self.fe.take() {
                self.profile_session = menu::ProfileSession::from_fe(&old);
                Self::inherit_music_statics(&mut fe, old.music.bank_flip, old.music.loaded_bank, old.music.music_master);
            } else {
                Self::inherit_music_statics(&mut fe, self.music_bank_flip, self.music_loaded_bank, self.music_master);
            }
            fe
        } else {
            self.fe.take().unwrap_or_else(menu::FrontEnd::new)
        };
        // settings-persist: the option file's values, not the bare defaults.
        let (opts, warn) = settings::load_default();
        if let Some(w) = warn { eprintln!("[settings] {w}"); }
        fe.opts = opts;
        // profile-session: keep the standing codename session (the other re-creates' rule).
        self.inherit_profile_session(&mut fe);
        // net-desync-detail: the fail box carries the diagnosis + the report zip's path (the
        // draw code renders it; the field is also the App's `desync_report` mirror).
        fe.desync_report = self.desync_report.clone();
        fe.mp_net_return(rejoin, msg);
        if in_match {
            // `mp_results`' order: the runner flow, the front-end map request, the fade on top.
            fe.flow.stack = vec![menu::ST_RUNNER];
            fe.flow.current_level = menu::FRONTEND_LEVEL;
            fe.requested_level = Some(menu::FRONTEND_LEVEL);
            fe.flow.reset_map_level_to_load(menu::FRONTEND_LEVEL, false, false);
            fe.flow.push_state(7, 84);
            // The load tail re-opens the queued page (`reopen_queued_page`): queue the online page.
            fe.flow.page = fe.page.id();
        }
        self.fe = Some(fe);
        // The online page needs its fresh session now (the first press may be Start/Connect).
        self.ensure_lobby_session();
        if self.scene.sim.is_some() { self.grab(false); }
    }

    /// The menu's 640×480 space for a window pixel: the menus draw as a centred 4:3 picture (Anchor::Centre).
    fn menu_space(&self, px: f64, py: f64) -> Option<(f32, f32)> {
        let g = self.gfx.as_ref()?;
        let (w, h) = (g.config.width as f32, g.config.height.max(1) as f32);
        let (bw, bh) = if w / h >= 4.0 / 3.0 { (h * 4.0 / 3.0, h) } else { (w, w * 3.0 / 4.0) };
        let (bx, by) = ((w - bw) / 2.0, (h - bh) / 2.0);
        Some(((px as f32 - bx) / bw * 640.0, (py as f32 - by) / bh * 480.0))
    }

    /// Draws the level's loading screen and presents it [M ResetMap_Load 0x1d6360 state 3 →
    /// ResetMap_GenLoadScreen 0x1d5de0, skipped for the front end 0x07000048]. async-level-load:
    /// called once with 0 dots before the worker spawns, then once per frame by
    /// `poll_async_load` with `loading::dots_lit` over the load's elapsed vblanks — the game's
    /// `VBlankInt` 0x124088 animates the same six dots while the main thread is inside the load.
    fn draw_loading_screen(&mut self, level: u32, dots: u32) {
        if level & 0xff00_ffff == 0x0700_0048 { return; }
        if self.loading_art.is_none() {
            self.loading_art = Some(read_files_bin(&self.iso, loading::ARCHIVE).map(|b| loading::art(&b)).unwrap_or_default());
        }
        let Some(g) = self.gfx.as_mut() else { return };
        if g.minimized { return; }
        let level = if level & 0xff00_ffff == 0x0700_0018 { 0x0700_001b } else { level };
        let picture = self.loading_art.as_ref().and_then(|a| a.get(&loading::image_for(level)));
        let lang = self.scene.sim.as_ref().and_then(|s| s.lang.as_ref()).or(self.lang.as_ref());
        let aspect = g.config.width as f32 / g.config.height.max(1) as f32;
        // One pick per load (the game builds the load screen once [M ResetMap_GenLoadScreen 0x1d5de0]):
        // the port re-picked every redraw with a clock seed, so the hint line flickered through the
        // pool many times a second.
        // One pick per load (`loading::HintPick` [M ResetMap_GenLoadScreen 0x1d5de0]): the port re-picked
        // every redraw with a clock seed, so the hint line flickered through the pool.
        let mut seed = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.subsec_nanos()).unwrap_or(1) | 1;
        let hint = self.loading_hint.get(level, || { seed ^= seed << 13; seed ^= seed >> 17; seed ^= seed << 5; seed >> 8 });
        let hud = match self.scene.hud.as_ref() {
            Some(t) => menu::loading_overlay(&t.font, lang, &self.scene.hud_textures, aspect, level, hint, dots),
            None => gpu::HudFrame::default(),
        };
        let frame = match g.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(f) | wgpu::CurrentSurfaceTexture::Suboptimal(f) => f,
            _ => return,
        };
        let view = frame.texture.create_view(&Default::default());
        g.renderer.screen_aspect = aspect;
        // A zero view-projection collapses every world vertex (w = 0), so only the picture and the overlay draw.
        let id = [[0.0f32; 4]; 4];
        let movie = picture.map(|(w, h, rgba)| (*w, *h, rgba.as_slice()));
        let enc = g.renderer.render(&g.device, &g.queue, &view, &g.depth, id, id, [0.0; 3], 0, &[], &hud, movie, None);
        g.queue.submit([enc.finish()]);
        g.queue.present(frame);
    }

    /// The level transition's first half (async-level-load): capture the carry-over exactly as
    /// the synchronous path did, draw the screen's first frame and hand the build to a worker.
    fn start_async_load(&mut self, next: u32) {
        self.loading_hint.reset(); // a new load picks its own hint
        // `Trigger_Activate`'s exit order: the success/fallback channel write (already applied in
        // the sim), then `Player_RamSave` [M 0x1b2318:97-116] — capture the old sim now, before
        // the scene swap drops it (fix-level2 F2). The old sim stays in `self.scene` until the
        // swap, so the worker's `save` snapshot and the loading screen's font/picture reads see
        // the same state the synchronous path saw.
        // fix-restart-target: the fail/death menu's Restart (`P_ENDMISSION_Handler`
        // 0x216618:19-21, menu 0x40000042's row 0) is a WARM reload of the CURRENT level on
        // the ENTRY checkpoint: `ResetMap_LevelToLoad(GameState._12_4_, 1, 1)` +
        // `Player_RamLoad(Bond, 1)`. The game does NOT capture the dying state — it re-serves
        // the RamSave buffer as of level entry (re-saved with the post-load state by
        // `Player_InitWeapon` [M 0x1a7ef8:325]), whose ch 99/100 predate the fail, so the
        // reload cannot re-fail (the 07000007 replay loop). The boost arg 1 floors the health
        // at the difficulty's `ContinueHealthBoost*` — 50.0 on all three [L ELF 0x30cc34-3c].
        // The PAUSE menu's Restart stays the COLD arm to BaseMap (`C_GCPAUSE_Handler`
        // 0x204408:139 — `level_exit_cold`, save = None).
        let cold = self.scene.sim.as_ref().is_some_and(|sim| sim.drones.level_exit_cold);
        let restart = self.scene.sim.as_ref().is_some_and(|sim| sim.drones.level_exit_restart);
        let save = if cold { None } else if restart {
            // The ENTRY checkpoint (+ the health floor) — `fail_restart_save` carries the
            // evidence (`P_ENDMISSION_Handler` 0x216618:19-21 + `Player_RamLoad`'s floored
            // buffer). fix-restart-health: the entry save is the POST-load capture the
            // load arm makes (`Player_InitWeapon`'s unconditional `Player_RamSave`
            // [M 0x1a7ef8:325]) — after a cold load it holds health 100, full health
            // (the old zero-buffer fallback served 50.0).
            Some(fail_restart_save(self.entry_save.as_ref(), self.scene.sim.as_ref().unwrap()))
        } else {
            Some(warm_exit_save(self.scene.sim.as_ref().unwrap()))
        };
        let name = level_archive_name(next);
        eprintln!("level: loading {name} ({})", if save.is_some() { "warm" } else { "cold" });
        // drive-ptp: the DRIVING return (`Boot_LoadPTPData` 0x1c8560) rides the reload —
        // taken from the OLD sim (the handover hook stashed it, scene.rs) and applied to
        // the NEW one at the swap: the mission status (`Mission_SetStatus` 6/3/2 from PTP
        // +0x978) and the 12 `PlrStats` driving slots (`PlrStats_SetDrivingData`
        // 0x1c8560's call). The resume level itself is the return's 0x07000048 [M tail].
        let ptp_return = self.scene.sim.as_ref().and_then(|s| s.drones.driving_return);
        self.draw_loading_screen(next, 0);
        let iso = self.iso.clone();
        let all = self.all;
        let dev = self.dev.is_some();
        // net-match-host: an online match's handover loads the MP core from the negotiated
        // settings (the same `load_mp_carry` on every machine — the seed rides inside). The local
        // menu confirm's settings ride here too now (they were set but never read before).
        // difficulty-effects: otherwise the front-end pick rides every front-end load [M the game's
        // level load reads GameState+0x28; the direct-load wrappers keep the replay 1].
        let mp_settings = self.pending_mp_settings.take();
        let difficulty = self.selected_difficulty;
        // pickup-medal-gate: the profile's `bonus[0]` word rides every front-end load — the game's
        // `bonus[]` global is ONE store that survives every map load (only `LS_LoadBonus` 0x1ce488
        // and the award path write it [M Menu_SetBonus 0x1ff898 callers]), so the kind-6 gate
        // [M 0x18dda0:93-95] reads the save's medals on every level. No profile → the fresh-boot
        // zero word [L ee_start_05 0x2e1b88].
        let bonus = self.fe.as_ref().map_or(0, |fe| fe.bonus_bits());
        let job = async_load::Job::spawn(move || {
            if let Some(st) = mp_settings {
                load_mp_carry(&iso, &name, all, save.as_ref(), st)
            } else if dev { load_with_carry_dev(&iso, &name, all, save.as_ref()) }
            else { load_with_carry_medals(&iso, &name, all, save.as_ref(), difficulty, bonus) }
                .map_err(|e| e.to_string())
        });
        self.async_load = Some(LevelLoad { job, next, ptp_return, ready: None });
    }

    /// One frame of the running load (async-level-load): `false` = the worker still runs — the
    /// loading screen was redrawn with the dots advanced (`loading::dots_lit` over the elapsed
    /// 60 Hz vblanks, the window's redraws standing in for `VBlankInt` 0x124088's), the caller
    /// ends the frame and asks for the next; `true` = nothing running or the result arrived and
    /// the swap is done, the caller falls through to normal play.
    fn poll_async_load(&mut self) -> bool {
        let Some(load) = self.async_load.as_mut() else { return true; };
        if load.ready.is_none() { load.ready = load.job.poll(); }
        // A successful build into a level with a loading screen waits out LOADING_HOLD (the
        // front-end map draws no loading screen [M ResetMap_GenLoadScreen skipped for 0x07000048];
        // a failure goes at once).
        let hold = load.next != menu::FRONTEND_LEVEL
            && load.job.elapsed_ticks() < (LOADING_HOLD * 60.0) as u32
            && !matches!(load.ready, Some(Some(Err(_))) | Some(None));
        if load.ready.is_none() || hold {
            let (level, dots) = (load.next, loading::dots_lit(load.job.elapsed_ticks()));
            self.draw_loading_screen(level, dots);
            if let Some(g) = &self.gfx { g.window.request_redraw(); }
            return false;
        }
        // The job is over whatever the result: take it and let the frame finish normally.
        let mut load = self.async_load.take().unwrap();
        let res = load.ready.take().unwrap();
        match res {
            Some(Ok(scene)) => {
                let name = level_archive_name(load.next);
                self.swap_in_scene(scene, load.ptp_return);
                // The level now running (GameState+0xc [M ResetMap_Load phase 2]). It was only ever
                // set from the command line, so a mission reached through the menus still read as the
                // front end: Esc found "no mission", released the mouse, and the next Esc quit the
                // game.
                self.level_code = load.next;
                // A real level loaded from the front end: the menu manager is gone [M the MP
                // confirm's / the SP pick's MenuManager_Delete — "the host closing the overlay once
                // the load leaves the front-end map"]. The MP start loads its arena directly (no
                // briefing hand-over), and the front end stayed open over it — the arena never
                // showed, only the menu's backdrop.
                if load.next != menu::FRONTEND_LEVEL
                    && self.fe.as_ref().is_some_and(|fe| fe.flow.get_state() != menu::ST_MOVIE)
                {
                    self.stash_music_statics();
                    self.stash_profile_session();
                    self.fe = None;
                }
                // The mission takes the mouse on arrival. The briefing path closed its front end at
                // the phase-2 resolve, BEFORE the level existed (close_front_end's grab saw no sim),
                // so the level came up un-grabbed: no mouse look until a click, and Esc quit.
                if load.next != menu::FRONTEND_LEVEL && self.fe.is_none() && self.scene.sim.is_some() { self.grab(true); }
                // frontend-music: `Sound_Ready`'s front-end case [M 0x1dceb8 case 0x7000048] — the
                // load alternates bank 0xc (SBI slot 0 = SB_0, jingle 0x1d7) and 0x38 (slot 1 =
                // SB_1, jingle 0x470) through DAT_002e1b11, then `MenuManager_Create` resets
                // `menu_movie` [M 0x21b908:125] so the first page enter restarts the pair. The
                // port keeps every bank resident, so the flip re-points the driver's primary slot.
                if load.next == menu::FRONTEND_LEVEL {
                    if let Some(fe) = self.fe.as_mut() {
                        let hash = fe.music.front_end_sound_ready();
                        fe.music.jingles_up = false;
                        if let Some(s) = self.sound.as_mut() { s.driver.set_primary(if hash == 0x38 { 1 } else { 0 }); }
                    }
                    self.fe_music_page = None; // the manager re-create re-runs the page's 0x4c [M 0x21b908:125]
                    // The load's own MenuManager_Create re-opens the queued page [M 0x1d6360:160-163]:
                    // the quit arms' 0x4000001c lands on the mission list, not the placeholder page
                    // they left up during the load (pause-results-flows).
                    if let Some(fe) = self.fe.as_mut() { fe.reopen_queued_page(); }
                }
                eprintln!("level: loaded {name} (loading screen {:.1} s)", load.job.elapsed_ticks() as f64 / 60.0);
            }
            Some(Err(e)) => eprintln!("level: failed to load {}: {e}", level_archive_name(load.next)),
            None => eprintln!("level: load of {} panicked (worker thread died)", level_archive_name(load.next)),
        }
        // The lock channel's edge memory starts clean on the new level (the game's `_prev`
        // comes back from the RamSave; 0x48 has no writer before the party). The sync path
        // ran this on both the Ok and Err arms — every terminal branch here does too.
        self.prev_lock_ch = false;
        true
    }

    /// The load's main-thread half (async-level-load): the sound stack, the camera, the PTP
    /// return, the renderer rebuild — GPU upload, this thread owns the device — and the entry
    /// checkpoint, in the exact order the synchronous path ran.
    fn swap_in_scene(&mut self, mut scene: scene::Scene, ptp_return: Option<nf_game::ptp::DriveReturn>) {
        // The previous scene's skin palette and its interpolation cameras belong to the OLD skin
        // layout: uploading them into the new renderer overran its palette buffer every frame
        // until a tick rebuilt them — and a level that doesn't tick at once (an MP arena with the
        // menu still up) never did (787 GPU validation errors).
        self.pal_cur = None;
        self.pal_prev = None;
        self.cam_prev = None;
        self.cam_cur = None;
        self.sound = build_sound(&mut scene);
        // The fresh sound stack is unpaused: reset the mirror so the per-frame diff
        // re-applies the current hold (a menu can still be up over the reload).
        self.audio_paused = false;
        // The per-boot text globals (DoorMsg / LoadMsg) survive the level swap [M .sdata].
        if let (Some(old), Some(new)) = (self.scene.sim.as_ref(), scene.sim.as_mut()) { new.hud.carry_boot_globals(&old.hud); }
        if let Some(start) = scene.start { self.cam = Camera { pos: Vec3::new(start.0[0], start.0[1] + 1.6, start.0[2]), yaw: start.1, pitch: 0.0, fov: GAME_FOV }; }
        self.scene = scene;
        self.anim_time = 0.0;
        // net-p2p two-window check (2026-10-18): the online match's load spawns every human in slot
        // order with slot 0's body in `sim.bond`; a joiner must move into ITS slot's body
        // (`Sim::swap_local_slot` existed but nothing called it — the joiner played slot 0's spawn
        // and had no body for the host, a desync at tick 0). The camera follows the swapped body.
        if self.net.is_some() && self.net_await_load {
            self.net_await_load = false;
            self.net_tick = 0;
            self.net_clock = Some(std::time::Instant::now());
            self.net_pacer.reset();
            // The first `input_delay` ticks have no sampled input (nothing ran that many ticks
            // before them): every machine submits NEUTRAL for its own slot there, so tick 0 is ready
            // at once instead of each window waiting on itself (the two-window freeze, 2026-10-18).
            if let Some(n) = self.net.as_ref() {
                let lead = n.0.borrow().input_delay();
                for t in 0..lead { n.0.borrow_mut().local_input(t, nf_net::NEUTRAL); }
            }
            // 700v6: the local tick's private generator (seeded per machine; nothing hashed reads it).
            let slot = self.net_local_slot as u32;
            if let Some(sim) = self.scene.sim.as_mut() {
                sim.net_rng = Some(nf_game::drone::Rng::new(0x5eed_0b0b ^ slot.wrapping_mul(0x9e37_79b9)));
            }
        }
        if self.net.is_some() && self.net_local_slot != 0 {
            if let Some(sim) = self.scene.sim.as_mut() {
                sim.swap_local_slot(self.net_local_slot);
                // The joiner's own slot's loadout (the scenario bonus weapon keys on its team).
                // mp-respawn-fists-draw: the equip leaves the fists 0x47 held with the spawn
                // weapon pending [M 0x1855f8:50-53] — the draw runs on the joiner's own machine
                // like every other spawn's (the old `switch_request = gun.weapon` line cancelled
                // that pending draw and left the joiner on the fists for good).
                sim.mp_equip_local();
                self.cam = Camera { pos: Vec3::new(sim.bond.pos[0], sim.bond.pos[1] + 1.6, sim.bond.pos[2]), yaw: sim.bond.yaw, pitch: 0.0, fov: GAME_FOV };
            }
        }
        // Render-check aid (NF_MP_FACE_OTHER=1): an online match starts facing the first other human
        // body (the two-window check's shots — the arena's spawns face away from each other).
        if self.net.is_some() {
            if let Some(sim) = self.scene.sim.as_mut() {
                if face_other_aid(sim) { self.cam.yaw = sim.bond.yaw; }
            }
        }
        // drive-ptp: `Boot_LoadPTPData` 0x1c8560's mirror runs on the returned
        // session — the mission status + the 12 slots, after the hub's own
        // `Mission_Init` (the game's `Mission_Init` skips 0x7000048 entirely
        // [M 0x182270:20-22], so the Boot-set status stands).
        if let Some(ret) = ptp_return {
            if let Some(sim) = self.scene.sim.as_mut() {
                ret.apply(&mut sim.drones.mission, &mut sim.drones.driving_stats, &mut sim.drones.progress.difficulty);
                // app-state-parity: `Boot_LoadPTPData` writes the block's +0x144 back to
                // GameState+0x28 on EVERY driving return [M 0x1c8560:35-36]; the tick code reads
                // the `Drones::difficulty` mirror, so it follows the Progress store here (the
                // same pair every `load_with_carry_difficulty` build seeds).
                sim.drones.difficulty = sim.drones.progress.difficulty;
                eprintln!("[ptp] return applied: MissionState {} (status byte {}), 12 slots carried, difficulty {}", sim.drones.mission.status(), ret.status_byte, sim.drones.difficulty);
            }
        }
        // mission-unlock-flow: the game's sp_level+0x10 flags are ONE global that survives every
        // map load; the port splits the store across the front end (FrontEnd::unlocked) and the
        // per-level sim (Progress::nightfire_status), so each fresh sim is seeded from the live
        // front end here — the results page's award (results_enter → set_level_bonus → the
        // mirror) then accumulates on the session's bits instead of restarting from the boot 0x3
        // [D ACTION.ELF .data 0x1d91e0]; Menu_GetNightfireStatus packs the 12 flag bytes at
        // sp_level+0x10 (0x18 stride) into one word [M 0x202360:9-19], Menu_SetLevelBonus writes
        // them [M 0x1ffa10], and no load path resets them — Mission_Init skips the front end
        // entirely [M 0x182270:19-22]. Without a front end (headless loads, boot) the fresh
        // boot value stands. The seed glue below is the port's mirror of that single store:
        // the front end holds the session's bits, every fresh sim is seeded from it.
        if let Some(fe) = self.fe.as_ref() {
            if let Some(sim) = self.scene.sim.as_mut() {
                sim.drones.progress.nightfire_status = fe.packed_unlocks();
            }
        }
        // The renderer holds the old scene's GPU resources (textures, buffers): rebuild it
        // IN PLACE — the window and surface must survive the level change (the game's window
        // does not close). Dropping `Gfx` here closed the window and nothing recreated it
        // (winit's `resumed` only runs at startup), so the process sat on a headless event
        // loop with the audio still playing.
        if let Some(g) = &mut self.gfx {
            g.renderer = gpu::Renderer::new(&g.device, &g.queue, g.config.format, &self.scene, self.notex, self.nofog);
        }
        // fix-restart-health: the loaded level's checkpoint is what a later
        // fail-Restart restores — the game re-saves the buffer with the POST-load
        // state on EVERY load (`Player_InitWeapon`'s unconditional `Player_RamSave`
        // [M 0x1a7ef8:325]), cold loads included (the `Player_RamLoad` body is
        // gated on `IsWarmReset` [M 0x1957a8:9], so `Player_Init`'s health 100 /
        // armour 0 [M 0x1910f8:128-138] stand and the arm's loadout/armour are
        // captured). The old model kept the save the level was LOADED with —
        // `None` after a cold load — and the restart then served the zeroed
        // buffer: health 50.0, half health.
        self.entry_save = Some(entry_checkpoint_save(self.scene.sim.as_ref().unwrap()));
    }

    /// F8/F9: show the mission-complete / mission-failed results page for the loaded mission —
    /// a port-only debug hook (the game reaches the page from `Mission_Update` 0x182c60:74's
    /// event, not from any key; the port's mission-event → results glue is separate
    /// host work).
    /// mp-match-end: the local match's end handover — the `MP_Update` 0x184bd0 case-4 sequence
    /// [M :62-78: `GameState+0x1C = 0x40000033` + `ResetMap_LevelToLoad(0x7000048)` + the fade +
    /// the per-slot `Player_SetCamMode`/`GS_PausePlayer`] on the front end the SP end built
    /// (`demo_results`'s shape): a fresh `FrontEnd` inheriting the music statics, the flow on the
    /// runner state, the front-end map requested, the debriefing page up with the board, and the
    /// MP menu state restored from the match's settings ([M `Menu_RestoreMPSettings` 0x202848 —
    /// the game's backup restore; the port restores at the handover because the manager that
    /// started the match was deleted). Confirm then returns to the scenario page (C_MPDBG
    /// 0x217860:8-11) and a second match starts from it through the existing flow.
    fn mp_results(&mut self, board: nf_game::mp::debrief::MpDebriefing, settings: nf_game::mp::MpSettings) {
        let mut fe = menu::FrontEnd::new();
        // The same music-statics inheritance `demo_results` runs (frontend-music): the match ran
        // with no front end, so the statics come from the last stash.
        if let Some(old) = self.fe.take() {
            self.profile_session = menu::ProfileSession::from_fe(&old);
            Self::inherit_music_statics(&mut fe, old.music.bank_flip, old.music.loaded_bank, old.music.music_master);
        } else {
            Self::inherit_music_statics(&mut fe, self.music_bank_flip, self.music_loaded_bank, self.music_master);
        }
        // profile-session: the fresh manager inherits the standing session (the game's globals
        // outlive the handover [M 0x2e1ae0/0x2e1ae8]).
        self.inherit_profile_session(&mut fe);
        // The game's handover deletes the menu manager with the match-start request; the fresh
        // manager boots into the runner state [M 0x1c98c8 case 1] — the flow must be live for the
        // page's own `reset_map_level_to_load`-shaped pushes.
        fe.flow.stack = vec![menu::ST_RUNNER];
        fe.flow.current_level = menu::FRONTEND_LEVEL;
        // The restore BEFORE the page open: C_MPDBG's Confirm must land on a scenario page that
        // already holds the match's choices (the game's manager-side restore is the same data).
        fe.restore_mp_match(&settings);
        // net-match-end: the board's fold, taken before the board moves into the page (the ack
        // and the host's end-wait comparisons fold it; docs/systems/net-match-end.md §4).
        let board_fold = nf_game::mp::debrief::board_hash(&board);
        let final_tick = self.scene.sim.as_ref().map(|s| s.drones.frame).unwrap_or(0);
        fe.show_mp_debriefing(board);
        // Online: the finished match's handover (net-match-end, docs/systems/net-match-end.md
        // §4 — port-only, the disc has no online [D action-symtab.tsv census: zero
        // socket/TCP/UPnP symbols]). Every machine announces its own end first — the final tick plus
        // its board's fold (Data tag 4) — before anything happens to its session. The HOST then
        // holds the LIVE session until every human peer has acked or departed (a client still
        // draining its last ticks keeps the relay; a dead peer can't hold it — the wall
        // deadline); a client (or a host with no human peers) runs the old tail at once.
        if self.net.is_some() {
            let bh = board_fold;
            if let Some(s) = self.net.as_ref() { s.0.borrow_mut().send_debrief_ack(final_tick, bh); }
            let is_host = self.net.as_ref().is_some_and(|s| s.is_host());
            let peers = self.net.as_ref().map(|s| s.0.borrow().human_peers()).unwrap_or_default();
            if is_host && !peers.is_empty() {
                self.net_end_wait = Some(NetEndWait {
                    peers,
                    acked: Vec::new(),
                    gone: Vec::new(),
                    board_hash: bh,
                    deadline: std::time::Instant::now() + std::time::Duration::from_secs(15),
                    board_mismatch: false,
                });
            } else {
                // The finished session stays open (polled, not ticked) while the debriefing is
                // up: closing it here told a peer still finishing its last ticks that the host
                // had left (the two-match check's "player 1 left" on the client). It closes when
                // the page goes.
                self.net_finished = self.net.take();
                // mp-online-failures: the finished match takes no new peer — a client that
                // pressed Continue first re-dials this port, and its HELLO must wait for the
                // host's next lobby, not late-join the match that just ended (it landed in a
                // dead lobby).
                if let Some(s) = self.net_finished.as_ref() { s.0.borrow_mut().finish_match(); }
            }
            self.net_lobby = None;
            self.net_menu_load = false;
            self.net_late_self = None;
            self.net_join_pending = None;
            fe.mp_online_rejoin = self.net_rejoin.take();
        }
        // The level request + the fade, the game's order [M 0x184bd0:64-78]: the front-end map
        // load pushes first, the fade state lands on top. The sim (the arena) freezes: the tick
        // loop runs only while `fe.is_none()` — the game's GS_PausePlayer per slot.
        fe.requested_level = Some(menu::FRONTEND_LEVEL);
        fe.flow.reset_map_level_to_load(menu::FRONTEND_LEVEL, false, false);
        fe.flow.push_state(7, 84); // the raw GameFlow id: PushState(7) becomes the fade hold [M 0x1c9f38]
        self.fe = Some(fe);
        self.grab(false);
    }

    fn demo_results(&mut self, complete: bool) {
        if self.level_code == 0 || self.level_code == menu::FRONTEND_LEVEL { return; }
        let mut fe = menu::FrontEnd::new();
        // frontend-music: `DAT_002e1b11` and `MusicMasterVolume` are statics, not per-manager —
        // the demo fe inherits the bank-flip state (the next front-end load must flip like the
        // game's), the resident bank and the music volume [M 0x1dceb8:94-101, 0x30c958]. The
        // stash outlives the dropped manager (menu-audio-state): a mission runs with no front
        // end at all, so the statics come from the last stash when the results open.
        if let Some(old) = self.fe.take() {
            // profile-session: the drop's stash — the codename session stands [M the globals].
            self.profile_session = menu::ProfileSession::from_fe(&old);
            Self::inherit_music_statics(&mut fe, old.music.bank_flip, old.music.loaded_bank, old.music.music_master);
        } else {
            Self::inherit_music_statics(&mut fe, self.music_bank_flip, self.music_loaded_bank, self.music_master);
        }
        // profile-session: the fresh manager inherits the standing session — without it the
        // results page's save gate (`1 < DAT_002e1ae0` [M 0x208e78:30/194]) never held and
        // finishing a mission under a codename could not save progress.
        self.inherit_profile_session(&mut fe);
        // settings-persist: this fresh front end boots the option file's values too — its
        // defaults would otherwise ride the results page's quit back into the front end.
        let (opts, warn) = settings::load_default();
        if let Some(w) = warn { eprintln!("[settings] {w}"); }
        fe.opts = opts;
        // The demo hook stands in for Mission_Update 0x182c60, which fires in state 2 — the flow
        // must be in a state reset_map_level_to_load accepts (its gate [M 0x1d3800]) for the outro
        // request to wrap into the movie state. The results PAGE opens only at the front-end
        // load's tail (`reopen_queued_page` — the game's MenuManager_Create [M 0x1d6360:160-163]),
        // i.e. after the outro movie for the 0x7300xx endings; the fields below build now (the
        // game's own commit runs at the page enter — the data lands before any page can read it).
        fe.flow.stack = vec![menu::ST_RUNNER];
        fe.flow.current_level = self.level_code;
        // mission-results-page: the results data rides the mission's tracked tallies — the game's
        // results page and its Stats sub-page read the live `PlrMissionStats` through GetScore's
        // row refill [M 0x1984b0:66-84; P_NFRESULTS 0x208e78:77, P_NFSTATS 0x209938:36]. The old
        // `menu::Stats::default()` zeroed the mission-end read.
        let end_stats = self.scene.sim.as_ref().map(|s| mission_end_stats(&s.drones.progress));
        fe.mission_ended(if complete { menu::Outcome::Complete } else { menu::Outcome::Failed },
            end_stats.unwrap_or_default());
        // The completed page's enter runs against the live progress layer [M P_NFRESULTS 0x208e78
        // :77 GetScore(0) + :169 Menu_SetLevelBonus] — the fields the overlay draws
        // (front-end-chain). GameState+0x48 is BaseMap [M 0x182c60:68-70's clamp].
        if complete {
            // The front end's language table isn't loaded while a mission runs — fall back to the
            // level's own (the medal name drew as a raw "<0x2c7>" on the live end sequence).
            let lang = self.lang.clone().or_else(|| self.scene.sim.as_ref().and_then(|s| s.lang.clone()));
            if let Some(sim) = self.scene.sim.as_mut() {
                let b = sim.drones.mission.base_map;
                let base = if b == u32::MAX || b == 0 { self.level_code } else { b };
                let diff = sim.drones.progress.difficulty;
                if fe.results_enter(&mut sim.drones.progress, base, 6, diff, lang.as_ref()) {
                    // profile-session: the game writes its MSSN/BNUS from the same live state
                    // the results page reads [M PlrStat_GetScore's DoneBetter commit rides the
                    // enter 0x1984b0:301-308; Menu_UpdateMessageBox case 1 then saves it] —
                    // merge the sim's tally into the profile BEFORE the save prompt can run.
                    fe.sync_profile_progress(&sim.drones.progress);
                }
                // mission-results-page: the sub-pages' data source. The sim dies with the
                // front-end load, but the game's PlrMissionStats/ScoringTable are BSS globals
                // the pages keep reading (the Stats page's own GetScore(0) [M 0x209938:36]; the
                // bonus page's Menu_GetLevelBonuses [M 0x209670:14]) until the next Mission_Init
                // wipes them [M 0x182270:65]. The snapshot stands in; the menu_after hook builds
                // the page the action opened through `FrontEnd::results_subpages_enter`.
                self.results_progress = Some(sim.drones.progress.clone());
            }
        }
        self.fe = Some(fe);
        self.grab(false);
    }

    /// frontend-music: the front end's audio, one call per redraw beside `movie_tick`. Runs the
    /// page-change transitions (the manager's msg-0x44 re-runs the new page's 0x4c arm [M
    /// 0x21c858:345-346]), applies the queued [`menu_music::MusicCmd`]s to the sound driver, and
    /// pumps the IOP stand-in while the menus are up (the sim path's `sound_tick` only runs in
    /// the tick gate, which the front end skips). The game runs the same shape: its frame update
    /// calls Sound_UpdateListeners → Sound_UpdateSounds every not-frozen frame, in the same block
    /// as MenuManager_Update [M Game_Run 0x1c9790:7-11]. Silent under NF_SILENT: without an output
    /// device `build_sound` returns None and the commands just bookkeep on the state machine.
    fn menu_music_tick(&mut self) {
        // frontend-music (menu-audio-state): the transitions + the drain live in the extracted
        // `fe_music_step` (menu_music.rs) so the headless flows drive the SAME glue the window
        // runs. This wrapper only applies the commands to the sound driver.
        let cmds = if self.fe.is_some() {
            let mut cmds = Vec::new();
            if let Some(fe) = self.fe.as_mut() {
                cmds = menu_music::fe_music_step(fe, &mut self.fe_music_page);
            }
            cmds
        } else { Vec::new() };
        if !cmds.is_empty() {
            // `StartJingle` only sounds on the front end: with a mission's bank as the primary the
            // game's own `SFXStart(0x1d7/0x470)` finds no effect (the mission banks lack both ids —
            // [D the bank probe: e.g. SB_12 carries 0x1d8-0x1da but neither jingle]), so the
            // results page over a mission stays silent exactly like the game.
            let front_end_loaded = self.level_code == 0 || self.level_code == menu::FRONTEND_LEVEL;
            if let Some(s) = self.sound.as_mut() {
                for c in cmds {
                    match c {
                        menu_music::MusicCmd::StopJingles => { s.driver.remove(0, Some(0x1d7)); s.driver.remove(0, Some(0x470)); }
                        menu_music::MusicCmd::StartJingle(id) => {
                            if front_end_loaded {
                                self.menu_handle += 1;
                                let h = self.menu_handle;
                                s.batch(|drv, out| { drv.start(out, id, None, h, None); });
                            }
                        }
                        menu_music::MusicCmd::StartMusic(track, section) => {
                            match menu_music_files(&self.iso, track) {
                                Some((t, smf, ssd)) => { s.music_files = Some((t, smf, ssd)); s.music_pending.push((0, track, section)); }
                                None => eprintln!("menu music: track {track:#x} files not on the disc — no credits music"),
                            }
                        }
                        menu_music::MusicCmd::StopMusic => { s.music = None; }
                        menu_music::MusicCmd::MusicVolume(v) => { s.driver.music_master = v; }
                        menu_music::MusicCmd::FadeDownMusic => { s.driver.fade_down(true); }
                    }
                }
            }
        }
        // The pump: one driver update + output send per frame while the menus are up and the
        // audio is not held (the pause page's hold mirrors `MenuManager_Create(pause_sound=1)`).
        if !self.audio_paused {
            if let Some(s) = self.sound.as_mut() { s.batch(|_, _| {}); }
        }
    }

    /// The ONE movie glue, one call per redraw. Every movie the game plays routes through the one
    /// `self.movie` player, by the game's own start sites: the P_INTRO pair while the run is up
    /// [M `P_INTRO_Handler` 0x209e88], the GameFlow state-6 briefing/outro [M 0x1c98c8 case 6 +
    /// `psiStartBackgroundMovie` 0x1081a8], and the page's LOOPING aux backdrop on every page
    /// change [M `Manager_SendMessage` 0x21c858:338 — stop when the new page's aux is 0, restart
    /// when it differs from the playing one]. Plus the `psiMovieFinished` poll [M 0x1082e8] and
    /// the phase-2 resolve routing the host's level load [M ResetMap_Load 0x1d6360 LoadState 2].
    fn movie_tick(&mut self) {
        let movie_iso = self.movie_iso_path();
        let Some(fe) = self.fe.as_mut() else {
            // No front-end page is up: no page wants the looping backdrop — stop it. The game's
            // page-change arm stops the aux when the new page's aux is 0 [M 0x21c858:334-338];
            // the front end's close is that arm with no page at all (menu-audio-leaks finding 1:
            // the backdrop — the menu music — played on through gameplay and the pause). A
            // non-looping movie (a briefing's tail) keeps playing to its end.
            if self.movie.as_ref().is_some_and(|p| p.is_looping()) {
                if let Some(p) = self.movie.take() { let c = p.code(); p.stop(); eprintln!("movie: stopped {c:08x} (no front-end page)"); }
            }
            return;
        };
        // A pending end_to request from mission_ended rides the same path a menu pick does.
        if let Some(code) = fe.requested_level.take() {
            // A cold menu reload (the pause page's RESTART [M 0x204408:139 ResetMap_LevelToLoad(.., 0, 1)])
            // must not carry the current state over: the sim's cold flag makes the switch load fresh.
            if !fe.flow.warm_reset { if let Some(sim) = self.scene.sim.as_mut() { sim.drones.level_exit_cold = true; } }
            self.pending_level = Some(code);
            // net-match-host: the settings ride with the level (same as menu_after).
            if let Some(st) = fe.mp_match_settings.take() { self.pending_mp_settings = Some(st); }
        }
        if fe.intro_playing {
            // The intro run (P_INTRO's boot logos / P_ESTHERO's START movie) owns the screen; its
            // movies are menu background movies on the ONE BGFMV slot [M psiStartBackgroundMovie
            // 0x1081a8 restarts the slot; 0x209e88:62, 0x20a060:24]. The page's 0x50 decides when
            // one starts (`fe.intro_movie`); the host plays it and feeds psiMovieFinished back.
            let code = fe.intro_movie_code();
            if fe.intro_movie {
                let wrong = match &self.movie { Some(p) => p.code() != code || p.is_looping(), None => true };
                if wrong {
                    if let Some(p) = self.movie.take() { p.stop(); }
                    match movie::Player::start(&movie_iso, code, movie_sfx_volume(fe)) {
                        Ok(p) => {
                            self.movie = Some(p); eprintln!("movie: playing {code:08x} (intro)");
                            // frontend-music: the page's movie start removed the pair (menu_movie
                            // = 0) [M 0x209e88:42-44, 0x20a060:20-22]; the movie's own audio
                            // replaces it.
                            fe.music.movie_started();
                        }
                        Err(e) => eprintln!("movie: {code:08x} not started: {e:?} — the headless countdown carries the intro"),
                    }
                }
                // psiMovieFinished [M 0x1082e8, polled by the page's 0x50]: while the real movie
                // decodes the headless budget is held; its end is the finish. The no-decode
                // player's BGFMVPlaying stand-in advances one frame per poll ([`Player::
                // headless_advance`] — the opened PSS holds for its real length [M 0x1081a8 +
                // 0x108060]).
                match &mut self.movie {
                    Some(p) if p.code() == code && !p.is_looping() => {
                        p.headless_advance();
                        if p.finished() { fe.intro_movie_finished(); } else { fe.intro_live = true; }
                    }
                    _ => {}
                }
            } else if self.movie.as_ref().is_some_and(|p| !p.is_looping() && p.code() == code) {
                // The movie was skipped (Menu_StopMovie → psiStopBackgroundMovie [M 0x215a50:11])
                // or it ended: free the slot.
                if let Some(p) = self.movie.take() { p.stop(); }
            } else if self.movie.as_ref().is_some_and(|p| p.is_looping()) {
                // The title's / a page's looping backdrop yields to the run (P_INTRO / P_PARISENUM /
                // P_ESTHERO carry no aux [M 0x21c858:334-338 stops it on the page change]).
                if let Some(p) = self.movie.take() { p.stop(); }
            }
        } else if fe.attract_playing {
            // attract-mode: P_ATTRACT's 0x4c starts the attract movie on the ONE BGFMV slot
            // [M psiStartBackgroundMovie 0x1081a8 — it replaces the playing backdrop; the page
            // aux loop resumes on the pop — 0x21c858:338]. The host polls psiMovieFinished
            // [M 0x21c3d8:37] and runs the 0x5f pop through `attract_movie_finished` when it
            // fires; a declined start reports finished at once [M `BGFMVPlaying == 0` — the
            // game's un-started movie — so the headless attract ends immediately].
            let code = fe.attract_movie;
            let wrong = match &self.movie { Some(p) => p.code() != code || p.is_looping(), None => true };
            if wrong {
                if let Some(p) = self.movie.take() { p.stop(); }
                match movie::Player::start(&movie_iso, code, movie_sfx_volume(fe)) {
                    Ok(p) => {
                        self.movie = Some(p); eprintln!("movie: playing {code:08x} (attract)");
                        // frontend-music: P_ATTRACT's 0x4c removes the menu-music pair before the
                        // movie (menu_movie = 0) [M 0x20a1f0:20-22] — the trailer's own soundtrack
                        // replaces it. Missing here, the menu music kept playing over the trailer
                        // (the attract movie has its own audio).
                        fe.music.movie_started();
                    }
                    Err(e) => {
                        // A declined start = the game's un-started movie: BGFMVPlaying == 0, so
                        // psiMovieFinished is true from tick one and the page closes at once
                        // [M 0x1082e8 + the 0x50 finish path 0x20a1f0:39-65].
                        eprintln!("movie: {code:08x} not started: {e:?} — the attract closes at once");
                        fe.attract_movie_finished();
                    }
                }
            }
            // psiMovieFinished [M 0x1082e8]: the poll advances the no-decode stand-in one frame
            // per call ([`Player::headless_advance`]).
            let ended = match self.movie.as_mut() {
                Some(p) if p.code() == code => { p.headless_advance(); p.finished() }
                _ => false,
            };
            if ended {
                // The 0x50 finish arm restarts the pair while it is down [M 0x20a1f0:34-39].
                fe.music.movie_finished();
                fe.attract_movie_finished();
            }
        } else if mission_movie_glue(fe, &mut self.movie, &movie_iso) {
        } else {
            // The page's aux loop [M 0x21c858:338]: stop when the page has no aux, (re)start when
            // the playing code differs from the page's.
            let aux = menu::aux_movie_for(fe.page_id());
            // A skipped attract (the Enter skip ends it inside the FrontEnd, not through the
            // finished poll above) leaves the trailer's player here: its 0x50 arm restarts the
            // menu-music pair the same way the natural end does [M 0x20a1f0:34-39].
            if self.movie.as_ref().is_some_and(|p| !p.is_looping() && matches!(p.code(), menu::ATTRACT_MOVIE_A | menu::ATTRACT_MOVIE_B)) {
                fe.music.movie_finished();
            }
            let wrong = match &self.movie {
                Some(p) => aux == 0 || p.code() != aux || !p.is_looping(),
                None => aux != 0,
            };
            if wrong {
                if let Some(p) = self.movie.take() {
                    let code = p.code();
                    p.stop();
                    eprintln!("movie: stopped {code:08x}");
                }
                if aux != 0 {
                    match movie::Player::start_looping(&movie_iso, aux, movie_sfx_volume(fe)) {
                        Ok(p) => { self.movie = Some(p); eprintln!("movie: looping {aux:08x} (page backdrop)"); }
                        Err(e) => eprintln!("movie: {aux:08x} not started: {e:?} — the dark plate stands in"),
                    }
                }
            }
        }
        // The phase-2 resolve: the movie code became the level to load [M 0x1d6360 LoadState 2].
        if let Some(resolved) = fe.flow.phase2_resolved.take() {
            self.pending_level = Some(resolved);
            if resolved != menu::FRONTEND_LEVEL {
                // The briefing's return: the real level loads and the front end closes — the game
                // leaves the menu system and Game_Draw takes over.
                self.close_front_end();
            }
            // The outro's return: the front end stays and the results page reopens on the map
            // (phase 2 set flow.page = 0x40000036 [M]; fe.page follows below in the hud build).
        }
    }

    fn grab(&mut self, on: bool) {
        let Some(g) = &self.gfx else { return };
        if on {
            let ok = g.window.set_cursor_grab(CursorGrabMode::Locked).or_else(|_| g.window.set_cursor_grab(CursorGrabMode::Confined)).is_ok();
            g.window.set_cursor_visible(!ok);
            self.grabbed = ok;
        } else {
            let _ = g.window.set_cursor_grab(CursorGrabMode::None);
            g.window.set_cursor_visible(true);
            self.grabbed = false;
        }
    }

    // -------------------------------------------------------------------------
    // fix-menu-movies2: the front end's background movies ride the ONE player/decoder path
    // (movie::Player over MovieStream's closed-GOP worker) and the ONE full-screen plane in
    // gpu.rs — the game plays them all through psiStartBackgroundMovie 0x001081a8 /
    // psiStopBackgroundMovie 0x00108328 (the page switch plays/stops the page's aux loop
    // [M Manager_SendMessage 0x21c858:338]; P_INTRO runs the intro pair [M 0x209e88]).
    // -------------------------------------------------------------------------

    /// The image the movies load from: NF_MOVIE_ISO (a full retail image) over the running one —
    /// the same precedence movie.rs's tests use [L the on-foot ISO's /MOVIES/30_FPS files are
    /// zero-length stubs].
    fn movie_iso_path(&self) -> String { std::env::var("NF_MOVIE_ISO").unwrap_or_else(|_| self.iso.clone()) }

    /// A click on the dev menu (F2 open): hit-test the mouse against the menu rows and apply the action (WORKLOG 0081,
    /// pages in fix-ingame-dev-menu). The hit-test shares the draw fn's layout (`dev::layout_rows`).
    fn dev_menu_click(&mut self) {
        let aspect = self.gfx.as_ref().map_or(1.0, |g| g.config.width as f32 / g.config.height as f32);
        let mouse = match self.dev.as_deref_mut() {
            None => return,
            Some(d) => d.mouse,
        };
        let Some(idx) = dev::menu_hit(self.dev.as_deref().unwrap(), Some(&self.scene), mouse.0, mouse.1, aspect) else { return };
        // B-9: a click also moves the keyboard selection to the clicked row.
        if let Some(d) = self.dev.as_deref_mut() { d.menu_sel = idx; }
        let Some(action) = dev::menu_action(self.dev.as_deref().unwrap(), Some(&self.scene), idx) else { return };
        self.apply_menu_action(action);
    }

    /// B-9: apply a dev-menu action and set the on-screen result message. Shared by mouse-click and keyboard
    /// (Enter) so both produce the same feedback. The result stays ~3 s (180 ticks).
    /// Apply the dev panel's queued commands and keep its catalog current (every 30 frames, and on level change).
    /// A numbered screenshot plus its spot: `nf-shots/NNN-<level>.png` and `.txt` holding the `--bond x,y,z,yaw,pitch` and
    /// level to re-render the view headless (F12 and the panel's "Screenshot + log spot").
    fn spot_shot(&mut self) -> String {
        let dir_buf = crate::settings::output_dir().join("nf-shots");
        let dir = dir_buf.as_path();
        if let Err(e) = std::fs::create_dir_all(dir) { return format!("screenshot: {e}"); }
        let level = self.scene.sim.as_ref().map_or(0, |s| s.level);
        let n = (1..10000).find(|n| !dir.join(format!("{n:03}-{level:08x}.png")).exists()).unwrap_or(0);
        let png = dir.join(format!("{n:03}-{level:08x}.png"));
        let spot = self.scene.sim.as_ref().map(|s| (s.bond.pos, s.bond.yaw, s.bond.pitch)).map(|(p, y, pt)| format!("--bond {:.2},{:.2},{:.2},{:.3},{:.3}", p[0], p[1], p[2], y, pt))
            .unwrap_or_else(|| { let p = self.cam.pos; format!("camera {:.2},{:.2},{:.2}", p.x, p.y, p.z) });
        let line = format!("{level:08x} {spot} frame {}", self.anim_time as u32);
        let _ = std::fs::write(png.with_extension("txt"), format!("{line}\n"));
        // mp-screens: capture what the window shows — the page's looping backdrop rides the ONE
        // movie plane and the redraw blanks/replaces the overlay by the same gates. spot_shot
        // dropped the frame (movie: None): with `fe.backdrop_live` standing (the live loop set
        // it) the overlay skips its dark plate, the movie plane draws nothing and a front-end
        // page shots the raw black void of the front-end level — an all-dark
        // capture (F12 on a menu page).
        match self.capture_png(&png) {
            Ok(()) => format!("saved {} ({line})", png.display()),
            Err(e) => format!("screenshot failed: {e} ({line})"),
        }
    }

    /// What the window shows, rendered to `png` (the spot shot's and the bug report's capture).
    fn capture_png(&mut self, png: &std::path::Path) -> Result<()> {
        let (movie_owned, overlay_blank) = self.spot_movie_frame();
        let fe_menu = if overlay_blank {
            // The intro/attract/briefing movie owns the screen — the redraw sends an empty HUD
            // frame there (the quad draws inside render(), before the HUD).
            Some(gpu::HudFrame::default())
        } else {
            self.fe.as_ref().and_then(|fe| self.scene.hud.as_ref().map(|t| {
                let lang = self.lang.as_ref().or_else(|| self.scene.sim.as_ref().and_then(|s| s.lang.as_ref()));
                menu::menu_overlay(fe, &t.font, &t.font_header, lang, &self.scene.hud_textures, 1280.0 / 720.0)
            }))
        };
        shot(&self.scene, &self.cam, &png.to_string_lossy(), self.choice, self.anim_time, self.opts, self.notex, self.nofog, fe_menu.as_ref(), movie_owned.as_ref())
    }

    /// Player bug report (F10, bugreport.rs): the frame, where the player is, the system, the session recording and
    /// the log go into a zip in the reports folder; the browser opens the GitHub bug form pre-filled and Explorer
    /// shows the zip to drag in.
    fn bug_report(&mut self) -> String {
        let id = bugreport::report_id();
        let mut files: Vec<(String, Vec<u8>)> = Vec::new();
        let tmp = std::env::temp_dir().join(format!("nf-report-{id}.png"));
        let shot_note = match self.capture_png(&tmp) {
            Ok(()) => match std::fs::read(&tmp) { Ok(b) => { files.push(("screenshot.png".into(), b)); "yes".to_string() } Err(e) => format!("unreadable ({e})") },
            Err(e) => format!("failed ({e})"),
        };
        let _ = std::fs::remove_file(&tmp);
        let (session_files, session_note) = bugreport::session_files();
        let gpu = bugreport::GPU_INFO.get().cloned().unwrap_or_default();
        let window = self.gfx.as_ref().map(|g| { let s = g.window.inner_size(); format!("{}x{} scale {:.2} fullscreen {}", s.width, s.height, g.window.scale_factor(), g.window.fullscreen().is_some()) }).unwrap_or_default();
        let (place, level_field) = match self.scene.sim.as_ref() {
            Some(sim) => {
                let objectives: Vec<String> = sim.drones.mission.states.iter().map(|v| format!("{v:?}")).collect();
                (format!("level: {:08x}\ntick: {}\nbond: pos {:.2?} yaw {:.3} pitch {:.3} substate {} health {:.1} armour {:.1}\nweapon: {:#04x} clip {} spare {}\nmission: internal state {} state {} base {:#010x} order {}\nobjective states: {}\nbond spot (for --bond): {:.2},{:.2},{:.2},{:.3},{:.3}",
                    sim.level, sim.drones.frame, sim.bond.pos, sim.bond.yaw, sim.bond.pitch, sim.bond.substate, sim.drones.health.health, sim.drones.health.armour,
                    sim.gun.weapon, sim.gun.clip, sim.gun.spare, sim.drones.mission.internal, sim.drones.mission.state, sim.drones.mission.base_map, sim.drones.mission.order,
                    objectives.join(" "), sim.bond.pos[0], sim.bond.pos[1], sim.bond.pos[2], sim.bond.yaw, sim.bond.pitch),
                 format!("{:08x}", sim.level))
            }
            None => ("level: (none — front end)".into(), "front end".into()),
        };
        let menu = self.fe.as_ref().map(|fe| format!("front end page: {:?} (flow state {})", fe.page, fe.flow.get_state())).unwrap_or_else(|| "front end: closed".into());
        let online = match &self.net {
            Some(n) => { let s = n.0.borrow(); format!("online: slot {} net tick {} agents {} reach {:?} lobby {}", self.net_local_slot, self.net_tick, s.agents().len(), s.reach(), self.net_lobby.is_some()) }
            None => "online: no".into(),
        };
        let mp = self.scene.sim.as_ref().and_then(|s| s.mp.as_ref()).map(|m| format!("mp match: scenario {:#x} level {:#010x} humans {} bots {} seed {:#x} state {}", m.settings.scenario, m.settings.level_code, m.settings.human_count, m.settings.bot_count, m.settings.seed, m.game.state)).unwrap_or_else(|| "mp match: no".into());
        let report = format!("kind: player report (F10)\nreport_id: {id}\nversion: {}\nos: {}\ngpu: {gpu}\nwindow: {window}\nfps: {:.0}\ndifficulty: {}\n{place}\n{menu}\n{online}\n{mp}\n{session_note}\nscreenshot: {shot_note}\n",
            bugreport::version_line(), bugreport::os_line(), self.fps_meter.shown, self.selected_difficulty);
        files.insert(0, ("report.txt".into(), bugreport::scrub(&report).into_bytes()));
        files.extend(session_files);
        if let Some(p) = settings::config_dir().map(|d| d.join("settings.toml")) {
            if let Ok(t) = std::fs::read_to_string(p) { files.push(("settings.toml".into(), bugreport::scrub(&t).into_bytes())); }
        }
        if let Some(t) = bugreport::log_tail(3000) { files.push(("game-log.txt".into(), t.into_bytes())); }
        let url = bugreport::issue_url(&format!("[Bug] {level_field}: "), &[
            ("version", bugreport::version_line()),
            ("level", level_field.clone()),
            ("report_id", id.clone()),
            ("details", bugreport::scrub(&format!("{}\n{gpu}\n{window}\n{place}", bugreport::os_line()))),
        ]);
        match bugreport::save(&id, files, url) {
            Ok(saved) => {
                // Hand the mouse back and open the form + the zip (the player finishes the report there).
                self.grab(false);
                let opened = bugreport::open_url(&saved.url);
                bugreport::reveal(&saved.zip);
                match opened {
                    Ok(()) => format!("bug report {} saved: {} — finish it in the browser and drag the zip in", saved.id, saved.zip.display()),
                    Err(e) => format!("bug report saved: {} (the browser didn't open: {e}; the form is {})", saved.zip.display(), saved.url),
                }
            }
            Err(e) => format!("bug report failed: {e}"),
        }
    }

    /// mp-screens: the spot shot's movie composition — the same gates the redraw uses
    /// (lib.rs's `movie_up`): the state-6 briefing, the intro pair, the attract movie and the
    /// page's looping aux backdrop, all through the ONE player. Returns the frame to draw on the
    /// movie plane (owned — the player's buffer borrows end here) and whether the overlay must be
    /// blank (the movie owns the screen). Public to the tests through `crate::spot_movie_gates`.
    fn spot_movie_frame(&mut self) -> (Option<(u32, u32, Vec<u8>)>, bool) {
        let state6 = self.fe.as_ref().is_some_and(|fe| fe.flow.get_state() == menu::ST_MOVIE);
        let intro_up = self.fe.as_ref().is_some_and(|fe| fe.intro_playing);
        let attract_up = self.fe.as_ref().is_some_and(|fe| fe.attract_playing);
        let looping = self.movie.as_ref().is_some_and(|p| p.is_looping());
        let pending = self.fe.as_ref().is_some_and(|fe| fe.flow.movie_pending);
        let (want_frame, blank) = spot_movie_gates(state6, intro_up, attract_up, looping, pending, self.movie.is_some());
        let frame = if want_frame { self.movie.as_mut().and_then(|p| p.present()).map(|(w, h, rgba)| (w, h, rgba.to_vec())) } else { None };
        if let Some(fe) = self.fe.as_mut() {
            // The redraw keeps `backdrop_live` in step with the live frame (the overlay's
            // dark-plate gate reads it — menu.rs's plate skip).
            if looping { fe.backdrop_live = frame.is_some(); }
        }
        // The overlay: blank when the movie owns the screen — the briefing always (the redraw's
        // flow_up arm), the intro/attract once a frame exists; the page backdrop draws under.
        let overlay_blank = match blank {
            Blank::Always => true,
            Blank::WithFrame => frame.is_some(),
            Blank::Never => false,
        };
        (frame, overlay_blank)
    }

    /// fix-weapon-assets: true when the weapon's first-person assets are absent (its `+0xDC` skin
    /// is in no merged archive) — the view model would not build and the view scripts would not
    /// resolve, on any weapon the level doesn't carry.
    fn weapon_needs_assets(&self, w: u8) -> bool {
        self.scene.sim.as_ref().is_some_and(|sim| {
            let fp = sim.weapons.first_person(w as usize);
            fp != 0 && !sim.skins.contains_key(&fp)
        })
    }

    fn poll_panel(&mut self) {
        let Some(h) = self.panel.clone() else { return };
        let queued: Vec<String> = h.lock().map(|mut g| std::mem::take(&mut g.queue)).unwrap_or_default();
        let mut changed = !queued.is_empty();
        for q in queued {
            let q = panel::query(&q);
            let what = q.iter().find(|(k, _)| k == "do").map(|(_, v)| v.clone()).unwrap_or_default();
            let status = match what.as_str() {
                "god" => match self.dev.as_deref_mut() { Some(d) => { d.god_mode = !d.god_mode; format!("god mode {}", if d.god_mode { "on" } else { "off" }) } None => "no dev mode".into() },
                "shot" => self.spot_shot(),
                "ammo" => match self.dev.as_deref_mut() { Some(d) => { d.infinite_ammo = !d.infinite_ammo; format!("infinite ammo {}", if d.infinite_ammo { "on" } else { "off" }) } None => "no dev mode".into() },
                _ => {
                    let (status, switch) = panel::apply(&mut self.scene, &q);
                    if let Some(w) = switch {
                        self.weapon_switch_to = Some(w);
                        // fix-weapon-assets: the level archive lacks the weapon — answer the give
                        // with the warm asset reload (the tick loop's pending_weapon_assets arm).
                        if self.weapon_needs_assets(w) { self.pending_weapon_assets = Some(w); }
                    }
                    status
                }
            };
            eprintln!("[panel] {status}");
            if let Ok(mut g) = h.lock() { g.status = status; }
        }
        let level = self.scene.sim.as_ref().map_or(0, |s| s.level);
        self.panel_frames += 1;
        if changed || level != self.panel_level || self.panel_frames % 30 == 0 {
            changed = true;
            self.panel_level = level;
        }
        if changed { let c = panel::catalog(&self.scene); if let Ok(mut g) = h.lock() { g.catalog = c; } }
    }

    /// B-9: apply a dev-menu action and set the on-screen result message. Shared by mouse-click and keyboard
    /// (Enter) so both produce the same feedback. The result stays ~3 s (180 ticks).
    ///
    /// fix-ingame-dev-menu: the whole action match moved into `dev::apply_menu_action_to_scene` — the REAL path,
    /// shared with the headless menu-flow tests (`tests/dev_menu_flow.rs`). This shell only applies what needs the
    /// windowed App: the weapon switch request, the pending level, the viewer cel reset, the noclip state and the
    /// screenshot. Every give/spawn/clear command still routes through `panel::apply` (the shared command layer).
    fn apply_menu_action(&mut self, action: dev::MenuAction) {
        let eye = [self.cam.pos.x, self.cam.pos.y, self.cam.pos.z];
        let (out, msg) = match self.dev.as_deref_mut() {
            None => return,
            Some(d) => {
                let out = dev::apply_menu_action_to_scene(d, &mut self.scene, action, eye, self.cam.yaw, self.noclip);
                let msg = out.status.clone();
                (out, msg)
            }
        };
        if let Some(w) = out.switch_to {
            self.weapon_switch_to = Some(w);
            // fix-weapon-assets: same as poll_panel — a give the level archive can't dress yet.
            if self.weapon_needs_assets(w) { self.pending_weapon_assets = Some(w); }
        }
        if let Some(id) = out.pending_level { self.pending_level = Some(id); }
        if let Some(on) = out.noclip_to { self.set_noclip(on); }
        if out.shot { let m = self.spot_shot(); eprintln!("[spot] {m}"); }
        if out.viewer_cel_reset {
            // The game's own placement clears the viewer's cel [M 0x1b9920] — the next
            // `Camera_CheckLocation` re-finds it at Bond's position.
            if let Some(g) = self.gfx.as_mut() { g.renderer.viewer_cel = crate::scene::ViewerCel::default(); }
        }
        // "spawn at crosshair" (`Dev::spawn_guard`) adds a Character; the renderer's skin layout was
        // built for the old count and `skin_palette` indexed past it (a panic on the next frame).
        // Rebuild the renderer like the weapon-asset reload does.
        if let Some(g) = self.gfx.as_mut() {
            if g.renderer.skin_layout.char_base.len() != self.scene.characters.len() {
                g.renderer = gpu::Renderer::new(&g.device, &g.queue, g.config.format, &self.scene, self.notex, self.nofog);
            }
        }
        if let Some(d) = self.dev.as_deref_mut() { d.result_msg = Some(msg); d.result_timer = 180; }
    }

    /// One dev-window event (devwin.rs): navigation/filter in the window, actions through the shared path.
    fn dev_window_event(&mut self, event: WindowEvent) {
        let Some(dpi) = self.devwin.as_ref().map(|w| w.window.scale_factor() as f32) else { return };
        match &event {
            WindowEvent::Resized(s) => {
                if let (Some(w), Some(g)) = (self.devwin.as_mut(), self.gfx.as_ref()) { w.resize(&g.device, s.width, s.height); }
                return;
            }
            WindowEvent::ScaleFactorChanged { .. } => { if let Some(w) = self.devwin.as_ref() { w.window.request_redraw(); } return; }
            WindowEvent::RedrawRequested => {
                if let (Some(w), Some(g), Some(d)) = (self.devwin.as_mut(), self.gfx.as_ref(), self.dev.as_deref()) { w.render(&g.device, &g.queue, &self.scene, d); }
                return;
            }
            _ => {}
        }
        let (Some(w), Some(d)) = (self.devwin.as_mut(), self.dev.as_deref_mut()) else { return };
        let size = w.window.inner_size();
        let rows = dev::menu_items(d, Some(&self.scene));
        let req = devwin::handle(&event, d, &mut w.ui, &rows, (size.width as f32, size.height as f32), dpi);
        match req {
            devwin::Request::None => {}
            devwin::Request::Redraw => w.window.request_redraw(),
            devwin::Request::Hide => w.window.set_visible(false),
            devwin::Request::Apply(a) => {
                w.window.request_redraw();
                self.apply_menu_action(a);
                if let Some(m) = self.dev.as_deref().and_then(|d| d.result_msg.clone()) { eprintln!("[dev] {m}"); }
            }
        }
    }

    /// The N key's noclip toggle, shared with the Toggles page row (fix-ingame-dev-menu).
    fn set_noclip(&mut self, on: bool) {
        self.noclip = on;
        // Back on foot: Bond where the free camera is.
        if !self.noclip {
            if let Some(sim) = self.scene.sim.as_mut() {
                sim.bond = nf_game::player::Bond::stand_at(&sim.world, [self.cam.pos.x, self.cam.pos.y - 0.7, self.cam.pos.z], self.cam.yaw);
            }
            // The game's own placement clears the viewer's cel [M 0x1b9920] — the
            // next frame re-finds at Bond's position.
            if let Some(g) = self.gfx.as_mut() { g.renderer.viewer_cel = crate::scene::ViewerCel::default(); }
        }
    }
    /// headless-app-harness: the RedrawRequested half that needs no window or GPU — the
    /// movie glue, the menu music, the pending-weapon reload, the level-load schedule, the
    /// async load poll, the net poll + events, the sound hold, the 60 Hz sim tick gate and
    /// the mission-end / debriefing handovers. `dt` = this frame's seconds (the window
    /// passes its wall clock, the headless harness 1/60). The draw half (`draw_frame`)
    /// runs after it; the window event loop calls both, in this order.
    fn frame(&mut self, dt: f32) {
        // pad-menus (results-stats-key): the pad drives the front end's arms — the game reads its
        // style-independent menu actions (0x1a-0x22) off the pad edges every frame and the widgets
        // send their messages on those edges [M psiInput_MapInputs 0x10cd80:377-424 +
        // Button_Update 0x1ef948:74-91; the mapping cites in menu::pad_menu_action], so the port
        // feeds the same edges through the keyboard's Action path. No pad (or the feature off):
        // nothing. `poll_menu` runs every frame while the backend exists so no edge leaks across
        // the menus/gameplay boundary; the action is consumed only when the front end owns input.
        #[cfg(feature = "gamepad")]
        if let Some(pad) = self.pad.as_mut() {
            let a = pad.poll_menu();
            if self.fe.is_some() {
                if let Some(a) = a { self.menu_input(a); }
            }
        }
                // fix-movie-playback: the movie glue runs before the load path so a phase-2
                // resolve lands in the same frame the state 6 popped.
                self.menu_script_tick();
                self.movie_tick();
                self.adopt_lobby_session();
                if self.net_finished.is_some() {
                    let on_debrief = self.fe.as_ref().is_some_and(|fe| fe.page == menu::Page::MpDebriefing);
                    if on_debrief {
                        if let Some(s) = self.net_finished.as_ref() { s.0.borrow_mut().poll(); }
                    } else if let Some(s) = self.net_finished.take() {
                        s.0.borrow_mut().close();
                    }
                }
                // net-match-end: the host's end wait — the live session keeps polling (the
                // relay feeds a client still draining its final ticks [M nf-net relay_horizon])
                // while the wait collects each peer's DebriefAck / departure; when every peer is
                // done (or the wall deadline passes — a dead peer can't hold the debriefing),
                // the session runs the normal finished tail. docs/systems/net-match-end.md §4.
                if self.net_end_wait.is_some() {
                    let mut events = Vec::new();
                    if let Some(s) = self.net.as_ref() {
                        let mut core = s.0.borrow_mut();
                        core.poll();
                        events = core.take_app_events();
                    }
                    // The notices the wait saw this frame (a differing board fold, a departure)
                    // surface on the debriefing's notice line — port net protocol, the disc has
                    // no online [D action-symtab.tsv census: zero socket/TCP/UPnP symbols].
                    let mut board_notice = false;
                    if let Some(w) = self.net_end_wait.as_mut() {
                        for ev in events {
                            match ev {
                                mpnet::CoreEvent::DebriefAck { slot, final_tick, board_hash } => {
                                    let s = slot as usize;
                                    if !w.acked.contains(&s) { w.acked.push(s); }
                                    if board_hash != w.board_hash {
                                        w.board_mismatch = true;
                                        board_notice = true;
                                        eprintln!("[net] slot {} reached its debriefing at tick {} — its board differs from ours", slot + 1, final_tick);
                                    } else {
                                        eprintln!("[net] slot {} reached its debriefing at tick {} — the boards agree", slot + 1, final_tick);
                                    }
                                }
                                mpnet::CoreEvent::ClientLeft { slot, graceful } => {
                                    let s = slot as usize;
                                    if !w.gone.contains(&s) { w.gone.push(s); }
                                    eprintln!("[net] player {} {} while the debriefing waits", slot + 1, if graceful { "left" } else { "timed out" });
                                    if let Some(fe) = self.fe.as_mut() { fe.net_notice = Some(format!("Player {} left the match.", slot + 1)); }
                                }
                                _ => {}
                            }
                        }
                    }
                    let done = self.net_end_wait.as_ref().is_some_and(|w| w.done() || std::time::Instant::now() >= w.deadline);
                    if done {
                        let mismatch = self.net_end_wait.as_ref().is_some_and(|w| w.board_mismatch);
                        let timed_out = self.net_end_wait.as_ref().is_some_and(|w| !w.done());
                        self.net_end_wait = None;
                        self.net_finished = self.net.take();
                        if let Some(s) = self.net_finished.as_ref() { s.0.borrow_mut().finish_match(); }
                        eprintln!("[net] the match's end handover complete (all peers at their debriefing{}{})",
                            if mismatch { "; a board differed" } else { "" },
                            if timed_out { "; the wait timed out" } else { "" });
                        if mismatch {
                            if let Some(fe) = self.fe.as_mut() { fe.net_notice = Some("A player's debriefing board differs from yours.".into()); }
                        }
                    } else if board_notice {
                        if let Some(fe) = self.fe.as_mut() { fe.net_notice = Some("A player's debriefing board differs from yours.".into()); }
                    }
                }
                // frontend-music: the menus' own audio (the jingle pair, the credits track), the
                // page-change transitions and the per-frame pump.
                self.menu_music_tick();
                // The Auto Aim option reaches the game (it was stored and never read), and
                // options-census generalizes the same drain to every wired row
                // (docs/systems/options-census.md).
                self.drain_options();
                // fix-weapon-assets: a dev-menu/panel give of a weapon the level archive lacks.
                // The game never gives cross-level weapons, so there is no game path to mirror —
                // the port answers the give with a warm reload of the SAME level with the weapon's
                // assets merged (`LoaderLoad` of the same archive + the merge), re-places Bond
                // where he stood and re-issues the switch. One hitch per newly-given weapon.
                if let Some(w) = self.pending_weapon_assets.take() {
                    if self.weapon_needs_assets(w) {
                        let (pos, yaw) = self.scene.sim.as_ref().map(|s| (s.bond.pos, s.bond.yaw)).unwrap_or(([0.0; 3], 0.0));
                        let save = self.scene.sim.as_ref().map(warm_exit_save);
                        self.entry_save = save.clone();
                        let name = level_archive_name(self.scene.sim.as_ref().map_or(0, |s| s.level));
                        eprintln!("weapon assets: {w:#04x} is not on this level — warm reload with its assets");
                        match load_with_carry_weapons_difficulty(&self.iso, &name, self.all, save.as_ref(), &[w], self.selected_difficulty) {
                            Ok(mut scene) => {
                                self.sound = build_sound(&mut scene);
                                if let Some(sim) = scene.sim.as_mut() {
                                    sim.bond = nf_game::player::Bond::stand_at(&sim.world, pos, yaw);
                                    sim.gun.switch_request = w;
                                }
                                if let (Some(old), Some(new)) = (self.scene.sim.as_ref(), scene.sim.as_mut()) { new.hud.carry_boot_globals(&old.hud); }
                                self.scene = scene;
                                if let Some(g) = &mut self.gfx {
                                    g.renderer = gpu::Renderer::new(&g.device, &g.queue, g.config.format, &self.scene, self.notex, self.nofog);
                                }
                                self.weapon_switch_to = Some(w);
                            }
                            Err(e) => eprintln!("weapon assets: reload failed: {e}"),
                        }
                    }
                }
                // Bug 5: level transition. A `LOAD_LEVEL` (0xE8) trigger set `pending_level`; reload the scene
                // with the next archive (`ResetMap_LevelToLoad` 0x1d3800 → `LoaderLoad` 0x1d4a00).
                if let Some(next) = self.pending_level.take() {
                    // A movie-wrapped load plays its movie FIRST: the game loads the empty backdrop
                    // 0x7000500, the briefing/outro runs over it, and only phase 3 loads the real
                    // level [M 0x1d6360 phases 0/3]. Hold the request while the flow is heading to
                    // state 6; the phase-2 resolve re-issues it (movie_tick).
                    if self.fe.as_ref().is_some_and(|fe| fe.flow.movie_pending) {
                        self.pending_level = Some(next);
                    } else {
                        // async-level-load: the build runs on a worker; the frames below redraw
                        // the loading screen with the dots until the scene is ready.
                        self.start_async_load(next);
                    }
                }
                // async-level-load: while a worker builds the next scene, this thread is the
                // game's main thread inside `ResetMap_Load` 0x1d6360 state 3 — nothing else of
                // the frame runs (the game's vblank interrupt alone animates the dots,
                // `VBlankInt` 0x124088). Each frame redraws the screen with `dots_lit` advanced
                // and ends; the frame the result arrives, `poll_async_load` swaps the scene in
                // (the renderer's GPU rebuild stays on this thread) and the frame falls through
                // to normal play.
                // The online session keeps pumping through the loading screen: the frame returns
                // early below, and a load longer than the session's 5 s timeout (a 5.2 s arena load
                // was measured) dropped the other player before tick 0. Its events wait in
                // the core until the load ends and the normal drain below runs.
                if self.async_load.is_some() {
                    if let Some(session) = self.net.as_mut() { session.0.borrow_mut().poll(); }
                }
                if self.async_load.is_some() && !self.poll_async_load() { return; }
                // The Toggles page's fps display (wall-clock; a dev readout, never in the Sim).
                if let Some(d) = self.dev.as_deref_mut() {
                    if dt > 0.0 { d.fps = if d.fps <= 0.0 { 1.0 / dt } else { d.fps * 0.9 + (1.0 / dt) * 0.1 }; }
                }
                let speed = if self.keys.contains(&KeyCode::ShiftLeft) { 40.0 } else { 8.0 } * dt;
                let fwd = self.cam.dir();
                let right = self.cam.right();
                let mut mv = Vec3::ZERO;
                for (k, d) in [(KeyCode::KeyW, fwd), (KeyCode::KeyS, -fwd), (KeyCode::KeyD, right), (KeyCode::KeyA, -right), (KeyCode::Space, Vec3::Y), (KeyCode::ControlLeft, -Vec3::Y)] {
                    if self.keys.contains(&k) { mv += d; }
                }
                if self.noclip || self.scene.sim.is_none() { self.cam.pos += mv * speed; }
                // One animation frame per 60 Hz game update (FRAME_RATE_MUL = 1), docs/formats/animation.md.
                self.ticked = false;
                self.poll_panel();
                // fix-pause-audio: the pause page holds the sound. `MenuManager_Create` with
                // `pause_sound` runs `SFXPause` (batch op 9) + `SFXPauseAllStreams` (op 0x16) and
                // `MenuManager_Delete` runs the unpause pair [M 0x21b908:121-124, 0x21c2b0:11-22];
                // GameState+0x50 is read by NO sound code — the pause rides these ops. The port's
                // hook is the pause page (the in-game pause manager's create) or the viewer's P
                // pause — NOT the front end's own pages: its manager passes pause_sound = 0 and
                // the jingle pair keeps playing over every page [M frontend-music].
                // `set_paused` reaches the output at once because `sound_tick` (and the
                // batch) only run inside the tick gate below. The fail/results page opens with
                // `pause_sound` = 0 in the game (sound keeps playing [M Mission_Update 0x182c60:78])
                // — the port's demo results hook rides the same gate [S, remaining host glue].
                // net-consolidate: pump the ONE session once per frame (receive, timeouts,
                // handshake, the armed GO). The events drive the same handover a menu confirm
                // drives; the lobby pages drive the same core through their `MpSession` proxy.
                if let Some(session) = self.net.as_mut() {
                    session.0.borrow_mut().poll();
                }
                // Drain first, then react: the borrow must be gone before the
                // handlers touch `self` again (net_fail replaces the session).
                let net_events = self.net.as_mut()
                    .map(|session| session.0.borrow_mut().take_app_events())
                    .unwrap_or_default();
                for ev in net_events {
                    match ev {
                            mpnet::CoreEvent::MatchStarted(st) => {
                                // Every machine takes the same handover: the level loads from the
                                // settings (the seed rides inside), the menu closes. The lobby
                                // pages also see this through `match_started()` (mp_net_poll),
                                // which mirrors it into the same fields — idempotent.
                                eprintln!("[net] match starting: arena {:08x}, {} humans, seed {:#x}",
                                    st.level_code, st.human_count, st.seed);
                                // net-desync-detail: a fresh match gets a fresh watcher (the old
                                // match's history and diagnosis do not carry over).
                                self.desync_watch = desync::Watcher::default();
                                self.desync_report = None;
                                self.desync_wait = None;
                                self.net_local_slot = self.net.as_ref().map_or(0, |s| s.0.borrow().local_slot());
                                // The lobby path already requested this arena (the menu handover
                                // + `adopt_lobby_session`): a second request reloaded it warm on
                                // both machines — two loading screens per match (the two-window
                                // menu check). The flag paths (--net-host/--net-join) still load here.
                                // Only the lobby path: a flag start (--net-host/--net-join) has the
                                // arena loaded from the command line and still needs the match load.
                                let already = std::mem::take(&mut self.net_menu_load);
                                if already { if let Some(fe) = self.fe.as_mut() { fe.closed = true; } continue; }
                                self.pending_level = Some(st.level_code);
                                self.pending_mp_settings = Some(*st);
                                self.net_await_load = true;
                                if let Some(fe) = self.fe.as_mut() { fe.closed = true; }
                            }
                            mpnet::CoreEvent::AgentJoined { slot } => {
                                eprintln!("[net] agent {slot} joined");
                                // The flag path's auto-start: the first join arms the match.
                                if let Some(st) = self.net_armed.take() {
                                    let n = 1 + slot as u32;
                                    let mut armed = st;
                                    armed.human_count = n;
                                    armed.player_count = n as u16;
                                    armed.total_players = n;
                                    if let Some(s) = self.net.as_mut() { s.0.borrow_mut().start_match(&armed); }
                                    self.pending_level = Some(armed.level_code);
                                    self.pending_mp_settings = Some(armed);
                                    self.net_await_load = true;
                                }
                            }
                            mpnet::CoreEvent::Refused => {
                                self.net_fail("Refused: a different build or the match is full");
                            }
                            mpnet::CoreEvent::Desync { tick, local, remote } => {
                                // net-desync-resync: the session decided the policy —
                                // a mismatching CLIENT has already asked for a resync
                                // (its `ready_inputs` is None: it freezes on the last
                                // ticked frame) and the host re-serves the late-join
                                // family on the live connection; the machine replays
                                // the host's log (the LateJoinHandover arm below) and
                                // rejoins at the re-announce. A fatal desync (a second
                                // one inside 60 s, a request never served, a peer that
                                // never asked) arrives as `DesyncFatal`.
                                // net-desync-detail: the component exchange still runs
                                // (the socket stays up through the resync) — but NO
                                // `desync_wait` here: its 3 s timeout is the fail path,
                                // and a resync may legitimately take longer. A fatal
                                // end arms the wait itself, with the report if the
                                // exchange for the fatal tick completed.
                                eprintln!("[net] state desync at tick {tick}: local {local:016x} remote {remote:016x} — resyncing (the diverged machine replays the host's log)");
                                for a in self.desync_watch.on_desync(tick) {
                                    self.send_desync_action(a);
                                }
                                if self.net.is_some() {
                                    self.net_resyncing = true; // the draw code's "Resyncing…" note
                                }
                            }
                            mpnet::CoreEvent::DesyncFatal { tick } => {
                                // net-desync-resync: the session gave up (a second
                                // mismatch inside the 60 s window, an unserved request,
                                // an unanswered watch). The net-desync-detail wait arms
                                // here — the report if the exchange for this tick
                                // completed within 3 s, the plain fail box otherwise;
                                // the match never freezes longer than the exchange's
                                // 3 s either way.
                                self.desync_wait = Some((tick, std::time::Instant::now()));
                            }
                            mpnet::CoreEvent::HashComps { tick, comps } => {
                                for a in self.desync_watch.on_peer_comps(tick, comps) {
                                    self.send_desync_action(a);
                                }
                            }
                            mpnet::CoreEvent::HashDetail { tick, component, items } => {
                                for a in self.desync_watch.on_peer_detail(tick, component, items) {
                                    self.send_desync_action(a);
                                }
                            }
                            mpnet::CoreEvent::ClientLeft { slot, graceful } => {
                                // DropPolicy::Neutral: the dropped agent rides NEUTRAL inputs and
                                // the match runs on — the game's own disconnected-pad shape
                                // (psiInput_PollDevices 0x10ca78 writes buttons 0 / analog 0x7f
                                // for a dead pad), not the old terminal PeerLost.
                                eprintln!("[net] player {} {} — the match runs on with NEUTRAL inputs",
                                    slot + 1, if graceful { "left" } else { "timed out" });
                            }
                            mpnet::CoreEvent::HostTimeout => {
                                self.net_fail("Lost the connection to the host");
                            }
                            mpnet::CoreEvent::HostLeft => {
                                // mp-online-failures: the host quit — every client is told at
                                // once and lands back on its Join Online page (address kept).
                                self.net_fail("The host left the match");
                            }
                            mpnet::CoreEvent::LateJoinStarted { slot } => {
                                // The match runs on untouched; the draw code may show
                                // "player joining" from this.
                                eprintln!("[net] late join: serving the run log to slot {slot}");
                            }
                            mpnet::CoreEvent::LateJoinHandover { slot, settings, snapshot_tick, snapshot_hash, log, joins } => {
                                // The joiner's handover: the app replays the log headless
                                // (`mp_late_join_catchup`), compares the hash, then confirms
                                // or refuses. The replay is seconds of sim — the flag path
                                // runs it inline here (the draw code may thread it later);
                                // the joining page owns the pacing.
                                eprintln!("[net] late join: slot {slot} handover, replaying {} ticks from the host's log", snapshot_tick + 1);
                                match crate::mp_late_join_catchup_joins(&self.iso, *settings.clone(), &log, snapshot_tick, &joins) {
                                    Ok((scene, hash)) => {
                                        if hash == snapshot_hash {
                                            eprintln!("[net] late join: replay verified at tick {snapshot_tick}");
                                            // `swap_in_scene`'s scene-bound parts (not its match
                                            // start: the replay already ran that): the old skin
                                            // palette / cameras, the sound stack, the renderer's
                                            // GPU resources — a bare assignment kept the old
                                            // scene's buffers and the draw overran them (a panic).
                                            let mut scene = scene;
                                            self.pal_cur = None;
                                            self.pal_prev = None;
                                            self.cam_prev = None;
                                            self.cam_cur = None;
                                            self.sound = build_sound(&mut scene);
                                            self.audio_paused = false;
                                            if let (Some(old), Some(new)) = (self.scene.sim.as_ref(), scene.sim.as_mut()) { new.hud.carry_boot_globals(&old.hud); }
                                            self.scene = scene;
                                            if let Some(g) = &mut self.gfx {
                                                g.renderer = gpu::Renderer::new(&g.device, &g.queue, g.config.format, &self.scene, self.notex, self.nofog);
                                            }
                                            if let Some(sim) = self.scene.sim.as_ref() {
                                                self.cam = Camera { pos: Vec3::new(sim.bond.pos[0], sim.bond.pos[1] + 1.6, sim.bond.pos[2]), yaw: sim.bond.yaw, pitch: 0.0, fov: GAME_FOV };
                                            }
                                            self.net_tick = snapshot_tick + 1;
                                            // The replayed sim is slot 0's view: it keeps ticking
                                            // as slot 0's machine (slot 0's input on `bond`) until
                                            // the announced join tick, then moves into its own
                                            // body after that tick's hash (`net_late_self`).
                                            self.net_local_slot = 0;
                                            self.net_late_self = Some(slot as usize);
                                            if let Some(s) = self.net.as_mut() { s.0.borrow_mut().confirm_late_join(); }
                                            // mp-online-failures: the menu path's join page closes
                                            // (the tick loop runs only without a front end) and
                                            // the arena is the level now.
                                            self.level_code = settings.level_code;
                                            if self.fe.is_some() { self.close_front_end(); }
                                            if self.scene.sim.is_some() { self.grab(true); }
                                        } else {
                                            eprintln!("[net] late join: replay hash mismatch at tick {snapshot_tick} — refusing");
                                            if let Some(s) = self.net.as_mut() { s.0.borrow_mut().late_join_verify_failed(); }
                                            self.net_fail("The join could not be verified against the match state");
                                        }
                                    }
                                    Err(e) => {
                                        eprintln!("[net] late join: replay failed: {e}");
                                        if let Some(s) = self.net.as_mut() { s.0.borrow_mut().late_join_verify_failed(); }
                                        self.net_fail(&format!("The join failed: {e}"));
                                    }
                                }
                                let _ = &snapshot_hash; // (the comparison used the replay's hash)
                                self.net_late_handover = Some((settings, snapshot_tick, snapshot_hash, log));
                            }
                            mpnet::CoreEvent::LateJoinSlot { slot, join_tick } => {
                                // Every machine arms the slot at the announced tick: the
                                // spawn runs inside the net tick (the shared-stream draw is
                                // part of the deterministic tick), the joiner's own view
                                // adopts the slot there.
                                eprintln!("[net] late join: slot {slot} arms at tick {join_tick}");
                                self.net_join_pending = Some((slot, join_tick));
                                // The joiner had no match clock (it never ran `swap_in_scene`'s
                                // start): the host picked the join tick JOIN_MARGIN_TICKS past its
                                // head, so the clock reads from the host's head and the budget
                                // fast-forwards the replay gap (net-internet: reading from the join
                                // tick itself put the clock 1.5 s ahead of every machine for the
                                // rest of the match — the joiner never paced, it ran whatever had
                                // arrived each frame and stalled between).
                                if self.net_late_self == Some(slot as usize) {
                                    let head = join_tick.saturating_sub(nf_net::session::JOIN_MARGIN_TICKS);
                                    self.net_clock = std::time::Instant::now().checked_sub(std::time::Duration::from_secs_f64(head as f64 / 60.0));
                                    self.net_pacer.reset();
                                }
                            }
                            // net-match-end: the debriefing acks ride the wait block above (the
                            // host's end wait consumes them from its own drain); the match's
                            // event loop ignores them.
                            mpnet::CoreEvent::DebriefAck { .. } => {}
                    }
                }
                // net-desync-detail: the diagnosis exchange's wait — the report once the peer's
                // components + items arrived (or the fail box after a short timeout, the
                // exchange incomplete), then the same back-to-lobby path. The socket must stay
                // up through the exchange, so this runs BEFORE net_fail.
                if let Some((dtick, started)) = self.desync_wait {
                    if self.desync_watch.diagnosis.as_ref().is_some_and(|d| d.tick == dtick) {
                        self.finish_desync_report();
                    } else if started.elapsed() > std::time::Duration::from_secs(3) {
                        eprintln!("[net] desync at tick {dtick}: the component exchange did not complete in 3 s");
                        self.net_fail(&format!("State desync at tick {dtick}\n(the component exchange did not complete)\nReturning to the lobby"));
                    }
                }
                let menu_hold = self.paused || self.fe.as_ref().is_some_and(|fe| fe.page == menu::Page::Pause);
                // `GS_IsPaused` → the health pane writes BL+0x8BC = 1.0 every paused frame [M
                // HUD_UpdateHealthPane 0x19b9c8:117-122]: a pause brings the faded panes back. Not
                // online (the match runs on under one machine's pause page; a local sim write would
                // desync the lockstep).
                if menu_hold && self.net.is_none() {
                    if let Some(sim) = self.scene.sim.as_mut() { sim.drones.pain_alpha = 1.0; }
                }
                if menu_hold != self.audio_paused {
                    self.audio_paused = menu_hold;
                    // The game's pause entry IS the SFXPaused setter: `MenuManager_Create` with
                    // pause_sound=1 runs the op-9 batch whose EE side sets `SFXPaused` 0x30c92c
                    // [M 0x21b908:121-124, 0x116b70:11-13] — `UpdateMusicalEvents` freezes on it
                    // [M 0x114e70:61-64]. The port mirrors it on the same edge.
                    if let Some(sim) = self.scene.sim.as_mut() { sim.drones.music.sfx_paused = menu_hold; }
                    if let Some(s) = self.sound.as_mut() { s.set_paused(menu_hold); }
                }
                // An online match never stops for one machine's pause page: the others would stall on
                // its missing inputs (lockstep). The match runs on under the menu with this machine's
                // player idle (NEUTRAL input below) — a port-only case, since the shipped game has
                // no online [D action-symtab.tsv census: zero socket/TCP/UPnP symbols]; its couch
                // pause stops the game for every pad (GS_IsPaused gates control_movement_object_handler
                // out of Game_Run [M 0x1c9790:31-36]) and there is no shared screen to pause online.
                let net_menu = self.net.is_some() && !self.net_await_load
                    && self.fe.as_ref().is_some_and(|fe| fe.page == menu::Page::Pause);
                if !self.paused && (self.fe.is_none() || net_menu) {
                    // Game logic at its fixed 60 Hz, decoupled from the render rate: one AI tick per whole tick of time.
                    // Dev mode's time scale changes the number of ticks run (WORKLOG 0081), not the tick rate.
                    let before = self.anim_time as u32;
                    // fix-perf-level: the catch-up cap. A slow render frame must not burst a spiral of
                    // ticks — on 07000006 a tick costs ~4.1 ms, so the old 8-tick cap let the sim
                    // burst 33 ms behind one frame (the jitter fix-render3 measured). The game itself
                    // has no catch-up: under load it slows down. Capped ticks are DROPPED, not
                    // deferred — logic stays fixed-60 Hz and deterministic (the tick count depends
                    // only on frame times, never on their content). `NF_MAXTICKS` retunes it.
                    let max_ticks = *MAX_TICKS.get_or_init(|| std::env::var("NF_MAXTICKS").ok()
                        .and_then(|v| v.parse::<u32>().ok()).filter(|v| *v > 0).unwrap_or(4));
                    // G-043: the loop's tick scheduling, extracted verbatim into `ticks_for_frame`
                    // (the fps-determinism test drives the same arithmetic headless).
                    let elapsed = ticks_for_frame(&mut self.anim_time, dt, max_ticks);
                    self.frame_ticks = elapsed;
                    // fix-ingame-dev-menu: the open menu pauses the sim — 0 ticks unless F11's
                    // one-shot step was pressed (`paused_tick_count`).
                    let n_ticks = match self.dev.as_deref_mut() {
                        Some(d) if d.menu => d.paused_tick_count(),
                        Some(d) => d.tick_count(elapsed),
                        None => elapsed,
                    };
                    // Online: the tick budget is the match clock's, not the frame's — ticks a stall
                    // held back run on the next frames, so a stall delays the match instead of
                    // permanently slowing it (the two-window check ran at ~15 Hz). net-internet:
                    // the backlog is made up at a bounded rate (`TickPacer`: 1.25x real time for a
                    // short one, 2x up to a second, the old <= 4-per-frame fast-forward beyond) — a
                    // burst of 4 ticks in one frame was a visible 67 ms jump after every stall.
                    let n_ticks = match (self.net.is_some(), self.net_clock) {
                        (true, Some(c)) => {
                            let target = (c.elapsed().as_secs_f64() * 60.0) as u32;
                            self.net_pacer.ticks_for_frame(target, self.net_tick, dt as f64, 4)
                        }
                        _ => n_ticks,
                    };
                    self.ticked = n_ticks > 0;
                    // mp-perf-16: the online ticks this frame actually ran (past the lockstep gate).
                    let mut net_ran = 0u32;
                    let eye = [self.cam.pos.x, self.cam.pos.y, self.cam.pos.z];
                    let look = self.cam.dir().to_array();
                    // mp-host-match-logic: set inside the tick loop when the match's debriefing
                    // request latched and the host opened the page (the grab waits for this
                    // scope — the sim borrow).
                    let mut debrief_opened = false;
                    let fp_t0 = std::time::Instant::now();
                    if let Some(sim) = self.scene.sim.as_mut() {
                        for _ in 0..n_ticks {
                            if self.noclip { sim.step(eye, look, self.trigger, self.aiming); continue; }
                            // net-consolidate: the lockstep gate — the sim advances tick t only
                            // when every live agent's input for t has arrived. A stalled tick
                            // renders the last ticked frame (`ticked` false keeps the
                            // interpolation honest), and NO capture runs on it, so key edges
                            // survive the stall. Never guess: the session's missing input is a
                            // wait, not a default.
                            if let Some(session) = self.net.as_mut() {
                                if self.net_await_load { self.ticked = false; break; }
                                if session.0.borrow_mut().ready_inputs(self.net_tick).is_none() {
                                    NET_STALLS.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                                    // mp-perf-16 (the joiner's "54 ticks/s, 240 stalls/s"): (1) a frame
                                    // whose earlier iterations RAN ticks keeps `ticked` — the old
                                    // `false` here threw those ticks' palette/camera snapshots away,
                                    // and a machine running at the edge of the others' inputs (every
                                    // frame runs a tick, then stalls on the next) never rebuilt its
                                    // palette at all (NF_FPSLOG "tick frames 0" on the joiner while
                                    // it ran 60 ticks/s). (2) The stall pauses the match clock: the
                                    // clock is re-anchored on the waiting tick, so the time spent
                                    // waiting is not owed back as a lead. The joiner's clock started
                                    // at ITS load, ~6 s before the host's first inputs arrived; that
                                    // lead never drained, so every frame asked for up to 4 ticks,
                                    // stalled (240 stalls/s at 240 Hz) and NF_NET_RATE's
                                    // ticks / clock read 54/s while the joiner really ran 60.
                                    self.ticked = net_ran > 0;
                                    self.net_clock = std::time::Instant::now().checked_sub(std::time::Duration::from_secs_f64(self.net_tick as f64 / 60.0));
                                    break;
                                }
                                net_ran += 1;
                            }
                            // B-5 record/replay: when `--replay` is set, the input comes from the recording, not the
                            // keyboard. The recorded tick drives `step_bond` exactly as it did when recorded; the player
                            // can only watch (Esc to quit). When the recording ends, the sim pauses.
                            let (mut input, trigger, reload, use_key) = if let Some(rp) = &self.replay {
                                if self.replay_idx < rp.ticks.len() {
                                    let t = rp.ticks[self.replay_idx];
                                    self.replay_idx += 1;
                                    t.to_parts()
                                } else {
                                    self.paused = true; // recording exhausted — hold the last frame.
                                    break;
                                }
                            } else {
                                // Keys as full-deflection actions: W / S forward (action 2), A / D strafe (action 1, + = left).
                                let held = |k: KeyCode| if self.keys.contains(&k) { 1.0 } else { 0.0 };
                                let input = nf_game::player::Input {
                                    forward: held(KeyCode::KeyW) - held(KeyCode::KeyS),
                                    strafe: held(KeyCode::KeyA) - held(KeyCode::KeyD),
                                    // Zero-g's vertical thruster: the jump/crouch keys HELD, the
                                    // combined (action 7 − action 8) analog [M Player_ZeroG
                                    // 0x192ab0:136-138] — walk ignores the field.
                                    lift: held(KeyCode::Space)
                                        - if self.keys.contains(&KeyCode::KeyC) || self.keys.contains(&KeyCode::ControlLeft) { 1.0 } else { 0.0 },
                                    // The stabiliser arms off the E/R edges instead — step_bond
                                    // ORs the use_key/reload lanes in (both are the game's action
                                    // 0xe) [M 0x192ab0:163-166].
                                    stabilise: false,
                                    turn: std::mem::take(&mut self.turn),
                                    // options-census: the Y-axis row negates the look channel
                                    // [M Input_Update 0x17ee88:50-52] — the port's accumulated
                                    // delta (mouse + pad merge) flips sign here.
                                    look: std::mem::take(&mut self.look) * if self.look_inverted { -1.0 } else { 1.0 },
                                    jump: self.just.remove(&KeyCode::Space),
                                    // options-census: the Crouch row picks the feed [M
                                    // Player_SSCrouch 0x1924c0:84-105] — Toggle reads press edges
                                    // (the game's Input_Action(8/7, 4)), Hold reads the level
                                    // (Input_Action(8, 1)).
                                    crouch: if self.crouch_toggle_opt {
                                        self.just.remove(&KeyCode::KeyC) | self.just.remove(&KeyCode::ControlLeft)
                                    } else {
                                        self.keys.contains(&KeyCode::KeyC) || self.keys.contains(&KeyCode::ControlLeft)
                                    },
                                    crouch_hold: !self.crouch_toggle_opt,
                                    aiming: self.aiming,
                                    weapon_cycle: std::mem::take(&mut self.weapon_cycle),
                                    gadget_cycle: std::mem::take(&mut self.gadget_cycle),
                                    zoom_cycle: std::mem::take(&mut self.zoom_cycle),
                                    switch_to: self.weapon_switch_to.take(),
                                    fire: self.trigger,
                                    zoom: self.zoom,
                                    scope: self.zoom,
                                    // The fire-mode button (pad ○; keyboard M) [M Player_Weapon 0x1a24c8].
                                    mode_switch: self.just.remove(&KeyCode::KeyB),
                                    // The goggles key: pad action 13 only (the styles). No keyboard
                                    // bind — the disc grants the goggles nowhere, and the game's
                                    // latch-clear press requests item 0x5d with no ownership check
                                    // [M HUD_MonitorNightSight 0x19e9d8:82-84; Player_WeaponSelect
                                    // 0x1a2cc0 has none either], so a port-only key would draw an
                                    // item Bond does not carry.
                                    goggles_action: false,
                                };
                                (input, self.trigger, self.just.remove(&KeyCode::KeyR), self.just.remove(&KeyCode::KeyE))
                            };
                            // The online pause (net_menu): the keys belong to the menu — this player idles.
                            let (mut input, trigger, reload, use_key) = if net_menu {
                                (Default::default(), false, false, false)
                            } else { (input, trigger, reload, use_key) };
                            // Test aid (NF_NET_AUTOFIRE="a-b"): hold the trigger on match ticks a..=b — the
                            // two-window desync hunt fires without a hand on the window.
                            let trigger = trigger || (self.net.is_some() && std::env::var("NF_NET_AUTOFIRE").ok()
                                .and_then(|v| v.split_once('-').and_then(|(a, b)| Some((a.parse::<u32>().ok()?, b.parse::<u32>().ok()?))))
                                .is_some_and(|(a, b)| (a..=b).contains(&self.net_tick) && (self.net_tick - a) % 12 < 6));
                            if trigger { input.fire = true; }
                            // Test aid (NF_NET_AUTOMOVE="a-b"): walk and turn on match ticks a..=b (the
                            // pattern of tests/mp_bots_moving_live.rs) — the two-window moving check
                            // without a hand on either window.
                            if self.net.is_some() && std::env::var("NF_NET_AUTOMOVE").ok()
                                .and_then(|v| v.split_once('-').and_then(|(a, b)| Some((a.parse::<u32>().ok()?, b.parse::<u32>().ok()?))))
                                .is_some_and(|(a, b)| (a..=b).contains(&self.net_tick))
                            {
                                let ph = self.net_tick + self.net_local_slot as u32 * 97;
                                input.forward = if ph % 240 < 200 { 1.0 } else { -0.5 };
                                input.strafe = if ph % 150 < 30 { 0.8 } else if ph % 150 < 45 { -0.8 } else { 0.0 };
                                input.turn = if ph % 180 < 60 { 0.03 } else if ph % 180 < 90 { -0.02 } else { 0.0 };
                                input.look = if ph % 120 < 20 { 0.01 } else if ph % 120 < 40 { -0.01 } else { 0.0 };
                                input.jump = ph % 211 == 0;
                                input.crouch = ph % 307 == 0;
                            }
                            // Test aid (NF_NET_AUTOPICK="a-b", mp-world-sync): on match ticks a..=b
                            // steer this player at a live pickup (one of the three nearest, the pick
                            // rotating every 10 s so a stuck walk moves on) — the two-window pickup
                            // check without a hand on either window. Input only; the sim is untouched.
                            if self.net.is_some() && std::env::var("NF_NET_AUTOPICK").ok()
                                .and_then(|v| v.split_once('-').and_then(|(a, b)| Some((a.parse::<u32>().ok()?, b.parse::<u32>().ok()?))))
                                .is_some_and(|(a, b)| (a..=b).contains(&self.net_tick))
                            {
                                let b = &sim.bond;
                                let mut live: Vec<[f32; 3]> = sim.drones.pickups.iter().filter(|p| !p.gone && !p.hidden)
                                    .map(|p| [p.m[3][0], p.m[3][1], p.m[3][2]]).collect();
                                let d2 = |p: &[f32; 3]| (p[0] - b.pos[0]).powi(2) + (p[2] - b.pos[2]).powi(2);
                                live.sort_by(|x, y| d2(x).total_cmp(&d2(y)));
                                let k = ((self.net_tick / 600) as usize + self.net_local_slot) % live.len().clamp(1, 3);
                                if let Some(tg) = live.get(k) {
                                    let want = (tg[0] - b.pos[0]).atan2(tg[2] - b.pos[2]);
                                    let mut dy = (want - b.yaw) % std::f32::consts::TAU;
                                    if dy > std::f32::consts::PI { dy -= std::f32::consts::TAU; }
                                    if dy < -std::f32::consts::PI { dy += std::f32::consts::TAU; }
                                    input.turn = dy.clamp(-0.08, 0.08);
                                    input.forward = if dy.abs() < 0.8 { 1.0 } else { 0.2 };
                                    input.look = (-b.pitch).clamp(-0.05, 0.05);
                                    input.jump = self.net_tick % 97 == 0;
                                }
                            }
                            // Test aid (NF_NET_SWITCH="tick:id,tick:id,…", ids hex 0x.. or decimal):
                            // mp-combat-weapons — on each listed match tick this machine equips the
                            // weapon (its local inventory: three clips, `Player_EquipWeapon`'s give as
                            // `--give` does) and switches to it through the normal `switch_to` lane;
                            // the two-window weapon-class check without a hand on the window.
                            if self.net.is_some() {
                                if let Ok(v) = std::env::var("NF_NET_SWITCH") {
                                    for e in v.split(',') {
                                        let Some((t, id)) = e.trim().split_once(':') else { continue };
                                        let id = id.strip_prefix("0x").and_then(|h| usize::from_str_radix(h, 16).ok()).or_else(|| id.parse().ok());
                                        if t.parse::<u32>().ok() == Some(self.net_tick) {
                                            if let Some(id) = id.filter(|&i| i < 0x72) {
                                                let wd = sim.weapons.clone();
                                                let rounds = wd.clip_size(id).max(1);
                                                sim.drones.inventory.equip_weapon(&wd, id, rounds, true);
                                                sim.drones.inventory.equip_ammo(&wd, id, (rounds * 3).max(rounds));
                                                input.switch_to = Some(id as u8);
                                                eprintln!("[net] test switch to {id:#04x} at tick {}", self.net_tick);
                                            }
                                        }
                                    }
                                }
                            }
                            // weapon-matrix: the dev menu's "Test weapon" run (dev.rs) merges its
                            // inputs into this tick — the same lanes the keyboard uses
                            // (`switch_to`, the trigger, the R edge), never during a replay
                            // (the recording drives the sim alone).
                            let (tw_switch, tw_fire, tw_reload) = if self.replay.is_none() {
                                self.dev.as_deref_mut().map_or((None, false, false), |d| d.test_inputs())
                            } else { (None, false, false) };
                            input.switch_to = input.switch_to.or(tw_switch);
                            let trigger = trigger || tw_fire;
                            let reload = reload || tw_reload;
                            // The gamepad (fix-input, `gamepad` feature): the pad's path through the
                            // game's own mapping/curves/ramps merges into the keyboard's Input; no pad,
                            // no change. Mouse look is untouched (its deltas are in input.turn/look; the
                            // pad's contributions add on top).
                            #[cfg(feature = "gamepad")]
                            if let Some(pad) = self.pad.as_mut() {
                                pad.merge(&mut input, sim.zoom, sim.bond.underwater, false);
                            }
                            // The party weapon lock (`Player_Weapon` 0x1a24c8:242-275): on 07000007 with
                            // ch 0x48 off the cycle actions cannot leave the fists (fix-level2 F4).
                            nf_game::player::weapon_lock_input(sim.level, sim.drones.channels.on(nf_game::player::WEAPON_LOCK_CHANNEL), &mut input);
                            // net-consolidate: the finalized input goes to the session (this slot's
                            // entry in the per-tick exchange; the trigger/reload/use_key lanes ride
                            // the 24-byte wire input now — 053's 20-byte packing dropped them).
                            // The lockstep fix (the two-window freeze, 2026-10-18): the input sampled now
                            // is scheduled `input_delay` ticks ahead, and THIS tick runs on the input
                            // every machine has for it (the session prefilled the first ticks with
                            // NEUTRAL). Submitting for `net_tick` after the gate had already asked for
                            // it meant no machine ever had its own input: every window froze. The
                            // local view lanes (zero-g lift/stabilise, the wheel's zoom steps) stay
                            // local — they never ride the wire.
                            let (trigger, reload, use_key) = if let Some(session) = self.net.as_mut() {
                                let mut ni = mpnet::to_net_input(&input, trigger, reload, use_key);
                                // 700v6: this player's weapon and whether its own pipeline fired a round
                                // on the last local tick — every machine resolves that shot identically.
                                ni.held = sim.gun.weapon;
                                // mp-combat-weapons: the round's description — the fired record,
                                // the aim ray, the scope flag (every machine runs the round).
                                if let Some(sh) = self.net_shot_pending.take() {
                                    ni.shot = true;
                                    ni.shot_weapon = sh.weapon;
                                    ni.shot_flags = sh.scoped as u8;
                                    ni.shot_aim = nf_game::mp::combat::aim_from_dir(sh.dir);
                                }
                                // net-internet: the session places it `delay_at(net_tick)` ahead
                                // (the host's adaptive delay, switched at an announced tick).
                                session.0.borrow_mut().submit_local(self.net_tick, ni);
                                let ready = session.0.borrow_mut().ready_inputs(self.net_tick);
                                match ready {
                                    Some(r) => {
                                        // mp-world-sync: the row rides into `step_bond` (the remotes
                                        // move before the local body; every slot's use/shot reaches
                                        // the shared world).
                                        sim.net_row = Some(r);
                                        let li = r[self.net_local_slot.min(nf_net::MAX_AGENTS - 1)];
                                        let view = (input.lift, input.stabilise, input.zoom_cycle);
                                        input = mpnet::to_player_input(&li);
                                        (input.lift, input.stabilise, input.zoom_cycle) = view;
                                        (li.trigger, li.reload, li.use_key)
                                    }
                                    None => (trigger, reload, use_key),
                                }
                            } else {
                                (trigger, reload, use_key)
                            };
                            // mp-join-in-progress: the announced join spawn — every machine creates
                            // the slot's agent at the same tick (the shared-stream pick is part of
                            // this deterministic tick); the joiner's own view adopts the slot (the
                            // camera and the full local path tick it from here on). mp-world-sync:
                            // BEFORE the tick (`step_bond` now moves the remotes inside it) — the
                            // late joiner's replay (`mp_late_join_catchup_joins`) spawns before its
                            // tick too, so both see the same shared-stream order.
                            let mut swap_after: Option<usize> = None;
                            if self.net.is_some() && sim.net_row.is_some() {
                                if let Some((js, jt)) = self.net_join_pending {
                                    if self.net_tick == jt {
                                        sim.mp_join_spawn(js as usize);
                                        if js as usize == self.net_local_slot {
                                            sim.swap_local_slot(js as usize);
                                        } else if self.net_late_self == Some(js as usize) {
                                            // The joiner: this tick runs as slot 0's machine
                                            // (the host's exact tick); the move follows the hash.
                                            swap_after = Some(js as usize);
                                        }
                                        self.net_join_pending = None;
                                    }
                                }
                            }
                            // B-5 record: capture the exact input passed to step_bond, before the call.
                            if let Some(r) = &mut self.recorder { r.push(replay::TickInput::from_parts(&input, trigger, reload, use_key)); }
                            // Player bug reports: the always-on recording of this level (a new one on every
                            // (re)start) and, once a second, the crash context (bugreport.rs).
                            bugreport::record_tick(sim, self.entry_save.as_ref(), self.selected_difficulty, replay::TickInput::from_parts(&input, trigger, reload, use_key));
                            // Test aid (NF_BUG_REPORT_AT=tick): the F10 report at that tick, taken after the frame.
                            if bugreport::report_at() == Some(sim.drones.frame) { bugreport::DUE.store(true, std::sync::atomic::Ordering::Relaxed); }
                            // Test aid (NF_TEST_PANIC_AT=tick): a deliberate panic, to exercise the crash report.
                            if std::env::var("NF_TEST_PANIC_AT").ok().and_then(|v| v.parse::<u32>().ok()) == Some(sim.drones.frame) {
                                panic!("NF_TEST_PANIC_AT: deliberate test panic at tick {}", sim.drones.frame);
                            }
                            if sim.drones.frame % 60 == 0 {
                                bugreport::set_context(format!("level: {:08x}\ntick: {}\ndifficulty: {}\nbond: pos {:.2?} yaw {:.3} pitch {:.3} substate {} health {:.1} armour {:.1}\nweapon: {:#04x} clip {} spare {}\nmission: internal state {} base {:#010x}\nonline: {}",
                                    sim.level, sim.drones.frame, self.selected_difficulty, sim.bond.pos, sim.bond.yaw, sim.bond.pitch, sim.bond.substate,
                                    sim.drones.health.health, sim.drones.health.armour, sim.gun.weapon, sim.gun.clip, sim.gun.spare,
                                    sim.drones.mission.internal, sim.drones.mission.base_map, self.net.is_some()));
                            }
                            // The PC-port rapid fire (`--rapid-fire` / the dev Toggles page): local play only —
                            // an online match keeps every machine on the game's rules (lockstep).
                            sim.rapid_fire = RAPID_FIRE.load(std::sync::atomic::Ordering::Relaxed) && self.net.is_none();
                            sim.step_bond(input, trigger, reload, use_key);
                            // Port dev aid: re-apply `--bond` after the opening cut-scene's hand-back.
                            if let Some(b) = self.dev_bond {
                                if sim.disabled {
                                    self.dev_bond_cut = true;
                                } else if self.dev_bond_cut {
                                    sim.bond = nf_game::player::Bond::new([b[0], b[1], b[2]], b[3]);
                                    sim.bond.pitch = b[4];
                                    self.dev_bond = None;
                                } else if sim.drones.frame > 900 {
                                    self.dev_bond = None;
                                }
                            }
                            // weapon-matrix: the "Test weapon" run watches the tick's result (the
                            // phase machine in dev.rs; `sim` and `self.dev` are disjoint fields).
                            if let Some(d) = self.dev.as_deref_mut() { d.test_observe(&*sim); }
                            // net-consolidate: the remote agents' movement tick with the session's
                            // inputs (the same `Bond::tick_ctx` core on every machine), the MP
                            // slots' live positions, and the state hash on the session's own
                            // cadence (hash_every 15; the previous hash piggybacks every packet).
                            if self.net.is_some() {
                                let session = self.net.as_mut().unwrap();
                                let slot = self.net_local_slot;
                                // Bound first: an `if let` keeps its scrutinee's RefCell borrow for the
                                // whole body, and submit_hash below borrows again ("RefCell already
                                // borrowed" — latent until the lockstep fix let a tick run).
                                let ready = session.0.borrow_mut().ready_inputs(self.net_tick);
                                if let Some(net_inputs) = ready {
                                    // mp-join-in-progress: the host records the exact transcript
                                    // a late joiner replays (bit-exact by construction).
                                    session.0.borrow_mut().record_run_tick(self.net_tick, net_inputs);
                                    // (The announced join spawn ran before `step_bond` — see there.)
                                    let inputs = net_inputs.map(|i| mpnet::to_player_input(&i));
                                    sim.step_net_remotes(&inputs, slot);
                                    sim.step_net_combat(&net_inputs, slot);
                                    // The local pipeline's shot this tick rides the next submitted input.
                                    if let Some(sh) = sim.net_shot.take() { self.net_shot_pending = Some(sh); }
                                    // GAPS G-053 item 4: the same every-slot post-move write the tick
                                    // path runs — one shared writer, no fixed 4.
                                    write_net_slot_positions(sim, slot);
                                    // net-desync-detail: the named component hashes — the
                                    // session compares their fixed-order fold; the watcher keeps
                                    // the last 600 ticks of components plus, at the exchange
                                    // cadence, the per-item rows the desync exchange diagnoses
                                    // from (a desync is only ever detected at an exchanged tick).
                                    // Test aid (NF_NET_PERTURB_AT=<tick>): nudge this machine's own body once at that net tick
                                    // — the live desync check (the fail box, the diagnosis, the report zip).
                                    if std::env::var("NF_NET_PERTURB_AT").ok().and_then(|v| v.parse::<u32>().ok()) == Some(self.net_tick) {
                                        sim.bond.pos[0] += 0.01;
                                        eprintln!("[net] test aid: perturbed this machine's body at net tick {}", self.net_tick);
                                    }
                                    let comps = sim.net_component_hashes(slot);
                                    let h = crate::scene::fold_component_hashes(&comps);
                                    self.desync_watch.on_tick(self.net_tick, comps);
                                    let every = session.0.borrow().hash_every().max(1);
                                    if self.net_tick % every == 0 {
                                        let items = sim.net_component_items(slot);
                                        self.desync_watch.on_detail_tick(self.net_tick, items);
                                    }
                                    session.0.borrow_mut().submit_hash(self.net_tick, h);
                                    if let Some(js) = swap_after {
                                        // The late joiner takes its own body: the slot-0 replica
                                        // parks in mp_bonds[0]; the new agent starts with the
                                        // match's start loadout and its slot's health.
                                        sim.swap_local_slot(js);
                                        sim.mp_equip_local();
                                        sim.drones.health.health = sim.mp_hp[js];
                                        sim.net_rng = Some(nf_game::drone::Rng::new(0x5eed_0b0b ^ (js as u32).wrapping_mul(0x9e37_79b9)));
                                        self.net_local_slot = js;
                                        self.net_late_self = None;
                                        self.net_shot_pending = None;
                                        // net-desync-resync: the resyncing machine is playing its
                                        // own slot again — the note comes down.
                                        let was_resync = std::mem::take(&mut self.net_resyncing);
                                        // net-desync-resync: a recovered desync's diagnosis (if the
                                        // exchange finished while the replay ran) is spent — a later
                                        // fatal desync must not report this older mismatch.
                                        self.desync_watch.diagnosis = None;
                                        self.cam = Camera { pos: Vec3::new(sim.bond.pos[0], sim.bond.pos[1] + 1.6, sim.bond.pos[2]), yaw: sim.bond.yaw, pitch: 0.0, fov: GAME_FOV };
                                        eprintln!("[net] late join: playing slot {js} from tick {}{}", self.net_tick, if was_resync { " (resync complete)" } else { "" });
                                    }
                                }
                                self.net_tick += 1;
                                // NF_NET_RATE: once a second, the ticks run / stalls / frames.
                                if self.net_tick % 60 == 0 && std::env::var_os("NF_NET_RATE").is_some() {
                                    let c = self.net_clock.map_or(0.0, |c| c.elapsed().as_secs_f64());
                                    let (rtt, delay, bytes, pkts) = session.0.borrow().net_stats();
                                    eprintln!("[netrate] tick {} at {:.1} s ({:.1}/s), stalls {}, rtt {} ms, delay {} ticks, sent {} B / {} pkts", self.net_tick, c, self.net_tick as f64 / c.max(1e-3), NET_STALLS.swap(0, std::sync::atomic::Ordering::Relaxed), rtt.map_or("-".to_string(), |r| format!("{r:.0}")), delay, bytes, pkts);
                                }
                                // Render-check aid (NF_NET_SHOT=<tick>[,<tick>…] or <a>-<b>): the F12 capture at
                                // those match ticks (a range catches the 2-tick muzzle flashes, mp-remote-fx).
                                let tick = self.net_tick;
                                if std::env::var("NF_NET_SHOT").ok().is_some_and(|v| v.split(',').any(|x| match x.trim().split_once('-') {
                                    Some((a, b)) => a.parse::<u32>().ok().zip(b.parse::<u32>().ok()).is_some_and(|(a, b)| (a..=b).contains(&tick)),
                                    None => x.trim().parse::<u32>().ok() == Some(tick),
                                })) {
                                    NET_SHOT_DUE.store(true, std::sync::atomic::Ordering::Relaxed);
                                }
                                // NF_NET_SHOT_FX=<n> (mp-remote-fx render aid): capture the first n ticks on
                                // which another slot fired (its muzzle flash is up for this tick's draw).
                                if let Some(n) = std::env::var("NF_NET_SHOT_FX").ok().and_then(|v| v.parse::<u32>().ok()) {
                                    let f = sim.drones.frame;
                                    let fired = (0..nf_net::MAX_AGENTS).any(|sl| sl != sim.mp_local_slot && sim.mp_fx.last_shot[sl] == Some(f));
                                    if fired && NET_SHOT_FX_TAKEN.load(std::sync::atomic::Ordering::Relaxed) < n {
                                        NET_SHOT_FX_TAKEN.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                                        eprintln!("[mpfx] capture at frame {f} (a remote slot fired)");
                                        NET_SHOT_DUE.store(true, std::sync::atomic::Ordering::Relaxed);
                                    }
                                }
                            }
                            // The lock's rising edge auto-draws the handgun (`Upgrade_Weapon(6)` = 2 [L])
                            // [M 0x1a24c8:247-250]. `prev` = last tick's channel (the game's
                            // `switch_channels_prev` semantics).
                            let on = sim.drones.channels.on(nf_game::player::WEAPON_LOCK_CHANNEL);
                            if nf_game::player::weapon_lock_auto_draw(sim.level, on, self.prev_lock_ch) {
                                eprintln!("[weapon-lock] ch 0x48 rising edge: switch request → 2 (P2K)");
                                sim.gun.switch_request = 2;
                            }
                            self.prev_lock_ch = on;
                            if let Some(s) = self.sound.as_mut() { sound_tick(s, sim); }
                            // mp-host-match-logic: the match-end handover — `MP_Update` 0x184bd0's
                            // case 4 sets `GameState+0x1C = 0x40000033` (the port's
                            // `MpGame::request_debriefing`, state 5 waits); the host opens
                            // P_MPDEBRIEFING with the board built from the match [M 0x212198 msg
                            // 0x4c]. The fe is gone during gameplay (the load closed it), so the
                            // host re-creates it (the net_fail pattern) — the frame loop's
                            // `fe.is_none()` gate then stops the sim ticks behind the page, and
                            // the debriefing page's own arms take over.
                            if sim.mp.as_ref().is_some_and(|m| m.game.request_debriefing)
                                && self.fe.as_ref().map_or(true, |fe| fe.mp_debriefing.is_none())
                            {
                                let m = sim.mp.as_ref().unwrap();
                                let fresh_fe = self.fe.is_none();
                                let fe = self.fe.get_or_insert_with(|| {
                                    let mut fe = menu::FrontEnd::new();
                                    // batch-1016b: the re-created manager inherits the music statics
                                    // and the option file like the pause's create (menu-audio-state
                                    // [M 0x1dceb8:94-101]; settings-persist).
                                    Self::inherit_music_statics(&mut fe, self.music_bank_flip, self.music_loaded_bank, self.music_master);
                                    let (opts, warn) = settings::load_default();
                                    if let Some(w) = warn { eprintln!("[settings] {w}"); }
                                    fe.opts = opts;
                                    fe
                                });
                                // profile-session: a fresh debriefing manager inherits the standing
                                // codename session (the pause's rule — the game's globals stand).
                                if fresh_fe {
                                    let session = self.profile_session.clone();
                                    session.apply_to(fe);
                                }
                                fe.open_mp_debriefing(m);
                                debrief_opened = true; // the grab waits for the sim borrow (below)
                            }
                        }
                        // Dev mode after-tick: god mode / infinite ammo (the game's own state, set the way the game sets it).
                        // (Moved below the sim borrow — see the end of this block.)
                        // Bug 5: level transition. When a `LOAD_LEVEL` (0xE8) trigger fires, the sim sets
                        // `drones.level_exit` to the next level's hashcode. Reload the scene with that archive.
                        // Script event 0x12 requests ride the same field (07000006's cut-scene → 07000007;
                        // 0700004a's pack cut-scene → the 0x0720004a movie code) — resolve the movie/wrapper
                        // codes the way the game's load pipeline does [M 0x1d3800 + 0x1d6360 phase 2 +
                        // 0x1d3560; nf_game::triggers::resolve_exit_code].
                        if let Some(end_to) = sim.drones.mission_end.take() {
                            eprintln!("level: mission over (LevelToEndTo {end_to:08x}) — the outro + results");
                            self.pending_mission_end = true;
                        }
                        // mp-match-end: `MP_Update` 0x184bd0 case 4's handover — the debriefing
                        // request carries the finished match's board + settings to the front end
                        // (the drain runs after the tick block, beside `pending_mission_end`'s).
                        if self.pending_mp_debrief.is_none() {
                            if let Some((board, settings)) = sim.take_mp_debrief_request() {
                                eprintln!("mp: match over — the debriefing handover ({} rows)", board.rows.len());
                                self.pending_mp_debrief = Some((board, settings));
                            }
                        }
                        if let Some(next) = sim.drones.level_exit.take() {
                            // mayhew-end-transition: the exit hold — `Trigger_Activate` pushed
                            // `GameFlow_PushState(300.0f, 7, 0xff)` AFTER `ResetMap_LevelToLoad`
                            // [M 0x1b2318:90-92]: the loader (state 3) sits UNDER the fade (state
                            // 8), which pops only when the 300-frame fade ends [M 0x1c9f38:18-24,
                            // 0x1c98c8:173-189]. Defer the drain while the hold runs; the load
                            // starts the frame after it clears (docs/systems/exit-hold.md).
                            if sim.drones.exit_hold.is_some() {
                                sim.drones.level_exit = Some(next);
                            } else {
                                // doublecross-transitions: the exit rides the game's real path —
                                // `ResetMap_LevelToLoad(next, 1, 0)` WRAPS the story code to its
                                // 0x0710000N briefing code [M 0x1d3800:27-101, 102-103], then the
                                // loader's phase 2 resolves it (`GetLevelToLoadFromMovieCode`
                                // 0x1d3560; a briefing not in `UsingMovie` never starts a movie —
                                // state 6 pops on its first tick [M 0x1c98c8 case 6] — and phase 2
                                // strips the wrap back). A Double Cross hop therefore loads the raw
                                // part code by the wrapped route; the whitelisted briefings/outros
                                // ride the front-end/mission paths (`movie_pending`, `mission_ended`).
                                let wrapped = nf_game::triggers::wrap_intro(next);
                                let resolved = nf_game::triggers::resolve_exit_code(wrapped);
                                if resolved != next { eprintln!("level: exit {next:08x} wraps to {wrapped:08x} and resolves to {resolved:08x} [M 0x1d3800 + 0x1d3560]"); }
                                eprintln!("level: exit trigger fired → loading {resolved:08x}");
                                self.pending_level = Some(resolved);
                            }
                        }
                        // movie-trigger-41: the movie-player trigger's request (the game's
                        // `GameState+0x10 = out` + GameFlow pushes 6/8, `Trigger_Update` case 0xd
                        // 0x1b1f60-0x1b1f78 / `Trigger_Activate` 0x1b2318:56-61). The game's own
                        // state-6 body pops the same tick when no movie is playing
                        // (`psiMovieFinished` 0x1082e8 = `BGFMVPlaying == 0`) and NOTHING on this
                        // path starts one — the movie code 0 the disc's only record carries [D
                        // 07000041 #81] isn't even in `psiStartBackgroundMovie`'s whitelist. The
                        // port therefore consumes and drops (the observable game effect is the
                        // state-8 fade, a view concern — see the task report's live checks).
                        if let Some(code) = sim.drones.movie_request.take() {
                            eprintln!("level: movie trigger fired (code {code:08x}) — states 6/8 push and pop (no movie started on this path [M 0x1082e8])");
                        }
                        if !self.noclip {
                            let h = sim.bond.head();
                            self.cam = Camera { pos: Vec3::new(h[0], h[1], h[2]), yaw: sim.bond.yaw, pitch: sim.bond.pitch * std::f32::consts::FRAC_PI_2, fov: GAME_FOV / sim.zoom };
                            // mp-local-look (online aim lagged the mouse): the lockstep
                            // runs this player's turn/look `input_delay` ticks late (3 at least, up to
                            // 15 with the adaptive delay over the internet). The camera adds the turn/
                            // look already scheduled but not yet run, plus the mouse gathered since the
                            // last tick, so the view answers at once; the shared sim (shots, the body
                            // other players see) keeps the delayed input. Not while aiming — the aim
                            // mode spends the turn on the crosshair first (Sim::step_bond's cursor arm).
                            if let Some(n) = self.net.as_ref() {
                                if !sim.aiming && !sim.disabled {
                                    let (pt, pl) = n.0.borrow().pending_local_look(self.net_tick.wrapping_sub(1));
                                    self.cam.yaw += pt + self.turn;
                                    let pitch = (sim.bond.pitch + pl + self.look).clamp(-1.0, 1.0);
                                    self.cam.pitch = pitch * std::f32::consts::FRAC_PI_2;
                                    // Measurement aid (NF_CAMLOG, the 0.1.18 release check): the view's
                                    // yaw against the shared sim's each frame — the lead is the input the
                                    // lockstep has scheduled but not yet run.
                                    if std::env::var_os("NF_CAMLOG").is_some() {
                                        eprintln!("[camlog] tick {} sim_yaw {:.4} cam_yaw {:.4} pending_turn {:.4}", self.net_tick, sim.bond.yaw, self.cam.yaw, pt);
                                    }
                                }
                            }
                            // A script's cut-scene camera takes over the view (`Camera_Enable(4)`).
                            if let Some((p, yaw, pitch, fov)) = sim.script_camera() { self.cam = Camera { pos: Vec3::new(p[0], p[1], p[2]), yaw, pitch, fov }; }
                            // fix-sentinel-camera-play [M `Player_PositionCamera` 0x18f5d8 case 0xc]: while the
                            // guided camera rides the missile, the view IS the missile's (`Sim::view_override`
                            // = its pos/dir) — without it the view stayed in first person: the window never
                            // consulted the override (the sim side rode; the matrix's gcam saw it). The HUD
                            // ammo pane stays; the view model and the crosshair do not draw (frame_state's
                            // ride hide + hud.rs's gate).
                            if let Some((p, d)) = sim.view_override() {
                                self.cam = Camera { pos: Vec3::new(p[0], p[1], p[2]), yaw: d[0].atan2(d[2]), pitch: d[1].clamp(-1.0, 1.0).asin(), fov: GAME_FOV };
                            }
                            // fix-wallhug-prompt [M `Player_PositionCamera` 0x18f5d8 case 0xb → `camera_tracking`
                            // 0x1ba5e0]: the wall shimmy's camera is the game's third-person tracking camera
                            // (third person, not first).
                            if let Some((p, yaw, pitch, fov)) = sim.creep_camera() {
                                self.cam = Camera { pos: Vec3::new(p[0], p[1], p[2]), yaw, pitch, fov: GAME_FOV / fov };
                            }
                            // nis-event9-cnis [M `Player_PositionCamera` 0x18f5d8 case 2]: NIS event
                            // 9's behind-Bond cut-scene orbit (`Player_Cam2Mode` 0x195fc8 mode 3 →
                            // `Player_SetCamMode(BL, 2)`) — 10 units behind on the XZ plane at Bond's
                            // own Y, looking at him, unzoomed (`viewer+0x108 = 1.0` [M :154]).
                            if let Some((p, yaw, pitch, fov)) = sim.cam2_camera() {
                                self.cam = Camera { pos: Vec3::new(p[0], p[1], p[2]), yaw, pitch, fov: GAME_FOV / fov };
                            }
                        }
                        // Dev readout in the window title (the real HUD panes are the draw code's):
                        // the game's own pane inputs — health BL+0x894 [M HUD_UpdateHealthPane
                        // 0x19b9c8:159] and the hit flash BL+0x968 [M 0x19b9c8:92-113].
                        if before / 30 != self.anim_time as u32 / 30 {
                            if let Some(g) = &self.gfx {
                                let h = &sim.drones.health;
                                let hit = if h.flash > 0.0 { "  ** HIT **" } else { "" };
                                // The last pickup message (`Text_AddMsg`, shown 180 ticks).
                                let msg = sim.drones.taken.last().filter(|(f, _)| sim.drones.frame < f + 180).map_or(String::new(), |(_, t)| match t {
                                    nf_game::pickup::Taken::Weapon { weapon } => format!("  picked up weapon {weapon:#x}"),
                                    nf_game::pickup::Taken::Ammo { weapon, rounds } => format!("  picked up {rounds} rounds (weapon {weapon:#x})"),
                                    nf_game::pickup::Taken::Key { id } => format!("  picked up key {id}"), // 0x2000010 [M 0x18e650:285-289]
                                    nf_game::pickup::Taken::Armour => "  picked up armour".to_string(),   // 0x200000F [M 0x18e650:290-294]
                                    nf_game::pickup::Taken::Bonus => "  picked up a 007 bonus".to_string(), // 0x200004A [M 0x18e650:315-321]
                                    nf_game::pickup::Taken::Label { .. } => "  picked up the item".to_string(), // case 4 [M 0x18e650]; the real message binds the label (scene.rs)
                                    nf_game::pickup::Taken::Message => "  picked up a pickup".to_string(), // 0x2000013 [M 0x18e650 case 5]
                                });
                                g.window.set_title(&format!("nf-view — Bond health {:.0}{}  P2K {} / {}{hit}{msg}", h.health, if h.health <= 0.0 { " (dead)" } else { "" }, sim.gun.clip, sim.gun.spare));
                            }
                        }
                    }
                    // net-internet: the inputs this frame's ticks scheduled leave now, not at the
                    // next frame's poll (a frame less of latency on every remote input).
                    if let Some(session) = self.net.as_ref() { session.0.borrow_mut().flush(); }
                    // Dev mode after-tick: god mode / infinite ammo (the game's own state, set the way the game sets it).
                    frame_part(0, fp_t0.elapsed());
                    if debrief_opened { self.grab(false); } // the menus own the cursor again
                    if let Some(d) = self.dev.as_deref_mut() { d.after_tick(&mut self.scene); }
                    // The mission's end sequence (`Mission_Update` case 4 → `ResetMap_LevelToLoad`
                    // of the outro or the front end): the same front-end path the F8 hook drives.
                    if let Some(ch) = self.force_channel {
                        if let Some(sim) = self.scene.sim.as_mut() {
                            if sim.drones.frame >= 120 { sim.drones.set_channel(ch); self.force_channel = None; eprintln!("[dev] channel {ch} raised"); }
                        }
                    }
                    if std::mem::take(&mut self.pending_mission_end) { self.demo_results(true); }
                    // mp-match-end: the debriefing handover, drained beside the mission end.
                    if let Some((board, settings)) = self.pending_mp_debrief.take() { self.mp_results(board, settings); }
                }
        // fe-60hz: the front end is the game's 60 Hz menu tick (`MenuManager_Update` 0x21c3d8 runs once
        // per game frame, NTSC 60 Hz [M main 0x109080's vblank pacing]). Ticking it once per RENDERED
        // frame ran every menu timer — the 2700-tick attract countdown, the P_INTRO/P_ESTHERO movie
        // budgets, the Press START flash — 2.4× fast on a 144 Hz display (attract after
        // ~19 s instead of 45; the boot intro cut short). Its own clock, at most 8 ticks per frame.
        // Moved here from the draw half (headless-app-harness): the draw built its HUD from the
        // same post-tick state, and the async-load early return above skipped it exactly as now.
        if let Some(fe) = self.fe.as_mut() {
            self.fe_clock += dt;
            let fe_n = (self.fe_clock * 60.0) as u32;
            self.fe_clock -= fe_n as f32 / 60.0;
            for _ in 0..fe_n.min(8) { fe.tick(); }
        }
    }

    /// headless-app-harness: the one movie-plane decision, shared by the draw half and the
    /// headless harness (the harness pumps `movie::present()` on exactly the frames the
    /// window's draw would). Same reads as the RedrawRequested block below.
    fn movie_plane_up(&self) -> bool {
        let state6 = self.fe.as_ref().is_some_and(|fe| fe.flow.get_state() == menu::ST_MOVIE);
        let intro_up = self.fe.as_ref().is_some_and(|fe| fe.intro_playing);
        let attract_up = self.fe.as_ref().is_some_and(|fe| fe.attract_playing);
        let flow_up = state6 && !intro_up && self.movie.is_some();
        let aux_up = self.movie.as_ref().is_some_and(|p| p.is_looping())
            && !self.fe.as_ref().is_some_and(|fe| fe.flow.movie_pending);
        intro_up || flow_up || aux_up || attract_up
    }

    /// The RedrawRequested draw half (everything after `frame` that needs the GPU):
    /// interpolation palettes, the surface, the HUD/menu/movie overlay, the render, present,
    /// the fps cap. No game logic — `frame` ran it already.
    fn draw_frame(&mut self, dt: f32) {
                if NET_SHOT_DUE.swap(false, std::sync::atomic::Ordering::Relaxed) { let m = self.spot_shot(); eprintln!("[spot] {m}"); }
                if bugreport::DUE.swap(false, std::sync::atomic::Ordering::Relaxed) { let m = self.bug_report(); eprintln!("[bug-report] {m}"); }
                let Some(g) = &mut self.gfx else { return };
                if g.minimized {
                    // Keep the game ticking at a modest pace without drawing.
                    std::thread::sleep(std::time::Duration::from_millis(16));
                    g.window.request_redraw();
                    return;
                }
                let (r, u) = cam_axes(&self.cam);
                self.opts.right = r;
                self.opts.up = u;
                let interp0 = !self.noclip && !self.paused && self.scene.sim.is_some() && std::env::var_os("NF_NOINTERP").is_none()
                    && self.dev.as_deref().map_or(true, |d| d.interp);
                // G-043 (fix-interp-camera-bracket): rebuild the palette AND advance the camera
                // bracket HERE, before the draw camera and the blend fraction are taken. The
                // bracket is one thing: `cam_prev`/`cam_cur` and `pal_prev`/`pal_cur` must be the
                // same two ticks. It used to be two: `draw_cam` was read from the OLD bracket
                // while `pal_prev`/`pal_cur` were blended from the NEW pair, so every frame that
                // ran a tick drew the camera one whole tick behind the world. Measured on
                // 0700004a's lift (drawn eye y, tick = -0.0303 u): -22.72629, -22.76416,
                // -22.77174, -22.77932, -22.75659 - a 0.75-tick BACKWARD jump every fourth frame
                // (21 px at 1 m), i.e. the double image on moving things, on every level, interp on only.
                let pal_rebuilt = self.ticked || self.pal_cur.is_none();
                if pal_rebuilt { PAL_REBUILDS.fetch_add(1, std::sync::atomic::Ordering::Relaxed); }
                if pal_rebuilt {
                    // Culling applies to map geometry only — characters, doors, pickups and props are never room-culled; only the
                    // World draws take the room/portal cull (render()).
                    let fp_pal = std::time::Instant::now();
                    let new = self.scene.skin_palette(&g.renderer.skin_layout, self.choice, self.anim_time, self.opts, None);
                    frame_part(3, fp_pal.elapsed());
                    g.renderer.char_light_centre = self.scene.char_light_centres();
                    self.pal_prev = self.pal_cur.take();
                    // perf-240: the entries that move between the two ticks, and the blend buffer seeded
                    // with the newer tick (every other entry blends to it).
                    self.pal_moving.clear();
                    if let Some((pp, _)) = &self.pal_prev {
                        if pp.len() == new.0.len() {
                            self.pal_moving.extend(pp.iter().zip(&new.0).enumerate().filter(|(_, (x, y))| x != y).map(|(i, _)| i as u32));
                        }
                    }
                    self.pal_blend.clear();
                    self.pal_blend.extend_from_slice(&new.0);
                    self.pal_cur = Some(new);
                    self.beam_entries = self.scene.beam_entries(&g.renderer.skin_layout);
                }
                let (draw_cam, alpha0) = draw_bracket(&mut self.cam_prev, &mut self.cam_cur, self.cam, pal_rebuilt, self.anim_time.fract(), interp0);
                let aspect0 = g.config.width as f32 / g.config.height as f32;
                // fix-cutscene-letterbox: a live script camera frames the view at the game's own
                // 4:3 aspect and pillar-boxes the render into the band (the game's letterbox bars
                // ride inside it). Gameplay keeps the window's widescreen. Same condition the HUD
                // gate (`overlay_for`) drives the cut-scene HUD blank from.
                let cutscene = self.scene.sim.as_ref().is_some_and(|s| s.script_camera().is_some());
                // cutscene-side-strips: the front end owns the AV options' widescreen row
                // (0x10000223 → viewer+0x114 [M 0x1baf70:28-32]); feed the same aspect the
                // cut-scene view builds at (`Sim::view_aspect`, the gun tilt's input) and frame
                // the cut-scene with it — see cutscene_frame. The game's default (row off) is 4/3.
                let game_aspect = self.fe.as_ref().map_or(4.0 / 3.0, |fe| fe.view_aspect());
                if let Some(sim) = self.scene.sim.as_mut() { sim.view_aspect = game_aspect; }
                let (proj_aspect, band) = if cutscene { cutscene_frame(g.config.width as f32, g.config.height as f32, game_aspect) } else { (aspect0, [0.0; 4]) };
                let (vp0, _) = draw_cam.view_proj(proj_aspect);
                // One visibility walk per frame, shared by the palette's object cull and render's
                // World cull (fix-portal): the eye's cel from `Camera_CheckLocation` 0x1bae90's
                // persistent state, the walk from it [M 0x1e86e8].
                // The Toggles page's portal-culling row: off passes no world, so the walk returns
                // everything (the whole level draws — the isolation view).
                let portal = self.dev.as_deref().map_or(true, |d| d.portal_cull);
                let door_closed = if portal { self.scene.sim.as_ref().map(|s| s.drones.doors.closed_flags()) } else { None };
                let vis = g.renderer.cel_visibility(draw_cam.pos.to_array(), vp0, if portal { self.scene.sim.as_ref().map(|s| &s.world) } else { None }, door_closed.as_deref());
                let interp = interp0;
                let alpha = alpha0;
                let fp_blend = std::time::Instant::now();
                if let Some((pal, cols)) = &self.pal_cur {
                    match &self.pal_prev {
                        Some((pp, _)) if interp => {
                            let vm_entries = view_model_entries(&self.scene, &g.renderer.skin_layout);
                            let blended = &mut self.pal_blend;
                            if pp.len() == pal.len() && blended.len() == pal.len() {
                                lerp_palette_moving(blended, pp, pal, &self.pal_moving, vm_entries.as_deref().unwrap_or(&[]), alpha);
                            } else {
                                *blended = lerp_palette(pp, pal, alpha);
                            }
                            // fix-interp-ghosting: the view model (posed in world space from each
                            // tick's camera [M Player_PositionGun 0x1a3b40]) re-attaches to the
                            // interpolated draw camera — the same camera-space scheme lerp_cam
                            // uses for the view — so mid-turn the gun cannot swim against the
                            // view. With a live script camera the tick cams are not Bond's: the
                            // gun keeps the plain world blend there.
                            if self.scene.sim.as_ref().map_or(true, |s| s.script_camera().is_none()) {
                                if let (Some(entries), Some(cp), Some(cc)) =
                                    (vm_entries.as_ref(), self.cam_prev, self.cam_cur) {
                                    reattach_view_model(blended, pp, pal, alpha, &cp, &cc, &draw_cam, entries);
                                    reattach_beams(blended, pp, pal, alpha, &cp, &cc, &draw_cam, &self.beam_entries);
                                }
                            }
                            g.renderer.upload_skin(&g.queue, blended, cols);
                        }
                        _ => g.renderer.upload_skin(&g.queue, pal, cols),
                    }
                    // fix-gpu-facial-morphs: the facial morph weights ride the same tick — the
                    // shader blends them into the bind-pose vertices before the palette (the
                    // shot path's `update_skin` does the same upload).
                    // perf-240: the weights are tick state (no interpolation) — upload them when the
                    // palette is rebuilt (each tick, and the first frame of a scene), not every frame.
                    if pal_rebuilt || self.scene.sim.is_none() { g.renderer.upload_morph_weights(&g.queue, &self.scene); }
                }
                frame_part(4, fp_blend.elapsed());
                let frame = match g.surface.get_current_texture() {
                    wgpu::CurrentSurfaceTexture::Success(f) | wgpu::CurrentSurfaceTexture::Suboptimal(f) => f,
                    wgpu::CurrentSurfaceTexture::Outdated => { g.surface.configure(&g.device, &g.config); g.window.request_redraw(); return; }
                    wgpu::CurrentSurfaceTexture::Lost => {
                        // Recreate the surface (`Lost` cannot be fixed by configuring the old one).
                        match g.instance.create_surface(g.window.clone()) {
                            Ok(sf) => { g.surface = sf; g.surface.configure(&g.device, &g.config); }
                            Err(e) => eprintln!("surface lost and not recreated: {e}"),
                        }
                        g.window.request_redraw();
                        return;
                    }
                    // Timeout, occluded, a validation error: try again next frame.
                    _ => { g.window.request_redraw(); return; }
                };
                let view = frame.texture.create_view(&Default::default());
                let aspect = proj_aspect;
                let (_, vp_rot) = draw_cam.view_proj(aspect);
                let vp = vp0;
                let snow = snow_for(&self.scene, &draw_cam);
                // fix-menus: a menu page up → the menu overlay replaces the HUD. fix-menu-movies2:
                // the ONE movie plane (gpu.rs) serves every movie — the state-6 briefing/outro
                // (fix-movie-playback), the P_INTRO pair and the page's looping backdrop
                // (fix-menu-movies) — and draws BEFORE the HUD runs, so the menu pages' controls
                // paint over their backdrop exactly like the game's page background
                // [M PS2UploadBGMImage 0x108740 + movies.md §Presentation].
                // The borrow rules: read the state first, then pull the frame — `movie_frame`
                // holds a borrow into `self.movie` until render() consumes it.
                let state6 = self.fe.as_ref().is_some_and(|fe| fe.flow.get_state() == menu::ST_MOVIE);
                // fe-audit: the intro run is a menu background movie, not GameFlow state 6.
                let intro_up = self.fe.as_ref().is_some_and(|fe| fe.intro_playing);
                // attract-mode: the attract movie owns the screen like the intro does — the
                // attract page has no controls [L the menu-def's 0x40000035 record], the movie
                // plane IS the page [M P_ATTRACT_Handler 0x20a1f0's psiStartBackgroundMovie].
                let attract_up = self.fe.as_ref().is_some_and(|fe| fe.attract_playing);
                let flow_up = state6 && !intro_up && self.movie.is_some();
                // The looping page player is the backdrop under the overlay; the early-started
                // briefing player (non-looping, held through ST_MAPLOAD) is not.
                let aux_up = self.movie.as_ref().is_some_and(|p| p.is_looping())
                    && !self.fe.as_ref().is_some_and(|fe| fe.flow.movie_pending);
                let movie_up = intro_up || flow_up || aux_up || attract_up;
                let movie_frame = if movie_up { self.movie.as_mut().and_then(|p| p.present()) } else { None };
                // The MP standings keep a constant on-screen text size (the lobby's view_h rule).
                hud::set_view_h(g.config.height as f32);
                let mut hud = if let Some(fe) = self.fe.as_mut() {
                    let lang = self.lang.as_ref().or_else(|| self.scene.sim.as_ref().and_then(|s| s.lang.as_ref()));
                        // fe-60hz: the front end's 60 Hz tick moved into `frame` (the draw half
                        // consumed the same post-tick state — headless-app-harness).
                    fe.backdrop_live = aux_up && movie_frame.is_some();
                    if flow_up || (intro_up && movie_frame.is_some()) || (attract_up && movie_frame.is_some()) || fe.results_queued {
                        // The movie plane replaces the overlay: the state-6 briefing blits over the
                        // empty backdrop and the fog pass is suppressed while a movie plays [M
                        // DecodeBGFMVFrame 0x108550 + PS2UploadBGMImage 0x108740, psiDrawFogPass
                        // 0x10b960]; the intro FMV owns the screen the same way [M 0x209e88]. The
                        // quad itself is drawn inside render(), before the (empty) HUD runs.
                        // reward-after-outro: the queued results page draws nothing either — the
                        // game has no MenuManager until the load's tail [M 0x1d6360:160-163]; the
                        // mission's last frame (under the GameFlow fade) shows until the movie
                        // starts, exactly like the game's state-7 window.
                        gpu::HudFrame::default()
                    } else {
                        // fix-pause-menu2: the OBJECTIVES tab's rows come from the sim's mission
                        // states (Mission_ObjectiveState's data [M 0x204408:251-271]; state 4 =
                        // the done marker), refreshed while the pause page is up.
                        // The burning woman (crate::woman): `Game_Run` 0x1c9790:36-41 runs `psiDecompressWoman` once per
                        // frame in the paused branch — the pause page and the fail menu both pause the game.
                        let shows_woman = fe.page == menu::Page::Pause || (fe.page == menu::Page::Results && fe.outcome == menu::Outcome::Failed);
                        if shows_woman {
                            if let Some(w) = self.scene.woman.as_mut() {
                                let mut changed = false;
                                // The paused frames' own 60 Hz clock: `frame_ticks` is the GAME's tick count and stays frozen
                                // at its last value while a menu is up, so ticking by it ran the figure once per render frame
                                // × that stale count (several times too fast on a fast display).
                                self.menu_clock += dt;
                                let n = (self.menu_clock * 60.0) as u32;
                                self.menu_clock -= n as f32 / 60.0;
                                for _ in 0..n.min(8) { changed |= w.tick(); }
                                if changed { g.renderer.update_woman(&g.queue, &w.rgba()); }
                            }
                        }
                        if fe.page == menu::Page::Pause {
                            if let Some(sim) = self.scene.sim.as_ref() { fe.pause_objectives = pause_objective_rows(sim); fe.stats = pause_stats(sim); }
                        }
                        fe.view_h = g.config.height as f32;
                        let mut f = match &self.scene.hud {
                            Some(tables) => menu::menu_overlay(fe, &tables.font, &tables.font_header, lang, &self.scene.hud_textures, aspect),
                            None => overlay_for(&self.scene, aspect),
                        };
                        // fix-pause-menu2: the in-game HUD stays visible under the pause page —
                        // the game's pause menu draws OVER the paused frame and its HUD (radar
                        // ring, ammo pane) [L PCSX2 captures; the
                        // port replaced the HUD with the overlay]. The HUD's quads go FIRST so
                        // the menu draws over them (frame order = draw order).
                        if fe.page == menu::Page::Pause && !matches!(fe.flow.get_state(), menu::ST_MOVIE) {
                            // The menu's runs shift by the HUD's vertex count (the HUD's vertices go first) and the HUD
                            // keeps its own runs — the old merge shifted by the MENU's count and dropped the HUD's runs,
                            // so the live pause page drew from the wrong vertices (the plate, the figure and most rows
                            // missing, the cursor a square) and the radar/ammo vanished while paused.
                            let hud_frame = overlay_for(&self.scene, aspect);
                            let base = hud_frame.v.len() as u32;
                            let menu_runs = std::mem::take(&mut f.runs);
                            let mut v = hud_frame.v;
                            v.extend(f.v.drain(..));
                            f.runs = hud_frame.runs;
                            for (tex, blend, start, count) in menu_runs {
                                f.runs.push((tex, blend, start + base, count));
                            }
                            f.v = v;
                        }
                        f
                    }
                } else { overlay_for(&self.scene, aspect) };
                if let Some(d) = self.dev.as_deref() { if d.overlay || d.menu { hud.append(&dev::hud(d, &self.scene, aspect, vp)); } }
                if self.fps_meter.shown > 0.0 { hud.append(&dev::fps_counter_hud(&self.scene, aspect, self.fps_meter.shown)); }
                // The scene's script overlay belongs to the 3D view — off a movie frame and off the
                // front-end level (`scene_overlay_up`).
                if scene_overlay_up(&self.scene, movie_frame.is_some()) { hud.append(&self.scene.script_overlay_hud(aspect)); }
                g.renderer.lights = self.scene.frame_lights();
                g.renderer.screen_aspect = g.config.width as f32 / g.config.height.max(1) as f32;
                // The destroyed-glist swap's draw side: the dead Destroy leaves' centres
                // (see Scene::destroy_centres) — the doorway ends open [L] once the leaf is dead.
                g.renderer.destroy_dead = dead_destroy_centres(&self.scene);
                g.renderer.destroy_tint = destroy_tints(&self.scene);
                let fp_enc = std::time::Instant::now();
                g.renderer.set_shards(&g.queue, &self.scene.shard_draws());
                let enc = g.renderer.render(&g.device, &g.queue, &view, &g.depth, vp, vp_rot, draw_cam.pos.to_array(), self.anim_time as u32, &snow, &hud, movie_frame, if cutscene { Some(band) } else { None });
                let fp_sub = std::time::Instant::now();
                frame_part(5, fp_sub - fp_enc);
                frame_part(1, fp_sub - self.frame_start);
                g.queue.submit([enc.finish()]);
                g.queue.present(frame);
                frame_part(2, fp_sub.elapsed());
                if let Some(w) = self.devwin.as_mut() { w.tick(); }
                if let Some(fps) = self.fps_cap.filter(|f| *f > 0.0) {
                    let target = std::time::Duration::from_secs_f32(1.0 / fps);
                    // fps-cap: sleep to ~1.5 ms short of the deadline, then spin — Windows' sleep
                    // overshoots by a scheduler tick, which held `--fps 240` at ~222 fps.
                    let margin = std::time::Duration::from_micros(1500);
                    let el = self.frame_start.elapsed();
                    if el + margin < target { std::thread::sleep(target - el - margin); }
                    while self.frame_start.elapsed() < target { std::hint::spin_loop(); }
                }
                // `NF_FPSLOG=1`: one stderr line per second — frames, average and worst frame time.
                if std::env::var_os("NF_FPSLOG").is_some() { fps_log(self.frame_start.elapsed()); }
                self.fps_meter.frame();
                self.frame_start = std::time::Instant::now();
                g.window.request_redraw();
    }

    /// headless-app-harness: the whole KeyboardInput arm, no winit objects beyond the plain
    /// KeyCode/ElementState data. `k` = the physical key, `state` = pressed/released,
    /// `repeat` = the OS auto-repeat. Returns TRUE when the app asks to exit — the bare
    /// viewer's un-grabbed Esc (the window event loop turns it into `el.exit()`; the
    /// headless harness asserts it never fires in a mission).
    fn app_key(&mut self, k: KeyCode, state: ElementState, repeat: bool) -> bool {
                // The skip is the game's `Input_Action(-1, 0x19, 1)` read in `SP_Update` state 2 →
                // `Script_FFwd(cur, 1)` unless the script is a death NIS [M SP_Update 0x1dbda8:129-141,
                // Script_FFwd 0x1dac78]. Action 0x19 is bound to raw pad mask 0x840 — R1 (0x800) or
                // D-pad DOWN (0x040), pad 0 only [M psiInput_MapInputs 0x10cd80:415-417] — so MOVEMENT
                // NEVER SKIPS (a W press must not skip the 07000007 Kimiko scene). The port maps that action to the same two buttons' PC keys: ArrowDown
                // (the D-pad-down key) and R (R1) — a keyboard binding cannot carry [M]/[D]/[L]
                // (the PS2's only input surface is the pad, which psiInput_MapInputs reads), so it
                // is the port's documented input adapter (cutscene_skip_key; docs/systems/input.md
                // §3 holds the pad side). The viewer's dedicated keys keep their own functions and
                // never skip; `--skip-cutscenes` (lib.rs 1556) still skips every tick as a dev override.
                if state == ElementState::Pressed && !repeat
                    && cutscene_skip_key(k)
                    && self.scene.sim.as_ref().is_some_and(|s| s.drones.scripts.cutscene_playing()) {
                    if let Some(sim) = self.scene.sim.as_mut() { sim.drones.scripts.skip_request = true; }
                }
                // fix-menus: while a menu page is up, the keys drive the menu and never the game.
                // F12 stays the viewer's screenshot — fix-pause-menu: it previously fell through to
                // the menu branch (key_action(F12) is None → early return), so the pause menu could not
                // be captured.
                if state == ElementState::Pressed && k == KeyCode::F12 {
                    let m = self.spot_shot();
                    eprintln!("[spot] {m}");
                    return false
                }
                // Player bug report (bugreport.rs): F10 works on every screen, menus included.
                if state == ElementState::Pressed && k == KeyCode::F10 && !repeat {
                    let m = self.bug_report();
                    eprintln!("[bug-report] {m}");
                    return false
                }
                // No menu input while a level load runs: the loading screen owns the frame and the
                // game's main loop runs no menu manager inside ResetMap_Load [M 0x1d6360 state 3].
                // Keys pressed there used to reach the still-open front end — a second Enter on
                // the MP confirm page started a second load of the same arena.
                if state == ElementState::Pressed && self.fe.is_some() && self.async_load.is_some() { return false }
                if state == ElementState::Pressed && self.fe.is_some() {
                    // The game's own skip keys for state 6 (the briefing/outro movie): action 0x19
                    // is R1/D-pad-down, mask 0x840, pad 0 [M psiInput_MapInputs 0x10cd80:416], and
                    // GameFlow case 6 skips the movie THE MOMENT that action fires — Input_Action(-1,
                    // 0x19, 1) with NO time gate [M GameFlow_Main 0x1c98c8:145-149]. The port binds
                    // the same two buttons' PC keys (cutscene_skip_key) to that skip here, ungated —
                    // the in-mission arm above already maps them when a script plays. Enter/Confirm
                    // keeps its 1.0 s guard in menu_input (it is not the game's skip button).
                    if !repeat
                        && cutscene_skip_key(k)
                        && self.fe.as_ref().is_some_and(|fe| fe.flow.get_state() == menu::ST_MOVIE)
                    {
                        if let Some(fe) = self.fe.as_mut() { fe.flow.skip_movie(); }
                        return false
                    }
                    // net-online-lobby: the join address page takes the raw keys FIRST — digits,
                    // dots and the colon type, Backspace deletes, Ctrl+C/Ctrl+V work the page's
                    // own clipboard — and every other press is swallowed, so the letter rows
                    // (W/A/S/D are the menu's own keys) never fire while an address is typed
                    // [D — the page is the port's own online surface: the shipped game has no net
                    // layer at all (the ACTION.ELF symtab carries no socket/inet/htons/sockaddr
                    // symbol; its only _bind/connection names are SIF-RPC, the multitap driver and
                    // SFX listeners), so there is no game rule to port; the typing is the port's
                    // documented protocol, docs/systems/net-online-lobby.md §2.2]. Enter/Escape
                    // still confirm and back out below for the non-typing keys.
                    let ctrl = self.keys.contains(&KeyCode::ControlLeft) || self.keys.contains(&KeyCode::ControlRight);
                    // The host page's port row types digits too (Backspace deletes); its other keys
                    // (arrows, Enter, Esc) still drive the page below.
                    if let Some(fe) = self.fe.as_mut().filter(|fe| fe.page == menu::Page::MpHost && fe.cursor == 0 && !fe.mp_hosting) {
                        let typed = match k {
                            KeyCode::Backspace => fe.mp_net_backspace(),
                            _ => menu::net_char_key(k).is_some_and(|c| fe.mp_net_type(c)),
                        };
                        if typed { return false }
                    }
                    // net-p2p: the host's lobby — Ctrl+C copies the join code to the system
                    // clipboard, Ctrl+V reads a friend's code from it and starts punching.
                    if ctrl && matches!(k, KeyCode::KeyC | KeyCode::KeyV) {
                        if let Some(fe) = self.fe.as_mut().filter(|fe| fe.mp_hosting && fe.page == menu::Page::MpJoin) {
                            if let Some(sess) = fe.mp_session.as_mut() {
                                if k == KeyCode::KeyC {
                                    if let Some(code) = sess.reach().and_then(|r| r.code) { clipboard::set(&code); }
                                } else if let Some(text) = clipboard::get() {
                                    sess.expect_peer(text.trim());
                                }
                            }
                            return false;
                        }
                    }
                    let typing = self.fe.as_ref().is_some_and(|fe| fe.page == menu::Page::MpNetJoin);
                    if typing {
                        let fe = self.fe.as_mut().unwrap();
                        match k {
                            KeyCode::Backspace => { fe.mp_net_backspace(); }
                            // net-p2p: the system clipboard first (a friend's code), then the page's own.
                            KeyCode::KeyV if ctrl => { let clip = clipboard::get().unwrap_or_else(|| fe.mp_net_clip.clone()); fe.mp_net_paste(&clip); }
                            // While dialing, Ctrl+C copies THIS machine's code (for a host that must punch).
                            KeyCode::KeyC if ctrl => {
                                fe.mp_net_copy();
                                let mine = fe.mp_session.as_ref().and_then(|s| s.reach()).and_then(|r| r.my_code);
                                clipboard::set(mine.as_deref().unwrap_or(&fe.mp_net_addr));
                            }
                            KeyCode::Enter | KeyCode::Space => { if fe.action(menu::Action::Confirm) { self.menu_after(); } }
                            KeyCode::Escape => { if fe.action(menu::Action::Back) { self.menu_after(); } }
                            _ => if let Some(c) = menu::net_char_key(k) { fe.mp_net_type(c); }
                        }
                        return false
                    }
                    // create-save: the name page's typing — a typed letter/digit presses the
                    // grid cell the game's own keyboard sends, and that cell types the FIRST BYTE
                    // of its own text [M C_KEYBOARD_Handler 0x213a98:15 (the msg-0x4b entry),
                    // :72-75 (the msg-0x2f read of the focused cell's text, whose first byte is
                    // appended to the buffer)] — the port's typed key maps to the cell id and
                    // routes through the same cn_press (cnname_char_key); a PC keyboard binding
                    // cannot carry [M]/[D]/[L] (the pad's grid is the game's only entry surface),
                    // so it is the port's documented adapter, the grid stays the reference —
                    // docs/systems/profile-session.md. Backspace is the ← cell 1000 [M :39-42,
                    // the erase]. The other keys fall to key_action below: Enter/Space press the
                    // focused cell (the ✓ holds the focus on open [M 0x213790:47-48]), Esc backs out.
                    if self.fe.as_ref().is_some_and(|fe| fe.page == menu::Page::CnName && fe.cn_profile_mode) {
                        let fe = self.fe.as_mut().unwrap();
                        let consumed = match k {
                            KeyCode::Backspace => fe.cn_press(menu::FrontEnd::CN_KEY_DELETE),
                            _ => menu::cnname_char_key(k).is_some_and(|cell| fe.cn_press(cell)),
                        };
                        if consumed { return false; }
                    }
                    // Windows key auto-repeat never confirms, backs out or skips: the game's actions
                    // are edge-triggered [M Input_Action(.., 1)], and a held Enter's repeats skipped
                    // the briefing the moment it started. Held
                    // arrows still scroll.
                    if let Some(a) = menu::key_action(k) {
                        if repeat && matches!(a, menu::Action::Confirm | menu::Action::Back) { return false }
                        self.menu_input(a);
                    }
                    return false
                }
                // fix-ingame-dev-menu: while the dev menu is open, the keys drive the menu and never the game
                // (the same rule the front-end pages follow). The sim is paused under the open menu
                // (`paused_tick_count`), so nothing can race the menu's actions. F1/F11/F12 keep their viewer
                // functions; Esc/F2 close and re-grab the mouse.
                if self.dev.as_deref().is_some_and(|d| d.menu) {
                    if state != ElementState::Pressed { return false }
                    match k {
                        KeyCode::F1 => if let Some(d) = self.dev.as_deref_mut() { d.overlay = !d.overlay; },
                        KeyCode::F2 => {
                            if let Some(d) = self.dev.as_deref_mut() { d.menu = false; }
                            if self.scene.sim.is_some() { self.grab(true); }
                        }
                        KeyCode::F11 => if let Some(d) = self.dev.as_deref_mut() { d.step = true; },
                        KeyCode::F12 => { let m = self.spot_shot(); eprintln!("[spot] {m}"); }
                        KeyCode::Escape => {
                            if let Some(d) = self.dev.as_deref_mut() { d.menu = false; }
                            if self.scene.sim.is_some() { self.grab(true); }
                        }
                        KeyCode::PageUp => if let Some(d) = self.dev.as_deref_mut() { dev::menu_key(d, dev::MenuKey::PageUp); },
                        KeyCode::PageDown => if let Some(d) = self.dev.as_deref_mut() { dev::menu_key(d, dev::MenuKey::PageDown); },
                        KeyCode::ArrowUp => if let Some(d) = self.dev.as_deref_mut() { dev::menu_key(d, dev::MenuKey::Up); },
                        KeyCode::ArrowDown => if let Some(d) = self.dev.as_deref_mut() { dev::menu_key(d, dev::MenuKey::Down); },
                        KeyCode::Backspace => if let Some(d) = self.dev.as_deref_mut() { dev::menu_key(d, dev::MenuKey::Backspace); },
                        KeyCode::Enter => {
                            // Compute the action while holding the dev borrow, drop it, then apply.
                            let action = self.dev.as_deref_mut().and_then(|d| dev::menu_activate(d, Some(&self.scene)));
                            if let Some(a) = action { self.apply_menu_action(a); }
                        }
                        _ => { if let Some(c) = dev_filter_char(k) { if let Some(d) = self.dev.as_deref_mut() { dev::menu_type(d, c); } } }
                    }
                    return false
                }
                // The viewer's debug keys (noclip, enemy fire, every channel on, the demo results, …) are for
                // dev builds and `--dev` only: a release build (NF_RELEASE_VERSION) hands those keys to the game,
                // so a player can't noclip or break a level by accident (release review 2026-10-19).
                let dbg = self.dev.is_some() || crate::launcher::update::current_version().is_none();
                match (k, state) {
                    (KeyCode::Escape, ElementState::Pressed) => {
                        // Esc in a mission ALWAYS pauses, grabbed or not: the briefing hand-over left the
                        // mouse un-grabbed, and the un-grabbed Esc quit the game from inside a mission
                        // instead of pausing. Only the bare
                        // viewer (no mission) quits on an un-grabbed Esc.
                        let in_mission = self.scene.sim.is_some() && self.level_code != 0 && self.level_code != menu::FRONTEND_LEVEL;
                        if self.grabbed || in_mission {
                            self.grab(false);
                            // Esc in a mission opens the pause page [S: the game pauses on Start].
                            // The game's pause gate: the action AND GameState+0x36==0 AND
                            // ScriptCam==0 [M Player_Update 0x193600:72] — a script camera up (the
                            // 1b intro cinema 0x0600085c, the Drake NISs) refuses the pause; the
                            // port opened one over the cinema (pause-results-flows live check 1).
                            if self.scene.sim.is_some() && self.level_code != 0 && self.level_code != menu::FRONTEND_LEVEL
                                && self.scene.sim.as_ref().and_then(|s| s.script_camera()).is_none() {
                                let base = self.scene.sim.as_ref().map(|s| s.drones.mission.base_map).filter(|&b| b != 0 && b != u32::MAX).unwrap_or(self.level_code);
                                let fresh_fe = self.fe.is_none();
                                let fe = self.fe.get_or_insert_with(|| {
                                    let mut fe = menu::FrontEnd::new();
                                    // frontend-music: the fresh manager still meets the game's
                                    // statics (menu-audio-state — the stash from the front-end close;
                                    // the quit-to-front-end load must flip to the OTHER bank).
                                    Self::inherit_music_statics(&mut fe, self.music_bank_flip, self.music_loaded_bank, self.music_master);
                                    // settings-persist: same boot as the title's front end.
                                    let (opts, warn) = settings::load_default();
                                    if let Some(w) = warn { eprintln!("[settings] {w}"); }
                                    fe.opts = opts;
                                    fe
                                });
                                // profile-session: a FRESH pause manager inherits the standing codename
                                // session [M the globals outlive the manager] — the pause's quit to
                                // the front end must keep the codename in play. (Field-disjoint reads:
                                // `fe` borrows `self.fe`, the session sits beside it.)
                                if fresh_fe {
                                    let session = self.profile_session.clone();
                                    session.apply_to(fe);
                                }
                                fe.flow.current_level = self.level_code;
                                // RESTART reloads the running mission's base map [M 0x204408:138-141
                                // Mission_BaseMapHCode()]: a level started straight from the command line never
                                // went through the mission list, so pick the row from the mission itself (row 0
                                // is the driving Paris Prelude 0x09000001, which the old default reloaded).
                                if let Some(i) = menu::CAMPAIGN.iter().position(|c| c.3 == base) { fe.mission = i; }
                                // The GameState rides the runner state while a mission plays [M
                                // GameFlow_Main 0x1c98c8 case 2 — the game's stack sits on 2 under
                                // Mission_Update], so the pause arms' ResetMap_LevelToLoad accepts
                                // (its state gate 0x1d3800:11-17 wants {2,7,8,0xd,0xe}).
                                fe.flow.stack = vec![menu::ST_RUNNER];
                                fe.open_pause();
                            }
                        } else { return true }
                    }
                    (KeyCode::F12, ElementState::Pressed) => { let m = self.spot_shot(); eprintln!("[spot] {m}"); }
                    // Player bug report (bugreport.rs).
                    (KeyCode::F10, ElementState::Pressed) => if !repeat { let m = self.bug_report(); eprintln!("[bug-report] {m}"); },
                    // fix-menus demo hooks: F8 = mission complete, F9 = mission failed (the results page).
                    (KeyCode::F8, ElementState::Pressed) if dbg => self.demo_results(true),
                    (KeyCode::F9, ElementState::Pressed) if dbg => self.demo_results(false),
                    (KeyCode::BracketRight | KeyCode::BracketLeft, ElementState::Pressed) if dbg => {
                        self.choice = match (k == KeyCode::BracketRight, self.choice) {
                            (true, None) => Some(0),
                            (true, Some(n)) => Some(n + 1),
                            (false, Some(0) | None) => None,
                            (false, Some(n)) => Some(n - 1),
                        };
                        self.anim_time = 0.0;
                        if let Some(g) = &self.gfx { g.window.set_title(&format!("nf-view — {}", self.scene.sequence_label(self.choice))); }
                    }
                    (KeyCode::KeyP, ElementState::Pressed) if dbg => self.paused = !self.paused,
                    // Dev mode keys (WORKLOG 0081): F1 = AI overlay, F2 = dev menu, F11 = step one tick (when paused).
                    (KeyCode::F1, ElementState::Pressed) => if let Some(d) = self.dev.as_deref_mut() { d.overlay = !d.overlay; },
                    (KeyCode::F2, ElementState::Pressed) => if self.devwin.is_some() { self.grab(false); if let Some(w) = self.devwin.as_ref() { w.show(); } } else if let Some(d) = self.dev.as_deref_mut() { d.menu = !d.menu; if d.menu { self.grab(false); } else if self.scene.sim.is_some() { self.grab(true); } },
                    (KeyCode::F11, ElementState::Pressed) => if let Some(d) = self.dev.as_deref_mut() { if d.time_scale <= 0.0 { d.step = true; } },
                    (KeyCode::F3, ElementState::Pressed) if dbg => self.opts.reinforcements = !self.opts.reinforcements,
                    (KeyCode::KeyF, ElementState::Pressed) if dbg => self.opts.firing = !self.opts.firing,
                    (KeyCode::KeyN, ElementState::Pressed) if dbg => { let on = !self.noclip; self.set_noclip(on); }
                    (KeyCode::KeyT, ElementState::Pressed) if dbg => { self.auto_aim_opt = !self.auto_aim_opt; if let Some(fe) = self.fe.as_mut() { fe.opts.insert(0x1000_011b, self.auto_aim_opt as i32); } if let Some(o) = self.opts_seen.as_mut() { o.insert(0x1000_011b, self.auto_aim_opt as i32); } },
                    // Weapon switching: number keys 1–9 select directly, Q = prev, E (when not use) = next.
                    // The mapping: 1 = P2K (weapon 2), 2 = SG5 Commando (0x16), 3 = sniper (0x24), etc. The exact
                    // slot mapping follows the game's weapon order (selectable weapons with `weapon_data+0x04 == 1`).
                    (KeyCode::Digit1, ElementState::Pressed) => if self.grabbed { self.weapon_switch_to = Some(2) },
                    (KeyCode::Digit2, ElementState::Pressed) => if self.grabbed { self.weapon_switch_to = Some(0x16) },
                    (KeyCode::Digit3, ElementState::Pressed) => if self.grabbed { self.weapon_switch_to = Some(0x24) },
                    // Gadgets (4-7) are disabled until at least one is ported (docs/port-map/gadgets.md).
                    // (KeyCode::Digit4, ElementState::Pressed) => if self.grabbed { self.weapon_switch_to = Some(0x4a) },
                    // (KeyCode::Digit5, ElementState::Pressed) => if self.grabbed { self.weapon_switch_to = Some(0x4e) },
                    // (KeyCode::Digit6, ElementState::Pressed) => if self.grabbed { self.weapon_switch_to = Some(0x50) },
                    // (KeyCode::Digit7, ElementState::Pressed) => if self.grabbed { self.weapon_switch_to = Some(0x54) },
                    (KeyCode::KeyQ, ElementState::Pressed) => if self.grabbed && !self.noclip { self.weapon_cycle = -1 },
                    // Gadgets (G): the cycle is live — the weapon_data's +0x04 selectable flag is 1 for
                    // the gadget ids [D ACTION.ELF weapon_data] and the switch/equip path is tested
                    // (gadget_select_live). The gadgets' fire-side substates land per gadget.
                    (KeyCode::KeyG, ElementState::Pressed) => if self.grabbed && !self.noclip { self.gadget_cycle = 1 },
                    // The fire-mode button is the game's ○ (action 12); the keyboard binding is B (a port choice),
                    // which the catch-all below already lands in `just` (read in the Input build:
                    // `mode_switch`) [M Player_Weapon 0x1a24c8: `Input_Action(slot, 0xc, 4)`].
                    // Turn every switch channel on: drones waiting for one (WaitSwitch) start (triggers are not ported).
                    (KeyCode::KeyK, ElementState::Pressed) if dbg => if let Some(sim) = self.scene.sim.as_mut() { for c in 1..=255 { sim.drones.set_channel(c) } },
                    // Bug 3: log the name of the object Bond is touching (its AABB contains his feet).
                    (KeyCode::KeyC, ElementState::Pressed) if dbg => {
                        let p = if let Some(sim) = self.scene.sim.as_ref() { sim.bond.pos } else { [self.cam.pos.x, self.cam.pos.y, self.cam.pos.z] };
                        let mut hits: Vec<&str> = self.scene.colliders.iter()
                            .filter(|(_, lo, hi)| p[0] >= lo[0] - 0.5 && p[0] <= hi[0] + 0.5 && p[2] >= lo[2] - 0.5 && p[2] <= hi[2] + 0.5 && p[1] >= lo[1] - 2.0 && p[1] <= hi[1] + 0.5)
                            .map(|(n, _, _)| n.as_str()).collect();
                        hits.sort(); hits.dedup();
                        eprintln!("coll: at {:?} touching: {}", p, if hits.is_empty() { "(nothing in range)".to_string() } else { hits.join(", ") });
                    }
                    (_, ElementState::Pressed) => { if !repeat { self.just.insert(k); } self.keys.insert(k); }
                    (_, ElementState::Released) => { self.keys.remove(&k); }
                }
        false
    }

    /// headless-app-harness: the App skeleton with every field at its neutral value — gfx: None, no dev, no panel,
    /// no net — and the sound driver built from the scene's bank (`build_sound`; under NF_SILENT a real driver with no
    /// output device). The window `run()` and the headless harness build the SAME skeleton and apply their knobs on
    /// top, so a headless frame runs the exact per-frame path the window runs. The driver is built HERE: run() used to
    /// build it beside this call and drop it, so the front end had no driver and every menu sound was silent in every
    /// build from 2026-10-17 to v0.1.6.
    fn bare(iso: String, mut scene: scene::Scene, cam: Camera, opts: scene::ViewOpts, choice: Option<usize>, anim_time: f32) -> App {
        let sound = build_sound(&mut scene);
        App {
            sound,
            scene,
            cam,
            gfx: None,
            keys: HashSet::new(),
            grabbed: false,
            trigger: false,
            aiming: false,
            last: std::time::Instant::now(),
            choice,
            anim_time,
            paused: false,
            opts,
            noclip: false,
            just: HashSet::new(),
            turn: 0.0,
            look: 0.0,
            weapon_cycle: 0,
            gadget_cycle: 0,
            weapon_switch_to: None,
            zoom_cycle: 0,
            zoom: false,
            dev: None,
            notex: false,
            nofog: false,
            iso,
            pending_level: None,
            selected_difficulty: 2,
            pending_weapon_assets: None,
            entry_save: None,
            all: false,
            recorder: None,
            replay: None,
            replay_idx: 0,
            prev_lock_ch: false,
            ticked: false,
            frame_ticks: 0,
            menu_clock: 0.0,
            fe_clock: 0.0,
            pending_mission_end: false,
            results_progress: None,
            pending_mp_debrief: None,
            force_channel: None,
            cam_prev: None,
            cam_cur: None,
            pal_prev: None,
            pal_cur: None,
            pal_blend: Vec::new(),
            pal_moving: Vec::new(),
            beam_entries: Vec::new(),
            fps_cap: None,
            fps_meter: FpsMeter::default(),
            frame_start: std::time::Instant::now(),
            panel: None,
            panel_level: 0,
            panel_frames: 0,
            fe: None,
            auto_aim_opt: true,
            look_inverted: false,
            crouch_toggle_opt: true,
            opts_seen: None,
            level_code: 0,
            dev_bond: None,
            dev_bond_cut: false,
            fe_music_page: None,
            menu_handle: 0,
            lang: None,
            profile_session: menu::ProfileSession::default(),
            #[cfg(feature = "gamepad")]
            pad: None,
            movie: None,
            devwin: None,
            audio_paused: false,
            loading_art: None,
            loading_hint: Default::default(),
            menu_pointer: None,
            net: None,
            net_lobby: None,
            net_local_slot: 0,
            net_tick: 0,
            pending_mp_settings: None,
            net_armed: None,
            net_await_load: false,
            net_menu_load: false,
            net_late_self: None,
            net_rejoin: None,
            net_finished: None,
            net_end_wait: None,
            net_clock: None,
            net_pacer: nf_net::pace::TickPacer::new(),
            net_resyncing: false,
            net_shot_pending: None,
            desync_watch: desync::Watcher::default(),
            desync_report: None,
            desync_wait: None,
            net_join_pending: None,
            net_late_handover: None,
            async_load: None,
            music_bank_flip: false,
            music_loaded_bank: 0xc,
            music_master: 70,
        }
    }

}

impl ApplicationHandler for App {
    fn resumed(&mut self, el: &ActiveEventLoop) {
        if self.gfx.is_some() { return; }
        let window = Arc::new(el.create_window(Window::default_attributes().with_title("nf-view")).expect("create window"));
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
        let surface = instance.create_surface(window.clone()).expect("create surface");
        let (adapter, device, queue) = pollster::block_on(device(&instance, Some(&surface))).expect("GPU device");
        // A GPU validation error is reported, not fatal (wgpu's default handler panics).
        device.on_uncaptured_error(Arc::new(|e| eprintln!("GPU error: {e}")));
        let size = window.inner_size();
        let mut config = surface.get_default_config(&adapter, size.width.max(1), size.height.max(1)).expect("surface config");
        // The GS writes its colours straight to the framebuffer (no gamma step), and the shaders output those values as
        // they are — like the headless `--shot` path's Rgba8Unorm. An sRGB surface would encode them again and wash the
        // picture out, so use the plain (non-sRGB) variant of the surface's format.
        config.format = config.format.remove_srgb_suffix();
        // Vsync at the display rate by default (a 120/144 Hz display gets 120/144 fps); `--fps N` turns it off and caps.
        config.present_mode = if self.fps_cap.is_some() { wgpu::PresentMode::AutoNoVsync } else { wgpu::PresentMode::AutoVsync };
        surface.configure(&device, &config);
        let depth = gpu::depth_view(&device, config.width, config.height);
        let renderer = gpu::Renderer::new(&device, &queue, config.format, &self.scene, self.notex, self.nofog);
        // `--dev`: open the dev window beside the game (devwin.rs).
        if self.dev.is_some() && self.devwin.is_none() {
            self.devwin = devwin::DevWindow::new(el, &window, &instance, &adapter, &device, &queue, &self.scene);
            if self.devwin.is_some() {
                // The window's tabs are the index: open on Weapons.
                if let Some(d) = self.dev.as_deref_mut() { if d.page == 0 { d.page = devwin::TAB_PAGES[0]; } }
                eprintln!("dev window open beside the game (F2 raises it)");
            }
        }
        self.gfx = Some(Gfx { window, instance, minimized: size.width == 0 || size.height == 0, surface, device, queue, config, depth, renderer, adapter });
    }

    fn device_event(&mut self, _el: &ActiveEventLoop, _id: winit::event::DeviceId, event: DeviceEvent) {
        if let DeviceEvent::MouseMotion { delta: (dx, dy) } = event {
            if self.grabbed {
                if self.noclip || self.scene.sim.is_none() {
                    self.cam.yaw -= dx as f32 * MOUSE_SENSITIVITY;
                    self.cam.pitch = (self.cam.pitch - dy as f32 * MOUSE_SENSITIVITY).clamp(-1.55, 1.55);
                } else {
                    // Mouse look (PLAN.md): raw deltas straight into Bond's yaw / pitch at the next tick.
                    // When scoped, the sensitivity is divided by the zoom factor (the scope zooms the view,
                    // so the same mouse delta should turn less) [M Player_Zoom: the zoom affects the view].
                    let zoom = self.scene.sim.as_ref().map_or(1.0, |s| s.zoom);
                    self.turn -= dx as f32 * MOUSE_SENSITIVITY / zoom;
                    self.look -= dy as f32 * MOUSE_SENSITIVITY / zoom / std::f32::consts::FRAC_PI_2;
                }
            }
        }
        // Mouse wheel: with the dev menu open it moves the menu's selection; otherwise weapon
        // switching (next/prev). Up = previous row / next weapon (−1 direction, matching the old
        // weapon-cycle mapping: wheel up = next).
        if let DeviceEvent::MouseWheel { delta: MouseScrollDelta::LineDelta(_x, y) } = event {
            if self.fe.is_some() && !self.dev.as_deref().is_some_and(|d| d.menu) {
                if let Some(a) = menu::wheel_scroll_action(y) { self.menu_input(a); }   // up = previous row
            } else if self.dev.as_deref().is_some_and(|d| d.menu) {
                if let Some(d) = self.dev.as_deref_mut() {
                    let n = dev::menu_items(d, Some(&self.scene)).len();
                    if n > 0 {
                        if y > 0.0 { d.menu_sel = d.menu_sel.saturating_sub(1); } else { d.menu_sel = (d.menu_sel + 1).min(n - 1); }
                    }
                }
            } else if self.grabbed && !self.noclip {
                // microcam-wheel-zoom: while the right button (aim) is held and the held weapon can
                // zoom, the wheel steps the zoom instead of switching. The game's side: aiming a
                // scoped weapon gates the zoom on weapon_data +0x68 & 0x40 [M Player_Weapon
                // 0x1a24c8:195] and Player_Zoom 0x1a1a70 steps the magnification while action 6 —
                // the zoom axis, read as an analog Input_Actionf [M 0x1a1a70:18-19; docs/systems/
                // input.md §3 "zoom = action 6 in aim mode"] — is held. The PS2 pad has no wheel,
                // so the binding cannot carry [M]/[D]/[L]: each wheel notch is the port's discrete
                // stand-in for that analog axis, and the
                // zoomable gate below is the game's own scope flag + magnification-above-1 pair
                // [M weapon_data 0x2bf150; the sim's zoom block gates on the same two]. Otherwise
                // the wheel keeps switching weapons (the pre-feature behaviour).
                let zoomable = self.scene.sim.as_ref().is_some_and(|s| {
                    wheel_steps_zoom(self.zoom, s.weapons.flags(s.gun.weapon as usize), s.weapons.zoom(s.gun.weapon as usize))
                });
                if zoomable {
                    if y > 0.0 { self.zoom_cycle = 1; }
                    else if y < 0.0 { self.zoom_cycle = -1; }
                } else if y > 0.0 { self.weapon_cycle = 1; }
                else if y < 0.0 { self.weapon_cycle = -1; }
            }
        }
    }

    fn window_event(&mut self, el: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        // The dev window's events never reach the game (devwin.rs): its clicks/keys drive the dev menu only, and the
        // game keeps running.
        if self.devwin.as_ref().is_some_and(|w| w.id() == id) {
            self.dev_window_event(event);
            return;
        }
        match event {
            WindowEvent::CloseRequested => el.exit(),
            WindowEvent::Resized(s) => if let Some(g) = &mut self.gfx {
                // Minimised windows report 0×0; a surface cannot be configured at that size.
                g.minimized = s.width == 0 || s.height == 0;
                if !g.minimized {
                    g.config.width = s.width;
                    g.config.height = s.height;
                    g.surface.configure(&g.device, &g.config);
                    g.depth = gpu::depth_view(&g.device, g.config.width, g.config.height);
                }
                g.window.request_redraw();
            },
            // Focus lost (Alt-Tab, a click elsewhere): let the mouse go and drop held keys and the trigger.
            WindowEvent::Focused(false) => {
                self.grab(false);
                self.keys.clear();
                self.just.clear();
                self.trigger = false;
            }
            // Left button: grab the mouse, then fire (action 9) while held. With the dev menu open (F2), a left click
            // selects a menu item instead (WORKLOG 0081).
            WindowEvent::MouseInput { state, button: MouseButton::Left, .. } if self.fe.is_some() && self.async_load.is_none() && !self.dev.as_deref().is_some_and(|d| d.menu) => {
                if state == ElementState::Pressed {
                    if let (Some((x, y)), Some(fe)) = (self.menu_pointer, self.fe.as_mut()) {
                        if fe.flow.get_state() == menu::ST_MOVIE { self.menu_input(menu::Action::Confirm); }
                        else if fe.mouse_click(x, y) { self.menu_after(); }
                    }
                }
            }
            WindowEvent::MouseInput { state: ElementState::Pressed, button: MouseButton::Right, .. } if self.fe.is_some() && self.async_load.is_none() && !self.dev.as_deref().is_some_and(|d| d.menu) => {
                self.menu_input(menu::MOUSE_RIGHT_ACTION);   // menu-mouse-parity: right press = △ = Back
            }
            WindowEvent::MouseInput { state, button: MouseButton::Left, .. } => {
                if self.dev.as_deref().is_some_and(|d| d.menu) {
                    if state == ElementState::Pressed { self.dev_menu_click(); }
                } else if !self.grabbed { if state == ElementState::Pressed { self.grab(true) } } else { self.trigger = state == ElementState::Pressed }
            }
            // Right button: scope zoom (mouse-first, like the game's aim button). Held = zoomed in.
            WindowEvent::MouseInput { state, button: MouseButton::Right, .. } => {
                if self.grabbed { self.zoom = state == ElementState::Pressed; }
            }
            // Cursor moves: track the NDC position for the dev menu's click hit-test (when the menu is open).
            WindowEvent::CursorMoved { position, .. } if self.fe.is_some() && self.async_load.is_none() && !self.dev.as_deref().is_some_and(|d| d.menu) => {
                self.menu_pointer = self.menu_space(position.x, position.y);
                if let (Some((x, y)), Some(fe)) = (self.menu_pointer, self.fe.as_mut()) {
                    if fe.mouse_move(x, y) { self.menu_after(); }   // the hover's focus-move sound
                }
            }
            WindowEvent::CursorMoved { position, .. } => if let (Some(g), Some(d)) = (&self.gfx, self.dev.as_deref_mut()) {
                if d.menu {
                    let (w, h) = (g.config.width as f32, g.config.height as f32);
                    d.mouse = ((position.x as f32 / w) * 2.0 - 1.0, 1.0 - (position.y as f32 / h) * 2.0);
                }
            },
            // No manual-aim mode with the mouse (PLAN.md D-002): the mouse aims directly; `aiming` stays for a pad.
            WindowEvent::KeyboardInput { event, .. } => if let PhysicalKey::Code(k) = event.physical_key {
                // headless-app-harness: the whole arm moved into `App::app_key` (no winit
                // objects beyond the plain KeyCode/ElementState data). true = the app asks
                // to exit (the bare viewer's un-grabbed Esc — the old `el.exit()` arm).
                if self.app_key(k, event.state, event.repeat) { el.exit(); }
            },
            WindowEvent::RedrawRequested => {
                // headless-app-harness: the frame logic (no winit/wgpu) then the draw half —
                // the exact order the old inline body ran.
                let dt = self.last.elapsed().as_secs_f32().min(0.1);
                self.last = std::time::Instant::now();
                self.frame(dt);
                // While a level loads, `frame` already drew and presented the loading screen
                // (poll_async_load → draw_loading_screen) and returned early. The old inline body's
                // `return` skipped the draw; since the frame/draw_frame split (7f14552b) the scene
                // was drawn and presented right after it, so every redraw alternated loading screen /
                // old scene hundreds of times a second (a strobing flash). Skip the scene draw for the whole load.
                if self.async_load.is_none() { self.draw_frame(dt); }
            }
            _ => {}
        }
    }
}

/// The AV volume rows reach the sound layer's volumes — the game's own settings load applies
/// the persisted volumes straight to the IOP statics at boot [M LS_LoadGlobalSettings
/// 0x1cdbc8:69-70 `SFXMusicSetVolume/SFXSetVolume(loaded)`; Boot_LoadPTPData 0x1c8560:81-82].
/// The driver's `music_master`/`master` are those statics: `ES_CombineVolumes` 0x1300 scales
/// every jingle (0x1d7/0x470) by `music_master` and every other sample by `master` [M
/// nf-audio/src/sfx.rs `combine`]. The menu-music model carries the same music volume so a
/// mission-return fe rebuild inherits it (inherit_music_statics' stash seeds from the boot fe).
/// The option map stores the volume scale the restore-defaults list writes [M 0x214818:97-110
/// — slider 0xe → volume 70, sfx 0x14 → 100], so the rows map 1:1 onto the driver fields.
pub fn apply_av_volumes(fe: &mut menu::FrontEnd, driver: Option<&mut nf_audio::sfx::Driver>) {
    let music = fe.opt_value(0x1000_0135);
    let sfx = fe.opt_value(0x1000_0134);
    fe.music.music_master = music;
    if let Some(d) = driver {
        d.music_master = music;
        d.master = sfx;
    }
}

impl App {
    /// options-census: the per-frame option drain (docs/systems/options-census.md) — the
    /// auto-aim drain generalized to every wired row. Game-side these are profile globals read
    /// at the point of consumption; the port's `fe.opts` is that profile and this is the read,
    /// every frame so the values survive across missions. Volume rows commit on page exit in
    /// the game [M P_CNAVOPTIONS_Handler 0x214818:72-81] — the per-frame apply is the same net
    /// effect through the persisted profile.
    fn drain_options(&mut self) {
        // fix-909c: refresh the copy while the front end is up; read the copy while it is closed
        // (play). A run that never had a front end (the command-line mission harness) keeps the
        // Auto Aim mirror only — the dev T key and tests write it directly.
        if let Some(fe) = self.fe.as_ref() {
            if self.opts_seen.as_ref() != Some(&fe.opts) { self.opts_seen = Some(fe.opts.clone()); }
        }
        let Some(opts) = self.opts_seen.clone() else {
            if let Some(sim) = self.scene.sim.as_mut() { sim.auto_aim = self.auto_aim_opt; }
            return;
        };
        let ov = |c: u32| opts.get(&c).copied().unwrap_or(0);
        // The AV volume rows reach the sound layer [M LS_LoadGlobalSettings 0x1cdbc8:69-70's
        // SFXMusicSetVolume/SFXSetVolume — apply_av_volumes is the boot arm of the same].
        if let Some(s) = self.sound.as_mut() {
            s.driver.music_master = ov(0x1000_0135);
            s.driver.master = ov(0x1000_0134);
        }
        // Y-axis: the game negates the two analog look channels when set [M Input_Update
        // 0x17ee88:50-52]; the port negates the accumulated look delta at the input fill.
        self.look_inverted = ov(0x1000_00c9) != 0;
        self.crouch_toggle_opt = ov(0x1000_0125) != 0;
        let Some(sim) = self.scene.sim.as_mut() else { return };
        // Auto Aim: the SP row vs the MP row, picked by the match kind [M Player_AutoAim
        // 0x1a7c98:39-45 picks DAT_002a38c9 vs DAT_002a38ca on MPSettings+0x180]. The port's
        // MP matches run the shared pickup pass / carry a net session — [S] the detector, the
        // game's flag is the MPSettings word.
        let mp = self.net.is_some() || sim.drones.mp_shared_pickups;
        sim.auto_aim = ov(if mp { 0x1000_0129 } else { 0x1000_011b }) != 0;
        // Weapon Auto Switch gates the better-weapon take [M Player_EquipWeapon 0x1a8a58:76].
        sim.drones.auto_switch = ov(0x1000_01cb) != 0;
        // Crosshairs [M HUD_UpdateCrossHair 0x19e6d0:96], the near-flash tint [M
        // View_SetupRenderModes_Tweak 0x1e5888:26], the health-pane fade gate [M
        // Player_Update 0x193600:414], the Subtitles byte [M Sound_DoSubtitle 0x1dde58:21].
        sim.drones.crosshair = ov(0x1000_0124) != 0;
        sim.drones.near_flash = ov(0x1000_0226) != 0;
        sim.drones.decay_gate = ov(0x1000_023b) != 0;
        sim.drones.crouch_toggle = self.crouch_toggle_opt;
        sim.drones.subtitle_track.enabled = ov(0x1000_0136) != 0;
    }
}

/// The movie start sites' `SFXGetVolume()` [M 0x116e18: the `SfxMasterVolume` static, 100 at boot
/// [L 0x30c95c]]. Every `psiStartBackgroundMovie` caller passes it as the volume arg [M
/// Menu_PlayMovie 0x215990:18-19, P_INTRO 0x209e88:45+62, P_ATTRACT 0x20a1f0:23+36, P_FMVPLAYER
/// 0x20ad30:25-26, P_TRAILER 0x214f10:19-20, ResetMap_Load 0x1d6360]. The front end's SFX option
/// IS the port's `SfxMasterVolume`: the game's options page multiplies its 0..0x14 slider by 5
/// into the static [M P_CNAVOPTIONS 0x214818:74 `SFXSetVolume(iVar2 * 5)`, the commit :73-74, the
/// cancel restore :80-81, the enter snapshot :22-23] and the persisted settings load straight
/// into it [M LS_LoadGlobalSettings 0x1cdbc8:70 `SFXSetVolume(loaded)`], the same raw 0..100 the
/// port's map stores. A missing key boots the default 100 [L 0x30c95c; `default_opts` and
/// `settings::load_default` both fill it]. (The game's `SFXGetVolume` returns 1 while
/// `switch_CONFIG_SOUND` == 0 [M 0x116e18]; the port carries no sound-off option, so the master
/// branch is the only reachable one.)
pub fn movie_sfx_volume(fe: &menu::FrontEnd) -> u32 {
    fe.opts.get(&0x1000_0134).copied().unwrap_or(100).max(0) as u32
}

pub fn run() -> Result<()> {
    // A closed output pipe (the reader of our stdout/stderr went away) must not crash the game: the
    // log lines are diagnostics (stdio_guard.rs).
    stdio_guard::install();
    // A panic also goes to nf-view-crash.log (message, place, backtrace): a crash in a window without a console
    // leaves a trace.
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let text = format!("{info}\n{}\n", std::backtrace::Backtrace::force_capture());
        let _ = std::fs::write(crate::settings::output_dir().join("nf-view-crash.log"), &text);
        default_hook(info);
    }));
    // Player bug reports: a panic also writes a crash report the launcher offers to send (bugreport.rs).
    bugreport::install_crash_hook();
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.iter().any(|a| a == "--rapid-fire") { RAPID_FIRE.store(true, std::sync::atomic::Ordering::Relaxed); }
    const USAGE: &str = "usage: nf-view [<iso>] <level> [--all] [--seq K] [--frame F] [--fire] [--reinforcements] [--dev [--panel]] [--scenario <name>] [--record <file>] [--replay <file>] [--no-fog] [--no-tex] [--shot out.png [--cam x y z yaw pitch] [--dev-page <name>]]\n\
        <level> is a FILES.BIN name such as 07000005 (\".bin\" optional); without <iso> the image path comes from NF_ISO\n\
        --dev --scenario <name>: place Bond for a feature test (weapon-switch, sg5-pickup, wire-music, autoaim)\n\
        --record <file>: record every logic tick's input to <file> (B-5). --replay <file>: replay a recording.\n\
        --no-fog: skip the fog screen pass (isolate vertex colour/texture). --no-tex: ignore texture samples (solid white, isolate lighting/fog).";
    // --movie <name/code> --frame N --shot out.png: headless PSS movie test (fix-movies) — demux, decode one
    // frame of the disc's MPEG-2 video, dump it as a PNG (no window, no GPU). The game plays these through
    // psiStartBackgroundMovie 0x001081a8 (cdrom0:\movies\30_FPS\%8.8x.pss;1). Needs `--features movies`.
    #[cfg(feature = "movies")]
    if let Some(m) = args.iter().position(|a| a == "--movie") {
        let name = args.get(m + 1).context("--movie needs a movie name or hashcode (e.g. 07100001)")?;
        let frame: usize = args.iter().position(|a| a == "--frame").and_then(|i| args.get(i + 1)).and_then(|v| v.parse().ok()).unwrap_or(0);
        let out = args.iter().position(|a| a == "--shot").and_then(|i| args.get(i + 1)).map(std::path::PathBuf::from).context("--movie is headless only: pass --shot <out.png>")?;
        let iso_path = args.first().cloned().or_else(|| std::env::var("NF_ISO").ok()).context("no <iso> given and NF_ISO is not set")?;
        let iso = nf_formats::iso9660::IsoImage::open(&iso_path)?;
        movie::movie_frame_shot(&iso, name, frame, &out)?;
        return Ok(());
    }
    // Player bug reports, our side (bugreport.rs): `--bug-replay <report.zip> [--shot out.png]`, NF_ISO = the disc.
    if let Some(i) = args.iter().position(|a| a == "--bug-replay") {
        let zip = args.get(i + 1).context("--bug-replay needs <report.zip>")?;
        let iso = std::env::var("NF_ISO").context("--bug-replay needs NF_ISO (the disc image)")?;
        let shot_out = args.iter().position(|a| a == "--shot").and_then(|j| args.get(j + 1)).map(std::path::PathBuf::from);
        for line in bugreport::bug_replay(&iso, std::path::Path::new(zip), shot_out.as_deref())? { println!("{line}"); }
        return Ok(());
    }
    // Positional arguments are the ones before the first flag.
    let pos: Vec<&String> = args.iter().take_while(|a| !a.starts_with("--")).collect();
    // No <level> at all: the front end (fix-menus) — the game's own boot loads the front-end map
    // 0x7000048 and opens the P_START page on it [M bootup 0x1c7e78 + ResetMap_Load 0x1d6360].
    // A lone positional that names an existing file is the image, not a level (`nf-view <iso>` boots the
    // front end too).
    let lone_iso = matches!(pos.as_slice(), [p] if std::path::Path::new(p.as_str()).is_file());
    let (iso, level) = match pos.as_slice() {
        [iso, level] => (iso.to_string(), level.to_string()),
        [iso] if lone_iso => (iso.to_string(), "07000048".to_string()),
        [level] => (std::env::var("NF_ISO").map_err(|_| anyhow::anyhow!("NF_ISO is not set and no <iso> was given\n{USAGE}"))?, level.to_string()),
        [] => (std::env::var("NF_ISO").map_err(|_| anyhow::anyhow!("NF_ISO is not set and no <iso> was given\n{USAGE}"))?, "07000048".to_string()),
        _ => bail!("{USAGE}"),
    };
    let front_end = pos.is_empty() || lone_iso;
    let level = if level.to_ascii_lowercase().ends_with(".bin") { level } else { format!("{level}.bin") };
    let all = args.iter().any(|a| a == "--all");
    let dev = args.iter().any(|a| a == "--dev");
    // `--fps N`: cap the frame rate at N with vsync off (0 = uncapped). Without it: vsync at the display rate.
    let fps_cap: Option<f32> = args.iter().position(|a| a == "--fps").and_then(|i| args.get(i + 1)).and_then(|v| v.parse::<f32>().ok());
    let mut scenario = args.iter().position(|a| a == "--scenario").and_then(|i| args.get(i + 1)).cloned();
    let record_path = args.iter().position(|a| a == "--record").and_then(|i| args.get(i + 1)).cloned();
    let replay_path = args.iter().position(|a| a == "--replay").and_then(|i| args.get(i + 1)).cloned();
    // --replay <file>: the level and scenario come from the recording's header, not the command line.
    let replay_recording = match &replay_path { Some(p) => Some(replay::Replay::load(p)?), None => None };
    let (level, level_for_record): (String, String) = match &replay_recording {
        Some(r) => {
            let lvl = r.header.level.clone();
            if let Some(s) = &r.header.scenario { scenario = Some(s.clone()); }
            eprintln!("[replay] {} ({} ticks, git {}); level {}, scenario {}",
                replay_path.as_deref().unwrap_or("?"), r.tick_count(), r.header.git_hash, lvl, r.header.scenario.as_deref().unwrap_or("(none)"));
            (lvl.clone(), lvl)
        }
        None => (level.clone(), level.clone()),
    };
    // fix-weapon-assets: the `--give` ids must reach the load — each given weapon's assets merge
    // from its owning archive BEFORE the scene builds, so the weapon draws with its skin,
    // sequences, muzzle flash and sounds on any level (`nf_view::load_weapons`).
    let give_ids: Vec<u8> = args.iter().position(|a| a == "--give").and_then(|i| args.get(i + 1))
        .map(|list| list.split(',').filter_map(|t| {
            let t = t.trim();
            let id = t.strip_prefix("0x").and_then(|h| usize::from_str_radix(h, 16).ok()).or_else(|| t.parse().ok());
            id.filter(|&i| i < 0x72).map(|i| i as u8)
        }).collect()).unwrap_or_default();
    let scene = if dev { load_dev(&iso, &level, all)? }
        else if !give_ids.is_empty() { load_weapons(&iso, &level, all, &give_ids)? }
        else if let Some(i) = args.iter().position(|a| a == "--mp-match") {
            // Render aid (mp body checks): `--mp-match H,B[,c0,c1,…]` — the online match load with
            // H humans, B bots and the humans' character words c0.. (the `MP_skins` index), so a
            // headless `--shot` draws the other slots' bodies (`--mp` carries no settings → no bodies).
            let v: Vec<u32> = args.get(i + 1).context("--mp-match needs H,B[,chars]")?.split(',').map(|t| t.trim().parse()).collect::<Result<_, _>>()?;
            let code = u32::from_str_radix(level.trim_end_matches(".bin"), 16).unwrap_or(0x0700_0023);
            let mut st = nf_game::mp::MpSettings::deathmatch(code);
            st.human_count = v.first().copied().unwrap_or(2).clamp(1, nf_game::mp::MP_MAX_SLOTS as u32);
            st.bot_count = v.get(1).copied().unwrap_or(0);
            st.player_count = st.human_count as u16;
            // mp-perf-16: a windowed `--mp-match` run plays (deathmatch()'s raw 10 ends the match
            // in 10 s, straight to the debriefing); NF_NET_TIME=seconds as on the flag host.
            st.time_limit = std::env::var("NF_NET_TIME").ok().and_then(|v| v.parse::<i32>().ok()).unwrap_or(10 * 60);
            for (slot, &c) in v.iter().skip(2).enumerate() { if slot < nf_game::mp::MP_MAX_SLOTS { st.players[slot].skin = c; } }
            load_mp_carry(&iso, &level, all, None, st)?
        }
        else if args.iter().any(|a| a == "--mp") { load_mp(&iso, &level, all)? }
        else { load(&iso, &level, all)? };
    let mut cam = start_camera(&scene);
    if let Some(i) = args.iter().position(|a| a == "--cam") {
        let v: Vec<f32> = args[i + 1..i + 6].iter().map(|s| s.parse()).collect::<Result<_, _>>()?;
        cam = Camera { pos: Vec3::new(v[0], v[1], v[2]), yaw: v[3], pitch: v[4], fov: GAME_FOV };
    }
    let num = |flag: &str| -> Result<Option<f32>> { match args.iter().position(|a| a == flag) { Some(i) => Ok(Some(args.get(i + 1).context("missing value")?.parse()?)), None => Ok(None) } };
    let choice = num("--seq")?.map(|k| k as usize);
    let time = num("--frame")?.unwrap_or(0.0);
    let notex = args.iter().any(|a| a == "--no-tex");
    let nofog = args.iter().any(|a| a == "--no-fog");
    if dev {
        eprintln!("dev mode ON — F1: AI overlay  F2: dev menu  F11: step one tick (when paused)");
        if let Some(ref n) = scenario {
            match dev::scenario(n) {
                Some((label, _)) => eprintln!("scenario: {n} — {label}"),
                None => { eprintln!("unknown scenario: {n}\nscenarios: {}", dev::SCENARIOS.join(", ")); }
            }
        }
    }
    let opts = scene::ViewOpts { reinforcements: args.iter().any(|a| a == "--reinforcements"), firing: args.iter().any(|a| a == "--fire"), right: [1.0, 0.0, 0.0], up: [0.0, 1.0, 0.0] };
    eprintln!("animation: {}", scene.sequence_label(choice));
    // B-6: `--frame N --shot out.png` now applies `--dev --scenario`, so the headless render shows the scenario's
    // placement (Bond near a guard, etc.) instead of the level start. Apply the scenario to a mutable scene before
    // the tick loop. (The event-loop path applies it below.)
    let apply_scenario = |sc: &mut scene::Scene, name: &str| {
        if let Some((_, f)) = dev::scenario(name) { f(sc); }
        else { eprintln!("[dev] unknown scenario '{name}'. Available: {}", dev::SCENARIOS.join(", ")); }
    };
    if let Some(i) = args.iter().position(|a| a == "--shot") {
        // Headless: run the game for `--frame` ticks with Bond at the camera, then render. (B-6: apply the scenario
        // first, so Bond is placed for the feature test and the rendered frame shows it.)
        let mut scene = scene;
        if let Some(n) = &scenario { apply_scenario(&mut scene, n); }
        // The burning woman for the paused-page renders: advance her by the shot's tick count, as the paused frames
        // would (crate::woman), so `--page pause|failed --frame N` shows frame N/2 of her loop.
        if args.iter().position(|a| a == "--page").and_then(|i| args.get(i + 1)).is_some_and(|p| p == "pause" || p == "failed" || p == "results") {
            if let Some(w) = scene.woman.as_mut() {
                for _ in 0..time as u32 { w.tick(); }
                let rgba = w.rgba();
                if let Some((_, t)) = scene.hud_textures.iter_mut().find(|(h, _)| *h == 0x0300_00db) { if t.rgba.len() == rgba.len() { t.rgba = rgba; } }
            }
        }
        // Render aid (mp-body-anim): `--mp-drive "a-b:f=1,s=0,t=0,l=0,c,j,k;…"` drives slot 1's session
        // input over match ticks a..b (f forward, s strafe, t turn, l look, c crouch held, j jump, k kill
        // the slot at tick a) through the online tick (`net_slot0_tick`), for `--frame` ticks; `--mp-shots
        // t1,t2,…` saves an `--mp-look` capture at each listed tick as <shot stem>_tNNNN.png.
        if let Some(di) = args.iter().position(|a| a == "--mp-drive") {
            let spec = args.get(di + 1).context("--mp-drive needs a script")?.clone();
            let out = args.get(i + 1).context("--shot needs a file name")?.clone();
            return mp_drive_shots(&mut scene, &args, &spec, &out, time as u32, opts);
        }
        if let Some(sim) = scene.sim.as_mut() {
            // `--pull 100,130`: the trigger is pressed on those ticks (one tick each).
            let pulls: Vec<u32> = match args.iter().position(|a| a == "--pull") {
                Some(i) => args.get(i + 1).context("--pull needs ticks")?.split(',').map(|t| t.parse()).collect::<Result<_, _>>()?,
                None => Vec::new(),
            };
            // Without `--cam`, Bond stands at the start and the camera is his eye (the first-person gun shows).
            let walk = !args.iter().any(|a| a == "--cam");
            // `--zoom-hold`: hold aim + the zoom key every tick, so the headless --shot renders can show the
            // zoom overlays (the overlay draws only while the key is held; `Player_Zoom` 0x1a1a70 raises the
            // zoom on the held input — the same path the fix-zoom-overlays tests drive).
            let zoom_hold = args.iter().any(|a| a == "--zoom-hold");
            // fix-aims20-lights-render: hold a strafe stick deflection every tick — the analog half of the
            // idle-counter reset (DAT_002a3a1c [M 0x17ee88:80-84]), so the aims lights' gates evaluate on a
            // moving Bond exactly as a strafing render does, instead of the dead-still all-off
            // state 0.75 s in.
            let strafe_hold = args.iter().any(|a| a == "--strafe-hold");
            let shot_input = nf_game::player::Input { aiming: zoom_hold, zoom: zoom_hold, strafe: if strafe_hold { 1.0 } else { 0.0 }, ..Default::default() };
            // `--channels`: every switch channel on first (as K).
            if args.iter().any(|a| a == "--channels") { for c in 1..=255 { sim.drones.set_channel(c) } }
            // `--set-channel N` (dev, headless too): raise one switch channel before the ticks — the window
            // path raises it at frame 120; render checks of channel-driven draws need it here as well.
            if let Some(c) = args.iter().position(|a| a == "--set-channel").and_then(|i| args.get(i + 1)).and_then(|v| v.parse::<u8>().ok()) { sim.drones.set_channel(c); }
            // `--debris-probe x,y,z` (dev render aid): four blast-debris pieces (the MetalHash_132 meshes,
            // `Debris_CreateEx` 0x1ac808's class-0x10 object) thrown up so they are back at that height on
            // tick 60 under the debris gravity (`--frame 60`), so a shot shows their draw.
            if let Some(v) = args.iter().position(|a| a == "--debris-probe").and_then(|i| args.get(i + 1)) {
                let c: Vec<f32> = v.split(',').filter_map(|t| t.parse().ok()).collect();
                if c.len() == 3 {
                    for (k, &h) in nf_game::explode::METAL_HASH_132.iter().enumerate() {
                        sim.drones.blast_debris.push(nf_game::shatter::Debris { model: h, pos: [c[0] + 0.3 * k as f32 - 0.45, c[1], c[2]],
                            rot: [0.3 * k as f32, 0.5, 0.0], vel: [0.0, 0.0803, 0.0], spin: [0.0; 3], life: 100_000, fade: 0xff, e8: 1.0 });
                    }
                }
            }
            // `--hold-npcs` (dev): guards freeze where they stand but stay shootable — aim render checks at them.
            if args.iter().any(|a| a == "--hold-npcs") { sim.drones.hold_npcs = true; }
            // `--bond x,y,z`: Bond starts exactly there (in the air he falls), instead of the level start.
            if let Some(i) = args.iter().position(|a| a == "--bond") {
                let v: Vec<f32> = args.get(i + 1).context("--bond needs x,y,z")?.split(',').map(|t| t.parse()).collect::<Result<_, _>>()?;
                sim.bond = nf_game::player::Bond::new([v[0], v[1], v[2]], v.get(3).copied().unwrap_or(sim.bond.yaw)); if let Some(&pt) = v.get(4) { sim.bond.pitch = pt; }
            }
            // Render aid (cutscene-side-strips): `NF_WIDESCREEN=1` raises the headless sim's view
            // aspect to the AV options' widescreen row's 16/9 — the value the game writes at
            // viewer+0x114 [M Camera_CalcViewAngles 0x1baf70:28-32] and the cut-scene view builds
            // at [M Script_UpdateCamera 0x1db518:9]. The window path feeds the same value from the
            // front end's row (App redraw); the headless sim has no front end, so shots of the
            // widened cut-scene frame set it here.
            if std::env::var_os("NF_WIDESCREEN").is_some() { sim.view_aspect = 16.0 / 9.0; }
            // `--draw <id>`: equip that weapon (the `--give` path's equip) and switch to it before the ticks, so a
            // headless render can show a fire script frame by frame (`--draw 0x34 --pull 60 --frame 70`).
            let mut draw: Option<u8> = None;
            if let Some(i) = args.iter().position(|a| a == "--draw") {
                let t = args.get(i + 1).context("--draw needs a weapon id")?;
                let id = match t.strip_prefix("0x") { Some(h) => usize::from_str_radix(h, 16)?, None => t.parse()? };
                let wd = sim.weapons.clone();
                let rounds = wd.clip_size(id).max(1);
                if sim.drones.inventory.equip_weapon(&wd, id, rounds, true) { sim.drones.inventory.equip_ammo(&wd, id, rounds * 3); }
                draw = Some(id as u8);
            }
            // `--skip-cutscenes`: hold the game's own skip while a script camera runs (`Script_FFwd` 0x1dac78), for image pairs.
            let skip_cuts = args.iter().any(|a| a == "--skip-cutscenes");
            let modes: Vec<u32> = match args.iter().position(|a| a == "--mode") {
                Some(i) => args.get(i + 1).context("--mode needs ticks")?.split(',').map(|t| t.parse()).collect::<Result<_, _>>()?,
                None => Vec::new(),
            };
            if let Some(w) = draw { sim.step_bond(nf_game::player::Input { switch_to: Some(w), ..shot_input }, false, false, false); }
            // fix-aims20-lights-render: the headless `--shot` path honours `--give` too — the same equip +
            // switch_request the window path applies below (the shot path returned before that block, so
            // `nf-view <iso> <level> --dev --give 0x1a --shot …` rendered the level's starting gun, so the
            // AIMS-20's view model never reached a headless render). Runs BEFORE the
            // ticks, so the draw anim completes and the aims lights' gates evaluate on the held 0x1a/0x1b.
            if let Some(list) = args.iter().position(|a| a == "--give").and_then(|i| args.get(i + 1)) {
                let wd = sim.weapons.clone();
                let mut first = None;
                for t in list.split(',') {
                    let t = t.trim();
                    let id = if let Some(h) = t.strip_prefix("0x") { usize::from_str_radix(h, 16).ok() } else { t.parse().ok() };
                    let Some(id) = id.filter(|&i| i < 0x72) else { eprintln!("--give: bad id {t:?}"); continue };
                    let rounds = wd.clip_size(id).max(1);
                    if sim.drones.inventory.equip_weapon(&wd, id, rounds, true) {
                        sim.drones.inventory.equip_ammo(&wd, id, (rounds * 3).max(rounds));
                        first.get_or_insert(id as u8);
                        eprintln!("--give: equipped {id:#04x}");
                    } else if sim.drones.inventory.owned.get(id).copied().unwrap_or(false) {
                        first.get_or_insert(id as u8);
                        eprintln!("--give: {id:#04x} already owned — drawing it");
                    } else {
                        eprintln!("--give: {id:#04x} refused (no first-person model)");
                    }
                }
                if let Some(id) = first { sim.gun.switch_request = id; }
            }
            sim.rapid_fire = RAPID_FIRE.load(std::sync::atomic::Ordering::Relaxed);
            // Render aid (death-blood-pane): `NF_KILL_AT=t` deals Bond a fatal hit on tick t, so `--frame`
            // past it renders the blood pane at that many dead ticks.
            let kill_at: Option<u32> = std::env::var("NF_KILL_AT").ok().and_then(|v| v.parse().ok());
            // Render aids (gunimp-mounted-pose): `NF_USE_AT=t` presses E on tick t; `NF_FWD=a-b` holds
            // forward and `NF_TURN=a-b:v` holds the turn stick at v over those ticks.
            let range = |k: &str| -> Option<(u32, u32, f32)> {
                let v = std::env::var(k).ok()?;
                let (r, val) = v.split_once(':').map_or((v.as_str(), 1.0), |(r, x)| (r, x.parse().unwrap_or(1.0)));
                let (a, b) = r.split_once('-')?;
                Some((a.parse().ok()?, b.parse().ok()?, val))
            };
            let use_at: Option<u32> = std::env::var("NF_USE_AT").ok().and_then(|v| v.parse().ok());
            // Render aid (break-debris draw): `NF_LIST_BREAKS=1` prints every breakable's index, kind
            // and box centre (pick a spot to shoot one from).
            if std::env::var_os("NF_LIST_BREAKS").is_some() {
                for (i, b) in sim.drones.breaks.list.iter().enumerate() {
                    let c = [(b.lo[0] + b.hi[0]) / 2.0, (b.lo[1] + b.hi[1]) / 2.0, (b.lo[2] + b.hi[2]) / 2.0];
                    eprintln!("[breaks] #{i} kind {} records {} glass {} centre {:.2},{:.2},{:.2}", b.kind, b.src.records.len(), b.src.glass.len(), c[0], c[1], c[2]);
                }
            }
            let (fwd, turn) = (range("NF_FWD"), range("NF_TURN"));
            for t in 1..=time as u32 {
                if kill_at == Some(t) { let (h, r) = (&mut sim.drones.health, &mut sim.drones.rng); h.pain_generic(r, 500.0, 1, 0x0700_0005, 0); }
                // Render aid (copter-script-draw): `NF_CH_AT=t:ch` sets switch channel `ch` on tick t
                // (the headless counterpart of the dev `--set-channel`).
                if let Some((ct, cc)) = std::env::var("NF_CH_AT").ok().and_then(|v| v.split_once(':').and_then(|(a, b)| Some((a.parse::<u32>().ok()?, b.parse::<u32>().ok()?)))) {
                    if ct == t { let f = sim.drones.frame; sim.drones.channels.set(cc, 1, f); }
                }
                // Render aid (keypad LOD swap): `NF_UNLOCK_AT=t:out` unlocks the locks with that out
                // channel on tick t (the state `Lock_Unlock` 0x1b0650 leaves), to render the swapped model.
                if let Some((ut, uo)) = std::env::var("NF_UNLOCK_AT").ok().and_then(|v| v.split_once(':').and_then(|(a, b)| Some((a.parse::<u32>().ok()?, b.parse::<u16>().ok()?)))) {
                    if ut == t { for l in sim.drones.locks.iter_mut().filter(|l| l.out == uo) { l.unlocked = true; } }
                }
                // Render aid (destroy hit flash): `NF_DESTROY_COLOUR_AT=t:placement:byte` sets that Destroy
                // leaf's colour pair on tick t (the state the laser's drain leaves), to render the flash;
                // `t:placement:dead` marks the leaf dead (the destroyed-glist swap's draw side).
                if let Some((dt, rest)) = std::env::var("NF_DESTROY_COLOUR_AT").ok().and_then(|v| v.split_once(':').map(|(a, b)| (a.to_string(), b.to_string()))) {
                    if let (Ok(dt), Some((pl, by))) = (dt.parse::<u32>(), rest.split_once(':')) {
                        if dt == t {
                            if let Ok(pl) = pl.parse::<usize>() {
                                for d in sim.drones.props.destroy.iter_mut().filter(|d| d.placement == pl) {
                                    if by == "dead" { d.dead = true; } else if let Ok(by) = by.parse::<u8>() { d.colour = [by, by]; }
                                }
                            }
                        }
                    }
                }
                // Render aid (health-pane-fade): `NF_PANE_FADE=1` turns the pane-fade option on
                // (`decay_gate`, PlayerSetting+0xB) from tick 1, so BL+0x8BC decays and the pane fades.
                if t == 1 && std::env::var_os("NF_PANE_FADE").is_some() { sim.drones.decay_gate = true; }
                // Render aid (bond-moment-flash): `NF_MOMENT_AT=t` marks Bond moment 0x65 done on tick t
                // (the BondMoments entry PlrStat_DoneBondMoment sets), so the HUD's 007 flash runs.
                if std::env::var("NF_MOMENT_AT").ok().and_then(|v| v.parse::<u32>().ok()) == Some(t) {
                    sim.drones.progress.bond_moments[0] = 1;
                }
                // `NF_BREAK_AT=t:i`: breakable #i is killed on tick t through `Breaks::kill` (the shot's
                // kill path) with a ray from Bond's head to the box centre.
                if let Some((bt, bi)) = std::env::var("NF_BREAK_AT").ok().and_then(|v| v.split_once(':').and_then(|(a, b)| Some((a.parse::<u32>().ok()?, b.parse::<usize>().ok()?)))) {
                    if bt == t && bi < sim.drones.breaks.list.len() {
                        let b = &sim.drones.breaks.list[bi];
                        let c = [(b.lo[0] + b.hi[0]) / 2.0, (b.lo[1] + b.hi[1]) / 2.0, (b.lo[2] + b.hi[2]) / 2.0];
                        let head = sim.bond.head();
                        let k = sim.drones.breaks.kill(bi, head, c, &mut sim.drones.rng);
                        eprintln!("[breaks] #{bi} killed: {:?} — {} debris, {} shards", k.map(|k| k.kind), sim.drones.breaks.shatter.debris.len(), sim.drones.breaks.shatter.shards.len());
                    }
                }
                if skip_cuts && sim.disabled { sim.drones.scripts.skip_request = true; }
                // `--mode 70,90`: the fire-mode press (the B key) on those ticks.
                let mode_now = modes.contains(&t);
                let mut inp = nf_game::player::Input { mode_switch: mode_now, ..shot_input };
                if let Some((a, b, v)) = fwd { if (a..=b).contains(&t) { inp.forward = v; } }
                if let Some((a, b, v)) = turn { if (a..=b).contains(&t) { inp.turn = v; } }
                if walk { sim.step_bond(inp, pulls.contains(&t), false, use_at == Some(t)); } else { sim.step([cam.pos.x, cam.pos.y, cam.pos.z], cam.dir().to_array(), pulls.contains(&t), false); }
            }
            // With `--skip-cutscenes`, a skipped cut-scene's hand-back moves Bond: put him back on the `--bond` spot (and
            // let one tick settle) so the image pair matches the real game's placement.
            if skip_cuts {
                if let Some(i) = args.iter().position(|a| a == "--bond") {
                    let v: Vec<f32> = args.get(i + 1).map(|s| s.split(',').filter_map(|t| t.parse().ok()).collect()).unwrap_or_default();
                    if v.len() >= 3 { sim.bond = nf_game::player::Bond::new([v[0], v[1], v[2]], v.get(3).copied().unwrap_or(sim.bond.yaw)); if let Some(&pt) = v.get(4) { sim.bond.pitch = pt; } sim.step_bond(shot_input, false, false, false); }
                }
            }
            if walk { let h = sim.bond.head(); cam = Camera { pos: Vec3::new(h[0], h[1], h[2]), yaw: sim.bond.yaw, pitch: sim.bond.pitch * std::f32::consts::FRAC_PI_2, fov: GAME_FOV / sim.zoom }; }
            if walk { if let Some((p, yaw, pitch, fov)) = sim.script_camera() { cam = Camera { pos: Vec3::new(p[0], p[1], p[2]), yaw, pitch, fov }; } }
            eprintln!("Bond at {:?}{}, health {}", sim.bond.pos, if sim.disabled { " (disabled by a script)" } else { "" }, sim.drones.health.health);
            for u in sim.unposed(&scene.characters) { eprintln!("unposed: {u}"); }
            for p in &sim.drones.pickups { eprintln!("pickup: weapon {:#x}, {} rounds at {:?}", p.weapon, p.rounds, &p.m[3][..3]); }
            if let Some(dr) = sim.drones.list.iter().find(|d| !d.shots.is_empty() || d.state != 4) {
                eprintln!("AI: a drone in state {:#x}, {} shots, clip {}", dr.state, dr.shots.len(), dr.clip);
            }
        }
        // fix-armour-ring: `--armour <n>` writes BLData+0x8b0 after the ticks (the pane's armour — the
        // same write the dev panel's armour row and cheats.py's god make every frame), so the health
        // pane's armour state renders headless.
        if let Some(i) = args.iter().position(|a| a == "--armour") {
            let n: f32 = args.get(i + 1).context("--armour needs a value (0..=50)")?.parse()?;
            if let Some(sim) = scene.sim.as_mut() { sim.drones.health.armour = n; }
        }
        let (mut menu_frame, mut movie_frame) = front_end_menu(&args, &scene, front_end, &iso);
        // `--loading`: the level's loading screen (crate::loading) — the picture on the movie plane, the text and the
        // dots (`--frame` vblanks into the animation) over it.
        if args.iter().any(|a| a == "--loading") {
            let level = u32::from_str_radix(level.trim_end_matches(".bin"), 16).unwrap_or(0);
            if let Some(art) = read_files_bin(&iso, loading::ARCHIVE).map(|b| loading::art(&b)) {
                movie_frame = art.get(&loading::image_for(level)).cloned();
            }
            let lang = scene.sim.as_ref().and_then(|s| s.lang.clone());
            let mut seed = 0x1234_5678u32;
            let hint = loading::hint_for(level, || { seed = seed.wrapping_mul(1_103_515_245).wrapping_add(12345); seed >> 16 });
            if let Some(t) = scene.hud.as_ref() {
                menu_frame = Some(menu::loading_overlay(&t.font, lang.as_ref(), &scene.hud_textures, 1280.0 / 720.0, level, hint, loading::dots_lit(time as u32)));
            }
        }
        // fix-ingame-dev-menu: `--dev-page <name>` draws the F2 menu's page (menu/weapons/gadgets/
        // grenades/ammo/npcs/teleport/level/toggles/screenshot) over the game HUD, so each page
        // can be rendered and checked headless.
        if let Some(pg) = args.iter().position(|a| a == "--dev-page") {
            let name = args.get(pg + 1).context("--dev-page needs a page name")?;
            let page = dev::page_index(name).with_context(|| format!("--dev-page: unknown page {name:?} (one of: {})", dev::PAGES.join(", ")))?;
            let d = dev::Dev { menu: true, page, ..Default::default() };
            let f = dev::hud(&d, &scene, 1280.0 / 720.0, [[0.0; 4]; 4]);
            menu_frame.get_or_insert_with(|| overlay_for(&scene, 1280.0 / 720.0)).append(&f);
        }
        // Render aid (mp body checks): `--mp-look slot[,dist[,height[,angle]]]` — after the ticks, the
        // camera stands `dist` (2.5) from that slot's body, `angle` (0 = in front) around it, `height`
        // (1.2) above its feet, looking at its chest; the local view model is hidden.
        if let Some(li) = args.iter().position(|a| a == "--mp-look") {
            let v: Vec<f32> = args.get(li + 1).context("--mp-look needs slot[,dist,height,angle]")?.split(',').map(|t| t.trim().parse()).collect::<Result<_, _>>()?;
            let slot = v.first().copied().unwrap_or(1.0) as usize;
            let (dist, height, angle) = (v.get(1).copied().unwrap_or(2.5), v.get(2).copied().unwrap_or(1.2), v.get(3).copied().unwrap_or(0.0));
            let mut scene = scene;
            // mp-remote-fx: a dead slot's death body counts too.
            let st = scene.sim.as_ref().and_then(|s| s.mp_body_state(slot).or_else(|| s.mp_body_death(slot).map(|(p, y, _)| (p, y, 0.0)))).context("--mp-look: that slot has no drawn body")?;
            let (p, yaw) = (st.0, st.1 + angle);
            // The body's `pos` is Bond::pos (floor + 1.1) for humans, the bot's eye − BOT_EYE for bots.
            let feet_y = p[1] - 1.1;
            let eye = [p[0] + yaw.sin() * dist, feet_y + height, p[2] + yaw.cos() * dist];
            let target = [p[0], feet_y + 1.0, p[2]];
            let d = [target[0] - eye[0], target[1] - eye[1], target[2] - eye[2]];
            let cam = Camera { pos: Vec3::new(eye[0], eye[1], eye[2]), yaw: d[0].atan2(d[2]), pitch: (d[1] / (d[0] * d[0] + d[2] * d[2]).sqrt()).atan(), fov: GAME_FOV };
            if let Some(s) = scene.sim.as_mut() { s.first_person = false; }
            eprintln!("--mp-look: slot {slot} body at {p:?} yaw {:.2} held {}", st.1, scene.sim.as_ref().map_or(0, |s| s.mp_held[slot]));
            return shot(&scene, &cam, args.get(i + 1).context("--shot needs a file name")?, choice, time, opts, notex, nofog, menu_frame.as_ref(), movie_frame.as_ref());
        }
        // Render aid (NF_SHOT_AT_PICKUP=k): frame pickup k from its DRAWN matrix (`Pickup::m`, what the renderer
        // places), 1.8 units out on x and z and 1.2 up — no coordinate guessing.
        if let Some(k) = std::env::var("NF_SHOT_AT_PICKUP").ok().and_then(|v| v.parse::<usize>().ok()) {
            let p = scene.sim.as_ref().and_then(|s| s.drones.pickups.get(k)).map(|p| [p.m[3][0], p.m[3][1], p.m[3][2]]).context("NF_SHOT_AT_PICKUP: no such pickup")?;
            let eye = [p[0] + 1.8, p[1] + 1.2, p[2] + 1.8];
            let d = [p[0] - eye[0], p[1] - eye[1], p[2] - eye[2]];
            let cam = Camera { pos: Vec3::new(eye[0], eye[1], eye[2]), yaw: d[0].atan2(d[2]), pitch: (d[1] / (d[0] * d[0] + d[2] * d[2]).sqrt()).atan(), fov: GAME_FOV };
            if let Some(s) = scene.sim.as_ref() { let q = &s.drones.pickups[k]; eprintln!("NF_SHOT_AT_PICKUP {k}: drawn at {p:?}, weapon {:#x}, hidden {} gone {}", q.weapon, q.hidden, q.gone); }
            return shot(&scene, &cam, args.get(i + 1).context("--shot needs a file name")?, choice, time, opts, notex, nofog, menu_frame.as_ref(), movie_frame.as_ref());
        }
        return shot(&scene, &cam, args.get(i + 1).context("--shot needs a file name")?, choice, time, opts, notex, nofog, menu_frame.as_ref(), movie_frame.as_ref());
    }
    let event_loop = EventLoop::new()?;
    // Sound: built in `App::bare` (the level's bank in the driver, the default output device; silent without one).
    let mut scene = scene;
    // `--dev --scenario <name>`: place Bond and set up the world for a specific feature test. (B-3a/b: the two
    // scenario systems are merged into `dev::scenario` / `dev::SCENARIOS` in `dev.rs`; `apply_scenario` is gone.)
    #[cfg(feature = "gamepad")]
    let pad_backend = gamepad::Backend::new(&iso);
    // `--hold-npcs` (dev): the window run holds every guard in place, still shootable.
    if dev && args.iter().any(|a| a == "--hold-npcs") { if let Some(sim) = scene.sim.as_mut() { sim.drones.hold_npcs = true; } }
    let force_channel = if dev { args.iter().position(|a| a == "--set-channel").and_then(|i| args.get(i + 1)).and_then(|v| v.parse::<u8>().ok()) } else { None };
    menu::SAVE_NAMES_FROM_DISK.store(true, std::sync::atomic::Ordering::Relaxed);
    // headless-app-harness: the constructor moved into `App::bare` (the headless harness
    // builds the same skeleton, gfx: None / sound silent); the CLI-only knobs apply on top.
    let mut app = App::bare(iso, scene, cam, opts, choice, time);
    app.fps_cap = fps_cap;
    app.notex = notex;
    app.nofog = nofog;
    app.all = all;
    app.dev = dev.then(|| Box::new(dev::Dev { infinite_ammo: true, ..Default::default() }));
    app.force_channel = force_channel;
    #[cfg(feature = "gamepad")]
    {
        app.pad = pad_backend;
    }
    if dev {
        // fix-ingame-dev-menu: the browser panel is no longer part of `--dev` — the in-window F2
        // menu carries ALL of its functions now (through the same `panel::apply` command layer).
        // `--panel` puts the HTTP page back for anyone who wants a second screen.
        app.panel = if args.iter().any(|a| a == "--panel") { panel::start() } else { None };
        match &app.panel {
            Some(_) => eprintln!("dev panel: open http://127.0.0.1:{} in a browser (beside the game window)", panel::PORT),
            None if args.iter().any(|a| a == "--panel") => eprintln!("dev panel: port {} busy (another nf-view?) — panel not started", panel::PORT),
            None => {}
        }
        if args.iter().any(|a| a == "--arena") {
            let (status, _) = panel::apply(&mut app.scene, &[("do".to_string(), "arena".to_string())]);
            eprintln!("[arena] {status}");
        }
        if let (Some(d), Some(n)) = (app.dev.as_deref_mut(), scenario.as_ref()) {
            if let Some((_, f)) = dev::scenario(n) {
                f(&mut app.scene);
                d.scenario = Some(leak_str(n));
            } else {
                eprintln!("[dev] unknown scenario '{n}'. Available: {}", dev::SCENARIOS.join(", "));
            }
        }
    }
    // fix-menus: the front end — the level code, the game's strings, and the boot page.
    app.level_code = u32::from_str_radix(level.trim_end_matches(".bin"), 16).unwrap_or(0);
    // Every mission open in the window unless `--locked` (a port choice) — set
    // before any front end exists, so the boot, pause, quit and debriefing managers all see it.
    menu::UNLOCK_ALL.store(!std::env::args().any(|a| a == "--locked"), std::sync::atomic::Ordering::Relaxed);
    if front_end {
        app.fe = Some(menu::FrontEnd::new());
        // boot-flow: the window boot runs P_INTRO's EA + MGM logos before the title [M 0x209e88];
        // the headless --shot/--page paths build their own FrontEnd and stay on the page they ask for.
        // `--no-intro`: port dev aid (the game's intro is unskippable
        // [M 0x209e88]) — open straight on the title.
        if !std::env::args().any(|a| a == "--no-intro") {
            if let Some(fe) = app.fe.as_mut() { fe.start_boot_intro(); }
        }
        // frontend-music: the boot front end's own `Sound_Ready` case 0x7000048 consumes the
        // flip [M 0x1dceb8:94-101 — DAT_002e1b11 (.bss) = 0 → bank 0xc resident, DAT → 1;
        // P_START's once arm pins it right after, 0x20a408:31]. The port consumes it here so
        // the first mission-return load flips to 0x38 like the game's, and re-points the
        // driver's primary at the boot bank's slot.
        if let Some(fe) = app.fe.as_mut() {
            let hash = fe.music.front_end_sound_ready();
            if let Some(s) = app.sound.as_mut() { s.driver.set_primary(if hash == 0x38 { 1 } else { 0 }); }
        }
        // settings-persist: the front end boots with the option file's values (missing or
        // corrupt → the defaults, the warning on stderr). Window settings aren't persisted:
        // the port has no size/fullscreen state yet (the window is winit's default, lib.rs
        // resumed).
        let (opts, warn) = settings::load_default();
        if let Some(w) = warn { eprintln!("[settings] {w}"); }
        app.fe.as_mut().unwrap().opts = opts;
        // options-persist-roundtrip: the AV volume rows reach the sound layer at boot [M
        // LS_LoadGlobalSettings 0x1cdbc8:69-70 — the loaded settings call
        // SFXMusicSetVolume/SFXSetVolume].
        let fe = app.fe.as_mut().unwrap();
        apply_av_volumes(fe, app.sound.as_mut().map(|s| &mut s.driver));
        // The front end needs the game's strings even when the front-end map carries no Sim
        // (`Txt_LoadLanguage` 0x1caad0 loads `FNames[SelLang]` = USATxt.dat [M lib.rs load_impl]).
        match read_files_bin(&app.iso, "USATxt.dat") {
            Some(bytes) => match nf_formats::hud::Lang::parse(&bytes) {
                Ok(lang) => app.lang = Some(lang),
                Err(e) => eprintln!("USATxt.dat: {e} — menu labels show their label ids"),
            },
            None => eprintln!("no USATxt.dat in FILES.BIN — menu labels show their label ids"),
        }
        // create-save (profile-session): the port's memory card IS the config dir (the
        // settings' own directory, %APPDATA%\NightfirePort — docs/systems/profile-session.md).
        // The boot enumerates the saved codenames the way the game's card enums run [M
        // Menu_EnumSaves 0x1f7b78] — without it the name page's ✓ refused silently
        // (`request_create` without a dir) and Select Codename listed nothing, so no save
        // could be created. The validator's reserved names come
        // from the two def_codename labels through the language bank [M Menu_IsValidCodename
        // 0x1f9930 strstr's the uppercased name against them — 0x29b "New Codename",
        // 0x211 "Default"; both [D USATxt.dat]].
        app.profile_session.dir = settings::config_dir();
        if let Some(fe) = app.fe.as_mut() {
            let session = app.profile_session.clone();
            session.apply_to(fe);
            if let Some(lang) = app.lang.as_ref() {
                fe.cn_reserved = vec![
                    lang.bind(0x29b).unwrap_or("New Codename").to_string(),
                    lang.bind(0x211).unwrap_or("Default").to_string(),
                ];
            }
        }
        eprintln!("[front end] title screen (P_START 0x40000009); Enter/click = start, arrows = move, Esc = back");
    } else {
        // options-persist-roundtrip: a mission started straight from the command line boots
        // the persisted volumes too — the game's boot applies them before any menu exists
        // [M Boot_LoadPTPData 0x1c8560:81-82 SFXSetVolume/SFXMusicSetVolume(DAT_0031c17c/78)].
        // The headless --shot path returned above and never touches the player's config.
        let (opts, warn) = settings::load_default();
        if let Some(w) = warn { eprintln!("[settings] {w}"); }
        let music = opts.get(&0x1000_0135).copied().unwrap_or(70);
        let sfx = opts.get(&0x1000_0134).copied().unwrap_or(100);
        if let Some(s) = app.sound.as_mut() { s.driver.music_master = music; s.driver.master = sfx; }
    }
    // `--bond x,y,z` in the window too (it only applied to the headless `--frame` path): Bond starts exactly there.
    if let Some(i) = args.iter().position(|a| a == "--bond") {
        if let (Some(v), Some(sim)) = (args.get(i + 1), app.scene.sim.as_mut()) {
            let v: Vec<f32> = v.split(',').filter_map(|t| t.parse().ok()).collect();
            if v.len() >= 3 {
                sim.bond = nf_game::player::Bond::new([v[0], v[1], v[2]], v.get(3).copied().unwrap_or(sim.bond.yaw));
                if let Some(&pt) = v.get(4) { sim.bond.pitch = pt; }
                app.dev_bond = Some([v[0], v[1], v[2], sim.bond.yaw, sim.bond.pitch]);
            }
        }
    }
    // net-match-host: `--net-host [port] [arena]` listens for clients and starts the match (the
    // demo settings: an Arena deathmatch with a fresh per-match seed) when the first one joins;
    // `--net-join host:port` dials it. These flags are the runnable path
    // without the lobby pages. The seed comes from the wall clock OUTSIDE the Sim: it is
    // matchmaking data (sent to every machine in the START packet), never sim state.
    if let Some(i) = args.iter().position(|a| a == "--net-host") {
        let port = args.get(i + 1).and_then(|v| v.parse::<u16>().ok()).unwrap_or(27420);
        let arena = args.get(i + 2).and_then(|v| u32::from_str_radix(v.trim_start_matches("0x"), 16).ok()).unwrap_or(0x0700_0023);
        let mut st = nf_game::mp::MpSettings::deathmatch(arena);
        // The menu's default duration, 10 minutes, in the SECONDS `MPSettings+0x1a0` holds (the
        // menu writes minutes × 0x3c [M P_MPCONFIRM]); `deathmatch()`'s raw 10 ended the demo match
        // after 10 seconds — straight to the debriefing (the two-window check, 2026-10-18).
        st.time_limit = 10 * 60;
        // Test aid (NF_NET_TIME=seconds): a shorter flag-path match (the debriefing checks).
        if let Some(s) = std::env::var("NF_NET_TIME").ok().and_then(|v| v.parse::<i32>().ok()) { st.time_limit = s; }
        // Test aid (NF_NET_FRAGS=n, −1 = none): the frag limit (the long-match soak: 14 armed bots hit the default 20 in
        // under 3 minutes).
        if let Some(f) = std::env::var("NF_NET_FRAGS").ok().and_then(|v| v.parse::<i32>().ok()) { st.frag_limit = f; }
        // Test aid (NF_NET_BOTS=n): n bots in the flag host's match (the online bots check).
        if let Some(n) = std::env::var("NF_NET_BOTS").ok().and_then(|v| v.parse::<u32>().ok()) {
            st.bot_count = n.min(nf_game::mp::MP_MAX_SLOTS as u32 - 1);
        }
        // Test aids (mp-bot-objectives): NF_NET_SCENARIO=<hex scenario word> plays that scenario
        // on the flag path (the team-game bit derived as `MP_Init` does [M 0x183438:41]) and
        // NF_NET_TEAMS="t0,t1,…" sets the slots' team words in slot order (humans, then bots).
        if let Some(sc) = std::env::var("NF_NET_SCENARIO").ok().and_then(|v| u32::from_str_radix(v.trim_start_matches("0x"), 16).ok()) {
            st.scenario = sc;
            st.gametype_mask = sc;
            st.team_game = (sc >> 29) & 1;
        }
        if let Ok(list) = std::env::var("NF_NET_TEAMS") {
            for (slot, t) in list.split(',').enumerate().take(nf_game::mp::MP_MAX_SLOTS) {
                if let Ok(t) = t.trim().parse::<u32>() { st.players[slot].team = t; }
            }
        }
        let nanos = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.subsec_nanos()).unwrap_or(1);
        st.seed = nanos | 1;
        app.net_armed = Some(st);
        if let Err(e) = app.net_host(port) { eprintln!("[net] host failed: {e}"); }
        else { eprintln!("[net] hosting on udp/{port}, arena {arena:08x} — waiting for a client"); }
    }
    if let Some(i) = args.iter().position(|a| a == "--net-join") {
        let addr = args.get(i + 1).cloned().unwrap_or_else(|| "127.0.0.1:27420".to_string());
        if let Err(e) = app.net_join(&addr) { eprintln!("[net] join failed: {e}"); }
        else { eprintln!("[net] dialing {addr}"); }
    }
    // `--give id[,id...]` (hex like 0x50 or decimal): equip those weapons/gadgets at the start, the way the dev panel's
    // weapon button does (`Player_EquipWeapon` 0x1a8a58 + three clips of ammo), and draw the first one. For
    // checks that need a gadget the level hasn't handed out yet.
    if let Some(i) = args.iter().position(|a| a == "--give") {
        if let (Some(list), Some(sim)) = (args.get(i + 1), app.scene.sim.as_mut()) {
            let wd = sim.weapons.clone();
            let mut first = None;
            for t in list.split(',') {
                let t = t.trim();
                let id = if let Some(h) = t.strip_prefix("0x") { usize::from_str_radix(h, 16).ok() } else { t.parse().ok() };
                let Some(id) = id.filter(|&i| i < 0x72) else { eprintln!("--give: bad id {t:?}"); continue };
                let rounds = wd.clip_size(id).max(1);
                if sim.drones.inventory.equip_weapon(&wd, id, rounds, true) {
                    sim.drones.inventory.equip_ammo(&wd, id, (rounds * 3).max(rounds));
                    // The panel give's twin fill (fix-aims20-modes-lights, panel.rs): a twin-linked record
                    // (+0x05, the AIMS-20 0x1a ↔ its grenade 0x1b) needs rounds in the twin's own slot or
                    // the mode press is refused (`Player_WeaponHasAmmo` 0x1a16c0) — a bare
                    // `--give 0x1a` left B dead and the grenade light dark.
                    let twin = wd.records.get(id).map(|r| r[5] as i8).unwrap_or(0);
                    if twin != 0 {
                        let t = (id as i32 + twin as i32) as usize;
                        if let Some(c) = sim.drones.inventory.clip.get_mut(t) {
                            if *c == 0 { *c = wd.clip_size(t).max(1); }
                        }
                    }
                    first.get_or_insert(id as u8);
                    eprintln!("--give: equipped {id:#04x}");
                } else if sim.drones.inventory.owned.get(id).copied().unwrap_or(false) {
                    // Already owned and the game's own equip refuses it (the ammo-type cap — the Q-Claw
                    // 0x50 on 07000005 is owned at the start [L Inventory::level_start] with ammo type 0's
                    // cap 0 [L +0x90/+0x92], so `Player_EquipWeapon` 0x1a8a58's owned branch declines it
                    // [M]). The give's contract is to DRAW the thing: the switch request is the game's own
                    // `Player_WeaponSelect` path for an owned weapon. Without this
                    // `--give 0x50` printed "refused" and Bond stayed on the P2K — firing gave the real
                    // bullet impact instead of the grapple.
                    first.get_or_insert(id as u8);
                    eprintln!("--give: {id:#04x} already owned — drawing it");
                } else {
                    eprintln!("--give: {id:#04x} refused (no first-person model)");
                }
            }
            if let Some(id) = first { sim.gun.switch_request = id; }
        }
    }
    // fix-restart-health: the level opened from the command line is a load too — the game re-saves
    // the RamSave buffer with the post-load state on EVERY load [M `Player_InitWeapon` 0x1a7ef8:325],
    // so a fail-Restart serves health 100. Only the in-play level change set `entry_save`; the
    // first level kept None and its Restart served the zero buffer's 50 floor (seen on
    // 07000005). Taken after `--give` so the restart keeps the given gear.
    app.entry_save = app.scene.sim.as_ref().map(entry_checkpoint_save);
    // B-5: start the recorder AFTER the scenario is applied (so the header captures the post-scenario starting state:
    // Bond near a guard, the RNG seed, etc.). On replay, the same starting state is reproduced.
    if let Some(path) = record_path {
        app.recorder = Some(replay::Recorder::start(&app.scene, &path, &level_for_record, scenario.as_deref(), dev)?);
        eprintln!("[replay] recording to {path} (Esc-Esc to stop & write)");
    }
    // B-5: hand the parsed recording to the App; the tick loop reads from it instead of the keyboard.
    if let Some(r) = replay_recording { app.replay = Some(r); app.replay_idx = 0; app.grab(true); }
    event_loop.run_app(&mut app)?;
    // B-5: flush the recording on a clean exit (Esc-Esc). A panic does NOT flush (the file would be partial).
    if let Some(r) = app.recorder.take() { let _ = r.finish(); }
    Ok(())
}

/// Leak a `String` to a `&'static str` (the scenario name lives for the program's life).
fn leak_str(s: &str) -> &'static str { Box::leak(s.to_string().into_boxed_str()) }

/// The pause OBJECTIVES tab's rows: the mission's visible objectives (label / hint / done) [M C_GCPAUSE_Handler
/// 0x204408:251-268 lists `Mission_NumVisObjectives` rows; 0x182540 counts the objectives whose state (record+0x14)
/// is > 1 or < 0; a row is done at state 4].
fn pause_objective_rows(sim: &scene::Sim) -> Vec<menu::PauseObjective> {
    // objective-hints [M C_GCPAUSE_Handler 0x204408:251-271]: the rows come from
    // Mission_NumVisObjectives 0x182540 + Mission_ObjectiveState 0x182630 — the WHOLE mission's
    // announced objectives in LEVELS order (prior parts included, future parts and unannounced
    // rows excluded), not just the current level's record. state 4 = the ~J done marker; the
    // hint stays 0xffffffff for hint-less records (the view's ✕ applies the 0x31d fallback).
    let m = &sim.drones.mission;
    (0..m.num_vis_objectives()).filter_map(|i| m.objective_state(i))
        .map(|o| menu::PauseObjective { text: o.text, hint: o.hint, done: o.state == 4 })
        .collect()
}

/// bonus-007: the pause SCORE tab's live counters, straight off the progress layer — the game's
/// handler fills its list box from `PlrMissionStats` [M C_GCPAUSE_Handler 0x204408:447-498; the
/// record layout is docs/fields/player-stats.md]. Shots/accuracy/time keep their existing fill.
fn pause_stats(sim: &scene::Sim) -> menu::Stats {
    let s = sim.drones.progress.stats();
    menu::Stats {
        bond_moves: s.bond_moments,
        dispatched: s.dispatched,
        subdued: s.disabled,
        surrendered: s.surrendered,
        bonus_pickups: s.bonus_pickups,
        ..menu::Stats::default()
    }
}

/// The mission's tracked tallies as the results page's stats (mission-results-page). The game's
/// results screen and its Stats sub-page read the SAME counters through `PlrStat_GetScore`'s row
/// refill [M 0x1984b0:66-84 — row0 +0x20 bond moments, row1 +0x0C dispatched, row2 +0x10
/// disabled, row3 +0x14 surrendered, row5 +0x04/+0x08 shots, row7 +0x30 the 1/100-s elapsed,
/// row8 +0x24 bonus pickups]. The accuracy percentage is the row-5 ratio the game rounds
/// [M 0x1984b0's shared ratio block — round(min/count × 100)], full marks with no shots
/// [M :74-78 — row+0x8 = row+0x0 when fired == 0].
pub fn mission_end_stats(p: &nf_game::progress::Progress) -> menu::Stats {
    let s = p.stats();
    menu::Stats {
        shots_fired: s.shots_fired,
        shots_hit: s.shots_hit_enemy,
        accuracy_pct: if s.shots_fired > 0 {
            (s.shots_hit_enemy as f32 / s.shots_fired as f32 * 100.0).round() as u32
        } else { 100 },
        time_s: s.elapsed / 100, // +0x30 counts 1/100 s [M 0x1995a0:10-13]
        bond_moves: s.bond_moments,
        dispatched: s.dispatched,
        subdued: s.disabled,
        surrendered: s.surrendered,
        bonus_pickups: s.bonus_pickups,
    }
}

/// The mission-flow movie glue (the GameFlow state-6 briefing/outro and its decode-ahead), one call
/// per redraw from `App::movie_tick` — a free function so the live tests drive the SAME code the
/// window runs (the briefing once never showed, and no test ran this path). Returns false
/// when no mission movie is due (the caller runs the page's aux loop instead).
/// The PC-port "Rapid fire" option (`--rapid-fire`, or the dev window's Toggles page): every held
/// weapon fires one round per tick from a full clip — a documented port-only option, off = the
/// game (the shipped game has no fire-rate control: the PS2's option screens carry no such row
/// [D — the options census, standins-drone-926's report] and the symtab has no rapid concept; the
/// game's cadence is the weapon data's own refire [M weapon_data]). Read into
/// the sim before each local tick; never applied to an online match.
pub static RAPID_FIRE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

pub fn mission_movie_glue(fe: &mut menu::FrontEnd, slot: &mut Option<movie::Player>, iso: &str) -> bool {
    if fe.flow.get_state() == menu::ST_MOVIE {
        // psiStartBackgroundMovie replaces whatever the one BGFMV slot holds [M 0x1081a8] —
        // a looping page backdrop standing in the slot must yield to the state-6 movie
        // (menu-audio-leaks: menu music played under the EA logo).
        if slot.as_ref().is_some_and(|p| p.is_looping()) {
            if let Some(p) = slot.take() { let c = p.code(); p.stop(); eprintln!("movie: stopped {c:08x} (state 6 replaces the backdrop)"); }
        }
        match slot {
            Some(p) => {
                // BGFMVPlaying follows the player: 1 while the movie's frames are unconsumed,
                // 0 once they are — state 6 pops through the psiMovieFinished arm on the next
                // main_tick [M 0x1082e8 + 0x1c98c8 case 6]. On EVERY build the hold is the
                // host's per-frame poll count to the movie's real length [M 0x1081a8's
                // BGFMVPlaying = 1 + 0x108060's end-of-file clear; D the PSS's audio track]:
                // the stand-in advances one frame per host call, then `finished` fires and the
                // pop is the game's own — the `movies` feature's decoder thread only feeds the
                // picture path and does not pace this decision (mission-outros-4).
                // One player poll per flow tick (60 Hz), not per redraw: at 240 Hz the per-redraw poll
                // ran the hold out in a quarter of the movie (the Double Cross outro cut out early).
                while fe.flow.movie_polled < fe.flow.movie_clock {
                    p.headless_advance();
                    fe.flow.movie_polled += 1;
                }
                fe.flow.movie_playing = !p.finished();
            }
            None => {
                let code = fe.flow.movie_code;
                if code != 0 {
                    // mission-outros: the subtitle script rides the movie [M 0x1d6360's
                    // SP_Create beside psiStartBackgroundMovie] — loaded once off 07000500.bin.
                    if fe.flow.outro_subs.is_none() {
                        fe.flow.outro_subs = menu::subtitle_track(iso, code);
                        if let Some(t) = &fe.flow.outro_subs {
                            eprintln!("movie: subtitle track {code:08x} → {} entries, length {}", t.entries.len(), t.length);
                        }
                    }
                    if !fe.flow.outro_hold_armed {
                        match movie::Player::start(iso, code, movie_sfx_volume(fe)) {
                            Ok(p) => {
                                eprintln!("movie: playing {code:08x} (state 6)");
                                fe.flow.movie_playing = true; // BGFMVPlaying = 1 [M 0x1081a8]
                                *slot = Some(p);
                            }
                            Err(e) => {
                                eprintln!("movie: {code:08x} not started: {e:?} — the headless countdown stands in");
                                fe.flow.movie_playing = false; // an un-started movie: BGFMVPlaying == 0 [M 0x1082e8]
                                // mission-outros: the no-movie hold runs the subtitle track's
                                // length — the game holds state 6 on the retail PSS's life
                                // [M 0x1082e8], and on a stubbed image (a test image's
                                // /MOVIES/*.PSS are 0-byte [D]) it would pop at once; the
                                // track's length keeps the outro up with its text. [S] the
                                // stand-in duration — the retail timing (the track spans the
                                // movie [D the parsed lengths 2695/2292]) — is the model.
                                if let Some(t) = &fe.flow.outro_subs { fe.flow.movie_frames = t.length; }
                                fe.flow.outro_hold_armed = true;
                            }
                        }
                    } else {
                        fe.flow.movie_playing = false;
                    }
                } else {
                    fe.flow.movie_playing = false;
                }
            }
        }
        return true;
    }
    if fe.flow.movie_pending {
        // A movie-wrapped load is on its way: the looping page backdrop yields the single
        // player slot to it [M psiStartBackgroundMovie replaces the playing movie].
        if let Some(p) = slot.as_ref() {
            if p.is_looping() {
                let code = p.code();
                if let Some(p) = slot.take() { p.stop(); }
                eprintln!("movie: stopped {code:08x} (a briefing is pending)");
            }
        }
        if slot.is_none() {
            // Early start: the decoder works through the fade and the rest of the menu so
            // state 6 opens on the first frame instead of a black screen (the game's IPU
            // starts streaming instantly [M 0x1081a8]). Any state: since gameflow-state-stack
            // the 84-frame fade hold [M 0x20ba50] runs before ST_MAPLOAD, and one redraw's 8
            // front-end ticks could carry the flow from the fade through phase 0's state-6 push
            // to its pop before this ever saw ST_MAPLOAD — the briefing skipped again
            // (`briefing_plays_live`, bisected to the a2cdd771 merge).
            let code = fe.flow.level_to_load;
            if matches!((code >> 20) & 0xff, 0x71 | 0x73) {
                // mission-outros: the subtitle script loads with the movie [M 0x1d6360's
                // SP_Create beside psiStartBackgroundMovie — the early-started decoder still
                // runs beside it].
                if fe.flow.outro_subs.is_none() {
                    fe.flow.outro_subs = menu::subtitle_track(iso, code);
                }
                match movie::Player::start(iso, code, movie_sfx_volume(fe)) {
                    Ok(p) => {
                        *slot = Some(p);
                        // BGFMVPlaying = 1 from the start [M psiStartBackgroundMovie 0x1081a8]: state 6
                        // (pushed by ResetMap_Load phase 0 with no frame budget of its own) must see
                        // the movie as playing on its first tick, or the countdown pops it at once
                        // and the briefing never shows (straight to the
                        // loading screen; it only worked while the boot intro left a budget behind).
                        fe.flow.movie_playing = true;
                        eprintln!("movie: decoding {code:08x} ahead of state 6");
                    }
                    Err(e) => eprintln!("movie: {code:08x} not started: {e:?} — the headless countdown stands in"),
                }
            }
        }
        return true;
    }
    false
}

/// net-p2p: the reachability config for the app's real sessions — None with NF_NO_REACH=1 (LAN
/// only); NF_STUN ("host:port,host:port") replaces the public STUN list [D — the whole net layer
/// is the port's: the shipped game's symtab carries no socket/inet symbol at all, so the env
/// config is the port's documented protocol, net-p2p.md §1's own framing].
fn reach_config() -> Option<nf_net::p2p::ReachConfig> {
    if std::env::var_os("NF_NO_REACH").is_some() { return None; }
    let mut cfg = nf_net::p2p::ReachConfig::default();
    if let Ok(s) = std::env::var("NF_STUN") {
        cfg.stun_servers = s.split(',').map(|x| x.trim().to_string()).filter(|x| !x.is_empty()).collect();
    }
    // The lobby directory (docs/tools/listing-server.md): NF_LOBBY_SERVER, else the
    // `lobby_server = host:port` line of <config dir>/net.txt; empty = none.
    let server = std::env::var("NF_LOBBY_SERVER").ok().or_else(|| {
        let text = std::fs::read_to_string(settings::config_dir()?.join("net.txt")).ok()?;
        text.lines().find_map(|l| l.split_once('=').filter(|(k, _)| k.trim() == "lobby_server").map(|(_, v)| v.trim().trim_matches('"').to_string()))
    });
    cfg.directory = server.filter(|s| !s.is_empty()).and_then(|s| {
        use std::net::ToSocketAddrs;
        let s = if s.contains(':') { s } else { format!("{s}:{}", nf_net::directory::DEFAULT_PORT) };
        s.to_socket_addrs().ok()?.find(|a| a.is_ipv4())
    });
    Some(cfg)
}

/// net-cloud-lobby: the lobby service's URL — `NF_LOBBY_SERVICE`, else the `lobby_service = <url>`
/// line of <config dir>/net.txt, else the built-in constant (empty until a service is deployed = off).
/// A player-set `nf-lobbyd` (`lobby_server =`) overrides it: None when the directory is set
/// [D — the game has no net layer (the symtab census), so this precedence is the port's own
/// documented protocol, docs/systems/net-cloud-lobby.md "Override"].
fn cloud_service_url(directory_set: bool) -> Option<String> {
    if directory_set { return None; }
    let raw = std::env::var("NF_LOBBY_SERVICE").ok().or_else(|| {
        let text = std::fs::read_to_string(settings::config_dir()?.join("net.txt")).ok()?;
        text.lines().find_map(|l| l.split_once('=').filter(|(k, _)| k.trim() == "lobby_service").map(|(_, v)| v.trim().trim_matches('"').to_string()))
    }).unwrap_or_else(|| nf_net::cloud::LOBBY_SERVICE_URL.to_string());
    let raw = raw.trim().to_string();
    (!raw.is_empty()).then_some(raw)
}

/// The real sessions' net services: reachability (net-p2p) + the cloud meeting point
/// (net-cloud-lobby, unless a player-set nf-lobbyd overrides it) [D — both services are the
/// port's own layer (the shipped game has none: the ACTION.ELF symtab carries no socket/inet
/// symbol), documented in docs/systems/net-p2p.md + net-cloud-lobby.md].
fn enable_net_services(p: &mpnet::SessionProxy) {
    let cfg = reach_config();
    if let Some(c) = &cfg { p.0.borrow_mut().enable_reach(c.clone()); }
    let dir_set = cfg.as_ref().is_some_and(|c| c.directory.is_some());
    p.0.borrow_mut().enable_cloud(cloud_service_url(dir_set));
}

/// The NF_NET_SHOT render-check aid's pending capture (set inside the tick loop's sim borrow).
static NET_SHOT_DUE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
static NET_SHOT_FX_TAKEN: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

/// NF_NET_RATE's stall counter.
static NET_STALLS: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
// ---------------------------------------------------------------------------
// headless-app-harness: the window host's test surface. `App` grew its per-frame
// logic (`frame`) and its key handling (`app_key`) as methods with no winit/wgpu
// objects; this module builds one with `gfx: None` and a silent sound stack and
// drives the EXACT path the window's event loop drives — three earlier bugs
// (the briefing pop, the mission Esc quit, the unpolled session
// through the load) all lived in code no test ran.
// ---------------------------------------------------------------------------
#[cfg(test)]
mod headless_app_harness {
    use super::*;
    use winit::keyboard::KeyCode;

    /// A windowless `App`. `frame(dt)` is one `RedrawRequested` minus the draw;
    /// `key(k)` is one key press. `exit_requested` is the old `el.exit()` arm.
    pub struct HeadlessApp {
        pub app: App,
        pub exit_requested: bool,
    }

    impl HeadlessApp {
        /// The window's boot: the front-end map 07000048 loaded, the front end up,
        /// the boot intro started, the game's strings read [M bootup 0x1c7e78 +
        /// ResetMap_Load 0x1d6360; run()'s front-end block]. `gfx: None`, sound
        /// silent. `UNLOCK_ALL` is the window's default (run(): every mission open
        /// unless --locked).
        pub fn boot(iso: &str) -> HeadlessApp {
            Self::silence();
            menu::UNLOCK_ALL.store(true, std::sync::atomic::Ordering::Relaxed);
            let scene = crate::load(iso, "07000048.bin", false).expect("the front-end map 07000048");
            let cam = start_camera(&scene);
            let mut app = App::bare(iso.to_string(), scene, cam, Self::opts(), None, 0.0);
            app.level_code = menu::FRONTEND_LEVEL;
            app.fe = Some(menu::FrontEnd::new());
            if let Some(fe) = app.fe.as_mut() { fe.start_boot_intro(); }
            if let Some(fe) = app.fe.as_mut() {
                // run(): the boot bank flip [M 0x1dceb8 case 0x7000048] re-points the driver's
                // primary slot (the driver exists here too: `App::bare` builds it, silent).
                let hash = fe.music.front_end_sound_ready();
                if let Some(s) = app.sound.as_mut() { s.driver.set_primary(if hash == 0x38 { 1 } else { 0 }); }
            }
            // run(): the game's strings for the front end (Txt_LoadLanguage 0x1caad0).
            if let Some(bytes) = read_files_bin(&app.iso, "USATxt.dat") {
                if let Ok(lang) = nf_formats::hud::Lang::parse(&bytes) { app.lang = Some(lang); }
            }
            HeadlessApp { app, exit_requested: false }
        }

        /// A mission straight off the command line's shape (run() with a level): the
        /// level loaded, `level_code` set (the fix the Esc-quit bug needed), no front
        /// end. The Esc/pause/quit tests ride this; the boot walk test proves the same
        /// state is reached through the real menus.
        pub fn mission(iso: &str, level: &str) -> HeadlessApp {
            Self::silence();
            let scene = crate::load(iso, level, false).unwrap_or_else(|e| panic!("{level}: {e}"));
            let cam = start_camera(&scene);
            let mut app = App::bare(iso.to_string(), scene, cam, Self::opts(), None, 0.0);
            app.level_code = u32::from_str_radix(level.trim_end_matches(".bin"), 16).unwrap_or(0);
            // fix-restart-health: the load re-saves the entry checkpoint [M 0x1a7ef8:325].
            app.entry_save = app.scene.sim.as_ref().map(entry_checkpoint_save);
            HeadlessApp { app, exit_requested: false }
        }

        fn silence() {
            // The suite sets this; a bare `cargo test` of this crate must not either.
            if std::env::var_os("NF_SILENT").is_none() { std::env::set_var("NF_SILENT", "1"); }
        }

        fn opts() -> scene::ViewOpts {
            scene::ViewOpts { reinforcements: false, firing: false, right: [1.0, 0.0, 0.0], up: [0.0, 1.0, 0.0] }
        }

        /// One frame: the window's RedrawRequested minus the draw. The movie pump
        /// (`present()` on the frames the draw would call it) rides here — the decoder's
        /// clock and the skip gate's `shown_for` live on it [M psiMovieFinished 0x1082e8].
        pub fn frame(&mut self, dt: f32) {
            self.app.frame(dt);
            if self.app.movie_plane_up() {
                if let Some(p) = self.app.movie.as_mut() { p.present(); }
            }
        }

        /// One key press (the game's actions are edge-triggered [M Input_Action(.., 1)]).
        pub fn key(&mut self, k: KeyCode) {
            if self.app.app_key(k, ElementState::Pressed, false) { self.exit_requested = true; }
        }

        /// `n` synthetic 60 Hz frames (logic ticks only — no wall time).
        pub fn frames(&mut self, n: u32) {
            for _ in 0..n { self.frame(1.0 / 60.0); }
        }

        /// Pump frames until `pred` or the wall budget ends — the load's hold and the
        /// movie clocks are wall-clock (`Job::elapsed_ticks`, `shown_for`), so their
        /// waits are real seconds; 4 ms a frame keeps the pump light.
        pub fn pump(&mut self, wall_secs: f32, mut pred: impl FnMut(&App) -> bool) -> bool {
            let t0 = std::time::Instant::now();
            while t0.elapsed().as_secs_f32() < wall_secs {
                self.frame(1.0 / 60.0);
                if pred(&self.app) { return true; }
                std::thread::sleep(std::time::Duration::from_millis(4));
            }
            false
        }

        /// The real menu walk, boot to the mission pick — the same walk
        /// briefing_plays_live drives, through the App's own key handling.
        /// Returns after `flow.level_to_load` names the briefing movie.
        fn walk_to_exchange(&mut self, iso: &str) {
            // The boot logos (P_INTRO) run out — real decodes with the movies feature,
            // the headless countdown without [M 0x209e88].
            assert!(self.pump(60.0, |a| a.fe.as_ref().is_some_and(|fe| fe.page == menu::Page::Start && !fe.intro_playing)),
                "the boot logos never reached the title");
            // START → the P_ESTHERO intro run [M 0x209e88 → 0x20a060].
            self.key(KeyCode::Enter);
            assert!(self.pump(10.0, |a| a.fe.as_ref().is_some_and(|fe| fe.page == menu::Page::EstHero)),
                "START never opened the P_ESTHERO run");
            // Its skip honours the 180-tick grace [M 0x215a50:6]; 210 ticks covers it.
            self.frames(210);
            self.key(KeyCode::Enter);
            assert!(self.pump(10.0, |a| a.fe.as_ref().is_some_and(|fe| fe.page == menu::Page::Main && !fe.intro_playing)),
                "the intro never reached the main menu");
            // Main → the Night Fire wheel → the save prompt → the difficulty → the mission
            // list [the briefing_plays walk's four confirms]. The manager's input block
            // (msg 0x68/1 — a delayed page change or the page's open block) swallows a press
            // [M 0x21c3d8:70], so each step presses until the page moves.
            let mut on_list = false;
            for _ in 0..20 {
                self.key(KeyCode::Enter);
                self.frames(10);
                if self.app.fe.as_ref().is_some_and(|fe| fe.page == menu::Page::MissionSelect) { on_list = true; break; }
            }
            assert!(on_list, "the walk never reached the mission list (page {:?})",
                self.app.fe.as_ref().map(|fe| fe.page));
            // The Exchange is row 1 of the mission list [L CAMPAIGN row 1 = 0x07000005].
            assert!(self.key_until_page(KeyCode::ArrowDown, 4.0, |fe| fe.cursor == 1),
                "the cursor never reached The Exchange row");
            let picked = self.key_until_page(KeyCode::Enter, 8.0, |fe| fe.flow.level_to_load == 0x0710_0005);
            assert!(picked, "the mission pick never wrapped to the briefing 0x07100005 [M 0x1d3800's 0x71 wrap]");
        }

        /// Press `k` every few frames until the front end satisfies `pred` — the input
        /// block may swallow a press right after a page change [M 0x21c3d8:70].
        fn key_until_page(&mut self, k: KeyCode, wall_secs: f32, mut pred: impl FnMut(&menu::FrontEnd) -> bool) -> bool {
            let t0 = std::time::Instant::now();
            while t0.elapsed().as_secs_f32() < wall_secs {
                self.key(k);
                self.frames(6);
                if self.app.fe.as_ref().is_some_and(|fe| pred(fe)) { return true; }
            }
            false
        }
    }

    /// Menu sounds were missing entirely: run() built the sound driver and dropped it, so the
    /// front end had none; the event-level tests (menu_sfx_flow) and the driver MODEL (menu_audio_flows) could not see
    /// it. This drives the REAL nf-audio driver through the real boot (silent: NF_SILENT, no output device): the
    /// driver exists, and START on the title starts the select blip 0x1da [docs/systems/menu-sfx.md §1 MENU_SOUND 0]
    /// in it.
    #[test]
    fn the_title_start_plays_its_blip_in_the_real_driver() {
        let Some(iso) = std::env::var_os("NF_ISO") else { return; };
        let iso = iso.to_string_lossy().to_string();
        let mut h = HeadlessApp::boot(&iso);
        assert!(h.app.sound.is_some(), "the front end has a sound driver (App::bare builds it)");
        assert!(h.pump(60.0, |a| a.fe.as_ref().is_some_and(|fe| fe.page == menu::Page::Start && !fe.intro_playing)), "title");
        let ids = |h: &HeadlessApp| h.app.sound.as_ref().unwrap().driver.ids();
        // Sister: the boot logos and the title start no select blip by themselves.
        assert!(!ids(&h).contains(&0x1da), "no select blip before START: {:x?}", ids(&h));
        h.key(KeyCode::Enter);
        h.frames(2);
        assert!(ids(&h).contains(&0x1da), "START's select blip 0x1da reaches the real driver: {:x?}", ids(&h));
    }

    /// (a) A fresh boot walked through the real menus to The Exchange: the briefing
    /// state holds while its movie plays, the level loads after it, and the host
    /// knows the level (the old Esc-quit bug's `level_code` half).
    #[test]
    fn boot_to_the_exchange_briefing_holds_then_the_level_loads() {
        let Some(iso) = std::env::var_os("NF_ISO") else { return; };
        let iso = iso.to_string_lossy().to_string();
        let mut h = HeadlessApp::boot(&iso);
        h.walk_to_exchange(&iso);

        // The briefing: the state-6 movie [M 0x1d6360 phase 0 → 0x1c98c8 case 6].
        let mut state6_frames = 0u32;
        let mut saw_player = false;
        let _ = h.pump(20.0, |a| {
            if a.fe.as_ref().is_some_and(|fe| fe.flow.get_state() == menu::ST_MOVIE) { state6_frames += 1; }
            if a.movie.as_ref().is_some_and(|p| p.code() == 0x0710_0005) { saw_player = true; }
            // Ends when the front end closes (the resolve) or the real movie has shown
            // 1.2 s (past the port's 1.0 s Confirm guard — the game's own skip has NO time gate
            // [M GameFlow_Main 0x1c98c8:145-149]; the 1.0 guard is the port's Confirm adapter
            // guard, menu_input, because Enter is the pick key, not the game's skip button).
            a.fe.is_none() || a.movie.as_ref().is_some_and(|p| p.code() == 0x0710_0005 && p.shown_for() >= 1.2)
        });
        // The hold is the game's only when a player can start: the on-foot ISO's PSS files
        // are zero-length stubs [D /MOVIES/30_FPS — NoMovieData], so the movie never starts —
        // and the game's own no-movie path does the same thing the port's headless countdown
        // does: psiStartBackgroundMovie leaves BGFMVPlaying 0 when the .pss fails to open
        // [M 0x1081a8:41-50], psiMovieFinished is `BGFMVPlaying == 0` [M 0x1082e8:6], and case 6
        // pops at once [M 0x1c98c8:145-149] — no state-6 frame is observable between the fe
        // ticks. briefing_plays_live pins the hold itself on a movie-bearing image.
        // movie-sfx-volume: master's start takes the SFX master volume [M 0x116e18]; the
        // harness has no option file here — the boot default 100 [L 0x30c95c].
        let movie_data = crate::movie::Player::start(&iso, 0x0710_0005, 100).is_ok();
        #[cfg(feature = "movies")]
        if movie_data {
            // Sister (fails on the pre-a2cdd771 behaviour: state 6 popped on its first
            // tick — straight to the loading screen): the movie
            // plane held state 6 for real frames.
            assert!(state6_frames >= 30, "the briefing was cut before it played ({state6_frames} state-6 frames) [M 0x1c98c8 case 6 holds while BGFMVPlaying]");
            assert!(saw_player, "the briefing 07100005 never started [M psiStartBackgroundMovie 0x1081a8]");
            // The skip — the game's action 0x19, edge-read with NO time gate in case 6
            // [M GameFlow_Main 0x1c98c8:145-149; the pad binding R1/D-pad-down mask 0x840, M
            // psiInput_MapInputs 0x10cd80:416; the port binds ArrowDown/R ungated in app_key and
            // Confirm behind the 1.0 guard in menu_input] — resolves the
            // phase-2 level and closes the front end [M 0x1d3560 + 0x1d6360 LoadState 2].
            h.key(KeyCode::Enter);
        }
        #[cfg(not(feature = "movies"))]
        let _ = (state6_frames, saw_player);
        let loading = h.pump(6.0, |a| a.async_load.is_some() && a.fe.is_none());
        assert!(loading, "the briefing resolve never started the level load");
        let loaded = h.pump(20.0, |a| a.level_code == 0x0700_0005 && a.async_load.is_none() && a.fe.is_none());
        assert!(loaded, "the level never finished loading (the 5 s LOADING_HOLD + the build)");
        assert_eq!(h.app.level_code, 0x0700_0005, "level_code must name the loaded mission [M GameState+0xc, ResetMap_Load phase 2]");
        assert!(h.app.scene.sim.is_some(), "the Exchange sim is up");
        assert!(!h.exit_requested, "the walk never asks to exit");
    }

    /// (a2) The state-6 skip keys, ported from the game's rule: action 0x19 (R1/D-pad-down,
    /// mask 0x840 [M psiInput_MapInputs 0x10cd80:416]) skips the briefing at ONCE — GameFlow
    /// case 6 reads Input_Action(-1, 0x19, 1) with no time gate [M GameFlow_Main 0x1c98c8:145-149]
    /// — and the port binds ArrowDown (the D-pad-down key) to that skip ungated (app_key's
    /// front-end arm). Movies side: ArrowDown ends the hold while the movie is still young
    /// (< 1.0 s, where the Confirm guard refuses) — fails on the stand-in, where ArrowDown was
    /// Action::Down and died in menu_input's ST_MOVIE arm. Sister: Enter inside the 1.0 s guard
    /// does NOT skip (the press that picked the mission must not skip its own briefing — the
    /// neighbour arm this port sits next to). On the slim ISO the stub PSS never start a movie,
    /// so state 6 pops at once with no key — the game's own no-movie path [M 0x1081a8:41-50 +
    /// 0x1082e8:6 + 0x1c98c8:145-149], pinned by the resolve the test above already asserts.
    #[test]
    fn the_briefing_skips_on_the_games_own_keys_and_confirm_keeps_its_guard() {
        let Some(iso) = std::env::var_os("NF_ISO") else { return; };
        let iso = iso.to_string_lossy().to_string();
        let movie_data = crate::movie::Player::start(&iso, 0x0710_0005, 100).is_ok();
        #[cfg(feature = "movies")]
        if movie_data {
            // Fails on the stand-in: the ungated game skip key.
            let mut h = HeadlessApp::boot(&iso);
            h.walk_to_exchange(&iso);
            assert!(h.pump(20.0, |a| a.movie.as_ref().is_some_and(|p| p.code() == 0x0710_0005 && p.shown_for() >= 0.2)),
                "the briefing 07100005 never started playing");
            h.key(KeyCode::ArrowDown);
            let skipped = h.pump(8.0, |a| a.async_load.is_some() && a.fe.is_none());
            assert!(skipped, "ArrowDown did not skip the young briefing (the game's action 0x19 skips at once [M 0x1c98c8:145-149])");
            let loaded = h.pump(20.0, |a| a.level_code == 0x0700_0005 && a.async_load.is_none() && a.fe.is_none());
            assert!(loaded, "the skip never resolved the level load");

            // Sister (guards the Confirm neighbour): Enter inside the 1.0 s guard must NOT skip.
            // The movie clock is wall time, so the press lands < 1.0 s after the pick.
            let mut h2 = HeadlessApp::boot(&iso);
            h2.walk_to_exchange(&iso);
            assert!(h2.pump(20.0, |a| a.movie.as_ref().is_some_and(|p| p.code() == 0x0710_0005 && p.shown_for() >= 0.1)),
                "the second briefing never started playing");
            h2.key(KeyCode::Enter);
            h2.frames(30);
            assert!(h2.app.fe.as_ref().is_some_and(|fe| fe.flow.get_state() == menu::ST_MOVIE),
                "Enter inside the 1.0 s guard skipped the briefing — the Confirm guard regressed");
        }
        #[cfg(not(feature = "movies"))]
        let _ = movie_data;
    }

    /// (b) Esc in a menu-started mission opens the pause page; Esc again resumes; the
    /// process never requests an exit. The old behaviour quit the game (level_code was
    /// only ever set from the command line, so the Esc arm saw no mission).
    #[test]
    fn esc_in_a_mission_pauses_then_resumes_and_never_quits() {
        let Some(iso) = std::env::var_os("NF_ISO") else { return; };
        let mut h = HeadlessApp::mission(iso.to_string_lossy().as_ref(), "07000005.bin");
        assert_eq!(h.app.level_code, 0x0700_0005, "the mission boot knows its level");
        h.frames(10);
        // Esc #1: the pause page — the port's Esc is the PC adapter for the game's Start pause
        // (the pad has no Esc; the binding is the documented input adapter): the game's rule is
        // action 0x1e gated on GameState+0x36 == 0 && ScriptCam == 0, opening page 0x4000004b +
        // GS_PauseGame(1) [M Player_Update 0x193600:71-77].
        h.key(KeyCode::Escape);
        assert!(h.app.fe.as_ref().is_some_and(|fe| fe.page == menu::Page::Pause),
            "Esc in the mission did not open the pause page");
        assert!(!h.exit_requested, "the pause Esc requested an exit");
        h.frames(10);
        // Esc #2: Back on the pause page resumes [App::menu_input's Pause+Back arm;
        // the game's resume is its own row, C_GCPAUSE_Handler 0x204408].
        h.key(KeyCode::Escape);
        assert!(h.app.fe.is_none(), "the second Esc did not resume");
        assert!(!h.exit_requested, "the resume Esc requested an exit (the old quit bug)");
        assert!(h.app.scene.sim.is_some(), "the mission is still live after the resume");
        h.frames(10);
        assert!(h.app.scene.sim.as_ref().is_some_and(|s| s.drones.frame >= 10), "the sim resumed ticking");
    }

    /// (c) Pause > Quit returns to the mission list [M C_GCPAUSE_Handler 0x204408:141-152 —
    /// +0x1c = 0x4000001c, ResetMap_LevelToLoad(0x7000048, 0, 0), the fade, the delete].
    #[test]
    fn pause_quit_returns_to_the_mission_list() {
        let Some(iso) = std::env::var_os("NF_ISO") else { return; };
        let mut h = HeadlessApp::mission(iso.to_string_lossy().as_ref(), "07000005.bin");
        h.key(KeyCode::Escape);
        assert!(h.app.fe.as_ref().is_some_and(|fe| fe.page == menu::Page::Pause));
        // The MISSION rows: CONTINUE / RESTART / QUIT [L page 0x4000004b's d1=6 list] —
        // cursor 2 = QUIT, its YES/NO box opens on NO [M 0x204408:181/204].
        h.key(KeyCode::ArrowDown);
        h.key(KeyCode::ArrowDown);
        h.key(KeyCode::Enter);
        assert!(h.app.fe.as_ref().is_some_and(|fe| fe.confirming_quit), "the QUIT row did not arm its confirm box");
        h.key(KeyCode::ArrowUp); // NO → YES
        h.key(KeyCode::Enter);
        // The front-end map loads and the queued page reopens on it [M 0x1d6360:160-163].
        let loaded = h.pump(25.0, |a| a.level_code == menu::FRONTEND_LEVEL && a.async_load.is_none());
        assert!(loaded, "the quit never loaded the front-end map 0x07000048");
        assert!(h.app.fe.as_ref().is_some_and(|fe| fe.page == menu::Page::MissionSelect),
            "the front end did not reopen on the mission list (0x4000001c) [M 0x204408:141-152]");
        assert!(!h.exit_requested, "the quit flow never asks to exit");
        assert!(h.app.scene.sim.as_ref().is_some_and(|s| s.level == menu::FRONTEND_LEVEL),
            "the front-end map 07000048 is the level running after the quit");
    }

    /// (d) A level load keeps an online session polled: a fake 6 s load (the session's
    /// timeout is 5 s [M nf-net Config.timeout_ms 5000]) must not drop the peer. The old
    /// frame returned early through the loading screen without the poll — a 5.2 s arena
    /// load dropped the other player before tick 0 (net-consolidate's fix, now pinned).
    #[test]
    fn a_level_load_keeps_the_online_session_polled() {
        let Some(iso) = std::env::var_os("NF_ISO") else { return; };
        let iso = iso.to_string_lossy().to_string();
        let mut h = HeadlessApp::mission(&iso, "07000005.bin");
        // A real loopback pair (the same cores the lobby pages wrap).
        let mut bound = None;
        'ports: for p in 27970..27990 {
            if let Ok(s) = mpnet::SessionProxy::host(p) { bound = Some((s, p)); break 'ports; }
        }
        let Some((host, port)) = bound else { panic!("no UDP port to bind"); };
        let client = mpnet::SessionProxy::connect(&format!("127.0.0.1:{port}")).expect("dial the host");
        h.app.net = Some(host.clone());
        // The handshake: HELLO → WELCOME → AgentJoined.
        let t0 = std::time::Instant::now();
        let joined = loop {
            host.0.borrow_mut().poll();
            client.0.borrow_mut().poll();
            let evs = host.0.borrow_mut().take_app_events();
            if evs.iter().any(|e| matches!(e, mpnet::CoreEvent::AgentJoined { .. })) { break true; }
            if t0.elapsed().as_secs_f32() > 10.0 { break false; }
            std::thread::sleep(std::time::Duration::from_millis(4));
        };
        assert!(joined, "the loopback client never joined the host");
        assert!(h.app.net.as_ref().is_some_and(|s| s.0.borrow().local_slot() == 0), "the host is slot 0");

        // The fake 6 s load: the worker sleeps past the 5 s timeout, then builds the real
        // scene (the same Result<Scene, String> the level path returns).
        let iso2 = iso.clone();
        let job = async_load::Job::spawn(move || {
            std::thread::sleep(std::time::Duration::from_secs(6));
            crate::load(&iso2, "07000005.bin", false).map_err(|e| e.to_string())
        });
        h.app.async_load = Some(LevelLoad { job, next: 0x0700_0005, ptp_return: None, ready: None });

        // The load's frames: each one polls the session through the async branch (the fix
        // under test). The client's poll is the peer's own app loop.
        let t0 = std::time::Instant::now();
        let mut done = false;
        while t0.elapsed().as_secs_f32() < 25.0 {
            h.frame(1.0 / 60.0);
            client.0.borrow_mut().poll();
            if h.app.level_code == 0x0700_0005 && h.app.async_load.is_none() { done = true; break; }
            std::thread::sleep(std::time::Duration::from_millis(4));
        }
        assert!(done, "the 6 s load never finished");
        // The peer survived on both sides: the session still adopted (a drop would have
        // run net_fail — session gone, the notice page up), no timeout event anywhere.
        assert!(h.app.net.is_some(), "the session was dropped during the load (the old behaviour)");
        assert!(h.app.fe.is_none() || h.app.fe.as_ref().is_some_and(|fe| fe.net_notice.is_none()),
            "net_fail raised the disconnect notice through the load");
        assert!(!host.core_is_closed(), "the host core closed through the load");
        let host_events = host.0.borrow_mut().take_app_events();
        assert!(host_events.iter().all(|e| !matches!(e, mpnet::CoreEvent::ClientLeft { .. })), "the host dropped the client through the load");
        assert!(host_events.iter().all(|e| !matches!(e, mpnet::CoreEvent::Desync { .. })));
        let client_events = client.0.borrow_mut().take_app_events();
        assert!(client_events.iter().all(|e| !matches!(e, mpnet::CoreEvent::HostTimeout)), "the client timed the host out through the load");
    }

    /// Auto-aim only worked through the dev T key — the Advanced options'
    /// Auto Aim (default ON, profile +0x01 [M Player_AutoAim 0x1a7c98:41]) was stored and never
    /// read. The option now drives the sim on every frame; sister: turning it off turns it off.
    #[test]
    fn the_auto_aim_option_drives_the_sim() {
        let Ok(iso) = std::env::var("NF_ISO") else { return };
        let mut h = HeadlessApp::mission(&iso, "07000005.bin");
        h.frames(2);
        assert!(h.app.scene.sim.as_ref().unwrap().auto_aim, "Auto Aim defaults ON, as the game's profile does");
        h.app.auto_aim_opt = false;
        h.frames(2);
        assert!(!h.app.scene.sim.as_ref().unwrap().auto_aim, "the option set Off turns it off");
    }

    // standins-drone-922: the opponent object fold — the game's `Drone_tag+0x170` points at
    // Bond while he lives [M NDrone2_FindOpponent 0x1424e0:100-120] and zeroes when his
    // class-3 health ≤ 0; the port stamps `bond_alive` per tick from the health.
    #[test]
    fn the_opponent_pointer_follows_bonds_life() {
        let Ok(iso) = std::env::var("NF_ISO") else { return };
        let mut h = HeadlessApp::mission(&iso, "07000005.bin");
        h.frames(2);
        assert!(h.app.scene.sim.as_ref().unwrap().drones.list.iter().all(|dr| dr.bond_alive),
            "a live Bond is every guard's opponent");
        h.app.scene.sim.as_mut().unwrap().drones.health.health = 0.0;
        h.frames(1);
        assert!(h.app.scene.sim.as_ref().unwrap().drones.list.iter().all(|dr| !dr.bond_alive),
            "a dead Bond leaves no opponent [M 0x1424e0:110-118]");
    }

    // bond-moment-flash: a NEW Bond moment sets BL+0x948 = 200
    // [M PlrStat_DoneBondMoment 0x199448:23-25]; the health pane takes 1 per frame while >= 1
    // [M HUD_UpdateHealthPane 0x19b9c8:258-260]. Sister: a moment already done when the
    // level starts never flashes.
    #[test]
    fn a_new_bond_moment_flashes_the_007_for_200_frames() {
        let Ok(iso) = std::env::var("NF_ISO") else { return };
        let mut h = HeadlessApp::mission(&iso, "07000005.bin");
        h.app.scene.sim.as_mut().unwrap().drones.progress.bond_moments[1] = 1; // done before the first tick
        h.frames(3);
        assert_eq!(h.app.scene.sim.as_ref().unwrap().hud.bond_moment_flash, 0, "a moment done at load does not flash");
        h.app.scene.sim.as_mut().unwrap().drones.progress.bond_moments[0] = 1;
        h.frames(1);
        let f0 = h.app.scene.sim.as_ref().unwrap().hud.bond_moment_flash;
        assert_eq!(f0, 200, "the new moment sets 200");
        h.frames(100);
        assert_eq!(h.app.scene.sim.as_ref().unwrap().hud.bond_moment_flash, 100, "-1 per frame");
        h.frames(150);
        assert_eq!(h.app.scene.sim.as_ref().unwrap().hud.bond_moment_flash, 0, "floored at 0");
    }

    /// The release's real path for the Auto Aim option: boot → title → main menu → Edit Codename →
    /// Default → Advanced → Auto Aim ▶ Off → Confirm → back to the main menu → Night Fire → The
    /// Exchange through the briefing → the loaded level's sim has auto aim OFF. The option file
    /// goes to a scratch config dir (Confirm saves it — never the player's %APPDATA% copy).
    #[test]
    fn auto_aim_off_in_the_advanced_menu_reaches_the_mission() {
        let Some(iso) = std::env::var_os("NF_ISO") else { return; };
        let iso = iso.to_string_lossy().to_string();
        let dir = std::env::temp_dir().join(format!("nf-autoaim-walk-{}", std::process::id()));
        let _ = std::fs::create_dir_all(&dir);
        std::env::set_var("NF_CONFIG_DIR", &dir);
        let mut h = HeadlessApp::boot(&iso);
        assert!(h.pump(60.0, |a| a.fe.as_ref().is_some_and(|fe| fe.page == menu::Page::Start && !fe.intro_playing)), "title");
        h.key(KeyCode::Enter);
        assert!(h.pump(10.0, |a| a.fe.as_ref().is_some_and(|fe| fe.page == menu::Page::EstHero)), "intro run");
        // The intro run: skip it only while it is still up (on a movie-less image the headless
        // countdown ends it at once — an Enter then would pick Night Fire on Main).
        h.frames(210);
        if h.app.fe.as_ref().is_some_and(|fe| fe.intro_playing) { h.key(KeyCode::Enter); }
        assert!(h.pump(120.0, |a| a.fe.as_ref().is_some_and(|fe| fe.page == menu::Page::Main && !fe.intro_playing)), "main menu");
        // Main row 2 = Edit Codename [menus_flow the_codenames_row_reaches_the_option_pages].
        assert!(h.key_until_page(KeyCode::ArrowDown, 4.0, |fe| fe.cursor == 2), "cursor to Edit Codename");
        assert!(h.key_until_page(KeyCode::Enter, 4.0, |fe| fe.page == menu::Page::CnSelect), "Edit Codename opens the codename list");
        assert!(h.key_until_page(KeyCode::Enter, 4.0, |fe| fe.page == menu::Page::CnMenu), "Default opens its wheel");
        assert!(h.key_until_page(KeyCode::ArrowDown, 4.0, |fe| fe.cursor == 3), "the wheel to Advanced (row 3)");
        assert!(h.key_until_page(KeyCode::Enter, 4.0, |fe| fe.page == menu::Page::CnAdvanced), "Advanced opens");
        let auto_aim = menu::ADV_ROWS[1].ctrl;
        assert_eq!(h.app.fe.as_ref().unwrap().opt_value(auto_aim), 1, "Auto Aim starts On");
        assert!(h.key_until_page(KeyCode::ArrowDown, 4.0, |fe| fe.cursor == 1), "cursor to Auto Aim");
        assert!(h.key_until_page(KeyCode::ArrowRight, 4.0, |fe| fe.opt_value(auto_aim) == 0), "▶ sets Auto Aim Off");
        assert!(h.key_until_page(KeyCode::Enter, 4.0, |fe| fe.page != menu::Page::CnAdvanced), "Confirm keeps the edit");
        assert_eq!(h.app.fe.as_ref().unwrap().opt_value(auto_aim), 0, "the confirmed value is Off");
        assert!(h.key_until_page(KeyCode::Escape, 6.0, |fe| fe.page == menu::Page::Main), "back out to the main menu");
        assert!(h.key_until_page(KeyCode::ArrowUp, 4.0, |fe| fe.cursor == 0), "cursor to Night Fire");
        let mut on_list = false;
        for _ in 0..20 {
            h.key(KeyCode::Enter);
            h.frames(10);
            if h.app.fe.as_ref().is_some_and(|fe| fe.page == menu::Page::MissionSelect) { on_list = true; break; }
        }
        assert!(on_list, "the mission list (page {:?})", h.app.fe.as_ref().map(|fe| fe.page));
        assert!(h.key_until_page(KeyCode::ArrowDown, 4.0, |fe| fe.cursor == 1), "The Exchange row");
        assert!(h.key_until_page(KeyCode::Enter, 8.0, |fe| fe.flow.level_to_load == 0x0710_0005), "the pick");
        let loaded = h.pump(90.0, |a| {
            a.fe.is_none() && a.level_code == 0x0700_0005 && a.scene.sim.is_some() && a.async_load.is_none()
        });
        if !loaded { h.key(KeyCode::Enter); } // a movie-bearing image: skip the briefing
        assert!(loaded || h.pump(90.0, |a| a.fe.is_none() && a.level_code == 0x0700_0005 && a.scene.sim.is_some() && a.async_load.is_none()),
            "The Exchange loaded (level {:#x})", h.app.level_code);
        h.frames(2);
        assert!(!h.app.scene.sim.as_ref().unwrap().auto_aim, "the menu's Auto Aim Off reaches the mission's sim");
        let _ = std::fs::remove_dir_all(&dir);
    }

    // options-census: the windowed drain (App::drain_options) moves every wired row to its
    // consumer. Each block carries a flipped sister so a constant-pass cannot sneak in.
    #[test]
    fn the_option_drain_moves_the_wired_rows() {
        let Ok(iso) = std::env::var("NF_ISO") else { return };
        let mut h = HeadlessApp::mission(&iso, "07000005.bin");
        let mut fe = menu::FrontEnd::new();
        fe.opts = menu::default_opts();
        // Sisters first: the defaults assert one way, the overrides the other.
        h.app.fe = Some(fe);
        h.app.drain_options();
        {
            let sim = h.app.scene.sim.as_ref().unwrap();
            assert!(sim.drones.crosshair, "Crosshairs default On [M Input_Init 0x17edd8]");
            assert!(sim.drones.auto_switch, "Weapon Auto Switch default On [M 0x17edd8]");
            assert!(sim.drones.near_flash, "the near-flash gate defaults On [M 0x17edd8 +0x0c = 1]");
            assert!(!sim.drones.decay_gate, "the pane-fade gate defaults Off [M 0x17edd8 +0x0b = 0]");
            assert!(!sim.drones.subtitle_track.enabled, "Subtitles default Off [M bootup 0x1c7e78:34]");
            assert!(sim.drones.crouch_toggle, "Crouch defaults Toggle [M 0x17edd8 +0x04 = 1]");
        }
        let fe = h.app.fe.as_mut().unwrap();
        fe.opts.insert(0x1000_0124, 0);
        fe.opts.insert(0x1000_01cb, 0);
        fe.opts.insert(0x1000_0226, 0);
        fe.opts.insert(0x1000_023b, 1);
        fe.opts.insert(0x1000_0136, 1);
        fe.opts.insert(0x1000_0125, 0);
        h.app.drain_options();
        let sim = h.app.scene.sim.as_ref().unwrap();
        assert!(!sim.drones.crosshair, "Crosshairs Off hides the sprite [M HUD_UpdateCrossHair 0x19e6d0:96]");
        assert!(!sim.drones.auto_switch, "Weapon Auto Switch Off gates the better-weapon take [M 0x1a8a58:76]");
        assert!(!sim.drones.near_flash, "the near-flash tint gate Off [M View_SetupRenderModes_Tweak 0x1e5888:26]");
        assert!(sim.drones.decay_gate, "the pane-fade gate On decays the alpha [M Player_Update 0x193600:414]");
        assert_eq!(sim.drones.pain_alpha, 1.0, "the fade alpha sits at 1.0 until damage/pickup [M 0x1910f8:127]");
        assert!(sim.drones.subtitle_track.enabled, "Subtitles On lets the captions through [M Sound_DoSubtitle 0x1dde58:21]");
        assert!(!sim.drones.crouch_toggle, "Crouch Hold [M Player_SSCrouch 0x1924c0:84-105]");
    }

    // fix-909c: the front end is closed during play (`fe` None); the options must hold from the
    // last-seen copy — a new level's fresh sim starts at the defaults and the drain re-applies
    // the player's rows. Sister: a run that never had a front end keeps the Auto Aim mirror.
    #[test]
    fn options_hold_while_the_front_end_is_closed() {
        let Ok(iso) = std::env::var("NF_ISO") else { return };
        let mut h = HeadlessApp::mission(&iso, "07000005.bin");
        h.app.drain_options();
        assert!(h.app.scene.sim.as_ref().unwrap().auto_aim, "sister: no front end ever — the mirror (On)");
        let mut fe = menu::FrontEnd::new();
        fe.opts = menu::default_opts();
        fe.opts.insert(0x1000_011b, 0); // Auto Aim Off
        fe.opts.insert(0x1000_0124, 0); // Crosshairs Off
        h.app.fe = Some(fe);
        h.app.drain_options();
        h.app.fe = None; // play: the front end closes
        {   // a fresh level's sim starts at the defaults
            let sim = h.app.scene.sim.as_mut().unwrap();
            sim.auto_aim = true;
            sim.drones.crosshair = true;
        }
        h.app.drain_options();
        let sim = h.app.scene.sim.as_ref().unwrap();
        assert!(!sim.auto_aim, "Auto Aim Off holds with the front end closed");
        assert!(!sim.drones.crosshair, "Crosshairs Off holds with the front end closed");
    }

    // options-census: the auto-aim pair — the SP row in a single-player match, the MP row in an
    // MP one [M Player_AutoAim 0x1a7c98:39-45 picks DAT_002a38c9 vs DAT_002a38ca on
    // MPSettings+0x180; the port's MP detector is [S], see the drain's comment].
    #[test]
    fn the_mp_auto_aim_row_wins_in_mp() {
        let Ok(iso) = std::env::var("NF_ISO") else { return };
        let mut h = HeadlessApp::mission(&iso, "07000005.bin");
        let mut fe = menu::FrontEnd::new();
        fe.opts = menu::default_opts();
        fe.opts.insert(0x1000_011b, 0); // the SP row Off
        fe.opts.insert(0x1000_0129, 1); // the MP row On
        h.app.fe = Some(fe);
        h.app.drain_options();
        assert!(!h.app.scene.sim.as_ref().unwrap().auto_aim, "an SP match reads the SP row");
        h.app.scene.sim.as_mut().unwrap().drones.mp_shared_pickups = true; // the port's MP-match mark [S]
        h.app.drain_options();
        assert!(h.app.scene.sim.as_ref().unwrap().auto_aim, "an MP match reads the MP row [M 0x1a7c98:44]");
    }

    // options-census: the pane-fade alpha (BLData+0x8bc) decays under the gate and pins at 1.0
    // without it [M Player_Update 0x193600:414-429].
    #[test]
    fn the_pain_alpha_decays_under_the_gate() {
        let Ok(iso) = std::env::var("NF_ISO") else { return };
        let mut h = HeadlessApp::mission(&iso, "07000005.bin");
        h.app.scene.sim.as_mut().unwrap().drones.decay_gate = false;
        h.app.scene.sim.as_mut().unwrap().drones.pain_alpha = 1.0;
        h.frames(3);
        assert_eq!(h.app.scene.sim.as_ref().unwrap().drones.pain_alpha, 1.0, "row Off pins 1.0 [M 0x193600:414]");
        h.app.scene.sim.as_mut().unwrap().drones.decay_gate = true;
        h.frames(3);
        assert!(
            h.app.scene.sim.as_ref().unwrap().drones.pain_alpha < 1.0,
            "row On decays 0.0020833/tick [M 0x193600:414-429]"
        );
    }

    // options-census: the Crouch option picks the stand condition [M Player_SSCrouch
    // 0x1924c0:84-105]. Toggle stands on the crouch/jump press edge; Hold stands when the
    // crouch action is not held. The transitions carry a 60-tick lockout [M 0x1924c0], so each
    // waits it out before the next press.
    #[test]
    fn the_crouch_option_picks_the_stand_condition() {
        let Ok(iso) = std::env::var("NF_ISO") else { return };
        let mut h = HeadlessApp::mission(&iso, "07000005.bin");
        let inp = |crouch: bool, toggle: bool| nf_game::player::Input { crouch, crouch_hold: !toggle, ..Default::default() };
        let sim = h.app.scene.sim.as_mut().unwrap();
        sim.drones.crouch_toggle = true;
        sim.step_bond(inp(true, true), false, false, false);
        assert!(sim.bond.crouched, "toggle: a crouch edge crouches");
        for _ in 0..61 { sim.step_bond(inp(false, true), false, false, false); }
        assert!(sim.bond.crouched, "toggle: no edge holds the crouch");
        sim.step_bond(inp(true, true), false, false, false);
        assert!(!sim.bond.crouched, "toggle: the second edge stands [M 0x1924c0:99-105]");
        for _ in 0..61 { sim.step_bond(inp(false, false), false, false, false); }
        sim.step_bond(inp(true, false), false, false, false);
        assert!(sim.bond.crouched, "hold: the held action crouches [M 0x1924c0:84-88]");
        for _ in 0..61 { sim.step_bond(inp(true, false), false, false, false); }
        assert!(sim.bond.crouched, "hold: still held, still crouched");
        sim.step_bond(inp(false, false), false, false, false);
        assert!(!sim.bond.crouched, "hold: released stands [M 0x1924c0:84-88]");
    }

    // profile-options-2 (the re-land of 610's volume test on master's drain path): the AV page's
    // two sliders reach the audio driver's masters. The game commits slider·5 on the page's
    // exit-confirm [M P_CNAVOPTIONS_Handler 0x214818:70-75] into SFXMusicSetVolume 0x116cf8 /
    // SFXSetVolume 0x116d98 (MusicMasterVolume 0x30c958 / SfxMasterVolume 0x30c95c); the port's
    // per-frame drain writes the same two statics (docs/systems/options-census.md).
    #[test]
    fn the_av_volume_rows_reach_the_driver_masters() {
        let Ok(iso) = std::env::var("NF_ISO") else { return };
        let mut h = HeadlessApp::mission(&iso, "07000005.bin");
        let mut fe = menu::FrontEnd::new();
        fe.opts = menu::default_opts();
        fe.opts.insert(0x1000_0135, 100); // the music slider's 20 · 5
        fe.opts.insert(0x1000_0134, 55);  // the effects slider's 11 · 5
        h.app.fe = Some(fe);
        h.app.drain_options();
        {
            let s = h.app.sound.as_ref().expect("the harness boots a silent driver");
            assert_eq!(s.driver.music_master, 100, "the music slider lands on MusicMasterVolume [M 0x116cf8]");
            assert_eq!(s.driver.master, 55, "the effects slider lands on SfxMasterVolume [M 0x116d98]");
        }
        // Sister: a lowered slider re-syncs on the next drain (the game commits on page exit;
        // the per-frame drain is the same net effect through the persisted profile).
        h.app.fe.as_mut().unwrap().opts.insert(0x1000_0135, 0);
        h.app.drain_options();
        assert_eq!(h.app.sound.as_ref().unwrap().driver.music_master, 0, "the lowered music slider re-syncs");
        // Sister 2: the driver's own boot defaults are the game's [L MusicMasterVolume 70 /
        // SfxMasterVolume 100 at 0x30c958/0x30c95c — Driver::new carries them].
        assert_eq!(nf_audio::sfx::Driver::new().music_master, 70);
        assert_eq!(nf_audio::sfx::Driver::new().master, 100);
    }
}

#[cfg(test)]
mod bonus_pause_stats {
    //! bonus-007-2: the pause SCORE tab's counter mapping — `pause_stats` reads the running
    //! Sim's `PlrMissionStats` rows (+0x20 Bond moves / +0x0C dispatched / +0x10 subdued /
    //! +0x14 surrendered / +0x24 the 007 bonus pickups → the tab's five counters)
    //! [M C_GCPAUSE_Handler 0x204408:447-498; docs/fields/player-stats.md]. ISO-gated: it builds
    //! a real level's Sim through the crate's own `load`.

    #[test]
    fn pause_stats_maps_the_progress_rows_the_tab_draws() {
        let Some(iso) = std::env::var("NF_ISO").ok().filter(|p| std::path::Path::new(p).exists()) else {
            eprintln!("skip: NF_ISO not set");
            return;
        };
        let mut scene = crate::load(&iso, "07000005.bin", false).expect("load 07000005");
        // Sister first: an untouched level starts with every SCORE counter at 0 — a non-zero
        // default would make the mapping assertions below vacuous.
        let zero = crate::pause_stats(scene.sim.as_ref().unwrap());
        assert_eq!(
            (zero.bond_moves, zero.dispatched, zero.subdued, zero.surrendered, zero.bonus_pickups),
            (0, 0, 0, 0, 0),
            "a fresh level's SCORE counters must all read 0"
        );
        // The main: poke the record the game's Log functions write, then assert the exact row
        // mapping — the tab draws what `PlrMissionStats` holds, nothing else. Distinct values so
        // any crossed wire between rows fails.
        scene.sim.as_mut().unwrap().drones.progress.stats[0].bond_moments = 2;
        scene.sim.as_mut().unwrap().drones.progress.stats[0].dispatched = 3;
        scene.sim.as_mut().unwrap().drones.progress.stats[0].disabled = 4;
        scene.sim.as_mut().unwrap().drones.progress.stats[0].surrendered = 5;
        scene.sim.as_mut().unwrap().drones.progress.stats[0].bonus_pickups = 6;
        let s = crate::pause_stats(scene.sim.as_ref().unwrap());
        assert_eq!(s.bond_moves, 2, "+0x20 → the Bond Moves row");
        assert_eq!(s.dispatched, 3, "+0x0C → Dispatched");
        assert_eq!(s.subdued, 4, "+0x10 → Subdued");
        assert_eq!(s.surrendered, 5, "+0x14 → Surrendered");
        assert_eq!(s.bonus_pickups, 6, "+0x24 → the 007 Bonus row");
    }
}

#[cfg(test)]
mod beam_reattach {
    //! Render interpolation: the gadget beam re-attaches to the interpolated
    //! camera. Port presentation only.
    use super::*;

    fn cam(yaw: f32) -> Camera { Camera { pos: Vec3::new(1.0, 2.0, 3.0), yaw, pitch: 0.1, fov: 1.0 } }

    #[test]
    fn the_beam_stays_fixed_in_the_view_through_a_turn() {
        // A beam fixed relative to the view (watch to crosshair), its z stretched by 7 units.
        let mut rel = [[1.0, 0.0, 0.0, 0.0], [0.0, 1.0, 0.0, 0.0], [0.0, 0.0, 7.0, 0.0], [0.2, -0.3, 0.5, 1.0]];
        rel[2][1] = 0.4;
        let (cp, cc) = (cam(0.0), cam(0.5));
        let wp = nf_formats::anim::mul(&rel, &cam_view_to_world(&cp));
        let wc = nf_formats::anim::mul(&rel, &cam_view_to_world(&cc));
        let draw = lerp_cam(&cp, &cc, 0.5);
        let mut out = vec![lerp_entry(&wp, &wc, 0.5)];
        reattach_beams(&mut out, &[wp], &[wc], 0.5, &cp, &cc, &draw, &[0]);
        let back = nf_formats::anim::mul(&out[0], &cam_world_to_view(&draw));
        for i in 0..4 { for j in 0..4 { assert!((back[i][j] - rel[i][j]).abs() < 1e-4, "[{i}][{j}] {} vs {}", back[i][j], rel[i][j]); } }
        // Sister: the plain world blend alone leaves the beam's start off the watch mid-turn.
        let plain = nf_formats::anim::mul(&lerp_entry(&wp, &wc, 0.5), &cam_world_to_view(&draw));
        let off = (0..3).map(|k| (plain[3][k] - rel[3][k]).powi(2)).sum::<f32>().sqrt();
        assert!(off > 1e-3, "the world blend drifts ({off}) — the case this fixes");
    }

}
