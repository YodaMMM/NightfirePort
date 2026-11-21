//! Disc test: every vehicle model of the nine `/DRIVING/` archives (docs/formats/car-model.md
//! §11.6): it decodes, names the sheet beside it, its strips' textures resolve in that sheet, which
//! run-time part kinds it carries, and, for the player vehicles the missions spawn (`MSet`, see
//! docs/systems/drive-player-vehicles.md), which items each draw owner draws and how the moving
//! parts turn. Opt-in via NF_ISO (a slim image without the `/DRIVING/` extents skips).

use std::collections::{BTreeMap, BTreeSet};

use nf_formats::carmesh::{car_mesh, Attach, CarMesh};
use nf_formats::carpose::{facing_matrix, part_matrix, transform, CarPose, DrawOwner, DrawState, PartMatrix};
use nf_formats::iso9660::IsoImage;

const ARCHIVES: [&str; 9] = ["MIS01", "MIS11", "MIS13A", "MIS13B", "MIS13C", "MIS3", "MIS4", "RACE", "MISC"];

struct Car {
    viv: &'static str,
    stem: String,
    mesh: CarMesh,
    /// The entry names of `data\car\model\<sheet>` in the same archive, `None` when absent.
    sheet: Option<BTreeSet<String>>,
}

fn read(iso: &IsoImage, base: u64, off: u32, size: u32) -> Option<Vec<u8>> {
    let raw = iso.read_bytes(base + off as u64, size as u64).ok()?;
    if raw.iter().take(64).all(|&b| b == 0) {
        return None;
    }
    Some(if nf_formats::refpack::is_refpack(&raw) { nf_formats::refpack::decompress(&raw).unwrap() } else { raw })
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
        let base = e.lba as u64 * 2048;
        for x in &g.entries {
            let n = x.name.to_ascii_lowercase();
            let Some(stem) = n.strip_prefix(r"data\car\model\").and_then(|s| s.strip_suffix(".crp")) else { continue };
            let Some(crp) = read(&iso, base, x.offset, x.size) else {
                eprintln!("skip: slim image");
                return None;
            };
            let mesh = car_mesh(&crp).unwrap_or_else(|e| panic!("{viv} {stem}: {e}"));
            let sheet = mesh.sheet.as_ref().and_then(|s| {
                let want = format!(r"data\car\model\{s}");
                let se = g.entries.iter().find(|y| y.name.eq_ignore_ascii_case(&want))?;
                let ssh = nf_formats::ssh::Ssh::parse(&read(&iso, base, se.offset, se.size)?).ok()?;
                Some(ssh.textures.into_iter().map(|t| t.name).collect())
            });
            out.push(Car { viv, stem: stem.to_owned(), mesh, sheet });
        }
    }
    Some(out)
}

fn kind(a: Attach) -> u8 {
    match a {
        Attach::Fixed => 0xff,
        Attach::Skel(_) => 0xfe,
        Attach::Other(k, _) => k,
    }
}

fn ids(v: Vec<&nf_formats::carmesh::CarPart>) -> Vec<usize> { v.iter().map(|p| p.index).collect() }

fn st(owner: DrawOwner, interior_view: bool, auto_drive: bool) -> DrawState { DrawState { owner, interior_view, auto_drive } }

const PLAYER: DrawOwner = DrawOwner::Vehicle { driver_kind: 0 };

#[test]
fn every_vehicle_model_decodes_names_its_own_sheet_and_resolves_its_textures() {
    let Some(cars) = cars() else { return };
    assert_eq!(cars.len(), 63);
    let mut missing: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let mut kinds: BTreeSet<u8> = BTreeSet::new();
    let mut anim_items = 0;
    for c in &cars {
        // The ns#0 record names `<stem>.ssh`, and that sheet sits beside the model in the same
        // archive for every car file [D]: no model needs another archive's or a shared sheet.
        assert_eq!(c.mesh.sheet.as_deref(), Some(format!("{}.ssh", c.stem).as_str()), "{} {}", c.viv, c.stem);
        let sheet = c.sheet.as_ref().unwrap_or_else(|| panic!("{} {}: sheet not in the archive", c.viv, c.stem));
        assert!(!c.mesh.parts.is_empty());
        for p in &c.mesh.parts {
            kinds.insert(kind(p.attach));
            // The `as` +0x08 word, when set, is an `an` short tag; FUN_001a30c0 reads it only
            // for kinds below 100, which no item has (below).
            assert!(p.anim == 0 || p.anim >> 16 == 0x616e, "{} {} item {}: {:#x}", c.viv, c.stem, p.index, p.anim);
            if p.anim != 0 {
                anim_items += 1;
            }
            for s in &p.strips {
                for t in s.texture.iter().chain(s.slot_textures.values()) {
                    if !sheet.contains(t) {
                        missing.entry(format!("{} {}", c.viv, c.stem)).or_default().insert(t.clone());
                    }
                }
            }
        }
    }
    println!("items naming an `an` record: {anim_items}");
    // The run-time kinds on the disc; none below 100 (the keyframe kinds) [D].
    assert_eq!(kinds, BTreeSet::from([0xee, 0xef, 0xf3, 0xfb, 0xfc, 0xfe, 0xff]));
    // The strip textures no entry of the model's own sheet holds [D]: only `blck` on the three
    // ultralight models (docs/formats/car-model.md §9 notes no sheet of their archives has it).
    let got: Vec<String> = missing.iter().map(|(k, v)| format!("{k}: {}", v.iter().cloned().collect::<Vec<_>>().join(","))).collect();
    assert_eq!(got, ["MIS13A staticultralight: blck", "MIS13A ultralight: blck", "MIS13B ultralight: blck", "MIS13C ultralightbig: blck"]);
}

