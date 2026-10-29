//! Scratch (level-06-every-objective): dump level 07000006's AI path records (map block 0x05):
//! every path/boundary record with its node positions and links. The guards' own walkable
//! connectivity — the F3 route evidence for the gallery → dressing-room leg.
use nf_formats::disc::files_bin_entries;
use nf_formats::iso9660::IsoImage;

fn main() {
    let path = std::env::var("NF_ISO").unwrap();
    let iso = IsoImage::open(&path).expect("iso");
    let (entries, bin) = files_bin_entries(&iso).expect("files.bin");
    let fe = entries
        .iter()
        .find(|e| e.name.eq_ignore_ascii_case("07000006.bin"))
        .expect("07000006.bin");
    let data = iso
        .read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64)
        .expect("read level");
    let lb = nf_formats::levelbin::LevelBin::parse(&data).expect("levelbin");
    for e in lb.entries.iter().filter(|e| matches!(e.kind, 0 | 1 | 2 | 11)) {
        let raw = lb.data(&data, e);
        let Ok(map) = nf_formats::map::MapFile::parse(raw) else { continue };
        for blk in map.all(0x05) {
            let net = nf_formats::aipath::AiNetwork::parse(blk.payload(raw)).expect("ai network");
            for (k, p) in net.paths.iter().enumerate() {
                println!("== record {k} kind {:#x} name {:?} nodes {} links {}", p.kind, p.name, p.nodes.len(), p.links.len());
                for (n, node) in p.nodes.iter().enumerate() {
                    println!("   node {n}: ({:.2}, {:.2}, {:.2})", node.pos[0], node.pos[1], node.pos[2]);
                }
                for l in &p.links {
                    println!("   link {} — {}", l.a, l.b);
                }
            }
        }
    }
}
