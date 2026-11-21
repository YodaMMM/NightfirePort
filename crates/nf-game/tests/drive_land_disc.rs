//! The land car on the real MIS01 track (opt-in: `NF_ISO=<image>`): the Vanquish's own .atr [D],
//! the track's CDat collision triangles [D] as the ground (class 0x04 skipped [M 0x2138b0]), the
//! game's EStartMission spawn [M 0x1fee68 via nf_formats::trackstart], 600 ticks of full throttle.

use nf_formats::crp::Crp;
use nf_formats::crp_mesh::CollisionTri;
use nf_formats::iso9660::IsoImage;
use nf_game::driving::{Car, CarInput, CarParams, CollisionGround, Ground, Rigid, DT};

fn unpack(raw: Vec<u8>) -> Vec<u8> {
    if raw.len() >= 2 && (raw[0] & 0x3e) == 0x10 && raw[1] == 0xfb {
        nf_formats::refpack::decompress(&raw).expect("refpack")
    } else {
        raw
    }
}

struct Track {
    tris: Vec<CollisionTri>,
    /// RoadNet segment samples (the AI road graph, `RNgp` [M 0x222468]) — the test's steering aim.
    road: Vec<[f32; 3]>,
    spawn: nf_formats::trackstart::TrackSpawn,
    atr: String,
}

fn load(viv: &str) -> Option<Track> {
    let iso_path = std::env::var("NF_ISO").ok()?;
    let iso = IsoImage::open(&iso_path).ok()?;
    let entry = iso.find(&format!("/DRIVING/{viv}.VIV"))?;
    let bigf = nf_formats::disc::read_bigf(&iso, entry).ok()?;
    let base = entry.lba as u64 * 2048;
    let read = |pred: &dyn Fn(&str) -> bool| -> Option<Vec<u8>> {
        let e = bigf.entries.iter().find(|x| pred(&x.name.to_ascii_lowercase()))?;
        Some(unpack(iso.read_bytes(base + e.offset as u64, e.size as u64).ok()?))
    };
    let crp_bytes = read(&|n| n.contains("data\\track\\") && n.ends_with(".crp"))?;
    if crp_bytes.iter().all(|&b| b == 0) { return None; }
    let atr = read(&|n| n == "data\\sim\\attrib\\pvehicle\\vanquish.atr")?;
    let crp = Crp::parse(&crp_bytes).ok()?;
    let tris = crp.extract_collision_tris().ok()?;
    let spawn = nf_formats::trackstart::track_spawn(&crp, &tris).ok()?;
    let net = nf_game::driving::road::RoadNet::parse_crp(&crp).ok()?;
    let mut road = Vec::new();
    for s in 0..net.segs.len() {
        for k in 0..=10 {
            if let Ok(p) = net.seg_point(s, k as f32 / 10.0) { road.push(p); }
        }
    }
    Some(Track { tris, road, spawn, atr: String::from_utf8_lossy(&atr).into_owned() })
}

/// The test's own driver (not the game's): steer toward the road sample 15-40 m ahead that lies
/// closest to the current heading.
fn steer_to_road(car: &Car, road: &[[f32; 3]]) -> f32 {
    let f = [car.yaw.sin(), car.yaw.cos()];
    let mut best: Option<(f32, f32)> = None;
    for p in road {
        let d = [p[0] - car.pos[0], p[2] - car.pos[2]];
        let l = (d[0] * d[0] + d[1] * d[1]).sqrt();
        if !(15.0..=40.0).contains(&l) { continue; }
        let c = (d[0] * f[0] + d[1] * f[1]) / l;
        if c < 0.5 { continue; }
        let err = d[0].atan2(d[1]) - car.yaw;
        let err = (err + std::f32::consts::PI).rem_euclid(std::f32::consts::TAU) - std::f32::consts::PI;
        if best.is_none_or(|(bc, _)| c > bc) { best = Some((c, err)); }
    }
    best.map_or(0.0, |(_, e)| (e * 4.0).clamp(-1.0, 1.0))
}

