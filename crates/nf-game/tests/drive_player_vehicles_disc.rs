//! The player's vehicle per driving mission and phase, and the MIS01 helicopter flying its
//! opening path, through the real disc (`NF_ISO`; prints "skip" without it).
//! Code: `nf_formats::drive_vehicle`, `nf_game::driving::path_drive`. Every pinned value is
//! [D] (measured from these bytes); the rules are cited in the modules [M].


use nf_formats::bigf::Bigf;
use nf_formats::crp::Crp;
use nf_formats::drive_script::MapScript;
use nf_formats::drive_vehicle::*;
use nf_formats::iso9660::IsoImage;
use nf_game::driving::flight::{self, PathFollower, IDENTITY};
use nf_game::driving::path_drive::*;
use nf_game::driving::sub_physics::{DrivePass, VehicleAttrs};
use nf_game::driving::{CarParams, CollisionGround};

struct Mission {
    entries: Vec<(String, Vec<u8>)>,
    script: MapScript,
    track: String,
}

impl Mission {
    fn atr(&self, name: &str) -> Option<String> {
        let want = format!("data\\sim\\attrib\\pvehicle\\{name}.atr");
        self.entries.iter().find(|(n, _)| n.eq_ignore_ascii_case(&want)).map(|(_, b)| String::from_utf8_lossy(b).into_owned())
    }
    fn entry(&self, name: &str) -> Option<&[u8]> {
        self.entries.iter().find(|(n, _)| n.eq_ignore_ascii_case(name)).map(|(_, b)| b.as_slice())
    }
}

fn load(iso: &IsoImage, viv: &str) -> Option<Mission> {
    let e = iso.find(&format!("/DRIVING/{viv}.VIV"))?;
    let raw = iso.read_bytes(e.lba as u64 * 2048, e.size as u64).ok()?;
    let g = Bigf::parse(&raw, raw.len() as u64).ok()?;
    let mut entries = Vec::new();
    for x in &g.entries {
        let b = &raw[x.offset as usize..(x.offset + x.size) as usize];
        if b.iter().take(64).all(|&c| c == 0) {
            continue;
        }
        let packed = b.len() >= 2 && (b[0] & 0x3e) == 0x10 && b[1] == 0xfb;
        let b = if packed { nf_formats::refpack::decompress(b).unwrap_or_else(|_| b.to_vec()) } else { b.to_vec() };
        entries.push((x.name.clone(), b));
    }
    for (n, b) in &entries {
        let l = n.to_ascii_lowercase();
        if !(l.starts_with("data\\track\\") && l.ends_with(".crp")) {
            continue;
        }
        let Ok(crp) = Crp::parse(b) else { continue };
        let Ok(script) = MapScript::load(&crp) else { continue };
        if script.chunks.contains_key("MSet") {
            let track = n.clone();
            return Some(Mission { entries: entries.clone(), script, track });
        }
    }
    None
}

fn iso() -> Option<IsoImage> {
    let Some(p) = std::env::var_os("NF_ISO") else { eprintln!("skip: NF_ISO not set"); return None };
    Some(IsoImage::open(&p).expect("open NF_ISO"))
}

