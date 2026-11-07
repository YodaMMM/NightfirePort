//! unread-params-doors: the p6 post-smash countdown on the REAL 07000014 placements [D].
//!
//! `Destroy_Create` 0x12c318:57 stores the p6 param as `(int)(param6 · −FRAME_RATE)` — the
//! load-time FRAME_RATE reads 30 [L door.rs CREATE_FRAME_RATE; the task's fresh-load rule] —
//! `Destroy_Smash` 0x12bb60:31 negates it, and `Destroy_Update` 0x12c4f8:17-25 counts it down
//! to the output-channel write. The old port left p6 unread: "inert" held only where p6 = 0
//! (0700000d), while 07000014 carries 17 non-zero rows (p6 1..10 [D the census,
//! placement_param_census_live]) — its smash chains ran at once instead of staggering
//! p6·30 ticks per link. This test drives the real level's own destroy leaves through
//! `Props::create` + the per-tick update: the death channel must NOT fire in the smash tick
//! (red on the old code) and must fire exactly p6·30 ticks later. The kill channel is seeded
//! (a test seam standing in for the game's script/chain writer — the same seam the panel
//! sequence test uses for 0x51); the pin is the countdown mechanics on the disc's own rows.
use nf_formats::iso9660::IsoImage;
use nf_formats::level::Level;
use nf_game::props::Props;

fn level_14() -> Level {
    let iso = std::env::var("NF_ISO").expect("NF_ISO");
    let img = IsoImage::open(&iso).expect("iso");
    let (entries, bin) = nf_formats::disc::files_bin_entries(&img).expect("entries");
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case("07000014.bin")).expect("14");
    let data = img.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).expect("read");
    Level::parse(&data).expect("parse")
}

/// The 17 non-zero p6 rows [D 07000014.bin]: the create store is −p6·30.
#[test]
fn the_p6_rows_store_their_negative_tick_counts_at_create() {
    let level = level_14();
    let m = level.main_map().expect("main map");
    let props = Props::create(&level);
    let mut non_zero = 0;
    for d in &props.destroy {
        let p = &level.maps[m].placements[d.placement];
        let param = |k: u32| p.params.iter().find(|(j, _)| *j == k).map_or(0, |&(_, v)| v);
        let p6 = param(6);
        if p6 != 0 {
            non_zero += 1;
            assert_eq!(d.countdown, -((p6 as f32) * 30.0) as i32,
            "placement #{}: countdown {} = −p6 {} · 30 [M 0x12c318:57, L the load FRAME_RATE]",
            d.placement, d.countdown, p6);
        } else {
            assert_eq!(d.countdown, 0, "p6 = 0 leaves count nothing [D]");
        }
    }
    assert_eq!(non_zero, 17, "the level's non-zero p6 rows [D the census]");
}

/// #923 (kill ch 13 → death ch 117, p6 = 10 → 300 ticks) and #294 (kill 225 → 79, p6 = 1 →
/// 30 ticks) [D 07000014.bin]: the smash (via the watch arm) latches without dying, the
/// death channel stays OFF through the count, and lands ON at exactly p6·30 ticks.
#[test]
fn the_generator_room_leaves_fire_their_death_channels_p6_times_30_ticks_after_the_smash() {
    let level = level_14();
    let mut props = Props::create(&level);
    let by_placement = |props: &Props, pl: usize| props.destroy.iter().position(|d| d.placement == pl).expect("the placement's destroy leaf");
    let i923 = by_placement(&props, 923);
    let i294 = by_placement(&props, 294);
    assert_eq!(props.destroy[i923].kill_channel, 13, "#923's kill channel [D]");
    assert_eq!(props.destroy[i923].channel, 117, "#923's death channel [D]");
    assert_eq!(props.destroy[i923].countdown, -300, "#923's p6 = 10 → −300 [D+M]");
    assert_eq!(props.destroy[i294].channel, 79, "#294's death channel [D]");
    assert_eq!(props.destroy[i294].countdown, -30, "#294's p6 = 1 → −30 [D+M]");

    let mut ch = nf_game::triggers::Channels::default();
    // Seed both kill channels (the seam — the game's writers are the scripts and the pair
    // #787's own chain; the countdown is what's under test).
    ch.set(13, 1, 1);
    ch.set(225, 1, 1);
    let smashed = props.channel_kills(&mut ch, 1);
    assert_eq!(smashed.len(), 2, "both leaves smash in the tick (kind ≠ 0 blasts [M 0x12bc50])");
    for &i in &[i923, i294] {
        assert!(props.destroy[i].latched, "smashed [M 0x12bb60:17-20]");
        assert!(!props.destroy[i].dead, "class 0x21 holds until the count lands [M]");
    }
    assert!(!ch.on(117) && !ch.on(79), "neither death channel fired in the smash tick [M 0x12bb60:39-41] — RED on the old code");
    // The counts need exactly p6·30 UPDATE ticks after the smash tick (the count reads
    // p6·30 at the smash; each update decrements once [M 0x12c4f8:17-22]). #294's 30 land
    // first; #923 still has 270 to go.
    for f in 2..32u32 { props.channel_kills(&mut ch, f); } // 30 updates
    assert!(ch.on(79), "#294's death 79 fires 30 ticks after its smash [M 0x12c4f8:17-25]");
    assert!(props.destroy[i294].dead, "#294 reclasses [M 0x12c4f8:25]");
    assert!(!ch.on(117), "#923 still counts (270 to go) [M]");
    for f in 32..302u32 { props.channel_kills(&mut ch, f); } // 270 more
    assert!(ch.on(117), "#923's death 117 fires 300 ticks after its smash [M 0x12c4f8:17-25]");
    assert!(props.destroy[i923].dead, "#923 reclasses [M 0x12c4f8:25]");
}
