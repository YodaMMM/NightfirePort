//! Disc test for `nf_formats::carpose` (docs/formats/car-model.md §11): over every vehicle model
//! of the nine `/DRIVING/` archives, the draw lists the selector can pick, the attachment kinds
//! and the part matrices each gets, the wheel rig the setup builds, and the `Base` header the rig
//! reads. Opt-in via NF_ISO (a slim image without the `/DRIVING/` extents skips).

use std::collections::BTreeMap;

use nf_formats::carmesh::{car_mesh, Attach, CarMesh, TAG_AS, TAG_BASE};
use nf_formats::carpose::{part_matrix, transform, CarPose, DrawOwner, DrawState, PartMatrix, WheelAttrs, WheelInput, WheelMemory, WheelRig, IDENTITY};
use nf_formats::crp::{tag, Crp};
use nf_formats::iso9660::IsoImage;

const ARCHIVES: [&str; 9] = ["MIS01", "MIS11", "MIS13A", "MIS13B", "MIS13C", "MIS3", "MIS4", "RACE", "MISC"];

struct Car {
    viv: &'static str,
    stem: String,
    crp: Vec<u8>,
    mesh: CarMesh,
}

fn cars() -> Option<Vec<Car>> {
    let Ok(path) = std::env::var("NF_ISO") else {
        eprintln!("skip: NF_ISO not set");
        return None;
    };
    let iso = IsoImage::open(&path).expect("open NF_ISO");
    let mut out = Vec::new();
    for viv in ARCHIVES {
        let e = iso.find(&format!("/DRIVING/{viv}.VIV"))?;
        let g = nf_formats::disc::read_bigf(&iso, e).ok()?;
        for x in &g.entries {
            let n = x.name.to_ascii_lowercase();
            let Some(stem) = n.strip_prefix(r"data\car\model\").and_then(|s| s.strip_suffix(".crp")) else { continue };
            let raw = iso.read_bytes(e.lba as u64 * 2048 + x.offset as u64, x.size as u64).ok()?;
            if raw.iter().take(64).all(|&b| b == 0) {
                eprintln!("skip: slim image");
                return None;
            }
            let crp = if nf_formats::refpack::is_refpack(&raw) { nf_formats::refpack::decompress(&raw).unwrap() } else { raw };
            let mesh = car_mesh(&crp).unwrap_or_else(|e| panic!("{viv} {stem}: {e}"));
            out.push(Car { viv, stem: stem.to_owned(), crp, mesh });
        }
    }
    Some(out)
}

fn ids(v: Vec<&nf_formats::carmesh::CarPart>) -> Vec<usize> { v.iter().map(|p| p.index).collect() }

const PLAYER: DrawOwner = DrawOwner::Vehicle { driver_kind: 0 };
fn st(owner: DrawOwner, interior_view: bool, auto_drive: bool) -> DrawState { DrawState { owner, interior_view, auto_drive } }

#[test]
fn the_selected_lists_split_the_two_body_cars_and_lists_3_4_are_never_picked() {
    let Some(cars) = cars() else { return };
    assert_eq!(cars.len(), 63, "car files in the nine archives [D]");
    // Which vd numbers exist, over all files [D].
    let mut numbers: BTreeMap<u16, usize> = BTreeMap::new();
    for c in &cars {
        for (n, _) in &c.mesh.draw_lists {
            *numbers.entry(*n).or_default() += 1;
        }
    }
    assert_eq!(numbers, BTreeMap::from([(1, 7), (2, 36), (3, 16), (4, 16), (5, 13)]));
    let range = |a: usize, b: usize| (a..=b).collect::<Vec<_>>();
    let mut changed = Vec::new();
    for c in &cars {
        let normal = ids(c.mesh.list_for(&st(PLAYER, false, false)));
        let all = ids(c.mesh.drawn(None));
        // Every selected list names items that exist, and an AI car always draws list 1.
        assert_eq!(ids(c.mesh.list_for(&st(DrawOwner::Vehicle { driver_kind: 2 }, true, true))), normal, "{} {}", c.viv, c.stem);
        assert_eq!(ids(c.mesh.list_for(&st(DrawOwner::Helicopter, false, true))), all);
        if normal != all {
            changed.push(format!("{} {}", c.viv, c.stem));
        }
        if c.stem == "supersnow" {
            // The two overlapping bodies: list 1 in play, list 5 under AutoDrive; both shared the
            // screen when every item was drawn.
            let tail = range(25, 31);
            // (item 14 is hidden: +0x0c = 0x41)
            assert_eq!(normal, [range(1, 13), range(15, 17), tail.clone()].concat(), "{}", c.viv);
            assert_eq!(ids(c.mesh.list_for(&st(PLAYER, false, true))), [vec![0], range(18, 24), tail.clone()].concat());
            assert_eq!(ids(c.mesh.list_for(&st(PLAYER, true, false))), tail, "an interior view keeps only 25-31");
            assert_eq!(all, [range(0, 13), range(15, 31)].concat());
        }
        if c.stem == "vanquish" {
            let interior = ids(c.mesh.list_for(&st(PLAYER, true, false)));
            assert_eq!(interior.len(), 30, "{}", c.viv);
            // The 0xef (dial) items: 3, 4, 5, 7, 20 are in the interior list and not in list 1;
            // 21 (dial 0, the steer slot) is in list 1 only.
            let dials: Vec<usize> = c.mesh.parts.iter().filter(|p| matches!(p.attach, Attach::Other(0xef, _))).map(|p| p.index).collect();
            assert_eq!(dials, [3, 4, 5, 7, 20, 21]);
            assert_eq!(dials.iter().filter(|i| interior.contains(i)).copied().collect::<Vec<_>>(), [3, 4, 5, 7, 20]);
            assert_eq!(dials.iter().filter(|i| normal.contains(i)).copied().collect::<Vec<_>>(), [21]);
        }
    }
    // Models whose list 1 differs from drawing every item [D]. The helicopter's list 1 holds two
    // geometry-less items, but an AHelicopter never selects a list.
    assert_eq!(changed, ["MIS01 helicopter", "MIS01 vanquish", "MIS3 supersnow", "MIS4 supersnow", "MIS4 vanquish", "RACE supersnow"]);
}

#[test]
fn every_part_kind_on_the_disc_has_a_matrix_and_the_spin_keeps_wheels_in_place() {
    let Some(cars) = cars() else { return };
    let mut kinds: BTreeMap<String, usize> = BTreeMap::new();
    let mut unported = 0;
    let mut flag10 = 0;
    for c in &cars {
        let rest = CarPose::rest(&c.mesh);
        let mut spun = rest.clone();
        spun.frame = 37.0;
        for m in spun.skel.iter_mut() {
            *m = nf_formats::carpose::rot_x_turns(0.3);
        }
        for p in &c.mesh.parts {
            let k = match p.attach {
                Attach::Fixed => "ff".to_string(),
                Attach::Skel(_) => "fe".to_string(),
                Attach::Other(k, _) => format!("{k:02x}"),
            };
            *kinds.entry(k).or_default() += 1;
            if p.flags & 0x10 != 0 {
                flag10 += 1;
            }
            match (part_matrix(p, &rest), part_matrix(p, &spun)) {
                (PartMatrix::Drawn(a), PartMatrix::Drawn(b)) => {
                    // At rest every kind is the item's own matrix (identity Skel, frame 0, dials 0).
                    let own = nf_formats::carpose::own_matrix(p);
                    assert!((0..4).all(|i| (0..4).all(|j| (a[i][j] - own[i][j]).abs() < 1e-6)), "{} {} item {}", c.viv, c.stem, p.index);
                    // Rotations act in the part frame: the part's origin does not move.
                    let (o0, o1) = (transform(&a, [0.0; 3]), transform(&b, [0.0; 3]));
                    assert!(o0.iter().zip(o1).all(|(x, y)| (x - y).abs() < 1e-5), "{} {} item {}", c.viv, c.stem, p.index);
                }
                (PartMatrix::NotDrawn, PartMatrix::NotDrawn) => assert!(p.hidden()),
                (PartMatrix::Unported(_), _) => unported += 1,
                other => panic!("{other:?}"),
            }
        }
    }
    assert_eq!(
        kinds,
        BTreeMap::from([
            ("ee".into(), 1),
            ("ef".into(), 13),
            ("f3".into(), 16),
            ("fb".into(), 11),
            ("fc".into(), 16),
            ("fe".into(), 333),
            ("ff".into(), 725),
        ])
    );
    assert_eq!(flag10, 0, "no item reads its value from the run-time entry");
    assert_eq!(unported, 1, "only the camera-facing 0xee item (the MIS01 helicopter)");
}

#[test]
fn the_wheel_rig_finds_every_wheel_and_every_article_names_its_base() {
    let Some(cars) = cars() else { return };
    let mut rigs = 0;
    let mut missing = Vec::new();
    for c in &cars {
        // Every `as` payload starts with the `Base` tag; the Base payload is 0x20-byte rows with
        // the 0xffffffff placeholder first and the box min/max at words 4-6 / 8-10 [D].
        let crp = Crp::parse(&c.crp).unwrap();
        let arti = crp.pool().unwrap().into_iter().find(|r| r.tag == tag::ARTI).unwrap();
        let (s, e) = arti.window(crp.data).unwrap();
        for r in crp.window_records(s, e).unwrap() {
            if r.tag_code() == TAG_AS {
                let p = crp.payload(&r).unwrap();
                assert_eq!(u32::from_le_bytes(p[0..4].try_into().unwrap()), TAG_BASE, "{} {}", c.viv, c.stem);
            }
        }
        let b = c.mesh.base.unwrap_or_else(|| panic!("{} {}: no Base", c.viv, c.stem));
        assert!(b[0].is_nan() && (0..3).all(|k| b[4 + k] <= b[8 + k]), "{} {} {b:?}", c.viv, c.stem);
        let wheels = ["leftfrontwheel", "rightfrontwheel", "leftrearwheel", "rightrearwheel"];
        if !wheels.iter().all(|w| c.mesh.skel.iter().any(|s| s == w)) {
            continue;
        }
        rigs += 1;
        let rig = WheelRig::new(&c.mesh, false);
        // Each wheel's offset comes from an item that really is that wheel.
        for k in 0..4 {
            let found = c.mesh.parts.iter().find(|p| p.attach == Attach::Skel(rig.wheel[k] - 1));
            match found {
                Some(p) if (rig.off[k] - (p.origin[1] - c.mesh.wheel_ref_height().unwrap())).abs() < 1e-6 => {}
                _ => missing.push(format!("{} {} wheel {k}", c.viv, c.stem)),
            }
        }
        // One update with the wheels rolled: every wheel item turns about its own centre.
        let attrs = WheelAttrs { rest: 0.24, limit_draw: 0.125, wheel_radius: 0.36, is_sub: false };
        let mut pose = CarPose::rest(&c.mesh);
        let input = WheelInput { spin: [0.25, 0.25], steer: 0.0, compression: [0.0; 4], slow_steer: false, fins: None };
        rig.update(&attrs, &input, &mut WheelMemory::default(), &mut pose.skel);
        for k in 0..4 {
            assert_ne!(pose.skel[rig.wheel[k] as usize - 1], IDENTITY);
        }
    }
    assert_eq!(rigs, 40, "cars with the four wheel names [D]");
    // The scan does not restart per wheel (WheelRig::new): on the two buses the front wheels'
    // items come after the rear ones, so both rear scans run off the end and keep offset 0.
    assert_eq!(missing, ["MISC bus_paris wheel 2", "MISC bus_paris wheel 3", "MISC bus_tokyo wheel 2", "MISC bus_tokyo wheel 3"]);
}
