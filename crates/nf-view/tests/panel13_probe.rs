//! level-13 probe 4: the east-strip route (0x56 off?) + the armour take approaches.
#[path = "common/walker_b2.rs"]
mod walker;

use walker::{Walker, Wp};

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

#[test]
#[ignore = "probe dump; run with --ignored"]
fn probe_13d_east_strip_and_armour() {
    let mut w = Walker::new(nf_view::load(&iso(), "07000013.bin", false).unwrap());
    w.wait_enabled(3000, "the 13 intro");
    w.wait(120);
    let legs: [([f32; 3], f32, &str); 9] = [
        ([5.0, -368.4, 24.0], 1.2, "the east lane south"),
        ([8.2, -368.4, 26.0], 1.2, "the east strip mouth"),
        ([8.5, -368.4, 30.0], 1.2, "north through the east strip"),
        ([0.0, -368.4, 40.0], 1.5, "the desks' north side"),
        ([-14.0, -368.4, 40.0], 1.5, "the lobby north-west"),
        ([-22.0, -368.4, 39.5], 1.5, "past the glass band"),
        ([-24.5, -368.4, 38.8], 1.2, "at the security-room door"),
        ([-27.0, -368.4, 37.0], 1.0, "through the doorway"),
        ([-25.5, -368.4, 32.5], 1.2, "the room south-east"),
    ];
    for (at, r, label) in legs {
        let ok = w.try_walk_to(Wp { at, act: walker::Act::None, arrive: r }, label);
        let sim = w.scene.sim.as_ref().unwrap();
        eprintln!("[east] {label}: ok {ok} pos ({:.2},{:.2},{:.2}) 0x56 {}", sim.bond.pos[0], sim.bond.pos[1], sim.bond.pos[2],
            sim.drones.channels.on(0x56));
        if !ok { break; }
    }
    w.walk_to(Wp::at(-26.65, -368.36, 37.44, walker::Act::Use { use_ticks: 90 }).r(1.4), "E the door");
    w.wait(120);
    w.walk_to(Wp::new(-27.0, -368.4, 35.0).r(1.2), "inside");
    w.walk_to(Wp::new(-22.0, -368.4, 29.0).r(1.2), "the room south");
    w.walk_to(Wp::new(-20.48, -368.4, 25.38).r(1.2), "the Spas12 shelf");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let owned = sim.drones.inventory.owned[0x1c] || sim.gun.weapon == 0x1c;
        eprintln!("[east] Spas12 owned {owned} 0x56 {}", sim.drones.channels.on(0x56));
    }
    // the armour: approach via the room's west strip (north of the shelf), then close in
    w.walk_to(Wp::new(-26.5, -368.4, 27.5).r(1.0), "the room west, north of the shelf");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        eprintln!("[east] at the west strip: pos ({:.2},{:.2},{:.2}) y-floor {:.2}",
            sim.bond.pos[0], sim.bond.pos[1], sim.bond.pos[2], sim.bond.pos[1] + 368.5);
    }
    let ok = w.try_walk_to(Wp::new(-27.66, -368.4, 25.57).r(0.7), "the BodyArmour shelf close");
    let sim = w.scene.sim.as_ref().unwrap();
    eprintln!("[east] armour leg ok {ok}: pos ({:.2},{:.2},{:.2}) armour owned {} (dist to pickup {:.2})",
        sim.bond.pos[0], sim.bond.pos[1], sim.bond.pos[2], sim.drones.inventory.owned[0x32],
        ((sim.bond.pos[0] + 27.66).powi(2) + (sim.bond.pos[2] - 25.57).powi(2)).sqrt());
    for _ in 0..240 {
        let mut sim = w.scene.sim.as_mut().unwrap();
        // face the pickup and wait (the take is a walk-over; give it ticks)
        let p = sim.bond.pos;
        let d = [(p[0] + 27.66 - 0.0f32).abs(), 0.0, (p[2] - 25.57).abs()];
        let _ = d;
        let input = nf_game::player::Input { forward: 1.0, turn: 0.02, ..Default::default() };
        sim.step_bond(input, false, false, false);
        let sim = w.scene.sim.as_ref().unwrap();
        if sim.drones.inventory.owned[0x32] { eprintln!("[east] armour TAKEN at ({:.2},{:.2},{:.2})", sim.bond.pos[0], sim.bond.pos[1], sim.bond.pos[2]); break; }
    }
    let sim = w.scene.sim.as_ref().unwrap();
    eprintln!("[east] armour owned final {} pos ({:.2},{:.2},{:.2})",
        sim.drones.inventory.owned[0x32], sim.bond.pos[0], sim.bond.pos[1], sim.bond.pos[2]);
}
