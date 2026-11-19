//! Scratch RE probe (dev-only): follow a track article's relocated `Model[0]` symbol chain from a
//! driving `.crp` to the vertex block, to crack the GeoPrim/Model layout against the article's
//! own bounding box. `cargo run --example crp_module_dump -- <iso> <VIV>`
//! (`WRITE_MODULE_O=<path>` also writes the reconstructed relocatable module).

use nf_formats::crp::Crp;
use nf_formats::crp_mesh::TrackModule;
use nf_formats::disc;
use nf_formats::iso9660::IsoImage;

fn main() -> anyhow::Result<()> {
    let mut args = std::env::args().skip(1);
    let iso_path = args.next().expect("usage: crp_module_dump <iso> <VIV-name>");
    let viv = args.next().expect("usage: crp_module_dump <iso> <VIV-name>");
    let iso = IsoImage::open(&iso_path)?;
    let e = iso
        .find(&format!("/DRIVING/{viv}.VIV"))
        .ok_or_else(|| anyhow::anyhow!("no /DRIVING/{viv}.VIV on the disc"))?;
    let g = disc::read_bigf(&iso, e)?;
    let crp_entry = g
        .entries
        .iter()
        .find(|x| x.name.to_lowercase().contains("data\\track\\") && x.name.ends_with(".crp"))
        .expect("no data\\track\\*.crp entry in the VIV");
    let raw = iso.read_bytes(e.lba as u64 * 2048 + crp_entry.offset as u64, crp_entry.size as u64)?;
    let streamed = nf_formats::refpack::decompress(&raw)?;
    let crp = Crp::parse(&streamed)?;
    let mut module = TrackModule::load(&crp)?;
    let moved = module.apply_relocations()?;
    println!("relocated words: {moved}");

    let articles: Vec<(String, [f32; 3], [f32; 3])> = crp
        .track_articles()?
        .into_iter()
        .map(|m| (m.name, m.bbox_min, m.bbox_max))
        .collect();
    let (name, min, max) = articles
        .iter()
        .find(|a| a.0 == "S6_CustomLowRBnknRoad")
        .map(|a| (a.0.clone(), a.1, a.2))
        .unwrap_or_else(|| {
            articles
                .iter()
                .max_by(|a, b| vol(a).total_cmp(&vol(b)))
                .map(|a| (a.0.clone(), a.1, a.2))
                .expect("no articles")
        });
    println!("article: {name}  bbox min {min:?} max {max:?}");

    let segs = module.model_segments(&name);
    println!("model segments: {segs:?}");
    for (seg, addr) in segs.iter().take(2) {
        dump_model(&module, *seg, *addr, &min, &max);
    }
    let strips = module.extract_article_mesh(&name, min, max);
    let total_v: usize = strips.iter().map(|s| s.positions.len()).sum();
    println!("extracted strips: {}, total vertices: {total_v}", strips.len());
    if let Some(s0) = strips.first() {
        println!("  strip[0]: {} verts, {} uvs, {} colors, tex={:?}, v0={:?}", s0.positions.len(), s0.uvs.len(), s0.colors.len(), s0.texture, s0.positions.first());
    }
    if std::env::var_os("ALL_ARTICLES").is_some() {
        let mut a_verts = 0usize;
        let mut a_strips = 0usize;
        for a in &articles {
            let st = module.extract_article_mesh(&a.0, a.1, a.2);
            a_strips += st.len();
            a_verts += st.iter().map(|s| s.positions.len()).sum::<usize>();
        }
        println!("ALL {} articles -> {} strips, {} vertices", articles.len(), a_strips, a_verts);
    }
    // Dev probe: write the reconstructed relocatable module for objdump/mipsdis work
    // (ELFd ++ ELFr restores the split .o exactly; relocations applied in .data are harmless).
    if let Some(path) = std::env::var_os("WRITE_MODULE_O") {
        std::fs::write(&path, &module.image)?;
        println!("module.o -> {}", path.to_string_lossy());
    }
    Ok(())
}

fn vol(a: &(String, [f32; 3], [f32; 3])) -> f32 {
    (0..3).map(|k| (a.2[k] - a.1[k]).abs()).product::<f32>()
}

/// Symbol value (combined-image address) → image offset, through the section that holds it:
/// `image_offset = section.data_offset + (value − section.offset)` [D ModuleSection docs].
fn addr_to_offset(module: &TrackModule, addr: u32) -> Option<usize> {
    let s = module
        .sections
        .iter()
        .find(|s| s.size > 0 && addr >= s.offset && addr < s.offset.saturating_add(s.size))?;
    let off = s.data_offset as u64 + (addr - s.offset) as u64;
    (off < module.image.len() as u64).then_some(off as usize)
}

fn dump_model(module: &TrackModule, seg: u8, addr: u32, min: &[f32; 3], max: &[f32; 3]) {
    let img = &module.image;
    let word = |off: usize| u32::from_le_bytes([img[off], img[off + 1], img[off + 2], img[off + 3]]);
    // The Model symbol is a 4-byte POINTER slot; follow the chain until the words look like
    // f32 vertex lanes inside the article's bbox (span > 1.5 — rejects the 0/1 descriptors).
    let mut cur = addr;
    for depth in 0..5 {
        let Some(off) = addr_to_offset(module, cur) else {
            println!("depth {depth}: {cur:#010x} unmapped");
            return;
        };
        let words: Vec<u32> = (0..16).map(|k| word(off + k * 4)).collect();
        let floats: Vec<f32> = words.iter().map(|x| f32::from_bits(*x)).collect();
        let vertexy = floats.iter().filter(|v| v.is_finite() && v.abs() > 1.5 && v.abs() < 60.0).count();
        println!(
            "depth {depth} @ {cur:#010x} (img +{off:#x}): {} vertex-like/16 — {:08x?}",
            vertexy,
            &words[..8.min(words.len())]
        );
        if vertexy >= 6 {
            let f: Vec<f32> = img[off..(off + 0x400).min(img.len())]
                .chunks_exact(4)
                .map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]]) as f32)
                .collect();
            for (i, chunk) in f.chunks(4).enumerate().take(24) {
                println!("  +{:03x}: {:>11.4} {:>11.4} {:>11.4} | {:>8.3}", i * 16, chunk[0], chunk[1], chunk[2], chunk[3]);
            }
            return;
        }
        cur = words[0];
    }
    println!("no vertex-like block found in the chain");
}
