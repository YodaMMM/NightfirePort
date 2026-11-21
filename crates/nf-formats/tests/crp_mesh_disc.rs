//! Disc proof for the driving `.crp` mesh and collision layer (`nf_formats::crp_mesh`): every
//! track and car `.crp` of the eight mission archives and `MISC.VIV` decodes with no error, and
//! the collision world sits under the track's road network.
//!
//! Reads the full retail image named by `NF_ISO` (the `DRIVING/*.VIV` archives); skips cleanly
//! when the variable is unset or the image has no `DRIVING/` folder.

use nf_formats::crp::{Crp, Record};
use nf_formats::crp_mesh::{CollisionTri, TrackModule};
use nf_formats::iso9660::IsoImage;
use nf_formats::{disc, refpack};

const ARCHIVES: [&str; 9] = ["MIS01", "MIS11", "MIS13A", "MIS13B", "MIS13C", "MIS3", "MIS4", "RACE", "MISC"];

struct CrpFile {
    archive: &'static str,
    name: String,
    bytes: Vec<u8>,
}

fn disc_crps() -> Option<Vec<CrpFile>> {
    let Some(path) = std::env::var_os("NF_ISO") else {
        eprintln!("NF_ISO unset: skipping");
        return None;
    };
    let iso = IsoImage::open(&path).expect("open NF_ISO");
    let mut out = Vec::new();
    for archive in ARCHIVES {
        let Some(e) = iso.find(&format!("/DRIVING/{archive}.VIV")) else {
            eprintln!("no /DRIVING/{archive}.VIV on this image: skipping");
            return None;
        };
        let g = disc::read_bigf(&iso, e).expect("BIGF");
        for ent in &g.entries {
            let lower = ent.name.to_ascii_lowercase();
            let wanted = lower.ends_with(".crp")
                && (lower.contains("data\\track\\") || lower.contains("data\\car\\model\\"));
            if !wanted {
                continue;
            }
            let raw = iso
                .read_bytes(e.lba as u64 * 2048 + ent.offset as u64, ent.size as u64)
                .expect("read entry");
            let bytes = if refpack::is_refpack(&raw) {
                refpack::decompress(&raw).unwrap_or_else(|err| panic!("{archive} {}: {err}", ent.name))
            } else {
                raw
            };
            out.push(CrpFile { archive, name: ent.name.clone(), bytes });
        }
    }
    Some(out)
}

/// The road-network node positions (`RNgp` → `rn#i` payload +0x00 3×f32) [M 0x220c28, 0x225c48].
fn road_nodes(crp: &Crp) -> Vec<[f32; 3]> {
    let mut out = Vec::new();
    for r in crp.pool().unwrap() {
        if r.tag != 0x524e_6770 {
            continue;
        }
        let (s, e) = r.window(crp.data).unwrap();
        let mut recs: Vec<(u16, [f32; 3])> = Vec::new();
        for o in (s..e).step_by(16) {
            let q = Record::parse(crp.data, o).unwrap();
            if q.tag_code() == 0x726e_2020 {
                let p = crp.payload(&q).unwrap();
                let f = |i: usize| f32::from_le_bytes(p[i..i + 4].try_into().unwrap());
                recs.push((q.instance().unwrap(), [f(0), f(4), f(8)]));
            }
        }
        recs.sort_by_key(|x| x.0);
        out.extend(recs.into_iter().map(|x| x.1));
    }
    out
}

