//! Bond's auto-aim maths in the game's own units: the crosshair offset `BL+0x120 / +0x124` is a
//! screen position (x right, y up, ±1 at the screen edges, the units `HUD_UpdateCrossHair` 0x19e6d0
//! draws), not an angle. `Check_AutoAim` 0x1a73f8 writes a target's projected screen position into it
//! and `Player_GetAimingPoint` 0x1a4a98 turns it back into the shot's aim point. Spec:
//! docs/systems/auto-aim.md.

use crate::trig;
use nf_formats::ps2float::{div, mul};

type V3 = [f32; 3];

/// The viewer's size (`Viewer+0x10C / +0x110`, `Camera_Create` 0x1bb0f8 [M; L 512 × 448]).
pub const SCREEN_W: f32 = 512.0;
pub const SCREEN_H: f32 = 448.0;
/// The on-foot view angle `Camera_CalcViewAngles(0x3F860A92)` [M Player_Update 0x193600:155].
const VIEW_ANGLE: f32 = f32::from_bits(0x3f86_0a92);
/// `Viewer+0x114`, the DAT_002a37c4 == 0 aspect [M 0x1baf70:29-33].
const ASPECT: f32 = f32::from_bits(0x3faa_aaab);
/// `MAX_VIEW_CONE` 0x30d0fc, 2000.0 [L symtab + ee_snapshot].
const MAX_VIEW_CONE: f32 = 2000.0;

/// The projection scales `Viewer+0x100` (x) and `+0x104` (y) as `Camera_CalcViewAngles` 0x1baf70 sets
/// them [M :43-47]: k = −((MVC + 0.2) / (0.2 − MVC)); y = k·H·0.5·(1/tan(a/2)); x = (k·W·0.5·(1/tan(a/2)))
/// / aspect. The `tanf` is the host's (the libm `tanf` chain is not ported) [S]. W/(2x) = 0.76966 and
/// H/(2y) = 0.57724, the half-screen tangents the port has used since [L].
pub fn focal() -> (f32, f32) {
    let it = div(1.0, (VIEW_ANGLE * 0.5).tan());
    let k = -div(trig::ee_add(MAX_VIEW_CONE, 0.2), trig::ee_sub(0.2, MAX_VIEW_CONE));
    let fy = mul(mul(mul(k, SCREEN_H), 0.5), it);
    let fx = div(mul(mul(mul(k, SCREEN_W), 0.5), it), ASPECT);
    (fx, fy)
}

/// Bond's view frame as the rows the port's camera uses: R0 = screen-left (x), R1 = up (y), R2 = the
/// view direction (`look`). The game builds the same frame twice, the viewer matrix the projection
/// reads (`Viewer+0x160..0x180`) and `obj+0x90 · RotMatrixX(−pitch)` in `Player_GetAimingPoint`, and the
/// auto-aim only needs the two to agree. The port takes both from Bond's yaw and view pitch (radians).
/// *Unproven*: the game's matrices' last-bit values (the RotMatrix lanes) [S].
pub fn view_basis(yaw: f32, pitch: f32, look: V3) -> [V3; 3] {
    let (sy, cy) = (trig::sinf(yaw), trig::cosf(yaw));
    let (sp, cp) = (trig::sinf(pitch), trig::cosf(pitch));
    [[cy, 0.0, -sy], [-mul(sp, sy), cp, -mul(sp, cy)], look]
}

fn dot(r: V3, d: V3) -> f32 {
    trig::ee_add(trig::ee_add(mul(r[0], d[0]), mul(r[1], d[1])), mul(r[2], d[2]))
}

/// `View_3DPoint2Screen` 0x1e7708 followed by `Check_AutoAim`'s store [M disasm 0x1e7708-0x1e77ec,
/// 0x1a7784-0x1a77d4]: v = the view frame · (p − eye); `None` (the −1 return) when v.z < 0; then
/// sx = W − (v.x·(fx/z) + W·0.5), sy = v.y·(fy/z) + H·0.5 and the stored offset is
/// ((sx − W·0.5)/(W·0.25), (sy − H·0.5)/(H·0.25)), **twice** the target's screen position: the
/// end of `Player_AutoAim` halves it before the HUD and the shot read it.
pub fn project_offset(eye: V3, basis: &[V3; 3], p: V3) -> Option<[f32; 2]> {
    let d = [trig::ee_sub(p[0], eye[0]), trig::ee_sub(p[1], eye[1]), trig::ee_sub(p[2], eye[2])];
    let (vx, vy, vz) = (dot(basis[0], d), dot(basis[1], d), dot(basis[2], d));
    if vz < 0.0 { return None; }
    let (fx, fy) = focal();
    let sx = trig::ee_sub(SCREEN_W, trig::ee_add(mul(vx, div(fx, vz)), mul(SCREEN_W, 0.5)));
    let sy = trig::ee_add(mul(vy, div(fy, vz)), mul(SCREEN_H, 0.5));
    Some([
        div(trig::ee_sub(sx, mul(SCREEN_W, 0.5)), mul(SCREEN_W, 0.25)),
        div(trig::ee_sub(sy, mul(SCREEN_H, 0.5)), mul(SCREEN_H, 0.25)),
    ])
}

