//! The driving maps' invisible boundary sheets (DRIVING.ELF `unk_331A60` = 0x331a60) —
//! docs/systems/driving-world-bounds.md. Decoded from the IDA database of `work/elfs/DRIVING.ELF`
//! (sha1 `d7b8a57f…`): 19 convex-quad sheets — five horizontal ceilings at the per-map altitudes
//! and fourteen walls around the drivable ring — tested per car in the tyre pass `sub_190F88`
//! @0x19210c-0x1923b4 and standalone in `sub_190EB8` (0x190eb8), with the penalty-force +
//! scrape-torque response at 0x19219c-0x1923a8. This corrects the drive-physics.md §4
//! "spring-damper colliders" reading (the :870-975 region IS these sheets).
//!
//! Table values are the IDB bytes [M]; the response's clamp/scale scalars are [S] (the doc's §3)
//! and arrive as arguments here so a live trace can pin them without reshaping.

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

    /// The full sheet test [M §2 of the doc]: the band and the quad.
    pub fn hit(&self, p: [f32; 4]) -> bool {
        let d = self.signed_dist(p);
        d >= 0.0 && d < self.thickness && self.inside_quad(p)
    }

    /// The tyre pass's scan: all 19 sheets in table order, the first hit wins [M 0x192130's
    /// counter-18..0 walk and `LABEL_134`'s decrement at 0x1923b4].
    pub fn first_hit(p: [f32; 4]) -> Option<usize> {
        ZONES.iter().position(|z| z.hit(p))
    }

    /// `sub_190F88` §3 clamp constants [M 0x1920ec..0x1922f8]:
    /// - `MIN_SCALE = 0.1` (`$f25 = 0x3DCCCCCD` at 0x1920fc)
    /// - `POS_VN_MIN = 1.0`, `NEG_VN_MAX = -1.0` (`$f23 = 1.0`, `$f24 = -1.0` at 0x1920ec/0x19210c)
    /// - `SCRAPE_SPEED_MIN = 10.0` (`$f21 = 10.0` at 0x192118)
    pub const MIN_SCALE: f32 = 0.1;
    pub const POS_VN_MIN: f32 = 1.0;
    pub const NEG_VN_MAX: f32 = -1.0;
    pub const SCRAPE_SPEED_MIN: f32 = 10.0;

    /// Clamps approach speed `v_n` per `sub_190F88` [M 0x1921f0..0x19221c].
    pub fn clamped_approach_speed(v_n: f32) -> f32 {
        if v_n < 0.0 {
            v_n.min(Self::NEG_VN_MAX)
        } else {
            v_n.max(Self::POS_VN_MIN)
        }
    }

    /// §3's linear spring [M structure & clamps 0x1920e0..0x192254]:
    /// `v_n` arrives as the (clamped) approach speed]: `F = n · scale · (dt·v_n − (f·n)/mass)`
    /// — the dt-scaled push along the normal (dt = `flt_339F4C`, the frame-delta global) with
    /// the accumulated inward acceleration removed (0x192248's `−dot(inst+0x60, n)/mass` term:
    /// an inward accumulation (f·n < 0) strengthens the push, an outward one weakens it).
    /// The caller applies it through the physics force API when it still points along +n
    /// (0x1922a0's gate — `k > 0`).
    pub fn boundary_force(&self, v_n: f32, force_acc: [f32; 4], mass: f32, dt: f32, scale: f32) -> [f32; 3] {
        let n = self.normal;
        let f_n = (force_acc[0] * n[0] + force_acc[1] * n[1] + force_acc[2] * n[2]) / mass;
        let k = scale * (dt * v_n - f_n);
        [n[0] * k, n[1] * k, n[2] * k]
    }
}

