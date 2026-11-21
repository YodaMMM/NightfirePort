//! Disc proof for the EA `SHPS` (`.ssh`) decoder (`nf_formats::ssh`).
//!
//! Every `.ssh` on the disc lives inside the driving BIGF archives (`/DRIVING/*.VIV`, most entries
//! RefPack-compressed; docs/formats/driving-viv.md). No loose ISO file starts with `SHPS` [D].
//! The test finds them all by magic (and checks the magic agrees with the `.ssh` name), decodes
//! every texture, and checks:
//! - no decode error, sizes in 1..=1024, `rgba` length = w*h*4;
//! - the CSM1 palette order: the decoded 8-bit images are far smoother than the same indices read
//!   through the palette in file order, for full (256) and partial palettes alike;
//! - the flag-0x2000 unswizzle: the GS-emulated index image is smoother than the raw bytes read
//!   linearly.
//!
//! Skips cleanly when NF_ISO is unset or the image carries no driving archive bytes.

use nf_formats::iso9660::IsoImage;
use nf_formats::ssh::{csm1_index, Ssh};
use nf_formats::{disc, refpack};
use std::collections::{BTreeMap, HashSet};

struct Sheet {
    archive: String,
    name: String,
    data: Vec<u8>,
}

fn sheets() -> Option<Vec<Sheet>> {
    let path = std::env::var_os("NF_ISO")?;
    let iso = IsoImage::open(&path).unwrap_or_else(|e| panic!("NF_ISO={path:?}: {e}"));
    let mut out = Vec::new();
    for e in iso.entries().iter().filter(|e| !e.is_directory && e.size >= 4) {
        let head = iso.read_bytes(e.lba as u64 * 2048, 4).unwrap();
        assert_ne!(&head, b"SHPS", "loose SHPS file {} (not covered by this test)", e.path);
        if !e.path.starts_with("/DRIVING/") || e.path.ends_with(".SYM") || &head != b"BIGF" {
            continue;
        }
        let g = disc::read_bigf(&iso, e).unwrap();
        let blob = iso.read_file(e).unwrap();
        for be in &g.entries {
            let raw = &blob[be.offset as usize..(be.offset + be.size) as usize];
            let data = if refpack::is_refpack(raw) { refpack::decompress(raw).unwrap() } else { raw.to_vec() };
            let by_name = be.name.to_ascii_lowercase().ends_with(".ssh");
            let by_magic = data.starts_with(b"SHPS");
            assert_eq!(by_name, by_magic, "{} {}: name/magic disagree", e.path, be.name);
            if by_magic {
                out.push(Sheet { archive: e.path.clone(), name: be.name.clone(), data });
            }
        }
    }
    if out.is_empty() {
        eprintln!("skip: no driving archive bytes in NF_ISO");
        return None;
    }
    Some(out)
}

fn colour_roughness(px: &[[u8; 4]], w: usize, h: usize) -> u64 {
    let d = |a: [u8; 4], b: [u8; 4]| (0..3).map(|c| (a[c] as i32 - b[c] as i32).unsigned_abs() as u64).sum::<u64>();
    let mut s = 0u64;
    for y in 0..h {
        for x in 0..w {
            if x + 1 < w { s += d(px[y * w + x], px[y * w + x + 1]); }
            if y + 1 < h { s += d(px[y * w + x], px[(y + 1) * w + x]); }
        }
    }
    s
}

fn roughness(img: &[u8], w: usize, h: usize) -> u64 {
    let mut s = 0u64;
    for y in 0..h {
        for x in 0..w {
            let v = img[y * w + x] as i32;
            if x + 1 < w { s += (v - img[y * w + x + 1] as i32).unsigned_abs() as u64; }
            if y + 1 < h { s += (v - img[(y + 1) * w + x] as i32).unsigned_abs() as u64; }
        }
    }
    s
}

