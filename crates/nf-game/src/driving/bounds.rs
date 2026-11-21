//! The invisible boundary sheets of the driving missions (DRIVING.ELF table 0x331a60) —
//! docs/systems/driving-world-bounds.md. 19 convex-quad sheets (five horizontal, normal
//! (0, -1, 0); fourteen vertical) tested in two places [M]:
//! - `FUN_00190f88` 0x192130-0x1923bc, the per-vehicle pass the controller `FUN_00188200` runs
//!   ONLY when the pvehicle attr `SUB_PHYSICS` (+0xc0) is non-zero [M 0x188bf8-0x188c04]. The
//!   shipped .atr files that set it are the subs (alpha_sub / mini_sub / vanquishsub, MIS11),
//!   the ultralights (MIS13A/B/C), plane and subcopter (MIS4) [D]. The land cars (vanquish has
//!   no SUB_PHYSICS line [D]) never reach this pass, so `physics::Car` does not run it.
//! - `FUN_00190eb8`, a bool "is this point in any sheet's band" helper; its one caller
//!   (`FUN_00182978` :126-138) tests a point 10 below an actor in state 0x16.
//!
//! Every sheet that passes the test responds in the same tick (no break in the loop [M
//! 0x1923b4-0x1923b8]). Which map each sheet belongs to is unproven.

/// One sheet: a convex quad (four corner points, w = 1.0 on the disc) in its own plane, the plane
/// normal pointing into the allowed side, and the response band's thickness.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ZoneQuad {
    pub corners: [[f32; 4]; 4],
    pub normal: [f32; 3],
    pub thickness: f32,
}

impl ZoneQuad {
    const fn new(corners: [[f32; 4]; 4], normal: [f32; 3], thickness: f32) -> Self {
        ZoneQuad { corners, normal, thickness }
    }

    /// The plane stage of `sub_190EB8` 0x190ee8-0x190f28 / `sub_190F88` 0x192150-0x192184 [M]:
    /// `d = (p − corners[3]) · normal` (corner 3 is the test's reference point, the +0x30 load).
    /// The band is `0 <= d < thickness` (0x192184's two compares).
    pub fn signed_dist(&self, p: [f32; 4]) -> f32 {
        let c = self.corners[3];
        (p[0] - c[0]) * self.normal[0] + (p[1] - c[1]) * self.normal[1] + (p[2] - c[2]) * self.normal[2]
    }

    /// `sub_190E08` (0x190e08-0x190eb0) [M]: for each edge (corners wrap `(i+1)&3`) the
    /// cross-product of the edge and the point-to-corner vector is dotted with the plane normal;
    /// any negative dot rejects the point (the `v9 = v0 < 0.0` break at 0x190e94); four passes
    /// are "inside".
    pub fn inside_quad(&self, p: [f32; 4]) -> bool {
        let n = self.normal;
        for i in 0..4 {
            let a = self.corners[i];
            let b = self.corners[(i + 1) & 3];
            let e = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
            let t = [p[0] - a[0], p[1] - a[1], p[2] - a[2]];
            let cross = [e[1] * t[2] - e[2] * t[1], e[2] * t[0] - e[0] * t[2], e[0] * t[1] - e[1] * t[0]];
            if cross[0] * n[0] + cross[1] * n[1] + cross[2] * n[2] < 0.0 {
                return false;
            }
        }
        true
    }

    /// The full sheet test [M 0x192168-0x192194]: the band, then the quad.
    pub fn hit(&self, p: [f32; 4]) -> bool {
        let d = self.signed_dist(p);
        d >= 0.0 && d < self.thickness && self.inside_quad(p)
    }

    /// `FUN_00190eb8`'s answer [M 0x190eb8]: the first sheet (table order) whose band holds
    /// `p`, or None. The game returns only a bool; the index is the port's convenience.
    pub fn first_hit(p: [f32; 4]) -> Option<usize> {
        ZONES.iter().position(|z| z.hit(p))
    }