/// The static sheet table at 0x331a60 — the exact IDB values [M]. No writer exists in the ELF
/// (the only two data references are the two readers [M-negative xref scan]), so these are the
/// runtime values. The walls' normal-y is float noise (±1e-8 in the bytes) written as 0.0 —
/// docs/systems/driving-world-bounds.md §1.
pub static ZONES: [ZoneQuad; 19] = [
    // 0-4: the horizontal ceilings — per-map altitudes (the five missions' maps sit at different
    // world origins [S]; disjoint heights, one static table).
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
    // 5-18: the drivable ring's walls (normals in xz; rec10 sits 0.65 higher — its own y).
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
        [-0.055556901, 0.0, 0.99845552], 11.0,
    ),
    ZoneQuad::new(
        [[125.305229, -63.991375, -18.9760132, 1.0], [291.624054, -63.9913788, 53.9586983, 1.0],
         [291.624023, 216.008621, 53.9586983, 1.0], [125.305222, 216.008621, -18.9760036, 1.0]],
        [-0.40263954, 0.0, 0.9153586], 11.0,
    ),
    ZoneQuad::new(
        [[316.073944, -63.991394, 58.4870415, 1.0], [376.60083, -63.991394, 178.492798, 1.0],
         [376.60083, 216.008606, 178.492798, 1.0], [316.073944, 216.008606, 58.4870377, 1.0]],
        [-0.89286232, 0.0, 0.4503296], 11.0,
    ),
    ZoneQuad::new(
        [[282.489136, -63.3417816, 51.0230713, 1.0], [327.178345, -63.3417854, 63.1325378, 1.0],
         [327.178345, 216.658218, 63.1325417, 1.0], [282.489136, 216.658218, 51.0230713, 1.0]],
        [-0.26153901, 0.0, 0.96519202], 11.0,
    ),
    ZoneQuad::new(
        [[370.372803, -63.9913979, 160.985809, 1.0], [412.110809, -63.9913902, 361.762695, 1.0],
         [412.110809, 216.008606, 361.762695, 1.0], [370.372803, 216.008606, 160.985794, 1.0]],
        [-0.97906846, 0.0, 0.20353122], 11.0,
    ),
    ZoneQuad::new(
        [[413.643951, -63.9913902, 384.294617, 1.0], [412.285492, -63.9913902, 426.325745, 1.0],
         [412.285492, 216.008606, 426.325714, 1.0], [413.643951, 216.008606, 384.294617, 1.0]],
        [-0.99947810, 0.0, -0.032303698], 11.0,
    ),
    ZoneQuad::new(
        [[413.839386, 215.245651, 388.84433, 1.0], [411.22525, 215.245667, 355.171967, 1.0],
         [411.225281, 20.9615822, 355.171967, 1.0], [413.839386, 20.9615822, 388.84433, 1.0]],
        [-0.99700004, 0.0, 0.077400520], 11.0,
    ),
    ZoneQuad::new(
        [[41.8399124, -63.9913788, 119.425362, 1.0], [-51.0078773, -63.9913788, 21.1331139, 1.0],
         [-51.007885, 216.008591, 21.1331081, 1.0], [41.8399086, 216.008591, 119.425362, 1.0]],
        [0.72695339, 0.0, -0.68668687], 11.0,
    ),
    ZoneQuad::new(
        [[2.12304497, -63.991375, 159.484451, 1.0], [41.9724503, -63.991375, 112.218529, 1.0],
         [41.9724503, 216.008591, 112.218506, 1.0], [2.12303257, 216.008591, 159.484436, 1.0]],
        [0.76454020, 0.0, 0.64457601], 11.0,
    ),
    ZoneQuad::new(
        [[62.7305946, -63.991375, 219.412811, 1.0], [2.48751187, -63.991375, 152.727997, 1.0],
         [2.48751426, 216.008606, 152.727966, 1.0], [62.7305946, 216.008591, 219.412811, 1.0]],
        [0.74203825, 0.0, -0.67035758], 11.0,
    ),
    ZoneQuad::new(
        [[117.162582, -63.9913902, 197.587463, 1.0], [55.4095268, -63.9913902, 217.432907, 1.0],
         [55.4095154, 216.008606, 217.432907, 1.0], [117.162575, 216.008591, 197.587463, 1.0]],
        [-0.30595696, 0.0, -0.95204538], 11.0,
    ),
    ZoneQuad::new(
        [[225.637848, -63.9913635, 270.589203, 1.0], [107.03611, -63.9913635, 199.135712, 1.0],
         [107.036102, 216.008606, 199.135712, 1.0], [225.637848, 216.008575, 270.589203, 1.0]],
        [0.51604784, 0.0, -0.85655975], 11.0,
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
    fn boundary_force_pushes_along_the_normal_and_drops_the_inward_accumulator() {
        let w = &ZONES[5]; // normal (0, 0, −1)
        // Approaching the wall from the allowed side: velocity toward +z, so v_n = −(v·n) > 0.
        let f = w.boundary_force(5.0, [0.0, 0.0, 0.0, 0.0], 1660.0, 1.0 / 60.0, 1.0);
        assert_eq!(f, [0.0, 0.0, -(5.0 / 60.0)]); // the push is along +n = −z (away from the wall)
        // An accumulated force INTO the wall (acc along +z; acc·n = −800 < 0) strengthens the
        // response — the inward push is removed through the same normal (0x192248).
        let f2 = w.boundary_force(5.0, [0.0, 0.0, 800.0, 0.0], 1660.0, 1.0 / 60.0, 1.0);
        assert!(f2[2] < f[2], "inward accumulation adds to the outward push");
        assert!((f2[2] - -(5.0 / 60.0 + 800.0 / 1660.0)).abs() < 1e-5);
        // Sister: an outward accumulation (acc·n > 0) weakens the push, and the gate (k > 0,
        // 0x1922a0) would reject a receding body outright.
        let f3 = w.boundary_force(5.0, [0.0, 0.0, -800.0, 0.0], 1660.0, 1.0 / 60.0, 1.0);
        assert!(f3[2] > f[2], "outward accumulation weakens the push");
        let receding = w.boundary_force(-5.0, [0.0; 4], 1660.0, 1.0 / 60.0, 1.0);
        assert!(receding[2] > 0.0, "k < 0: the 0x1922a0 gate would not apply this");
    }
}