#[test]
fn every_mission_chooses_its_player_vehicle() {
    let Some(iso) = iso() else { return };
    let mut seen = 0;
    // (archive, the MSet spawn vehicle, every vehicle the script can put the player in, the
    // lists that put the player on / off a spline) [D].
    type Case = (&'static str, &'static str, &'static [&'static str], &'static [u16], &'static [u16]);
    let cases: [Case; 8] = [
        ("MIS01", "helicopter", &["helicopter", "vanquish"], &[686], &[742, 776, 789]),
        ("MIS3", "supersnow", &["supersnow"], &[222, 256, 270, 288], &[108, 119, 259, 289, 296]),
        ("MIS4", "vanquishalps", &["vanquishalps"], &[268], &[]),
        ("MIS11", "vanquishsub", &["vanquishsub"], &[34], &[]),
        ("MIS13A", "jungle_truck", &["jungle_truck"], &[46], &[47]),
        ("MIS13B", "ultralight", &["ultralight"], &[79], &[]),
        ("MIS13C", "fodbase", &["fodbase"], &[18], &[]),
        ("RACE", "cobra_player", &["cobra_player"], &[225], &[]),
    ];
    for (viv, spawn, all_want, on, off) in cases {
        let Some(m) = load(&iso, viv) else { eprintln!("skip {viv}: no track extents"); continue };
        seen += 1;
        let name = mset_vehicle(&m.script.chunks["MSet"]).unwrap();
        let sec = |n: &str| m.atr(n).and_then(|t| secondary_type(&t));
        let all = mission_vehicles(&m.script, SpawnSwitches::default(), &sec);
        let lists = vehicle_lists(&m.script);
        eprintln!("{viv} {} spawn={name} secondary={:?} vehicles={all:?} lists={lists:?}", m.track, sec(&name));
        assert_eq!(name, spawn, "{viv}: MSet name");
        assert!(m.atr(&name).is_some(), "{viv}: the archive carries {name}.atr");
        assert_eq!(all, all_want.to_vec(), "{viv}: vehicles");
        let to: Vec<u16> = lists.iter().filter(|(_, e)| e.iter().any(|x| matches!(x, VehicleEvent::ToSpline(_)))).map(|l| l.0).collect();
        let end: Vec<u16> = lists.iter().filter(|(_, e)| e.contains(&VehicleEvent::EndSpline)).map(|l| l.0).collect();
        assert_eq!((to.as_slice(), end.as_slice()), (on, off), "{viv}: spline lists");
        // Only MIS01 carries ECustomTransition, and only its spawn vehicle has a SECONDARY_TYPE.
        let transitions: Vec<u16> =
            lists.iter().filter(|(_, e)| e.iter().any(|x| matches!(x, VehicleEvent::Transition(_)))).map(|l| l.0).collect();
        assert_eq!(transitions, if viv == "MIS01" { vec![742, 789] } else { vec![] }, "{viv}: transition lists");
        assert_eq!(sec(&name).is_some(), viv == "MIS01", "{viv}: secondary");
        // The pass the spawn vehicle gets off the spline (driver 0) [M FUN_00188200].
        let attrs = VehicleAttrs::parse_atr(&m.atr(&name).unwrap());
        let want = match viv {
            "MIS11" | "MIS13B" => DrivePass::Sub,
            "MIS3" => DrivePass::Snowmobile,
            _ => DrivePass::PlayerLand,
        };
        assert_eq!(player_pass(&attrs, 0, false, 1.0), want, "{viv}: pass");
        assert_eq!(player_pass(&attrs, 1, true, 1.0), DrivePass::Path, "{viv}: on a spline");
    }
    assert!(seen > 0, "no driving archive on the image");
}

/// Half the drawn LOD-0 geometry's box. The game reads the model header's box
/// (`FUN_001c9778`); this geometry box stands in for it [S].
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

/// The path events of a list for the player's follower (the reading the sniper-phase disc test
/// uses): 202 / 209 start / chain a track, 239 ends it, 116 / 153 / 215 the throttle.
fn run_path_events(s: &MapScript, l: u16, fol: &mut Option<PathFollower>, now: f32) {
    let splines = s.splines();
    for e in &s.lists[&l] {
        let player = e.word(1) == Some(0);
        match e.kind {
            202 | 209 if player => {
                let tr = e.reference(2).and_then(|r| s.spline_of(r)).and_then(|i| s.track_of_spline(&splines[i])).unwrap();
                if e.kind == 202 {
                    *fol = Some(PathFollower::start(tr, &s.tracks, now));
                } else if let Some(f) = fol {
                    f.next = Some(tr);
                }
            }
            239 if player => *fol = None,
            116 | 153 | 215 if player && e.word(2) == Some(0) => {
                if let Some(f) = fol {
                    match e.kind {
                        116 => f.set_desired(e.f32(3).unwrap()),
                        153 => f.set_time_fraction(e.f32(3).unwrap(), &s.tracks),
                        _ => f.change_throttle(e.word(3).unwrap() as i32, e.f32(4).unwrap(), e.f32(5).unwrap()),
                    }
                }
            }
            _ => {}
        }
    }
}

