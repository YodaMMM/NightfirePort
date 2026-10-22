//! level-14 probe: the well's catcher — where does ladder 02000751 live? Prints only.
//! The lid script 0x06000A3B's Entity command says type 25 (params [00,00]); the game's
//! `Script_CreateEntity` 0x1d8968 has NO 0x19 arm — type 25 falls to the generic re-bind
//! (obj+0xff class check), so the ladder must exist as a PLACEMENT (the wire sweep's
//! `0x1d04f0 case 0x22 → Ladder_Create`) and the script only re-binds it. This probe
//! prints the level's placement record for hash 0x02000751 and the port's ladder census.

use nf_formats::iso9660::IsoImage;
use nf_formats::level::Level;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

#[test]
fn probe_the_well_ladder_placement() {
    let iso = IsoImage::open(&iso()).unwrap();
    let (entries, bin) = nf_formats::disc::files_bin_entries(&iso).unwrap();
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case("07000014.bin")).expect("level");
    let data = iso.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).unwrap();
    let level = Level::parse(&data).unwrap();

    // 1. Every placement whose hash (or index-object hash) is 02000751, plus everything near the
    //    well (x 38..43, z 53..59, y < 3): the lid 02000750, the ladder, whatever else stands there.
    eprintln!("== placements of hash 02000751 / 02000750 ==");
    for (mi, m) in level.maps.iter().enumerate() {
        for p in &m.placements {
            let h = if p.by_hash() { Some(p.hashcode) } else {
                m.objects.get(p.index as usize).map(|o| o.hashcode)
            };
            if matches!(h, Some(0x0200_0751) | Some(0x0200_0750)) {
                let obj = h.and_then(|h| level.by_hash.get(&h)).and_then(|(om, oi)| level.maps.get(*om).and_then(|map| map.objects.get(*oi)));
                eprintln!(" map{mi} hash {h:?} kind {:#04x} pos {:?} rot {:?} params {:?} obj {:?}",
                    p.flags & 0xff, p.translation, p.rotation, p.params.iter().map(|&(i, v)| (i, v)).collect::<Vec<_>>(),
                    obj.map(|o| (o.hashcode, &o.name)));
            }
        }
    }
    // 2. The port's ladder census: every ladder, then the ones near the well box.
    let ladders = nf_game::ladder::all(&level);
    eprintln!("== ladder::all: {} ladders ==", ladders.len());
    for l in &ladders {
        eprintln!("  ladder pos {:?} rot {:?} span_y {:?}", l.centre, l.yaw, l.y_span);
    }
    let near: Vec<_> = ladders.iter().filter(|l| (38.0..43.0).contains(&l.centre[0]) && (53.0..59.0).contains(&l.centre[2])).collect();
    eprintln!("== ladders near the well box x 38..43 z 53..59: {} ==", near.len());
    for l in near { eprintln!("  NEAR {:?}", l.centre); }
    // 3. The by_hash entry itself.
    eprintln!("by_hash 02000751 → {:?}", level.by_hash.get(&0x0200_0751));
    eprintln!("by_hash 02000750 → {:?}", level.by_hash.get(&0x0200_0750));
}
