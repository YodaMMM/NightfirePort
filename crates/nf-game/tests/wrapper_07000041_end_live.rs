//! The 07000041 wrapper's end, pinned on the disc's own data (level-41-exit-event). The level
//! carries NO in-level exit carrier: the 7 type-7 scripts decode without a 0x12 event and nothing
//! ever runs them (the only two ScriptPlayer placements reference a script hash that exists in no
//! archive on the disc, with trigger mask 0); there is no 0xE8 LOAD_LEVEL; the one 0xF4 movie
//! trigger's E-arm carries movie code 0 and loads nothing [M 0x1b2518-55]. Opt-in: `NF_ISO`.
//!
//! The sisters are structural: each pin has a counterpart that fails if the data differed (the
//! positive controls are the level's OWN other records — the 0x3/0xF events, the 0xEA outs).

use nf_formats::iso9660::IsoImage;
use nf_formats::level::Level;
use nf_formats::nis::{Cmd, Nis};

fn level_of(name: &str) -> Option<Level> {
    let path = std::env::var_os("NF_ISO")?;
    let iso = IsoImage::open(&path).unwrap();
    let (entries, bin) = nf_formats::disc::files_bin_entries(&iso).unwrap();
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case(name)).unwrap();
    let data = iso.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).unwrap();
    Some(Level::parse(&data).unwrap())
}

/// The level's placements across ALL its maps (the main map 01000058 holds the interactive
/// records; the ScriptPlayers live in map 01000081).
fn all_placements(level: &Level) -> Vec<&nf_formats::placement::Placement> {
    level.maps.iter().flat_map(|m| m.placements.iter()).collect()
}

/// [D] No 0xE8 LOAD_LEVEL placement exists in any map. Sister: the level's 0xEA records prove
/// the scan reaches the records (a filter bug would zero them too).
#[test]
fn no_load_level_placement_on_the_wrapper() {
    let Some(level) = level_of("07000041.bin") else { eprintln!("skip: NF_ISO not set"); return };
    let ps = all_placements(&level);
    let e8: Vec<_> = ps.iter().filter(|p| p.flags & 0xff == 0xe8).collect();
    assert!(e8.is_empty(), "07000041 has no 0xE8 placement [D], found {} records", e8.len());
    let ea: Vec<_> = ps.iter().filter(|p| p.flags & 0xff == 0xea).collect();
    assert_eq!(ea.len(), 2, "the two 0xEA touch-once triggers [D]");
}

/// [D] The interactive records' parameters, straight from the disc: the one 0xF4 carries
/// params (0, 4, 0) — movie code 0, the E mask 4, input slot 0 (its update path is inert
/// [M 0x1b1f24-28]); the 0xEA pair outs ch 0x6e (mask 8, the auto touch) and ch 1 (mask 4,
/// the E arm) — the approach chain's channel law.
#[test]
fn the_wrapper_interactive_records_params() {
    let Some(level) = level_of("07000041.bin") else { eprintln!("skip: NF_ISO not set"); return };
    let ps = all_placements(&level);
    let param = |p: &nf_formats::placement::Placement, k: u32| {
        p.params.iter().find(|(j, _)| *j == k).map(|&(_, v)| v).unwrap_or(0)
    };
    let f4: Vec<_> = ps.iter().filter(|p| p.flags & 0xff == 0xf4).collect();
    assert_eq!(f4.len(), 1, "exactly one 0xF4 movie-player trigger [D census: 1]");
    let p = f4[0];
    assert_eq!((param(p, 0), param(p, 1), param(p, 2)), (0, 4, 0),
        "07000041's 0xF4 params (movie code 0, mask 4, slot 0) [D]");
    // Sister on the port's read side: the built trigger carries movie_code 0 — nothing the
    // player can do here produces a level load through the movie path.
    let trigs = nf_game::triggers::build(&level);
    let movies: Vec<_> = trigs.iter().filter(|t| t.kind == nf_game::triggers::kind::MOVIE).collect();
    assert_eq!(movies.len(), 1);
    assert_eq!(movies[0].movie_code, 0, "the disc's movie code is 0 [D] — the E-arm loads nothing [M 0x1b2518-55]");
    assert!(trigs.iter().all(|t| t.next_level == 0), "no built trigger carries a next level [D]");
    // The 0xEA pair's outs: ch 1 (the approach chain, E-armed) and ch 0x6e (auto-touch).
    let mut outs: Vec<(u32, u32)> = Vec::new();
    for p in &ea {
        outs.push((param(p, 0), param(p, 1)));
    }
    outs.sort();
    assert_eq!(outs, vec![(1, 4), (0x6e, 8)], "0xEA outs (ch, mask): the approach chain + the auto touch [D]");
}

