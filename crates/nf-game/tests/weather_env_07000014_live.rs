//! 07000014's two 0xDF weather placements: the game's `Env_Create` 0x17b968 stores
//! `GlobalEnv = env` on EVERY successful call (the cel gate early-outs before the store), and
//! each store also runs `InitDrops(drop_type)` 0x18bf68:8 — so the LAST created 0xDF placement
//! owns the live DropType. 07000014's main map carries #425 "Weather" (p0 = 0xff) and #588
//! "Bubble_gen" (p0 = 2) [D census_07000014], in placement order — the same order
//! `parsemap_create_dynamic_objects` 0x1d04f0 walks — so the game's live env is Bubble_gen's
//! (DropType 2; `InitDrops` gives type 2 its own quarter-scale random table [M 0x18bf68:39-44]).
//! The port took the FIRST match, which pinned Weather's 0xff.
//!
//! [S unverified] the game's `build_FindCel` gate can skip a placement whose cel lookup fails
//! or whose cel already carries an env (0x17b968 returns early, no store) — this test pins the
//! port's last-match rule, not a live PCSX2 reading.
use nf_formats::iso9660::IsoImage;
use nf_formats::level::Level;
use nf_formats::{disc, placement::Placement};

fn level_14() -> Level {
    let iso = std::env::var("NF_ISO").expect("NF_ISO");
    let image = IsoImage::open(&iso).expect("iso");
    let (entries, bin) = disc::files_bin_entries(&image).expect("entries");
    let fe = entries
        .iter()
        .find(|e| e.name.eq_ignore_ascii_case("07000014.bin"))
        .expect("14");
    let data = image
        .read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64)
        .expect("read");
    Level::parse(&data).expect("parse")
}

#[test]
fn the_weather_is_the_last_0xdf_placement_bubble_gen() {
    let level = level_14();
    let m = level.main_map().expect("main map");
    let placements = &level.maps[m].placements;
    let p0 = |p: &Placement| {
        p.params.iter().find(|(k, _)| *k == 0).map(|&(_, v)| v as u8).unwrap_or(0)
    };

    // [D] the two weather rows in placement order (census flat ids #425/#588 = "Weather" then
    // "Bubble_gen"; positions verbatim), with their drop types.
    let dfs: Vec<&Placement> = placements
        .iter()
        .filter(|p| p.flags & 0xff == 0xdf)
        .collect();
    assert_eq!(dfs.len(), 2, "07000014's kind-0xDF rows (measured on the disc)");
    let close = |a: [f32; 3], b: [f32; 3]| a.iter().zip(b).all(|(x, y)| (x - y).abs() < 0.01);
    assert!(close(dfs[0].translation, [0.0, 0.0, 0.0]), "[D] first 0xDF = #425 Weather at the origin");
    assert!(close(dfs[1].translation, [75.67, 1.25, 47.86]), "[D] last 0xDF = #588 Bubble_gen");
    assert_eq!(p0(dfs[0]), 0xff, "[D] Weather p0 = 0xff");
    assert_eq!(p0(dfs[1]), 2, "[D] Bubble_gen p0 = 2 (bubbles)");

    // [M 0x17b968] the port's env must come from the LAST 0xDF row — Bubble_gen, not Weather.
    let mut sim = nf_game::drone::Drones::new(Vec::new(), nf_game::drone::Rng::new(1));
    sim.attach_level_id(&level, 0x0700_0014);
    let (kind, _) = match sim.pending_env_drop_type() { Some(k) => (k, ()), None => panic!("the port builds weather from a 0xDF row") };
    assert_eq!(
        kind, 2,
        "GlobalEnv/InitDrops take the LAST 0xDF's drop type [M 0x17b968 stores on every call]"
    );
    assert_eq!(sim.sun_vector, [0.0; 3], "[D] Bubble_gen carries no sun params (p9 = 0)");
}
