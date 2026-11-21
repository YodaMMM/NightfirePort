//! Disc test for `nf_formats::gal` (docs/formats/hud-gal.md): every `.gal` gallery in the driving
//! archives — `data\gfx\data\hudNTSC.gal` in the 7 mission archives + RACE.VIV (RefPack-compressed)
//! and the 4 plain galleries in MISC.VIV — decodes through `Gal::parse`, and the result matches
//! goldens written by an independent walker (a separate script over the same disc bytes, not this
//! parser): `golden/gal_census.tsv` (per gallery: bytes, SHA-1, sheets, animations, elements, keys)
//! and `golden/hud_gal_elements.tsv` (every HUD element's first `gk`#0x3000 key: x, y, w, h bits,
//! ARGB, sheet). Every embedded `tx` sheet parses as SHPS.
//!
//! Opt-in via NF_ISO (the full image; a slim image whose `/DRIVING/` extents are zero skips).

use nf_formats::gal::{Gal, GB, HUD_KEYSET};
use nf_formats::golden::diff_tsv;
use nf_formats::iso9660::IsoImage;
use nf_formats::sha1::{hex, sha1};

fn iso() -> Option<IsoImage> {
    let p = std::env::var_os("NF_ISO")?;
    IsoImage::open(p).ok()
}

/// (key `/DRIVING/<VIV>#<table index>`, entry name, decompressed bytes) for every `.gal` entry;
/// `None` on a slim image.
fn galleries(iso: &IsoImage) -> Option<Vec<(String, String, Vec<u8>)>> {
    let mut vivs: Vec<_> = iso.entries().iter()
        .filter(|e| e.path.starts_with("/DRIVING/") && e.path.ends_with(".VIV")).cloned().collect();
    vivs.sort_by(|a, b| a.path.cmp(&b.path));
    let mut out = Vec::new();
    for e in &vivs {
        let g = nf_formats::disc::read_bigf(iso, e).ok()?;
        for (i, x) in g.entries.iter().enumerate() {
            if !x.name.to_ascii_lowercase().ends_with(".gal") { continue; }
            let raw = iso.read_bytes(e.lba as u64 * 2048 + x.offset as u64, x.size as u64).ok()?;
            if raw.iter().take(64).all(|&b| b == 0) { return None; }
            let data = if nf_formats::refpack::is_refpack(&raw) { nf_formats::refpack::decompress(&raw).unwrap() } else { raw };
            out.push((format!("{}#{i}", e.path), x.name.clone(), data));
        }
    }
    Some(out)
}

fn census_row(key: &str, name: &str, data: &[u8], g: &Gal) -> String {
    let elements: usize = g.anims.iter().map(|a| a.elements.len()).sum();
    let keys: usize = g.anims.iter().flat_map(|a| &a.elements).flat_map(|e| &e.keysets).map(|(_, k)| k.len()).sum();
    format!("{key}\t{name}\t{}\t{}\t{}\t{}\t{elements}\t{keys}\n", data.len(), hex(&sha1(data)), g.sheets.len(), g.anims.len())
}

fn element_rows(g: &Gal) -> String {
    let mut s = String::from("element\tname\tkeys\tx_bits\ty_bits\tw_bits\th_bits\targb\tsheet\n");
    for a in &g.anims {
        for e in &a.elements {
            let id = format!("{}/{}#{}", a.name, e.kind_str(), e.index);
            let set = e.keysets.iter().find(|(k, _)| *k == HUD_KEYSET).map(|(_, k)| k.as_slice()).unwrap_or(&[]);
            let Some(k) = set.first() else {
                s += &format!("{id}\t{}\t0\t-\t-\t-\t-\t-\t-\n", e.name);
                continue;
            };
            let [x, y, _] = k.pos();
            if e.kind == GB {
                let [w, h] = k.size();
                s += &format!("{id}\t{}\t{}\t{:08x}\t{:08x}\t{:08x}\t{:08x}\t{:08x}\t{}\n", e.name, set.len(),
                    x.to_bits(), y.to_bits(), w.to_bits(), h.to_bits(), k.colour_argb(), k.sheet());
            } else {
                s += &format!("{id}\t{}\t{}\t{:08x}\t{:08x}\t-\t-\t-\t-\n", e.name, set.len(), x.to_bits(), y.to_bits());
            }
        }
    }
    s
}

