//! Disc test for `nf_formats::geoprim` (docs/formats/geoprim.md): every driving track's placed
//! world geometry decodes, its counts are pinned, the road surface lies under the car start and
//! under the road network, it agrees with nothing it could not (no copied expectations: the road
//! nodes and the collision world are independent data in the same file).
//!
//! Opt-in via NF_ISO (the full image; a slim image without the `/DRIVING/` extents skips).

use std::collections::HashMap;

use nf_formats::crp::{Crp, Record};
use nf_formats::geoprim::{track_geometry, TrackGeometry, WorldStrip};
use nf_formats::iso9660::IsoImage;

/// (archive, placements naming a part, unresolved parts, strips, triangles, road nodes, road
/// nodes with a decoded surface within 1 m) [D measured]. The node coverage is pinned exactly: a
/// wrong vertex rule or placement direction moves it (read as the inverse matrix, paris's placed
/// geometry had a surface within 6 m of only 82 of the 880 nodes — docs/formats/geoprim.md §4;
/// decoding each entry object's own batch instead of its draw chain gave 772/307/272/198 for
/// MIS01/MIS3/MIS4/RACE — §4b).
const COUNTS: &[(&str, usize, usize, usize, usize, usize, usize)] = &[
    ("MIS01", 2552, 86, 60327, 257412, 880, 806),
    ("MIS11", 1958, 107, 40275, 281548, 136, 0),
    ("MIS13A", 3021, 76, 55963, 325807, 88, 88),
    ("MIS13B", 1412, 24, 22347, 121974, 56, 50),
    ("MIS13C", 371, 21, 5657, 32086, 0, 0),
    ("MIS3", 3689, 65, 61842, 283733, 348, 336),
    ("MIS4", 3331, 45, 43977, 184494, 440, 419),
    ("RACE", 3314, 45, 43421, 180393, 345, 332),
];

/// `RNgp` — the road-network article; `rn  #i` node records, payload +0x00 = x,y,z
/// [M 0x220c28 / 0x225c48:179-186, crates/nf-game/src/driving/road.rs].
const TAG_RNGP: u32 = 0x524e_6770;
const TAG_RN: u32 = 0x726e_2020;

fn road_nodes(crp: &Crp<'_>) -> Vec<[f32; 3]> {
    let Some(rn) = crp.pool().unwrap().into_iter().find(|r| r.tag == TAG_RNGP) else { return Vec::new() };
    let (s, e) = rn.window(crp.data).unwrap();
    let mut out: Vec<(u16, [f32; 3])> = Vec::new();
    for o in (s..e).step_by(16) {
        let r = Record::parse(crp.data, o).unwrap();
        if r.tag_code() == TAG_RN {
            let p = crp.payload(&r).unwrap();
            let f = |k: usize| f32::from_le_bytes(p[k * 4..k * 4 + 4].try_into().unwrap());
            out.push((r.instance().unwrap(), [f(0), f(1), f(2)]));
        }
    }
    out.sort_by_key(|x| x.0);
    out.into_iter().map(|x| x.1).collect()
}

/// A 16 m xz grid of the world triangles for point-under queries.
struct Grid(HashMap<(i32, i32), Vec<[[f32; 3]; 3]>>);

