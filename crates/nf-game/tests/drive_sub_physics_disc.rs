//! Disc test for the SUB_PHYSICS layer (`nf_game::driving::sub_physics`): each of MIS11, MIS13A,
//! MIS13B and MIS13C, from its `EStartMission` start (docs/formats/trackstart.md), runs 600 ticks
//! of full throttle under `FUN_00190f88` + the VU0 integration with the mission's own SUB_PHYSICS
//! pvehicle .atr, and stays inside the track's world (the collision geometry's box).
//!
//! Opt-in via NF_ISO (the full image; a slim image without the `/DRIVING/` extents skips).

use nf_formats::bigf::Bigf;
use nf_formats::crp::Crp;
use nf_formats::iso9660::IsoImage;
use nf_game::driving::bounds::ZoneQuad;
use nf_game::driving::physics::{Rigid, DT};
use nf_game::driving::sub_physics::*;

fn entry(iso: &IsoImage, base: u64, g: &Bigf, name: &str) -> Option<Vec<u8>> {
    let e = g.entries.iter().find(|x| x.name.eq_ignore_ascii_case(name))?;
    let raw = iso.read_bytes(base + e.offset as u64, e.size as u64).ok()?;
    if raw.iter().take(64).all(|&b| b == 0) {
        return None;
    }
    Some(if nf_formats::refpack::is_refpack(&raw) { nf_formats::refpack::decompress(&raw).unwrap() } else { raw })
}

/// Half the drawn model's box (the game reads the model header's min/max, `FUN_001c9778`; this
/// takes the decoded LOD-0 geometry's box instead [S]).
fn half_extents(car: &[u8]) -> [f32; 3] {
    let mesh = nf_formats::carmesh::car_mesh(car).unwrap();
    let mut lo = [f32::MAX; 3];
    let mut hi = [f32::MIN; 3];
    for p in mesh.drawn(None) {
        for s in &p.strips {
            for &v in &s.positions {
                let w = p.apply(v);
                for k in 0..3 {
                    lo[k] = lo[k].min(w[k]);
                    hi[k] = hi[k].max(w[k]);
                }
            }
        }
    }
    [(hi[0] - lo[0]) * 0.5, (hi[1] - lo[1]) * 0.5, (hi[2] - lo[2]) * 0.5]
}