#[test]
fn the_player_vehicles_draw_lists_per_owner() {
    let Some(cars) = cars() else { return };
    let car = |viv: &str, stem: &str| &cars.iter().find(|c| c.viv == viv && c.stem == stem).unwrap_or_else(|| panic!("{viv} {stem}")).mesh;
    let all = |m: &CarMesh| ids(m.drawn(None));
    let verts = |m: &CarMesh, list: Vec<&nf_formats::carmesh::CarPart>| -> usize { let _ = m; list.iter().map(|p| p.strips.iter().map(|s| s.positions.len()).sum::<usize>()).sum() };

    // MIS01 helicopter, spawned as the player's pvehicle (driver kind 0, or 1 on the spline):
    // its vd lists 1-4 all hold items 8 and 9, which have no model. So outside AutoDrive the
    // player-driven helicopter draws no geometry; under AutoDrive (no vd#5) every item draws.
    let h = car("MIS01", "helicopter");
    for driver_kind in [0, 1] {
        let me = DrawOwner::Vehicle { driver_kind };
        assert_eq!(ids(h.list_for(&st(me, false, false))), [8, 9]);
        assert_eq!(ids(h.list_for(&st(me, true, false))), [8, 9]);
        assert_eq!(verts(h, h.list_for(&st(me, false, false))), 0, "items 8 and 9 have no LOD");
        assert_eq!(ids(h.list_for(&st(me, false, true))), (0..10).collect::<Vec<_>>());
    }
    assert_eq!(ids(h.list_for(&st(DrawOwner::Vehicle { driver_kind: 2 }, false, false))), [8, 9]);
    assert_eq!(ids(h.list_for(&st(DrawOwner::Helicopter, false, false))), (0..10).collect::<Vec<_>>());
    assert_eq!(verts(h, h.drawn(None)), 2913);

    // MIS13A / MISC jungle_truck: only vd#2 (the interior view: items 0 and 3-35); list 1 and 5
    // are absent, so outside an interior view every item but the hidden 24 draws.
    for viv in ["MIS13A", "MISC"] {
        let j = car(viv, "jungle_truck");
        let every: Vec<usize> = (0..36).filter(|&i| i != 24).collect();
        assert_eq!(all(j), every);
        assert_eq!(ids(j.list_for(&st(PLAYER, false, false))), every, "{viv}");
        assert_eq!(ids(j.list_for(&st(PLAYER, false, true))), every);
        let interior: Vec<usize> = [0].into_iter().chain(3..36).filter(|&i| i != 24).collect();
        assert_eq!(ids(j.list_for(&st(PLAYER, true, false))), interior, "the two rearlicense items 1-2 leave");
        assert_eq!(ids(j.list_for(&st(DrawOwner::Vehicle { driver_kind: 2 }, true, false))), every);
        assert!(j.parts[24].hidden());
    }

    // MIS13C fodbase: one fixed item, one strip of three vertices textured `whit`, no lists.
    let f = car("MIS13C", "fodbase");
    assert_eq!(f.parts.len(), 1);
    assert!(f.draw_lists.is_empty());
    let s = &f.parts[0].strips;
    assert_eq!((s.len(), s[0].positions.len(), s[0].texture.as_deref()), (1, 3, Some("whit")));
    let span = s[0].positions.iter().flat_map(|a| s[0].positions.iter().map(move |b| ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt())).fold(0.0f32, f32::max);
    println!("fodbase triangle {:?} span {span}", s[0].positions);
    assert!(span < 0.5, "a single small triangle: {span}");

    // Every other spawn vehicle of the table (MIS3 supersnow, MIS4 vanquishalps → vanquish.crp,
    // MIS11 vanquishsub, MIS13B ultralight, RACE cobra_player → paradis_car.crp) [D atr
    // RENDER_FILENAME]: the item count each owner draws.
    let n = |m: &CarMesh, s: DrawState| m.list_for(&s).len();
    let rows: Vec<(&str, &str, [usize; 4])> = [("MIS3", "supersnow"), ("MIS4", "vanquish"), ("MIS11", "vanquishsub"), ("MIS13B", "ultralight"), ("RACE", "paradis_car")]
        .into_iter()
        .map(|(v, s)| {
            let m = car(v, s);
            (v, s, [n(m, st(PLAYER, false, false)), n(m, st(PLAYER, true, false)), n(m, st(PLAYER, false, true)), all(m).len()])
        })
        .collect();
    assert_eq!(
        rows,
        [
            ("MIS3", "supersnow", [23, 7, 15, 31]),
            ("MIS4", "vanquish", [47, 30, 55, 57]),
            ("MIS11", "vanquishsub", [30, 18, 30, 30]),
            ("MIS13B", "ultralight", [19, 17, 18, 19]),
            ("RACE", "paradis_car", [11, 7, 11, 11]),
        ]
    );
}

