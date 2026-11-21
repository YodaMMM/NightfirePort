//! Disc test for `nf_formats::carmesh` (docs/formats/car-model.md §9): every vehicle model of the
//! 8 driving archives decodes through the draw chain, each part's geometry matches the half-height
//! its own `as` record stores, every texture it names is in the car's sheet or the shared one, and
//! the old `crp_mesh::TrackModule::extract_car_mesh` reading does not reproduce the placed car.
//!
//! Opt-in via NF_ISO (the full image; a slim image without the `/DRIVING/` extents skips).

use std::collections::{BTreeSet, HashMap, HashSet};

use nf_formats::carmesh::{car_mesh, CarMesh};
use nf_formats::crp::{tag, Crp};
use nf_formats::crp_mesh::TrackModule;
use nf_formats::geoprim::ModelIndex;
use nf_formats::iso9660::IsoImage;

/// (archive, car files, `in#0` items, items with LOD-0 geometry, strips, vertices) [D measured].
const COUNTS: &[(&str, usize, usize, usize, usize, usize)] = &[
    ("MIS01", 10, 201, 141, 6319, 40999),
    ("MIS11", 4, 58, 58, 2185, 13918),
    ("MIS13A", 5, 111, 109, 5169, 31412),
    ("MIS13B", 5, 120, 63, 3020, 17719),
    ("MIS13C", 4, 36, 27, 1488, 9769),
    ("MIS3", 6, 155, 100, 4715, 27574),
    ("MIS4", 13, 210, 195, 7858, 47738),
    ("RACE", 11, 139, 126, 5643, 35638),
];

struct Car {
    stem: String,
    crp: Vec<u8>,
    ssh: Option<Vec<u8>>,
}

fn read(iso: &IsoImage, lba: u32, off: u32, size: u32) -> Option<Vec<u8>> {
    let raw = iso.read_bytes(lba as u64 * 2048 + off as u64, size as u64).ok()?;
    if raw.iter().take(64).all(|&b| b == 0) {
        return None; // slim image: extent zeroed
    }
    Some(if nf_formats::refpack::is_refpack(&raw) { nf_formats::refpack::decompress(&raw).unwrap() } else { raw })
}

