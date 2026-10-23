//! cover-nodes (the sub-map half answered by cover-nodes-submap): the AI cover node
//! placements (0xE5 corner / 0xE6 low) become live nodes from the MAIN MAP's placement block only
//! — the way the game creates them — and a 07000006 guard under fire takes one of the game's own
//! cover nodes instead of standing in the open.
//!
//! Evidence: `ResetMap_Load` 0x1d6360 creates the live dynamic objects in ONE
//! `parsemap_block_map_data_dynamic(&DynamicBH, DynamicPtr, 1)` call (the only flag-1 caller);
//! `DynamicPtr` (0x30cff0) is reset to 0 at every map file's open (`parsemap_parsemap` 0x1d1cf0
//! state 0, disasm 0x1d1dc8/0x1d1dd0) and records each file's block 0x1a
//! (`parsemap_handle_block_id` 0x1d1020 case 0x1a, disasm 0x1d121c) — so the create runs over the
//! LAST-parsed map entry's placements. Every mission archive lists its kind-1 main map LAST
//! [D — docs/systems/cover-nodes-load.md], so the live list is the main map's 0xE5/0xE6
//! placements in placement order; the shared sub-map 010000f3's template pair is never
//! instantiated. An earlier "every map entry" claim contradicted this chain — the tests below
//! pin the proven behaviour.
//! Opt-in: `NF_ISO`.

use nf_view::dev::Dev;
use std::f32::consts::FRAC_PI_4;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

fn settle(scene: &mut nf_view::scene::Scene, ticks: u32) {
    for _ in 0..3000 { scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false); if !scene.sim.as_ref().unwrap().disabled { break; } }
    for _ in 0..ticks { scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false); }
}

/// The live cover list is the main map's 0xE5/0xE6 placements in placement order
/// ([M] ResetMap_Load 0x1d6360's single flag-1 create over `DynamicPtr` — the last-parsed map
/// entry's block 0x1a, the main map on every mission archive [D — docs/systems/cover-nodes-load.md]).
/// 07000006's main map 01000098 holds 80 corner + 34 low cover placements [D placement bytes,
/// docs/systems/script-census.md census]; its 010000f3 sub-map holds 1+1 more that the game never
/// instantiates — a level-wide scan would make 81+35, and this fails 114 != 116
/// if the scan ever comes back.
#[test]
fn cover_06_spawns_the_main_map_s_nodes() {
    let mut scene = nf_view::load(&iso(), "07000006.bin", false).expect("load 07000006");
    settle(&mut scene, 5);
    let sim = scene.sim.as_ref().unwrap();
    let corners = sim.covers.iter().filter(|n| n.is_corner()).count();
    assert_eq!(corners, 80, "corner nodes = the main map's 0xE5 placements [D]");
    assert_eq!(sim.covers.len() - corners, 34, "low nodes = the main map's 0xE6 placements [D]");
    assert!(!sim.covers.iter().any(|n| n.pos == [0.0, 0.0, 0.0]),
        "a (0,0,0) template node is live — the sub-map scan came back [M parsemap_handle_block_id 0x1d1020:82-84 (the last block 0x1a wins) + the single flag-1 create at ResetMap_Load 0x1d6360:146]");
    // Sister [D]: the disc really carries 81+35 across all maps (the scan's premise) — the two
    // extra are 010000f3's template pair (cover_nodes_submap_live pins them per level).
    let iso_img = nf_formats::iso9660::IsoImage::open(&iso()).unwrap();
    let (entries, bin) = nf_formats::disc::files_bin_entries(&iso_img).unwrap();
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case("07000006.bin")).unwrap();
    let data = iso_img.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).unwrap();
    let level = nf_formats::level::Level::parse(&data).unwrap();
    let all: usize = level.maps.iter().map(|m| m.placements.iter()
        .filter(|p| matches!(p.flags, 0xe5 | 0xe6)).count()).sum();
    assert_eq!(all, 116, "0xE5/0xE6 placements across every map entry [D] — the scan's premise");
}