impl Grid {
    fn new(strips: &[WorldStrip]) -> Grid {
        let mut g: HashMap<(i32, i32), Vec<[[f32; 3]; 3]>> = HashMap::new();
        for s in strips {
            for t in s.triangles() {
                let (x0, x1) = (t.iter().map(|p| p[0]).fold(f32::MAX, f32::min), t.iter().map(|p| p[0]).fold(f32::MIN, f32::max));
                let (z0, z1) = (t.iter().map(|p| p[2]).fold(f32::MAX, f32::min), t.iter().map(|p| p[2]).fold(f32::MIN, f32::max));
                if x1 - x0 > 400.0 || z1 - z0 > 400.0 {
                    continue; // sky domes / horizon cards
                }
                for gx in (x0 / 16.0).floor() as i32..=(x1 / 16.0).floor() as i32 {
                    for gz in (z0 / 16.0).floor() as i32..=(z1 / 16.0).floor() as i32 {
                        g.entry((gx, gz)).or_default().push(t);
                    }
                }
            }
        }
        Grid(g)
    }
    /// Heights of every triangle above/below (x, z).
    fn heights(&self, x: f32, z: f32) -> Vec<f32> {
        let mut out = Vec::new();
        let Some(ts) = self.0.get(&((x / 16.0).floor() as i32, (z / 16.0).floor() as i32)) else { return out };
        for [a, b, c] in ts {
            let d = (b[2] - c[2]) * (a[0] - c[0]) + (c[0] - b[0]) * (a[2] - c[2]);
            if d.abs() < 1e-9 {
                continue;
            }
            let l1 = ((b[2] - c[2]) * (x - c[0]) + (c[0] - b[0]) * (z - c[2])) / d;
            let l2 = ((c[2] - a[2]) * (x - c[0]) + (a[0] - c[0]) * (z - c[2])) / d;
            let l3 = 1.0 - l1 - l2;
            if l1 >= -1e-4 && l2 >= -1e-4 && l3 >= -1e-4 {
                out.push(l1 * a[1] + l2 * b[1] + l3 * c[1]);
            }
        }
        out
    }
    /// The nearest surface height to y under (x, z) or under a point 0.5 m away (road nodes sit on
    /// tile seams, where neighbouring tiles leave cracks of a few decimetres), if any lies within
    /// `tol` metres.
    fn surface_near(&self, p: [f32; 3], tol: f32) -> Option<f32> {
        [(0.0, 0.0), (0.5, 0.0), (-0.5, 0.0), (0.0, 0.5), (0.0, -0.5)]
            .iter()
            .flat_map(|(dx, dz)| self.heights(p[0] + dx, p[2] + dz))
            .filter(|h| (h - p[1]).abs() <= tol)
            .min_by(|a, b| (a - p[1]).abs().total_cmp(&(b - p[1]).abs()))
    }
}

fn track_crp(iso: &IsoImage, viv: &str) -> Option<Vec<u8>> {
    let e = iso.find(&format!("/DRIVING/{viv}.VIV"))?;
    let g = nf_formats::disc::read_bigf(iso, e).ok()?;
    let t = g.entries.iter().find(|x| {
        let n = x.name.to_ascii_lowercase();
        n.starts_with("data\\track\\") && n.ends_with(".crp")
    })?;
    let raw = iso.read_bytes(e.lba as u64 * 2048 + t.offset as u64, t.size as u64).ok()?;
    if raw.iter().take(64).all(|&b| b == 0) {
        return None; // slim image: extent zeroed
    }
    Some(if nf_formats::refpack::is_refpack(&raw) { nf_formats::refpack::decompress(&raw).unwrap() } else { raw })
}

fn tri_count(g: &TrackGeometry) -> usize { g.strips.iter().map(|s| s.positions.len().saturating_sub(2)).sum() }

#[test]
fn every_driving_track_decodes_with_the_road_under_the_network() {
    let Ok(path) = std::env::var("NF_ISO") else { eprintln!("skip: NF_ISO not set"); return };
    let iso = IsoImage::open(&path).expect("open NF_ISO");
    if track_crp(&iso, "MIS01").is_none() {
        eprintln!("skip: no /DRIVING/ extents in this image");
        return;
    }
    let mut got = Vec::new();
    for &(viv, ..) in COUNTS {
        let dec = track_crp(&iso, viv).unwrap_or_else(|| panic!("{viv}: no track .crp"));
        let g = track_geometry(&dec).unwrap_or_else(|e| panic!("{viv}: {e}"));
        let crp = Crp::parse(&dec).unwrap();
        let nodes = road_nodes(&crp);
        let grid = Grid::new(&g.strips);
        let near = nodes.iter().filter(|&&n| grid.surface_near(n, 1.0).is_some()).count();
        // The car start is the first road node (the rule the PR #5 driving view uses; the game's own
        // start rule is not traced). A node can float a few metres over the surface (MIS13B's
        // first node is 3.6 m above it), so the start check takes the nearest surface within 4 m.
        let start = nodes.first().map(|&n| (n, grid.surface_near(n, 4.0)));
        eprintln!(
            "{viv}: placements {} unresolved {} strips {} tris {} | road nodes {} with a surface within 1 m {near} | start {:?}",
            g.placements, g.unresolved_parts, g.strips.len(), tri_count(&g), nodes.len(), start
        );
        got.push((viv, g.placements, g.unresolved_parts, g.strips.len(), tri_count(&g), nodes.len(), near));
        assert_eq!(g.missing_models, 0, "{viv}: model references without a module symbol");
        assert!(g.strips.iter().all(|s| s.positions.iter().all(|p| p.iter().all(|c| c.is_finite()))), "{viv}: non-finite vertex");
        // The underwater track (MIS11: the network is the submarine's path through open water) and
        // MIS13C (no road nodes) have no road under the start.
        if viv != "MIS11" && !nodes.is_empty() {
            let (p, s) = start.unwrap();
            assert!(s.is_some(), "{viv}: no surface within 4 m of the car start {p:?}");
        }
    }
    assert_eq!(got, COUNTS.to_vec(), "(track, placements, unresolved, strips, triangles, road nodes, nodes over a surface)");
}

