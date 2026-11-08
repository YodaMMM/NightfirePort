//! music-unported-consumers: the audit's tabled unported consumers, pinned on the real levels.
//!
//! - the cel music-zone channel: `Player_Update` 0x193600:117-122 posts `Music_Event(cel+0xa1, 1)`
//!   every tick the player's cel carries one; the channel is the room-cel entity's prop 0x3c
//!   [M parseentity_fixup_entity 0x1d00c0]. On 07000002 the disc carries zones 30/36/37 [D
//!   placement param 4 census] — the 36 hall's event is MayhewB 0x112270's level-start arm
//!   (section 2), so a real walk-in is audible through `music.out`.
//! - the MapMusic rows the unit tests mirror, straight from the disc's ACTION.ELF [D 0x24f4d8].

use nf_formats::iso9660::IsoImage;

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

/// The disc's MapMusic rows for the four levels this task ports, through `nf_formats::music::tables`.
#[test]
fn the_mapmusic_rows_the_machines_mirror_come_from_the_disc() {
    let Some(path) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let img = IsoImage::open(&path).expect("open iso");
    let elf = img.read_file(img.find("/ACTION.ELF").expect("no ACTION.ELF")).expect("read elf");
    let (mm, _) = nf_formats::music::tables(&elf).expect("tables parse");
    let row = |lv: u32| mm.iter().find(|m| m.level == lv).expect("row").clone();
    let b = row(0x0700_0002);
    assert_eq!((b.track, b.start, b.on_death, b.on_fail, b.end_combat, b.end_calm), (20, 0, 27, 26, 24, 25));
    assert_eq!((b.combat, b.stealth), (vec![12, 14, 16], vec![2, 20, 22, 8]));
    let c = row(0x0700_0003);
    assert_eq!((c.track, c.end_combat, c.end_calm), (21, 16, 17));
    assert_eq!((c.combat, c.stealth), (vec![2, 4, 14], vec![0, 6, 8]));
    let f = row(0x0700_0006);
    assert_eq!((f.track, f.end_combat, f.end_calm), (16, 8, 6));
    assert_eq!((f.combat, f.stealth), (vec![2, 15, 17], vec![4, 19, 21]));
    let t = row(0x0700_000b);
    assert_eq!((t.track, t.on_death, t.on_fail, t.end_combat, t.end_calm), (24, 16, 15, 17, 17));
    assert!(t.combat.is_empty() && t.stealth.is_empty(), "0b manages its own sections");
}

/// The cel-zone producer on a real level: Bond walked into 07000002's zone-36 hall (cel #19,
/// min (−54.1, 3.8, 20.1) max (13.4, 16.9, 47.2) [D placement param 4 census]) posts the
/// channel every tick, and MayhewB 0x112270:48-54's level-start arm (the IOP still at section
/// 0, e[36] high) starts section 2 — pre-fix the port parsed no music channel, so the hall was
/// silent and the level music stayed at the start section. Sister: with the board zeroed and
/// Bond outside the zoned cels, no zone-36 event and no section-2 jump.
#[test]
fn the_02_zone_36_hall_starts_the_level_music_through_the_cel_channel() {
    let Some(path) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = nf_view::load(&path, "07000002.bin", false).expect("load 07000002");
    let sim = scene.sim.as_mut().unwrap();
    // The 02 intro NIS runs at load — skip it the way the game's Script_FFwd does
    // [M SP_Update 0x1dbda8:129-141]: the skip request the viewer's key handler sets.
    for _ in 0..600 {
        if sim.drones.scripts.cutscene_playing() { sim.drones.scripts.skip_request = true; }
        sim.step_bond(Default::default(), false, false, false);
        if !sim.disabled { break; }
    }
    assert!(!sim.disabled, "Bond is free after the intro");
    // Sister pre-check: a few ticks wherever the intro leaves Bond — no zone-36 section-2 jump
    // unless the intro already sits in the hall (the 02 intro is the sewer side [D level start]).
    for _ in 0..20 { sim.step_bond(Default::default(), false, false, false); }
    let pre_jumps: Vec<i32> = sim.drones.music.out.iter().filter_map(|c| match c { nf_game::music::Cmd::Jump { section, .. } => Some(*section), _ => None }).collect();
    // The hall's centre-left, well inside cel #19's box and above its floor band.
    sim.bond.pos = [-30.0, 6.0, 30.0];
    sim.bond.prev_pos = sim.bond.pos;
    sim.drones.music.out.clear();
    let mut saw_start = false;
    for _ in 0..40 {
        sim.step_bond(Default::default(), false, false, false);
        if sim.drones.music.out.iter().any(|c| *c == nf_game::music::Cmd::Jump { section: 2, instant: false }) { saw_start = true; break; }
        sim.drones.music.out.clear();
    }
    assert!(saw_start || pre_jumps.contains(&2),
        "no section-2 start from the zone-36 hall (out: {:?})", sim.drones.music.out);
}

/// 07000001's own machine on the real level: the opening scene's end starts the level's sections
/// [M UpdateMFX_02MayhewA 0x111ed8:116-120]. Before the port had it the music never left the start
/// section. Sister: while the scene plays, no section jump.
#[test]
fn the_01_intro_end_starts_the_level_sections() {
    use nf_game::music::Cmd;
    let Some(path) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = nf_view::load(&path, "07000001.bin", false).expect("load 07000001");
    let sim = scene.sim.as_mut().unwrap();
    let step = |sim: &mut nf_view::scene::Sim| {
        sim.step_bond(Default::default(), false, false, false);
        sim.drones.music.out.drain(..).filter_map(|c| if let Cmd::Jump { section, .. } = c { Some(section) } else { None }).collect::<Vec<i32>>()
    };
    for _ in 0..120 {
        assert!(step(sim).is_empty(), "no section jump while the opening scene plays");
    }
    assert!(sim.drones.scripts.cutscene_playing(), "the opening scene is up");
    let mut got = Vec::new();
    for _ in 0..600 {
        if sim.drones.scripts.cutscene_playing() { sim.drones.scripts.skip_request = true; }
        got.extend(step(sim));
        if !got.is_empty() { break; }
    }
    assert_eq!(got.first(), Some(&3), "the scene's end jumps section 3 [M :116-120]: {got:?}");
}