#[test]
fn the_helicopters_rotors_spin_and_its_camera_facing_part_turns_to_the_camera() {
    let Some(cars) = cars() else { return };
    let h = &cars.iter().find(|c| c.viv == "MIS01" && c.stem == "helicopter").unwrap().mesh;
    let kinds: Vec<(usize, u8, u16)> = h.parts.iter().filter_map(|p| if let Attach::Other(k, v) = p.attach { Some((p.index, k, v)) } else { None }).collect();
    assert_eq!(kinds, [(1, 0xfc, 4864), (2, 0xfc, 1792), (6, 0xee, 1024), (7, 0xf3, 0)]);
    let mut pose = CarPose::rest(h);
    pose.frame = 10.0;
    pose.car_pos = [100.0, 5.0, -40.0];
    pose.camera = [100.0 - 0.6155421, 9.0, -40.0 + 1.8011328 - 10.0];
    let m = |i: usize| match part_matrix(&h.parts[i], &pose) {
        PartMatrix::Drawn(m) => m,
        other => panic!("{i}: {other:?}"),
    };
    // The rotors: 4864/256 = 19° and 1792/256 = 7° per tick about their own +y, so after 10
    // ticks 190° and 70°; the hub stays at the item origin.
    for (i, deg) in [(1usize, 190f32), (2, 70.0)] {
        let p = &h.parts[i];
        let mm = m(i);
        assert!(transform(&mm, [0.0; 3]).iter().zip(p.origin).all(|(a, b)| (a - b).abs() < 1e-5));
        // Local x after the spin, in the item's own frame: R_y(deg) maps x to (cos, 0, −sin).
        let x = transform(&mm, [1.0, 0.0, 0.0]);
        let x = [x[0] - p.origin[0], x[1] - p.origin[1], x[2] - p.origin[2]];
        let r = deg.to_radians();
        let want = [0usize, 1, 2].map(|k| r.cos() * p.axes[0][k] - r.sin() * p.axes[2][k]);
        assert!(x.iter().zip(want).all(|(a, b)| (a - b).abs() < 1e-4), "item {i}: {x:?} vs {want:?}");
    }
    // Item 6 (0xee, v 1024): the camera straight behind it on −z (item world x/z = car + origin):
    // the facing rows are the identity, the rock at frame 10 is phase 1024/65535·10/60 → 0.0026
    // → triangle 0.0052 → (0.0052 − 0.5)·0.1 turn about z.
    let mm = m(6);
    assert_eq!(mm, facing_matrix(&h.parts[6], 1024, &pose));
    let z = transform(&mm, [0.0, 0.0, 1.0]);
    let o = h.parts[6].origin;
    assert!((z[2] - o[2] - 1.0).abs() < 1e-5 && (z[0] - o[0]).abs() < 1e-5, "{z:?}");
    let x = transform(&mm, [1.0, 0.0, 0.0]);
    let phase = 1024.0 * f32::from_bits(0x3780_0080) * 10.0 / 60.0;
    let turn = (phase * 2.0 - 0.5) * 0.1;
    assert!(((x[1] - o[1]).atan2(x[0] - o[0]) / std::f32::consts::TAU - turn).abs() < 1e-5);
    // Item 7 (0xf3): its own matrix.
    assert_eq!(m(7), nf_formats::carpose::own_matrix(&h.parts[7]));
}
