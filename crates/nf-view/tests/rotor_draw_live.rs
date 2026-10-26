//! The rotors (0x71) draw at their spinning pose. `rotor_init` 0x18f4b8 creates the object at the
//! placement ROT (`control_create_object(0x20, pos, rot)`, class 7) and `rotor_update` 0x18f568 adds
//! ROT·FRAME_RATE_MUL to obj+0x50 every tick the gate allows (gate 0, or the gate channel equal to
//! ext+0x11) [M]. 07000011's three fans (#199, #205, #206 [D the level-11 audit]) held the placement
//! pose forever: the draw never read the sim's angle.
//!
//! Opt-in: `NF_ISO`. NF_SILENT everywhere.

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

fn opts() -> nf_view::scene::ViewOpts { nf_view::scene::ViewOpts::default() }

#[test]
fn the_07000011_fans_draw_at_the_spun_pose() {
    let Ok(_) = std::env::var("NF_ISO") else { eprintln!("NF_ISO not set — skipping"); return };
    std::env::set_var("NF_SILENT", "1");
    let mut scene = nf_view::load(&iso(), "07000011.bin", false).expect("load 07000011");
    for _ in 0..30 { scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false); }
    assert_eq!(scene.rotor_placements.len(), 3, "07000011 carries three 0x71 rows [D the level-11 audit]");
    assert_eq!(scene.sim.as_ref().unwrap().drones.worldobjs.rotors.len(), 3, "one rotor per 0x71 row [M 0x18f4b8]");
    // The posed slot: `lock_slot`'s chain — doors + SSs + breakables + 2/lock + GUNIMP_SLOTS/gunimp +
    // 3/sensor + 1/dynamic object, then one per rotor.
    let base = {
        let sim = scene.sim.as_ref().unwrap();
        sim.drones.doors.list.len() + sim.drones.sss.list.len() + sim.drones.breaks.list.len()
            + 2 * sim.drones.locks.len()
            + (1 + nf_game::gunimp::TURRET_ARTS.len()) * scene.gunimp_placements
            + 3 * sim.drones.sensors.list.len()
            + scene.dynobj_matrices.len()
    };
    let (layout, _attrs) = scene.skin_layout();
    let pose = |scene: &nf_view::scene::Scene, ri: usize| {
        let (pal, _cols) = scene.skin_palette(&layout, None, 240.0, opts(), None);
        pal[layout.door_base as usize + base + ri]
    };
    let expect = |scene: &nf_view::scene::Scene, ri: usize| {
        let (rot, trans) = scene.rotor_placements[ri];
        let a = scene.sim.as_ref().unwrap().drones.worldobjs.rotors[ri].angle;
        nf_formats::placement::object_matrix([rot[0] + a[0], rot[1] + a[1], rot[2] + a[2]], trans)
    };
    let close = |a: [[f32; 4]; 4], b: [[f32; 4]; 4]| (0..4).all(|r| (0..4).all(|c| (a[r][c] - b[r][c]).abs() < 1e-3));
    let mut moved = false;
    let before: Vec<_> = (0..3).map(|ri| pose(&scene, ri)).collect();
    for ri in 0..3 {
        assert!(close(before[ri], expect(&scene, ri)), "rotor {ri}: the slot rides placement ROT + the sim angle [M 0x18f4b8, 0x18f568]");
        let verts = scene.char_verts.iter().filter(|v| v.door == Some((base + ri) as u16)).count();
        assert!(verts > 0, "rotor {ri}: the fan mesh rides its slot");
    }
    for _ in 0..10 { scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false); }
    for ri in 0..3 {
        let after = pose(&scene, ri);
        assert!(close(after, expect(&scene, ri)), "rotor {ri}: still the spun pose after 10 ticks");
        if !close(after, before[ri]) { moved = true; }
    }
    assert!(moved, "the fans turn between ticks (gate channels off at load [D])");
}
