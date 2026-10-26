//! level-0a-office-hatch-7 probe: the five settle-regression scenarios, per-tick state traces.
//! Run twice (settle armed / PROBE-DISABLED) and diff. Scratch tool — not a test.

use nf_game::player::Input;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

fn torpedo() {
    eprintln!("=== torpedo ===");
    const CATALOG: &[u8] = &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 20, 21, 22, 23, 24, 25, 26, 28, 30, 32, 33, 34, 36, 38, 40, 42, 43, 44, 46, 48, 50, 52, 54, 56, 58, 60, 62, 64, 66, 68, 70, 84, 85, 100, 101, 102, 113];
    let mut scene = nf_view::load_with_carry_weapons(&iso(), "07000005.bin", false, None, CATALOG).expect("load");
    for _ in 0..30 { scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false); }
    let (_, sw) = nf_view::panel::apply(&mut scene, &[
        ("do".to_string(), "weapon".to_string()), ("id".to_string(), "0x30".to_string()), ("rounds".to_string(), "3".to_string())]);
    eprintln!("[give] sw {sw:?}");
    for _ in 0..60 {
        scene.sim.as_mut().unwrap().step_bond(Input { switch_to: Some(0x30), ..Default::default() }, false, false, false);
        if scene.sim.as_ref().unwrap().gun.weapon == 0x30 { break; }
    }
    for _ in 0..200 {
        let sim = scene.sim.as_ref().unwrap();
        if sim.gun.switch_state == 0 && sim.gun.reload_state == 0 && !sim.gun.fire_state { break; }
        scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false);
    }
    for _ in 0..5 { scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false); }
    for _ in 0..10 { scene.sim.as_mut().unwrap().step_bond(Input { look: -0.5, ..Default::default() }, false, false, false); }
    { let sim = scene.sim.as_mut().unwrap();
      sim.gun.reload_state = 0; sim.gun.fire_state = false; sim.gun.pending = 0; sim.gun.clip = 3;
      let w = sim.gun.weapon as usize;
      if let Some(c) = sim.drones.inventory.clip.get_mut(w) { *c = 3; }
      if (0x30 as usize) < sim.drones.inventory.owned.len() { sim.drones.inventory.owned[0x30 as usize] = true; }
      if let Some(c) = sim.drones.inventory.clip.get_mut(0x30 as usize) { *c = 3; } }
    { let sim = scene.sim.as_ref().unwrap();
      eprintln!("[sw-states] switch {} reload {} fire {} pending {} anim-obj {}", sim.gun.switch_state, sim.gun.reload_state, sim.gun.fire_state, sim.gun.pending, sim.gun.weapon); }
    let sim = scene.sim.as_ref().unwrap();
    eprintln!("[pre-fire] pos {:?} pitch {} weapon {:#x} switch {} reload {} fire_state {} clip {:?} bullets {}",
        sim.bond.pos, sim.bond.pitch, sim.gun.weapon, sim.gun.switch_state, sim.gun.reload_state, sim.gun.fire_state,
        sim.drones.inventory.clip.get(0x30 as usize), sim.drones.player_bullets.len());
    for t in 0..10 {
        let fire = t == 0;
        scene.sim.as_mut().unwrap().step_bond(Default::default(), fire, false, false);
        let sim = scene.sim.as_ref().unwrap();
        eprintln!("[t{t}] pos-y {:.5} fire_state {} bullets {} last {:?}", sim.bond.pos[1], sim.gun.fire_state,
            sim.drones.player_bullets.len(), sim.drones.player_bullets.last().map(|b| (b.weapon, b.speed)));
    }
}

