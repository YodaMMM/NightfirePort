//! Disc test for the car shading data (docs/formats/car-model.md §10): every strip of every car
//! `.crp` of the 8 driving archives gets a render method (`carmesh::CarMaterial`, the external
//! symbol at its GeoPrim header + 8) and, unless it is the shadow, a CarRender lighting class (the
//! `CarLighting<X>Props` its render list uploads); each archive's `Render\CarRender\default.tun`
//! parses with every instance and class; the lighting tuning gives unit light directions.
//!
//! Opt-in via NF_ISO (the full image; a slim image without the `/DRIVING/` extents skips).

use std::collections::{BTreeMap, HashMap};

use nf_formats::carmesh::{car_mesh, entry_material, model_entries, CarLightClass, CarMaterial};
use nf_formats::crp::Crp;
use nf_formats::crp_mesh::TrackModule;
use nf_formats::geoprim::ModelIndex;
use nf_formats::iso9660::IsoImage;
use nf_formats::rendertun::{CarRender, Lighting};

const ARCHIVES: &[&str] = &["MIS01", "MIS11", "MIS13A", "MIS13B", "MIS13C", "MIS3", "MIS4", "RACE"];

/// Strips per (render method, lighting class) over all car files of the 8 archives, LOD 0 of every
/// placed `in#0` item, counted once per model set (as carmesh_disc counts strips) [D measured].
/// The total, 36,397, is carmesh_disc's strip total; no `Shadow` strip and no positions-only strip
/// is among them.
const STRIPS: &[(CarMaterial, Option<CarLightClass>, usize)] = &[
    (CarMaterial::TexturePacked, Some(CarLightClass::Paint), 10708),
    (CarMaterial::TextureDullPacked, Some(CarLightClass::Dull), 16431),
    (CarMaterial::TextureSwitchPacked, Some(CarLightClass::Paint), 972),
    (CarMaterial::GouraudDullPacked, Some(CarLightClass::Dull), 135),
    (CarMaterial::GlassPacked, Some(CarLightClass::Glass), 198),
    (CarMaterial::Chrome, Some(CarLightClass::Chrome), 2300),
    (CarMaterial::Dash, Some(CarLightClass::Dash), 5653),
];

fn read(iso: &IsoImage, lba: u32, off: u32, size: u32) -> Option<Vec<u8>> {
    let raw = iso.read_bytes(lba as u64 * 2048 + off as u64, size as u64).ok()?;
    if raw.iter().take(64).all(|&b| b == 0) {
        return None;
    }
    Some(if nf_formats::refpack::is_refpack(&raw) { nf_formats::refpack::decompress(&raw).unwrap() } else { raw })
}

/// `(name, bytes)` of every archive member whose lower-case name passes `want`.
fn members(iso: &IsoImage, viv: &str, want: impl Fn(&str) -> bool) -> Option<Vec<(String, Vec<u8>)>> {
    let e = iso.find(&format!("/DRIVING/{viv}.VIV"))?;
    let g = nf_formats::disc::read_bigf(iso, e).ok()?;
    let mut out = Vec::new();
    for x in &g.entries {
        let n = x.name.to_ascii_lowercase();
        if want(&n) {
            out.push((n, read(iso, e.lba, x.offset, x.size)?));
        }
    }
    Some(out)
}

fn iso() -> Option<IsoImage> {
    let Ok(path) = std::env::var("NF_ISO") else {
        eprintln!("skip: NF_ISO not set");
        return None;
    };
    let iso = IsoImage::open(&path).expect("open NF_ISO");
    if members(&iso, "MIS01", |n| n.ends_with(".crp") && n.contains(r"car\model")).map_or(true, |m| m.is_empty()) {
        eprintln!("skip: no /DRIVING/ extents in this image");
        return None;
    }
    Some(iso)
}

