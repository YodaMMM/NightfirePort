//! Dev tool: carve a minimal non-ISO asset directory (front-end + mission 1 / tutorials) for Android.
//! Usage: `cargo run --release -p nf-formats --example carve_android -- <iso> <out-dir> [--all]`
//! Writes only under `<out-dir>`: every output name goes through `safe_rel_path`.

use nf_formats::{disc, iso9660::{safe_rel_path, IsoImage}};
use std::path::Path;

fn main() -> anyhow::Result<()> {
    let mut args = std::env::args().skip(1);
    let iso_path = args.next().expect("usage: carve_android <iso> <out-dir> [--all]");
    let out_dir = args.next().expect("usage: carve_android <iso> <out-dir> [--all]");
    let carve_all = args.any(|a| a == "--all");
    let carve_movies = carve_all || std::env::args().any(|a| a == "--movies");
    let out = Path::new(&out_dir);
    let files_dir = out.join("FILES");
    std::fs::create_dir_all(&files_dir)?;

    let iso = IsoImage::open(&iso_path)?;

    // 1. Copy ACTION.ELF
    let action_entry = iso.find("/ACTION.ELF").expect("no /ACTION.ELF");
    let action_bytes = iso.read_file(action_entry)?;
    std::fs::write(out.join("ACTION.ELF"), &action_bytes)?;

    // 2. Read FileList and write FILES_INDEX.tsv + selected FILES.BIN entries
    let (list, files_bin) = disc::files_bin_entries(&iso)?;
    let files_base = files_bin.lba as u64 * 2048;

    let mut index_tsv = String::from("# name\toffset\tsize\n");
    let mut carved_bytes = 0u64;
    let mut carved_count = 0usize;

    for e in &list {
        index_tsv.push_str(&format!("{}\t{}\t{}\n", e.name, e.offset, e.size));
        let lower = e.name.to_ascii_lowercase();
        let is_level = lower.starts_with("07") && lower.ends_with(".bin");
        let keep_level = carve_all
            || lower.ends_with("00001.bin")
            || lower.ends_with("00002.bin")
            || lower == "07000048.bin";
        if !is_level || keep_level {
            let data = iso.read_bytes(files_base + e.offset as u64, e.size as u64)?;
            std::fs::write(files_dir.join(safe_rel_path(&e.name)?), &data)?;
            carved_bytes += e.size as u64;
            carved_count += 1;
            println!("carved FILES/{} ({:.2} MB)", e.name, e.size as f64 / (1024.0 * 1024.0));
        }
    }
    std::fs::write(out.join("FILES_INDEX.tsv"), index_tsv)?;

    // 3. Copy soundbank/music tables (SBINFO.SBI, MFXINFO.MXI), SFX banks, streams, music, and driving archives
    for entry in iso.entries() {
        if entry.is_directory {
            continue;
        }
        let p = &entry.path;
        let upper = p.to_ascii_uppercase();
        let keep = upper.ends_with("/SBINFO.SBI")
            || upper.ends_with("/MFXINFO.MXI")
            || upper.starts_with("/PS2/ENGLISH/SB_")
            || upper.starts_with("/PS2/MUSIC/MFX_00")
            || upper.starts_with("/PS2/MUSIC/MFX_01")
            || upper.starts_with("/PS2/MUSIC/MFX_02")
            || (carve_all && upper.starts_with("/PS2/MUSIC/"))
            || (carve_all && upper.starts_with("/PS2/ENGLISH/STREAMS/"))
            || (carve_movies && upper.starts_with("/MOVIES/") && upper.ends_with(".PSS"))
            || (carve_all
                && upper.starts_with("/DRIVING/")
                && (upper.ends_with(".VIV") || upper.ends_with(".MUS") || upper.ends_with(".SPE")));
        if keep {
            let dst = out.join(safe_rel_path(p)?);
            if let Some(parent) = dst.parent() {
                std::fs::create_dir_all(parent)?;
            }
            let data = iso.read_file(entry)?;
            std::fs::write(&dst, &data)?;
            carved_bytes += entry.size as u64;
            carved_count += 1;
        }
    }

    println!(
        "done: {carved_count} files, total {:.2} MB in {}",
        (carved_bytes + action_bytes.len() as u64) as f64 / (1024.0 * 1024.0),
        out.display()
    );
    Ok(())
}
