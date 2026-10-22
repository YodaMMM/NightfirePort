//! Double-Cross (0700) objective items through the real path. Opt-in: `NF_ISO`.
//!
//! The items [D placements/scripts via the level bins]:
//! - **07000003 "Retrieve Dragon Safe contents" (obj ch 0x4E)**: script 0x06000563 stream 1 spawns a
//!   `Pheonix Keycard` pickup — `Script_CreateEntity` 0x1d8968's obj_type-0xF0 arm → `Pickup_Create`
//!   0x18dda0 — with params [kind 2, id 0, rounds 0, **switch 0x4E**, sound 0xFFFF, respawn 0] [D the
//!   level bin's script decode]. The script player (0xD9 placement, mode 0x40, **mask 0x4**) sits at the
//!   glass display case (6.70, 6.14, −42.55): the use key activates it [M `Player_Activate` 0x1a5170
//!   :121-122 dispatches `SP_Activate` 0x1dc338 on the case's class-0 object; the SP's mask bit 2 is the
//!   activation trigger [M `SP_Update` 0x1dbda8 trigger flags]].
//! - The take: kind 2 never refuses (`Player_EquipKey`, [M `Pickup_Handler` 0x18e650:217-219]) and the
//!   take tail writes `switch_channels[+0x28] = 1` [M 0x18e650:330-333] — ch 0x4E is the objective's
//!   watch channel [M `Mission_MonitorObjectives` 0x1827d8:32-38], so the objective completes.
//! - **Stages 01/02/04 have NO objective-item pickups**: the 0xF0 placements are armour/007-bonus/ammo/
//!   weapons and the scripts spawn only props/particles (no obj_type-0xF0 Entity) [D the census in this
//!   file]. The keycard is the only objective item on 01–04.

use nf_formats::iso9660::IsoImage;

fn iso() -> Option<String> { std::env::var_os("NF_ISO").map(|v| v.to_string_lossy().into_owned()) }

fn load(iso_v: &str, archive: &str) -> nf_view::scene::Scene {
    nf_view::load(iso_v, archive, false).expect(archive)
}

/// The case use through the real walk: the F12 playtest spot (07000003 --bond 5.90,5.97,-42.41,
/// 1.710,-0.458) faces the display case; one E press activates the case's script player [M 0x1a5170],
/// the script reaches the Entity command and the keycard goes live [M 0x1d8968's 0xF0 arm].
#[test]
fn the_case_use_spawns_the_keycard_through_the_real_path() {
    let Some(iso_v) = iso() else { eprintln!("skip: NF_ISO not set"); return };
    let mut scene = load(&iso_v, "07000003.bin");
    let sim = scene.sim.as_mut().expect("sim");
    for _ in 0..30 { sim.step_bond(Default::default(), false, false, false); }
    let before: Vec<bool> = sim.drones.pickups.iter().filter(|p| p.kind == 2).map(|p| p.gone).collect();
    assert_eq!(before, vec![true], "the keycard reservation starts inert (gone) [port fix-safe08 stand-in]");
    assert!(sim.drones.script_pickup_slots.contains_key(&(0x0600_0563, 1)), "the script spawn has a reserved slot [D script 0x06000563 s1]");

    sim.bond.pos = [5.90, 5.97, -42.41];
    sim.bond.yaw = 1.710;
    sim.bond.pitch = -0.458;
    sim.step_bond(Default::default(), false, false, true); // the E press on the case
    // Stop on the creation tick: the stream re-poses the card from then on (dragon-safe-follow).
    for _ in 0..120 {
        sim.step_bond(Default::default(), false, false, false);
        if sim.drones.pickups.iter().any(|p| p.kind == 2 && !p.gone) { break; }
    }

    let live: Vec<&nf_game::pickup::Pickup> = sim.drones.pickups.iter().filter(|p| p.kind == 2 && !p.gone).collect();
    assert_eq!(live.len(), 1, "exactly ONE live keycard: the game's spawn creates one object [M Pickup_Create 0x18dda0]");
    let k = live[0];
    assert_eq!((k.kind, k.weapon, k.switch_channel), (2, 0, 0x4e), "kind 2 (key), id 0, switch ch 0x4E [D script 0x06000563 s1 params]");
    let d = ((k.m[3][0] - 6.61).powi(2) + (k.m[3][1] - 5.92).powi(2) + (k.m[3][2] + 42.56).powi(2)).sqrt();
    assert!(d < 0.1, "the keycard spawns at the case interior (its creation pose), at ({:.2},{:.2},{:.2})", k.m[3][0], k.m[3][1], k.m[3][2]);
}