    /// Every sheet that holds `p`, in table order. The `FUN_00190f88` loop applies all of
    /// them; it does not stop at the first [M 0x1923b4: the hit path falls through to the
    /// same `addiu s4,-1` / `bgez` as a miss].
    pub fn hits(p: [f32; 4]) -> impl Iterator<Item = usize> {
        (0..ZONES.len()).filter(move |&i| ZONES[i].hit(p))
    }

    /// The response constants of `FUN_00190f88` [M, each loaded as an immediate]:
    /// - `MIN_SCALE = 0.1` (`$f25`, lui 0x3dcc / ori 0xcccd at 0x1920fc-0x192104)
    /// - `POS_VN_MIN = 1.0` (`$f23 = $f21`, delay slot 0x1920cc; `$f21 = 1.0` at 0x192000)
    /// - `NEG_VN_MAX = -1.0` (`$f24`, lui 0xbf80 at 0x19210c)
    /// - `SCRAPE_SPEED_MIN = 10.0` (`$f21`, lui 0x4120 at 0x192118)
    pub const MIN_SCALE: f32 = 0.1;
    pub const POS_VN_MIN: f32 = 1.0;
    pub const NEG_VN_MAX: f32 = -1.0;
    pub const SCRAPE_SPEED_MIN: f32 = 10.0;

    /// The band depth weight [M 0x1921d8-0x1921ec]: `max((thickness - d) / thickness, 0.1)`:
    /// 1.0 on the sheet's plane, falling to the 0.1 floor at the far edge of the band.
    pub fn penetration_scale(&self, d: f32) -> f32 {
        ((self.thickness - d) / self.thickness).max(Self::MIN_SCALE)
    }

    /// The approach-speed clamp [M 0x1921f0-0x19221c]: negative values go to at most -1.0,
    /// positive values to at least 1.0, and exactly 0.0 stays 0.0 (neither `c.olt.s` branch
    /// is taken).
    pub fn clamped_approach_speed(v_n: f32) -> f32 {
        if v_n < 0.0 {
            v_n.min(Self::NEG_VN_MAX)
        } else if v_n > 0.0 {
            v_n.max(Self::POS_VN_MIN)
        } else {
            v_n
        }
    }

    /// The sheet's push [M 0x192220-0x1922c4], with `v_n` the clamped approach speed
    /// `-(vel.n)`, `force_acc` the body's force accumulator (inst+0x60), `mass` inst+0x4c,
    /// `fps` the integer frame rate at sim+0xdbc (0x339f4c, `cvt.s.w` at 0x192224: an
    /// integer, not a time step) and `scale` = `penetration_scale(d)`:
    /// `C = n*(fps*v_n*scale) + n*(-(f.n)/mass*scale)`; when `C.n > 0` (0x192298) the game
    /// multiplies C by `scale` AGAIN (0x1922b8) and adds it through `FUN_001f87d8`
    /// (`acc += C*mass`). Returns that applied vector (an acceleration), or None when the
    /// gate fails.
    pub fn boundary_force(&self, v_n: f32, force_acc: [f32; 4], mass: f32, fps: f32, scale: f32) -> Option<[f32; 3]> {
        let n = self.normal;
        let f_n = (force_acc[0] * n[0] + force_acc[1] * n[1] + force_acc[2] * n[2]) / mass;
        let a = fps * v_n * scale;
        let b = -f_n * scale;
        let c = [n[0] * a + n[0] * b, n[1] * a + n[1] * b, n[2] * a + n[2] * b];
        if c[0] * n[0] + c[1] * n[1] + c[2] * n[2] > 0.0 {
            Some([c[0] * scale, c[1] * scale, c[2] * scale])
        } else {
            None
        }
    }

