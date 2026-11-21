//! The driving effects' data against the disc [D] (docs/systems/driving-effects.md): the particle
//! types DRIVING.ELF carries in `.data`, its slot / surface tables, the per-track particle sheets,
//! `Render\FX\<track>.tun`, `Render\Glare\default.tun`, `Render\SwitchEffects\default.tun` and
//! `data\render\decal.dat` in every `/DRIVING/*.VIV`. Opt-in: NF_ISO.

use nf_formats::drivefx::{self, fourcc, FxTuning};
use nf_formats::{disc, iso9660::IsoImage, refpack, ssh::Ssh};
use nf_game::driving::effects::{DriveRng, Emitter, EmitterFrame, ParticleDraw, ParticleLib, ParticleSystem};

const VIVS: [&str; 8] = ["MIS01", "MIS11", "MIS13A", "MIS13B", "MIS13C", "MIS3", "MIS4", "RACE"];

fn viv_file(iso: &IsoImage, viv: &str, name: &str) -> Option<Vec<u8>> {
    let entry = iso.find(&format!("/DRIVING/{viv}.VIV"))?;
    let bigf = disc::read_bigf(iso, entry).ok()?;
    let e = bigf.entries.iter().find(|x| x.name.eq_ignore_ascii_case(name))?;
    let raw = iso.read_bytes(entry.lba as u64 * 2048 + e.offset as u64, e.size as u64).ok()?;
    Some(if refpack::is_refpack(&raw) { refpack::decompress(&raw).expect("refpack") } else { raw })
}

fn viv_text(iso: &IsoImage, viv: &str, name: &str) -> Option<String> {
    viv_file(iso, viv, name).map(|b| String::from_utf8_lossy(&b).into_owned())
}

fn driving_elf(iso: &IsoImage) -> Vec<u8> {
    let e = iso.find("/DRIVING.ELF").expect("DRIVING.ELF");
    iso.read_file(e).expect("read")
}

#[test]
fn the_elf_particle_types_decode_as_traced() {
    let Ok(path) = std::env::var("NF_ISO") else { eprintln!("NF_ISO not set - skipping"); return };
    let iso = IsoImage::open(&path).expect("iso");
    let elf = driving_elf(&iso);
    let types = drivefx::elf_types(&elf).expect("types");
    let get = |k: &[u8; 4]| types.iter().find(|(x, _)| *x == fourcc(k)).map(|(_, t)| *t).unwrap();

    let d = get(b"dflt");
    assert_eq!((d.name, d.life, d.count, d.flags), (fourcc(b"smk5"), 80, 200, 8));
    assert_eq!(d.keys[0][0], 255.0);
    let s = get(b"SMRT");
    assert_eq!((s.name, s.life, s.count, s.speed, s.drag), (fourcc(b"smk5"), 60, 30, 0.015, 0.96));
    assert_eq!((s.size0, s.size1, s.gravity, s.spin_var), (0.2, 0.7, -0.2, 180.0));
    assert_eq!(s.cell, [96.5 / 256.0, 160.5 / 256.0, 31.0 / 256.0, 0.0], "FUN_001ede48: the smoke cell");
    let k = get(b"SPRK");
    assert_eq!((k.flags, k.life, k.delay, k.duration, k.gravity), (0x22, 24, 3, 1, 30.0));
    assert_eq!(k.keys[0], [255.0, 512.0, 512.0, 192.0]);
    let w = get(b"WATR");
    assert_eq!((w.name, w.gravity, w.cell), (fourcc(b"grvl"), 4.0, s.cell));
    for (key, t) in &types {
        assert!(t.life <= 255, "{key:08x}: lifetime fits the byte");
        assert_eq!(t.slot, 0, "{key:08x}: +0xac is written only when added");
    }
    // the slot / surface tables
    assert_eq!(drivefx::elf_words(&elf, drivefx::SLOT_TYPES_EE, 14).unwrap(), drivefx::SLOT_TYPES.to_vec());
    assert_eq!(drivefx::elf_words(&elf, drivefx::SURFACE_SPRAY_EE, 16).unwrap(), drivefx::SURFACE_SPRAY.to_vec());
    // the sparks' up vector and the spark-velocity scale
    assert_eq!(drivefx::elf_words(&elf, 0x393830, 3).unwrap(), vec![0, 0x3f80_0000, 0]);
    assert_eq!(drivefx::elf_words(&elf, 0x335420, 4).unwrap(), vec![0x4000_0000, 0x4000_0000, 0x4000_0000, 0xbf80_0000]);
    // the RNG constants FUN_001c08c0 writes are immediates; the ELF's initial word at 0x333e64 is 1
    assert_eq!(drivefx::elf_words(&elf, 0x333e64, 1).unwrap(), vec![1]);
}

