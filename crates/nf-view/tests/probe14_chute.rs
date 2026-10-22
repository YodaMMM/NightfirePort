//! level-14 probe (level-14-complete-2): every collision triangle in the stairwell chute box
//! (x 39.5..42.5, y −3.6..4.0, z 54.0..57.0), printed with vertices and normals — the slot's
//! real shape read off the disc. Prints only.

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

#[test]
fn probe_chute_tris() {
    let iso = iso();
    let mut scene = nf_view::load(&iso, "07000014.bin", false).unwrap();
    {
        let sim = scene.sim.as_ref().unwrap();
        let (lo, hi) = ([39.5f32, -3.6, 54.0], [42.5f32, 4.0, 57.0]);
        let mut n = 0;
        eprintln!("== static ==");
        for set in ["static", "missile", "dynamic"] {
            let tris = match set {
                "static" => sim.world.tris_near(lo, hi),
                "missile" => sim.world.missile_tris_near(lo, hi),
                _ => sim.world.dynamic_tris_near(lo, hi),
            };
            eprintln!("== {set}: {} candidate tris ==", tris.len());
            for t in tris {
            // Keep only the tris whose OWN bbox meets the box (tris_near filters by object box).
            let (tl, th) = (
                [t[0][0].min(t[1][0]).min(t[2][0]), t[0][1].min(t[1][1]).min(t[2][1]), t[0][2].min(t[1][2]).min(t[2][2])],
                [t[0][0].max(t[1][0]).max(t[2][0]), t[0][1].max(t[1][1]).max(t[2][1]), t[0][2].max(t[1][2]).max(t[2][2])],
            );
            if (0..3).any(|k| th[k] < lo[k] || tl[k] > hi[k]) { continue; }
            let t: &[nf_game::world::V3; 3] = &t;
            let ab = [t[1][0] - t[0][0], t[1][1] - t[0][1], t[1][2] - t[0][2]];
            let ac = [t[2][0] - t[0][0], t[2][1] - t[0][1], t[2][2] - t[0][2]];
            let nx = ab[1] * ac[2] - ab[2] * ac[1];
            let ny = ab[2] * ac[0] - ab[0] * ac[2];
            let nz = ab[0] * ac[1] - ab[1] * ac[0];
            let l = (nx * nx + ny * ny + nz * nz).sqrt();
            if l < 1e-9 { return; }
            eprintln!(
                "tri ({:.3},{:.3},{:.3}) ({:.3},{:.3},{:.3}) ({:.3},{:.3},{:.3}) n ({:.2},{:.2},{:.2})",
                t[0][0], t[0][1], t[0][2], t[1][0], t[1][1], t[1][2], t[2][0], t[2][1], t[2][2],
                nx / l, ny / l, nz / l
            );
            n += 1;
        }
        }
        eprintln!("total {n} tris in the chute box");
    }
}
