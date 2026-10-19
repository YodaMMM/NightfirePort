//! mp-character-data: the MP body data end to end.
//!
//! - Every MP character's mesh (all 29 `MP_skins` skins + their main meshes + skeleton + skin
//!   DATA file) resolves inside three MP arenas' own map packs [D — the arenas are the only
//!   source the game loads for MP characters; `MP_PostLoad_Init` 0x183718 loads nothing [M]].
//! - The parked bodies: an MP load builds one `MpBody` per human slot and per bot from the
//!   settings' character word [M `Player_Init` 0x1910f8:108-117, `BOT_init` 0x1249e8:100,160],
//!   parked below the world and hidden, with the body's script → sequence set resolved.
//!
//! Opt-in via `NF_ISO`.

use nf_formats::iso9660::IsoImage;
use nf_formats::level::Level;
use nf_formats::levelbin::LevelBin;
use nf_formats::mpanim::MpSkins;

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

/// The three arenas the test pins (the game's MP arenas are 0x07000021-29/4b/4c per
/// `ResetMap_LevelToLoad` 0x1d3800 [M]; 07000023 = the online test's arena).
const ARENAS: [&str; 3] = ["07000023.bin", "07000025.bin", "07000028.bin"];

fn arena_level(img: &IsoImage, name: &str) -> (Level, LevelBin, Vec<u8>) {
    let (entries, bin) = nf_formats::disc::files_bin_entries(img).unwrap();
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case(name)).unwrap_or_else(|| panic!("{name} not in FILES.BIN"));
    let data = img.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).unwrap();
    let level = Level::parse(&data).unwrap_or_else(|e| panic!("{name}: {e}"));
    let lb = LevelBin::parse(&data).unwrap();
    (level, lb, data)
}

#[test]
fn every_mp_characters_mesh_resolves_in_three_arenas() {
    let Some(path) = iso() else { eprintln!("skip: NF_ISO not set"); return };
    let img = IsoImage::open(&path).unwrap();
    let elf = img.read_file(img.find("/ACTION.ELF").unwrap()).unwrap();
    let skins = MpSkins::from_action_elf(&elf).unwrap();
    assert_eq!(skins.entries.len(), 29);
    for arena in ARENAS {
        let (level, lb, data) = arena_level(&img, arena);
        // The kind-6 skeleton entries carry no hashcode — identity is the parsed skeleton id.
        let skeleton_ids: std::collections::HashSet<u8> = lb.entries.iter().filter(|e| e.kind == 6)
            .filter_map(|e| nf_formats::character::parse_skeleton(lb.data(&data, e)).ok().map(|s| s.id as u8))
            .collect();
        for (i, e) in skins.entries.iter().enumerate() {
            let skin = level.skins.get(&e.skin).unwrap_or_else(|| panic!("{arena}: MP_skins[{i}] skin {:08x} not in the archive", e.skin));
            assert!(!skin.main.is_empty(), "{arena}: skin {:08x} has no main meshes", e.skin);
            for mh in skin.main.iter().flatten() {
                assert!(level.by_hash.contains_key(mh), "{arena}: skin {:08x} main mesh {mh:08x} unresolved", e.skin);
            }
            assert!(skeleton_ids.contains(&skin.skeleton),
                "{arena}: skin {:08x} skeleton {} (a kind-6 LevelBin entry) missing", e.skin, skin.skeleton);
            assert!(lb.entries.iter().any(|pe| pe.hash() == e.file),
                "{arena}: skin {:08x} DATA file {:08x} not in the LevelBin", e.skin, e.file);
        }
        eprintln!("{arena}: all 29 MP_skins skins resolve (meshes, skeleton, data file)");
    }
}

#[test]
fn an_mp_load_parks_one_body_per_slot_hidden_with_a_sequence_set() {
    let Some(path) = iso() else { eprintln!("skip: NF_ISO not set"); return };
    // Two humans (Bond 0 + a Phoenix Commando 9) and two bots (the disc's INITIAL records:
    // slot 4 = character 6, slot 5 = character 7 [D INITIAL_BOT_RECORDS]).
    let mut settings = nf_game::mp::MpSettings::deathmatch(0x0700_0023);
    settings.human_count = 2;
    settings.bot_count = 2;
    settings.players[0].skin = 0;
    settings.players[1].skin = 9;
    let mut scene = nf_view::load_mp_carry(&path, "07000023.bin", false, None, settings).expect("load 07000023");
    let bodies = &scene.mp_char_bodies;
    assert_eq!(bodies.len(), 4, "2 humans + 2 bots = 4 bodies");
    let skins = bodies.iter().map(|b| b.skin).collect::<Vec<_>>();
    assert_eq!(skins, [0x0500_0089, 0x0500_00c6, 0x0500_0097, 0x0500_0098],
        "slot chars 0/9 from the settings, 6/7 from the bot records (BOT_init 0x1249e8:160) [D MP_skins]");
    for b in bodies {
        assert_eq!(b.char_index as usize, match b.slot { 0 => 0, 1 => 9, 4 => 6, 5 => 7, s => panic!("slot {s}") });
        assert!(!b.chars.is_empty(), "slot {} has drawable characters", b.slot);
        for &ci in &b.chars {
            let c = &scene.characters[ci];
            assert!(c.matrix[3][1] <= -999_000.0, "slot {} char {ci} parked below the world", b.slot);
            assert!(scene.char_hidden(ci, Default::default()), "parked slot {} char {ci} is hidden", b.slot);
            assert_eq!(c.skin.hashcode, b.skin);
        }
        // The body's sequence set: the spawn idle (0x60001d4 [M 0x1910f8:157]) and the death
        // script (0x6000151 [M 0x190630]) resolve to sequences of the body's skeleton.
        for id in [0x0600_01d4u32, 0x0600_0151] {
            let seqs = b.scripts.get(&id).unwrap_or_else(|| panic!("{:08x}: body script {id:08x} not in the arena's kind-5 scripts", b.skin));
            assert!(!seqs.is_empty(), "{:08x}: script {id:08x} resolved no {} sequences", b.skin, b.skeleton);
        }
    }
    // The class-1 set (a P2K bot) resolves all five set words through the arena.
    for id in [0x0600_09dcu32, 0x0600_09de, 0x0600_09db, 0x0600_09d7, 0x0600_09d8] {
        assert!(!bodies[0].scripts.get(&id).unwrap_or_else(|| panic!("script {id:08x} missing")).is_empty(),
            "class-1 set word {id:08x} resolves on 07000023");
    }
}
