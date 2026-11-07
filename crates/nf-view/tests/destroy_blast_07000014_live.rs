//! A destroyable's blast on 07000014: `Destroy_Smash` hands `Explode_Create` the radius in f13
//! (tag+8 = p5 × 0.01) and the damage in f14 (the type byte × 10.0) [M 0x12bcdc-0x12bd0c]. The
//! generator boxes' group-1 leaves #714/#713 carry type 0x32 and p5 1000: 500 damage over a
//! 10-unit radius, the price of the wrong wire [D]. The barrels are type 3, p5 250: 30 over 2.5.
//!
//! Opt-in via NF_ISO.

/// The main-map index of an audit row (docs/levels/07000014-audit.md numbers across all maps).
const ROW_BASE: usize = 326;
/// The +0xAC default Rook keeps [M 0x14b300:52; D #777 no stats section].
const ROOK_STATS: f32 = 10.0;

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

/// The falloff is a straight line to Bond's sphere edge, no integer step [M 0x17c5f8-0x17c614; L
/// obj+0x8C 1.0576]: the east box #620 (type 4, p5 400: 40 over 4.0), Bond a known way off.
#[test]
fn a_blast_falls_off_in_a_straight_line_to_bonds_sphere() {
    let Some(mut scene) = loaded() else { eprintln!("skip: NF_ISO not set"); return };
    let sim = scene.sim.as_mut().unwrap();
    let i = leaf(sim, 620);
    let d = &sim.drones.props.destroy[i];
    let c = [(d.lo[0] + d.hi[0]) / 2.0, (d.lo[1] + d.hi[1]) / 2.0, (d.lo[2] + d.hi[2]) / 2.0];
    assert_eq!((d.blast_radius, d.blast_damage()), (4.0, 40.0), "[D p2 4, p5 0x190]");
    stand(sim, [c[0] + 2.5, c[1], c[2]]);
    for _ in 0..30 { sim.step_bond(Default::default(), false, false, false); }
    let frame = sim.drones.frame;
    sim.drones.channels.set(0xe1, 1, frame);
    let before = sim.drones.health.health;
    // The box smashes at once; the blast lands the same tick, the channel 30 ticks on.
    let (mut sph, mut lost) = ([0.0f32; 3], 0.0f32);
    for _ in 0..4u32 {
        let sim = scene.sim.as_mut().unwrap();
        sph = sim.bond.sphere_centre();
        sim.step_bond(Default::default(), false, false, false);
        lost = before - sim.drones.health.health;
        if lost > 0.0 { break; }
    }
    let gap = ((sph[0] - c[0]).powi(2) + (sph[1] - c[1]).powi(2) + (sph[2] - c[2]).powi(2)).sqrt() - 1.0576;
    let sim = scene.sim.as_ref().unwrap();
    let want = 40.0 * (1.0 - gap / 4.0) * nf_game::bullet::damage_factor(0x0700_0014, sim.drones.difficulty);
    assert!((lost - want).abs() < 0.2, "gap {gap}: lost {lost}, the line gives {want}");
}

/// `NDrone2_DefaultInit`: Rook (kind 0x10 on this level) stands three times his stats' health
/// [M 0x14b300:708-713, :776], and his head takes the torso's double, not the head's multiple.
#[test]
fn rook_stands_three_times_his_stats_health() {
    let Some(scene) = loaded() else { eprintln!("skip: NF_ISO not set"); return };
    let sim = scene.sim.as_ref().unwrap();
    let rook = sim.drones.list.iter().find(|d| d.skin == 0x0500_0066).expect("Rook [D]");
    assert_eq!(rook.kind, 0x10);
    assert_eq!((rook.health, rook.max_health), (ROOK_STATS * 3.0, ROOK_STATS * 3.0));
}

/// `NDrone2_DefaultInit`'s close-in on this level: no guard here engages from past 10 units
/// [M 0x14b300:441-460; D the level's anim sets].
#[test]
fn the_guards_close_to_ten_units_on_this_level() {
    let Some(scene) = loaded() else { eprintln!("skip: NF_ISO not set"); return };
    let sim = scene.sim.as_ref().unwrap();
    let mut n = 0;
    for d in sim.drones.list.iter().filter(|d| d.anim_set != 5) {
        assert_eq!(d.engage_range, if d.anim_set == 9 { 7.0 } else { 10.0 }, "guard {} anim set {}", d.index, d.anim_set);
        n += 1;
    }
    assert!(n > 10, "{n} guards read");
}

/// `DroneVision_AlertSound` 0x177800:35-37 on the live level: a guard walking his mission route
/// does not break off for a noise; off the route the same guard hears it.
#[test]
fn a_guard_on_his_mission_route_keeps_walking_through_a_noise() {
    let Some(mut scene) = loaded() else { eprintln!("skip: NF_ISO not set"); return };
    let sim = scene.sim.as_mut().unwrap();
    let i = sim.drones.list.iter().position(|d| d.skin == 0x0500_0087).expect("a guard [D]");
    let mut guard = sim.drones.list[i].clone();
    guard.dist = 10.0;
    guard.sound_share = 0.5;
    let (d, mut rng) = (sim.data(), nf_game::drone::Rng::new(1));
    let mut on_route = guard.clone();
    on_route.state = nf_game::drone::state::ENEMY_MISSION;
    on_route.send(&d, &mut rng, 100, nf_game::drone::msg::SOUND);
    assert_eq!(on_route.state, nf_game::drone::state::ENEMY_MISSION);
    let mut idle = guard.clone();
    idle.state = nf_game::drone::state::IDLE;
    idle.send(&d, &mut rng, 100, nf_game::drone::msg::SOUND);
    assert_ne!(idle.state, nf_game::drone::state::IDLE, "sister: the guard can hear");
}