/// The user's report ("still can't grab the dragon objective in double cross"), the real path: from
/// the playtest spot the E press opens the case; the script's stream 0 raises the case's riser and
/// stream 1 (parented to it) lifts the keycard — the created pickup IS the stream's entity [M 0x1d8968
/// returns the `Pickup_Create` object; 0x1d8d78:96 stores it at stream+0x10], re-posed every tick by
/// `Script_Interp` 0x1d9a08:94-110 (y 5.51 → 6.05 [D 0x06000563 s0/s1 keys]) with its sphere rebuilt
/// by the movement pass (0x1336f8, the +0xFA 0x20 flag `Script_Interp` sets). Lifted above the
/// counter's rim (y 6.00) the take's line of sight [M 0x18e148:210-214 → 0x1edf50] clears it once
/// Bond steps back from the counter; the take writes ch 0x4E [M 0x18e650:330-333].
/// Fail-first: frozen at its creation pose (y 5.92) no spot on the counter's front side ever saw
/// the card (the counter's front face, x 6.446, blocked every ray).
#[test]
fn the_case_use_lifts_the_keycard_and_bond_takes_it() {
    let Some(iso_v) = iso() else { eprintln!("skip: NF_ISO not set"); return };
    let mut scene = load(&iso_v, "07000003.bin");
    let sim = scene.sim.as_mut().expect("sim");
    for _ in 0..30 { sim.step_bond(Default::default(), false, false, false); }
    sim.bond.pos = [5.90, 5.97, -42.41];
    sim.bond.yaw = 1.710;
    sim.bond.pitch = -0.458;
    sim.step_bond(Default::default(), false, false, true); // the E press on the case
    for _ in 0..130 { sim.step_bond(Default::default(), false, false, false); }
    let k = sim.drones.pickups.iter().find(|p| p.kind == 2 && !p.gone).expect("the keycard is live");
    let live_pos = [k.m[3][0], k.m[3][1], k.m[3][2]];
    assert!(k.m[3][1] > 6.04, "the stream lifted the keycard to its last key (y 6.05); it sits at y {:.3}", k.m[3][1]);
    // The draw: the keycard's script slot (mesh 0x020005F1) sits at the live pickup's pose.
    let slot = scene.attachments.iter().position(|a| matches!(a.script, Some((0x0600_0563, 1, 0x0200_05F1, 0)))).expect("the keycard's script draw slot");
    let opts = nf_view::scene::ViewOpts { reinforcements: false, firing: false, right: [1.0, 0.0, 0.0], up: [0.0, 1.0, 0.0] };
    let m = scene.frame_state(None, 0.0, opts).attach[slot];
    assert!((0..3).all(|j| (m[3][j] - live_pos[j]).abs() < 1e-3), "the slot draws the card at its live pose {live_pos:?}; got {:?}", m[3]);
    let sim = scene.sim.as_mut().expect("sim");
    // At the counter the card is NOT taken: the counter's rim sits on the line from Bond's sphere
    // centre to the card [L PCSX2 2026-10-22, audits/dragon-live: Square at (5.896, 5.967, -42.41)
    // yaw 1.71, ch 0x4E still 0 four seconds later; the first back-step (x 5.55) took it].
    for _ in 0..60 { sim.step_bond(Default::default(), false, false, false); }
    assert_eq!(sim.drones.channels.v[0x4e], 0, "standing at the counter the take's LOS is blocked, as in the game [L]");
    // Walk back from the counter (the case faces +x; Bond backs off along −x).
    let back = nf_game::player::Input { forward: -1.0, ..Default::default() };
    let mut taken_at = None;
    for t in 0..120 {
        sim.step_bond(back.clone(), false, false, false);
        if sim.drones.channels.v[0x4e] != 0 { taken_at = Some(t); break; }
    }
    assert!(taken_at.is_some(), "backing off the counter, Bond takes the keycard and ch 0x4E goes on (Bond at {:?})", sim.bond.pos);
    assert!(sim.drones.pickups.iter().filter(|p| p.kind == 2).all(|p| p.gone), "the taken keycard is gone");
    // Taken = deleted in the game (respawn 0): nothing of it stays drawn at the case.
    let m = scene.frame_state(None, 0.0, opts).attach[slot];
    assert!(m[3][1] < -1000.0, "the taken keycard's draw collapses; it sits at {:?}", m[3]);
}