#[test]
fn every_disc_ssh_decodes() {
    let Some(sheets) = sheets() else { eprintln!("skip: NF_ISO not set"); return };
    let mut per_archive: BTreeMap<&str, u32> = BTreeMap::new();
    let mut per_format: BTreeMap<String, u32> = BTreeMap::new();
    let mut unique = HashSet::new();
    let mut textures = 0u32;
    // [full, partial] palettes: (decoded roughness, file-order roughness, decoded smoother count, total)
    let mut csm = [(0u64, 0u64, 0u32, 0u32); 2];
    let mut swizzled = Vec::new();
    for s in &sheets {
        *per_archive.entry(&s.archive).or_default() += 1;
        let ssh = Ssh::parse(&s.data).unwrap_or_else(|e| panic!("{} {}: {e}", s.archive, s.name));
        assert!(!ssh.textures.is_empty(), "{} {}", s.archive, s.name);
        if !unique.insert(nf_formats::sha1::sha1(&s.data)) { continue; }
        for t in &ssh.textures {
            textures += 1;
            assert!((1..=1024).contains(&t.width) && (1..=1024).contains(&t.height), "{} {} {}: {}x{}", s.archive, s.name, t.name, t.width, t.height);
            assert_eq!(t.rgba.len(), (t.width * t.height * 4) as usize, "{} {}", s.name, t.name);
            let key = format!("type {:#04x} ({}-bit){}{}", t.format, t.bpp,
                t.clut.map(|c| format!(" clut {c:#04x}")).unwrap_or_default(),
                if t.swizzled { " swizzled" } else { "" });
            *per_format.entry(key).or_default() += 1;
            if t.bpp == 8 {
                let (w, h) = (t.width as usize, t.height as usize);
                let (idx, pal) = (t.indices.as_ref().unwrap(), t.palette.as_ref().unwrap());
                let decoded: Vec<[u8; 4]> = t.rgba.chunks_exact(4).map(|c| c.try_into().unwrap()).collect();
                let file_order: Vec<[u8; 4]> = idx.iter().map(|&i| pal[csm1_index(i as usize)]).collect();
                let (a, b) = (colour_roughness(&decoded, w, h), colour_roughness(&file_order, w, h));
                let k = &mut csm[(t.clut_width != Some(256)) as usize];
                k.0 += a;
                k.1 += b;
                k.2 += (a < b) as u32;
                k.3 += 1;
                if t.swizzled && t.width >= 32 { swizzled.push((s.name.clone(), t.clone())); }
            }
        }
    }
    println!("ssh files on disc: {} ({} unique), textures in unique sheets: {textures}", sheets.len(), unique.len());
    for (a, n) in &per_archive { println!("  {a}: {n} sheets"); }
    for (k, n) in &per_format { println!("  {n:5} {k}"); }
    assert!(sheets.len() >= 200, "expected the driving archives' 218 sheets, found {}", sheets.len());
    for (label, (a, b, won, n)) in ["256-entry", "partial"].iter().zip(csm) {
        println!("  {label} palettes: roughness CSM1 {a} vs file order {b}; CSM1 smoother in {won}/{n}");
        assert!(n > 0 && a * 2 < b && won * 10 > n * 6, "{label} palettes: CSM1 order is not the smoother reading");
    }

    // Flag 0x2000: the GS emulation must give a smoother index image than the raw linear bytes.
    assert!(!swizzled.is_empty(), "expected swizzled 8-bit textures (ext.ssh, blood.ssh)");
    for (sheet, t) in &swizzled {
        let s = sheets.iter().find(|s| &s.name == sheet).unwrap();
        let ssh = Ssh::parse(&s.data).unwrap();
        let i = ssh.textures.iter().position(|x| x.name == t.name).unwrap();
        let off = u32::from_le_bytes(s.data[16 + i * 8 + 4..16 + i * 8 + 8].try_into().unwrap()) as usize;
        assert_eq!(u32::from_le_bytes(s.data[off + 12..off + 16].try_into().unwrap()) & 0x1000, 0);
        let (w, h) = (t.width as usize, t.height as usize);
        let linear = &s.data[off + 16..off + 16 + w * h];
        let pal = t.palette.as_ref().unwrap();
        let decoded: Vec<[u8; 4]> = t.rgba.chunks_exact(4).map(|c| c.try_into().unwrap()).collect();
        let lin_rgba: Vec<[u8; 4]> = linear.iter().map(|&i| pal[i as usize]).collect();
        let (r_gs, r_lin) = (roughness(t.indices.as_ref().unwrap(), w, h), roughness(linear, w, h));
        let (c_gs, c_lin) = (colour_roughness(&decoded, w, h), colour_roughness(&lin_rgba, w, h));
        println!("  swizzled {sheet} {} {w}x{h}: index roughness {r_gs} vs linear {r_lin}; colour {c_gs} vs {c_lin}", t.name);
        assert!(r_gs * 3 < r_lin * 2 && c_gs * 3 < c_lin * 2, "{sheet} {}: unswizzle not smoother", t.name);
    }
}
