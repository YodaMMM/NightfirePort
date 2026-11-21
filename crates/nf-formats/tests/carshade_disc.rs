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

/// The car passes (docs/formats/car-model.md §10.6): every GeoPrimState a car list uploads parses
/// with the traced handlers; the specular / reflection passes of every strip find the `name=envmap`
/// (paint, chrome, glass: Cs·As + Cd) or `name=specmap` (dash: Cs·Ad + Cd) state in the slot the
/// microcode kicks; the run-time maps are uploaded to the SpecularMap / ReflectionMap2 slots; and
/// the textures they name are in the archives' sheets [D].
#[test]
fn every_car_pass_finds_its_traced_state_and_textures() {
    use nf_formats::carmesh::{entry_uploads, CarPassKind, CarPassTexture, GeoPrimState, Upload, SPECULAR_MAP_ENTRY};
    let Some(iso) = iso() else { return };
    // (material, pass) → state string (None = no upload in this model before the strip) → strips.
    let mut by: BTreeMap<(CarMaterial, CarPassKind), BTreeMap<Option<String>, usize>> = BTreeMap::new();
    // VU1 slot → external texture names op 6 puts there.
    let mut ext_slots: BTreeMap<u16, BTreeMap<String, usize>> = BTreeMap::new();
    let (mut states, mut base_tex_same, mut base_tex_diff) = (0usize, 0usize, 0usize);
    for viv in ARCHIVES {
        for (name, crp_bytes) in members(&iso, viv, is_car).unwrap() {
            let crp = Crp::parse(&crp_bytes).unwrap();
            let module = TrackModule::load(&crp).unwrap();
            let ext = module.external_refs().unwrap();
            let ix = ModelIndex::new(&module).unwrap();
            for sym in module.symbols.iter().filter(|s| s.name.starts_with("__Model:::")) {
                let part = ix.model_part(sym.value).unwrap();
                for po in model_entries(&ix, sym.value).unwrap() {
                    for u in entry_uploads(&ix, &ext, po, part).unwrap_or_else(|e| panic!("{viv} {name}: {e}")) {
                        match u.what {
                            Upload::State(s) => {
                                GeoPrimState::parse(&s).unwrap_or_else(|e| panic!("{viv} {name}: {e}"));
                                states += 1;
                            }
                            Upload::ExternalTexture(t) => *ext_slots.entry(u.vu).or_default().entry(t).or_default() += 1,
                            _ => {}
                        }
                    }
                }
            }
            let m = car_mesh(&crp_bytes).unwrap();
            let mut seen = std::collections::HashSet::new();
            for p in &m.parts {
                if p.strips.is_empty() || !seen.insert(p.set) {
                    continue;
                }
                for s in &p.strips {
                    for pass in s.passes() {
                        *by.entry((s.material, pass.kind)).or_default().entry(pass.state.clone()).or_default() += 1;
                        if pass.kind == CarPassKind::Base {
                            if let CarPassTexture::Module(at) = pass.texture {
                                if s.slot_textures.get(&at) == s.texture.as_ref() {
                                    base_tex_same += 1;
                                } else {
                                    // Every disagreement is a slot whose descriptor has no name.
                                    assert!(s.slot_textures.get(&at).is_none(), "{viv} {name}: base slot {:?} vs {:?}", s.slot_textures.get(&at), s.texture);
                                    base_tex_diff += 1;
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    for (k, v) in &by {
        eprintln!("{k:?}: {v:?}");
    }
    eprintln!("ext slots {ext_slots:?}; {states} states; base texture slot = strip texture {base_tex_same}, differs {base_tex_diff}");
    // The overlay passes: one state each, the traced blend.
    for ((m, kind), v) in &by {
        if !matches!(kind, CarPassKind::Specular | CarPassKind::Reflection) {
            continue;
        }
        for st in v.keys().flatten() {
            let g = GeoPrimState::parse(st).unwrap();
            let want = if *m == CarMaterial::Dash { 0x80_0000_0058 } else { 0x80_0000_0048 };
            assert!(st.starts_with("name="), "{m:?} {kind:?}: {st}");
            assert_eq!(g.alpha, want, "{m:?} {kind:?}: {st}");
            assert!(g.abe() && g.tme() && !g.fge(), "{m:?} {kind:?}: {st}");
        }
    }
    // Strips per overlay pass and state [D measured]: none lacks its state.
    let count = |m: CarMaterial, k: CarPassKind| by.get(&(m, k)).map(|v| v.iter().map(|(s, n)| (s.clone(), *n)).collect::<Vec<_>>()).unwrap_or_default();
    let env = |prim: &str, n: usize| (Some(format!("name=envmap;primtype={prim}")), n);
    for k in [CarPassKind::Specular, CarPassKind::Reflection] {
        assert_eq!(count(CarMaterial::TexturePacked, k), vec![env("tristrip", 10708)], "{k:?}");
        assert_eq!(count(CarMaterial::Chrome, k), vec![env("tristrip", 2300)], "{k:?}");
        assert_eq!(count(CarMaterial::GlassPacked, k), vec![env("trilist", 198)], "{k:?}");
    }
    assert_eq!(
        count(CarMaterial::Dash, CarPassKind::Specular),
        vec![(Some("name=specmap;primtype=trilist".to_owned()), 34), (Some("name=specmap;primtype=tristrip".to_owned()), 5619)]
    );
    assert!(count(CarMaterial::Dash, CarPassKind::Reflection).is_empty());
    assert_eq!(count(CarMaterial::GlassPacked, CarPassKind::Base), vec![(Some("name=glassdmgmap;primtype=trilist".to_owned()), 198)]);
    assert_eq!((states, base_tex_same, base_tex_diff), (5013, 34770, 1492));
    assert_eq!(ext_slots.values().map(|m| m.values().sum::<usize>()).collect::<Vec<_>>(), vec![1906, 1906]);
    assert_eq!(ext_slots.get(&0x11).map(|m| m.keys().cloned().collect::<Vec<_>>()), Some(vec!["__EAGL::TAR:::GAME::SpecularMap".to_owned()]));
    assert_eq!(ext_slots.get(&0x18).map(|m| m.keys().cloned().collect::<Vec<_>>()), Some(vec!["__EAGL::TAR:::GAME::ReflectionMap".to_owned()]));
    // The sheets the passes and the reflection map name.
    for viv in ARCHIVES {
        for (sheet, want) in [(r"data\render\common.ssh", vec![(SPECULAR_MAP_ENTRY, 32), ("rmsk", 64)]), (r"data\render\ext.ssh", vec![("cref", 128)])] {
            let f = members(&iso, viv, |n| n == sheet).unwrap();
            assert_eq!(f.len(), 1, "{viv}: {sheet}");
            let ssh = nf_formats::ssh::Ssh::parse(&f[0].1).unwrap();
            for (n, w) in want {
                let t = ssh.textures.iter().find(|t| t.name == n).unwrap_or_else(|| panic!("{viv}: no {n} in {sheet}"));
                assert_eq!((t.width, t.height), (w, w), "{viv}: {n}");
            }
        }
    }
}