    /// The scrape torque that follows an applied push [M 0x1922cc-0x192398], in the body's
    /// local frame before `FUN_001f84d0` transforms it and `FUN_001f8888` adds it (times mass)
    /// to the torque accumulator: with `row2_n` = (orientation row 2, *(inst+0x5c) + 0x20).n,
    /// `row0_n` = (row 0).n and `speed` the vehicle's +0x288:
    /// `m = max(speed, 10)*|row2_n|`, `x = 2*m*|n.y|` (negated when `row2_n > 0`),
    /// `y = 1.5*m*sqrt(n.x^2 + n.z^2)` (negated when `row0_n < 0`), `z = 0`.
    pub fn scrape_torque_local(&self, speed: f32, row2_n: f32, row0_n: f32) -> [f32; 3] {
        let n = self.normal;
        let m = speed.max(Self::SCRAPE_SPEED_MIN) * row2_n.abs();
        let mut x = m * n[1].abs();
        x += x;
        if row2_n > 0.0 {
            x = -x;
        }
        let mut y = m * (n[0] * n[0] + n[2] * n[2]).sqrt() * 1.5;
        if row0_n < 0.0 {
            y = -y;
        }
        [x, y, 0.0]
    }
}

/// The whole `FUN_00190f88` sheet loop for one body [M 0x192130-0x1923bc]: the summed applied
/// pushes (accelerations) of every sheet holding `pos`. The gate in front of the loop
/// (`+0x2c8 != 2 || FUN_00195a30() > 0`, 0x1920c4-0x1920e8) is the caller's; its meaning is
/// unproven.
pub fn sheet_pass(pos: [f32; 3], vel: [f32; 3], force_acc: [f32; 4], mass: f32, fps: f32) -> [f32; 3] {
    let p = [pos[0], pos[1], pos[2], 1.0];
    let mut sum = [0.0f32; 3];
    for i in ZoneQuad::hits(p) {
        let z = &ZONES[i];
        let n = z.normal;
        let v_n = ZoneQuad::clamped_approach_speed(-(vel[0] * n[0] + vel[1] * n[1] + vel[2] * n[2]));
        let scale = z.penetration_scale(z.signed_dist(p));
        if let Some(f) = z.boundary_force(v_n, force_acc, mass, fps, scale) {
            sum[0] += f[0];
            sum[1] += f[1];
            sum[2] += f[2];
        }
    }
    sum
}