#[test]
fn every_driving_gallery_matches_the_independent_census() {
    let Some(iso) = iso() else { eprintln!("skip: NF_ISO not set"); return };
    let Some(gals) = galleries(&iso) else { eprintln!("skip: slim image (DRIVING extents zeroed)"); return };
    assert_eq!(gals.len(), 12, "8 hudNTSC copies + 4 MISC.VIV galleries [D]");
    let mut actual = String::from("entry\tname\tbytes\tsha1\tsheets\tanims\telements\tkeys\n");
    for (key, name, data) in &gals {
        let g = Gal::parse(data).unwrap_or_else(|e| panic!("{key} {name}: {e}"));
        actual += &census_row(key, name, data, &g);
        for (i, _, r) in &g.sheets {
            nf_formats::ssh::Ssh::parse(&data[r.clone()]).unwrap_or_else(|e| panic!("{key} tx#{i}: {e}"));
        }
    }
    let d = diff_tsv(include_str!("../golden/gal_census.tsv"), &actual);
    assert!(d.is_empty(), "{} differences:\n{}", d.len(), d.join("\n"));
    // Sister: a one-element miscount on one row must be caught.
    let tampered = actual.replacen("\t196\t204\n", "\t195\t204\n", 1);
    assert!(!diff_tsv(include_str!("../golden/gal_census.tsv"), &tampered).is_empty());
}

#[test]
fn the_hud_gallery_elements_match_the_independent_walk() {
    let Some(iso) = iso() else { eprintln!("skip: NF_ISO not set"); return };
    let Some(gals) = galleries(&iso) else { eprintln!("skip: slim image (DRIVING extents zeroed)"); return };
    let (_, _, data) = gals.iter().find(|(k, _, _)| k.starts_with("/DRIVING/MIS01.VIV#")).expect("MIS01 hud gallery");
    let g = Gal::parse(data).unwrap();
    let actual = element_rows(&g);
    let golden = include_str!("../golden/hud_gal_elements.tsv");
    let d = diff_tsv(golden, &actual);
    assert!(d.is_empty(), "{} differences:\n{}", d.len(), d.join("\n"));
    // The elements the HUD init looks up by name exist where FUN_0023ae30 asks for them [M].
    let base = g.anim("hud_base").unwrap();
    for n in ["weapon", "ammo", "stylin", "a007", "healthBorder", "rechargeBar", "fbw1"] {
        assert!(base.element(n).is_some(), "hud_base/{n}");
    }
    for i in 1..=7 { assert!(base.element(&format!("iris{i}")).is_some() && base.element(&format!("arm{i}")).is_some()); }
    for i in 0..6 { assert!(base.element(&format!("gadget{i}")).is_some()); }
    assert!(g.anim("hud_msg").unwrap().element("txt").is_some());
    assert!(g.anim("pauseN").unwrap().element("scratchTxt").is_some());
    // Names the init asks for that the gallery does NOT carry (FUN_00236cc0 returns 0 for them) [D].
    for n in ["radar", "radar_border", "radar_stencil", "pulse"] { assert!(base.element(n).is_none(), "{n} is absent"); }
    // Sister: the element walk must see a moved key (one bit of ammo's x flipped).
    let tampered = actual.replacen("\tammo\t1\t44174000", "\tammo\t1\t44174001", 1);
    assert_ne!(tampered, actual, "the ammo row is present");
    assert!(!diff_tsv(golden, &tampered).is_empty());
}
