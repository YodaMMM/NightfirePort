//! AI cars on the land physics over the real tracks (opt-in: `NF_ISO=<image>`): the traffic
//! brain (`AiDriver` = FUN_0012e808 → FUN_00130508 → FUN_00133ab0) driving `Car::step` over the
//! track's collision triangles and 2-D walls, with the game's AI pass (`FUN_0018ecb8`), the
//! model's hull points and the land pass B/C responses. Each car runs 3600 ticks (60 s at the
//! 60 Hz step) and must never end below the ground, never roll, and never leave the road
//! network by more than one lane width.
//!
//! The vehicles are the tracks' own [D]: the traffic hatchback (`hatchback.atr` +
//! `hatchback.crp`, kind 3, the traffic profile 0xc at 7.5 m/s) and the hostile sedan
//! (`paris_henchbigbox` on MIS01, `paris_hench.atr` with the default sedan body elsewhere,
//! kind 2, the road-limit profile 0xf at the segment's limit).

use nf_formats::crp::Crp;
use nf_formats::iso9660::IsoImage;
use nf_game::driving::ai::{spawn_lane, AiCar, AiDriver, DriveRng, PROFILE_ROAD_LIMIT};
use nf_game::driving::road::{lane_offset, LaneSet, RoadNet};
use nf_game::driving::{Car, CarParams, CollisionGround, DriverKind, Ground, Rigid, DT};

fn unpack(raw: Vec<u8>) -> Vec<u8> {
    if raw.len() >= 2 && (raw[0] & 0x3e) == 0x10 && raw[1] == 0xfb {
        nf_formats::refpack::decompress(&raw).expect("refpack")
    } else {
        raw
    }
}

struct Track {
    viv: &'static str,
    crp: Vec<u8>,
    files: Vec<(String, Vec<u8>)>,
}

fn load(iso: &IsoImage, viv: &'static str) -> Track {
    let entry = iso.find(&format!("/DRIVING/{viv}.VIV")).expect("viv");
    let bigf = nf_formats::disc::read_bigf(iso, entry).expect("bigf");
    let base = entry.lba as u64 * 2048;
    let mut crp = Vec::new();
    let mut files = Vec::new();
    for e in &bigf.entries {
        let n = e.name.to_ascii_lowercase();
        let want_track = n.contains("data\\track\\") && n.ends_with(".crp");
        let want_car = (n.contains("pvehicle") && n.ends_with(".atr")) || (n.contains("data\\car\\model\\") && n.ends_with(".crp"));
        if !(want_track || want_car) { continue; }
        let bytes = unpack(iso.read_bytes(base + e.offset as u64, e.size as u64).expect("read"));
        if want_track { crp = bytes; } else { files.push((n, bytes)); }
    }
    assert!(!crp.is_empty(), "{viv}: no track .crp");
    Track { viv, crp, files }
}

impl Track {
    fn file(&self, suffix: &str) -> Option<&[u8]> {
        self.files.iter().find(|(n, _)| n.ends_with(suffix)).map(|(_, b)| b.as_slice())
    }
    /// The car's .atr with its model's header box and hull points [D].
    fn car(&self, atr: &str, model: &str) -> CarParams {
        let text = self.file(&format!("\\{atr}.atr")).unwrap_or_else(|| panic!("{}: no {atr}.atr", self.viv));
        let mut p = CarParams::parse_atr(&String::from_utf8_lossy(text));
        let m = self.file(&format!("\\{model}.crp")).unwrap_or_else(|| panic!("{}: no {model}.crp", self.viv));
        p.half_extents = nf_formats::carmesh::header_half_extents(m).expect("box");
        p.hull = nf_formats::carmesh::hull_points(m).expect("hull");
        p
    }
}

#[derive(Debug, Default)]
struct Outcome {
    runs: usize,
    failures: Vec<String>,
    worst_off: f32,
    dist: f32,
    /// Runs ended early because the brain orbits a carrot it cannot reach (see `STALL_TICKS`).
    stalls: Vec<String>,
}