fn is_car(n: &str) -> bool { n.starts_with(r"data\car\model\") && n.ends_with(".crp") }

#[test]
fn every_car_strip_has_a_render_method_and_a_lighting_class() {
    let Some(iso) = iso() else { return };
    let mut counts: BTreeMap<(CarMaterial, Option<CarLightClass>), usize> = BTreeMap::new();
    // render method → the lighting classes its strips carry.
    let mut classes: BTreeMap<CarMaterial, BTreeMap<Option<CarLightClass>, usize>> = BTreeMap::new();
    let (mut files, mut entries, mut microcode_checked) = (0usize, 0usize, 0usize);
    for viv in ARCHIVES {
        for (name, crp_bytes) in members(&iso, viv, is_car).unwrap() {
            files += 1;
            let m = car_mesh(&crp_bytes).unwrap_or_else(|e| panic!("{viv} {name}: {e}"));
            let mut seen = std::collections::HashSet::new();
            for p in &m.parts {
                if p.strips.is_empty() || !seen.insert(p.set) {
                    continue;
                }
                for s in &p.strips {
                    *counts.entry((s.material, s.lighting)).or_default() += 1;
                    *classes.entry(s.material).or_default().entry(s.lighting).or_default() += 1;
                    if s.material == CarMaterial::Shadow {
                        assert!(s.uvs.is_empty() && s.normals.is_empty(), "{viv} {name}: a shadow strip with attributes");
                    } else {
                        assert!(s.lighting.is_some(), "{viv} {name} item {}: {:?} strip with no lighting class", p.index, s.material);
                        assert!(!s.normals.is_empty(), "{viv} {name} item {}: {:?} strip without normals", p.index, s.material);
                        assert_eq!(s.uvs.is_empty(), !s.material.textured(), "{viv} {name} item {}: {:?} UVs vs its Texture slot", p.index, s.material);
                    }
                }
            }
            // Independent of the header word: every entry whose render list loads microcode (op 1)
            // loads `<material>0__EAGLMicroCode` [M FUN_002a3290 case 1 uploads `*(op + 4)`].
            let crp = Crp::parse(&crp_bytes).unwrap();
            let module = TrackModule::load(&crp).unwrap();
            let ext = module.external_refs().unwrap();
            // The EnvMapState2C/2G ALPHA words (`FUN_001c1950`) are not what the car passes use.
            assert!(ext.values().all(|n| !n.contains("EnvMapState2")), "{viv} {name}: names an EnvMapState2 word");
            let ix = ModelIndex::new(&module).unwrap();
            for sym in module.symbols.iter().filter(|s| s.name.starts_with("__Model:::")) {
                for po in model_entries(&ix, sym.value).unwrap() {
                    entries += 1;
                    let mat = entry_material(&ix, &ext, po).unwrap_or_else(|e| panic!("{viv} {name}: {e}"));
                    for (at, o) in ix.render_ops(po).unwrap() {
                        if o >> 24 == 1 {
                            let mc = ext.get(&(at + 4)).unwrap_or_else(|| panic!("{viv} {name} entry {po:#x}: op 1 names no microcode"));
                            assert_eq!(mc, &format!("{}0__EAGLMicroCode", mat.symbol()), "{viv} {name} entry {po:#x}");
                            microcode_checked += 1;
                        }
                    }
                }
            }
        }
    }
    eprintln!("{files} car files, {entries} group entries, {microcode_checked} microcode loads matching their header's method");
    for (m, c) in &classes {
        eprintln!("  {m:?}: {c:?}");
    }
    let got: Vec<_> = counts.iter().map(|(&(m, c), &n)| (m, c, n)).collect();
    eprintln!("STRIPS = {got:?}");
    // Each render method uploads one class [D].
    for (m, c) in &classes {
        assert_eq!(c.len(), 1, "{m:?} strips carry several classes: {c:?}");
    }
    assert!(microcode_checked > 0);
    assert_eq!(got, STRIPS.to_vec());
}

/// The CarRender table of every archive: the 52 instances in each, the vanquish values pinned [D].
#[test]
fn every_archives_car_render_table_parses_with_all_instances() {
    let Some(iso) = iso() else { return };
    let mut first: Option<CarRender> = None;
    for viv in ARCHIVES {
        let tun = members(&iso, viv, |n| n == r"data\tuning\render\carrender\default.tun").unwrap();
        assert_eq!(tun.len(), 1, "{viv}: CarRender\\default.tun");
        let c = CarRender::parse(&String::from_utf8_lossy(&tun[0].1)).unwrap_or_else(|e| panic!("{viv}: {e}"));
        assert_eq!(c.entries.len(), 52, "{viv}: instances");
        assert_eq!(c.entries.iter().filter(|e| e.name == "default").count(), 2);
        for n in ["vanquish", "taxi", "Characters", "DynamicObjects"] {
            assert!(c.get(n).is_some(), "{viv}: no {n}");
        }
        let v = c.get("vanquish").unwrap();
        let p = v.props(CarLightClass::Paint);
        assert_eq!((p.diffuse, p.ambient, p.reflect, p.specular), (0.42, 0.79, 0.38, 0.37), "{viv}: vanquish [Paint]");
        let g = v.props(CarLightClass::Glass);
        assert_eq!((g.diffuse, g.ambient, g.reflect, g.specular), (2.0, 1.0, 1.0, 0.5), "{viv}: vanquish [Glass]");
        let d = v.props(CarLightClass::Dull);
        assert_eq!((d.diffuse, d.ambient), (0.800001, 0.57), "{viv}: vanquish [Dull]");
        assert_eq!((v.fresnel, v.reflection_warp, v.specular_spot), (0.21, 0.37, 0.900002), "{viv}: vanquish");
        assert_eq!((c.sky_blend, c.sky_bright, c.refl_angle), (24, 127, 120.0), "{viv}: globals");
        match &first {
            None => first = Some(c),
            Some(f) => assert_eq!(&c, f, "{viv}: differs from MIS01's table"),
        }
    }
}

/// Every track's Light 1 / Light 2 directions are unit vectors at or above the horizon.
#[test]
fn every_tracks_light_directions_are_unit_and_point_up() {
    let Some(iso) = iso() else { return };
    let mut by: HashMap<&str, [[f32; 3]; 2]> = HashMap::new();
    for viv in ARCHIVES {
        let tun = members(&iso, viv, |n| n.starts_with(r"data\tuning\render\lighting\")).unwrap();
        assert_eq!(tun.len(), 1, "{viv}: Lighting tun");
        let l = Lighting::parse(&String::from_utf8_lossy(&tun[0].1)).unwrap();
        let (a, b) = (l.world.light1_dir(), l.world.light2_dir());
        for d in [a, b] {
            assert!((d.iter().map(|c| c * c).sum::<f32>() - 1.0).abs() < 1e-5, "{viv}: {d:?}");
            assert!(d[1] >= -1e-6, "{viv}: {d:?} below the horizon");
        }
        assert_eq!((l.world.dual_hemi_angle_x, l.character.dual_hemi_angle_x), (-90.0, -90.0), "{viv}: dual hemi");
        eprintln!("{viv}: world light1 {a:?} light2 {b:?}");
        by.insert(viv, [a, b]);
    }
    assert_eq!(by.len(), ARCHIVES.len());
}
