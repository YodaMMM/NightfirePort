//! Disc test for `nf_formats::rendertun`: every driving track's `Render\Fog` and
//! `Render\Lighting` tuning file parses; the fog values are pinned [D].
//!
//! Opt-in via NF_ISO (the full image; a slim image without the `/DRIVING/` extents skips).

use nf_formats::iso9660::IsoImage;
use nf_formats::rendertun::{Fog, Lighting};

/// (archive, fog start, end, density, FOGCOL RGB, World ambient sky RGB, lightmaps on) [D].
const FOG: &[(&str, f32, f32, f32, [u8; 3], [f32; 3], bool)] = &[
    ("MIS01", 1.0, 1450.21, 0.000475, [22, 15, 20], [0.72, 0.72, 0.72], false),
    ("MIS11", 1.0, 250.95, 1.0, [32, 66, 130], [0.5, 0.6, 0.56], false),
    ("MIS13A", 1.0, 400.42, 1.0, [103, 119, 151], [0.72, 0.72, 0.72], true),
    ("MIS13B", 1.0, 2000.1, 1.0, [153, 129, 101], [0.64, 0.74, 0.84], false),
    ("MIS13C", 1.0, 1200.26, 1.0, [158, 144, 106], [0.72, 0.72, 0.72], false),
    ("MIS3", 1.0, 250.949905, 1.0, [40, 45, 50], [0.64, 0.74, 0.84], true),
    ("MIS4", 1.0, 550.890015, 1.0, [168, 144, 150], [0.68, 0.72, 0.76], false),
    ("RACE", 1.0, 550.890015, 1.0, [168, 144, 150], [0.68, 0.72, 0.76], false),
];

fn tun(iso: &IsoImage, viv: &str, dir: &str) -> Option<String> {
    let e = iso.find(&format!("/DRIVING/{viv}.VIV"))?;
    let g = nf_formats::disc::read_bigf(iso, e).ok()?;
    let want = format!("data\\tuning\\render\\{dir}\\");
    let t = g.entries.iter().find(|x| x.name.to_ascii_lowercase().starts_with(&want))?;
    let raw = iso.read_bytes(e.lba as u64 * 2048 + t.offset as u64, t.size as u64).ok()?;
    if raw.iter().take(16).all(|&b| b == 0) {
        return None;
    }
    let dec = if nf_formats::refpack::is_refpack(&raw) { nf_formats::refpack::decompress(&raw).unwrap() } else { raw };
    Some(String::from_utf8_lossy(&dec).into_owned())
}

#[test]
fn every_driving_tracks_fog_and_lighting_tuning_parses() {
    let Ok(path) = std::env::var("NF_ISO") else { eprintln!("skip: NF_ISO not set"); return };
    let iso = IsoImage::open(&path).expect("open NF_ISO");
    if tun(&iso, "MIS01", "fog").is_none() {
        eprintln!("skip: no /DRIVING/ extents in this image");
        return;
    }
    for &(viv, start, end, density, colour, sky, lightmaps) in FOG {
        let f = Fog::parse(&tun(&iso, viv, "fog").unwrap()).unwrap();
        let l = Lighting::parse(&tun(&iso, viv, "lighting").unwrap()).unwrap();
        eprintln!("{viv}: fog {}..{} density {} colour {:?} slope {} | world sky {:?} ground {:?} light1 {:?} light2 {:?} lightmaps {}",
            f.start, f.end, f.density, f.colour, f.slope(), l.world.ambient_sky, l.world.ambient_ground, l.world.light1, l.world.light2, l.enable_lightmaps);
        assert_eq!((f.start, f.end, f.density, f.colour, f.mode), (start, end, density, colour, 3), "{viv} fog");
        assert_eq!(([l.world.ambient_sky[0], l.world.ambient_sky[1], l.world.ambient_sky[2]], l.enable_lightmaps), (sky, lightmaps), "{viv} lighting");
    }
}
