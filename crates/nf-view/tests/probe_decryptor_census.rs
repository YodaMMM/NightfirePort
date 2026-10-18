//! Probe (scratch): dump every 0x2E placement disc-wide with its params — the lock census for the
//! decryptor task. Not a pin: prints to stdout, asserts nothing.

use std::fs::File;
use std::io::BufReader;

#[test]
fn dump_lock_placements() {
    let iso_path = std::env::var("NF_ISO").expect("NF_ISO");
    let f = File::open(&iso_path).expect("open iso");
    let iso = nf_formats::iso9660::IsoImage::new(BufReader::new(f)).expect("iso");
    let mut out = String::from("archive\tentry\tmap_kind\tidx\tfx\tparams\n");
    let _ = nf_formats::disc::for_each_map(&iso, |archive, i, e, map| {
        let mf = match nf_formats::map::MapFile::parse(map) { Ok(m) => m, Err(_) => return Ok(()) };
        for blk in &mf.blocks {
            if blk.id != nf_formats::map::id::MAP_DATA_STATIC { continue; }
            let placements = match nf_formats::placement::parse(blk.payload(map)) { Ok(p) => p, Err(_) => continue };
            for (k, p) in placements.iter().enumerate() {
                if p.flags & 0xff != 0x2e { continue; }
                let params: Vec<String> = p.params.iter().map(|(j, v)| format!("{j}={v}")).collect();
                out.push_str(&format!("{}\t{}\t{}\t{}\t{:x}\t{}\n",
                    archive.trim_end_matches(".bin"), i, e.kind, k, p.flags >> 8,
                    params.join(",")));
            }
        }
        Ok(())
    });
    print!("{out}");
    std::fs::write("/home/z/my-project/locks_census.tsv", &out).unwrap();
}
