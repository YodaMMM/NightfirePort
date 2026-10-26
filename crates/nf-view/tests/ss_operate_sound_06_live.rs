//! The SS operate arm's own sample (level-06-audit): `SS_Update`'s 0x40 -> 1 transition plays
//! `Sound_Play3D(…, ext+0x14, obj+0x80)` [M 0x1a9948:86-89]. 07000006's barrel traps carry the
//! sound word: #353/#354/#389/#390 p4 = 0x11b (283), the rest 0xffff [D census]. The port emits
//! (sample, position) into `SsList::pending_sounds` on the arm and the Drones tick drains it into
//! the audio sink (drone.rs, the lock's pending_sounds routing). Opt-in: NF_ISO.

use nf_formats::{disc, iso9660::IsoImage, level::Level};

fn level(iso: &str, name: &str) -> Level {
    let img = IsoImage::open(iso).expect("iso");
    let (entries, bin) = disc::files_bin_entries(&img).expect("FILES.BIN");
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case(name)).expect("archive");
    let data = img.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).expect("read");
    Level::parse(&data).expect("parse")
}

#[test]
fn ss_operate_emits_the_trap_sample() {
    let Ok(iso) = std::env::var("NF_ISO") else { eprintln!("NF_ISO not set — skipping"); return };
    let lv = level(&iso, "07000006.bin");
    let mm = lv.main_map().unwrap();
    let pb: Vec<&[u8]> = lv.path_blocks();
    let mut ss = nf_game::ss::SsList::create(&lv.maps[mm].placements, &pb, |_p| (0, 0.0));

    // The disc's four sound-carrying traps [D census #353/#354/#389/#390 p4 = 0x11b].
    let traps: Vec<usize> = ss.list.iter().enumerate()
        .filter(|(_, s)| s.sound == 0x11b)
        .map(|(i, _)| i).collect();
    assert_eq!(traps.len(), 4, "the census's four 0x11b barrel traps exist");
    // ... and everything else armed on this level carries no sample.
    assert!(ss.list.iter().enumerate().filter(|(i, _)| !traps.contains(i))
        .all(|(_, s)| s.sound == 0xffff || s.sound == 0), "no stray operate samples");

    // Drive trap #353's operate through the channel half [M 0x1a9df8: (mask & contacts)==0 ->
    // the channel word]: mask 1 matches no contact class, so the channel path is the trap's arm.
    let k = traps.iter().copied().find(|&i| {
        let p = &lv.maps[mm].placements[ss.list[i].placement];
        p.translation[0] > 6.0 && p.translation[0] < 7.0 && p.translation[2] > 2.0 && p.translation[2] < 3.0
    }).expect("the west-cluster trap #353");
    assert_eq!(ss.list[k].channel, 0xe, "#353's out word is ch 14 [D]");
    assert_eq!(ss.list[k].state, 0x40, "armed at create [M 0x1a9c58]");
    let pos0 = [ss.list[k].matrix[0][3], ss.list[k].matrix[1][3], ss.list[k].matrix[2][3]];

    let mut ch = nf_game::triggers::Channels::default();
    ch.set(0xe, 1, 0);
    ss.tick(&mut ch, &vec![0u8; ss.list.len()], 1);

    // BOTH west-cluster traps #353/#354 share ch 14 and the 0x11b sample [D census] — one tick
    // arms both; the 0xffff sisters (#355-365) operate silently on the same channel.
    assert_eq!(ss.pending_sounds.len(), 2, "the two 0x11b west traps fired on ch 14");
    assert!(ss.pending_sounds.iter().all(|(id, _)| *id == 0x11b), "sample 283 [D census]");
    assert!(ss.pending_sounds.iter().all(|(_, p)| (0..3).all(|a| (p[a] - pos0[a]).abs() < 0.01)),
        "the position is the object's own matrix translation [M 0x1a9948 obj+0x80]");
    assert_eq!(ss.list[k].state, 1, "latched to playing [M 0x1a9948's 0x40 arm]");

    // The east cluster (#389/#390, ch 0x2b) adds its two samples; the silent traps add none.
    let mut ch2 = nf_game::triggers::Channels::default();
    ch2.set(0x2b, 1, 0);
    ss.tick(&mut ch2, &vec![0u8; ss.list.len()], 2);
    assert_eq!(ss.pending_sounds.len(), 4, "the east pair completes the census's four");
    assert_eq!(ss.list.iter().filter(|s| s.channel == 0xe && s.state == 1).count(), 13,
        "the whole west cluster latched (2 with sound, 11 silent) [D census]");
}
