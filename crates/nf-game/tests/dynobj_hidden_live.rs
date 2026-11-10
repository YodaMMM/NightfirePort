//! dynobj-hidden-skip's real-level check [D]. `DynamicObject_Create` 0x18dbf8:24-32 ORs 0x30 into
//! obj+0xF0 when the placement's p0 short is non-zero, and `Collide_PickObj` 0x1eae80:72-82 keeps
//! 0x20 in every skip set, so the game never picks such an object for any ray — a guard's sight
//! included. The census (`dynobj_p0_census.rs`) found the disc's hidden placements; this file pins
//! the port end-to-end on a real level:
//! - a hidden placement's collision part is NOT in the Sim's dynamic set, so a ray from a real
//!   level-start guard's vantage through the object's centre is not stopped by it (its first hit
//!   never lands inside the object's bounds);
//! - the same ray against a set that carries the part (the pre-fix world) stops INSIDE those
//!   bounds — the proof the check can detect the failure, and that the sight line really crosses
//!   the object's triangles;
//! - the sister: a visible p0 == 0 dynobj on a guard sight line DOES stop the ray inside its own
//!   bounds on the real set (the same data tells hidden from visible).
//!
//! The guard vantage is the level guard's live position + 0.9 [S — a stand-in head offset]; the
//! sight-line claim rests on the proven triangle crossing, not on the exact eye byte.
//!
//! Opt-in: NF_ISO.

use nf_formats::anim::Script;
use nf_formats::iso9660::IsoImage;
use nf_formats::{disc, drone, level::Level, levelbin::LevelBin, weapon::WeaponData};
use nf_game::drone::{Data, Drones, Player, Rng};
use std::collections::HashMap;

type V3 = [f32; 3];
type Mat4 = nf_formats::anim::Mat4;

fn sub(a: V3, b: V3) -> V3 { [a[0] - b[0], a[1] - b[1], a[2] - b[2]] }

/// Everything `Data` borrows, owned (the ai_live loader shape).
struct Fixture {
    tables: drone::DroneAnimData,
    props: drone::PropertyTables,
    weapons: WeaponData,
    level: Level,
    scripts: HashMap<u32, Script>,
    world: nf_game::world::CollisionWorld,
    sequences: HashMap<u32, nf_formats::anim::Sequence>,
    skeletons: HashMap<u8, nf_formats::character::Skeleton>,
    nav: nf_game::nav::Network,
    covers: Vec<nf_game::cover::CoverNode>,
    sfx: nf_formats::sfx::SfxTable,
    tuning: nf_formats::tuning::Tuning,
    hitboxes: nf_formats::hitbox::CollisionBones,
    volumes: Vec<nf_game::aivolume::AiVolume>,
    models: std::collections::HashMap<u32, nf_game::pickup::Model>,
    hud: nf_formats::hud::HudTables,
    surfaces: nf_formats::sfx::SurfaceSounds,
    voices: Vec<nf_formats::sfx::VoiceLine>,
}

