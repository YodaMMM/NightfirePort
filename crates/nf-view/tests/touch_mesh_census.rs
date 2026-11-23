//! touch-mesh-contact: where the port's standing trigger touch fires versus where the game's
//! two-stage HITTEST pick fires, over every Touch / TouchOnce volume and every ladder on every
//! level of `nf_view::dev::LEVELS`.
//!
//! The game's contact for a trigger is Bond's own walk HITTEST picking the trigger object
//! (triggers are passive: `Trigger_Create` 0x1b1720 arms no HITTEST of their own [M]):
//!
//! * stage 1, `Collide_PickObj` 0x1eae80:152-168 (mode 0x800): the trigger's world sphere
//!   (`Control_BuildWorldSph` 0x133070: glist bounds centre through obj+0x90, radius glist+0x1c ·
//!   |scale|) inflated ×1.5, plus the HITTEST radius, against the closest point on the HITTEST
//!   segment, strict `<` [M :166];
//! * stage 2, `Collide_Intersect` 0x1ebaa8 → `Intersect_CylGeom` 0x109da0 over the glist's
//!   collision triangles: `Trigger::game_contact` → `triggers::cyl_geom_min`, the raw branch for a
//!   glist whose flags carry bit 0x40 (`Collide_Intersect` 0x1ebaa8:89-96 sets HITDATA+0x51 |= 2
//!   [M]) and the reference-point law for any other.
//!
//! The walk HITTEST is the default arm: A = pos + 0.275, B = pos + 0.55 − cc, r 0.55 [M
//! Player_Collision 0x194df0 :315-333], cc = `player::ROOT_HEIGHT`.
//!
//! "port" columns are the shipped stand-in (`drone::human_touchers` → `box_touchers`: the HITTEST
//! segment on beam glists, the measured box `Toucher::player(pos − 0.6)` with its containment
//! branch, the raw-plate segment), so the rows list every volume whose standing contact a switch
//! to the game's model would change. `drone::game_touchers` is checked against the model at every
//! sample.
//!
//! Samples: Bond STANDING on the world floor (cel collision, rays straight down) on a 0.2 grid
//! over each volume's footprint grown by 3.0, at every floor layer the ray meets in the volume's
//! height band ± 3, pos = floor + 1.086 (the standing rest [L player.rs ROOT_HEIGHT doc]). Not
//! modelled on either side: the straddled-cel scoping (`Collide_Pick` 0x1eb320) and the
//! nearest-solid cut-off of the reflected contact list (`Collide_Update` 0x1ea398:160-200).
//!
//! Ladders already run the two-stage pick on their own 0x2E triangles (`Ladder::pick_hit` →
//! `refine` → `capsule_hits_tris_strict`), so the ladder rows count meshes only.
//!
//! Opt-in via NF_ISO; prints only.

use nf_formats::disc;
use nf_formats::iso9660::IsoImage;
use nf_formats::level::Level;
use nf_formats::placement::{transform, view_rot_trans_matrix};
use nf_game::triggers::{self, kind, Trigger};
use nf_view::dev;

const R: f32 = 0.55;
const STAND: f32 = 1.086;

fn hittest(pos: [f32; 3]) -> ([f32; 3], [f32; 3]) {
    let cc = nf_game::player::ROOT_HEIGHT;
    ([pos[0], pos[1] + 0.275, pos[2]], [pos[0], pos[1] + 0.55 - cc, pos[2]])
}

fn closest_on_seg(a: [f32; 3], b: [f32; 3], p: [f32; 3]) -> [f32; 3] {
    let ab = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let ap = [p[0] - a[0], p[1] - a[1], p[2] - a[2]];
    let l = ab[0] * ab[0] + ab[1] * ab[1] + ab[2] * ab[2];
    let t = if l < 1e-12 { 0.0 } else { ((ap[0] * ab[0] + ap[1] * ab[1] + ap[2] * ab[2]) / l).clamp(0.0, 1.0) };
    [a[0] + ab[0] * t, a[1] + ab[1] * t, a[2] + ab[2] * t]
}

#[derive(Clone)]
struct Vol { lcentre: [f32; 3], t: Trigger, centre: [f32; 3], radius: f32, bit40: bool, flags: u32, name: String }

