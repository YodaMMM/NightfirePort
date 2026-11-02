//! Probe 6 (level-14-tripbomb-real-route): the gap through the crate band at x -36.6..-35.6
//! between the catwalk and the lane floor. Fine floor grid + body-height segments, then a walk
//! through the found gap.

#[path = "common/walker_b2.rs"]
mod walker;
use walker::{Walker, Wp};

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

#[test]
fn probe_lane_gap() {
    let iso_path = iso();
    let scene = nf_view::load(&iso_path, "07000014.bin", false).unwrap();
    let sim = scene.sim.as_ref().unwrap();
    for z in [32.6f32, 32.8, 33.0, 33.2, 33.4, 33.6, 33.8, 34.0, 34.2, 34.4, 34.6] {
        let mut row = String::new();
        for x in [-36.6f32, -36.4, -36.2, -36.0, -35.8, -35.6, -35.4] {
            let cell = match sim.world.floor_below([x, 6.0, z], 12.0) {
                Some(v) => format!("({x},{v:.2})"),
                None => format!("({x},--)"),
            };
            row.push_str(&format!("{cell} "));
        }
        // Body-height clear? A 0.55-high ray (chest) and a 1.4-high ray (head) across the band.
        let chest = sim.world.segment([-37.4f32, 0.55, z], [-34.6, 0.55, z]);
        let head = sim.world.segment([-37.4f32, 1.4, z], [-34.6, 1.4, z]);
        eprintln!("[probe6] z {z}: {row} chest {chest:?} head {head:?}");
    }
    drop(sim);
    // The lane's SOUTH entrance: nav routes from the loop stop and the descent landing to the
    // lane south (-33.2, 0.7, 29.5), then a walk in and north to the rack.
    #[path = "common/walker_b2.rs"]
    mod wl;
    use wl::nav_route as nr;
    let mut w = Walker::new(nf_view::load(&iso_path, "07000014.bin", false).unwrap());
    w.wait_enabled(3000, "the intro");
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.bond.pos = [-30.34f32, 0.6, 23.13];
        for _ in 0..15 { sim.step_bond(Default::default(), false, false, false); }
    }
    let sim = w.scene.sim.as_ref().unwrap();
    match nr(sim, [-30.34f32, 0.6, 23.13], [-33.2f32, 0.7, 29.5]) {
        Some(pts) => {
            eprintln!("[probe6] lane-south route {} nodes:", pts.len());
            for (k, p) in pts.iter().enumerate() { eprintln!("[probe6]   {k}: ({:.2},{:.2},{:.2})", p[0], p[1], p[2]); }
        }
        None => eprintln!("[probe6] NO nav route to the lane south"),
    }
    drop(sim);
    for (label, wp) in [
        ("south-west", Wp::new(-31.5, 0.4, 26.5).r(1.0)),
        ("the lane south mouth", Wp::new(-33.0, 0.6, 28.8).r(0.8)),
        ("into the lane", Wp::new(-33.2, 0.7, 30.9).r(0.7)),
        ("the lane floor", Wp::new(-33.3, 0.8, 33.3).r(0.7)),
        ("north to the rack front", Wp::new(-33.2, 0.85, 35.7).r(0.4)),
    ] {
        let ok = w.try_walk_to(wp, label);
        eprintln!("[probe6] {label}: ok {ok} bond {:?}", w.scene.sim.as_ref().unwrap().bond.pos);
    }
}
