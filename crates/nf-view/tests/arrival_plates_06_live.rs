//! The castle arrival on 07000006: every route into the level raises ch 0x46 through OR #498
//! [D #495/#766 → 0x50, #497/#743-745 → 0x51, #767-769 → 0x52], and ch 0x46 completes 07000005's
//! "Breach the castle walls" and reveals "Find a way inside to the party" [L ee_start_05.bin record
//! 0/1]. The upper-door route's plates #495/#766 are flat quads at y 49.56 under start #511, 0.68
//! below where `Player_StandAtNewPosition` 0x190d88 stands Bond: past the measured box, inside the
//! game's raw reach [M 0x194df0 segment, 0x109da0 refine]. The tunnel's ch-0x34 plate #524 (y 42.44
//! at the foot of the ramp behind door #71) is the same case. Opt-in: NF_ISO.

use nf_game::player::RamSave;
use nf_view::scene::Sim;

fn carry(on: &[u32]) -> RamSave {
    let mut ch = nf_game::triggers::Channels::default();
    for &c in on { ch.set(c, 1, 0); }
    RamSave::capture(&Default::default(), 0, &nf_game::pickup::Inventory::level_start(0x73, 240), &ch)
}

fn row(level: u32) -> usize { nf_game::mission::LEVELS.iter().position(|l| l.level == level).expect("the record") }

/// Lands on 07000006 by the route `gate` and runs the first second of play.
fn arrive(iso: &str, gate: u32, ticks: u32) -> nf_view::scene::Scene {
    let mut s = nf_view::load_with_carry_difficulty(iso, "07000006.bin", false, Some(&carry(&[gate, 0x45])), 1).expect("load");
    for _ in 0..ticks { s.sim.as_mut().unwrap().step_bond(nf_game::player::Input::default(), false, false, false); }
    s
}

fn sim(s: &nf_view::scene::Scene) -> &Sim { s.sim.as_ref().unwrap() }

#[test]
fn every_route_in_breaches_the_walls() {
    let Ok(iso) = std::env::var("NF_ISO") else { eprintln!("NF_ISO not set"); return };
    for (gate, plate) in [(0x1du32, 0x51u32), (0x1f, 0x52), (0x1e, 0x50)] {
        let s = arrive(&iso, gate, 120);
        let sim = sim(&s);
        assert!(sim.drones.channels.on(plate), "route {gate:#x}: plate channel {plate:#x} rises under the arrival");
        assert!(sim.drones.channels.on(0x46), "route {gate:#x}: OR #498 raises ch 0x46");
        assert_eq!(sim.drones.mission.states[row(0x0700_0005)][0], 4, "route {gate:#x}: the breach objective completes");
        assert_eq!(sim.drones.mission.states[row(0x0700_0006)][0], 3, "route {gate:#x}: the party objective is revealed");
    }
}

/// Sister: each route lights its own plate family and no other.
#[test]
fn the_door_route_leaves_the_upper_plates_alone() {
    let Ok(iso) = std::env::var("NF_ISO") else { eprintln!("NF_ISO not set"); return };
    let s = arrive(&iso, 0x1f, 120);
    let sim = sim(&s);
    assert!(!sim.drones.channels.on(0x50), "ch 0x50 is the upper door's own plate pair");
    assert!(!sim.drones.channels.on(0x51), "ch 0x51 is the truck's own plate set");
}

/// The tunnel's reveal plate: walked down the ramp to door #71, Bond crosses #524 at the ramp's
/// foot and the courtyard lamps' channel rises, the way the upper gallery comes down to them.
#[test]
fn the_tunnel_plate_fires_on_the_walk_down() {
    let Ok(iso) = std::env::var("NF_ISO") else { eprintln!("NF_ISO not set"); return };
    let mut s = arrive(&iso, 0x1e, 30);
    let sim = s.sim.as_mut().unwrap();
    assert!(!sim.drones.channels.on(0x34), "the lamps are dark on arrival");
    sim.bond = nf_game::player::Bond::stand_at(&sim.world, [4.4, 44.5, -54.5], 0.0);
    let mut low = sim.bond.pos;
    for _ in 0..240 {
        sim.step_bond(nf_game::player::Input { forward: 1.0, ..Default::default() }, false, false, false);
        if sim.bond.pos[1] < low[1] { low = sim.bond.pos; }
        if sim.drones.channels.on(0x34) { break; }
    }
    assert!(sim.drones.channels.on(0x34), "plate #524 raises ch 0x34; lowest stand {low:?}, end {:?}", sim.bond.pos);
}