/// 600 ticks (10 s) of full throttle from the MIS01 start (steered along the road by the test's
/// driver): every tick all four wheels stand on
/// the track's ground triangles (the plane under each wheel is a real triangle within the
/// suspension's reach), the body stays upright and level with the road, and the car drives off
/// at speed. Sister: the old step on the same start floored the height at 0 — MIS01's start ground
/// is not at 0, so the old car left the road surface on tick 1.
#[test]
fn vanquish_full_throttle_600_ticks_stays_on_the_mis01_road() {
    let Some(t) = load("MIS01") else { eprintln!("skip: NF_ISO unset or no MIS01 track"); return };
    let params = CarParams::parse_atr(&t.atr);
    let v = CarParams::vanquish();
    assert_eq!((params.mass, params.maxacc, params.spring_stiffness, params.spring_rest), (v.mass, v.maxacc, v.spring_stiffness, v.spring_rest), "the disc .atr is the ported vanquish [D]");
    let ground_y = t.spawn.ground.expect("ground under the start marker");
    assert!(ground_y.abs() > 1.0, "MIS01's start ground is off 0 (y {ground_y}): the old 0 floor could not hold it");
    let g = CollisionGround::new(t.tris);
    let rigid = Rigid::game_defaults();
    let mut car = Car::new(params, t.spawn.pos, t.spawn.yaw);
    // Drop onto the road from the game's spawn lift (+1.0 [M 0x1fee68]) and settle.
    for _ in 0..60 {
        car.step(&CarInput::default(), &g, &rigid, DT);
    }
    assert_eq!(car.contacts, 4, "settled on the road at y {}", car.pos[1]);
    let start = car.pos;
    let mut full = CarInput { throttle: 1.0, brake: 0.0, steer: 0.0, handbrake: false };
    let reach = car.params.spring_rest + car.params.spring_limit;
    let mut worst_gap = 0.0f32;
    let mut min_contacts = 4;
    let mut surfaces = std::collections::BTreeSet::new();
    for tick in 0..600 {
        full.steer = steer_to_road(&car, &t.road);
        car.step(&full, &g, &rigid, DT);
        min_contacts = min_contacts.min(car.contacts);
        assert!(car.rot[1][1] > 0.9, "tick {tick}: the body tipped (up.y {})", car.rot[1][1]);
        for i in 0..4 {
            let wp = car.wheel_world(i);
            let pl = g.wheel_plane(wp).unwrap_or_else(|| panic!("tick {tick}: wheel {i} at {wp:?} has no ground triangle under it"));
            let gap = -((pl.v0[0] - wp[0]) * pl.normal[0] + (pl.v0[1] - wp[1]) * pl.normal[1] + (pl.v0[2] - wp[2]) * pl.normal[2]);
            worst_gap = worst_gap.max(gap);
            assert!(gap < reach, "tick {tick}: wheel {i} {gap} m above its ground (the spring reaches {reach})");
            surfaces.insert(pl.surface);
        }
    }
    let dist = ((car.pos[0] - start[0]).powi(2) + (car.pos[2] - start[2]).powi(2)).sqrt();
    eprintln!("MIS01: {dist:.1} m in 600 ticks, speed {:.2}, gear {}, min contacts {min_contacts}, worst wheel gap {worst_gap:.3}, surfaces {surfaces:?}, end {:?}", car.speed, car.gear, car.pos);
    assert!(car.speed > 20.0, "the car is driving, speed {}", car.speed);
    assert!(dist > 100.0, "it covered ground: {dist} m");
    assert_eq!(min_contacts, 4, "all four wheels in contact every tick");
    // The road: every wheel's triangle is surface 1, kPAVED [M the WSurface order; D the track].
    assert_eq!(surfaces.into_iter().collect::<Vec<_>>(), vec![1], "only PAVED under the wheels");
}