#[test]
fn the_road_surface_matches_the_collision_world_at_paris() {
    // The CDat collision items (`ci#0`, world→local matrices) placing each article's `ca#0` strips
    // are a second, independent copy of the drivable surface. Under the paris road nodes the two
    // must agree to a few centimetres where both exist.
    let Ok(path) = std::env::var("NF_ISO") else { eprintln!("skip: NF_ISO not set"); return };
    let iso = IsoImage::open(&path).expect("open NF_ISO");
    let Some(dec) = track_crp(&iso, "MIS01") else { eprintln!("skip: no /DRIVING/ extents"); return };
    let g = track_geometry(&dec).unwrap();
    let crp = Crp::parse(&dec).unwrap();
    let render = Grid::new(&g.strips);
    let coll = Grid::new(&collision_strips(&crp));
    let (mut both, mut agree) = (0, 0);
    for n in road_nodes(&crp) {
        if let (Some(a), Some(b)) = (render.surface_near(n, 1.0), coll.surface_near(n, 1.0)) {
            both += 1;
            if (a - b).abs() < 0.05 {
                agree += 1;
            }
        }
    }
    eprintln!("paris: render and collision both under {both} nodes, agree within 5 cm at {agree}");
    // Drawn from the entry objects' own batches (the pre-chain reading) this was 730 of 739.
    assert!(both >= 750 && agree + 5 >= both, "render/collision agreement {agree}/{both}");
}

/// The collision world as strips: `CDat ci#0` 0x40-byte items (rotation rows at +0/+0x20, the
/// translation at +0x30, `sr` ref at +0x1c) are world→local, so world = Rᵀ(local − t) [D: with this
/// reading the collision surface coincides with the render surface; read forward it does not];
/// each article's `ca#0` holds `count` groups of strips of 16-byte vertices (xyz + a word whose
/// bit 0 marks a real triangle) [D, the layout the PR #5 decoder used].
fn collision_strips(crp: &Crp<'_>) -> Vec<WorldStrip> {
    use nf_formats::crp::tag;
    let tag_sr = 0x7372_2020;
    let tag_ci = 0x6369_2020;
    let tag_ca = 0x6361_2020;
    let cstr = |p: &[u8]| String::from_utf8_lossy(&p[..p.iter().position(|&b| b == 0).unwrap_or(p.len())]).into_owned();
    let mut ca: HashMap<String, Vec<u8>> = HashMap::new();
    let mut sr: HashMap<u16, String> = HashMap::new();
    let mut ci: Vec<u8> = Vec::new();
    for r in crp.pool().unwrap() {
        let (s, e) = r.window(crp.data).unwrap();
        let recs = crp.window_records(s, e).unwrap();
        if r.tag == tag::ARTI {
            let name = recs.iter().find(|x| x.tag == tag::NAME).map(|x| cstr(crp.payload(x).unwrap()));
            if let (Some(n), Some(c)) = (name, recs.iter().find(|x| x.tag_code() == tag_ca)) {
                ca.insert(n, crp.payload(c).unwrap().to_vec());
            }
        } else if r.tag == tag::CDAT {
            for x in &recs {
                if x.tag_code() == tag_sr {
                    sr.insert(x.instance().unwrap(), cstr(crp.payload(x).unwrap()).trim_start_matches("CARP::").to_owned());
                } else if x.tag_code() == tag_ci {
                    ci = crp.payload(x).unwrap().to_vec();
                }
            }
        }
    }
    let f = |d: &[u8], o: usize| f32::from_le_bytes(d[o..o + 4].try_into().unwrap());
    let u16a = |d: &[u8], o: usize| u16::from_le_bytes(d[o..o + 2].try_into().unwrap()) as usize;
    let mut out = Vec::new();
    for it in ci.chunks_exact(64) {
        let Some(name) = sr.get(&(u32::from_le_bytes(it[28..32].try_into().unwrap()) as u16)) else { continue };
        let Some(c) = ca.get(name) else { continue };
        if c.len() < 32 {
            continue;
        }
        let r0 = [f(it, 0), f(it, 4), f(it, 8)];
        let r2 = [f(it, 32), f(it, 36), f(it, 40)];
        let t = [f(it, 48), f(it, 52), f(it, 56)];
        let rows = [r0, [0.0, 1.0, 0.0], r2];
        let world = |l: [f32; 3]| -> [f32; 3] {
            let q = [l[0] - t[0], l[1] - t[1], l[2] - t[2]];
            [0, 1, 2].map(|j| (0..3).map(|i| rows[j][i] * q[i]).sum())
        };
        let (groups, size) = (u16a(c, 0), u16a(c, 2));
        let gend = c.len().min(32 + size);
        if 32 + groups * 16 > c.len() {
            continue;
        }
        for gi in 0..groups {
            let next = if gi + 1 < groups { 32 + u16a(c, 32 + (gi + 1) * 16 + 14) } else { gend };
            let end = gend.min(next);
            let mut cur = 32 + u16a(c, 32 + gi * 16 + 14);
            while cur + 16 <= end {
                let n = u32::from_le_bytes(c[cur + 12..cur + 16].try_into().unwrap()) as usize;
                if n < 3 || cur + n * 16 > end {
                    break;
                }
                // Keep only flagged triangles: split the strip at unflagged ones.
                let v: Vec<[f32; 3]> = (0..n).map(|k| world([f(c, cur + k * 16), f(c, cur + k * 16 + 4), f(c, cur + k * 16 + 8)])).collect();
                for k in 2..n {
                    if u32::from_le_bytes(c[cur + k * 16 + 12..cur + k * 16 + 16].try_into().unwrap()) & 1 != 0 {
                        out.push(WorldStrip { article: name.clone(), part: 0, placement: 0, texture: None, positions: v[k - 2..=k].to_vec(), uvs: Vec::new(), colors: Vec::new() });
                    }
                }
                cur += n * 16;
            }
        }
    }
    out
}