/// The game's two-stage decision for the standing walk HITTEST at `pos`.
fn mesh_fires(v: &Vol, pos: [f32; 3]) -> bool {
    let (a, b) = hittest(pos);
    let c = closest_on_seg(a, b, v.centre);
    let d2 = (0..3).map(|k| (c[k] - v.centre[k]).powi(2)).sum::<f32>();
    let reach = v.radius * 1.5 + R;
    if d2 >= reach * reach { return false; }
    // The ported `Intersect_CylGeom` 0x109da0 (both branches, `Trigger::game_contact`).
    assert_eq!(v.t.raw, v.bit40);
    v.t.game_contact(a, b, R).is_some()
}

/// The shipped stand-in's standing decision (`drone::human_touchers`).
fn port_fires(v: &Vol, pos: [f32; 3]) -> bool {
    let (a, b) = hittest(pos);
    nf_game::drone::box_touchers(pos, (a, b, R), false, false, false).iter().any(|c| v.t.touched_by(c))
}

/// The game's own contact as a toucher set (`drone::game_touchers`).
fn new_fires(v: &Vol, pos: [f32; 3]) -> bool {
    let (a, b) = hittest(pos);
    nf_game::drone::game_touchers(pos, (a, b, R), false, false, false).iter().any(|c| v.t.touched_by(c))
}

/// Why the port fires where the game does not: 0 = the stage-1 sphere rejects (the mesh alone
/// would fire), 1 = containment only, 2 = the game's segment would touch both-sided but the
/// one-sided raw entry rejects, 3 = reach (the box's extra height).
fn port_only_cause(v: &Vol, pos: [f32; 3]) -> usize {
    let (a, b) = hittest(pos);
    let s2 = if v.bit40 { v.t.tris.iter().zip(&v.t.planes).any(|(t, (n, d))| triggers::seg_touches_tri_raw(a, b, R, t, n, *d)) }
        else { triggers::capsule_hits_tris_strict(a, b, R, &v.t.tris) };
    if s2 { return 0; }
    let no_contain = nf_game::drone::box_touchers(pos, (a, b, R), false, false, false).iter()
        .any(|c| v.t.touched_by(&nf_game::triggers::Toucher { contain: false, ..*c }));
    if !no_contain { return 1; }
    if triggers::capsule_hits_tris_strict(a, b, R, &v.t.tris) { return 2; }
    3
}

/// Horizontal distance from (x, z) to the volume's xz bounds rectangle (0 inside).
fn out_xz(v: &Vol, x: f32, z: f32) -> f32 {
    let dx = (v.t.lo[0] - x).max(0.0).max(x - v.t.hi[0]);
    let dz = (v.t.lo[2] - z).max(0.0).max(z - v.t.hi[2]);
    (dx * dx + dz * dz).sqrt()
}