fn seam() {
    eprintln!("=== seam ===");
    let mut scene = nf_view::load(&iso(), "07000001.bin", false).expect("load 01");
    { let sim = scene.sim.as_mut().unwrap();
      let idle = Input::default();
      let mut waited = 0;
      for _ in 0..2000 { if !sim.disabled { break; } sim.step_bond(idle, false, false, false); waited += 1; }
      eprintln!("[intro] waited {waited} disabled {} pos {:?}", sim.disabled, sim.bond.pos);
      sim.bond.pos = [-2.62, 6.468, -1.12];
      sim.bond.vel = [0.0; 3];
      for t in 0..3 {
          sim.step_bond(idle, false, false, false);
          eprintln!("[tp t{t}] pos {:?} vel {:?} grounded {} cel {:?}", sim.bond.pos, sim.bond.vel, sim.bond.grounded, sim.bond.cel);
      } }
    let sim = scene.sim.as_ref().unwrap();
    let (root, cap) = (sim.bond.cel, sim.bond.capsule_seg());
    let (a, b) = cap;
    eprintln!("[read] root {root:?} a {a:?} b {b:?} pos {:?}", sim.bond.pos);
    if let Some(root) = root {
        let world = &sim.world;
        let set = world.debug_straddle(root, a, b, nf_game::player::RADIUS);
        let (flat, flat_w) = world.capsule_push(a, b, nf_game::player::RADIUS, false, [1.0e9, 1.0e9, 1.0e9]);
        let (scoped, scoped_w) = world.capsule_push_scoped(Some(root), a, b, nf_game::player::RADIUS, false, [1.0e9, 1.0e9, 1.0e9]);
        eprintln!("[push] straddle {set:?} flat {flat:?} scoped {scoped:?} walk {flat_w}/{scoped_w}");
    }
}

fn armour() {
    eprintln!("=== armour ===");
    let mut scene = nf_view::load(&iso(), "07000009.bin", false).expect("load 09");
    { let sim = scene.sim.as_mut().unwrap();
      let idle = Input::default();
      for _ in 0..2000 { if !sim.disabled { break; } sim.step_bond(idle, false, false, false); }
      // settle_near's stand: the walkreal walker's stand-at (floor+1.1 via stand_at) then ticks
      let w = &sim.world;
      let p = [-21.2, -369.0, 24.86];
      let y = w.floor_below([p[0], p[1] + 1.0, p[2]], 10.0).map_or(p[1], |f| f + 1.1);
      sim.bond.pos = [p[0], y, p[2]];
      sim.bond.vel = [0.0; 3];
      eprintln!("[placed] pos {:?}", sim.bond.pos);
      for t in 0..12 {
          sim.step_bond(idle, false, false, false);
          let gone = sim.drones.pickups.iter().find(|p| (p.m[3][0] + 22.57).abs() < 0.5).map(|p| p.gone);
          eprintln!("[t{t}] pos {:?} grounded {} armour {:.3} gone {:?}", sim.bond.pos, sim.bond.grounded, sim.drones.health.armour, gone);
      } }
}

fn bathhouse() {
    eprintln!("=== bathhouse ===");
    let mut scene = nf_view::load(&iso(), "07000002.bin", false).expect("load 02");
    let mut waited = 0;
    { let sim = scene.sim.as_mut().unwrap();
      let idle = Input::default();
      for _ in 0..3000 { if !sim.disabled { break; } sim.step_bond(idle, false, false, false); waited += 1; }
      for _ in 0..300 { sim.step_bond(idle, false, false, false); }
      eprintln!("[intro] waited {waited} disabled {} pos {:?}", sim.disabled, sim.bond.pos);
      let c = sim.drones.triggers.iter().find(|t| t.out == 0x81).unwrap().centre();
      sim.bond.pos = [c[0], c[1] - 1.0, c[2]];
      eprintln!("[teleport] pos {:?}", sim.bond.pos); }
    for f in 0..40 {
        scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false);
        let sim = scene.sim.as_ref().unwrap();
        eprintln!("[f{f}] pos {:?} grounded {}", sim.bond.pos, sim.bond.grounded);
    }
}

fn main() {
    let which = std::env::args().nth(1).unwrap_or_default();
    match which.as_str() {
        "torpedo" => torpedo(),
        "seam" => seam(),
        "armour" => armour(),
        "bath" => bathhouse(),
        _ => { torpedo(); seam(); armour(); bathhouse(); }
    }
}