/// `Player_GetAimingPoint` 0x1a4a98 [M :47-72]: with `BL+0x950` clear, bx = ((W·s.x)/zoom)/(fx + fx),
/// by = −((H·s.y)/zoom)/(fy + fy) (`zoom` = `Viewer+0x108`); the aim point is head + (−256)·(bx·R0 +
/// by·R1 − R2). Returned as the unit direction from the head (the bullet normalises head → point).
pub fn aim_dir(basis: &[V3; 3], s: [f32; 2], zoom: f32) -> V3 {
    let (fx, fy) = focal();
    let bx = div(div(mul(SCREEN_W, s[0]), zoom), trig::ee_add(fx, fx));
    let by = -div(div(mul(SCREEN_H, s[1]), zoom), trig::ee_add(fy, fy));
    let v = [0, 1, 2].map(|k| trig::ee_add(trig::ee_add(mul(bx, basis[0][k]), mul(by, basis[1][k])), -basis[2][k]));
    let p = v.map(|c| mul(c, -256.0));
    let l = (p[0] * p[0] + p[1] * p[1] + p[2] * p[2]).sqrt();
    if l == 0.0 { return basis[2]; }
    p.map(|c| c / l)
}

/// `Vec_Dist3D` 0x1e2e48 [M disasm]: VU0 `vsub` + `vmul` (chopped), `add.s` x+y then +z, `sqrt.s`.
pub fn dist3d(a: V3, b: V3) -> f32 {
    let d = [trig::ee_sub(b[0], a[0]), trig::ee_sub(b[1], a[1]), trig::ee_sub(b[2], a[2])];
    nf_formats::ps2float::sqrt(trig::ee_add(trig::ee_add(mul(d[0], d[0]), mul(d[1], d[1])), mul(d[2], d[2])))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The projection scales reproduce the half-screen tangents the port's free aim has used [L].
    #[test]
    fn the_focal_scales_are_the_half_screen_tangents() {
        let (fx, fy) = focal();
        assert!((SCREEN_W / (2.0 * fx) - 0.76966).abs() < 2e-5, "{}", SCREEN_W / (2.0 * fx));
        assert!((SCREEN_H / (2.0 * fy) - 0.57724).abs() < 2e-5, "{}", SCREEN_H / (2.0 * fy));
    }

    /// A fresh lock: the halved projected offset, fed back through `Player_GetAimingPoint`, aims at the
    /// target itself (the game's Check_AutoAim → halve → GetAimingPoint chain).
    #[test]
    fn the_halved_offset_aims_at_the_target() {
        let (yaw, pitch) = (0.7f32, 0.3f32);
        let look = [pitch.cos() * yaw.sin(), pitch.sin(), pitch.cos() * yaw.cos()];
        let b = view_basis(yaw, pitch, look);
        let eye = [1.0, 2.0, 3.0];
        let tgt = [eye[0] + 8.0 * (yaw + 0.1).sin(), eye[1] + 3.1, eye[2] + 8.0 * (yaw + 0.1).cos()];
        let s = project_offset(eye, &b, tgt).unwrap();
        let dir = aim_dir(&b, [s[0] * 0.5, s[1] * 0.5], 1.0);
        let to = [tgt[0] - eye[0], tgt[1] - eye[1], tgt[2] - eye[2]];
        let l = (to[0] * to[0] + to[1] * to[1] + to[2] * to[2]).sqrt();
        for k in 0..3 { assert!((dir[k] - to[k] / l).abs() < 1e-4, "{dir:?} vs {:?}", to.map(|c| c / l)); }
        // A target to Bond's right (yaw +0.1 turns left in the object frame) sits left on screen.
        assert!(s[0] < 0.0, "{s:?}");
        // Behind the eye: no screen position.
        assert!(project_offset(eye, &b, [eye[0] - look[0], eye[1] - look[1], eye[2] - look[2]]).is_none());
    }
}