#[test]
fn every_sub_physics_mission_flies_600_ticks_of_throttle_inside_its_world() {
    let Ok(path) = std::env::var("NF_ISO") else { eprintln!("skip: NF_ISO not set"); return };
    let iso = IsoImage::open(&path).expect("open NF_ISO");
    // (archive, the archive's SUB_PHYSICS pvehicle, the pvehicle the view drives) [D attr files].
    let cases = [
        ("MIS11", "vanquishsub", "vanquishsub"),
        ("MIS13A", "ultralight", "ultralight"),
        ("MIS13B", "ultralight", "ultralight"),
        ("MIS13C", "ultralightbig", "patrolboat"),
    ];
    let rigid = Rigid::game_defaults();
    let mut ran = 0;
    for (viv, stem, view_stem) in cases {
        let Some(e) = iso.find(&format!("/DRIVING/{viv}.VIV")) else { continue };
        let g = nf_formats::disc::read_bigf(&iso, e).unwrap();
        let base = e.lba as u64 * 2048;
        let Some(track) = g
            .entries
            .iter()
            .find(|x| {
                let n = x.name.to_ascii_lowercase();
                n.starts_with("data\\track\\") && n.ends_with(".crp")
            })
            .and_then(|t| entry(&iso, base, &g, &t.name.clone()))
        else {
            eprintln!("skip {viv}: no track extents");
            continue;
        };
        let crp = Crp::parse(&track).unwrap();
        let tris = crp.extract_collision_tris().unwrap();
        let spawn = nf_formats::trackstart::track_spawn(&crp, &tris).unwrap();
        let mut lo = [f32::MAX; 3];
        let mut hi = [f32::MIN; 3];
        for t in &tris {
            for v in &t.v {
                for k in 0..3 {
                    lo[k] = lo[k].min(v[k]);
                    hi[k] = hi[k].max(v[k]);
                }
            }
        }
        let atr = |s: &str| {
            entry(&iso, base, &g, &format!("data\\sim\\attrib\\pvehicle\\{s}.atr")).map(|b| VehicleAttrs::parse_atr(&String::from_utf8_lossy(&b)))
        };
        if let Some(e) = g.entries.iter().find(|x| x.name.eq_ignore_ascii_case(&format!("data\\sim\\attrib\\pvehicle\\{stem}.atr"))) {
            let raw = iso.read_bytes(base + e.offset as u64, e.size as u64).unwrap();
            eprintln!("{viv} {stem}.atr raw refpack: {}", nf_formats::refpack::is_refpack(&raw));
        }
        let attrs = atr(stem).unwrap_or_else(|| panic!("{viv}: no {stem}.atr"));
        assert_eq!(attrs.sub_physics, 1, "{viv} {stem}");
        assert_eq!(drive_pass(&attrs, 0, false), DrivePass::Sub, "{viv}");
        let view_pass = drive_pass(&atr(view_stem).unwrap(), 0, false);
        let car = entry(&iso, base, &g, &format!("data\\car\\model\\{stem}.crp")).unwrap_or_else(|| panic!("{viv}: no {stem}.crp"));
        let half = half_extents(&car);
        let mut c = SubCraft::new(attrs, half, spawn.pos, spawn.yaw, 0);
        let input = SubInput { throttle: 1.0, ..Default::default() };
        let mut sheet_ticks = 0;
        let mut sheets: Vec<usize> = Vec::new();
        for tick in 0..600 {
            c.step(&input, &rigid, DT);
            let p = c.body.pos;
            assert!(p.iter().all(|x| x.is_finite()), "{viv}: tick {tick} {p:?}");
            for k in 0..3 {
                assert!(p[k] >= lo[k] && p[k] <= hi[k], "{viv}: tick {tick} left the world box on axis {k}: {p:?} box {lo:?}..{hi:?}");
            }
            let mut any = false;
            for i in ZoneQuad::hits([p[0], p[1], p[2], 1.0]) {
                any = true;
                if !sheets.contains(&i) {
                    sheets.push(i);
                }
            }
            sheet_ticks += any as u32;
        }
        let d = [c.body.pos[0] - spawn.pos[0], c.body.pos[1] - spawn.pos[1], c.body.pos[2] - spawn.pos[2]];
        let dist = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
        eprintln!(
            "{viv} {stem}: half {half:.2?} start {:.1?} yaw {:.1} -> {:.1?} ({dist:.1} m, |v| {:.2}), sheet ticks {sheet_ticks} (sheets {sheets:?}), box {lo:.0?}..{hi:.0?}; view's {view_stem} → {view_pass:?}",
            spawn.pos,
            spawn.yaw.to_degrees(),
            c.body.pos,
            c.speed
        );
        // 600 ticks of 14 m/s² against −0.5·v from rest: v(10 s) = 28·(1 − e^(−5)) ≈ 27.8 in
        // open space; a sheet's push only slows the craft.
        if sheet_ticks == 0 {
            assert!((c.speed - 27.8).abs() < 0.2, "{viv}: speed {}", c.speed);
            assert!(dist > 200.0 && dist < 260.0, "{viv}: {dist}");
        } else {
            assert!(c.speed < 27.9 && dist < 260.0, "{viv}: speed {} dist {dist}", c.speed);
        }
        ran += 1;
    }
    if ran == 0 {
        eprintln!("skip: no /DRIVING/ extents in this image");
    }
}