#[test]
#[ignore = "print-only census; run with --ignored --nocapture"]
fn touch_mesh_census_all_levels() {
    let Some(path) = std::env::var_os("NF_ISO") else { eprintln!("skip: NF_ISO not set"); return };
    let iso = IsoImage::open(path.to_str().unwrap()).unwrap();
    let (entries, bin) = disc::files_bin_entries(&iso).unwrap();
    let read = |name: &str| -> Option<Vec<u8>> {
        let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case(name))?;
        Some(iso.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).unwrap())
    };
    let only = std::env::var("NF_TMC_LEVEL").ok().and_then(|s| u32::from_str_radix(s.trim_start_matches("0x"), 16).ok());
    let (mut n_vol, mut n_diff, mut n_lad, mut n_lad_nomesh) = (0usize, 0usize, 0usize, 0usize);
    let mut tot_cause = [0usize; 4];
    let mut tot_vol_cause = [0usize; 4];
    let (mut tot_po, mut tot_mo, mut tot_both, mut vol_mo) = (0usize, 0usize, 0usize, 0usize);
    let mut new_diff = 0usize;
    for &id in dev::LEVELS {
        if only.is_some_and(|o| o != id) { continue; }
        let Some(data) = read(&nf_view::level_archive_name(id)) else { continue };
        let Ok(level) = Level::parse(&data) else { println!("{id:08x}: parse failed"); continue };
        let Some(mm) = level.main_map() else { continue };
        let map = &level.maps[mm];
        let world = nf_game::world::CollisionWorld::build(&level, &level.instances(true).0);
        // Walkable-floor witnesses: the level's AI path nodes (guards walk there) and its start spots.
        let nodes: Vec<[f32; 3]> = map.ai_network.as_ref().map(|n| nf_game::nav::Network::build(n, &world, id).paths.iter()
            .flat_map(|p| p.nodes.iter().map(|n| n.pos)).collect()).unwrap_or_default();
        let near_node = |p: [f32; 3]| -> f32 { nodes.iter().filter(|n| (n[1] - (p[1] - STAND)).abs() < 1.2)
            .map(|n| ((n[0] - p[0]).powi(2) + (n[2] - p[2]).powi(2)).sqrt()).fold(f32::INFINITY, f32::min) };
        // Ladders.
        for p in &map.placements {
            if p.flags & 0xff != 0x22 { continue; }
            let obj = if p.by_hash() { level.by_hash.get(&p.hashcode).map(|&(om, oi)| &level.maps[om].objects[oi]) }
                else { map.objects.get(p.index as usize) };
            if let Some(l) = nf_game::ladder::Ladder::create(p, obj) {
                n_lad += 1;
                if l.tris.is_empty() { n_lad_nomesh += 1; println!("{id:08x} ladder at {:?}: no 0x2E mesh", l.centre); }
            }
        }
        // Touch volumes Bond can fire (mask bit 8 or the 0x18 either-family).
        let vols: Vec<Vol> = triggers::build(&level).into_iter().filter_map(|t| {
            if !matches!(t.kind, kind::TOUCH | kind::TOUCH_ONCE) || t.tris.is_empty() || t.mask & 8 == 0 { return None; }
            let p = &map.placements[t.placement];
            let (om, oi) = if p.by_hash() { *level.by_hash.get(&p.hashcode)? } else { (mm, p.index as usize) };
            let o = &level.maps[om].objects[oi];
            let mx = view_rot_trans_matrix(p.rotation, p.translation);
            let s = p.scale()[0].abs();
            let s = if s.is_finite() && s > 0.0 { s } else { 1.0 };
            Some(Vol { lcentre: o.centre, centre: transform(&mx, o.centre), radius: o.radius * s, bit40: o.flags & 0x40 != 0, flags: o.flags, name: o.name.clone(), t })
        }).collect();
        let dump = std::env::var("NF_TMC_DUMP").ok().and_then(|s| s.parse::<usize>().ok());
        for v0 in &vols {
            n_vol += 1;
            // A moving volume (`Trigger_Create`'s key-list motion) is sampled at 8 poses over one pass.
            let poses: Vec<Vol> = match v0.t.motion.as_ref() {
                Some(m) if m.step > 0.0 => {
                    let ticks = ((m.last as f32 / m.step).ceil() as usize).max(1);
                    let mut cur = v0.clone();
                    let mut out = Vec::new();
                    for i in 0..ticks {
                        if i % (ticks / 8).max(1) == 0 && out.len() < 8 {
                            let mut c = cur.clone();
                            if let Some(m) = c.t.motion.as_ref() { c.centre = transform(&m.matrix, c.lcentre); }
                            out.push(c);
                        }
                        cur.t.step_motion();
                    }
                    out
                }
                _ => vec![v0.clone()],
            };
            let g = 3.0f32;
            let step = 0.2f32;
            let (mut port_only, mut mesh_only, mut both) = (0usize, 0usize, 0usize);
            let mut cause = [0usize; 4];
            let (mut port_far, mut mesh_far) = (0.0f32, 0.0f32);
            let mut port_far_at = [0.0f32; 3];
            let mut mesh_far_at = [0.0f32; 3];
            let (mut po_dy, mut mo_dy) = ((f32::MAX, f32::MIN), (f32::MAX, f32::MIN));
            let mut po_node = f32::INFINITY;
            let mut mo_node = f32::INFINITY;
            for v in &poses {
            let nx = (((v.t.hi[0] - v.t.lo[0]) + 2.0 * g) / step).ceil() as usize + 1;
            let nz = (((v.t.hi[2] - v.t.lo[2]) + 2.0 * g) / step).ceil() as usize + 1;
            let (top, bot) = (v.t.hi[1] + g, v.t.lo[1] - g);
            for ix in 0..nx.min(400) {
                for iz in 0..nz.min(400) {
                    let x = v.t.lo[0] - g + ix as f32 * step;
                    let z = v.t.lo[2] - g + iz as f32 * step;
                    let mut y = top;
                    for _layer in 0..6 {
                        let from = [x, y, z];
                        let to = [x, bot, z];
                        let Some(f) = world.segment(from, to) else { break };
                        let fy = from[1] + (to[1] - from[1]) * f;
                        let pos = [x, fy + STAND, z];
                        let (pf, mf) = (port_fires(v, pos), mesh_fires(v, pos));
                        if new_fires(v, pos) != mf { new_diff += 1; }
                        let o = out_xz(v, x, z);
                        if dump == Some(v.t.placement) && ix % 4 == 0 && iz % 4 == 0 { println!("  sample x {x:.2} z {z:.2} floor {fy:.3} port {pf} mesh {mf}"); }
                        match (pf, mf) {
                            (true, true) => both += 1,
                            (true, false) => { port_only += 1; po_node = po_node.min(near_node(pos)); cause[port_only_cause(v, pos)] += 1; po_dy = (po_dy.0.min(v.t.lo[1] - fy), po_dy.1.max(v.t.lo[1] - fy)); if o >= port_far { port_far = o; port_far_at = pos; } }
                            (false, true) => { mesh_only += 1; mo_node = mo_node.min(near_node(pos)); mo_dy = (mo_dy.0.min(v.t.lo[1] - fy), mo_dy.1.max(v.t.lo[1] - fy)); if o >= mesh_far { mesh_far = o; mesh_far_at = pos; } }
                            _ => {}
                        }
                        y = fy - 0.05;
                        if y <= bot { break; }
                    }
                }
            }
            }
            let v = &poses[0];
            let flat = v.t.lo[1] == v.t.hi[1];
            if std::env::var("NF_TMC_DUMP").ok().and_then(|s| s.parse::<usize>().ok()) == Some(v.t.placement) {
                for (t, (n, d)) in v.t.tris.iter().zip(&v.t.planes) { println!("  tri {:?} plane n {:?} d {:.3}", t, n, d); }
                println!("  mesh-only far sample {:?}  port-only far sample {:?}", mesh_far_at, port_far_at);
            }
            for k in 0..4 { tot_cause[k] += cause[k]; if cause[k] > 0 { tot_vol_cause[k] += 1; } }
            tot_po += port_only; tot_mo += mesh_only; tot_both += both; if mesh_only > 0 { vol_mo += 1; }
            if port_only + mesh_only > 0 {
                n_diff += 1;
                println!("{id:08x} #{} '{}'{} kind {} out {:#04x} flags {:#x}{} {} tris {} box {:?}..{:?} sph r {:.2}: both {both} port-only {port_only} (far {port_far:.2} at {:?}) mesh-only {mesh_only} (far {mesh_far:.2} at {:?}) | mask {:#x} | nav-node dist port-only {:.1} mesh-only {:.1} | causes sph/contain/side/reach {:?} | box-floor dy port-only {:.2}..{:.2} mesh-only {:.2}..{:.2}",
                    v.t.placement, v.name, if v.t.motion.is_some() { " MOVING" } else { "" }, v.t.kind, v.t.out, v.flags, if v.bit40 { " raw" } else { "" }, if flat { "flat" } else { "vol" }, v.t.tris.len(),
                    v.t.lo, v.t.hi, v.radius, port_far_at, mesh_far_at, v.t.mask, po_node, mo_node, cause, po_dy.0, po_dy.1, mo_dy.0, mo_dy.1);
            } else {
                println!("{id:08x} #{} kind {} out {:#04x} flags {:#x} {} same (both {both})", v.t.placement, v.t.kind, v.t.out, v.flags, if flat { "flat" } else { "vol" });
            }
        }
    }
    println!("== samples: both {tot_both}, port-only {tot_po} (causes sph/contain/side/reach {tot_cause:?}, volumes {tot_vol_cause:?}), mesh-only {tot_mo} ({vol_mo} volumes)");
    println!("== game_touchers vs the game model: {new_diff} samples differ (the stage-1 sphere only)");
    println!("== touch volumes {n_vol}, differing {n_diff}; ladders {n_lad} ({n_lad_nomesh} without a mesh, all on the two-stage pick already)");
}
