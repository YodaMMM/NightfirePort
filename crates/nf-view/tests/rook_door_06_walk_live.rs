//! rook-door-06-2: the driver's walk never starts — the port's `TRUCK_DRIVER_INIT` anim-done
//! arm hard-codes TruckDriverIdle, the game's case 5 enters TruckDriverMission when the kind-8
//! route was assigned (`NDrone2_DSTATE_TruckDriverInit` 0x160bd0 case 5: `+0x9f2 != 0` → 0x35).
//! Real path: the truck-ride carry (start #496), the intro NIS, the arrival plates (ch 185),
//! then the driver (skin 0x05000035, mode 0x17 — the family `Global` 0x15b830 routes to 0x32)
//! finishes his sit script and must walk his mission route (path 10). Opt-in via NF_ISO.

#[path = "common/walker.rs"]
mod walker;

use nf_game::player::Input;

use walker::Walker;

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

fn carry(on: &[u32]) -> nf_game::player::RamSave {
    let mut ch = nf_game::triggers::Channels::default();
    for &c in on { ch.set(c, 1, 0); }
    nf_game::player::RamSave::capture(&Default::default(), 0, &nf_game::pickup::Inventory::level_start(0x73, 34), &ch)
}

fn driver_index(sim: &nf_view::scene::Sim) -> Option<usize> {
    sim.drones.list.iter().position(|d| d.present && !d.corpse && d.skin == 0x05000035 && d.mode == 0x17)
}
fn escort_index(sim: &nf_view::scene::Sim) -> Option<usize> {
    sim.drones.list.iter().position(|d| d.present && !d.corpse && d.skin == 0x0500002e && d.switch_channel == 185)
}

/// The real path to the walk start: carry → intro NIS → the ch-185 plates → the driver's sit
/// script ends → he must enter TruckDriverMission (0x35) and walk (>2 units from his park
/// spot) inside 12k ticks. Sister: the escort (skin 0x0500002e, the other ch-185 waiter) never
/// enters 0x35 — the mission route is the driver's (mode 0x17), the escort is a team-1 guard
/// whose wake chain is PlayScript/resume, never the driver states [M 0x15b830 family gate].
#[test]
fn driver_walks_his_mission_route_after_the_sit_script() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut w = Walker::new(nf_view::load_with_carry(&iso, "07000006.bin", false, Some(&carry(&[0x1d, 0x45, 0x70]))).expect("load 07000006"));
    w.wait_enabled(4000, "the intro NIS");
    w.wait(300);
    let di = driver_index(w.scene.sim.as_ref().unwrap()).expect("the driver is present");
    let ei = escort_index(w.scene.sim.as_ref().unwrap()).expect("the escort is present");
    let park = w.scene.sim.as_ref().unwrap().drones.list[di].pos;
    eprintln!("[start] driver #{di} state {:#x} pos ({:.1},{:.1},{:.1}) route {}",
        w.scene.sim.as_ref().unwrap().drones.list[di].state, park[0], park[1], park[2],
        w.scene.sim.as_ref().unwrap().drones.list[di].route.as_ref().map(|r| format!("p{} {}/{}", r.path, r.index, r.nodes.len())).unwrap_or("-".into()));

    // The arrival plates: walk forward until ch 185 (0xB9) rises, then stand (the greeting).
    let mut input = Input::default();
    input.forward = 1.0;
    let mut ch185_at = None;
    for t in 0..1200u32 {
        w.tick(input, false);
        if w.scene.sim.as_ref().unwrap().drones.channels.on(0xB9) { ch185_at = Some(t); break; }
    }
    let ch185_at = ch185_at.expect("the plates raise ch 185");
    eprintln!("[plates] ch 0xB9 at +{ch185_at}");

    // The watch: the sit script (880 frames [L 05-truck.trace]) ends → 0x35 → the walk.
    let mut mission_at = None;
    let mut walked_at = None;
    for t in 0..12_000u32 {
        w.tick(Default::default(), false);
        let sim = w.scene.sim.as_ref().unwrap();
        let drv = &sim.drones.list[di];
        if mission_at.is_none() && drv.state == 0x35 { mission_at = Some(t); }
        if walked_at.is_none() && (drv.pos[0] - park[0]).hypot(drv.pos[2] - park[2]) > 2.0 { walked_at = Some(t); }
        if t % 1500 == 0 {
            eprintln!("[t +{t}] driver state {:#x} pos ({:.2},{:.2},{:.2}) route {}", drv.state, drv.pos[0], drv.pos[1], drv.pos[2],
                drv.route.as_ref().map(|r| format!("p{} i{}/{}", r.path, r.index, r.nodes.len())).unwrap_or("-".into()));
        }
        if mission_at.is_some() && walked_at.is_some() { break; }
    }
    let sim = w.scene.sim.as_ref().unwrap();
    let esc = &sim.drones.list[ei];
    let esc_mission = esc.state == 0x35;
    eprintln!("[end] driver state {:#x} walked_at {walked_at:?} mission_at {mission_at:?} | escort #ei state {:#x}",
        sim.drones.list[di].state, esc.state);
    let mission_at = mission_at.expect("the driver enters TruckDriverMission (0x35) after the sit script — the game's 0x160bd0 case 5");
    let walked_at = walked_at.expect("the driver walks his mission route (>2 units from the park spot)");
    assert!(walked_at > mission_at.saturating_sub(60), "the walk begins with/after the mission enter, not before");
    // Sister: the escort never enters the driver's mission state.
    assert!(!esc_mission, "the escort (team-1 guard) must never enter TruckDriverMission");
}
