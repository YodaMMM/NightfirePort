//! Disc test for `nf_formats::trackfx` (docs/systems/driving-effects.md §7): every effect entry
//! placed on the 8 driving tracks, its decoded kind, and the Giotto effect each particle entry
//! names, resolved in the gallery the game loads for that track.
//!
//! Opt-in via NF_ISO (the full image; a slim image without the `/DRIVING/` extents skips).

use std::collections::BTreeMap;

use nf_formats::crp::Crp;
use nf_formats::drivefx::parse_glare;
use nf_formats::gal::Gal;
use nf_formats::iso9660::IsoImage;
use nf_formats::trackfx::*;

const TRACKS: &[&str] = &["MIS01", "MIS11", "MIS13A", "MIS13B", "MIS13C", "MIS3", "MIS4", "RACE"];

fn iso() -> Option<IsoImage> {
    let Ok(path) = std::env::var("NF_ISO") else {
        eprintln!("NF_ISO unset: skipped");
        return None;
    };
    Some(IsoImage::open(&path).expect("open NF_ISO"))
}

/// The VIV's files: (lower-case name, decompressed bytes); None on a slim image.
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

fn file<'a>(files: &'a [(String, Vec<u8>)], pred: impl Fn(&str) -> bool) -> &'a [u8] {
    &files.iter().find(|(n, _)| pred(n)).expect("file").1
}

/// Per track: (static glare, static particle at load, static particle not started, static other,
/// scene glare, scene light, scene particle, scene other) [D measured].
/// Then the Giotto names the track uses that its gallery lacks: the game logs "Could not locate
/// Giotto effect" and that effect does not appear (`FUN_00232e18`, string 0x39bf20) [M + D].
const COUNTS: &[(&str, [usize; 8], &[&str])] = &[
    ("MIS01", [351, 10, 1, 0, 252, 0, 708, 0], &[]),
    ("MIS11", [26, 0, 0, 0, 208, 0, 76, 0], &["Un_SandR"]),
    ("MIS13A", [23, 13, 0, 0, 84, 0, 90, 0], &["Wd_steambig", "dummy"]),
    ("MIS13B", [12, 16, 6, 0, 85, 0, 171, 0], &["Ex_Oil", "dummy"]),
    ("MIS13C", [0, 9, 0, 0, 49, 0, 115, 0], &["Ex_Oil", "snowbridgeexplosion"]),
    ("MIS3", [79, 2, 0, 0, 16, 0, 158, 0], &["snowlandingdeep"]),
    ("MIS4", [16, 0, 0, 0, 86, 0, 121, 0], &["sm_splinter"]),
    ("RACE", [16, 0, 0, 0, 84, 0, 121, 0], &["Sm_Fruit", "sm_splinter"]),
];

#[test]
fn every_placed_effect_decodes_and_every_giotto_name_resolves() {
    let Some(iso) = iso() else { return };
    let Some(misc) = viv_files(&iso, "MISC") else { return };
    let gal = |n: &str| Gal::parse(file(&misc, |x| x.ends_with(n) && x.contains("gfx"))).unwrap();
    let mut got = Vec::new();
    for &viv in TRACKS {
        let Some(files) = viv_files(&iso, viv) else { return };
        let (cname, cdata) = files.iter().find(|(n, _)| n.starts_with("data\\track\\") && n.ends_with(".crp")).unwrap();
        let crp = Crp::parse(cdata).unwrap();
        // FUN_0024b7d0 compares the track file's base name (`snow2a_race`, `junglea_mis13a`).
        let track = cname.trim_start_matches("data\\track\\").trim_end_matches(".crp").to_owned();
        let g = gal(giotto_gallery(&track));
        let glare = parse_glare(&String::from_utf8_lossy(file(&files, |n| n.contains("glare") && n.ends_with(".tun"))));
        let fx = placed_effects(&crp).unwrap();
        let mut n = [0usize; 8];
        let mut list: BTreeMap<String, usize> = BTreeMap::new();
        let mut unresolved = Vec::new();
        for p in &fx {
            let slot = match (p.scene_object, p.e.kind) {
                (false, kind::GLARE) => 0,
                (false, kind::GIOTTO) if p.starts_at_load() => 1,
                (false, kind::GIOTTO) => 2,
                (false, _) => 3,
                (true, kind::GLARE) => 4,
                (true, kind::LIGHT) => 5,
                (true, kind::GIOTTO) => 6,
                (true, _) => 7,
            };
            n[slot] += 1;
            let what = match p.e.kind {
                kind::GLARE => {
                    assert!((p.e.glare_type() as usize) < glare.len(), "{viv} {}: glare type {}", p.article, p.e.glare_type());
                    let t = glare[p.e.glare_type() as usize];
                    format!("glare type {} flags {:#x} halo {:?} spike {:?} pulse {}/{}/{}/{}", p.e.glare_type(), p.e.flags, t.col[0], t.col[1], p.e.rate, p.e.duty, p.e.base, p.e.amp)
                }
                kind::GIOTTO => {
                    let nm = p.giotto_name().unwrap_or("?");
                    match giotto_effect(&g, nm) {
                        Some(e) => format!("giotto {nm} flags {:#x} life {} gp {:?} other {:?}", e.flags, e.life, e.emitters.iter().map(|x| x.element.as_str()).collect::<Vec<_>>(), e.others),
                        None => {
                            unresolved.push(nm.to_owned());
                            format!("giotto {nm} UNRESOLVED")
                        }
                    }
                }
                k => format!("kind {k} flags {:#x}", p.e.flags),
            };
            *list.entry(format!("{} part {} entry {} {} {what}", p.article, p.part, p.entry, if p.scene_object { "scene" } else { "static" })).or_default() += 1;
        }
        for (k, c) in &list {
            eprintln!("{viv} [{track} / {}] x{c}: {k}", giotto_gallery(&track));
        }
        unresolved.sort();
        unresolved.dedup();
        eprintln!("{viv}: counts {n:?}, unresolved Giotto names {unresolved:?}");
        got.push((viv, n, unresolved));
    }
    for (viv, n, unresolved) in &got {
        let (_, want, missing) = COUNTS.iter().find(|(t, _, _)| t == viv).unwrap();
        assert_eq!(n, want, "{viv}");
        assert_eq!(unresolved, missing, "{viv}");
    }
}
