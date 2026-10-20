//! rocket-stop-plane-08: the 07000008 start-corridor rocket's stop surface at z ≈ −96.7
//! (GAPS G-049 follow-up). The game's rocket usually explodes at −96.2/−96.71 (normal +z) and
//! sometimes passes to the wall at −100.0; the port's world probe found no surface there.
//!
//! Opt-in: set `NF_ISO` to the disc image.

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

/// Fire the unguided launcher down the corridor from the level start, and probe the port's
/// world + doors along the flight line (this run: diagnostic prints).
#[test]
fn probe_the_corridor_line() {
    unsafe { std::env::set_var("NF_SILENT", "1"); }
    let Some(iso) = iso() else { return };
    let mut scene = nf_view::load_weapons(&iso, "07000008.bin", false, &[0x2d]).expect("load 08");
    let sim = scene.sim.as_mut().unwrap();

    // The doors: where does each sit at boot (the closed pose)?
    for (k, d) in sim.drones.doors.list.iter().enumerate() {
        let p = &d.matrix[3];
        let name = format!("door {k} placement {}", d.placement);
        if (p[2] + 97.0).abs() < 3.0 {
            eprintln!("{name}: pos ({:.3} {:.3} {:.3}) state {} pos01 {:.3} glist {:#x} kind {}",
                p[0], p[1], p[2], d.state, d.pos01, d.glist, d.kind);
            let has = sim.world.has_shape(d.glist);
            eprintln!("  shape present: {has}");
        }
    }

    // The world segment along the flight line, both directions.
    let a: nf_game::world::V3 = [139.52, 50.77, -80.0];
    let b: nf_game::world::V3 = [139.52, 50.77, -110.0];
    let hit = sim.world.segment_surface_bullet(a, b);
    eprintln!("segment_bullet fwd (−z): {hit:?}");
    let hit = sim.world.segment_surface_bullet(b, a);
    eprintln!("segment_bullet back (+z): {hit:?}");

    // The real fire path.
    let wd = sim.weapons.clone();
    let rounds = wd.clip_size(0x2d).max(1);
    assert!(sim.drones.inventory.equip_weapon(&wd, 0x2d, rounds, true), "equip the unguided launcher");
    sim.drones.inventory.equip_ammo(&wd, 0x2d, rounds * 3);
    sim.gun.switch_request = 0x2d;
    for _ in 0..150 { sim.step_bond(Default::default(), false, false, false); }
    assert_eq!(sim.gun.weapon, 0x2d, "the launcher is drawn");
    for _ in 0..10 { sim.step_bond(Default::default(), false, false, false); }

    // Mid-sim probes (the dynamic set now holds the doors): the flight line at three x offsets —
    // leaf 1's right stile (wood), pane 2 (glass), leaf 3's left stile (wood).
    for (label, x) in [("stile1 x139.52", 139.52f32), ("pane2 x139.74", 139.74), ("stile3 x139.58", 139.58)] {
        let a: nf_game::world::V3 = [x, 50.77, -90.0];
        let b: nf_game::world::V3 = [x, 50.77, -110.0];
        eprintln!("mid-sim {label}: fwd {:?} back {:?}", sim.world.segment_surface_bullet(a, b), sim.world.segment_surface_bullet(b, a));
    }
    let mut spawn = None;
    for _ in 0..30 {
        let before = sim.drones.player_bullets.len();
        sim.step_bond(Default::default(), true, false, false);
        if sim.drones.player_bullets.len() > before {
            let bl = sim.drones.player_bullets.last().unwrap().clone();
            spawn = Some([bl.pos[0] - bl.dir[0] * bl.travelled, bl.pos[1] - bl.dir[1] * bl.travelled, bl.pos[2] - bl.dir[2] * bl.travelled]);
            break;
        }
    }
    let spawn = spawn.expect("a rocket was fired");
    eprintln!("rocket spawn ({:.3} {:.3} {:.3}) dir ({:.4} {:.4} {:.4})",
        spawn[0], spawn[1], spawn[2],
        sim.drones.player_bullets.last().unwrap().dir[0],
        sim.drones.player_bullets.last().unwrap().dir[1],
        sim.drones.player_bullets.last().unwrap().dir[2]);
    let mut last = spawn;
    for _ in 0..140 {
        if sim.drones.player_bullets.iter().all(|bl| bl.weapon != 0x2d) { break; }
        sim.step_bond(Default::default(), false, false, false);
        if let Some(bl) = sim.drones.player_bullets.iter().find(|bl| bl.weapon == 0x2d) { last = bl.pos; }
    }
    eprintln!("rocket last live pos ({:.3} {:.3} {:.3})", last[0], last[1], last[2]);
    for i in sim.drones.player_impacts.iter() { eprintln!("impact {i:?}"); }
}