#[derive(Clone, Copy)]
struct Setup {
    kind: DriverKind,
    road_limit: bool,
    /// false = the configuration before this branch: the player pass, the box stand-ins for the
    /// hull and the steep-triangle wall stand-in.
    game_parts: bool,
}

/// The ground height under `p` from above (any triangle up to 50 m over it).
fn surface_under(g: &CollisionGround, p: [f32; 3]) -> Option<f32> {
    let q = [p[0], p[1] + 50.0, p[2]];
    g.wheel_plane(q).map(|pl| pl.v0[1] - (pl.normal[0] * (p[0] - pl.v0[0]) + pl.normal[2] * (p[2] - pl.v0[2])) / pl.normal[1])
}

fn run(t: &Track, atr: &str, model: &str, s: Setup) -> Outcome {
    let crp = Crp::parse(&t.crp).expect("crp");
    let tris = crp.extract_collision_tris().expect("tris");
    let walls = crp.extract_collision_walls().expect("walls");
    let g = if s.game_parts { CollisionGround::with_walls(tris, walls) } else { CollisionGround::new(tris) };
    let net = RoadNet::parse_crp(&crp).expect("roads");
    let mut params = t.car(atr, model);
    if !s.game_parts { params.hull.clear(); }
    let lane_set = LaneSet::Default;
    // The road network: dense centre-line samples, each with the outermost lane centre of its
    // segment (lane_offset of the largest lane index) [M 0x2257c0 lane offsets; D the RNgp].
    let mut samples: Vec<([f32; 3], f32)> = Vec::new();
    for sg in 0..net.segs.len() {
        let seg = net.seg(sg).unwrap();
        let outer = seg.lanes_neg().unsigned_abs().max(seg.lanes_pos().unsigned_abs()) as i8;
        let w = lane_offset(lane_set, seg.road_class(), outer);
        for k in 0..=40 {
            if let Ok(p) = net.seg_point(sg, k as f32 / 40.0) { samples.push((p, w)); }
        }
    }
    // A junction node (three or more segments meeting) is road too: a disc one lane wider
    // than the widest road meeting there, for the turn across it [D the RNgp node links].
    let mut node_w: std::collections::BTreeMap<u16, (u32, f32)> = std::collections::BTreeMap::new();
    for sg in 0..net.segs.len() {
        let seg = net.seg(sg).unwrap();
        let outer = seg.lanes_neg().unsigned_abs().max(seg.lanes_pos().unsigned_abs()) as i8;
        let w = lane_offset(lane_set, seg.road_class(), outer);
        for n in [seg.node_a(), seg.node_b()] {
            let e = node_w.entry(n).or_insert((0, 0.0));
            e.0 += 1;
            e.1 = e.1.max(w);
        }
    }
    for (n, (count, w)) in node_w {
        if count >= 3 {
            if let Ok(node) = net.node(n as usize) { samples.push((node.pos(), w + LANE_WIDTH)); }
        }
    }
    let rigid = Rigid::game_defaults();
    let mut out = Outcome::default();
    let step = (net.segs.len() / 14).max(1);
    for seg in (0..net.segs.len()).step_by(step) {
        let Ok(Some((lane, fwd))) = spawn_lane(&net, seg, 0) else { continue };
        // Mid-segment (the game draws the start t at random [M 0x13c5e8:22]), so the car starts
        // facing along its piece rather than across a junction.
        let Ok(mut brain) = AiCar::traffic(&net, seg, 0.5, lane, fwd, lane_set) else { continue };
        if s.road_limit { brain.profile = PROFILE_ROAD_LIMIT; }
        let yaw = brain.nav.dir[0].atan2(brain.nav.dir[2]);
        let mut car = Car::new(params.clone(), brain.nav.pos, yaw);
        // Spawn only where the track has ground under all four wheels (the game's placement
        // FUN_00187eb8 queries each wheel's ground [M]); the body set at rest above it.
        let ys: Vec<Option<f32>> = (0..4).map(|i| surface_under(&g, car.wheel_world(i))).collect();
        if ys.iter().any(|y| y.is_none_or(|y| (y - brain.nav.pos[1]).abs() > 1.0)) { continue; }
        let gy = ys.iter().map(|y| y.unwrap()).fold(f32::MIN, f32::max);
        car.pos[1] = gy + car.rest_height() + 0.02;
        car.kind = if s.game_parts { s.kind } else { DriverKind::Player };
        let mut drv = AiDriver { brain, car, world: nf_game::driving::FlatWorld::paved(), rigid: rigid.clone(), rng: DriveRng::game_reset() };
        let mut rng = DriveRng::game_reset();
        out.runs += 1;
        let start = drv.car.pos;
        let mut crawl = 0;
        for tick in 0..3600 {
            let cmd = match drv.step_land(&net, &g, DT, None, &mut rng) {
                Ok(c) => c,
                Err(e) => {
                    out.failures.push(format!("{} seg {seg} tick {tick}: brain {e}", t.viv));
                    break;
                }
            };
            let c = &drv.car;
            crawl = if c.speed < 2.0 && cmd.steer.abs() >= 0.95 { crawl + 1 } else { 0 };
            if crawl >= STALL_TICKS {
                out.stalls.push(format!("{} seg {seg} tick {tick} at {:?}", t.viv, c.pos));
                break;
            }
            // Below the ground: a surface lies over the body centre within 50 m, and the body
            // centre is under it by more than the body's half height (the hull would be through).
            if let Some(y) = surface_under(&g, c.pos) {
                let near_above = g.wheel_plane([c.pos[0], c.pos[1] + 1.0, c.pos[2]]).is_some();
                if !near_above && y > c.pos[1] + c.params.half_extents[1] {
                    out.failures.push(format!("{} seg {seg} tick {tick}: below the ground at {:?} (surface {y})", t.viv, c.pos));
                    break;
                }
            } else if c.pos[1] < start[1] - 30.0 {
                out.failures.push(format!("{} seg {seg} tick {tick}: fell off the mesh to {:?}", t.viv, c.pos));
                break;
            }
            if c.rot[1][1] < 0.0 {
                out.failures.push(format!("{} seg {seg} tick {tick}: rolled over (up.y {})", t.viv, c.rot[1][1]));
                break;
            }
            let off = samples
                .iter()
                .map(|(q, w)| (((q[0] - c.pos[0]).powi(2) + (q[2] - c.pos[2]).powi(2)).sqrt() - w - LANE_WIDTH * 0.5 - c.params.half_extents[0]).max(0.0))
                .fold(f32::MAX, f32::min);
            out.worst_off = out.worst_off.max(off);
            if off > LANE_WIDTH {
                out.failures.push(format!("{} seg {seg} tick {tick}: {off:.1} beyond the road edge at {:?}", t.viv, c.pos));
                break;
            }
        }
        out.dist += ((drv.car.pos[0] - start[0]).powi(2) + (drv.car.pos[2] - start[2]).powi(2)).sqrt();
    }
    out
}

