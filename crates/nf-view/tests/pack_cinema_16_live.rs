//! U-051 (hurt-volumes-16b): the pack cinema 0x06000631 — the disc proof that the 07000016 pack
//! cinema never moves Bond. Its Bond-actor stream (stream 8, kind 1) carries exactly ONE key at
//! the pit floor, so the event-6 hand-back snapshot [M Script_EventHandler 0x1d94c0:125-134 →
//! Player_Enable 0x195498's teleport] re-enables the player INSIDE the ch-0xf3 hurt volume's box
//! — the game's escape is the pack flight, not a cinema move. Opt-in via NF_ISO.
#[test]
fn the_pack_cinema_never_moves_bond_stream8_is_one_pit_floor_key() {
    use nf_formats::{disc, iso9660::IsoImage, levelbin::LevelBin, nis::Nis};
    let Ok(iso_path) = std::env::var("NF_ISO") else { eprintln!("NF_ISO not set — skipping"); return };
    let iso = IsoImage::open(&iso_path).unwrap();
    let (entries, bin) = disc::files_bin_entries(&iso).unwrap();
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case("07000016.bin")).expect("07000016.bin");
    let data = iso.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).unwrap();
    let lb = LevelBin::parse(&data).unwrap();
    // The loadable-pack record [D]: (0x06000631, flag 1) → 07800016.bin (nibble 8).
    let rec = lb.entries.iter().filter(|e| e.kind == 0xC)
        .map(|e| lb.data(&data, e))
        .find(|r| r.len() >= 5 && u32::from_le_bytes([r[0], r[1], r[2], r[3]]) == 0x06000631)
        .expect("07000016's type-0xC record for the pack cinema");
    assert_eq!(rec[4], 1, "the pack cinema's flag nibble (07800016.bin) changed");
    let pfe = entries.iter().find(|e| e.name.eq_ignore_ascii_case("07800016.bin")).expect("07800016.bin on the disc");
    let pdata = iso.read_bytes(bin.lba as u64 * 2048 + pfe.offset as u64, pfe.size as u64).unwrap();
    let plb = LevelBin::parse(&pdata).unwrap();
    let pe = plb.entries.iter().find(|e| e.kind == 7 && u32::from_str_radix(&e.name, 16) == Ok(0x06000631))
        .expect("0x06000631 in the pack");
    let n = Nis::parse(0x06000631, plb.data(&pdata, pe)).expect("the pack cinema parses");
    assert_eq!(n.length, 750, "the pack cinema's length");
    // Stream 8: the Bond actor, kind 1 (the event-6 previous stream) — ONE key at the pit floor.
    let s8 = &n.streams[8];
    assert_eq!(s8.first_key, s8.last_key, "the Bond-actor stream must hold a single key (the actor never moves)");
    let key = &n.keys[s8.first_key as usize];
    assert!((key.time - 747.0).abs() < 0.5, "the single key's time {}", key.time);
    for k in 0..3 {
        assert!((key.pos[k] - [3.57f32, -22.76, -179.55][k]).abs() < 0.02,
            "the re-enable pose axis {k}: {}", key.pos[k]);
    }
    // Sister: the cinema HAS moving streams (the pack close-up animates) — the static Bond is the
    // script's choice, not an empty archive.
    assert!(n.streams.iter().any(|st| st.last_key > st.first_key), "the cinema lost its animated streams");
}