/// The shared sub-map 010000f3 carries the disc's template pair — "Cover Corner Node" and
/// "Cover Low Node", both at the placement origin with no parameters ([D] placement bytes) —
/// and the game NEVER instantiates it: the create runs over the last-parsed map entry's block
/// only ([M] ResetMap_Load 0x1d6360; the per-file `DynamicPtr = 0` reset in `parsemap_parsemap`
/// 0x1d1cf0 state 0), and every mission archive's kind-1 main map is parsed last. 07000004's
/// main map has NO cover placements, so its live list is EMPTY. The template's field mapping
/// through the creator's default arms is pinned on the disc placement itself: corner flags =
/// 0xC | 0x60 | 0x180 = 0x1EC with angle 45° = π/4 ([M NDrone2_FindAmbushCover 0x152958:127/149 — the ×0.1 scale; 0x136ff0 — p0..p2 not 1/2 take the else
/// arms, p3 = 0 → 0x3f490fdb]) and depth/offset bytes default 8 × 0.1; low: p0 = 0 → idle 0x11,
/// aim 0xA, no flags ([M 0x137340]).
#[test]
fn the_010000f3_template_pair_never_spawns_live() {
    let iso_img = nf_formats::iso9660::IsoImage::open(&iso()).unwrap();
    let (entries, bin) = nf_formats::disc::files_bin_entries(&iso_img).unwrap();
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case("07000004.bin")).unwrap();
    let data = iso_img.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).unwrap();
    let level = nf_formats::level::Level::parse(&data).unwrap();
    let f3 = level.maps.iter().find(|m| m.name == "010000f3").expect("the shared sub-map is in the archive [D]");
    let corner_p = f3.placements.iter().find(|p| p.flags == 0xe5).expect("template corner placement [D]");
    let low_p = f3.placements.iter().find(|p| p.flags == 0xe6).expect("template low placement [D]");
    assert_eq!(corner_p.translation, [0.0, 0.0, 0.0], "the corner template sits at the origin [D]");
    assert!(corner_p.params.is_empty() && low_p.params.is_empty(), "the templates carry no params [D]");
    // The port's mapping of the creator's default arms, pinned on the disc placement [M 0x136ff0/0x137340].
    let corner = nf_game::cover::CoverNode::from_placement(corner_p).expect("corner template maps");
    assert_eq!(corner.flags, 0x1ec);
    assert!((corner.angle - FRAC_PI_4).abs() < 1e-6, "angle {}", corner.angle);
    assert!((corner.depth - 0.8).abs() < 1e-6 && (corner.offset - 0.8).abs() < 1e-6);
    assert_eq!((corner.yaw.to_degrees().round() as i64) % 360, 0);
    let low = nf_game::cover::CoverNode::from_placement(low_p).expect("low template maps");
    assert_eq!(low.flags & 3, 0, "p0 = 0 → no crouch-only / flag-2 bits");
    assert_eq!(low.idle_anim, 0x11);
    assert_eq!(low.aim_anim, 0xa);
    // And the live list: 07000004's main map 010000c5 has no 0xE5/0xE6 placements [D], so the
    // game — and the port — create zero nodes there.
    let mut scene = nf_view::load(&iso(), "07000004.bin", false).expect("load 07000004");
    settle(&mut scene, 5);
    let sim = scene.sim.as_ref().unwrap();
    assert_eq!(sim.covers.len(), 0,
        "07000004 spawns no live cover nodes — the template pair is never instantiated [M 0x1d6360]");
}