/// A downward ground probe at (x, z): the highest triangle surface in [y − below, y + 3], skipping
/// the triangles the game's ground query skips (class bit 0x04) [M 0x2138b0]. The xz containment
/// test is winding-free; heights come from the triangle's plane.
fn ground_y_within(tris: &[CollisionTri], x: f32, y: f32, z: f32, below: f32) -> Option<f32> {
    let mut best: Option<f32> = None;
    for t in tris {
        if t.ground_query_skips() {
            continue;
        }
        let [a, b, c] = t.v;
        let (minx, maxx) = (a[0].min(b[0]).min(c[0]), a[0].max(b[0]).max(c[0]));
        let (minz, maxz) = (a[2].min(b[2]).min(c[2]), a[2].max(b[2]).max(c[2]));
        if x < minx || x > maxx || z < minz || z > maxz {
            continue;
        }
        let den = (b[2] - c[2]) * (a[0] - c[0]) + (c[0] - b[0]) * (a[2] - c[2]);
        if den.abs() < 1e-6 {
            continue;
        }
        let u = ((b[2] - c[2]) * (x - c[0]) + (c[0] - b[0]) * (z - c[2])) / den;
        let v = ((c[2] - a[2]) * (x - c[0]) + (a[0] - c[0]) * (z - c[2])) / den;
        let w = 1.0 - u - v;
        if u < -1e-4 || v < -1e-4 || w < -1e-4 {
            continue;
        }
        let h = u * a[1] + v * b[1] + w * c[1];
        if h >= y - below && h <= y + 3.0 {
            best = Some(best.map_or(h, |p: f32| p.max(h)));
        }
    }
    best
}

/// The probe points of a track: its road-network nodes, or — for the one track without a road
/// network (MIS13C) — its `Map` `Trgr` trigger centres [D].
fn probe_points(crp: &Crp) -> (Vec<[f32; 3]>, &'static str) {
    let nodes = road_nodes(crp);
    if !nodes.is_empty() {
        return (nodes, "road nodes");
    }
    (crp.extract_map_triggers().unwrap().into_iter().map(|t| t.pos).collect(), "triggers")
}

#[test]
fn every_track_crp_yields_collision_under_its_road_network() {
    let Some(files) = disc_crps() else { return };
    let mut tracks = 0;
    for f in files.iter().filter(|f| f.name.to_ascii_lowercase().contains("data\\track\\")) {
        tracks += 1;
        let crp = Crp::parse(&f.bytes).unwrap();
        let tris = crp.extract_collision_tris().unwrap_or_else(|e| panic!("{} {}: {e}", f.archive, f.name));
        let (points, kind) = probe_points(&crp);
        // The underwater track's nodes are swim lanes above the sea floor: any surface below
        // counts there; every other track's nodes sit on the road (within 12 units).
        let below = if f.archive == "MIS11" { 1e6 } else { 12.0 };
        let hits: Vec<Option<f32>> =
            points.iter().map(|n| ground_y_within(&tris, n[0], n[1], n[2], below)).collect();
        let hit = hits.iter().filter(|h| h.is_some()).count();
        let gaps: Vec<f32> = points.iter().zip(&hits).filter_map(|(n, h)| h.map(|h| n[1] - h)).collect();
        let mean_gap = gaps.iter().sum::<f32>() / gaps.len().max(1) as f32;
        eprintln!(
            "{}\t{}\ttris {}\t{kind} {}\tground hits {hit}\tmean gap {mean_gap:.2}\tfirst {:?}",
            f.archive, f.name, tris.len(), points.len(), &hits[..3.min(hits.len())]
        );
        // Every strip triangle of every placed article (count − 2 per strip): a decoder that
        // drops triangles (an earlier reading kept only words with bit 0 set) fails here.
        let want = match f.archive {
            "MIS01" => 42_826,
            "MIS11" => 128_493,
            "MIS13A" => 40_160,
            "MIS13B" => 59_210,
            "MIS13C" => 6_667,
            "MIS3" => 74_861,
            _ => 33_002, // MIS4 and RACE share the snow2a map
        };
        assert_eq!(tris.len(), want, "{}: collision triangle count", f.name);
        assert!(points.len() >= 3, "{}: fewer than 3 probe points", f.name);
        for (i, h) in hits.iter().take(3).enumerate() {
            assert!(h.is_some(), "{}: {kind} {i} {:?} has no ground under it", f.name, points[i]);
        }
        assert!(hit * 100 >= points.len() * 95, "{}: only {hit}/{} {kind} over ground", f.name, points.len());
    }
    assert_eq!(tracks, 8, "eight mission tracks");
}

