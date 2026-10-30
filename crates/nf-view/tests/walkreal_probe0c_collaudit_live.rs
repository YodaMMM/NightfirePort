//! walkreal probe #23 (level-0c-gate-band-crossing): the collision-block leaf coverage audit.
//! For every collision-bearing object: the header's ntri vs the sum of the leaf tri ranges vs
//! the decoded count. A gap = tris the port never decodes (the band floor over the void?!).
//! Also dumps any missed tris' vertices for the band objects. Diagnostic only, --nocapture.

use nf_formats::iso9660::IsoImage;
use nf_formats::level::Level;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

#[test]
fn probe_coll_leaf_coverage() {
    let isopath = iso();
    let img = IsoImage::open(&isopath).expect("open iso");
    let (entries, bin) = nf_formats::disc::files_bin_entries(&img).expect("FILES.BIN");
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case("0700000c.bin")).expect("0700000c.bin");
    let data = img.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).expect("read");
    let level = Level::parse(&data).expect("level");

    let mut bad = 0usize;
    for (mi, m) in level.maps.iter().enumerate() {
        for (oi, o) in m.objects.iter().enumerate() {
            let Some(c) = o.collision.as_ref() else { continue };
            let ntri_header = c.tris.len();
            let mut covered = vec![false; ntri_header];
            let mut leaves = 0usize;
            for b in &c.boxes {
                if b.children.is_some() { continue; }
                leaves += 1;
                for k in b.tris.start..b.tris.end {
                    if (k as usize) < ntri_header { covered[k as usize] = true; }
                }
            }
            let missed: Vec<usize> = (0..ntri_header).filter(|&k| !covered[k]).collect();
            let decoded = c.triangles().len();
            let want_decoded = ntri_header - missed.len();
            if !missed.is_empty() || decoded != want_decoded {
                bad += 1;
                eprintln!("map{mi} obj{oi} \"{}\": header ntri {ntri_header}, leaves {leaves}, missed {} tris, decoded {decoded} (want {want_decoded})",
                    o.name.trim(), missed.len());
                if missed.len() <= 40 {
                    // Where are the missed tris? Decode them raw (leaf 0's params as a guess is wrong —
                    // just report their halfword indices).
                    eprintln!("   missed idx: {:?}", &missed[..missed.len().min(40)]);
                }
            }
        }
    }
    eprintln!("objects with coverage gaps: {bad}");
    eprintln!("ok");
}
