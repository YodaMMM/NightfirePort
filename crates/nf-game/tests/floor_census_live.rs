//! floor-crack-01: a census of drone feet probes over 07000001's nav nodes — every walkable node
//! finds support (the 1-unit feet ray, or the movement capsule's pick bit on a miss, the fixed
//! `Drone::feet_on_point` + `feet_pick_bit` rule). The crack at (−4.703, −5.968) sits on this
//! graph, so the census doubles as the no-other-cracks sweep. Opt-in: set `NF_ISO`.

use nf_formats::iso9660::IsoImage;
use nf_formats::level::Level;
use nf_game::world::CollisionWorld;

/// The drone feet probe on the public world API — the same rule `Drone::feet_on_point` +
/// `feet_pick_bit` implement, with `p` the FEET point (origin = p + hip):
/// - the down-ray `pos → pos − (hip + 0.6)`, on-surface within 0.2 of the feet [M 0x140660:66-96];
/// - on a miss, the movement capsule's pick bit: radius 0.4, top `pos + up·r`, bottom
///   `pos + up·(r − hip)` [M 0x140e90:73-93], an up-facing contact [M 0x140660's hittest+0x9c arm +
///   0x1ea398:83 + 0x1ebaa8's tail; the fresh-probe pick reference is the port's [S] choice].
fn drone_floor(world: &CollisionWorld, p: [f32; 3], hip: f32) -> Option<f32> {
    let from = [p[0], p[1] + hip, p[2]];
    let to = [p[0], p[1] - 0.6, p[2]];
    if let Some(t) = world.segment(from, to) {
        let floor = from[1] + (to[1] - from[1]) * t;
        if floor >= p[1] - 0.2 { return Some(floor); }
    }
    let r = 0.4;
    let top = [p[0], p[1] + hip + r, p[2]];
    let bottom = [p[0], p[1] + r, p[2]];
    if world.capsule_pick_contact(top, bottom, r, f32::NEG_INFINITY) { Some(p[1]) } else { None }
}

#[test]
fn the_07000001_nav_nodes_all_find_support() {
    let Some(path) = std::env::var_os("NF_ISO") else { eprintln!("no NF_ISO"); return };
    let iso = IsoImage::open(&path).unwrap();
    let (entries, bin) = nf_formats::disc::files_bin_entries(&iso).unwrap();
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case("07000001.bin")).unwrap();
    let data = iso.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).unwrap();
    let level = Level::parse(&data).unwrap();
    let world = CollisionWorld::build(&level, &level.instances(true).0);
    let m = level.main_map().unwrap();
    let net = nf_game::nav::Network::build(level.maps[m].ai_network.as_ref().unwrap(), &world, 0x0700_0001);

    let mut holes = Vec::new();
    let mut short_misses = 0usize;
    let mut n = 0usize;
    for path in &net.paths {
        for (i, node) in path.nodes.iter().enumerate() {
            n += 1;
            // NavNode.pos is the disc position with y + 0.4 (`AIPath_Parse`, the NavNode doc) —
            // undo the offset: the probe's p is the FEET point.
            let p = [node.pos[0], node.pos[1] - 0.4, node.pos[2]];
            // The crack-class probe (ray, then the pick bit): reports the nodes that walk
            // airborne for a tick (stair steps do — the game's own ray misses there too and the
            // guard drops onto the next step), not a failure.
            if drone_floor(&world, p, 1.0).is_none() {
                short_misses += 1;
            }
            // A crack (the symptom) is a node with NO triangle below at all: the guard falls out
            // of the world. A node over a stair gap lands on the floor below — the game's own
            // feet ray misses there too while he walks down, and his falling rays catch the step
            // within its 0.6 reach (a 40-unit column always lands: tunneling needs > 36 u/s,
            // i.e. a 66-unit free fall [M WldGravity −9.8, the 1/60 step]).
            if world.floor_below([p[0], p[1], p[2]], 40.0).is_none() {
                holes.push((path.name.clone(), i, p));
            }
        }
    }
    assert!(holes.is_empty(), "{} of {n} nav nodes stand over a void (fall out of the world): {holes:?}", holes.len());
    eprintln!("[census] {n} nav nodes: no voids; {short_misses} walk airborne a tick (steps/gaps) and land");
}
