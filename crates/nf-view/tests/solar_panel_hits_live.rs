//! The 0700001b solar panels take a bullet on their triangles, not their bounding sphere.
//! PCSX2: 20 bursts along the walkway from (16.74,-1.84,4.62) broke nothing. NF_ISO-gated.

use nf_game::player::Input;

fn broken(sim: &nf_view::scene::Sim) -> usize {
    sim.drones.scripts.players.iter().filter(|p| (41..=56).contains(&p.placement))
        .filter(|p| !p.info[p.cur].as_ref().is_some_and(|s| s.delay != 0)).count()
}

#[test]
fn firing_along_the_walkway_breaks_no_panel() {
    let Ok(iso) = std::env::var("NF_ISO") else { return };
    for yaw in [-1.57f32, 1.57] {
        let mut scene = nf_view::load_with_carry_difficulty(&iso, "0700001b.bin", false, None, 1).unwrap();
        for _ in 0..3000 { scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, false); }
        let sim = scene.sim.as_mut().unwrap();
        sim.bond.pos = [16.74, -1.84, 4.62];
        sim.bond.yaw = yaw;
        for _ in 0..60 { sim.step_bond(Input::default(), false, false, false); }
        assert_eq!(broken(sim), 0);
        for i in 0..480 { sim.step_bond(Input::default(), i % 24 < 8, false, false); }
        assert_eq!(broken(sim), 0, "yaw {yaw}");
    }
}