impl Fixture {
    fn load(name: &str, id: u32) -> Option<Fixture> {
        let path = std::env::var_os("NF_ISO")?;
        let iso = IsoImage::open(&path).unwrap();
        let elf = iso.read_file(iso.find("/ACTION.ELF").unwrap()).unwrap();
        let (entries, bin) = disc::files_bin_entries(&iso).unwrap();
        let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case(name)).unwrap_or_else(|| panic!("{name} not in FILES.BIN"));
        let data = iso.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).unwrap();
        let level = Level::parse(&data).unwrap();
        let lb = LevelBin::parse(&data).unwrap();
        let scripts = lb.entries.iter().filter(|e| e.kind == 5).filter_map(|e| Script::parse(lb.data(&data, e)).ok()).map(|s| (s.hashcode, s)).collect();
        let world = nf_game::world::CollisionWorld::build(&level, &level.instances(true).0);
        let skeletons = lb.entries.iter().filter(|e| e.kind == 6).filter_map(|e| nf_formats::character::parse_skeleton(lb.data(&data, e)).ok()).map(|k| (k.id as u8, k)).collect();
        let m = level.main_map().unwrap();
        let Some(net) = level.maps[m].ai_network.as_ref() else { eprintln!("{name}: no AI network — skipping"); return None };
        let nav = nf_game::nav::Network::build(net, &world, id);
        let covers = nf_game::cover::build(&level, &nav, &world);
        let volumes = nf_game::aivolume::build(&level, id);
        Some(Fixture {
            tables: drone::DroneAnimData::from_action_elf(&elf).unwrap(),
            props: drone::PropertyTables::from_action_elf(&elf).unwrap(),
            weapons: WeaponData::from_action_elf(&elf).unwrap(),
            sfx: nf_formats::sfx::SfxTable::from_action_elf(&elf).unwrap(),
            tuning: { let mut t = nf_formats::tuning::Tuning::from_action_elf(&elf).unwrap(); nf_game::tuning::read_tuning_vars(&mut t, id); t },
            hitboxes: nf_formats::hitbox::CollisionBones::from_action_elf(&elf).unwrap(),
            sequences: disc::level_sequences(&iso, name).unwrap(),
            models: nf_game::pickup::models(&level),
            hud: nf_formats::hud::HudTables::from_action_elf(&elf).unwrap(),
            surfaces: nf_formats::sfx::SurfaceSounds::from_action_elf(&elf).unwrap(),
            voices: nf_formats::sfx::voice_lines(&elf).unwrap(),
            level, scripts, world, skeletons, nav, covers, volumes,
        })
    }

    fn data(&self, id: u32) -> Data<'_> {
        Data { tables: &self.tables, props: &self.props, scripts: &self.scripts, weapons: &self.weapons, world: &self.world, level: id,
            sequences: &self.sequences, skins: &self.level.skins, skeletons: &self.skeletons, nav: &self.nav, covers: &self.covers, tuning: &self.tuning, hitboxes: &self.hitboxes, volumes: &self.volumes,
            sfx: Some(&self.sfx), models: &self.models, surfaces: Some(&self.surfaces), voices: Some(&self.voices), difficulty: 1,
                ai_points: &[],
            }
    }

    /// The level's real drones at level start, on the Sim.
    fn sim(&self, id: u32, d: &Data<'_>) -> Drones {
        let mut rng = Rng::new(1);
        let mut lights = nf_game::light::LightList::default();
        let drones = nf_game::drone::level_drones(d, &self.level, &mut rng, &mut lights);
        let mut s = Drones::new(drones, rng);
        s.attach_level_id(&self.level, id);
        s.difficulty = 1;
        s
    }
}

fn bond_at(p: V3) -> Player { Player { obj_cel: None, yaw: 0.0, pos: p, target: [p[0], p[1] + 0.5, p[2]], eye: [p[0], p[1] + 1.0, p[2]], aim: [-1.0, 0.0, 0.0], sph: p, capsule: ([p[0], p[1] + 1.086, p[2]], p), hanging: false, body: Default::default(), cel: None, move_local: [0.0; 2], riding: false, crouched: false } }

/// The world AABB of a shape posed by `m`.
fn aabb(shape: &[[V3; 3]], m: &Mat4) -> (V3, V3) {
    let (mut lo, mut hi) = ([f32::MAX; 3], [f32::MIN; 3]);
    for v in shape.iter().flatten().map(|&v| nf_formats::placement::transform(m, v)) {
        for k in 0..3 { lo[k] = lo[k].min(v[k]); hi[k] = hi[k].max(v[k]); }
    }
    (lo, hi)
}

/// The segment's entry/exit parameters through the box, None when it misses.
fn span(lo: V3, hi: V3, a: V3, b: V3) -> Option<(f32, f32)> {
    let (mut t0, mut t1) = (0.0f32, 1.0f32);
    for k in 0..3 {
        let d = b[k] - a[k];
        if d.abs() < 1e-9 {
            if a[k] < lo[k] || a[k] > hi[k] { return None; }
        } else {
            let (mut ta, mut tb) = ((lo[k] - a[k]) / d, (hi[k] - a[k]) / d);
            if ta > tb { std::mem::swap(&mut ta, &mut tb); }
            t0 = t0.max(ta);
            t1 = t1.min(tb);
            if t0 > t1 { return None; }
        }
    }
    Some((t0, t1))
}

