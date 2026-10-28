//! scratch: who stands near 07000011's west flight C
//! bottom while Bond descends — dump the bots' kind_464 around the wedge. Assertion-free, --nocapture.
#[path = "common/walker.rs"]
mod walker;

use walker::{Walker, Wp};

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

#[test]
fn probe_07000011_flight_c() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load(&iso, "07000011.bin", false).expect("load 07000011"));
    w.wait_enabled(3000, "boot");
    w.walk_to(Wp::new(12.0, 47.4, 6.6).r(0.9), "the ledge west");
    w.walk_to(Wp::new(2.2, 47.4, 6.6).r(0.9), "the ledge's west end (n12)");
    w.walk_to(Wp::new(1.2, 47.4, 3.0).r(0.6), "the west bridge south");
    w.walk_to(Wp::new(1.4, 47.4, -1.0).r(0.8), "onto the north balcony (n45)");
    w.walk_to(Wp::new(5.0, 47.4, -1.8).r(0.9), "the balcony east (n46/n47)");
    w.walk_to(Wp::new(9.0, 47.4, -2.2).r(0.9), "the north strip east");
    w.walk_to(Wp::new(12.2, 47.4, -5.8).r(0.9), "the top flight east");
    w.walk_to(Wp::new(18.6, 46.4, -5.4).r(0.9), "the north flight down (46.8→44.4)");
    w.walk_to(Wp::new(22.9, 44.4, -3.4).r(0.9), "the north-east landing 43.1");
    w.walk_to(Wp::new(23.2, 40.4, 2.6).r(0.9), "the east flight down (43.1→39.1)");
    w.walk_to(Wp::new(21.3, 40.0, 5.2).r(0.9), "the south-east landing 39.1");
    w.walk_to(Wp::new(16.2, 36.4, 5.9).r(0.9), "the south run down (39.1→35.2)");
    w.walk_to(Wp::new(15.5, 36.2, 5.5).r(0.9), "the south-west landing 35.2");
    w.walk_to(Wp::new(14.8, 31.4, -2.6).r(1.0), "the west flight C down (35.2→31.1)");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let bp = sim.bond.pos;
        let ground = sim.world.segment([bp[0], bp[1], bp[2]], [bp[0], bp[1] - 3.0, bp[2]]);
        println!("at-band bond {:?} grounded {} ground_below {:?}", bp, sim.bond.grounded, ground);
        let legs: Vec<[f32; 3]> = vec![[bp[0], bp[1] - 0.6, bp[2] - 1.2], [bp[0], bp[1] - 0.6, bp[2] + 1.2], [bp[0] + 1.5, bp[1] - 0.6, bp[2]]];
        for t in legs {
            let hit = sim.world.segment([t[0], t[1] + 1.0, t[2]], [t[0], t[1] - 2.5, t[2]]);
            println!("probe floor at {:?} -> {:?}", t, hit);
        }
    }
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let bp = sim.bond.pos;
        println!("bond {:?}", bp);
        for (i, d) in sim.drones.list.iter().enumerate() {
            let dx = d.pos[0] - bp[0]; let dy = d.pos[1] - bp[1]; let dz = d.pos[2] - bp[2];
            let d2 = dx * dx + dy * dy + dz * dz;
            if d2 < 16.0 {
                println!("drone {} pos {:?} state {:#x} team {} enabled {} kind_464 {:#x} dist_to_player[0] {:.2} slots {:?}", i, d.pos, d.state, d.team, d.enabled, d.kind_464, d.dist_to_player[0], d.dist_to_player);
            }
        }
    }
}