#[test]
fn every_texture_a_track_names_is_in_its_sheets() {
    let Ok(path) = std::env::var("NF_ISO") else { eprintln!("skip: NF_ISO not set"); return };
    let iso = IsoImage::open(&path).expect("open NF_ISO");
    if track_crp(&iso, "MIS01").is_none() {
        eprintln!("skip: no /DRIVING/ extents in this image");
        return;
    }
    for &(viv, ..) in COUNTS {
        let e = iso.find(&format!("/DRIVING/{viv}.VIV")).unwrap();
        let bigf = nf_formats::disc::read_bigf(&iso, e).unwrap();
        let mut names = std::collections::HashSet::new();
        for x in bigf.entries.iter().filter(|x| x.name.to_ascii_lowercase().ends_with(".ssh")) {
            let n = x.name.to_ascii_lowercase();
            if !(n.starts_with("data\\track\\") || n == "data\\render\\common.ssh") {
                continue;
            }
            let raw = iso.read_bytes(e.lba as u64 * 2048 + x.offset as u64, x.size as u64).unwrap();
            let dec = if nf_formats::refpack::is_refpack(&raw) { nf_formats::refpack::decompress(&raw).unwrap() } else { raw };
            if let Ok(s) = nf_formats::ssh::Ssh::parse(&dec) {
                names.extend(s.textures.into_iter().map(|t| t.name.to_ascii_lowercase()));
            }
        }
        let g = track_geometry(&track_crp(&iso, viv).unwrap()).unwrap();
        let used: std::collections::BTreeSet<String> = g.strips.iter().filter_map(|s| s.texture.as_ref()).map(|t| t.to_ascii_lowercase()).collect();
        let missing: Vec<&String> = used.iter().filter(|t| !names.contains(*t)).collect();
        let untextured = g.strips.iter().filter(|s| s.texture.is_none()).count();
        eprintln!("{viv}: {} textures used, {} not in the track/common sheets {:?}; {untextured} untextured strips", used.len(), missing.len(), &missing[..missing.len().min(8)]);
        // Only the exporter's `badshapereference` placeholder (descriptor name `bads`) is absent.
        assert!(missing.iter().all(|t| t.as_str() == "bads"), "{viv}: textures missing from the sheets {missing:?}");
    }
}

/// The draw-chain reading against the entry objects' own batches, on every track, with two
/// independent checks (docs/formats/geoprim.md §4b):
/// 1. a model's `Colours` list (model `+0xb8` → `{0, name, n, 0}` + n × `{u16 vertex count, u16,
///    stride, base, flag}`) names one vertex count per group entry [D]; compare it with the counts of
///    the chain batches and of the objects' own batches;
/// 2. the model's vertices against its article's `Base` box (+ 1/128 slack), for 16-bit-position
///    batches.
/// Pinned per track: (models with a Colours list, lists matching the chain, lists matching the own
/// batches, models whose chain lies in the box, models whose own batches do, coloured strips).
const CHAIN: &[(&str, usize, usize, usize, usize, usize, usize)] = &[
    ("MIS01", 1357, 1357, 81, 1871, 394, 59950),
    ("MIS11", 314, 304, 0, 716, 266, 40738),
    ("MIS13A", 743, 743, 24, 1232, 424, 27314),
    ("MIS13B", 851, 851, 86, 1834, 472, 28578),
    ("MIS13C", 237, 237, 36, 616, 244, 12124),
    ("MIS3", 762, 744, 8, 1750, 754, 26622),
    ("MIS4", 1142, 1136, 94, 1792, 452, 21418),
    ("RACE", 1142, 1136, 106, 1789, 448, 21539),
];

