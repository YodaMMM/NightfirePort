//! Disc test for `nf_game::driving::smashfx`: the Giotto `gg` / `gd` / `gs` elements of the
//! three effect galleries, the smash and explosion effects run from the disc, the effect blocks
//! of the paris street lights and a traffic car model, and the decal ring with the disc's
//! `decal.dat` (docs/systems/driving-effects.md §9-§12).
//!
//! Opt-in via NF_ISO (the full image; a slim image without the `/DRIVING/` extents skips).

use nf_formats::crp::{tag, Crp};
use nf_formats::drivefx::parse_decals;
use nf_formats::gal::Gal;
use nf_formats::iso9660::IsoImage;
use nf_formats::trackfx::{giotto_effect, parse_ef_list, placed_effects, EfEntry, GiottoElement, PlacedEffect};
use nf_game::driving::effects::{DriveRng, ParticleLib, ParticleSystem};
use nf_game::driving::smashfx::*;

fn viv_files(iso: &IsoImage, viv: &str) -> Option<Vec<(String, Vec<u8>)>> {
    let e = iso.find(&format!("/DRIVING/{viv}.VIV"))?;
    let g = nf_formats::disc::read_bigf(iso, e).ok()?;
    let mut out = Vec::new();
    for x in &g.entries {
        let raw = iso.read_bytes(e.lba as u64 * 2048 + x.offset as u64, x.size as u64).ok()?;
        if raw.len() >= 64 && raw.iter().take(64).all(|&b| b == 0) {
            return None;
        }
        let d = if nf_formats::refpack::is_refpack(&raw) { nf_formats::refpack::decompress(&raw).unwrap_or(raw) } else { raw };
        out.push((x.name.to_ascii_lowercase(), d));
    }
    Some(out)
}

fn gallery(misc: &[(String, Vec<u8>)], name: &str) -> Gal {
    Gal::parse(&misc.iter().find(|(n, _)| n.ends_with(name) && n.contains("gfx")).unwrap().1).unwrap()
}

/// (gallery, anims, gp, gg, gd, gs elements with a key, elements left over) [D measured].
const ELEMENTS: &[(&str, usize, usize, usize, usize, usize, usize)] = &[
    ("gall1.gal", 106, 330, 18, 38, 21, 0),
    ("sav_gall1.gal", 79, 208, 15, 39, 13, 0),
    ("ears_gall1.gal", 77, 297, 17, 26, 6, 11),
];

#[test]
fn every_gallery_parses_its_glare_burst_and_sound_elements() {
    let Ok(path) = std::env::var("NF_ISO") else { return };
    let iso = IsoImage::open(&path).unwrap();
    let Some(misc) = viv_files(&iso, "MISC") else { return };
    for &(g, ..) in ELEMENTS {
        let gal = gallery(&misc, g);
        let mut n = (gal.anims.len(), 0, 0, 0, 0, 0);
        for a in &gal.anims {
            let e = giotto_effect(&gal, &a.name).unwrap();
            for el in &e.elements {
                match el {
                    GiottoElement::Emitter(_) => n.1 += 1,
                    GiottoElement::Glare(k) => {
                        n.2 += 1;
                        assert!(k.windows(2).all(|w| w[0].frame <= w[1].frame), "{g} {}: gg key frames ascend", a.name);
                    }
                    GiottoElement::Burst(_) => n.3 += 1,
                    GiottoElement::Sound(_) => n.4 += 1,
                }
            }
            n.5 += e.others.len();
        }
        eprintln!("{g}: {n:?}");
        let w = ELEMENTS.iter().find(|r| r.0 == g).unwrap();
        assert_eq!(n, (w.1, w.2, w.3, w.4, w.5, w.6), "{g}");
    }
}