/// [D] The 7 type-7 scripts decode through the port's own reader; NO stream carries a 0x12
/// level-load event (only 0x3 delay ×4 and 0xF callback ×3) and no 0xD channel write either —
/// the old stand-in's hoped-for carrier does not exist on the disc. Positive control: the truck
/// script on 07000005 (0x06000A22) fires 0xC and 0xD — the same reader sees real events.
#[test]
fn the_seven_scripts_carry_no_0x12_event() {
    let Some(level) = level_of("07000041.bin") else { eprintln!("skip: NF_ISO not set"); return };
    let mut hashes: Vec<u32> = level.nis.keys().copied().collect();
    hashes.sort();
    assert_eq!(hashes, vec![0x0600_0027, 0x0600_0044, 0x0600_0045, 0x0600_004f, 0x0600_0052, 0x0600_006f, 0x0600_07c4],
        "the 7 type-7 scripts the archive carries [D]");
    let mut kinds: Vec<u8> = Vec::new();
    for nis in level.nis.values() {
        for st in &nis.streams {
            for cmd in st.decode() {
                if let Cmd::Event { kind, .. } = cmd { kinds.push(kind); }
            }
        }
    }
    kinds.sort();
    assert_eq!(kinds, vec![0x3, 0x3, 0x3, 0x3, 0xf, 0xf, 0xf],
        "the wrapper's event census: delay ×4 + callback ×3, NO 0x12, NO 0xd [D]");
    // Sister: the truck script fires 0xC and 0xD — the same reader sees real events.
    let l05 = level_of("07000005.bin").unwrap();
    let truck = l05.nis.get(&0x0600_0a22).expect("the truck script parses");
    let mut seen: Vec<u8> = Vec::new();
    for st in &truck.streams { for cmd in st.decode() { if let Cmd::Event { kind, .. } = cmd { seen.push(kind); } } }
    assert!(seen.contains(&0xd), "the truck's channel event 0xD decodes (the reader sees real events) [D]");
}

/// [D] Nothing ever RUNS the 7 scripts: the level's only two 0xD9 ScriptPlayer placements (map
/// 01000081, idx 0 and 42) both carry script hash 0x06000000 — a hash no archive's type-7 entry
/// carries on the whole disc — and trigger mask p5 = 0 (state 6 + `trigger = flags & 0` → both
/// idle forever [M script-player.md]). Sister: 0x06000027 — the hash the old timer was calibrated
/// against — IS in the archive, so the param check cannot pass by accident.
#[test]
fn the_script_players_reference_a_hash_that_is_not_on_the_disc() {
    let Some(level) = level_of("07000041.bin") else { eprintln!("skip: NF_ISO not set"); return };
    let sps: Vec<_> = all_placements(&level).into_iter().filter(|p| p.flags & 0xff == 0xd9).collect();
    assert_eq!(sps.len(), 2, "exactly two ScriptPlayer placements [D census]");
    let param = |p: &nf_formats::placement::Placement, k: u32| {
        p.params.iter().find(|(j, _)| *j == k).map(|&(_, v)| v).unwrap_or(0)
    };
    for p in &sps {
        assert_eq!(param(p, 0), 0x0600_0000, "script A = 0x06000000 [D] — absent from every archive");
        assert_eq!(param(p, 5), 0, "trigger mask 0 → never triggers [D + M script-player.md]");
    }
    assert!(!level.nis.contains_key(&0x0600_0000), "no 06000000 script in the archive [D]");
    assert!(level.nis.contains_key(&0x0600_0027), "0x06000027 exists in the archive [D] — the control");
}

/// The Nis reader stands on its own two feet for the format facts the pins above lean on
/// (headless — the sister of `parses_header_streams_and_keys`).
#[test]
fn a_0x12_event_decodes_when_present() {
    // op 0x12, n=1, kind=0x12, param 0x0700001b (LE 1b 00 00 07), trailing byte — a stream of
    // size 12 (commands = size − 4 = 8).
    let mut v: Vec<u8> = Vec::new();
    for w in [0x1cu16, 0, 1, 0, 100] { v.extend(w.to_le_bytes()); }
    v.extend(0u16.to_le_bytes()); v.extend(1u16.to_le_bytes()); v.extend(12u32.to_le_bytes());
    v.extend([0x12, 0x01, 0x12, 0x1b, 0x00, 0x00, 0x07, 0x12]);
    v.extend(2u32.to_le_bytes()); v.extend(3u32.to_le_bytes()); v.extend([9, 9, 9]);
    for k in 0..2 { for i in 0..12 { v.extend(((k * 100 + i) as f32).to_le_bytes()); } }
    let s = Nis::parse(0x0600_0001, &v).unwrap();
    assert_eq!(s.streams[0].decode(), vec![Cmd::Event { kind: 0x12, params: vec![0x0700_001b] }],
        "the port's reader decodes the load event the old stand-in hoped for [M nis.rs decode]");
}
