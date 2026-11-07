//! unread-params-doors: the placement-parameter census over every on-foot level [D].
//!
//! For the door.rs-neighbour classes — 0x20 Break, 0x22 Ladder, 0x3b DynObject, 0xdb Door,
//! 0xe1 Destroy, 0xe2 SS, 0xfd CamSubject — this walks every level archive in
//! `nf_view::dev::LEVELS` and counts, per placement parameter the game's `*_Create` stores,
//! the rows whose value is non-zero. A param that is zero on every level is closed: no
//! reader can ever see a value. A non-zero value on any on-foot level means the port must
//! handle it (or the census pins why it can't matter).
//!
//! The creator→param maps are the audit's ([M] in docs/fields/*.md): Break p0/p1/p2
//! (0x12c230), Ladder p1 (0x1966c0), DynObject p0..p3 (0x18dbf8), Door p0..p11 + key header
//! (0x136100), Destroy p0..p6 (0x12c318), SS p0..p7 + key header (0x1a9c58), CamSubject p0
//! (0x1bc5d8). Opt-in: NF_ISO. `--nocapture` prints the table.
use std::collections::HashMap;

use nf_formats::iso9660::IsoImage;
use nf_formats::level::Level;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

/// (kind label, kind byte, the params that kind's creator stores)
const KINDS: &[(&str, u32, std::ops::RangeInclusive<u32>)] = &[
    ("Break", 0x20, 0..=2),
    ("Ladder", 0x22, 0..=1),
    ("DynObject", 0x3b, 0..=3),
    ("Door", 0xdb, 0..=11),
    ("Destroy", 0xe1, 0..=6),
    ("SS", 0xe2, 0..=7),
    ("CamSubject", 0xfd, 0..=0),
];

/// One (kind, param) cell: how many LEVELS carry a non-zero row, how many rows, the max
/// value, and the first "level#placement" that was non-zero.
#[derive(Default, Clone)]
struct Cell { levels: u32, rows: u32, max: u32, first: String, levels_seen: u32 }

#[test]
fn placement_param_census() {
    let iso = iso();
    let img = IsoImage::open(&iso).expect("open iso");
    let (entries, bin) = nf_formats::disc::files_bin_entries(&img).expect("FILES.BIN");
    let mut table: HashMap<(String, u32), Cell> = HashMap::new();
    let mut rows_checked = 0u32;
    let mut levels_seen = 0u32;
    for &level in nf_view::dev::LEVELS {
        let name = nf_view::level_archive_name(level);
        let Some(fe) = entries.iter().find(|e| e.name.eq_ignore_ascii_case(&name)) else {
            eprintln!("{name}: not in FILES.BIN, skipped");
            continue;
        };
        let data = img.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).expect("read");
        let lv = Level::parse(&data).unwrap_or_else(|e| panic!("{name}: {e}"));
        let Some(mm) = lv.main_map() else { continue };
        levels_seen += 1;
        // Per level: (label, param) -> (rows, nonzero rows, max value, first non-zero placement).
        let mut here: HashMap<(String, u32), (u32, u32, u32, String)> = HashMap::new();
        for (i, p) in lv.maps[mm].placements.iter().enumerate() {
            rows_checked += 1;
            let param = |k: u32| p.params.iter().find(|(j, _)| *j == k).map_or(0, |&(_, v)| v);
            for (label, kind, range) in KINDS {
                if p.flags & 0xff != *kind { continue; }
                for k in range.clone() {
                    let v = param(k);
                    let e = here.entry((label.to_string(), k)).or_insert((0, 0, 0, String::new()));
                    e.0 += 1;
                    if v != 0 {
                        if e.1 == 0 { e.3 = format!("{name}#{i}"); }
                        e.1 += 1;
                        if v > e.2 { e.2 = v; }
                    }
                }
            }
        }
        levels_seen += 1;
        for (key, (rows, nz, max, first)) in here {
            let c = table.entry(key).or_default();
            c.levels_seen += 1;
            c.rows += nz;
            if nz > 0 { c.levels += 1; if c.first.is_empty() { c.first = first; } }
            if max > c.max { c.max = max; }
            let _ = rows;
        }
    }
    eprintln!("\n== placement param census: {rows_checked} main-map placements, {levels_seen} levels [D] ==");
    let mut rows: Vec<(&(String, u32), &Cell)> = table.iter().collect();
    rows.sort_by(|a, b| a.0.cmp(b.0));
    eprintln!("kind        param | levels-nonzero rows-nonzero       max  first-nonzero");
    for ((label, k), c) in &rows {
        eprintln!("{label:9} p{k:<5} | {:>14} {:>12} {:>8}  {}", c.levels, c.rows, c.max, c.first);
    }
    // The gap the audit targets: Destroy p6 — `Destroy_Create` 0x12c318's post-smash countdown
    // store `(int)(param6 * -FRAME_RATE)`, decremented by `Destroy_Update` 0x12c4f8:17-25 —
    // must be non-zero somewhere (07000014's generator cascade [D]); the port must carry it.
    let c = table.get(&("Destroy".to_string(), 6)).expect("Destroy p6 cell");
    assert!(c.rows > 0, "Destroy p6 is all-zero across the on-foot levels: nothing to port (report the census)");
    eprintln!("\nDestroy p6 non-zero: {} rows over {} levels, max {}, first {} [D]", c.rows, c.levels, c.max, c.first);
}
