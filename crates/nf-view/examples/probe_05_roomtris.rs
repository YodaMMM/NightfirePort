//! Scratch (level-05-basin-floor): are the room collision blocks fully consumed by the parse?
//! For the main map's two room blocks (Room_Road_A1 / Room_Road_B) and every block: compare the
//! block byte size against 0x1C + nvb*0x40 + nbox*0x30 + ntri*(8+1) — a remainder means the port
//! read only part of the block. Also dumps Room_Road_A1's up-facing near-horizontal tris inside
//! the east band (x 82..168, z 96..146) by y — is the east-basin floor in the room's OWN block?
//! Run: NF_ISO=<iso> cargo run -p nf-view --release --example probe_05_roomtris

use nf_formats::levelbin::LevelBin;
use nf_formats::map::MapFile;

fn main() {
    let Some(iso) = std::env::var("NF_ISO").ok() else { eprintln!("NF_ISO not set"); return };
    let img = nf_formats::iso9660::IsoImage::open(&iso).expect("open iso");
    let (entries, bin) = nf_formats::disc::files_bin_entries(&img).expect("FILES.BIN");
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case("07000005.bin")).expect("archive");
    let data = img.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).expect("read");
    let lb = LevelBin::parse(&data).expect("levelbin");
    let last = lb.entries.iter().rfind(|e| e.kind == 1).expect("main map");
    let edata = lb.data(&data, last);
    let mf = MapFile::parse(edata).expect("map");
    println!("main map entry \"{}\" blocks {}", last.name, mf.blocks.len());
    let mut checked = 0usize;
    let mut remainder = 0usize;
    for (bi, blk) in mf.blocks.iter().enumerate() {
        if blk.id != 0x2e { continue; }
        let payload = blk.payload(edata);
        let c = match nf_formats::collision::Collision::parse(payload) { Ok(c) => c, Err(e) => { println!("  block {bi}: parse error {e:?}"); continue; } };
        checked += 1;
        let expect = 0x1c + c.words.len() / 2 * 0x40 + c.boxes.len() * 0x30 + c.tris.len() * 9;
        let ntri_hdr = u16::from_le_bytes([payload[6], payload[7]]) as usize;
        if expect != payload.len() || ntri_hdr != c.tris.len() {
            remainder += 1;
            println!("  block {bi}: size {} header-tris {ntri_hdr} parsed-tris {} expected {} REMAINDER {}", blk.size, c.tris.len(), expect, payload.len() as i64 - expect as i64);
        }
    }
    println!("checked {checked} blocks, {remainder} with a remainder / header mismatch");

    // Room_Road_A1 (plc 86) and Room_Road_B (plc 87): up-facing near-horizontal tris by y inside the bands.
    let lv = nf_formats::level::Level::parse(&data).expect("level");
    let main = lv.main_map().unwrap();
    for (pi, want, band) in [(86usize, "Room_Road_A1", ([82.0, 96.0], [168.0, 146.0])), (87, "Room_Road_B", ([10.0, 96.0], [80.0, 146.0]))] {
        let p = &lv.maps[main].placements[pi];
        let target = if p.by_hash() { lv.by_hash.get(&p.hashcode).copied() } else { Some((main, p.index as usize)) };
        let Some((om, oi)) = target else { println!("plc {pi} unresolved"); continue };
        let c = lv.maps[om].objects[oi].collision.as_ref().expect("room coll");
        println!("\n== plc {pi} \"{want}\" tris {} — up-facing horizontal tri plane-y histogram inside x {:?} z {:?} ==", c.triangles().len(), band.0, band.1);
        let mut hist: std::collections::BTreeMap<i32, u32> = Default::default();
        for t in c.triangles() {
            let e1 = [t[1][0] - t[0][0], t[1][1] - t[0][1], t[1][2] - t[0][2]];
            let e2 = [t[2][0] - t[0][0], t[2][1] - t[0][1], t[2][2] - t[0][2]];
            let n = [e1[1] * e2[2] - e1[2] * e2[1], e1[2] * e2[0] - e1[0] * e2[2], e1[0] * e2[1] - e1[1] * e2[0]];
            let l = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
            if l <= 0.0 || n[1] / l < 0.9 { continue; }
            let cy = (t[0][1] + t[1][1] + t[2][1]) / 3.0;
            let cx = (t[0][0] + t[1][0] + t[2][0]) / 3.0;
            let cz = (t[0][2] + t[1][2] + t[2][2]) / 3.0;
            if cx < band.0[0] || cx > band.1[0] || cz < band.0[1] || cz > band.1[1] { continue; }
            *hist.entry((cy * 2.0).round() as i32).or_insert(0) += 1;
        }
        for (yb, n) in hist { println!("  plane-y ~{:.1}: {} tris", yb as f32 / 2.0, n); }
        // Down-ray samples inside the band: what does a disc-faithful walk hit?
        for (tag, x, z) in [("n-floor", 120.0, 115.0), ("mid", 130.0, 120.0), ("lip", 132.0, 125.4), ("s-floor", 130.0, 135.0), ("e-floor", 160.0, 135.0), ("w-floor", 60.0, 105.0), ("w-south", 60.0, 130.0)] {
            let from = [x, 14.0, z];
            let dir = [0.0f32, -1.0, 0.0];
            let mut best: Option<(f32, [f32; 3])> = None;
            for t in c.triangles() {
                let e1 = [t[1][0] - t[0][0], t[1][1] - t[0][1], t[1][2] - t[0][2]];
                let e2 = [t[2][0] - t[0][0], t[2][1] - t[0][1], t[2][2] - t[0][2]];
                let n = [e1[1] * e2[2] - e1[2] * e2[1], e1[2] * e2[0] - e1[0] * e2[2], e1[0] * e2[1] - e1[1] * e2[0]];
                let dd = n[0] * dir[0] + n[1] * dir[1] + n[2] * dir[2];
                if dd >= 0.0 { continue; } // the game's one-sided skip
                // For a straight-down ray: barycentric containment in the xz plane, then y lerp.
                let e1x = t[1][0] - t[0][0]; let e1z = t[1][2] - t[0][2];
                let e2x = t[2][0] - t[0][0]; let e2z = t[2][2] - t[0][2];
                let det = e1x * e2z - e1z * e2x;
                if det.abs() < 1e-12 { continue; }
                let (px, pz) = (x - t[0][0], z - t[0][2]);
                let u = (px * e2z - pz * e2x) / det;
                let v = (e1x * pz - e1z * px) / det;
                if u < -0.001 || v < -0.001 || u + v > 1.001 { continue; }
                let y = t[0][1] + u * (t[1][1] - t[0][1]) + v * (t[2][1] - t[0][1]);
                if y <= 14.0 && best.map_or(true, |(by, _): (f32, [f32; 3])| y > by) { best = Some((y, [x, y, z])); }
            }
            println!("  down-ray ({tag}) at ({x},{z}): {:?}", best.map(|(y, _)| y));
        }
    }
}
