//! level-13 fall check: the warm chain to 07000013, NO panel E, walk the east lane — does the
//! floor hold? Opt-in NF_ISO; prints only.
#[path = "common/walker_b2.rs"]
mod walker;
#[path = "common/tower2.rs"]
mod tower2;

use walker::{Walker, Wp};

#[test]
#[ignore = "assertion-free probe; run with --ignored"]
fn probe_07000013_warm_no_e_east_lane() {
    let iso = std::env::var("NF_ISO").expect("NF_ISO");
    let save = {
        let mut w = Walker::new(nf_view::load_with_carry_difficulty(&iso, "07000012.bin", false, None, 3).unwrap());
        tower2::walk_12(&mut w);
        tower2::walk_12_exit(&mut w);
        let sim = w.scene.sim.as_ref().unwrap();
        nf_game::player::RamSave::capture(&sim.drones.health, sim.gun.switch_request,
            &sim.drones.inventory, &sim.drones.channels).with_mission(&sim.drones.mission)
    };
    let save = {
        let mut w = Walker::new(nf_view::load_with_carry(&iso, "0700004a.bin", false, Some(&save)).unwrap());
        w.wait_enabled(3000, "the 4A intro");
        for _ in 0..600 { w.tick(nf_game::player::Input::default(), false); }
        let sim = w.scene.sim.as_ref().unwrap();
        nf_game::player::RamSave::capture(&sim.drones.health, sim.gun.switch_request,
            &sim.drones.inventory, &sim.drones.channels).with_mission(&sim.drones.mission)
    };
    let mut w = Walker::new(nf_view::load_with_carry(&iso, "07000013.bin", false, Some(&save)).unwrap());
    w.wait_enabled(3000, "the 13 intro");
    w.wait(120);
    for k in 0..6 {
        w.wait(40);
        let sim = w.scene.sim.as_ref().unwrap();
        eprintln!("[warm noE spawn] +{} y {:.2} ch4a {}", (k + 1) * 40, sim.bond.pos[1], sim.drones.channels.on(0x4a));
    }
    w.walk_to(Wp::new(5.0, -368.4, 26.5).r(1.5), "13: the lobby east lane");
    w.walk_to(Wp::new(3.0, -368.4, 45.0).r(1.5), "13: the lobby mid");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        eprintln!("[warm noE legs] y {:.2}", sim.bond.pos[1]);
    }
}