/// A brain stall: below 2 m/s at full lock for 4 s. The carrot sits abeam inside the car's
/// turning circle (0.2 rad lock, about 13 m) and FUN_00133ab0 holds the 0.5 m/s floor, so the
/// car crawls round it. The game's stuck re-join (FUN_00131770) that would end this is not
/// ported, so such a run is stopped and counted, not failed; the physics checks above it held.
const STALL_TICKS: u32 = 240;

/// One lane: the Default lane table's slot spacing (2 → 6 → 10 → 14, D .data 0x33a678). The
/// road edge is the outermost lane centre plus half of it; a car fails when the side of its body
/// nearest the road (centre − the box's half width) is more than one lane width past that edge.
const LANE_WIDTH: f32 = 4.0;

const CARS: [(&str, &str, &str, &str, &str); 3] = [
    // (viv, traffic atr, traffic model, hostile atr, hostile model)
    ("MIS01", "hatchback", "hatchback", "paris_henchbigbox", "paris_henchbigbox"),
    ("MIS4", "hatchback", "hatchback", "paris_hench", "sedan"),
    ("RACE", "hatchback", "hatchback", "paris_hench", "sedan"),
];

fn iso() -> Option<IsoImage> {
    let p = std::env::var("NF_ISO").ok()?;
    IsoImage::open(&p).ok()
}