/// Sister: no E press — the keycard never goes live.
#[test]
fn the_keycard_stays_inert_without_the_case_use() {
    let Some(iso_v) = iso() else { eprintln!("skip: NF_ISO not set"); return };
    let mut scene = load(&iso_v, "07000003.bin");
    let sim = scene.sim.as_mut().expect("sim");
    sim.bond.pos = [5.90, 5.97, -42.41];
    sim.bond.yaw = 1.710;
    for _ in 0..150 { sim.step_bond(Default::default(), false, false, false); }
    assert!(sim.drones.pickups.iter().filter(|p| p.kind == 2).all(|p| p.gone), "no use, no spawn");
    assert_eq!(sim.drones.channels.v[0x4e], 0, "ch 0x4E untouched");
}

/// The take: kind 2 never refuses [M 0x18e650:217-219] and the take tail writes the switch channel
/// [M 0x18e650:330-333] — ch 0x4E completes "Retrieve Dragon Safe contents" through the mission
/// monitor [M 0x1827d8:32-38; the record table 0x2a3f18].
#[test]
fn taking_the_keycard_sets_the_objective_channel_and_the_monitor_completes_it() {
    let Some(_iso_v) = iso() else { eprintln!("skip: NF_ISO not set"); return };
    // The take, at the inventory level (the geometry-independent half of Pickup_Handler).
    let w = nf_formats::weapon::WeaponData { records: Vec::new(), ammo_cap: Vec::new(), ammo_raw: Vec::new() };
    let mut inv = nf_game::pickup::Inventory::empty(0x73, 34);
    let m = [[1.0, 0.0, 0.0, 0.0], [0.0, 1.0, 0.0, 0.0], [0.0, 0.0, 1.0, 0.0], [0.0; 4]];
    let p = nf_game::pickup::Pickup {
        kind: 2, weapon: 0, rounds: 0, model: 0,
        bounds: nf_game::pickup::Model { centre: [0.0; 3], radius: 0.5 },
        m, start: [0.0; 3], vel: [0.0; 3], state: 1, lifetime: 0, switch_channel: 0x4e,
        sound: None, hidden: false, gone: false, from: None, obj: None, respawn: 0,
        respawn_ticks: 0, spin: false, label: 0,
    };
    let mut armour = 0.0f32;
    assert_eq!(inv.take(&w, &p, false, &mut armour, 1), Some(nf_game::pickup::Taken::Key { id: 0 }),
        "kind 2 is never refused [M 0x18e650:217-219]");
    assert!(inv.keys[0], "BLData+0x70C+0 = 1 [M Player_EquipKey 0x1a97a0]");
    // The mission half: ch 0x4E on (what the take tail writes) completes the objective.
    let mut ch = nf_game::triggers::Channels::default();
    let mut mission = nf_game::mission::Mission::new();
    let mut out = Vec::new();
    mission.init(0x0700_0003, false, &mut ch, 0);
    ch.set(0x4d, 1, 1); // the record's reveal (enable) channel [D the mission table: en 0x4D]
    mission.update(&mut ch, 2, 0x0700_0003, false, &mut out);
    ch.set(0x4e, 1, 1);
    mission.update(&mut ch, 3, 0x0700_0003, false, &mut out);
    // The record's live state [M 0x1827d8's state machine]: LEVELS order, 07000003's record
    // 0x0400000a. With ch 0x4E on (what the take writes) the record completes (state 4).
    let (li, oi) = nf_game::mission::LEVELS.iter().enumerate()
        .find_map(|(li, l)| l.objectives.iter().enumerate()
            .find(|(_, o)| o.text == 0x0400_000a).map(|(oi, _)| (li, oi)))
        .expect("the mission table holds 'Retrieve Dragon Safe contents' [D]");
    assert_eq!(mission.states[li][oi], 4, "ch 0x4E completes it (state 4) [M 0x1827d8]; got {}",
        mission.states[li][oi]);
    // Sister: without the take's channel the record stays pending (state < 4).
    let mut ch2 = nf_game::triggers::Channels::default();
    let mut m2 = nf_game::mission::Mission::new();
    let mut out2 = Vec::new();
    m2.init(0x0700_0003, false, &mut ch2, 0);
    ch2.set(0x4d, 1, 1);
    m2.update(&mut ch2, 2, 0x0700_0003, false, &mut out2);
    assert!(m2.states[li][oi] < 4, "no ch 0x4E, no completion; got {}", m2.states[li][oi]);
}