/// The archive's car models and the shared `data\render\common.ssh`; `None` on a slim image.
fn archive(iso: &IsoImage, viv: &str) -> Option<(Vec<Car>, Option<Vec<u8>>)> {
    let e = iso.find(&format!("/DRIVING/{viv}.VIV"))?;
    let g = nf_formats::disc::read_bigf(iso, e).ok()?;
    let mut cars = Vec::new();
    let mut common = None;
    for x in &g.entries {
        let n = x.name.to_ascii_lowercase();
        if n == r"data\render\common.ssh" {
            common = read(iso, e.lba, x.offset, x.size);
        }
        let Some(stem) = n.strip_prefix(r"data\car\model\").and_then(|s| s.strip_suffix(".crp")) else { continue };
        let crp = read(iso, e.lba, x.offset, x.size)?;
        let ssh_name = format!(r"data\car\model\{stem}.ssh");
        let ssh = g.entries.iter().find(|y| y.name.eq_ignore_ascii_case(&ssh_name)).and_then(|y| read(iso, e.lba, y.offset, y.size));
        cars.push(Car { stem: stem.to_owned(), crp, ssh });
    }
    Some((cars, common))
}

fn sheet_names(b: &Option<Vec<u8>>) -> HashSet<String> {
    b.as_ref()
        .and_then(|b| nf_formats::ssh::Ssh::parse(b).ok())
        .map(|s| s.textures.into_iter().map(|t| t.name.to_ascii_lowercase()).collect())
        .unwrap_or_default()
}

/// `as#N` → its stored `+0x14` f32 (independent of the decoder: read straight off the records).
fn set_half_heights(crp_bytes: &[u8]) -> HashMap<u16, f32> {
    let crp = Crp::parse(crp_bytes).unwrap();
    let arti = crp.pool().unwrap().into_iter().find(|r| r.tag == tag::ARTI).unwrap();
    let (s, e) = arti.window(crp.data).unwrap();
    let mut out = HashMap::new();
    for r in crp.window_records(s, e).unwrap() {
        if r.tag_code() == 0x6173_2020 {
            let p = crp.payload(&r).unwrap();
            out.insert(r.instance().unwrap(), f32::from_le_bytes(p[0x14..0x18].try_into().unwrap()));
        }
    }
    out
}

fn iso() -> Option<IsoImage> {
    let Ok(path) = std::env::var("NF_ISO") else {
        eprintln!("skip: NF_ISO not set");
        return None;
    };
    let iso = IsoImage::open(&path).expect("open NF_ISO");
    if archive(&iso, "MIS01").map_or(true, |(c, _)| c.is_empty()) {
        eprintln!("skip: no /DRIVING/ extents in this image");
        return None;
    }
    Some(iso)
}

#[test]
fn every_driving_vehicle_decodes_with_each_part_matching_its_stored_height_and_sheets() {
    let Some(iso) = iso() else { return };
    let mut got = Vec::new();
    let (mut sets_checked, mut height_err) = (0usize, 0.0f32);
    let (mut unit_normals, mut normals, mut uv_untextured, mut mirrored, mut skel, mut deferred) = (0usize, 0usize, 0usize, 0usize, 0usize, 0usize);
    for &(viv, ..) in COUNTS {
        let (cars, common) = archive(&iso, viv).unwrap();
        let common = sheet_names(&common);
        let e = iso.find(&format!("/DRIVING/{viv}.VIV")).unwrap();
        let g = nf_formats::disc::read_bigf(&iso, e).unwrap();
        let mut others: HashMap<String, HashSet<String>> = HashMap::new();
        for x in g.entries.iter().filter(|x| x.name.to_ascii_lowercase().ends_with(".ssh")) {
            others.insert(x.name.to_ascii_lowercase(), sheet_names(&read(&iso, e.lba, x.offset, x.size)));
        }
        let (mut items, mut placed, mut strips, mut verts) = (0, 0, 0, 0);
        for car in &cars {
            let m: CarMesh = car_mesh(&car.crp).unwrap_or_else(|e| panic!("{viv} {}: {e}", car.stem));
            assert_eq!(m.sheet.as_deref().map(str::to_ascii_lowercase), Some(format!("{}.ssh", car.stem)), "{viv} {}: ns#0 sheet", car.stem);
            let own = sheet_names(&car.ssh);
            let heights = set_half_heights(&car.crp);
            let mut seen = HashSet::new();
            let mut missing = BTreeSet::new();
            items += m.parts.len();
            for p in &m.parts {
                assert!(p.lods.len() <= 1 || p.lods.windows(2).all(|w| w[0].0 <= w[1].0), "{viv} {}: LOD distances", car.stem);
                if p.strips.is_empty() {
                    continue;
                }
                placed += 1;
                mirrored += p.mirrored() as usize;
                skel += matches!(p.attach, nf_formats::carmesh::Attach::Skel(_)) as usize;
                deferred += (p.pass != 2) as usize;
                for s in &p.strips {
                    for n in &s.normals {
                        let l = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
                        normals += 1;
                        unit_normals += (0.95..=1.05).contains(&l) as usize;
                    }
                    uv_untextured += (!s.uvs.is_empty() && s.texture.is_none()) as usize;
                    assert!(s.positions.iter().flatten().all(|c| c.is_finite()), "{viv} {}: non-finite vertex", car.stem);
                    assert!(s.uvs.is_empty() || s.uvs.len() == s.positions.len());
                    assert!(s.normals.is_empty() || s.normals.len() == s.positions.len());
                    if let Some(t) = &s.texture {
                        let t = t.to_ascii_lowercase();
                        if !own.contains(&t) && !common.contains(&t) {
                            missing.insert(t);
                        }
                    }
                }
                if seen.insert(p.set) {
                    strips += p.strips.len();
                    verts += p.strips.iter().map(|s| s.positions.len()).sum::<usize>();
                    // The part's box: its `as` record's +0x14 is the y half-extent of exactly this
                    // geometry [D, 686 of 686 sets].
                    let (lo, hi) = p.strips.iter().flat_map(|s| &s.positions).fold((f32::MAX, f32::MIN), |(a, b), q| (a.min(q[1]), b.max(q[1])));
                    let d = ((hi - lo) / 2.0 - heights[&p.set]).abs();
                    height_err = height_err.max(d);
                    sets_checked += 1;
                    assert!(d < 0.002, "{viv} {} as#{:x}: y half {} stored {}", car.stem, p.set, (hi - lo) / 2.0, heights[&p.set]);
                }
            }
            // The one exception: the ultralights' `blck` descriptors (`{T0@85244673}` and
            // `{Tb@158bb0de}TEX0::blck`) name an entry no sheet of their archives holds [D]; other
            // archives' sheets have one (road_block_truck, supersnow, …).
            for t in &missing {
                let elsewhere: Vec<&String> = others.iter().filter(|(_, n)| n.contains(t)).map(|(k, _)| k).collect();
                eprintln!("  {viv} {}: {t} not in its sheet or common.ssh (other sheets here: {elsewhere:?})", car.stem);
                assert!(t == "blck" && car.stem.starts_with("ultralight") || t == "blck" && car.stem == "staticultralight", "{viv} {}: texture {t} in no sheet", car.stem);
                assert!(elsewhere.is_empty());
            }
            for (k, list) in &m.draw_lists {
                assert!(list.iter().all(|&i| (i as usize) < m.parts.len()), "{viv} {} vd#{k}: item index past in#0", car.stem);
            }
        }
        eprintln!("{viv}: {} cars, items {items}, placed {placed}, strips {strips}, vertices {verts}", cars.len());
        got.push((viv, cars.len(), items, placed, strips, verts));
    }
    eprintln!("sets checked {sets_checked}, worst y half-extent error {height_err}");
    eprintln!("normals {unit_normals} of {normals} unit length ±5%; strips with UVs and no texture set yet {uv_untextured}; placed items mirrored {mirrored}, on a Skel part {skel}, in a later pass {deferred}");
    // The V3-8 / second V4-32 array is a normal, not a colour [D].
    assert!(unit_normals * 1000 >= normals * 995, "{unit_normals} of {normals} normals unit length");
    let want: Vec<_> = COUNTS.to_vec();
    assert_eq!(got, want);
}

/// Every model's group walk ([M FUN_002843a0]) yields the same entries as the track's walk over
/// `+0xc8` groups: one group per model on every car, all groups visible.
#[test]
fn every_car_model_is_one_visible_group() {
    let Some(iso) = iso() else { return };
    let mut models = 0;
    for &(viv, ..) in COUNTS {
        for car in archive(&iso, viv).unwrap().0 {
            let crp = Crp::parse(&car.crp).unwrap();
            let module = TrackModule::load(&crp).unwrap();
            let ix = ModelIndex::new(&module).unwrap();
            for s in module.symbols.iter().filter(|s| s.name.starts_with("__Model:::")) {
                assert_eq!(nf_formats::carmesh::model_entries(&ix, s.value).unwrap(), ix.model_packets(s.value).unwrap(), "{viv} {}: {}", car.stem, s.name);
                models += 1;
            }
        }
    }
    eprintln!("{models} models");
}

/// The old reading (`extract_car_mesh`: `vd#1` taken as `as` indices, `as#k` placed with item k's
/// matrix, the batch behind each entry) against the new one, on the vanquish: the share of the old
/// vertices that coincide (within 1 mm) with a vertex of the car the new reading places. With a
/// right old reading this would be all of them.
#[test]
fn the_old_car_reading_does_not_reproduce_the_placed_vanquish() {
    let Some(iso) = iso() else { return };
    let car = archive(&iso, "MIS01").unwrap().0.into_iter().find(|c| c.stem == "vanquish").unwrap();
    let m = car_mesh(&car.crp).unwrap();
    let key = |p: [f32; 3]| [(p[0] * 1000.0).round() as i32, (p[1] * 1000.0).round() as i32, (p[2] * 1000.0).round() as i32];
    let mut new: HashSet<[i32; 3]> = HashSet::new();
    for p in m.drawn(None) {
        for s in &p.strips {
            for &q in &s.positions {
                let w = p.apply(q);
                // 1 mm cells, neighbours included
                for dx in -1..=1 {
                    for dy in -1..=1 {
                        for dz in -1..=1 {
                            let k = key(w);
                            new.insert([k[0] + dx, k[1] + dy, k[2] + dz]);
                        }
                    }
                }
            }
        }
    }
    let crp = Crp::parse(&car.crp).unwrap();
    let old = TrackModule::load(&crp).unwrap().extract_car_mesh(&crp, "vanquish");
    let total: usize = old.iter().map(|s| s.positions.len()).sum();
    let hit: usize = old.iter().flat_map(|s| &s.positions).filter(|&&q| new.contains(&key(q))).count();
    eprintln!("old reading: {hit} of {total} vertices on the new placed car");
    assert!(total > 0);
    assert!(hit * 10 < total * 9, "the old reading reproduces the car ({hit}/{total})");
}
