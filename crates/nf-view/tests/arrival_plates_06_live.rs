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

#[path = "common/walker.rs"]
#[allow(dead_code)]
mod walker;

/// Bond's collide segment bottom over a flat plate, `Player_Collision` 0x194df0's standing arm:
/// B = pos + 0.55 − cc [M]; the raw refine touches while the plate is within the radius 0.55 of B
/// [M 0x109da0:419, :523-531 via `Dist2Tri` 0x109770's distance].
fn raw_reach(pos_y: f32, cc: f32) -> f32 { pos_y + 0.55 - cc - 0.55 }

/// The balcony drop into the searchlight courtyard: walked off head #2's balcony (floor 46.0, Bond
/// 1.1 over it) west over the Spotlight Trigger poly #664 (y 45.93 [D]), the fall raises ch 0x34 on
/// the first tick Bond's segment comes within 0.55 of the plate, while he is still about a metre up.
/// The measured box (pos − 0.6) touched it only half a metre lower (pos 46.46).
#[test]
fn the_balcony_drop_touches_664_at_the_segment_reach() {
    let Ok(iso) = std::env::var("NF_ISO") else { eprintln!("NF_ISO not set"); return };
    let mut w = walker::Walker::new(nf_view::load_with_carry_difficulty(&iso, "07000006.bin", false, Some(&carry(&[0x1d, 0x45])), 1).expect("load"));
    for _ in 0..30 { w.tick(nf_game::player::Input::default(), false); }
    {
        let sim = w.scene.sim.as_mut().unwrap();
        assert!(!sim.drones.channels.on(0x34), "the lamps are dark on arrival");
        sim.bond = nf_game::player::Bond::stand_at(&sim.world, [10.48, 47.04, -23.05], 0.0);
    }
    w.rise_watch = vec![0x34];
    let _ = w.try_walk_to(walker::Wp::new(3.0, 41.0, -23.0).r(1.5), "the balcony's west end");
    eprintln!("on the west end at {:?}", w.scene.sim.as_ref().unwrap().bond.pos);
    if w.rises.is_empty() { let _ = w.try_walk_to(walker::Wp::new(6.8, 42.0, -17.1).r(1.0), "down into the courtyard"); }
    let first = w.rises.first().cloned().expect("the drop raises ch 0x34");
    assert_eq!(first.3, vec![664], "the drop's plate is #664");
    let cc = w.scene.sim.as_ref().unwrap().bond.cc;
    let plate = 45.93;
    assert!(raw_reach(first.2[1], cc) <= plate + 0.01, "touched inside the segment reach: {:?}", first.2);
    assert!(first.2[1] > 46.8, "touched high in the drop, not at the box reach (pos 46.46): {:?}", first.2);
}

/// Sister: standing on the balcony beside #664 (floor 46.0, its west edge x ~7) stays out of reach:
/// the segment bottom less the radius sits at 46.07, over the plate's 45.93.
#[test]
fn standing_on_the_balcony_leaves_664_alone() {
    let Ok(iso) = std::env::var("NF_ISO") else { eprintln!("NF_ISO not set"); return };
    let mut s = arrive(&iso, 0x1d, 30);
    let sim = s.sim.as_mut().unwrap();
    sim.bond = nf_game::player::Bond::stand_at(&sim.world, [8.0, 47.1, -23.0], 0.0);
    for _ in 0..120 { sim.step_bond(nf_game::player::Input::default(), false, false, false); }
    assert!((sim.bond.pos[1] - 47.1).abs() < 0.2, "Bond stands on the balcony: {:?}", sim.bond.pos);
    assert!(!sim.drones.channels.on(0x34), "no ch-0x34 plate under a Bond standing on the balcony");
}

/// The tunnel stair at the ramp's foot: the flat plates #674 (y 42.66, ch 0x38 → OR #708 → ch 0x2f,
/// the sound trigger #665) and #524 (y 42.44, ch 0x34) share one footprint over the 42.0/42.2/42.4
/// steps [D]. Standing on the 42.4 step puts #674 0.26 over the floor, inside the segment's reach
/// (B less the radius sits about 0.06 over the floor), and #524 0.04 over it, outside. So the stand
/// raises ch 0x38 and leaves ch 0x34 alone.
#[test]
fn the_top_tunnel_step_touches_674_and_not_524() {
    let Ok(iso) = std::env::var("NF_ISO") else { eprintln!("NF_ISO not set"); return };
    let mut s = arrive(&iso, 0x1e, 30);
    let sim = s.sim.as_mut().unwrap();
    assert!(!sim.drones.channels.on(0x38) && !sim.drones.channels.on(0x34), "both plates cold on arrival");
    sim.bond = nf_game::player::Bond::stand_at(&sim.world, [4.01, 43.4, -53.88], 0.0);
    for _ in 0..30 { sim.step_bond(nf_game::player::Input::default(), false, false, false); }
    let floor = sim.world.floor_below([4.01, 43.0, -53.88], 2.0).expect("the step");
    assert!((floor - 42.4).abs() < 0.02, "Bond stands on the 42.4 step: floor {floor}, pos {:?}", sim.bond.pos);
    assert!(raw_reach(sim.bond.pos[1], sim.bond.cc) < 42.66 && raw_reach(sim.bond.pos[1], sim.bond.cc) > 42.44, "the reach sits between the plates");
    assert!(sim.drones.channels.on(0x38), "#674 raises ch 0x38 under the standing Bond");
    assert!(!sim.drones.channels.on(0x34), "#524 sits under the reach");
}