struct Cand {
    slot: usize,
    shape: u32,
    matrix: Mat4,
    centre: V3,
    lo: V3,
    hi: V3,
    hidden: bool,
    live_flags: u32,
}

/// One level: the dynobj candidates with shapes (with the live flags the Sim's filter reads) and
/// the level-start guards' vantages, after 15 warm ticks on the real Sim.
fn level_cases(fix: &Fixture, id: u32) -> (Vec<Cand>, Vec<V3>, String) {
    let d = fix.data(id);
    let mut sim = fix.sim(id, &d);
    let away = bond_at([0.0, -500.0, 0.0]);
    for _ in 0..15 { sim.tick(&d, &away); }
    let mut cands = Vec::new();
    for (i, o) in sim.worldobjs.dynobjs.iter().enumerate() {
        if o.shape == 0 { continue; }
        let Some(s) = d.world.shape(o.shape) else { continue };
        let (lo, hi) = aabb(s, &o.matrix);
        cands.push(Cand { slot: i, shape: o.shape, matrix: o.matrix, centre: o.pos, lo, hi, hidden: o.flags & 0x20 != 0, live_flags: o.flags });
    }
    let eyes: Vec<V3> = sim.list.iter().filter(|dr| dr.present)
        .map(|dr| [dr.pos[0], dr.pos[1] + 0.9, dr.pos[2]]).collect();
    let summary = format!("{} level-start guards; dynobjs with shapes {} (hidden {}): {}", eyes.len(),
        cands.len(), cands.iter().filter(|c| c.hidden).count(),
        cands.iter().map(|c| format!("slot {} at ({:.2},{:.2},{:.2}) flags {:#x}{}", c.slot, c.centre[0], c.centre[1], c.centre[2], c.live_flags, if c.hidden { " HIDDEN" } else { "" })).collect::<Vec<_>>().join("; "));
    (cands, eyes, summary)
}

/// The geometric crossings: guard vantage → through the object's centre → 10 units past it, where
/// the segment really passes through the object's AABB (span length ≥ 0.02) and the eye is ≥ 2
/// units outside it.
fn crossings(eyes: &[V3], cands: &[Cand], want_hidden: bool) -> Vec<(usize, usize, V3, V3, (f32, f32))> {
    let mut out = Vec::new();
    for (gi, eye) in eyes.iter().enumerate() {
        for (ci, c) in cands.iter().enumerate() {
            if c.hidden != want_hidden { continue; }
            let dir = sub(c.centre, *eye);
            let len = (dir[0] * dir[0] + dir[1] * dir[1] + dir[2] * dir[2]).sqrt();
            if len < 2.0 { continue; }
            let beyond = [c.centre[0] + dir[0] / len * 10.0, c.centre[1] + dir[1] / len * 10.0, c.centre[2] + dir[2] / len * 10.0];
            let Some(sp) = span(c.lo, c.hi, *eye, beyond) else { continue };
            if sp.1 - sp.0 < 0.02 { continue; }
            out.push((gi, ci, *eye, beyond, sp));
        }
    }
    out
}

