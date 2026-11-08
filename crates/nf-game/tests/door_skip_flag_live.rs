//! door-scan-skip-bit: the `obj+0xF0` bit 0x10 the `Door_Update` auto-close scan skips is now
//! carried per body. The player's boot value per level (`door::player_door_skip` — the
//! `Control_SetGList` 0x134960:12-13 copy of the level celglist word, reached from `Player_Init`
//! 0x1910f8:115): SET on 07000001 (obj+0xF0 = 0x890), CLEAR on 07000005 (0x880) [L
//! ee_start_07000001.bin / ee_start_05.bin]. The drones' value per the enable lifecycle
//! (`NDrone2_DefaultInit` 0x14b300:606 clears 0x10|0x20 at spawn; `NDrone2_Enable(0,…)`
//! 0x149dd0:12-13 sets it for every waiting/disabled drone): the 07000001 boot drones read 0x8B0
//! [L]. The door behaviour pin runs the real 07000001 doors through the same scan the game runs
//! (`Doors::update`'s auto-close, `Door_Update` 0x1363c8's timer arm [M 0x13660c-0x136700]).
//! Opt-in: NF_ISO.

use nf_formats::iso9660::IsoImage;
use nf_formats::{disc, level::Level};
use nf_game::door::{self, Doors};
use nf_game::triggers::Channels;

fn load(code: u32) -> Option<Level> {
    let path = std::env::var_os("NF_ISO")?;
    let iso = IsoImage::open(&path).unwrap();
    let (entries, bin) = disc::files_bin_entries(&iso).unwrap();
    let name = format!("{code:08x}.bin");
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case(&name)).unwrap();
    let data = iso.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).unwrap();
    Some(Level::parse(&data).unwrap())
}

/// door.rs's live-sphere read (`transform(&d.matrix, d.sph_centre)`, the door's world matrix from
/// the last interp [M 0x1336f8:209], with the `obj+0x8C` radius).
fn live_sphere(d: &nf_game::door::Door) -> ([f32; 3], f32) {
    let m = &d.matrix;
    let mut o = [m[3][0], m[3][1], m[3][2]];
    for k in 0..3 { for c in 0..3 { o[c] += d.sph_centre[k] * m[k][c]; } }
    (o, d.sph.1)
}

/// The real 07000001 doors (`Doors::create` over the main map's placements, the same build the
/// sim's tick uses).
fn doors_01() -> Option<Doors> {
    let level = load(0x0700_0001)?;
    let m = level.main_map().unwrap();
    let ps = &level.maps[m].placements;
    let blocks = level.path_blocks();
    Some(Doors::create(ps, &blocks, |p| {
        let o = if p.by_hash() { level.by_hash.get(&p.hashcode).map(|&(om, oi)| &level.maps[om].objects[oi]) }
                else { level.maps[m].objects.get(p.index as usize) };
        o.map(|o| (o.hashcode, o.centre, o.radius, o.flags))
    }))
}

/// Open one auto-close-capable door (no keep-open flag) with the activator far away, and leave it
/// OPEN. Returns (door index, live sphere of the OPEN leaf).
fn open_a_door(doors: &mut Doors) -> (usize, ([f32; 3], f32)) {
    let i = doors.list.iter().position(|d| d.flags & 0x20 == 0)
        .expect("07000001 has a door without the keep-open flag");
    let mut ch = Channels::default();
    let mut sounds = Vec::new();
    let far = [([0.0f32, 0.0, 0.0], 0.4f32, false); 0]; // nobody near: the open proceeds
    assert!(doors.activate(i, &far, true, &mut ch, 0, &mut sounds), "door {i} opens on activate");
    doors.calc();
    let mut guard = 0;
    while doors.list[i].state != door::OPEN && guard < 60 {
        sounds.extend(doors.update(i, &mut ch, &[], guard)); doors.calc(); guard += 1;
    }
    assert_eq!(doors.list[i].state, door::OPEN, "door {i} reached OPEN in {guard} ticks");
    let sph = live_sphere(&doors.list[i]);
    (i, sph)
}