/// Each sub mission (MIS11, MIS13A, MIS13B, MIS13C): the player vehicle the track's `MSet` names
/// [M FUN_00176658 → FUN_002051f8 → FUN_00204d18; D], the start speed of its `EStartMission`
/// [M FUN_0015c738], the header box of the SUB_PHYSICS craft [M FUN_001c9778; D the `Base`
/// record], then 1200 ticks of full throttle with steering and pitch sweeps under
/// `SubCraft::step_world` (the world collision `FUN_001fc2f0` against the track's CDat
/// triangles): the craft's centre never crosses a collision triangle. In MIS13A and MIS13C the
/// spawn vehicle is a land-pass vehicle (jungle_truck / fodbase), so the archive's SUB_PHYSICS
/// craft (ultralight / ultralightbig) is the one flown there. The plain `step` (no world pass)
/// crosses a surface on at least one of the four.
#[test]
fn each_sub_mission_flies_1200_ticks_of_throttle_and_sweeps_without_entering_terrain() {
    use nf_game::driving::sub_collide::{Building, WorldMesh, QUERY_CLASS_MASK};
    let Ok(path) = std::env::var("NF_ISO") else { eprintln!("skip: NF_ISO not set"); return };
    let iso = IsoImage::open(&path).expect("open NF_ISO");
    // (archive, MSet vehicle, start speed, the SUB_PHYSICS craft flown, its header half extents)
    let cases = [
        ("MIS11", "vanquishsub", 0.0f32, "vanquishsub", [0.919f32, 0.578, 2.317]),
        ("MIS13A", "jungle_truck", 35.0, "ultralight", [5.344, 1.190, 2.998]),
        ("MIS13B", "ultralight", 2.0, "ultralight", [5.344, 1.190, 2.998]),
        ("MIS13C", "fodbase", 0.0, "ultralightbig", [10.688, 2.379, 5.995]),
    ];
    let rigid = Rigid::game_defaults();
    let building = Building::game_defaults();
    let mut ran = 0;
    let mut plain_crossed = 0;
    for (viv, mission_vehicle, speed, stem, want_half) in cases {
        let Some(e) = iso.find(&format!("/DRIVING/{viv}.VIV")) else { continue };
        let g = nf_formats::disc::read_bigf(&iso, e).unwrap();
        let base = e.lba as u64 * 2048;
        let Some(track) = g
            .entries
            .iter()
            .find(|x| {
                let n = x.name.to_ascii_lowercase();
                n.starts_with("data\\track\\") && n.ends_with(".crp")
            })
            .and_then(|t| entry(&iso, base, &g, &t.name.clone()))
        else {
            eprintln!("skip {viv}: no track extents");
            continue;
        };
        let crp = Crp::parse(&track).unwrap();
        assert_eq!(nf_formats::trackstart::mission_vehicle(&crp).unwrap().as_deref(), Some(mission_vehicle), "{viv}");
        let atr = |s: &str| {
            entry(&iso, base, &g, &format!("data\\sim\\attrib\\pvehicle\\{s}.atr")).map(|b| VehicleAttrs::parse_atr(&String::from_utf8_lossy(&b)))
        };
        let spawn_attrs = atr(mission_vehicle).unwrap_or_else(|| panic!("{viv}: no {mission_vehicle}.atr"));
        let spawn_pass = drive_pass(&spawn_attrs, 0, false);
        assert_eq!(spawn_pass == DrivePass::Sub, mission_vehicle == stem, "{viv}: {mission_vehicle} → {spawn_pass:?}");
        let tris = crp.extract_collision_tris().unwrap();
        let spawn = nf_formats::trackstart::track_spawn(&crp, &tris).unwrap();
        assert_eq!(spawn.speed_factor, speed, "{viv}");
        let attrs = atr(stem).unwrap();
        assert_eq!(attrs.sub_physics, 1, "{viv} {stem}");
        let car = entry(&iso, base, &g, &format!("data\\car\\model\\{stem}.crp")).unwrap();
        let half = nf_formats::carmesh::header_half_extents(&car).unwrap();
        for k in 0..3 {
            assert!((half[k] - want_half[k]).abs() < 0.001, "{viv} {stem}: header half {half:?}");
        }
        let lod = half_extents(&car);
        let mesh = WorldMesh::new(tris.iter().map(|t| (t.v, t.flags)));
        let mut c = SubCraft::new(attrs.clone(), half, spawn.pos, spawn.yaw, 0);
        c.start_mission(spawn.dir, spawn.speed_factor);
        let mut plain = c.clone();
        let mut contacts = 0usize;
        let mut contact_ticks = 0u32;
        let mut max_strength = 0.0f32;
        let mut damage = 0.0f32;
        let mut stuck_ticks = 0u32;
        let mut plain_cross: Option<usize> = None;
        let mut path = 0.0f32;
        for tick in 0..1200u32 {
            let t = tick as f32;
            let input = SubInput {
                throttle: 1.0,
                steer: (t * std::f32::consts::TAU / 240.0).sin(),
                pitch: 0.6 * (t * std::f32::consts::TAU / 330.0).sin(),
                ..Default::default()
            };
            let prev = c.body.pos;
            let out = c.step_world(&input, &rigid, &building, &mesh, DT);
            let p = c.body.pos;
            assert!(p.iter().all(|x| x.is_finite()), "{viv}: tick {tick} {p:?}");
            if let Some((ct, i)) = mesh.crossing(prev, p, QUERY_CLASS_MASK) {
                panic!("{viv} {stem}: tick {tick} the centre crossed triangle {i} ({:?}) at t {ct}: {prev:?} -> {p:?}", tris[i].v);
            }
            path += ((p[0] - prev[0]).powi(2) + (p[1] - prev[1]).powi(2) + (p[2] - prev[2]).powi(2)).sqrt();
            contacts += out.contacts.len();
            contact_ticks += (!out.contacts.is_empty()) as u32;
            stuck_ticks += out.nose_stuck as u32;
            for ct in &out.contacts {
                max_strength = max_strength.max(ct.strength);
                damage += ct.damage.unwrap_or(0.0);
            }
            let pp = plain.body.pos;
            plain.step(&input, &rigid, DT);
            if plain_cross.is_none() && mesh.crossing(pp, plain.body.pos, QUERY_CLASS_MASK).is_some() {
                plain_cross = Some(tick as usize);
            }
        }
        plain_crossed += plain_cross.is_some() as u32;
        eprintln!(
            "{viv}: spawn vehicle {mission_vehicle} ({spawn_pass:?}), flown {stem} header half {half:.3?} (LOD box {lod:.3?}), start {speed} m/s; \
             1200 ticks: path {path:.0} m, end {:.1?} |v| {:.1}, contacts {contacts} on {contact_ticks} ticks, max strength {max_strength:.2}, \
             damage {damage:.0}, nose-stuck ticks {stuck_ticks}; plain step first crosses at {plain_cross:?}",
            c.body.pos,
            c.speed
        );
        ran += 1;
    }
    if ran == 0 {
        eprintln!("skip: no /DRIVING/ extents in this image");
        return;
    }
    assert!(plain_crossed > 0, "the plain step crossed no surface: the check would not see a pass-through");
}
