//! 0700004a (Phoenix Fire 3, the Tower-2 elevator) placement-behaviour pins, added by
//! `level-4a-audit`: the two shaft shooters' arm-edge fire and the 162 doors' unlock map —
//! the two behaviours the audit found present in the port but untested at the placement level.
//!
//! Evidence: the placements are `nf-extract placements-all` rows [D]; the shooter behaviour is
//! `Shooter_Update` 0x17d0c0 (arm channel p0 = obj+0xF4, weapon p1 = obj+0xF6, one
//! `Bullet_init` 0x12c5a0 round, then the delete pass 0x1336f8 removes the object) [M]; the door
//! behaviour is `Door_Create` 0x136100 (p1 → data+0x6A group, p2 → +0x66 approach, p3 → +0x68
//! lock), `Door_Update` 0x1363c8:101–107 (the lock channel's rising edge → `Door_Activate`,
//! `+0x68 = 0`, bit 0x10) and `Doors_Calc` 0x135fe0 (same-group slaves re-pose to the master)
//! [M]. The full tables live in docs/levels/0700004a-audit.md §1/§4.
//!
//! Opt-in: `NF_ISO`.

use nf_formats::iso9660::IsoImage;
use nf_formats::level::Level;
use nf_formats::weapon::WeaponData;
use nf_formats::{disc, levelbin::LevelBin};

fn load_4a(iso: &IsoImage<std::fs::File>) -> (Level, WeaponData) {
    let (entries, files) = disc::files_bin_entries(iso).unwrap();
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case("0700004a.bin")).unwrap();
    let data = iso.read_bytes(files.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).unwrap();
    let level = Level::parse(&data).unwrap();
    let _lb = LevelBin::parse(&data).unwrap();
    let elf = iso.read_file(iso.find("/ACTION.ELF").unwrap()).unwrap();
    let weapons = WeaponData::from_action_elf(&elf).unwrap();
    (level, weapons)
}

/// The two `ShootyBangBang` placements (#286/#287 [D census]): the arm channel's rising tick
/// fires exactly one weapon-61 bullet at the placement's position, and never a second one —
/// the game's delete pass removes the shooter that tick [M 0x17d0c0 tail + 0x1336f8:86-87].
#[test]
fn the_shaft_shooters_fire_once_on_their_arm_channel() {
    let Ok(path) = std::env::var("NF_ISO") else { eprintln!("NF_ISO not set — skipping"); return };
    let iso = IsoImage::open(&path).unwrap();
    let (level, weapons) = load_4a(&iso);
    let m = level.main_map().unwrap();
    let paths = level.path_blocks();
    let world = nf_game::worldobj::WorldObjects::create(&level.maps[m].placements, &paths, |_| (0, [0.0; 3]));
    assert_eq!(world.shooters.len(), 2, "the [D] census: two 0xd2 placements");
    // #286 (3.0446, -48.3993, -1.6714) arms on ch 0x22 = 34 (the lift SP's out [D placement]);
    // #287 (2.0935, -117.7803, -1.9136) arms on ch 0x4f = 79 (plate #281 [D placement]).
    let s286 = &world.shooters[0];
    let s287 = &world.shooters[1];
    assert_eq!(s286.arm, 34, "[D] #286 p0");
    assert_eq!(s287.arm, 79, "[D] #287 p0");
    assert_eq!(s286.weapon, 61, "[D] #286 p1 = weapon record 61");
    assert_eq!(s287.weapon, 61, "[D] #287 p1 = weapon record 61");
    for (k, want) in [(0usize, 3.044577f32), (1, -48.399326), (2, -1.671356)] {
        assert!((s286.pos[k] - want).abs() < 1e-4, "#286 pos[{k}] = {} want {want}", s286.pos[k]);
    }
    let mut ch = nf_game::triggers::Channels::default();
    let mut rng = nf_game::drone::Rng::new(7);
    let mut bullets: Vec<nf_game::bullet::Bullet> = Vec::new();
    // No arm channel: the update returns without firing [M 0x17d0c0:8-10].
    let mut shooters = world.shooters.clone();
    shooters[0].tick(&ch, &weapons, &mut rng, &mut bullets);
    assert!(bullets.is_empty(), "ch 0x22 off → no bullet");
    // The arm: one bullet, weapon 61, from the placement's position [M 0x17d0c0:11-27].
    ch.set(34, 1, 0);
    shooters[0].tick(&ch, &weapons, &mut rng, &mut bullets);
    assert_eq!(bullets.len(), 1, "the 0x22 edge fires #286 once");
    assert_eq!(bullets[0].weapon, 61);
    for (k, want) in [(0usize, 3.044577f32), (1, -48.399326), (2, -1.671356)] {
        assert!((bullets[0].pos[k] - want).abs() < 1e-3, "bullet pos[{k}] = {}", bullets[0].pos[k]);
    }
    // The other shooter stays silent on 0x22 (its arm is 0x4f) [D placements].
    shooters[1].tick(&ch, &weapons, &mut rng, &mut bullets);
    assert_eq!(bullets.len(), 1, "#287 not armed by ch 0x22");
    // The fired shooter never fires again [M the obj+0xFE bit 0 delete pass].
    shooters[0].tick(&ch, &weapons, &mut rng, &mut bullets);
    assert_eq!(bullets.len(), 1, "one bullet per placement");
}