#[test]
fn the_draw_chain_not_the_entry_object_holds_each_models_geometry_and_colours() {
    use nf_formats::crp_mesh::TrackModule;
    use nf_formats::geoprim::{decode_packet, decode_vif, ModelIndex};
    let Ok(path) = std::env::var("NF_ISO") else { eprintln!("skip: NF_ISO not set"); return };
    let iso = IsoImage::open(&path).expect("open NF_ISO");
    if track_crp(&iso, "MIS01").is_none() {
        eprintln!("skip: no /DRIVING/ extents in this image");
        return;
    }
    let mut got = Vec::new();
    for &(viv, ..) in CHAIN {
        let dec = track_crp(&iso, viv).unwrap();
        let crp = Crp::parse(&dec).unwrap();
        let module = TrackModule::load(&crp).unwrap();
        let ix = ModelIndex::new(&module).unwrap();
        let img = &module.image;
        let rd = |p: u32| -> Option<u32> {
            let a = ix.data_offset + p as usize;
            img.get(a..a + 4).map(|b| u32::from_le_bytes(b.try_into().unwrap()))
        };
        let boxes: HashMap<String, ([f32; 3], [f32; 3])> =
            crp.track_articles().unwrap().into_iter().map(|a| (a.name, (a.bbox_min, a.bbox_max))).collect();
        let (mut lists, mut chain_ok, mut own_ok, mut box_chain, mut box_own, mut boxed) = (0, 0, 0, 0, 0, 0);
        let mut coloured = 0;
        for s in &module.symbols {
            let Some(name) = s.name.strip_prefix("__Model:::") else { continue };
            let part = ix.model_part(s.value).unwrap();
            let entries = ix.model_packets(s.value).unwrap();
            let chain: Vec<Vec<_>> = entries.iter().map(|&po| decode_vif(img, &ix.draw_chain(po, part).unwrap(), po).unwrap()).collect();
            let own: Vec<Vec<_>> = entries.iter().map(|&po| decode_packet(img, po).unwrap()).collect();
            coloured += chain.iter().flatten().filter(|s| !s.colors.is_empty()).count();
            let n_of = |v: &Vec<Vec<nf_formats::geoprim::LocalStrip>>| -> Vec<usize> { v.iter().map(|b| b.iter().map(|s| s.positions.len()).sum()).collect() };
            // 1. the Colours list
            let cl = rd(s.value + 0xb8).unwrap();
            let (cname, n) = (rd(cl + 4).unwrap(), rd(cl + 8).unwrap() as usize);
            if cname == rd(s.value).unwrap() && n <= 64 {
                lists += 1;
                let counts: Vec<usize> = (0..n).map(|i| (rd(cl + 0x10 + 16 * i as u32).unwrap() & 0xffff) as usize).collect();
                let (c, o) = (n_of(&chain), n_of(&own));
                chain_ok += (c.len() >= counts.len() && c[..counts.len()] == counts[..]) as usize;
                own_ok += (o.len() >= counts.len() && o[..counts.len()] == counts[..]) as usize;
            }
            // 2. the Base box
            let article = name.split("::").next().unwrap();
            if let Some((lo, hi)) = boxes.get(article) {
                let inside = |v: &Vec<Vec<nf_formats::geoprim::LocalStrip>>| {
                    v.iter().flatten().flat_map(|s| s.positions.iter()).all(|p| (0..3).all(|k| p[k] >= lo[k] - 1.0 / 128.0 && p[k] <= hi[k] + 1.0 / 128.0))
                };
                boxed += 1;
                box_chain += inside(&chain) as usize;
                box_own += inside(&own) as usize;
            }
        }
        eprintln!("{viv}: Colours lists {lists}: match chain {chain_ok}, own {own_ok} | {boxed} models with a box: chain inside {box_chain}, own inside {box_own} | coloured strips {coloured}");
        got.push((viv, lists, chain_ok, own_ok, box_chain, box_own, coloured));
    }
    assert_eq!(got, CHAIN.to_vec());
}
