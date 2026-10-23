//! probe 07000015 disc-side: the raw 0x2E collision tris near the ladder base (x 80..92,
//! y 20..32, z −6..3) from EVERY map/placement, transformed to world like Ladder::create does —
//! does the DISC carry deck floor under the east room that the port's world missed?
//! Diagnostic only — run with `--ignored --nocapture`.

use nf_formats::iso9660::IsoImage;
use nf_formats::level::Level;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

#[test]
#[ignore = "assertion-free probe dump; run with --ignored"]
fn probe_disc_tris_near_ladder() {
    let img = IsoImage::open(&iso()).expect("open iso");
    let (entries, bin) = nf_formats::disc::files_bin_entries(&img).expect("FILES.BIN");
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case("07000015.bin")).expect("07000015.bin");
    let data = img.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).expect("read");
    let lv = Level::parse(&data).expect("parse");

    let mut count = 0usize;
    for (mi, m) in lv.maps.iter().enumerate() {
        for p in &m.placements {
            let t = p.translation;
            // Cheap reject: anything with |x−86|>14 or |z+1|>12 or y out of 14..46 can't land in the box.
            if !(t[0] - 14.0..=t[0] + 14.0).contains(&86.0) { continue; }
            if !(-13.0..=11.0).contains(&t[2]) { continue; }
            if !(14.0..=46.0).contains(&t[1]) { continue; }
            let scale = p.scale();
            let sc = [scale[0].abs(), scale[1].abs(), scale[2].abs()];
            let yaw = p.rotation[1];
            let (s, c) = (yaw.sin(), yaw.cos());
            let Some(col) = m.objects.get(p.index as usize).and_then(|o| o.collision.as_ref()) else { continue; };
            for tri in col.triangles() {
                let w = tri.map(|v| [
                    t[0] + v[0] * sc[0] * c + v[2] * sc[2] * s,
                    t[1] + v[1] * sc[1],
                    t[2] - v[0] * sc[0] * s + v[2] * sc[2] * c,
                ]);
                let in_box = w.iter().all(|v| (80.0..=92.0).contains(&v[0]) && (20.0..=32.0).contains(&v[1]) && (-6.0..=3.0).contains(&v[2]));
                if !in_box { continue; }
                count += 1;
                eprintln!("map{mi} plc{} tri ({:.2},{:.2},{:.2}) ({:.2},{:.2},{:.2}) ({:.2},{:.2},{:.2})",
                    p.index, w[0][0], w[0][1], w[0][2], w[1][0], w[1][1], w[1][2], w[2][0], w[2][1], w[2][2]);
            }
        }
    }
    eprintln!("total tris in the ladder-base box (disc, all maps): {count}");
}