#[test]
fn a_smoking_wheel_emits_quads_from_the_main_sheet() {
    let Ok(path) = std::env::var("NF_ISO") else { eprintln!("NF_ISO not set - skipping"); return };
    let iso = IsoImage::open(&path).expect("iso");
    let elf = driving_elf(&iso);
    let lib = ParticleLib::from_elf_types(&drivefx::elf_types(&elf).unwrap());
    let mut f = EmitterFrame::new(fourcc(b"SMRT"));
    let mut e = Emitter::new(10.0, &f, &lib);
    e.scale = (1.0 - 0.2) * 4.0 + 0.1; // full slip
    f.axis_a = [0.0, 0.8, 0.0];
    let (mut pool, mut rng) = (ParticleSystem::default(), DriveRng::seeded(0));
    for tick in 1..=120u32 {
        e.tick(1, &f, &lib, &mut rng, &mut pool);
        pool.run_steps(tick, &lib);
    }
    let draws = pool.draws(&lib, |_| None);
    // rate = 15/60 * 3.3 = 0.825 per step; 60-step lifetime → ~49 alive
    assert!((40..=60).contains(&draws.len()), "{} quads", draws.len());
    for d in &draws {
        let ParticleDraw::Quad { cell, size, additive, .. } = d else { panic!("smoke is not a streak") };
        assert_eq!(*cell, [96.5 / 256.0, 160.5 / 256.0, 31.0 / 256.0]);
        assert!(!additive);
        assert!(*size > 0.28 && *size < 1.0, "between birth 0.2*1.4142 and death 0.7*1.4142: {size}");
    }
}

#[test]
fn every_track_ships_its_particle_sheet_and_effect_tunings() {
    let Ok(path) = std::env::var("NF_ISO") else { eprintln!("NF_ISO not set - skipping"); return };
    let iso = IsoImage::open(&path).expect("iso");
    // the sheet name per archive [D] (the ELF names the four files at EE 0x3ac750-0x3ac798)
    let sheets = [("MIS01", "partic01"), ("MIS11", "partic02"), ("MIS13A", "ears_partic01"), ("MIS13B", "ears_partic01"), ("MIS13C", "ears_partic01"), ("MIS3", "sav_partic01"), ("MIS4", "sav_partic01"), ("RACE", "partic01")];
    let fx = [("MIS01", "paris_mis01"), ("MIS11", "uw_mis11")];
    let glare0 = viv_text(&iso, "MIS01", "data\\tuning\\Render\\Glare\\default.tun").expect("glare");
    let switch0 = viv_text(&iso, "MIS01", "data\\tuning\\Render\\SwitchEffects\\default.tun").expect("switch");
    for (viv, sheet) in sheets {
        let b = viv_file(&iso, viv, &format!("data\\render\\{sheet}.ssh")).unwrap_or_else(|| panic!("{viv}: {sheet}.ssh"));
        let s = Ssh::parse(&b).expect("ssh");
        let main = s.textures.iter().find(|t| t.name == "MAIN").expect("MAIN");
        assert_eq!((main.width, main.height), (256, 256), "{viv}: the 256x256 atlas the cells index");
        assert!(s.textures.iter().any(|t| t.name == "puff"), "{viv}: puff");
        assert_eq!(viv_text(&iso, viv, "data\\tuning\\Render\\Glare\\default.tun").as_ref(), Some(&glare0), "{viv}: one glare file");
        assert_eq!(viv_text(&iso, viv, "data\\tuning\\Render\\SwitchEffects\\default.tun").as_ref(), Some(&switch0), "{viv}: one switch file");
        let dec = viv_file(&iso, viv, "data\\render\\decal.dat").expect("decal.dat");
        assert_eq!(dec.len(), drivefx::DECAL_DAT_SIZE, "{viv}: the loader copies 0x280");
        let d = drivefx::parse_decals(&dec).unwrap();
        assert!(d.iter().enumerate().all(|(i, r)| r.index == i as u32 && r.corners.iter().all(|c| c[2] == 1.0 && c[3] == 1.0)), "{viv}");
    }
    let t = FxTuning::parse(&viv_text(&iso, "MIS01", "data\\tuning\\Render\\FX\\paris_mis01.tun").unwrap());
    assert!(t.do_glow && t.vehicle_sparks && !t.particulate);
    assert_eq!(t.water_sun_dir, 157.0);
    let suv = t.bullet_streaks.iter().find(|b| b.class == "SUV").unwrap();
    assert_eq!((suv.colour, suv.texture, suv.width, suv.length, suv.speed, suv.count), ([255, 95, 150, 200], 3, 0.054, 6.7, 103.5, 1));
    for (viv, track) in fx {
        assert!(viv_text(&iso, viv, &format!("data\\tuning\\Render\\FX\\{track}.tun")).is_some(), "{viv}: FX tun");
    }
    let g = drivefx::parse_glare(&glare0);
    assert_eq!(g.len(), 80);
    assert_eq!((g[0].tex, g[0].col[0], g[0].size[0], g[0].zbias), ([9, 0], [255, 70, 70, 80], 0.3, 2.0));
    assert_eq!(g[64].tex[0], 0xcbcb_cbcb, "ID 64 is the fill pattern");
    let sw = drivefx::parse_switch_effects(&switch0);
    assert_eq!(sw.len(), 16);
    assert_eq!((sw[2].name.as_str(), sw[2].vscroll, sw[2].glow, sw[2].illuminate), ("Brakelights", 0.03, 0.08, 0.53));
    assert_eq!(sw[7].glow, 1.0, "Dashboard");
}

