//! The decompiled `Intersect_CylGeom` push tail on 0700000a against the game's own walked traces.
//!
//! The tail (`nf_game::world::set_cyl_tail`, the citations on `cyl_tail` in world.rs): both
//! `ASH_vecutil_Dist2Tri` 0x109770 calls pass `t2 = HITTEST copy + 0x50` as the out-pointer
//! [M disasm 0x10a2fc / 0x10a4f0], so the push tail's `v = p − HITTEST+0x50` is `p − q` against the
//! triangle under test: an ordinary capsule push `(r − |v|)·v̂` gated by `dot(v, n) > 0`, with the
//! vertical-face (|denom| ≤ 0.0002) arm taking p from the triangle centroid and never
//! re-projecting it [M 0x10a3f0-0x10a508]. Every test here turns the tail on for this binary.
//! `NF_CYL_TAIL_PROBE_OFF=1` runs the same pins on the default contact model (they fail there:
//! the proof the pins measure the tail).
#[path = "common/walker_b2.rs"]
mod walker;
use walker::{Walker, Wp};

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

fn load() -> Walker {
    if std::env::var_os("NF_CYL_TAIL_PROBE_OFF").is_none() { nf_game::world::set_cyl_tail(true); }
    let mut w = Walker::new(nf_view::load_with_carry(&iso(), "0700000a.bin", false, None).expect("load 0700000a"));
    // The lift's two SS doors (placements 524/525) finish their boot sweep first.
    for _ in 0..150 {
        let done = w.scene.sim.as_ref().unwrap().drones.sss.list.iter()
            .filter(|s| s.placement == 524 || s.placement == 525)
            .all(|s| s.state == 0 || s.state == 0x40);
        if done { break; }
        w.wait(5);
    }
    w
}

fn forward(w: &mut Walker) -> [f32; 3] {
    w.tick(nf_game::player::Input { forward: 1.0, ..Default::default() }, false);
    w.scene.sim.as_ref().unwrap().bond.pos
}

/// The west-hall climb (live-0a-walk3.csv, fresh boot): the game's leg out of the lobby holds
/// forward on a fixed heading (per-frame step (−0.0988, +0.0151) [L frames 10736-10749]), stops on
/// the column's east face at x −7.7545 [L 10750], slides north along it, starts to rise at z
/// ≈ 15.844 [L 10766: y 1.0901], peaks over the feet band (1.1819 [L 10780]), drops back and
/// glides 1.1324 while sliding round the jamb to its north edge z 16.8288 [L 10806-10820].
#[test]
fn the_west_hall_climb_follows_the_game_trace() {
    let mut w = load();
    w.walk_to(Wp::new(-3.9, 0.0, 13.2), "out of the lift");
    w.walk_to(Wp::new(-3.9, 0.0, 15.0).r(0.3), "lobby");
    w.walk_to(Wp::new(-4.79, 0.0, 15.20).r(0.15), "the leg's start [L 10720 (-4.7902, 15.2033)]");
    w.scene.sim.as_mut().unwrap().bond.yaw = (-0.98837f32).atan2(0.15123);
    let (mut stop_x, mut rise_z, mut peak, mut max_z) = (None, None, 0.0f32, 0.0f32);
    let mut slide_y = None;
    for _ in 0..80 {
        let p = forward(&mut w);
        if stop_x.is_none() && p[0] > -7.9 && p[0] < -7.7 && (p[2] - 15.65).abs() < 0.1 { stop_x = Some(p[0]); }
        if rise_z.is_none() && p[1] > 1.0895 { rise_z = Some(p[2]); }
        peak = peak.max(p[1]);
        max_z = max_z.max(p[2]);
        if slide_y.is_none() && p[2] > 16.6 { slide_y = Some(p[1]); }
    }
    eprintln!("[climb] stop x {stop_x:?} rise z {rise_z:?} peak {peak:.4} max z {max_z:.4} slide y {slide_y:?}");
    let stop_x = stop_x.expect("the leg reaches the column face");
    assert!((stop_x + 7.7545).abs() < 0.002, "the wall stop sits at the game's x −7.7545 [L], got {stop_x:.4}");
    let rise_z = rise_z.expect("the climb starts");
    assert!((rise_z - 15.844).abs() < 0.05, "the climb starts at the game's z 15.844 [L], got {rise_z:.4}");
    assert!(peak > 1.1347, "the climb peaks over the feet band's edge cc + 0.1 = 1.1347 [L peak 1.1819], got {peak:.4}");
    assert!((max_z - 16.8288).abs() < 0.003 || max_z > 16.8288, "the slide reaches the jamb's north edge z 16.8288 [L], max z {max_z:.4}");
    let slide_y = slide_y.expect("the slide passes z 16.6");
    assert!((slide_y - 1.1324).abs() <= 0.004, "the slide glides the game's 1.1324 [L 10806-10820], got {slide_y:.4}");
}
