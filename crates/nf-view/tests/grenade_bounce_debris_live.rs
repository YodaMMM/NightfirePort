//! The MGL 'Grenade' contact blast carries the surface it hit. `Bullet_CollisionHandler`'s
//! contact-explode arm loads the hit's surface byte and passes it as `Explode_Create`'s 12th
//! (stack) arg, the debris kind [M disasm 0x12d084 `lbu v0,0x50(s2)`, 0x12d094 `andi v0,v0,0x3f`,
//! 0x12d09c `sw v0,0x0(sp)`, 0x12d0bc `jal Explode_Create`]. `Explode_Update` spawns metal
//! debris (`Debris_CreateEx` 0x1ac808) for kinds 5/6 [M 0x17c350:31-67]. The port passed 0, so a
//! round that burst on a metal surface threw no pieces.
//!
//! Opt-in: `NF_ISO`. NF_SILENT everywhere.

use nf_game::player::Input;

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

/// Fires one contact round of weapon `w` straight ahead; returns the new blast's debris kind
/// and the most blast-debris pieces alive in the 60 ticks after it.
fn fire_ahead(scene: &mut nf_view::scene::Scene, w: u16) -> Option<(u16, usize)> {
    {
        let sim = scene.sim.as_mut().unwrap();
        let owned = sim.drones.inventory.owned.len();
        if (w as usize) < owned { sim.drones.inventory.owned[w as usize] = true; }
        sim.drones.inventory.clip[w as usize] = 6;
        sim.gun.weapon = w as u8;
        sim.gun.switch_request = w as u8;
        sim.gun.switch_state = 0;
    }
    for _ in 0..10 { scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false); }
    let sim = scene.sim.as_mut().unwrap();
    sim.gun.reload_state = 0;
    sim.gun.fire_state = false;
    sim.gun.clip = 6;
    sim.gun.pending = 0;
    let mut input = Input::default();
    input.fire = true;
    let mut kind = None;
    let mut most = 0usize;
    for t in 0..200 {
        let sim = scene.sim.as_mut().unwrap();
        let n0 = sim.drones.blasts.len();
        sim.step_bond(if t == 0 { input } else { Input::default() }, t == 0, false, false);
        let sim = scene.sim.as_mut().unwrap();
        if kind.is_none() && sim.drones.blasts.len() > n0 {
            let b = sim.drones.blasts.iter().rev().find(|b| b.weapon_type == w)?;
            kind = Some((b.debris, t));
        }
        if let Some((_, t0)) = kind {
            most = most.max(sim.drones.blast_debris.len());
            if t > t0 + 60 { break; }
        }
    }
    kind.map(|(k, _)| (k, most))
}

/// The MGL's 'Grenade' round (0x2A, a contact record: +0x70 & 4) fired at a metal shelf in the
/// 07000011 storeroom where the level places its MGL pickup: the blast's debris kind is the
/// shelf's surface byte (6), and metal pieces spawn. The old code's kind 0 spawned none. (The
/// AIMS-20's 0x1B round flies the port's bullet path, which already passed the material.)
#[test]
fn mgl_contact_blast_on_metal_throws_metal_debris() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    std::env::set_var("NF_SILENT", "1");
    let mut scene = nf_view::load_with_carry(&iso, "07000011.bin", false, None).expect("load 07000011");
    for _ in 0..30 { scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false); }
    // The MGL pickup's spot [D: the level's 0x2A pickup], facing the shelf 5.3 units away.
    let at = scene.sim.as_ref().unwrap().drones.pickups.iter().find(|p| p.weapon == 0x2a)
        .expect("07000011 places an MGL").start;
    let yaw = 0.35f32;
    let sim = scene.sim.as_ref().unwrap();
    let eye = [at[0], at[1] + 0.2, at[2]];
    let (_, _, m) = sim.world.segment_surface(eye, [eye[0] + yaw.sin() * 10.0, eye[1], eye[2] + yaw.cos() * 10.0], true)
        .expect("the shelf ahead");
    let mat = m & 0x3f;
    assert!(mat == 5 || mat == 6, "the shelf is metal (5/6), got {mat}");
    scene.sim.as_mut().unwrap().place_bond(at, yaw);
    let (kind, pieces) = fire_ahead(&mut scene, 0x2a).expect("the 0x2A round burst on contact");
    eprintln!("shelf mat {mat}, blast debris kind {kind}, most pieces alive {pieces}");
    assert!(kind == 5 || kind == 6,
        "the contact blast's debris kind is the hit's surface byte & 0x3f [M 0x12d084-0x12d09c], got {kind}");
    assert!(pieces > 0, "a metal contact blast throws metal debris [M 0x17c350:31-67, 0x1ac808]");
}