/// The static sheet table at 0x331a60, read from the DRIVING.ELF bytes (sha1 d7b8a57f...) [M].
/// Record (0x50) = 4 corners (+0x00..+0x3f, w = 1.0), normal xyz (+0x40) and thickness (+0x4c,
/// the normal's w lane; the dot products use .xyz only [M dest mask 0xe at 0x192150]). No
/// store to this address was found in a scan of the code [M-negative], so these read as the
/// runtime values; a live read would settle it. The tiny normal-y values are the stored bytes.
pub static ZONES: [ZoneQuad; 19] = [
    // 0-4: horizontal sheets, normal (0, -1, 0). Which mission each serves is unproven.
    ZoneQuad::new(
        [[257.605988, 78.4380035, 716.487976, 1.0], [-258.537994, 78.4380035, -131.720001, 1.0],
         [448.470001, 78.4380035, -411.895996, 1.0], [635.250977, 78.4380035, 616.252014, 1.0]],
        [0.0, -1.0, 0.0], 10.0,
    ),
    ZoneQuad::new(
        [[1629.27002, -240.477005, 1208.552, 1.0], [1710.60498, -240.477005, 1404.91199, 1.0],
         [1534.78296, -240.477005, 1477.73999, 1.0], [1453.448, -240.477005, 1281.38, 1.0]],
        [0.0, -1.0, 0.0], 15.0,
    ),
    ZoneQuad::new(
        [[2640.10889, -240.481995, 524.356995, 1.0], [2815.93188, -240.481995, 451.528992, 1.0],
         [2897.26709, -240.481995, 647.888977, 1.0], [2721.44507, -240.481995, 720.716003, 1.0]],
        [0.0, -1.0, 0.0], 15.0,
    ),
    ZoneQuad::new(
        [[3174.43701, -260.329987, 252.337997, 1.0], [2972.24097, -260.329987, 336.091003, 1.0],
         [2878.70605, -260.329987, 110.276001, 1.0], [3080.9021, -260.329987, 26.5240002, 1.0]],
        [0.0, -1.0, 0.0], 15.0,
    ),
    ZoneQuad::new(
        [[2998.1499, -95.4599991, 223.240997, 1.0], [2363.39893, -95.4599991, -411.509003, 1.0],
         [3131.60107, -95.4599991, -1179.71204, 1.0], [3766.35205, -95.4599991, -544.960999, 1.0]],
        [0.0, -1.0, 0.0], 15.0,
    ),
    // 5-18: vertical sheets (normals in xz).
    ZoneQuad::new(
        [[453.53299, 216.009003, 419.897003, 1.0], [453.53299, -63.9910011, 419.897003, 1.0],
         [163.533005, -63.9910011, 419.897003, 1.0], [163.533005, 216.009003, 419.897003, 1.0]],
        [0.0, 0.0, -1.0], 10.0,
    ),
    ZoneQuad::new(
        [[-38.5670013, 216.009003, 149.714005, 1.0], [-38.5670013, -63.9910011, 149.714005, 1.0],
         [-38.5670013, -63.9910011, -140.285995, 1.0], [-38.5670013, 216.009003, -140.285995, 1.0]],
        [1.0, 0.0, 0.0], 10.0,
    ),
    ZoneQuad::new(
        [[-45.4184914, -63.9913902, -21.901886, 1.0], [152.28096, -63.991394, -10.9013224, 1.0],
         [152.28096, 216.008606, -10.9013205, 1.0], [-45.4184952, 216.008606, -21.9018841, 1.0]],
        [-0.055556901, -5.22289234e-9, 0.99845552], 11.0,
    ),
    ZoneQuad::new(
        [[125.305229, -63.991375, -18.9760132, 1.0], [291.624054, -63.9913788, 53.9586983, 1.0],
         [291.624023, 216.008621, 53.9586983, 1.0], [125.305222, 216.008621, -18.9760036, 1.0]],
        [-0.40263954, -1.07501279e-7, 0.9153586], 11.0,
    ),
    ZoneQuad::new(
        [[316.073944, -63.991394, 58.4870415, 1.0], [376.60083, -63.991394, 178.492798, 1.0],
         [376.60083, 216.008606, 178.492798, 1.0], [316.073944, 216.008606, 58.4870377, 1.0]],
        [-0.89286232, 1.08083826e-8, 0.4503296], 11.0,
    ),
    ZoneQuad::new(
        [[282.489136, -63.3417816, 51.0230713, 1.0], [327.178345, -63.3417854, 63.1325378, 1.0],
         [327.178345, 216.658218, 63.1325417, 1.0], [282.489136, 216.658218, 51.0230713, 1.0]],
        [-0.26153901, 0.0, 0.96519202], 11.0,
    ),
    ZoneQuad::new(
        [[370.372803, -63.9913979, 160.985809, 1.0], [412.110809, -63.9913902, 361.762695, 1.0],
         [412.110809, 216.008606, 361.762695, 1.0], [370.372803, 216.008606, 160.985794, 1.0]],
        [-0.97906846, 1.10915721e-8, 0.20353122], 11.0,
    ),
    ZoneQuad::new(
        [[413.643951, -63.9913902, 384.294617, 1.0], [412.285492, -63.9913902, 426.325745, 1.0],
         [412.285492, 216.008606, 426.325714, 1.0], [413.643951, 216.008606, 384.294617, 1.0]],
        [-0.99947810, -1.67055756e-8, -0.032303698], 11.0,
    ),
    ZoneQuad::new(
        [[413.839386, 215.245651, 388.84433, 1.0], [411.22525, 215.245667, 355.171967, 1.0],
         [411.225281, 20.9615822, 355.171967, 1.0], [413.839386, 20.9615822, 388.84433, 1.0]],
        [-0.99700004, -1.23653354e-8, 0.077400520], 11.0,
    ),
    ZoneQuad::new(
        [[41.8399124, -63.9913788, 119.425362, 1.0], [-51.0078773, -63.9913788, 21.1331139, 1.0],
         [-51.007885, 216.008591, 21.1331081, 1.0], [41.8399086, 216.008591, 119.425362, 1.0]],
        [0.72695339, -4.37834373e-8, -0.68668687], 11.0,
    ),
    ZoneQuad::new(
        [[2.12304497, -63.991375, 159.484451, 1.0], [41.9724503, -63.991375, 112.218529, 1.0],
         [41.9724503, 216.008591, 112.218506, 1.0], [2.12303257, 216.008591, 159.484436, 1.0]],
        [0.76454020, 9.34096818e-8, 0.64457601], 11.0,
    ),
    ZoneQuad::new(
        [[62.7305946, -63.991375, 219.412811, 1.0], [2.48751187, -63.991375, 152.727997, 1.0],
         [2.48751426, 216.008606, 152.727966, 1.0], [62.7305946, 216.008591, 219.412811, 1.0]],
        [0.74203825, -4.40887531e-16, -0.67035758], 11.0,
    ),
    ZoneQuad::new(
        [[117.162582, -63.9913902, 197.587463, 1.0], [55.4095268, -63.9913902, 217.432907, 1.0],
         [55.4095154, 216.008606, 217.432907, 1.0], [117.162575, 216.008591, 197.587463, 1.0]],
        [-0.30595696, 3.27847949e-9, -0.95204538], 11.0,
    ),
    ZoneQuad::new(
        [[225.637848, -63.9913635, 270.589203, 1.0], [107.03611, -63.9913635, 199.135712, 1.0],
         [107.036102, 216.008606, 199.135712, 1.0], [225.637848, 216.008575, 270.589203, 1.0]],
        [0.51604784, -2.7619059e-8, -0.85655975], 11.0,
    ),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_table_is_19_sheets_with_the_measured_landmarks() {
        assert_eq!(ZONES.len(), 19);
        // rec0: the 78.438 ceiling slab; rec5: the z=419.897 wall; rec7: the first slanted ring wall.
        let (c0, w5, w7) = (&ZONES[0], &ZONES[5], &ZONES[7]);
        assert_eq!(c0.normal, [0.0, -1.0, 0.0]);
        assert_eq!(c0.thickness, 10.0);
        assert!((c0.corners[0][1] - 78.4380035).abs() < 1e-4);
        assert_eq!(w5.normal, [0.0, 0.0, -1.0]);
        assert!((w5.corners[0][2] - 419.897003).abs() < 1e-4);
        assert!((w7.normal[0] - -0.055556901).abs() < 1e-7);
        // Sister: the field comparisons bite (the ceilings' 10 differs from the walls' 15/11 set).
        assert_ne!(c0.thickness, ZONES[1].thickness);
        assert_ne!(w5.normal, w7.normal);
    }

    #[test]
    fn the_plane_band_is_distance_from_corner3_along_the_normal() {
        let c = &ZONES[0]; // corners at y = 78.438, normal (0,−1,0), thickness 10
        let inside = [300.0, 74.0, 500.0, 1.0]; // 4.438 below the sheet
        let above = [300.0, 90.0, 500.0, 1.0]; // past the allowed side
        let deep = [300.0, 60.0, 500.0, 1.0]; // outside the band (too far below)
        assert!((c.signed_dist(inside) - 4.4380035).abs() < 1e-3);
        assert!(c.signed_dist(above) < 0.0 && !c.hit(above));
        assert!(c.signed_dist(deep) >= c.thickness && !c.hit(deep));
        assert!(c.hit(inside));
        // Sister: flipping the normal flips the distance — the compare is not vacuous.
        let flipped = ZoneQuad { normal: [0.0, 1.0, 0.0], ..*c };
        assert!((flipped.signed_dist(inside) + c.signed_dist(inside)).abs() < 1e-5);
    }

    #[test]
    fn inside_quad_wraps_the_four_edges_and_rejects_outside_points() {
        let w = &ZONES[5]; // the vertical wall: x 163.533..453.533, y −63.991..216.009, z = 419.897
        let inside = [300.0, 100.0, 415.0, 1.0];
        assert!(w.inside_quad(inside));
        assert!(w.hit(inside)); // 4.897 off the wall, within the 10 band
        for p in [[500.0, 100.0, 415.0, 1.0], [300.0, -200.0, 415.0, 1.0], [100.0, 100.0, 415.0, 1.0]] {
            assert!(!w.inside_quad(p), "outside the footprint: {p:?}");
        }
        // Sister: the wrap edge (corner 3 → corner 0, the quad's top edge at y ≈ 216.009) is the
        // leg an `i in 0..3` loop would skip — a point above it must still reject.
        assert!(!w.inside_quad([300.0, 300.0, 415.0, 1.0]));
        // And just past corner 0's edge (x < 163.533) is outside the band's footprint.
        assert!(!w.hit([163.0, 100.0, 415.0, 1.0]));
    }

    #[test]
    fn first_hit_scans_in_table_order_and_reports_none_off_map() {
        // rec5's band point: the scan passes 0-4 (wrong altitudes) and lands on 5.
        let p = [300.0, 100.0, 415.0, 1.0];
        assert_eq!(ZoneQuad::first_hit(p), Some(5));
        // A rec0 band point (above the y=78.438 sheet's footprint, below its plane).
        let q = [300.0, 74.0, 500.0, 1.0];
        assert_eq!(ZoneQuad::first_hit(q), Some(0));
        // Sister: off every sheet (a point on no map) — proves the scan can return None.
        assert_eq!(ZoneQuad::first_hit([0.0, 500.0, 0.0, 1.0]), None);
        assert_ne!(ZoneQuad::first_hit(p), ZoneQuad::first_hit(q));
    }

    #[test]
    fn boundary_force_uses_fps_scales_twice_and_gates_on_the_normal() {
        let w = &ZONES[5]; // normal (0, 0, -1)
        // [M 0x192220-0x1922b8]: C = n*(fps*v_n*s) + n*(-(f.n)/m*s); applied = C*s when C.n > 0.
        // fps = 60 (sim+0xdbc is an int: cvt.s.w at 0x192224), v_n = 5, s = 0.5, no accumulator:
        // applied z = -(60*5*0.5)*0.5 = -75. The PR form (dt*v_n, scale once) gave -1/24.
        let f = w.boundary_force(5.0, [0.0; 4], 1660.0, 60.0, 0.5).expect("approaching: gate passes");
        assert_eq!(f, [0.0, 0.0, -75.0]);
        // An accumulated force into the wall (acc.n = -830) adds 830/1660*0.5*0.5 = 0.125.
        let f2 = w.boundary_force(5.0, [0.0, 0.0, 830.0, 0.0], 1660.0, 60.0, 0.5).unwrap();
        assert!((f2[2] - -(75.0 + 0.125)).abs() < 1e-4, "{f2:?}");
        // The gate [M 0x192298 c.olt.s 0, C.n]: a receding body (v_n = -1 after the clamp) gets
        // nothing — the PR returned a vector pointing into the wall here.
        assert_eq!(w.boundary_force(-1.0, [0.0; 4], 1660.0, 60.0, 0.5), None);
        // And an outward accumulator large enough to flip C also fails the gate.
        assert_eq!(w.boundary_force(1.0, [0.0, 0.0, -1660.0 * 61.0, 0.0], 1660.0, 60.0, 1.0), None);
    }

    #[test]
    fn penetration_scale_and_the_approach_clamp_match_the_immediates() {
        let w = &ZONES[5]; // thickness 10
        // [M 0x1921e0-0x1921ec]: max((t - d)/t, 0.1).
        assert_eq!(w.penetration_scale(0.0), 1.0);
        assert!((w.penetration_scale(4.0) - 0.6).abs() < 1e-6);
        assert!((w.penetration_scale(9.5) - 0.1).abs() < 1e-6, "floored at 0.1, not 0.05");
        // [M 0x1921f0-0x19221c]: <0 -> min(-1, v); >0 -> max(1, v); 0 stays 0.
        assert_eq!(ZoneQuad::clamped_approach_speed(-0.2), -1.0);
        assert_eq!(ZoneQuad::clamped_approach_speed(-3.0), -3.0);
        assert_eq!(ZoneQuad::clamped_approach_speed(0.2), 1.0);
        assert_eq!(ZoneQuad::clamped_approach_speed(3.0), 3.0);
        assert_eq!(ZoneQuad::clamped_approach_speed(0.0), 0.0, "zero is not pushed up to 1.0");
    }

    #[test]
    fn every_sheet_holding_the_point_responds_not_just_the_first() {
        // [M 0x1923b4]: the loop has no break. Find a point two sheets hold (the slanted ring
        // walls 7 and 8 meet near x = 125..152); fall back to proving the sum over a single hit.
        let mut both = None;
        'scan: for xi in 0..400 {
            for zi in 0..120 {
                let p = [100.0 + xi as f32 * 0.25, 50.0, -30.0 + zi as f32 * 0.5, 1.0];
                if ZoneQuad::hits(p).count() >= 2 {
                    both = Some(p);
                    break 'scan;
                }
            }
        }
        let p = both.expect("walls 7 and 8 overlap near their shared corner");
        let ids: Vec<usize> = ZoneQuad::hits(p).collect();
        assert_eq!(ZoneQuad::first_hit(p), Some(ids[0]));
        // Velocity into both walls: the pass sums both pushes, so it differs from the first alone.
        let n0 = ZONES[ids[0]].normal;
        let n1 = ZONES[ids[1]].normal;
        let vel = [-(n0[0] + n1[0]) * 3.0, 0.0, -(n0[2] + n1[2]) * 3.0];
        let sum = sheet_pass([p[0], p[1], p[2]], vel, [0.0; 4], 1660.0, 60.0);
        let z = &ZONES[ids[0]];
        let v_n = ZoneQuad::clamped_approach_speed(-(vel[0] * n0[0] + vel[1] * n0[1] + vel[2] * n0[2]));
        let first = z.boundary_force(v_n, [0.0; 4], 1660.0, 60.0, z.penetration_scale(z.signed_dist(p))).unwrap();
        assert!((sum[0] - first[0]).abs() > 1e-3 || (sum[2] - first[2]).abs() > 1e-3, "{sum:?} vs {first:?}");
    }

    #[test]
    fn scrape_torque_follows_the_0x1922cc_block() {
        let w = &ZONES[5]; // n = (0, 0, -1): |n.y| = 0, sqrt(nx^2+nz^2) = 1
        // speed 4 floors at 10 [M max.s with $f21 = 10.0 at 0x1922f8]; row2.n = 0.5 -> m = 5.
        assert_eq!(w.scrape_torque_local(4.0, 0.5, 1.0), [0.0, 7.5, 0.0]);
        assert_eq!(w.scrape_torque_local(20.0, 0.5, -1.0), [0.0, -15.0, 0.0], "row0.n < 0 flips y");
        let c = &ZONES[0]; // n = (0, -1, 0): x = 2*m, flipped when row2.n > 0
        assert_eq!(c.scrape_torque_local(10.0, 0.5, 1.0), [-10.0, 0.0, 0.0]);
        assert_eq!(c.scrape_torque_local(10.0, -0.5, 1.0), [10.0, 0.0, 0.0]);
    }

    #[test]
    fn the_table_bytes_carry_the_stored_normal_noise_and_thickness_lanes() {
        // [M DRIVING.ELF bytes at 0x331a60 + 0x50*i + 0x44 / + 0x4c].
        assert_eq!(ZONES[8].normal[1], -1.07501279e-7);
        assert_eq!(ZONES[15].normal[1], 9.34096818e-8);
        assert_eq!(ZONES[10].normal[1], 0.0);
        let t: Vec<f32> = ZONES.iter().map(|z| z.thickness).collect();
        assert_eq!(t, [10.0, 15.0, 15.0, 15.0, 15.0, 10.0, 10.0, 11.0, 11.0, 11.0, 11.0, 11.0, 11.0, 11.0, 11.0, 11.0, 11.0, 11.0, 11.0]);
    }
}