#[test]
fn ai_cars_stay_on_the_roads_for_3600_ticks() {
    let Some(iso) = iso() else { eprintln!("skip: NF_ISO unset"); return };
    let mut all = Vec::new();
    for (viv, ta, tm, ha, hm) in CARS {
        let t = load(&iso, viv);
        for (atr, model, kind, road_limit) in [(ta, tm, DriverKind::Traffic, false), (ha, hm, DriverKind::Hostile, true)] {
            let o = run(&t, atr, model, Setup { kind, road_limit, game_parts: true });
            eprintln!("{viv} {atr} {kind:?}: {} runs, {:.0} m driven, worst {:.1} past the road edge, stalls {:?}, failures {:?}", o.runs, o.dist, o.worst_off, o.stalls, o.failures);
            assert!(o.stalls.len() <= 2, "{viv} {atr}: {} brain stalls", o.stalls.len());
            assert!(o.runs >= 8, "{viv} {atr}: only {} spawns had ground", o.runs);
            assert!(o.dist > 1000.0, "{viv} {atr}: the cars drove ({:.0} m)", o.dist);
            all.extend(o.failures);
        }
    }
    assert!(all.is_empty(), "{} failures: {all:#?}", all.len());
}

/// Sister: the same runs in the configuration the land step had before this branch (every car
/// on the player pass, the hull's box stand-ins, the steep-triangle wall stand-in) do fail,
/// so the assertions above can fail.
#[test]
fn the_pre_branch_configuration_fails_the_same_runs() {
    let Some(iso) = iso() else { eprintln!("skip: NF_ISO unset"); return };
    let mut fails = 0;
    for (viv, ta, tm, ha, hm) in CARS {
        let t = load(&iso, viv);
        for (atr, model, road_limit) in [(ta, tm, false), (ha, hm, true)] {
            let o = run(&t, atr, model, Setup { kind: DriverKind::Player, road_limit, game_parts: false });
            eprintln!("pre-branch {viv} {atr}: {} runs, failures {}: {:?}", o.runs, o.failures.len(), o.failures.first());
            fails += o.failures.len();
        }
    }
    assert!(fails > 0, "the old configuration should fail at least one run");
}

/// The two disc lists this branch reads [D]: the wall records of every `ci` item (MIS01 24935,
/// MIS4 and RACE 6275 each) and a car model's hull (the vanquish: 12 points, the first at
/// x −0.821, z 2.306, its y 0.208 lowered by Base+0x38 − half height = 1.287 − 0.578).
#[test]
fn the_wall_lists_and_the_hull_read_off_the_disc() {
    let Some(iso) = iso() else { eprintln!("skip: NF_ISO unset"); return };
    for (viv, want) in [("MIS01", 24935usize), ("MIS4", 6275), ("RACE", 6275)] {
        let t = load(&iso, viv);
        let walls = Crp::parse(&t.crp).unwrap().extract_collision_walls().unwrap();
        assert_eq!(walls.len(), want, "{viv} wall count");
        assert!(walls.iter().all(|w| w.a[1] <= w.b[1]), "{viv}: the ends' heights are sorted");
        let tall = walls.iter().filter(|w| w.b[1] - w.a[1] > 0.5).count();
        assert!(tall * 2 > walls.len(), "{viv}: most walls stand taller than half a metre ({tall}/{})", walls.len());
    }
    let t = load(&iso, "MIS01");
    let hull = nf_formats::carmesh::hull_points(t.file(r"\vanquish.crp").unwrap()).unwrap();
    assert_eq!(hull.len(), 12);
    assert!((hull[0][0] + 0.821).abs() < 1e-3 && (hull[0][2] - 2.306).abs() < 1e-3, "{:?}", hull[0]);
    assert!((hull[0][1] - (0.208 - (1.287 - 0.578))).abs() < 2e-3, "the lowered y {}", hull[0][1]);
}
