//! A carved-asset directory (`IsoImage::open` on a folder of extracted disc files) must read back exactly what
//! the disc holds, and no name inside it may reach outside the folder.
//!
//! The identity test carves every FILES.BIN archive plus files from every disc directory out of the real image
//! (under this crate's cargo temp dir, deleted afterwards) and compares every byte both ways. It skips when no
//! image is configured.

use nf_formats::disc;
use nf_formats::iso9660::{safe_rel_path, IsoImage, SECTOR_SIZE};
use std::path::{Path, PathBuf};

fn tmp(name: &str) -> PathBuf {
    let d = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

#[test]
fn safe_rel_path_keeps_names_inside() {
    assert_eq!(safe_rel_path("/PS2/MUSIC/MFX_00.VIV").unwrap(), Path::new("PS2").join("MUSIC").join("MFX_00.VIV"));
    assert_eq!(safe_rel_path("07000001.bin").unwrap(), Path::new("07000001.bin"));
    for bad in ["", "/", "..", "../x", "/../x", "a/../b", "a//b", "./a", "a/.", "C:/x", "C:x", "\\x", "a\\..\\b", "//srv/share", "a\0b"] {
        assert!(safe_rel_path(bad).is_err(), "{bad:?} must be refused");
    }
}

#[test]
fn carved_index_names_cannot_escape_the_folder() {
    let root = tmp("carved_escape");
    let inside = root.join("inside");
    std::fs::create_dir_all(inside.join("FILES")).unwrap();
    std::fs::write(root.join("outside.bin"), vec![0xAB; 4096]).unwrap();
    std::fs::write(inside.join("FILES").join("ok.bin"), vec![0x11; 2048]).unwrap();

    // A plain index opens and maps the named file.
    std::fs::write(inside.join("FILES_INDEX.tsv"), "ok.bin\t0\t2048\n").unwrap();
    let iso = IsoImage::open(&inside).unwrap();
    let bin = iso.find("/FILES.BIN").unwrap();
    assert_eq!(iso.read_bytes(bin.lba as u64 * SECTOR_SIZE as u64, 2048).unwrap(), vec![0x11; 2048]);

    for name in ["../outside.bin", "..\\outside.bin", "FILES/../../outside.bin", "/outside.bin"] {
        std::fs::write(inside.join("FILES_INDEX.tsv"), format!("{name}\t0\t4096\n")).unwrap();
        assert!(IsoImage::open(&inside).is_err(), "index name {name:?} must be refused");
    }
    let abs = root.join("outside.bin");
    std::fs::write(inside.join("FILES_INDEX.tsv"), format!("{}\t0\t4096\n", abs.display())).unwrap();
    assert!(IsoImage::open(&inside).is_err(), "an absolute index name must be refused");
    std::fs::remove_dir_all(&root).unwrap();
}

#[test]
fn carved_folder_reads_the_same_bytes_as_the_image() {
    let Ok(iso_path) = std::env::var("NF_ISO") else { eprintln!("NF_ISO unset: skipped"); return };
    let iso = IsoImage::open(&iso_path).unwrap();
    let out = tmp("carved_identity");
    let files_dir = out.join("FILES");
    std::fs::create_dir_all(&files_dir).unwrap();

    // Every FILES.BIN archive, through the index the carver writes.
    let (list, bin) = disc::files_bin_entries(&iso).unwrap();
    let base = bin.lba as u64 * SECTOR_SIZE as u64;
    let mut index = String::from("# name\toffset\tsize\n");
    for e in &list {
        index.push_str(&format!("{}\t{}\t{}\n", e.name, e.offset, e.size));
        std::fs::write(files_dir.join(safe_rel_path(&e.name).unwrap()), iso.read_bytes(base + e.offset as u64, e.size as u64).unwrap()).unwrap();
    }
    std::fs::write(out.join("FILES_INDEX.tsv"), index).unwrap();

    // Disc-tree files: the first two of every directory (and the root), plus ACTION.ELF.
    let mut per_dir: std::collections::BTreeMap<String, usize> = Default::default();
    let mut tree = Vec::new();
    for e in iso.entries().iter().filter(|e| !e.is_directory && e.path != "/FILES.BIN") {
        let dir = e.path.rsplit_once('/').map_or("", |(d, _)| d).to_string();
        let n = per_dir.entry(dir).or_default();
        if *n < 2 || e.path == "/ACTION.ELF" {
            *n += 1;
            let dst = out.join(safe_rel_path(&e.path).unwrap());
            std::fs::create_dir_all(dst.parent().unwrap()).unwrap();
            std::fs::write(&dst, iso.read_file(e).unwrap()).unwrap();
            tree.push(e.clone());
        }
    }
    let dirs = iso.entries().iter().filter(|e| e.is_directory).count();
    assert!(per_dir.len() > dirs / 2, "sample spans the disc tree ({} of {dirs} directories hold files)", per_dir.len());

    let carved = IsoImage::open(&out).unwrap();
    // Tree files: same names, sizes and bytes.
    for e in &tree {
        let c = carved.find(&e.path).unwrap_or_else(|| panic!("{} missing from the carved folder", e.path));
        assert_eq!(c.size, e.size, "{}", e.path);
        assert!(carved.read_file(c).unwrap() == iso.read_file(e).unwrap(), "{} differs", e.path);
    }
    // FILES.BIN: the carved image yields the same FileList (read from its ACTION.ELF) and the same archive bytes,
    // whole and at unaligned sub-ranges.
    let (clist, cbin) = disc::files_bin_entries(&carved).unwrap();
    assert_eq!(clist.len(), list.len());
    let cbase = cbin.lba as u64 * SECTOR_SIZE as u64;
    let mut bytes = 0u64;
    for (a, b) in list.iter().zip(&clist) {
        assert_eq!((&a.name, a.offset, a.size), (&b.name, b.offset, b.size));
        let want = iso.read_bytes(base + a.offset as u64, a.size as u64).unwrap();
        assert!(carved.read_bytes(cbase + b.offset as u64, b.size as u64).unwrap() == want, "{} differs", a.name);
        for (off, len) in [(1u64, 7u64), (2047, 2), (a.size as u64 / 2 + 3, 4099)] {
            if off + len <= a.size as u64 {
                let got = carved.read_bytes(cbase + b.offset as u64 + off, len).unwrap();
                assert_eq!(got, want[off as usize..(off + len) as usize], "{} @{off}+{len}", a.name);
            }
        }
        bytes += a.size as u64;
    }
    eprintln!("compared {} archives ({} MB) and {} tree files", list.len(), bytes >> 20, tree.len());
    drop(carved);
    std::fs::remove_dir_all(&out).unwrap();
}
