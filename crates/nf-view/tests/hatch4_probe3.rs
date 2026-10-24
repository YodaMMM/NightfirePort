//! level-0a-office-hatch-4 probe 3: the port's floors along the walk route — the spawn, the
//! doorway line, the west hall, the corner — the port side of the [L] ledger.
#[path = "common/walker.rs"]
mod walker;
use walker::Walker;
fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

#[test]
#[ignore = "assertion-free dump; --ignored --nocapture"]
fn dump_route_floors() {
    let iso = iso();
    let w = Walker::new(nf_view::load(&iso, "0700000a.bin", false).unwrap());
    let sim = w.scene.sim.as_ref().unwrap();
    let pts: [(&str, [f32; 3]); 10] = [
        ("spawn cab", [-3.9319, 1.0, 10.3978]),
        ("cab mouth", [-3.9, 1.0, 13.2]),
        ("lobby south", [-3.9, 1.0, 15.5]),
        ("doorway -8.304", [-8.304, 1.0, 15.3]),
        ("doorway -8.304 z16", [-8.304, 1.0, 16.0]),
        ("hall -10.0", [-10.0, 1.0, 16.0]),
        ("hall -10.79", [-10.79, 1.0, 16.26]),
        ("hall -11.39", [-11.39, 1.0, 16.29]),
        ("corner -17.87", [-17.87, 1.0, 4.87]),
        ("corner exit", [-17.78, 1.0, 2.92]),
    ];
    for (tag, p) in pts {
        let f = sim.world.floor_below(p, 6.0);
        let b = sim.world.segment_bullet([p[0], p[1] + 0.5, p[2]], [p[0], p[1] - 6.0, p[2]])
            .map(|t| p[1] + 0.5 + (-6.0) * t);
        println!("{tag:16} ({:.2},{:.2}) port floor {:.5} bullet-set floor {:.5}", p[0], p[2], f.unwrap_or(f32::NAN), b.unwrap_or(f32::NAN));
    }
}