/// Per file: articles whose decoded mesh has vertices, and how many of those sit inside the
/// article's `Base` box (1/256 slack) and touch it on all six faces.
#[test]
fn every_track_and_car_crp_yields_meshes() {
    let Some(files) = disc_crps() else { return };
    let mut cars = 0;
    let (mut car_uv_total, mut car_uv_out) = (0usize, 0usize);
    for f in &files {
        let crp = Crp::parse(&f.bytes).unwrap();
        let module = TrackModule::load(&crp).unwrap_or_else(|e| panic!("{} {}: {e}", f.archive, f.name));
        let arts = crp.track_articles().unwrap_or_else(|e| panic!("{} {}: {e}", f.archive, f.name));
        let is_track = f.name.to_ascii_lowercase().contains("data\\track\\");
        let (mut strips, mut verts, mut tris, mut meshed, mut inside, mut exact, mut textured) =
            (0usize, 0usize, 0usize, 0usize, 0usize, 0usize, 0usize);
        for a in &arts {
            let st = if is_track {
                module.extract_article_mesh(&a.name, a.bbox_min, a.bbox_max)
            } else {
                module.extract_all_article_parts(&a.name, a.bbox_min, a.bbox_max)
            };
            strips += st.len();
            textured += st.iter().filter(|s| s.texture.is_some()).count();
            verts += st.iter().map(|s| s.positions.len()).sum::<usize>();
            tris += st.iter().map(|s| s.positions.len().saturating_sub(2)).sum::<usize>();
            let mut lo = [f32::MAX; 3];
            let mut hi = [f32::MIN; 3];
            for p in st.iter().flat_map(|s| s.positions.iter()) {
                assert!(p.iter().all(|c| c.is_finite()), "{} {}: non-finite vertex", f.name, a.name);
                for i in 0..3 {
                    lo[i] = lo[i].min(p[i]);
                    hi[i] = hi[i].max(p[i]);
                }
            }
            if lo[0] > hi[0] {
                continue;
            }
            meshed += 1;
            let tol = 1.0 / 256.0 + 1e-3;
            if (0..3).all(|i| lo[i] >= a.bbox_min[i] - tol && hi[i] <= a.bbox_max[i] + tol) {
                inside += 1;
            }
            if (0..3).all(|i| (lo[i] - a.bbox_min[i]).abs() <= tol && (hi[i] - a.bbox_max[i]).abs() <= tol) {
                exact += 1;
            }
        }
        if !is_track {
            cars += 1;
        }
        eprintln!(
            "{}\t{}\tarticles {}\tmeshed {meshed}\tinside Base {inside}\texact Base {exact}\tstrips {strips} (textured {textured})\tverts {verts}\tstrip tris {tris}",
            f.archive, f.name, arts.len()
        );
        assert!(strips > 0 && verts > 0, "{} {}: no mesh", f.archive, f.name);
        if is_track {
            // The decoded geometry of a track article lies inside its own Base box, and for most
            // articles spans it exactly: wrong packets (the one at entry + 8) or a box-mapped
            // dequantisation fail one of the two.
            assert!(meshed * 100 >= arts.len() * 80, "{}: only {meshed}/{} articles decode", f.name, arts.len());
            assert!(inside * 100 >= meshed * 93, "{}: only {inside}/{meshed} articles inside their Base box", f.name);
            assert!(exact * 100 >= meshed * 55, "{}: only {exact}/{meshed} articles span their Base box", f.name);
        } else {
            // A vehicle's texture coordinates address one sheet: they stay inside [0, 1] (the
            // V2-16 scale 1/1024 [D]; 1/256 puts most of them out to 4).
            let uvs: Vec<[f32; 2]> = arts
                .iter()
                .flat_map(|a| module.extract_all_article_parts(&a.name, a.bbox_min, a.bbox_max))
                .flat_map(|s| s.uvs)
                .collect();
            let out = uvs.iter().filter(|uv| uv.iter().any(|&c| !(-0.01..=1.01).contains(&c))).count();
            car_uv_total += uvs.len();
            car_uv_out += out;
        }
    }
    eprintln!("vehicle UVs outside [0, 1]: {car_uv_out}/{car_uv_total}");
    assert!(cars > 0);
    assert!(car_uv_out * 100 <= car_uv_total * 5, "vehicle UVs outside [0, 1]: {car_uv_out}/{car_uv_total}");
}