/// The per-level pin: the player's boot flag matches the live boards — SET on 07000001
/// (obj+0xF0 = 0x890 = 0x800|0x80|0x10 [L ee_start_07000001.bin]), CLEAR on 07000005 (0x880
/// [L ee_start_05.bin]). Other levels keep the 07000005 model (the only-evidenced default;
/// [S] unverified for every board except these two).
#[test]
fn player_door_skip_matches_the_live_boards() {
    assert!(door::player_door_skip(0x0700_0001), "07000001 player obj+0xF0 = 0x890: bit 0x10 SET [L]");
    assert!(!door::player_door_skip(0x0700_0005), "07000005 player obj+0xF0 = 0x880: bit 0x10 CLEAR [L]");
    // The 07000005 model everywhere else until a board says otherwise.
    assert!(!door::player_door_skip(0x0700_0014));
    assert!(!door::player_door_skip(0x0700_0016));
}

/// The behaviour pin: on 07000001 (player flag SET) the auto-close scan ignores Bond standing in
/// the open doorway and closes the door on him [M 0x136678-0x136684 + 0x13660c-0x136700]; the
/// 07000005 model (flag CLEAR) keeps the door open while he blocks it. This is the change the
/// flag makes on 07000001: the port used to hold every door open under him.
#[test]
fn on_07000001_the_auto_close_ignores_bond_in_the_doorway() {
    let Some(mut doors) = doors_01() else { eprintln!("skip: NF_ISO not set"); return };
    let (i, (centre, r)) = open_a_door(&mut doors);
    let mut ch = Channels::default();
    // 07000001: the player (flag SET — the 0x890 board [L]) stands in the OPEN leaf's sphere; the
    // scan skips him, so the 360-tick timer closes the door.
    let bond_01 = [(centre, 0.4f32, true)];
    let mut guard = 0;
    while doors.list[i].state == door::OPEN && guard < 420 {
        doors.update(i, &mut ch, &bond_01, guard); doors.calc(); guard += 1;
    }
    assert_ne!(doors.list[i].state, door::OPEN,
        "07000001: the door must auto-close through Bond (the scan skips his 0x10-set obj) — {guard} ticks, sphere r={r}");
    // 07000005 (the sister model): the same body with the flag CLEAR blocks the scan; the door
    // stays open (the timer resets every 360 without closing).
    let mut doors = doors_01().unwrap();
    let (i, (centre, _)) = open_a_door(&mut doors);
    let bond_05 = [(centre, 0.4f32, false)];
    let mut guard = 0;
    while guard < 420 {
        doors.update(i, &mut ch, &bond_05, guard); doors.calc(); guard += 1;
        assert_eq!(doors.list[i].state, door::OPEN,
            "07000005: Bond (obj+0xF0 = 0x880, bit CLEAR [L]) blocks the auto-close");
    }
}

/// The drone side of the pin [M NDrone2_DefaultInit 0x14b300:606 clears 0x10|0x20 at spawn,
/// NDrone2_Enable(0,…) 0x149dd0:12-13 sets it while waiting/disabled]: an unspawned drone (the
/// port's `started == false`, the 07000001 boot state — obj+0xF0 = 0x8B0 [L]) does NOT hold a
/// door open; a started one does.
#[test]
fn unspawned_drones_do_not_hold_doors_open_started_ones_do() {
    let Some(mut doors) = doors_01() else { eprintln!("skip: NF_ISO not set"); return };
    let (i, (centre, _)) = open_a_door(&mut doors);
    let mut ch = Channels::default();
    // Only bodies in the doorway are the player (07000001 flag SET) and an unstarted drone: the
    // scan sees nobody — the door closes. This is the 07000001 boot board: player 0x890, drones
    // 0x8B0 [L], every drone still waiting on its enable channel.
    let waiting = [(centre, 0.4f32, true), (centre, 0.4f32, true)];
    let mut guard = 0;
    while doors.list[i].state == door::OPEN && guard < 420 {
        doors.update(i, &mut ch, &waiting, guard); doors.calc(); guard += 1;
    }
    assert_ne!(doors.list[i].state, door::OPEN,
        "the unspawned drone (0x8B0 [L]) must not hold the door open — {guard} ticks");
    // Sister: the same drone after its spawn (DefaultInit cleared the bit [M 0x14b300:606] — the
    // port's `started`) blocks the scan; the door stays open.
    let mut doors = doors_01().unwrap();
    let (i, (centre, _)) = open_a_door(&mut doors);
    let spawned = [(centre, 0.4f32, false)];
    let mut guard = 0;
    while guard < 420 {
        doors.update(i, &mut ch, &spawned, guard); doors.calc(); guard += 1;
        assert_eq!(doors.list[i].state, door::OPEN,
            "a started drone (bit cleared at spawn [M 0x14b300:606]) blocks the auto-close");
    }
}
