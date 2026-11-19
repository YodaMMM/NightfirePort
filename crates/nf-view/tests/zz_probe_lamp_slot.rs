#[path = "common/walker.rs"]
#[allow(dead_code)]
mod walker;
use walker::Walker;
#[test]
#[ignore]
fn lamp_motion() {
    let iso = std::env::var("NF_ISO").unwrap();
    let mut w = Walker::new(nf_view::load(&iso, "07000004.bin", false).expect("load"));
    for m in w.scene.materials.iter() {
        let c = m.centre;
        if (c[0] - 8.01).abs() < 1.5 && (c[1] - 7.5).abs() < 2.0 && (c[2] - 23.03).abs() < 1.5 {
            eprintln!("mat near lamp40: layer {:?} centre {:?} r {:.2} skin {:?}", m.layer, c, m.radius, m.skin);
        }
    }
    for _ in 0..6000u32 {
        w.tick(Default::default(), false);
        let sim = w.scene.sim.as_ref().unwrap();
        if sim.drones.npc_enabled && !sim.disabled && sim.drones.channels.on(0x53) { break; }
    }
    w.walk_nav_to([8.0, 4.4, 26.5], "under", 1.2);
    let at = { let o = w.scene.sim.as_ref().unwrap().drones.rbodies.list.iter().find(|o| o.placement == 40).unwrap(); o.st.trans };
    let m0 = w.scene.sim.as_ref().unwrap().drones.rbodies.list.iter().find(|o| o.placement == 40).unwrap().st.matrix;
    w.shoot_point(at, "lamp", |sim| sim.drones.rbodies.list.iter().find(|o| o.placement == 40).unwrap().st.matrix != m0);
    for t in 0..180 {
        w.tick(Default::default(), false);
        if t % 15 == 0 {
            let o = w.scene.sim.as_ref().unwrap().drones.rbodies.list.iter().find(|o| o.placement == 40).unwrap();
            eprintln!("t{t} trans {:?} row1 {:?} omega {:?}", o.st.trans, o.st.matrix[1], o.st.omega);
        }
    }
}