/// A 07000006 guard under fire takes one of the game's own cover nodes: the sound of the shots
/// drives him through the heard-noise chain into combat, `DroneFunc_CombatState` enters
/// RunForCover 0x8C when `NDrone2_CoverAvailable` holds [M docs/systems/drone-cover.md §States],
/// and he ends at the node's in-cover point (UnderCoverIdle 0x94 leaves when "> 1.5 from the
/// goal" [M 0x94 update] — a drone STAYING in 0x94 is within 1.5 of `cover_point`).
#[test]
fn a_06_guard_under_fire_takes_a_game_cover_node() {
    let mut scene = nf_view::load(&iso(), "07000006.bin", false).expect("load 07000006");
    settle(&mut scene, 30);
    // Choose the target through the real data: a live guard with the cover prop 0x30 (the prop
    // that lets CombatState enter RunForCover [M docs/systems/drone-cover.md §07000005]) with a
    // usable corner node near him and clear air between Bond's eye and his chest.
    let (_di, spot) = {
        let sim = scene.sim.as_ref().unwrap();
        let data = nf_game::drone::Data {
            tables: &sim.tables, props: &sim.props, scripts: &sim.scripts, weapons: &sim.weapons,
            world: &sim.world, level: sim.level, sequences: &sim.seqs, skins: &sim.skins,
            skeletons: &sim.skeletons, nav: &sim.nav, covers: &sim.covers, tuning: &sim.tuning,
            hitboxes: &sim.hitboxes, volumes: &sim.volumes, sfx: sim.sfx.as_ref(), models: &sim.models,
            surfaces: sim.surfaces.as_ref(), voices: sim.voices.as_deref(), difficulty: sim.drones.difficulty, ai_points: &sim.ai_points,
        };
        let mut best: Option<(usize, [f32; 3])> = None;
        let mut best_score = f32::INFINITY;
        for (ix, &li) in sim.drones.llist.iter().enumerate() {
            let d = &sim.drones.list[li];
            if !d.enabled || !d.present || matches!(d.state, 0x44 | 0x46 | 0x47 | 0x48 | 0x49) { continue; }
            if !d.prop(&data, 0x30) { continue; }
            if d.cover.is_some() { continue; }
            // Bond stands 22 units out on the FLOOR (a downward cast from 6 above the guard to 30
            // below; the mid-air teleport dropped him forever), within 6 of the guard's height,
            // with a clear eye→chest segment, IN FRONT of the guard (within ~75° of his facing —
            // [S] reliability cone so the sighting happens), and with a corner node within 25
            // units of the guard that is (a) reachable from the guard's nav node (dist byte
            // ≠ 0xff — FindCover's emitter gate [M 0x152548]) and (b) usable from the REAL Bond
            // spot with side bits, the way Drone_ProcessCoverNodes computes them.
            let feet = d.feet();
            let gpath = d.nav_path.unwrap_or_else(|| sim.nav.nav_path_for(feet));
            let gnode = gpath.and_then(|p| sim.nav.node_from(&sim.world, p, feet));
            let (Some(_), Some(gfrom)) = (gpath, gnode) else { continue };
            let mut spot: Option<[f32; 3]> = None;
            for dir in 0..12 {
                let a = dir as f32 * std::f32::consts::TAU / 12.0;
                let (px, pz) = (d.pos[0] + a.cos() * 22.0, d.pos[2] + a.sin() * 22.0);
                let (top, bot) = ([px, d.pos[1] + 6.0, pz], [px, d.pos[1] - 30.0, pz]);
                let Some(t) = sim.world.segment(top, bot) else { continue };
                let y = top[1] + t * (bot[1] - top[1]);
                if (y - d.pos[1]).abs() > 6.0 { continue; }
                let cand = [px, y + 0.05, pz];
                let eye = [cand[0], cand[1] + 1.5, cand[2]];
                let chest = [d.pos[0], d.pos[1] + 0.9, d.pos[2]];
                if sim.world.segment(eye, chest).is_some() { continue; }
                // In front of the guard: the spot's bearing vs his facing.
                let to_bond = (cand[0] - d.pos[0]).atan2(cand[2] - d.pos[2]);
                let diff = (to_bond - d.yaw + std::f32::consts::PI).rem_euclid(2.0 * std::f32::consts::PI) - std::f32::consts::PI;
                if diff.abs() > 1.35 { continue; }
                spot = Some(cand);
                break;
            }
            let Some(spot) = spot else { continue };
            let eye = [spot[0], spot[1] + 1.5, spot[2]];
            let mut node_ok = false;
            for n in sim.covers.iter().filter(|n| n.is_corner()) {
                let ddx = n.pos[0] - d.pos[0]; let ddz = n.pos[2] - d.pos[2];
                if ddx * ddx + ddz * ddz > 625.0 { continue; }
                if n.path != gpath || n.dist.get(gfrom as usize).copied().unwrap_or(0xff) == 0xff { continue; }
                let mut rt = nf_game::cover::CoverRt::default();
                let _ = n.usable(&mut rt, &sim.world, &sim.drones.channels, sim.level, spot, eye);
                if rt.bits & 0xe != 0 { node_ok = true; break; }
            }
            if !node_ok { continue; }
            let score = d.pos[0] * d.pos[0] + d.pos[2] * d.pos[2];
            if score < best_score { best_score = score; best = Some((ix, spot)); }
        }
        match best { Some(x) => x, None => panic!("no live cover-prop guard with a usable node and LOS on 07000006") }
    };
    // Teleport Bond into the guard's sight and keep gunfire sounds coming from his spot.
    {
        let sim = scene.sim.as_mut().unwrap();
        sim.bond.pos = spot;
        sim.bond.vel = [0.0; 3];
    }
    let mut covered: Option<(usize, usize, u8)> = None;
    // god mode: the guard's return fire must not end the probe (Bond takes no damage).
    let mut dev = Dev { god_mode: true, ..Default::default() };
    for t in 0..1200u32 {
        {
            let sim = scene.sim.as_mut().unwrap();
            // Synthetic gunfire at Bond's spot ([S] unverified id/weight — the trigger, not the
            // assertion; weight 40 = a loud shot: `HandleSoundAlerts`' share = 0.01·loudness
            // [M 0x148a30 via drone.rs's port] crosses the 0.2 heard-noise gate in one push).
            if t < 150 {
                sim.drones.sounds.push(nf_game::drone::Sound { id: 0x5ef, pos: sim.bond.pos, inner: 10.0, outer: 40.0, weight: 40.0 });
            }
            sim.step_bond(Default::default(), false, false, false);
        }
        dev.after_tick(&mut scene);
        let sim = scene.sim.as_ref().unwrap();
        if t % 60 == 0 && std::env::var_os("NF_COVER_DEBUG").is_some() {
            let data = nf_game::drone::Data {
                tables: &sim.tables, props: &sim.props, scripts: &sim.scripts, weapons: &sim.weapons,
                world: &sim.world, level: sim.level, sequences: &sim.seqs, skins: &sim.skins,
                skeletons: &sim.skeletons, nav: &sim.nav, covers: &sim.covers, tuning: &sim.tuning,
                hitboxes: &sim.hitboxes, volumes: &sim.volumes, sfx: sim.sfx.as_ref(), models: &sim.models,
                surfaces: sim.surfaces.as_ref(), voices: sim.voices.as_deref(), difficulty: sim.drones.difficulty, ai_points: &sim.ai_points,
            };
            let mut states: Vec<String> = Vec::new();
            for (k, &li) in sim.drones.llist.iter().enumerate() {
                let d = &sim.drones.list[li];
                if d.enabled && d.present {
                    let pr = |id: usize| if d.prop(&data, id) { "1" } else { "0" };
                    states.push(format!("#{k}:{:#x}{}30 props a{}b{}c{}d{}f{}g10{}44{}{}", d.state,
                        if d.prop(&data, 0x30) { "+" } else { "-" },
                        pr(0xa), pr(0xb), pr(0xc), pr(0xd), pr(0xf), pr(0x10), pr(0x44),
                        if d.cover.is_some() { "*" } else { "" }));
                    if matches!(d.state, 0x58..=0x5f) {
                        let feet = d.feet();
                        let path = d.nav_path.unwrap_or_else(|| sim.nav.nav_path_for(feet));
                        let from = path.and_then(|p| sim.nav.node_from(&sim.world, p, feet));
                        let (mut ok, mut tot) = (0usize, 0usize);
                        if let (Some(p), Some(f)) = (path, from) {
                            for (i, n) in sim.covers.iter().enumerate() {
                                if n.path != Some(p) { continue; }
                                tot += 1;
                                if n.dist.get(f as usize).copied().unwrap_or(0xff) != 0xff { ok += 1; }
                            }
                        }
                        states.push(format!("[path {path:?} node {from:?} reach {ok}/{tot}]"));
                    }
                }
            }
            eprintln!("[cover] +{t}: bond {:?} states {}", sim.bond.pos, states.join(" "));
        }
        for (k, &li) in sim.drones.llist.iter().enumerate() {
            let d = &sim.drones.list[li];
            if matches!(d.state, 0x8c | 0x93..=0x9c) {
                if let Some(node) = d.cover {
                    covered = Some((k, node, d.state));
                    break;
                }
            }
        }
        if covered.is_some() { break; }
    }
    let (k, node, state) = covered.expect("no guard entered the cover states under fire");
    eprintln!("[cover] drone llist #{k} state {state:#x} claimed node {node}");
    {
        let sim = scene.sim.as_ref().unwrap();
        assert!(node < sim.covers.len(), "claimed node {node} is one of the game's own nodes");
        assert!(matches!(state, 0x8c | 0x93..=0x9c));
    }
    // RunForCover 0x8C's MoveToGoalPosition reports arrival at route distance < 2.0 → 0x93, which
    // slides to the in-cover point; UnderCoverIdle 0x94 leaves when "> 1.5 from the goal" [M
    // docs/systems/drone-cover.md §States] — a drone DWELLING in 0x94 is within 1.5 of his point.
    let mut arrived: Option<[f32; 3]> = None;
    for _ in 0..600u32 {
        {
            let sim = scene.sim.as_mut().unwrap();
            sim.step_bond(Default::default(), false, false, false);
        }
        dev.after_tick(&mut scene);
        let sim = scene.sim.as_ref().unwrap();
        let d = &sim.drones.list[sim.drones.llist[k]];
        if d.state == 0x94 {
            let dx = d.pos[0] - d.cover_point[0]; let dy = d.pos[1] - d.cover_point[1]; let dz = d.pos[2] - d.cover_point[2];
            let dist = (dx * dx + dy * dy + dz * dz).sqrt();
            assert!(dist < 1.5, "in-cover drone {dist:.2} from his cover point");
            arrived = Some(d.pos);
            break;
        }
        if matches!(d.state, 0x44 | 0x46 | 0x47 | 0x48 | 0x49) { break; }
    }
    let sim = scene.sim.as_ref().unwrap();
    match arrived {
        Some(p) => eprintln!("[cover] reached UnderCoverIdle at {p:?} (node {node}, kind {}, side {})", sim.covers[node].kind, sim.drones.list[sim.drones.llist[k]].cover_side),
        None => eprintln!("[cover] claim pinned at 0x8c (the arrival window lapsed — blocked or engaged); state family + node still pinned"),
    }
}