#[test]
fn the_mis01_helicopter_flies_its_opening_path() {
    let Some(iso) = iso() else { return };
    let Some(m) = load(&iso, "MIS01") else { eprintln!("skip: no MIS01 extents"); return };
    let s = &m.script;
    let atr = m.atr("helicopter").unwrap();
    let attrs = VehicleAttrs::parse_atr(&atr);
    let mut params = CarParams::parse_atr(&atr);
    params.half_extents = half_extents(m.entry("data\\car\\model\\helicopter.crp").expect("helicopter.crp"));
    assert_eq!(attrs.mass, 1660.0);
    assert_eq!((params.wheel_radius, params.spring_rest, params.spring_limit), (0.315, 0.35, 0.75));
    let crp = Crp::parse(m.entry(&m.track).unwrap()).unwrap();
    let ground = CollisionGround::new(crp.extract_collision_tris().unwrap());

    let fps = 60;
    let mut veh = PlayerVehicle::spawn(&mset_vehicle(&s.chunks["MSet"]).unwrap(), SpawnSwitches::default());
    let none = |_: &str| None;
    for e in vehicle_lists(s).into_iter().filter(|l| l.0 == 686).flat_map(|l| l.1) {
        veh.apply(&e, &none);
    }
    assert_eq!(veh, PlayerVehicle { name: "helicopter".into(), spline: Some("CARP::<<Map>>::{AISp}::0".into()) });
    let mut fol = None;
    run_path_events(s, 686, &mut fol, flight::clock60(0, fps));
    let fol = fol.expect("list 686 starts the spline");
    assert_eq!((fol.current, fol.desired), (Some(1), 0.71), "path_Helicopter02 = pt 0001, throttle 0.71");
    let mut heli = HeliFlight::new(fol, IDENTITY, params.clone(), attrs.clone());
    assert_eq!(heli.body.pos(), [heli.follower.pose[3][0], heli.follower.pose[3][1], heli.follower.pose[3][2]]);

    // The throttle-0.71 leg with no further script (the triggers are the sniper-phase test's).
    let mut top = 0.0f32;
    let mut skid_ticks = 0;
    let mut loaded_ticks = 0;
    let mut spin_max = 0.0f32;
    let mut samples = Vec::new();
    for tick in 1..=600 {
        heli.step(flight::clock60(tick, fps), fps, &s.tracks, &ground);
        let f = &heli.follower;
        assert_eq!(heli.body.pos(), [f.pose[3][0], f.pose[3][1], f.pose[3][2]], "tick {tick}: the body is on the path");
        assert_eq!(heli.body.velocity, f.velocity);
        assert_eq!(heli.body.momentum, f.velocity.map(|v| v * 1660.0));
        top = top.max(heli.pass.speed);
        spin_max = spin_max.max(heli.pass.wheel_spin);
        if heli.pass.skid {
            skid_ticks += 1;
        }
        if heli.pass.spring.iter().any(|&x| x > 0.0) {
            loaded_ticks += 1;
        }
        if tick % 100 == 0 {
            samples.push((tick, heli.body.pos(), heli.pass.speed, heli.pass.steer, heli.pass.spring, heli.pass.roll));
        }
    }
    eprintln!("top {top} spin_max {spin_max} skid {skid_ticks} loaded {loaded_ticks} follower speed {}", heli.follower.speed);
    for x in &samples {
        eprintln!("{x:?}");
    }
    // Throttle 0.71 on pt 0001: about 23.9 m/s along the river; the wheel spin is that over
    // WHEEL_RADIUS · 6.32 · 60; the heading leaves the velocity by more than 45 degrees on 13
    // ticks; the path runs west-north-west and the body's right axis points against it
    // (steer < 0). The spring lengths depend on the port's ground grid [S] and are not pinned.
    assert!((top - 23.927_55).abs() < 1e-3, "{top}");
    assert!((spin_max - top / (0.315 * 6.32 * 60.0)).abs() < 1e-4, "{spin_max}");
    assert_eq!(skid_ticks, 13);
    assert_eq!(heli.follower.speed, 0.71);
    let want = [
        (100, [780.188, 6.166, 241.534], -0.5758),
        (200, [751.143, 8.671, 268.544], -0.5828),
        (300, [719.945, 1.258, 292.165], -0.5072),
        (400, [692.470, -0.838, 320.719], -0.2050),
        (500, [662.830, -1.260, 347.321], -0.1389),
        (600, [633.339, -0.712, 374.136], -0.0963),
    ];
    for ((t, p, _, steer, _, _), (wt, wp, ws)) in samples.iter().zip(want) {
        assert_eq!(*t, wt);
        assert!((0..3).all(|k| (p[k] - wp[k]).abs() < 2e-3), "tick {t}: {p:?}");
        assert!((steer - ws).abs() < 1e-3, "tick {t}: steer {steer}");
    }

    // The hand-over list 789: off the spline, rebuilt as the Vanquish, the player land pass.
    let sec = |n: &str| m.atr(n).and_then(|t| secondary_type(&t));
    for e in vehicle_lists(s).into_iter().filter(|l| l.0 == 789).flat_map(|l| l.1) {
        veh.apply(&e, &sec);
    }
    assert_eq!(veh, PlayerVehicle { name: "vanquish".into(), spline: None });
    let van = VehicleAttrs::parse_atr(&m.atr("vanquish").unwrap());
    assert_eq!(player_pass(&van, 0, veh.spline.is_some(), 1.0), DrivePass::PlayerLand);
}
