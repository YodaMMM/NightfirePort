//! The one-sided wall's update (`OneSided_Update` 0x1e7d58, class 0x4C) against the disc's own
//! placement. The update-column census (update_column_census.rs) puts exactly one type-0xd5
//! placement on the disc, on 07000041 [D census] — and on a sub-map (map 99, "01000058"), not
//! the main map. The port's scene.rs comment used to claim the update's job was "the port's
//! triangle cull", but every wgpu pipeline runs `cull_mode: None` [M gpu.rs] — the update had no
//! port at all. Ported here: `worldobj::OneSided` (create 0x1e7ce8 → the placement pos/rot,
//! update 0x1e7d58 → the per-tick facing flip of obj+0xF0 bit 0x10).
//!
//! Scope note (the hand-off stub): `Drones::create` builds `WorldObjects` from the main map's
//! placements only — the same scope every worldobj class has (drone.rs `WorldObjects::create`
//! call) — so a sub-map 0xd5 placement does not join the port's walk yet; and the hidden flag's
//! draw consumer (the palette FAR-write) is not wired. Both are in the report's hand-off list.
//! This test pins the ported create + tick per map, which is the game's own per-map
//! `parsemap_create_dynamic_objects` granularity [M 0x1d04f0 takes one placements array].
//!
//! Run with NF_ISO set. Opt-in via `NF_ISO`.

use nf_formats::iso9660::IsoImage;
use nf_formats::level::Level;
use nf_view::dev;

#[test]
fn onesided_update_pins_on_07000041() {
    let Some(path) = std::env::var_os("NF_ISO") else { eprintln!("skip: NF_ISO not set"); return };
    let path = path.to_str().unwrap();
    let iso = IsoImage::open(path).unwrap();
    let (entries, bin) = nf_formats::disc::files_bin_entries(&iso).unwrap();
    let name = nf_view::level_archive_name(0x0700_0041);
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case(&name)).expect("no 07000041.bin on the disc");
    let data = iso.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).unwrap();
    let level = Level::parse(&data).unwrap();

    // The census row: 07000041 carries exactly one 0xd5 placement [D update_column_census], on a
    // sub-map — not the main map (index 0).
    let mut where_d5 = None;
    for (mi, m) in level.maps.iter().enumerate() {
        if m.placements.iter().any(|p| p.flags & 0xff == 0xd5) { where_d5 = Some(mi); }
    }
    assert_eq!(where_d5, Some(99), "the one 0xd5 placement sits on sub-map 99 of 07000041 [D]");
    let main = &level.maps[99];
    let d5: Vec<&nf_formats::placement::Placement> = main.placements.iter()
        .filter(|p| p.flags & 0xff == 0xd5)
        .collect();
    assert_eq!(d5.len(), 1, "expected exactly one type-0xd5 placement on 07000041 [D census]");

    // The 0xd5 arm creates the record from that map's placements (red on the old code: no arm, no
    // record — and the master comment claimed a draw-side cull that does not exist).
    let w = nf_game::worldobj::WorldObjects::create(&main.placements, &[], |_| (0, [0.0; 3], 0));
    assert_eq!(w.onesided.len(), 1, "the 0xd5 placement creates one OneSided record [M 0x1e7ce8]");
    assert!(w.onesided[0].placement == d5[0].index as usize, "the record joins back to the placement");

    // The dir is the placement matrix's third column ([M Mat_GetDir 0x1e3398 reads +0x20..+0x28;
    // obj+0x90 = View_RotTransMatrix of the placement rot/trans 0x1344e0:26-29]).
    let m = nf_formats::placement::view_rot_trans_matrix(d5[0].rotation, d5[0].translation);
    for k in 0..3 {
        assert_eq!(w.onesided[0].dir[k], m[k][2], "dir is the matrix's column 2");
    }
    assert_eq!(w.onesided[0].pos, d5[0].translation, "obj+0x30 = the placement translation");

    // The update [M 0x1e7d58]: hidden = dot(dir, pos − viewer) < 0. Step the viewer along the dir
    // through the wall plane: on the +dir side the dot is negative (pos − viewer points against
    // dir) → hidden; far on the −dir side it is positive → shown. Ticks through the walk's
    // WorldObjects::tick, the same call the control walk makes.
    let step = 100.0;
    let os = &w.onesided[0];
    let front = [os.pos[0] + os.dir[0] * step, os.pos[1] + os.dir[1] * step, os.pos[2] + os.dir[2] * step];
    let back = [os.pos[0] - os.dir[0] * step, os.pos[1] - os.dir[1] * step, os.pos[2] - os.dir[2] * step];
    let mut w2 = nf_game::worldobj::WorldObjects::create(&main.placements, &[], |_| (0, [0.0; 3], 0));
    let (mut ch, mut music, mut rng) = (nf_game::triggers::Channels::default(), nf_game::music::Music::default(), nf_game::drone::Rng::new(1));
    let (weapons, mut bullets) = (nf_formats::weapon::WeaponData::default(), Vec::new());
    w2.tick(&mut ch, front, &weapons, &mut rng, &mut bullets, 1, &mut music);
    assert!(w2.onesided[0].hidden, "viewer on the +dir side: dot(dir, pos − viewer) < 0 → obj+0xF0 bit 0x10 set [M 0x1e7d58]");
    w2.tick(&mut ch, back, &weapons, &mut rng, &mut bullets, 2, &mut music);
    assert!(!w2.onesided[0].hidden, "viewer on the −dir side: dot ≥ 0 → the bit clears [M 0x1e7d58]");
    // The flag re-arms every tick (the game rewrites the bit unconditionally each update).
    w2.tick(&mut ch, front, &weapons, &mut rng, &mut bullets, 3, &mut music);
    assert!(w2.onesided[0].hidden, "the flip re-evaluates per tick [M 0x1e7d58 runs every update pass]");
}
