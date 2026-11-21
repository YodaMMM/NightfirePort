//! Disc test for `nf_game::driving::trackfx`: the placed glare sprites and the particle effects
//! started at load on every driving track, built from the disc and run for a few ticks
//! (docs/systems/driving-effects.md §7).
//!
//! Opt-in via NF_ISO (the full image; a slim image without the `/DRIVING/` extents skips).

use nf_formats::crp::Crp;
use nf_formats::drivefx::parse_glare;
use nf_formats::gal::Gal;
use nf_formats::iso9660::IsoImage;
use nf_formats::trackfx::{giotto_effect, giotto_gallery, placed_effects};
use nf_game::driving::effects::{DriveRng, ParticleLib, ParticleSystem};
use nf_game::driving::trackfx::{glare_types, TrackFx};

const TRACKS: &[&str] = &["MIS01", "MIS11", "MIS13A", "MIS13B", "MIS13C", "MIS3", "MIS4", "RACE"];

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

/// (track, glares, Giotto instances started at load, particle types those add) [D measured].
const BUILT: &[(&str, usize, usize, usize)] = &[
    ("MIS01", 603, 10, 1),
    ("MIS11", 234, 0, 0),
    ("MIS13A", 107, 2, 10),
    ("MIS13B", 97, 16, 7),
    ("MIS13C", 49, 9, 1),
    ("MIS3", 95, 2, 6),
    ("MIS4", 102, 0, 0),
    ("RACE", 100, 0, 0),
];

#[test]
fn every_track_builds_its_glares_and_load_started_particles() {
    let Ok(path) = std::env::var("NF_ISO") else { return };
    let iso = IsoImage::open(&path).unwrap();
    let Some(misc) = viv_files(&iso, "MISC") else { return };
    for &viv in TRACKS {
        let Some(files) = viv_files(&iso, viv) else { return };
        let (cname, cdata) = files.iter().find(|(n, _)| n.starts_with("data\\track\\") && n.ends_with(".crp")).unwrap();
        let track = cname.trim_start_matches("data\\track\\").trim_end_matches(".crp");
        let gname = giotto_gallery(track);
        let gal = Gal::parse(&misc.iter().find(|(n, _)| n.ends_with(gname) && n.contains("gfx")).unwrap().1).unwrap();
        let tun = &files.iter().find(|(n, _)| n.contains("glare") && n.ends_with(".tun")).unwrap().1;
        let types = glare_types(&parse_glare(&String::from_utf8_lossy(tun)));
        let crp = Crp::parse(cdata).unwrap();
        let placed = placed_effects(&crp).unwrap();
        let mut lib = ParticleLib::default();
        let mut fx = TrackFx::new(&placed, |n| giotto_effect(&gal, n), &mut lib);
        let got = (fx.glares.len(), fx.giotto.len(), lib.types.len());
        eprintln!("{viv}: glares {}, load-started effects {} ({:?}), particle types {}", got.0, got.1, fx.giotto.iter().map(|g| g.name.as_str()).collect::<Vec<_>>(), got.2);
        let want = BUILT.iter().find(|r| r.0 == viv).unwrap();
        assert_eq!(got, (want.1, want.2, want.3), "{viv}");

        // Standing at every glare in turn (camera row 2 = -facing, LOD scale 1): how many are
        // kept at least once over a 50-tick pulse period. Static glares are distance-culled from
        // their item's origin, so lamp heads high above it drop out at this scale.
        let mut kept_any = 0;
        for (i, g) in fx.glares.iter().enumerate() {
            let cam = [g.pos[0] + 0.01, g.pos[1], g.pos[2]];
            let row2 = [-g.dir[0], -g.dir[1], -g.dir[2]];
            let hit = (0..50u32).any(|t| fx.glares(t, cam, row2, 1.0, &types, |it| it == g.item).kept.iter().any(|(k, _)| *k == i));
            kept_any += hit as usize;
        }
        eprintln!("{viv}: glares kept when the camera stands at them: {kept_any}");

        // The load-started effects put particles in the pool within 60 ticks.
        let (mut rng, mut pool) = (DriveRng::seeded(0), ParticleSystem::default());
        if let Some(cam) = fx.giotto.first().map(|g| g.pos) {
            for t in 1..=60u32 {
                fx.tick_particles(1, cam, &lib, &mut rng, &mut pool);
                pool.run_steps(t, &lib);
            }
            eprintln!("{viv}: particles alive after 60 ticks: {}", pool.cur.len());
            assert!(!pool.cur.is_empty(), "{viv}: the load-started effects emitted nothing");
        }
    }
}

/// The paris traffic lights: five scene objects, one flashing red glare each (type 49:
/// halo colour "255,96,0,0", spike "255,200,20,5"; pulse rate 0.02 per tick, duty 2/3, base
/// 0.5, amplitude 1, triangle; one-sided) [D through the M readers].
#[test]
fn paris_traffic_lights_flash_red() {
    let Ok(path) = std::env::var("NF_ISO") else { return };
    let iso = IsoImage::open(&path).unwrap();
    let Some(files) = viv_files(&iso, "MIS01") else { return };
    let cdata = &files.iter().find(|(n, _)| n.starts_with("data\\track\\") && n.ends_with(".crp")).unwrap().1;
    let tun = &files.iter().find(|(n, _)| n.contains("glare") && n.ends_with(".tun")).unwrap().1;
    let types = glare_types(&parse_glare(&String::from_utf8_lossy(tun)));
    let crp = Crp::parse(cdata).unwrap();
    let placed = placed_effects(&crp).unwrap();
    let lights: Vec<_> = placed.iter().filter(|p| p.article == "trafficlight" && p.e.kind == 2).collect();
    assert_eq!(lights.len(), 5);
    for p in &lights {
        assert!(p.scene_object);
        assert_eq!((p.e.glare_type(), p.e.flags, p.e.phase), (49, 0xc34, 0));
        assert_eq!((p.e.rate, p.e.base, p.e.amp), (f32::from_bits(0x3ca3d70a), 0.5, 1.0));
    }
    assert_eq!(types[49].colour[0] & 0xff_ffff, 0x00_0060, "red halo");
    assert_eq!(types[49].colour[1] & 0xff_ffff, 0x05_14c8, "orange-red spike");
    let mut lib = ParticleLib::default();
    let fx = TrackFx::new(&placed, |_| None, &mut lib);
    let i = fx.glares.iter().position(|g| g.item == lights[0].item && g.gtype == 49).unwrap();
    let g = fx.glares[i];
    let cam = [g.pos[0] - g.dir[0] * 5.0, g.pos[1], g.pos[2] - g.dir[2] * 5.0];
    let row2 = [-g.dir[0], -g.dir[1], -g.dir[2]];
    let on: Vec<bool> = (0..50u32).map(|t| fx.glares(t, cam, row2, 1.0, &types, |it| it == g.item).kept.iter().any(|(k, _)| *k == i)).collect();
    // Dark from tick 34 (past the duty) to the start of the next period, lit around the peak.
    assert!(on[25] && !on[34] && !on[49] && !on[0], "{on:?}");
    let f = fx.glares(25, cam, row2, 1.0, &types, |it| it == g.item);
    let s: Vec<_> = f.sprites.iter().filter(|s| s.rgba & 0xff_ffff == 0x60).collect();
    assert!(!s.is_empty());
}