/// The sweep: stages 01/02/04 have NO objective-item pickups — every 0xF0 placement's switch channel
/// and every script 0xF0 spawn sits outside the stages' objective watch channels [D the census].
/// Sister: 07000003's script spawn carries exactly the objective channel 0x4E.
#[test]
fn the_objective_item_census_of_stages_01_04() {
    let Some(iso_v) = iso() else { eprintln!("skip: NF_ISO not set"); return };
    let i = IsoImage::open(&iso_v).unwrap();
    let (entries, bin) = nf_formats::disc::files_bin_entries(&i).unwrap();
    // The objective watch channels per stage [M the mission table 0x2a3f18, docs/systems/objectives-all-levels.md].
    let obj_ch: &[(&str, &[u8])] = &[
        ("07000001.bin", &[0x47]),
        ("07000002.bin", &[0x48, 0x4b, 0x4d]),
        ("07000003.bin", &[0x4e, 0x51, 0x53]),
        ("07000004.bin", &[0x54]),
    ];
    for (stage, chans) in obj_ch {
        let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case(stage)).unwrap();
        let data = i.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).unwrap();
        let level = nf_formats::level::Level::parse(&data).unwrap();
        let mut item_spawns: Vec<u8> = Vec::new();
        // Script spawns: every stream's obj_type-0xF0 Entity's switch word [D].
        let mut hashes: Vec<u32> = level.nis.keys().copied().collect();
        hashes.sort_unstable();
        for hash in hashes {
            for st in &level.nis[&hash].streams {
                for c in st.decode() {
                    if let nf_formats::nis::Cmd::Entity { obj_type: 0xf0, params, .. } = c {
                        let sw = params.get(3).copied().unwrap_or(0) as u8;
                        item_spawns.push(sw);
                    }
                }
            }
        }
        if *stage == "07000003.bin" {
            assert_eq!(item_spawns, vec![0x4e], "the keycard is the level's only script pickup, switch 0x4E [D]");
        } else {
            assert!(item_spawns.iter().all(|sw| !chans.contains(sw)),
                "{stage}: no script pickup writes an objective channel [D] — its objective (if any) completes elsewhere");
        }
        // Placed pickups: kind-2/4 (key/label) would be objective items; 01-04 have none [D].
        let mm = level.main_map().unwrap_or(0);
        for p in &level.maps[mm].placements {
            if p.flags & 0xff != 0xf0 { continue; }
            let kind = p.params.iter().find(|(i, _)| *i == 0).map(|(_, v)| *v).unwrap_or(0);
            assert!(kind != 2 && kind != 4, "{stage}: placement pickup kind {kind} would be an objective item");
        }
    }
}

