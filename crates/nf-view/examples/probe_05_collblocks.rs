//! Scratch (level-05-basin-floor): the main map's raw block walk — every 0x2E collision block
//! with its tri count / root bbox and the object it attaches to under the port's
//! attach-to-next-object rule. Two 0x2E blocks before one ENTITY_PARAMS would make the port
//! keep only the LAST (level.rs `pending_coll = Some(..)` overwrites). The east-basin floor
//! (x 80..168, z 96..145, y ~5.1) exists in no parsed placement's collision, so look for it in
//! a dropped/overwritten block. Run: NF_ISO=<iso> cargo run -p nf-view --release --example probe_05_collblocks

use nf_formats::levelbin::LevelBin;
use nf_formats::map::MapFile;

fn main() {
    let Some(iso) = std::env::var("NF_ISO").ok() else { eprintln!("NF_ISO not set"); return };
    let img = nf_formats::iso9660::IsoImage::open(&iso).expect("open iso");
    let (entries, bin) = nf_formats::disc::files_bin_entries(&img).expect("FILES.BIN");
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case("07000005.bin")).expect("archive");
    let data = img.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).expect("read");
    let lb = LevelBin::parse(&data).expect("levelbin");
    for (ei, e) in lb.entries.iter().enumerate() {
        if !matches!(e.kind, 0 | 1 | 2 | 11) { continue; }
        let edata = lb.data(&data, e);
        let mf = MapFile::parse(edata).unwrap_or_else(|err| panic!("map {ei} parse: {err:?}"));
        if ei != 0x31 - 1 && ei != 50 { /* main map only — find by placement count */ }
        let mut pending: Option<(usize, usize, [f32; 3], [f32; 3])> = None; // (tris, version, lo, hi)
        let mut n2e = 0usize;
        println!("== entry {ei} kind {} blocks {} ==", e.kind, mf.blocks.len());
        for (bi, blk) in mf.blocks.iter().enumerate() {
            match blk.id {
                0x2e => {
                    n2e += 1;
                    let c = nf_formats::collision::Collision::parse(blk.payload(edata)).expect("coll");
                    let (mut lo, mut hi) = ([f32::MAX; 3], [f32::MIN; 3]);
                    for t in c.triangles() { for v in &t { for k in 0..3 { lo[k] = lo[k].min(v[k]); hi[k] = hi[k].max(v[k]); } } }
                    if pending.is_some() { println!("  !! block {bi} 0x2e OVERWRITES a pending block ({:?})", pending.as_ref().map(|p| p.0)); }
                    pending = Some((c.tris.len(), c.version as usize, lo, hi));
                    println!("  block {bi} id 0x2e size {} tris {} v{} bbox {:?}", blk.size, c.tris.len(), c.version, (lo, hi));
                }
                0x04 => { // ENTITY_PARAMS (id::ENTITY_PARAMS)
                    let p = nf_formats::buf::Buf(blk.payload(edata));
                    let tail = p.tail(0x30, "entity name").map(|t| t.bytes()).unwrap_or_default();
                    let name = String::from_utf8_lossy(&tail[..tail.iter().position(|&c| c == 0).unwrap_or(tail.len())]).into_owned();
                    let hash = p.u32(0).unwrap_or(0);
                    println!("  block {bi} id 0x04 object {hash:08x} \"{name}\" <- pending coll {:?}", pending.as_ref().map(|pc| pc.0));
                    pending = None;
                }
                _ => {}
            }
        }
        if pending.is_some() { println!("  !! entry {ei} ends with a PENDING collision block never attached"); }
        println!("  entry {ei}: {n2e} 0x2e blocks");
    }
}