#[test]
fn glass_and_sparks_run_from_the_disc() {
    let Ok(path) = std::env::var("NF_ISO") else { return };
    let iso = IsoImage::open(&path).unwrap();
    let Some(misc) = viv_files(&iso, "MISC") else { return };
    let gal = gallery(&misc, "gall1.gal");
    let (mut lib, mut types) = (ParticleLib::default(), GiottoTypes::default());
    let (mut rng, mut grng, mut pool) = (DriveRng::seeded(0), giotto_rng(), ParticleSystem::default());
    let id = [0.0, 0.0, 0.0, 1.0];
    // (effect, debris pieces per type after the first run, the sound, glares over 61 ticks) [D
    // through the M readers].
    let want: &[(&str, &[(usize, usize)], Option<(&str, u32, f32, f32)>, usize)] = &[
        ("Wd_Glass", &[(5, 4), (6, 3), (7, 7), (8, 7)], Some(("SFX_GlassShatter3", 2, 10.0, 100.0)), 0),
        ("Wd_Sprk", &[(5, 4), (6, 3)], None, 61),
        ("Sm_Wsplinter", &[(17, 7), (18, 7)], None, 0),
        ("Sm_Debris", &[(11, 6), (12, 6), (13, 6)], None, 0),
        ("Ex_SmallD2", &[(0, 3), (1, 2), (2, 2)], None, 0),
        ("Ex_Small", &[], None, 0),
    ];
    for &(name, pieces, sound, want_glares) in want {
        let e = giotto_effect(&gal, name).unwrap();
        let mut run = GiottoRun::new(&e, [0.0, 1.0, 0.0], id, &mut types, &mut lib);
        let mut debris = DebrisPool::default();
        let f0 = run.tick(1, [0.0, 1.0, 5.0], &lib, &mut rng, &mut grng, &mut pool, &mut debris, 1000);
        let per_type: Vec<(usize, usize)> = debris.types.iter().enumerate().filter(|(_, t)| !t.pieces.is_empty()).map(|(k, t)| (k, t.pieces.len())).collect();
        let mut glares = f0.glares.len();
        for t in 1..=60u32 {
            glares += run.tick(1, [0.0, 1.0, 5.0], &lib, &mut rng, &mut grng, &mut pool, &mut debris, 1000 + t).glares.len();
            pool.run_steps(t, &lib);
        }
        eprintln!("{name}: flags {:#x} life {} debris {per_type:?} sounds {:?} glares over 61 ticks {glares} particles {}", e.flags, e.life, f0.sounds.iter().map(|s| (&s.name, s.bank, s.volume, s.near, s.far)).collect::<Vec<_>>(), pool.cur.len());
        assert_eq!(per_type, pieces, "{name}");
        assert_eq!(f0.sounds.first().map(|s| (s.name.as_str(), s.bank, s.near, s.far)), sound, "{name}");
        assert_eq!(glares, want_glares, "{name}");
        assert_eq!(e.flags & 1, 1, "{name}: a counted (finite) effect");
    }
}

fn track_crp(iso: &IsoImage, viv: &str) -> Option<Vec<u8>> {
    let files = viv_files(iso, viv)?;
    files.into_iter().find(|(n, _)| n.starts_with("data\\track\\") && n.ends_with(".crp")).map(|(_, d)| d)
}

#[test]
fn paris_street_light_smash_effects_sit_outside_the_block() {
    let Ok(path) = std::env::var("NF_ISO") else { return };
    let iso = IsoImage::open(&path).unwrap();
    let Some(cdata) = track_crp(&iso, "MIS01") else { return };
    let crp = Crp::parse(&cdata).unwrap();
    let placed = placed_effects(&crp).unwrap();
    let item = placed.iter().find(|p| p.article == "P_streetlight").unwrap().item;
    let mine: Vec<&PlacedEffect> = placed.iter().filter(|p| p.item == item).collect();
    let names: Vec<_> = mine.iter().map(|p| (p.e.kind, p.e.flags, p.e.b17, p.giotto_name())).collect();
    eprintln!("P_streetlight item {item}: {names:?}");
    let mut b = EffectBlock::from_entries(mine.iter().copied());
    eprintln!("block: {} entries, giotto {:#x} events {:#x} damage {:#x}", b.entries.len(), b.giotto, b.events, b.damage);
    let mut start = |_: &str, _: [f32; 3]| Some(1);
    b.build(false, &mut start);
    eprintln!("running after build {:#x}", b.running);
    // The lamp glare (flags 0x14) is the block's only entry and is lit from the build; the
    // sparks and the glass (flags 0) are outside it: an explosion action starts them by name.
    assert_eq!(names, [(2, 0x14, 1, None), (6, 0, 2, Some("Wd_Sprk")), (6, 0, 3, Some("Wd_Sprk")), (6, 0, 4, Some("Wd_Glass"))]);
    assert_eq!((b.entries.len(), b.running, b.giotto), (1, 1, 0));
    let routes: Vec<_> = mine.iter().map(|p| static_explosion(p.e.flags, p.giotto_name(), p.e.b17, 0)).collect();
    assert_eq!(routes, [StaticExplosion::Event1 { id: 1 }, StaticExplosion::Start("Wd_Sprk".into()), StaticExplosion::Start("Wd_Sprk".into()), StaticExplosion::Start("Wd_Glass".into())]);
    let n = placed.iter().filter(|p| p.article == "P_streetlight").count();
    assert_eq!(n, 4 * 201);
}

/// Every ef entry of one car model `.crp`, as scene-object entries.
fn car_entries(files: &[(String, Vec<u8>)], model: &str) -> Vec<PlacedEffect> {
    let (_, d) = files.iter().find(|(n, _)| n.ends_with(model)).unwrap();
    let crp = Crp::parse(d).unwrap();
    let mut out = Vec::new();
    for arti in crp.pool().unwrap().iter().filter(|r| r.tag == tag::ARTI) {
        let (s, e) = arti.window(crp.data).unwrap();
        let recs = crp.window_records(s, e).unwrap();
        let mut sr = std::collections::HashMap::new();
        for r in &recs {
            if r.tag_code() == 0x7372_2020 {
                let p = crp.payload(r).unwrap();
                let k = p.iter().position(|&b| b == 0).unwrap_or(p.len());
                sr.insert(r.instance().unwrap_or(0), String::from_utf8_lossy(&p[..k]).into_owned());
            }
        }
        for r in recs.iter().filter(|r| r.tag_code() == nf_formats::trackobj::TAG_EF) {
            for en in parse_ef_list(crp.payload(r).unwrap()) {
                out.push(scene_entry(en, (en.kind == 6).then(|| sr.get(&(en.word18 as u16)).cloned()).flatten()));
            }
        }
    }
    out
}