/// The door unlock map [D census — 6 of the 81 pairs carry p3; the other 75 are sealed]:
/// the six unlock channels open exactly their pair (twin included, `Doors_Calc`); the sealed
/// pairs stay shut on the same channels; the spawner channels (0x17 etc.) open no door.
#[test]
fn the_door_unlock_map_opens_exactly_the_six_fight_floor_pairs() {
    let Ok(path) = std::env::var("NF_ISO") else { eprintln!("NF_ISO not set — skipping"); return };
    let iso = IsoImage::open(&path).unwrap();
    let (level, _weapons) = load_4a(&iso);
    let paths = level.path_blocks();
    let m = level.main_map().unwrap();
    let mut doors = nf_game::door::Doors::create(&level.maps[m].placements, &paths, |_| None);
    assert_eq!(doors.list.len(), 162, "[D] 162 door placements");
    // Every 4a door: no approach channel (p2 = 0) and class word 2 (p8/p9 = 1/2) — remote-only.
    assert!(doors.list.iter().all(|d| d.ch_open == 0), "[D] p2 = 0 on all 162");
    // The unlock map, exactly [D census §1]:
    let want: [(usize, u16, u16); 12] = [
        (88, 13, 61), (89, 13, 61), (110, 25, 64), (111, 25, 64), (128, 33, 63), (129, 33, 63),
        (140, 39, 69), (141, 39, 69), (204, 51, 62), (205, 51, 62), (220, 59, 79), (221, 59, 79),
    ];
    let got: Vec<(usize, u16, u16)> = doors.list.iter()
        .filter(|d| d.ch_lock != 0)
        .map(|d| (d.placement, d.group, d.ch_lock)).collect();
    assert_eq!(got.len(), want.len(), "exactly 12 doors carry p3");
    for (w_pl, w_grp, w_ch) in want {
        let d = doors.list.iter().find(|d| d.placement == w_pl).expect("placement present");
        assert_eq!((d.group, d.ch_lock), (w_grp, w_ch), "placement {w_pl} unlock map");
    }
    let idx = |doors: &nf_game::door::Doors, pl: usize| doors.list.iter().position(|d| d.placement == pl).unwrap();
    let no_actors: &[nf_game::door::DoorActor] = &[];
    // The -52 pair (#110/#111, group 25) unlocks on ch 0x40 — the lift NIS's t1610 write
    // [D 0x06000571 s5; M 0x1363c8:101-107]. Snapshot the off state, set the channel, snapshot
    // again (end_tick: prev ← hold ← channels), then run the tick.
    let run_tick = |doors: &mut nf_game::door::Doors, ch: &mut nf_game::triggers::Channels, frame: u32| {
        doors.begin_pass();
        for i in 0..doors.list.len() { doors.update(i, ch, no_actors, frame); }
        doors.calc();
        doors.end_tick(ch);
    };
    let mut ch = nf_game::triggers::Channels::default();
    run_tick(&mut doors, &mut ch, 0);
    assert!(doors.list[idx(&doors, 110)].is_closed(&doors.group_states) && doors.list[idx(&doors, 111)].is_closed(&doors.group_states));
    ch.set(64, 1, 1);
    run_tick(&mut doors, &mut ch, 1);
    assert!(!doors.list[idx(&doors, 110)].is_closed(&doors.group_states), "#110 opens on the ch 0x40 edge");
    assert!(!doors.list[idx(&doors, 111)].is_closed(&doors.group_states), "#111 opens with its twin [M 0x135fe0]");
    assert!(doors.list[idx(&doors, 110)].ch_lock == 0 && doors.list[idx(&doors, 111)].ch_lock == 0,
        "the game clears +0x68 on the unlock [M 0x1363c8:105-107]; bit 0x10 was set during the update and calc clears it the same tick [M 0x135fe0 tail]");
    // A sealed pair (#106/#107, y -48, group 23, p3 = 0) on the SAME channel: stays shut [M the
    // unlock block needs p3 non-zero; the approach block needs p2 non-zero — both are 0].
    assert!(doors.list[idx(&doors, 106)].is_closed(&doors.group_states) && doors.list[idx(&doors, 107)].is_closed(&doors.group_states),
        "the sealed -48 pair ignores ch 0x40");
    // The spawner channels (0x17 = 23 — spawner #233's start [D placement]) open no door: no door
    // carries 23 as its unlock channel [D census].
    assert!(doors.list.iter().all(|d| d.ch_lock != 23 && d.ch_open != 23), "[D] ch 0x17 is no door's channel");
    let mut doors2 = nf_game::door::Doors::create(&level.maps[m].placements, &paths, |_| None);
    let mut ch2 = nf_game::triggers::Channels::default();
    run_tick(&mut doors2, &mut ch2, 0);
    ch2.set(23, 1, 1);
    run_tick(&mut doors2, &mut ch2, 1);
    assert!(doors2.list.iter().all(|d| d.is_closed(&doors2.group_states)), "ch 0x17 opens nothing");
}