#[test]
fn a_hidden_dynobj_on_a_guard_sight_line_does_not_block() {
    // 07000007: the p0 = 21 rope barrier (pl 495) and the p0 = 9 Dynamic Blocker (pl 496) [D census];
    // 07000014: the channel-gated console/screen swaps (p0 = 0x4e..0xcc) [D census].
    let mut loaded = Vec::new();
    for (name, id) in [("07000007.bin", 0x0700_0007u32), ("07000014.bin", 0x0700_0014)] {
        if let Some(f) = Fixture::load(name, id) { loaded.push((name, id, f)); }
    }
    if loaded.is_empty() { eprintln!("skip: NF_ISO not set"); return };

    let mut sight_line_pairs = Vec::new();
    let mut visible_sisters = Vec::new();
    for (name, id, fix) in &loaded {
        let d = fix.data(*id);
        let (cands, eyes, summary) = level_cases(fix, *id);
        eprintln!("== {name}: {summary}");
        let hidden_x = crossings(&eyes, &cands, true);
        let visible_x = crossings(&eyes, &cands, false);
        // GREEN pass first: the world holds the Sim's real parts (the fixed filter dropped the
        // hidden ones).
        let green_t: Vec<Option<f32>> = hidden_x.iter().chain(visible_x.iter())
            .map(|(_, _, eye, beyond, _)| d.world.segment_hit(*eye, *beyond).map(|(t, _)| t)).collect();
        // RED pass: the pre-fix world — one hidden part alone in the dynamic set. A pair whose ray
        // stops inside the object's bounds is a clean sight-line crossing (nothing else stops it
        // first, and its triangles really block).
        let mut clean = vec![false; hidden_x.len()];
        for (k, (gi, ci, eye, beyond, sp)) in hidden_x.iter().enumerate() {
            let c = &cands[*ci];
            d.world.set_dynamic(&[(0x1000_0000_0000 | c.slot as u64, c.shape, c.matrix)]);
            let red = d.world.segment_hit(*eye, *beyond).map(|(t, _)| t);
            if let Some(tr) = red {
                if tr >= sp.0 - 1e-4 && tr <= sp.1 + 1e-4 {
                    clean[k] = true;
                    sight_line_pairs.push(format!("{name} guard {gi} eye ({:.2},{:.2},{:.2}) -> hidden slot {} at ({:.2},{:.2},{:.2}) flags {:#x}: the ray stops at t {tr:.3} in span [{:.3},{:.3}]", eye[0], eye[1], eye[2], c.slot, c.centre[0], c.centre[1], c.centre[2], c.live_flags, sp.0, sp.1));
                }
            }
        }
        // Restore the Sim's real dynamic set (a fresh Sim's ticks re-run set_dynamic with the
        // fixed parts) — the GREEN verdicts above were taken before the RED pass touched it.
        {
            let mut sim = fix.sim(*id, &d);
            let away = bond_at([0.0, -500.0, 0.0]);
            for _ in 0..16 { sim.tick(&d, &away); }
        }
        for (k, (gi, ci, _, _, sp)) in hidden_x.iter().enumerate() {
            if !clean[k] { continue; }
            let tg = green_t[k];
            let stopped_by_it = tg.is_some_and(|t| t >= sp.0 - 1e-4 && t <= sp.1 + 1e-4);
            assert!(!stopped_by_it, "{name}: the hidden dynobj slot {} stopped a guard sight ray (t {tg:?} in [{:.3},{:.3}])", cands[*ci].slot, sp.0, sp.1);
            eprintln!("GREEN {name} guard {gi} -> hidden slot {}: first hit t {tg:?}, object span [{:.3},{:.3}] — not stopped by it", cands[*ci].slot, sp.0, sp.1);
        }
        for (k, (gi, ci, _, _, sp)) in visible_x.iter().enumerate() {
            let tg = green_t[hidden_x.len() + k];
            if tg.is_some_and(|t| t >= sp.0 - 1e-4 && t <= sp.1 + 1e-4) {
                visible_sisters.push(format!("{name} guard {gi} -> visible slot {} at ({:.2},{:.2},{:.2}): the ray stops at t {tg:?} in [{:.3},{:.3}]", cands[*ci].slot, cands[*ci].centre[0], cands[*ci].centre[1], cands[*ci].centre[2], sp.0, sp.1));
            }
        }
    }
    eprintln!("hidden-on-sight-line pairs (the [D] evidence):\n  {}", if sight_line_pairs.is_empty() { "(none)".into() } else { sight_line_pairs.join("\n  ") });
    eprintln!("visible dynobjs that DID stop a guard sight ray (the sisters):\n  {}", if visible_sisters.is_empty() { "(none)".into() } else { visible_sisters.join("\n  ") });
    assert!(!sight_line_pairs.is_empty(), "no hidden dynobj sits on any guard sight line on the probed levels — widen the level list");
}