fn scene_entry(e: EfEntry, name: Option<String>) -> PlacedEffect {
    PlacedEffect {
        item: 0, article: String::new(), part: 0, source_part: 0, entry: 0, scene_object: true, e, name, world: e.pos, world_dir: [0.0; 3],
        item_origin: [0.0; 3], item_word15: 0, sub_bit7: false, sub_index: 0,
    }
}

#[test]
fn a_traffic_car_breaks_its_glass_on_panel_damage() {
    let Ok(path) = std::env::var("NF_ISO") else { return };
    let iso = IsoImage::open(&path).unwrap();
    let mut found = None;
    for viv in ["MIS01", "MIS11", "MIS13A", "MIS13B", "MIS13C", "MIS3", "MIS4", "RACE"] {
        let Some(files) = viv_files(&iso, viv) else { return };
        if files.iter().any(|(n, _)| n.ends_with("\\coupe.crp")) {
            found = Some((viv, files));
            break;
        }
    }
    let (viv, files) = found.unwrap();
    eprintln!("coupe.crp from {viv}");
    let ents = car_entries(&files, "\\coupe.crp");
    let mut b = EffectBlock::from_entries(&ents);
    eprintln!("coupe: {} entries, block {} (damage {:#x}, events {:#x}, giotto {:#x})", ents.len(), b.entries.len(), b.damage, b.events, b.giotto);
    let mut started = Vec::new();
    let mut start = |n: &str, _: [f32; 3]| { started.push(n.to_owned()); None };
    let mut panel = [[0.0f32; 2]; 16];
    panel[1] = [0.8, 0.0];
    let (e10, e9) = car_panel_events(&panel);
    let r = b.event(9, e9, 100, &mut start);
    eprintln!("panel 1 at 0.8: events 10 {e10:#x} 9 {e9:#x} -> {r:?}");
    let r0 = b.event(0, 0x8000_0000, 101, &mut start);
    eprintln!("event 0 bit 31 -> {r0:?}");
    assert_eq!((ents.len(), b.entries.len(), b.damage, b.events, b.giotto), (21, 21, 0x76c00, 0x1f_ffff, 0x19_fc00));
    assert_eq!((e10, e9), (0x2, 0x2));
    let got: Vec<(usize, &str)> = r.iter().map(|q| match q { BlockRequest::Start { entry, name, .. } => (*entry, name.as_str()), _ => (99, "") }).collect();
    assert_eq!(got, [(10, "Cr_Dmg2"), (11, "Ex_Glass"), (13, "Ex_Glass"), (14, "Ex_Glass"), (16, "Ex_Glass")]);
    assert!(matches!(&r0[..], [BlockRequest::Start { entry: 12, name, .. }] if name == "Cr_Dmg4"));
    assert_eq!(started.len(), 6);
}

#[test]
fn explosion_scorch_on_the_disc_decal_records() {
    let Ok(path) = std::env::var("NF_ISO") else { return };
    let iso = IsoImage::open(&path).unwrap();
    let Some(files) = viv_files(&iso, "MIS01") else { return };
    let d = &files.iter().find(|(n, _)| n.ends_with("decal.dat")).unwrap().1;
    let rec = parse_decals(d).unwrap();
    let mut ring = DecalRing::new();
    let flat = |p: [f32; 3]| GroundHit { hit: 1, tri: [[p[0], 0.0, p[2]], [p[0] + 1.0, 0.0, p[2]], [p[0], 0.0, p[2] + 1.0]], height: 0.0, surface: 1, no_decal: false, on_object: false };
    assert!(ground_decal(&mut ring, &rec, SCORCH.0, [0.0, 0.5, 0.0], SCORCH.1, flat));
    assert!(ground_decal(&mut ring, &rec, OIL_SLICK.0, [0.0, 0.5, 0.0], OIL_SLICK.1, flat));
    let (s, o) = (ring.slots[0].unwrap(), ring.slots[1].unwrap());
    eprintln!("scorch delay {} alpha {:#x} uv {:?}; oil delay {} alpha {:#x}", s.delay, s.alpha, s.uv[0], o.delay, o.alpha);
    assert_eq!((s.delay, s.alpha, s.rgba), (5, 0x28, 0x007f_7f7f));
    assert_eq!((o.delay, o.alpha), (5, 0x28));
    assert_eq!(s.uv[0], [0.0, 0.5, 1.0, 1.0]);
    for _ in 0..5 { ring.draw_tick(); }
    assert_eq!(ring.slots[0].unwrap().rgba, 0x287f_7f7f, "the scorch shows after five decal draws");
}
