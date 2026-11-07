//! A destroyable's blast on 07000014: `Destroy_Smash` hands `Explode_Create` the radius in f13
//! (tag+8 = p5 × 0.01) and the damage in f14 (the type byte × 10.0) [M 0x12bcdc-0x12bd0c]. The
//! generator boxes' group-1 leaves #714/#713 carry type 0x32 and p5 1000: 500 damage over a
//! 10-unit radius, the price of the wrong wire [D]. The barrels are type 3, p5 250: 30 over 2.5.
//!
//! Opt-in via NF_ISO.

/// The main-map index of an audit row (docs/levels/07000014-audit.md numbers across all maps).
const ROW_BASE: usize = 326;

fn loaded() -> Option<nf_view::scene::Scene> {
    let iso = std::env::var("NF_ISO").ok()?;
    let mut scene = nf_view::load(&iso, "07000014.bin", false).expect("load 07000014");
    for _ in 0..400 {
        if !scene.sim.as_ref().unwrap().disabled { break; }
        scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false);
    }
    Some(scene)
}

fn leaf(sim: &nf_view::scene::Sim, row: usize) -> usize {
    sim.drones.props.destroy.iter().position(|d| d.placement == row - ROW_BASE)
        .unwrap_or_else(|| panic!("destroy row #{row} [D]"))
}

fn stand(sim: &mut nf_view::scene::Sim, at: [f32; 3]) {
    let t = sim.world.segment(at, [at[0], at[1] - 30.0, at[2]]).expect("a floor under the stance");
    sim.bond.pos = [at[0], at[1] - 30.0 * t, at[2]];
}

#[test]
fn the_wrong_wire_at_a_generator_box_kills_bond_where_he_stands() {
    let Some(mut scene) = loaded() else { eprintln!("skip: NF_ISO not set"); return };
    let sim = scene.sim.as_mut().unwrap();
    let i = leaf(sim, 714);
    assert_eq!((sim.drones.props.destroy[i].kind, sim.drones.props.destroy[i].blast_radius), (0x32, 10.0), "#714's type and radius [D p2, p5]");
    let fb = sim.drones.fuseboxes.iter().find(|f| f.ch[1] == 0x71).expect("box #709 [D]").pos;
    stand(sim, [fb[0] + 0.6, fb[1] - 1.2, fb[2]]);
    let frame = sim.drones.frame;
    sim.drones.channels.set(0x71, 1, frame); // the group-1 word the cut writes [M 0x1adfe8:110-123]
    let mut dead_at = None;
    for t in 0..30u32 {
        let sim = scene.sim.as_mut().unwrap();
        sim.step_bond(Default::default(), false, false, false);
        if sim.drones.health.health <= 0.0 { dead_at = Some(t); break; }
    }
    let sim = scene.sim.as_ref().unwrap();
    assert!(sim.drones.props.destroy[i].dead && sim.drones.channels.on(0xdb), "the box blew and ch 0xdb rose");
    assert!(dead_at.is_some(), "500 over 10 units at the box takes all of Bond: health {}", sim.drones.health.health);
}

/// Sister: the right wire writes the channel the sabotage reads and nothing blows.
#[test]
fn the_right_wire_leaves_bond_standing() {
    let Some(mut scene) = loaded() else { eprintln!("skip: NF_ISO not set"); return };
    let sim = scene.sim.as_mut().unwrap();
    let fb = sim.drones.fuseboxes.iter().find(|f| f.ch[1] == 0x71).expect("box #709 [D]").pos;
    stand(sim, [fb[0] + 0.6, fb[1] - 1.2, fb[2]]);
    let before = sim.drones.health.health;
    let frame = sim.drones.frame;
    sim.drones.channels.set(0x87, 1, frame);
    for _ in 0..30u32 { scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false); }
    let sim = scene.sim.as_ref().unwrap();
    assert!(!sim.drones.props.destroy[leaf(sim, 714)].dead && !sim.drones.channels.on(0xdb));
    assert_eq!(sim.drones.health.health, before);
}

/// The hangar box #1249 (type 3, p5 500: 30 over 5.0) takes the barrels 3.4 away with it: the
/// falloff there is 12, past case 0x21's 1.0 gate [M 0x17c518], and each barrel blasts for 30
/// over 2.5. Bond 1.5 from the near barrel and out of the box's radius loses the barrel's share.
#[test]
fn a_barrel_going_up_beside_bond_hurts_at_the_games_scale() {
    let Some(mut scene) = loaded() else { eprintln!("skip: NF_ISO not set"); return };
    let sim = scene.sim.as_mut().unwrap();
    let (near, box_leaf) = (leaf(sim, 1054), leaf(sim, 1249));
    assert_eq!((sim.drones.props.destroy[near].blast_radius, sim.drones.props.destroy[near].blast_damage()), (2.5, 30.0), "the barrel [D p2 3, p5 0xfa]");
    let p = sim.drones.props.destroy[near].pos;
    stand(sim, [p[0] + 1.1, p[1] + 1.0, p[2] - 1.0]);
    let before = sim.drones.health.health;
    let frame = sim.drones.frame;
    sim.drones.channels.set(0x0d, 1, frame);
    for _ in 0..12u32 { scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false); }
    let sim = scene.sim.as_ref().unwrap();
    assert!(sim.drones.props.destroy[box_leaf].dead && sim.drones.props.destroy[near].dead, "the box took the barrel");
    let lost = before - sim.drones.health.health;
    assert!(lost >= 10.0, "the barrel's blast reaches Bond in full scale: lost {lost}");
}

/// `DroneSpawner_Create` converts p8 with `cvt.s.w` and defaults 0 to 10.0 [M 0x1379c0-0x137a1c]:
/// the wave near the Delta doors keeps 10 units off Bond [D #803 p8 0xa, #1346 p8 0, #599 p8 1].
#[test]
fn the_spawners_keep_their_disc_distance_from_bond() {
    let Some(scene) = loaded() else { eprintln!("skip: NF_ISO not set"); return };
    let sim = scene.sim.as_ref().unwrap();
    let dist = |row: usize| sim.drones.spawners.iter().find(|s| s.placement == row - ROW_BASE)
        .unwrap_or_else(|| panic!("spawner row #{row} [D]")).min_dist;
    assert_eq!((dist(803), dist(1346), dist(599)), (10.0, 10.0, 1.0));
}
