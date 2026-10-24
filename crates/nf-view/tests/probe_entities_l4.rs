//! level-4a-lift-4 probe: the entity streams of 07000001 and 0700004a's scripts — hash,
//! obj_type, parent, first key — to settle the hand-back marker class discriminator.
//! Assertion-free diagnostic, run with --ignored --nocapture.

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

use nf_formats::iso9660::IsoImage;
use nf_formats::levelbin::LevelBin;
use nf_formats::nis::{Cmd, Nis};

fn read_bytes(iso: &str, archive: &str) -> Vec<u8> {
    let img = IsoImage::open(iso).expect("open iso");
    let (entries, bin) = nf_formats::disc::files_bin_entries(&img).expect("FILES.BIN");
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case(archive)).expect("archive");
    img.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).expect("read")
}

fn dump_pack(iso: &str, archive: &str) {
    let data = read_bytes(iso, archive);
    let lb = LevelBin::parse(&data).expect("parse");
    for e in lb.entries.iter() {
        if e.kind != 7 { continue; }
        let Ok(nis) = Nis::parse(e.hash(), lb.data(&data, e)) else { continue };
        for (si, st) in nis.streams.iter().enumerate() {
            for c in st.decode() {
                if let Cmd::Entity { hash, parent, obj_type, .. } = &c {
                    let k0 = nis.keys.get(st.first_key as usize);
                    let pos = k0.map(|k| format!("({:.3},{:.3},{:.3})", k.pos[0], k.pos[1], k.pos[2])).unwrap_or("-".into());
                    println!("{archive} script {:08x} s{si} Entity hash {hash:08x} parent {parent:?} kind {obj_type:#04x} key0 {pos}", nis.hashcode);
                }
            }
        }
    }
}

fn dump(archive: &str) {
    let iso = iso();
    let data = read_bytes(&iso, archive);
    let lb = LevelBin::parse(&data).expect("parse");
    let mut n = 0;
    for e in lb.entries.iter() {
        if e.kind != 7 { continue; }
        let Ok(nis) = Nis::parse(e.hash(), lb.data(&data, e)) else { continue };
        n += 1;
        let mut ent = 0;
        for (si, st) in nis.streams.iter().enumerate() {
            for c in st.decode() {
                if let Cmd::Entity { hash, parent, obj_type, .. } = &c {
                    let k0 = nis.keys.get(st.first_key as usize);
                    let pos = k0.map(|k| format!("({:.3},{:.3},{:.3})", k.pos[0], k.pos[1], k.pos[2])).unwrap_or("-".into());
                    println!("{archive} script {:08x} s{si} Entity hash {hash:08x} parent {parent:?} kind {obj_type:#04x} key0 {pos}", nis.hashcode);
                    ent += 1;
                }
            }
        }
        let _ = ent;
    }
    println!("{archive}: {n} scripts");
}

#[test]
#[ignore = "assertion-free dump; run with --ignored"]
fn dump_01_and_4a_entities() {
    dump("07000001.bin");
    dump("0700004a.bin");
    for p in ["07800001.bin", "07900001.bin", "07a00001.bin", "07b00001.bin", "07c00001.bin", "07d00001.bin", "07e00001.bin", "07f00001.bin"] {
        if std::path::Path::new(&p).exists() { continue; }
        // packs live inside the ISO's FILES.BIN, not the fs; errors are fine (missing archive).
        dump_pack(&iso(), p);
    }
}
