//! Probe (print-only): for every 07000015 trigger, compare the glist's plane RECORD side against
//! the winding-derived normal side, and evaluate both at the use-pick sphere law's points.
//! The game's use-pick window is `(0, r]` against the RECORD plane [M Intersect_PointGeom
//! 0x1e92d8: fVar25 = 0 when HITTEST+0x9a & 8 == 0 — the use pick passes 0 — and X = the
//! record's n·p + d, loaded from the glist's record table]. The port's `sphere_meets_tris`
//! derives the normal from the winding instead. A sign flip between record and winding on a
//! tri the player's use sphere must admit = an E the game accepts and the port refuses.
//! Opt-in NF_ISO; prints only (a census, like census_07000015).
use nf_game::player::Input;

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

fn winding_normal(t: &[[f32; 3]; 3]) -> [f32; 3] {
    let a = [t[1][0] - t[0][0], t[1][1] - t[0][1], t[1][2] - t[0][2]];
    let b = [t[2][0] - t[0][0], t[2][1] - t[0][1], t[2][2] - t[0][2]];
    let n = [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]];
    let l = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
    if l <= 0.0 { [0.0, 0.0, 0.0] } else { [n[0] / l, n[1] / l, n[2] / l] }
}

#[test]
#[ignore = "assertion-free probe dump; run with --ignored"]
fn probe_07000015_record_vs_winding_sides() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set"); return };
    let mut scene = nf_view::load(&iso, "07000015.bin", false).expect("load 07000015");
    let sim = scene.sim.as_mut().unwrap();
    for _ in 0..3000 { sim.step_bond(Input::default(), false, false, false); if !sim.disabled { break; } }
    let world = &scene.sim.as_ref().unwrap().drones;
    for (ti, t) in world.triggers.iter().enumerate() {
        if t.tris.is_empty() { continue; }
        let kind = t.kind;
        let mut flips = 0usize;
        let mut same = 0usize;
        let mut rec_zero = 0usize;
        for (i, tri) in t.tris.iter().enumerate() {
            let Some((n_rec, d_rec)) = t.planes.get(i) else { continue };
            let n_win = winding_normal(tri);
            let dot = n_rec[0] * n_win[0] + n_rec[1] * n_win[1] + n_rec[2] * n_win[2];
            // A degenerate record (all-zero normal) reads as no side.
            if n_rec.iter().all(|&x| x == 0.0) { rec_zero += 1; continue; }
            if dot < -0.25 { flips += 1; } else if dot > 0.25 { same += 1; }
        }
        if flips > 0 || matches!(kind, 10 | 2 | 4) {
            println!("trigger #{ti} kind {kind} tris {} record-flipped {flips} same {same} zero-rec {rec_zero} \
                box lo {:?} hi {:?}", t.tris.len(), t.lo, t.hi);
        }
    }
    // The exit #445 and the gated gate #343 in detail: evaluate X_record and X_winding at the
    // two use points the walk test exercises (standing still after the doorway stop, and the
    // walk-in start), with the yaw-PI look the test uses.
    let spots: [(&str, [f32; 3], f32); 3] = [
        ("standstill head≈stop", [0.0, 41.6, -24.15], std::f32::consts::PI),
        ("walk-in start", [0.0, 41.6, -23.3], std::f32::consts::PI),
        ("sister #343 side", [70.9, 34.0, 0.9], std::f32::consts::PI),
    ];
    for (ti, t) in world.triggers.iter().enumerate() {
        if t.tris.is_empty() { continue; }
        if !(t.lo[0] < 1.0 && t.hi[0] > -1.0 && t.lo[2] < -24.0 && t.hi[2] > -26.0) { continue; }
        println!("== exit-like trigger #{ti} kind {} next {:?} gate {}", t.kind, t.next_level, t.gate);
        for (name, head, yaw) in spots {
            // yaw PI looks toward -z in the port's convention (the walk test walks forward to z↓).
            let look = [yaw.sin() * 0.0 - 0.0, 0.0, -(yaw.cos() * 0.0 + 1.0)];
            let up = [0.0, 1.0, 0.0];
            let _ = up;
            let use_point = [head[0] + look[0], head[1] + look[1], head[2] + look[2]];
            let mut x_rec_min = f32::INFINITY; let mut x_win_min = f32::INFINITY;
            for (i, tri) in t.tris.iter().enumerate() {
                let Some((n_rec, d_rec)) = t.planes.get(i) else { continue };
                let x_rec = n_rec[0] * use_point[0] + n_rec[1] * use_point[1] + n_rec[2] * use_point[2] + d_rec;
                let n_win = winding_normal(tri);
                let d_win = -(n_win[0] * tri[0][0] + n_win[1] * tri[0][1] + n_win[2] * tri[0][2]);
                let x_win = n_win[0] * use_point[0] + n_win[1] * use_point[1] + n_win[2] * use_point[2] + d_win;
                x_rec_min = x_rec_min.min(x_rec);
                x_win_min = x_win_min.min(x_win);
            }
            println!("   {name}: use point {:?}", use_point);
            println!("      min X over tris: record {x_rec_min:+.3}  winding {x_win_min:+.3}  (window (0,1.0])");
        }
    }
}
