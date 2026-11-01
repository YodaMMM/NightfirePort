//! weapon-handling: every player weapon's and firing gadget's effective handling, from the disc's
//! own weapon_data records (ACTION.ELF 0x2bf150, stride 0x10C) and the level's first-person
//! scripts, run through the port's trigger (`nf_game::gun::Gun` = `Player_WeaponFiring` 0x1a5460)
//! and zoom (`nf_game::gun::zoom_tick` = `Player_Zoom` 0x1a1a70).
//!
//! Per weapon the pin is (the `Gun` alone — the view-state gates are pinned by the Sim test below):
//! the ticks between shots held / tapped (the +0x40 refire timer, re-armed
//! to max(+0x40, 1) per shot [M 0x1a5460:661-663]; a fresh press arms `spp[mode]` shots
//! [M :619-624]), the shots 200 held ticks fire (one pull, refired on +0x40), the clip (+0x92),
//! the reload ticks (the +0xA8 start script, else the +0xA4 script at 1.35× from frame 6 for
//! +0x68 & 0x1000 [M :463-486]), the zoom max (+0x88) and the held-zoom ticks from 1× to it
//! (×1.025 per tick [M 0x1a1a70:34-41]). Opt-in via `NF_ISO`.

use nf_game::gun::{Gun, GunParams};

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

/// The player weapons and firing gadgets (selectable records, +0x04 == 1, with a shot) — the base
/// ids and their twins.
const WEAPONS: &[u8] = &[
    0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f, 0x10, 0x11,
    0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x1a, 0x1b, 0x1c, 0x1d, 0x1e, 0x1f, 0x20, 0x22, 0x24, 0x25, 0x26, 0x28, 0x2a, 0x2b,
    0x2c, 0x2d, 0x2e, 0x2f, 0x30, 0x32, 0x33, 0x34, 0x35, 0x36, 0x37, 0x3a, 0x3b, 0x41, 0x42, 0x43, 0x44, 0x45, 0x46,
    0x4a, 0x4c, 0x4e, 0x4f, 0x50, 0x51, 0x54, 0x56, 0x57,
];

/// One weapon's effective handling.
#[derive(Debug, PartialEq)]
struct Row { id: u8, held: Vec<usize>, tapped: Vec<usize>, /** shots in 200 held ticks */ pull: usize, clip: u32, reload: u32, reload_key: u32, zoom: f32, zoom_ticks: u32 }

fn row(id: u8, w: &[u8], script_len: impl Fn(u32) -> Option<u32>) -> Row {
    let p = GunParams::from_record(w, 0, &script_len);
    let big = p.clip.max(1) * 4;
    // Held: the trigger down for 200 ticks from a full clip.
    let mut g = Gun::new(id, p.clip.max(1), big);
    let held: Vec<usize> = (0..200).filter(|_| g.update(true, p)).collect();
    // Tapped: a fresh press every other tick.
    let mut g = Gun::new(id, p.clip.max(1), big);
    let tapped: Vec<usize> = (0..200).filter(|&t| g.update(t % 2 == 0, p)).collect();
    let pull = held.len();
    let zoom = f32::from_le_bytes(w[0x88..0x8c].try_into().unwrap());
    let (mut c, mut t, mut n) = (1.0f32, 1.0f32, 0u32);
    while c < zoom && n < 1000 { nf_game::gun::zoom_tick(&mut c, &mut t, zoom, -1.0); n += 1; }
    Row { id, held: held.into_iter().take(4).collect(), tapped: tapped.into_iter().take(4).collect(), pull, clip: p.clip, reload: if p.reload_start_ticks > 0 { p.reload_start_ticks + p.reload_loop_ticks } else { p.reload_ticks }, reload_key: if p.reload_start_ticks > 0 { p.reload_start_ticks + p.reload_loop_ticks } else { p.reload_key_ticks }, zoom, zoom_ticks: n }
}

/// The pinned rows: (id, first held shot ticks, first tapped shot ticks (a press every other tick),
/// shots in 200 held ticks, clip, auto-reload ticks, reload-key ticks, zoom max, held-zoom ticks 1x→max).
#[allow(clippy::type_complexity)]
const PINNED: &[(u8, &[usize], &[usize], usize, u32, u32, u32, f32, u32)] = &[
    (0x01, &[0], &[0, 32, 64, 96], 1, 1, 0, 0, 1.0, 0),
    (0x02, &[0], &[0, 10, 20, 30], 1, 7, 111, 115, 1.0, 0),
    (0x03, &[0], &[0, 10, 20, 30], 1, 7, 111, 115, 1.0, 0),
    (0x04, &[0], &[0, 10, 20, 30], 1, 7, 111, 115, 1.0, 0),
    (0x05, &[0], &[0, 10, 20, 30], 1, 7, 111, 115, 1.0, 0),
    (0x06, &[0], &[0, 10, 20, 30], 1, 16, 111, 115, 1.0, 0),
    (0x07, &[0], &[0, 10, 20, 30], 1, 16, 111, 115, 1.0, 0),
    (0x08, &[0], &[0, 10, 20, 30], 1, 16, 111, 115, 1.0, 0),
    (0x09, &[0], &[0, 10, 20, 30], 1, 16, 111, 115, 1.0, 0),
    (0x0a, &[0], &[0, 10, 20, 30], 1, 18, 138, 138, 1.0, 0),
    (0x0b, &[0, 4, 8], &[0, 6, 12, 18], 3, 18, 138, 138, 1.0, 0),
    (0x0c, &[0, 4, 8, 12], &[0, 6, 12, 18], 18, 18, 138, 138, 1.0, 0),
    (0x0d, &[0], &[0, 6, 12, 18], 1, 36, 223, 223, 1.0, 0),
    (0x0e, &[0], &[0, 16, 32, 48], 1, 9, 96, 100, 1.0, 0),
    (0x0f, &[0], &[0, 32, 64, 96], 1, 9, 96, 100, 1.0, 0),
    (0x10, &[0], &[0, 16, 32, 48], 1, 7, 96, 100, 1.0, 0),
    (0x11, &[0], &[0, 28, 56], 1, 3, 138, 138, 4.0, 57),
    (0x12, &[0, 6, 12, 18], &[0, 8, 16, 24], 32, 32, 134, 134, 1.0, 0),
    (0x13, &[0], &[0, 8, 16, 24], 1, 32, 127, 127, 1.0, 0),
    (0x14, &[0, 4, 8], &[0, 6, 12, 18], 3, 21, 134, 134, 1.0, 0),
    (0x15, &[0, 4, 8], &[0, 6, 12, 18], 3, 21, 134, 134, 1.0, 0),
    (0x16, &[0, 4, 8], &[0, 6, 12, 18], 3, 30, 149, 149, 1.0, 0),
    (0x17, &[0], &[0, 16, 32, 48], 1, 30, 149, 149, 1.0, 0),
    (0x1a, &[0, 4, 8], &[0, 6, 12, 18], 3, 30, 138, 138, 6.0, 73),
    (0x1b, &[0, 60, 120, 180], &[0, 62, 124, 186], 4, 6, 145, 145, 6.0, 73),
    (0x1c, &[0], &[0, 62, 124, 186], 1, 8, 66, 66, 1.0, 0),
    (0x1d, &[0, 30, 60, 90], &[0, 32, 64, 96], 7, 8, 66, 66, 1.0, 0),
    (0x1e, &[0], &[0, 122], 1, 5, 143, 143, 10.0, 94),
    (0x1f, &[0], &[0, 122], 1, 5, 143, 143, 10.0, 94),
    (0x20, &[0], &[0, 122], 1, 5, 143, 143, 16.0, 113),
    (0x22, &[0], &[0, 122], 1, 10, 143, 143, 16.0, 113),
    (0x24, &[0], &[0, 122], 1, 5, 143, 143, 10.0, 94),
    (0x25, &[0], &[0, 122], 1, 5, 143, 143, 10.0, 94),
    (0x26, &[0], &[0, 122], 1, 5, 143, 143, 16.0, 113),
    (0x28, &[0], &[0, 122], 1, 10, 143, 143, 16.0, 113),
    (0x2a, &[0], &[0, 62, 124, 186], 1, 6, 118, 118, 2.0, 29),
    (0x2b, &[0], &[0, 62, 124, 186], 1, 6, 118, 118, 2.0, 29),
    (0x2c, &[0], &[0, 62, 124, 186], 1, 4, 134, 134, 1.0, 0),
    (0x2d, &[0, 30, 60, 90], &[0, 32, 64, 96], 4, 4, 134, 134, 1.0, 0),
    (0x2e, &[0], &[0, 62, 124, 186], 1, 4, 134, 134, 1.0, 0),
    (0x2f, &[0, 30, 60, 90], &[0, 32, 64, 96], 4, 4, 134, 134, 1.0, 0),
    (0x30, &[0], &[0, 62, 124, 186], 1, 4, 134, 134, 1.0, 0),
    (0x32, &[0], &[0], 1, 100, 0, 0, 3.5, 51),
    (0x33, &[0, 30, 60, 90], &[0, 32, 64, 96], 7, 100, 0, 0, 3.5, 51),
    (0x34, &[0], &[0], 1, 1, 0, 0, 1.0, 0),
    (0x35, &[0], &[0], 1, 1, 0, 0, 1.0, 0),
    (0x36, &[0], &[0], 1, 1, 0, 0, 1.0, 0),
    (0x37, &[0], &[0], 1, 1, 0, 0, 1.0, 0),
    (0x3a, &[0], &[0], 1, 1, 0, 0, 1.0, 0),
    (0x3b, &[0], &[0], 1, 1, 0, 0, 1.0, 0),
    (0x41, &[0], &[0], 1, 1, 48, 48, 1.0, 0),
    (0x42, &[0], &[0], 1, 1, 63, 63, 1.0, 0),
    (0x43, &[0], &[0, 32, 64, 96], 1, 5, 108, 108, 1.0, 0),
    (0x44, &[0], &[0, 22, 44, 66], 1, 5, 108, 108, 1.0, 0),
    (0x45, &[0], &[0], 1, 100, 0, 0, 1.0, 0),
    (0x46, &[0, 30, 60, 90], &[0, 32, 64, 96], 7, 16, 115, 115, 1.0, 0),
    (0x4a, &[0, 1, 2, 3], &[0, 2, 4, 6], 100, 100, 0, 0, 1.0, 0),
    (0x4c, &[0, 1, 2, 3], &[0, 2, 4, 6], 100, 100, 0, 0, 1.0, 0),
    (0x4e, &[0, 1, 2, 3], &[0, 2, 4, 6], 200, 500, 0, 0, 1.0, 0),
    (0x4f, &[0, 1, 2, 3], &[0, 2, 4, 6], 200, 500, 0, 0, 1.0, 0),
    (0x50, &[0], &[0, 62, 124, 186], 1, 0, 0, 0, 1.0, 0),
    (0x51, &[0], &[0, 62, 124, 186], 1, 0, 0, 0, 1.0, 0),
    (0x54, &[0], &[0, 62, 124, 186], 1, 1, 0, 0, 8.0, 85),
    (0x56, &[0, 1, 2, 3], &[0, 2, 4, 6], 200, 0, 0, 0, 1.0, 0),
    (0x57, &[0, 1, 2, 3], &[0, 2, 4, 6], 200, 0, 0, 0, 1.0, 0),
];

/// Owns `id` with `rounds` in its clip and draws it through the real switch, then settles.
fn arm_and_draw(scene: &mut nf_view::scene::Scene, id: u8, rounds: i16) {
    use nf_game::player::Input;
    {
        let sim = scene.sim.as_mut().unwrap();
        if (id as usize) < sim.drones.inventory.owned.len() { sim.drones.inventory.owned[id as usize] = true; }
        if let Some(c) = sim.drones.inventory.clip.get_mut(id as usize) { *c = rounds; }
    }
    for _ in 0..60 {
        scene.sim.as_mut().unwrap().step_bond(Input { switch_to: Some(id), ..Default::default() }, false, false, false);
        if scene.sim.as_ref().unwrap().gun.weapon == id { break; }
    }
    for _ in 0..240 {
        let sim = scene.sim.as_ref().unwrap();
        if sim.gun.switch_state == 0 && sim.gun.reload_state == 0 && !sim.gun.fire_state { break; }
        scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false);
    }
    for _ in 0..5 { scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false); }
    assert_eq!(scene.sim.as_ref().unwrap().gun.weapon, id, "weapon {id:#x} equipped");
}

/// Taps the trigger every other tick for `ticks` and returns the shot ticks.
fn tap_shots(scene: &mut nf_view::scene::Scene, ticks: usize) -> Vec<usize> {
    use nf_game::player::Input;
    let mut out = Vec::new();
    for t in 0..ticks {
        let press = t % 2 == 0;
        scene.sim.as_mut().unwrap().step_bond(Input { fire: press, ..Default::default() }, press, false, false);
        if scene.sim.as_ref().unwrap().shot_tick { out.push(t); }
    }
    out
}

/// THE STATE-9 REFIRE GATE through the real Sim [M Player_WeaponFiring 0x1a5460:631-636]: the
/// pump Frinesi 0x1c (+0x68 = 0x8100, +0x40 = 60, fire script 80 [L/D]) refires only after its
/// fire script ends, not at its +0x40; the sister, the auto Frinesi 0x1d (+0x68 = 0x100 — no
/// 0x8000, +0x40 = 30, fire script 35), refires on its refire timer while the script still plays.
#[test]
fn the_8000_records_wait_for_the_fire_script() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut w = nf_view::load_with_carry_weapons(&iso, "07000005.bin", false, None, &[0x1c, 0x1d]).expect("load 07000005");
    for _ in 0..10 { w.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false); }
    arm_and_draw(&mut w, 0x1c, 8);
    let pump = tap_shots(&mut w, 200);
    let gaps: Vec<usize> = pump.windows(2).map(|p| p[1] - p[0]).collect();
    eprintln!("pump 0x1c shots {pump:?} gaps {gaps:?}");
    assert!(pump.len() >= 2, "the pump fired at least twice: {pump:?}");
    assert!(gaps.iter().all(|&g| g >= 80), "the pump waits out its 80-tick fire script, gaps {gaps:?}");
    // Sister: without 0x8000 the refire timer alone paces the shots (30 < the 35-tick script).
    arm_and_draw(&mut w, 0x1d, 8);
    let auto = tap_shots(&mut w, 200);
    let gaps: Vec<usize> = auto.windows(2).map(|p| p[1] - p[0]).collect();
    eprintln!("auto 0x1d shots {auto:?} gaps {gaps:?}");
    assert!(gaps.iter().any(|&g| g < 35), "the 0x1d twin refires inside its fire script, gaps {gaps:?}");
}

/// The Delta Repeater 0x11 (+0x68 = 0x180c0: 0x8000 set; +0x40 = 26; fire script +0xB0 80 frames,
/// the last round's +0xB4 30 [D 07000005]) [M Player_SetFiringAnim 0x1a3c70:25-33 + the state-9
/// gate 0x1a5460:631-636]: a full clip's first shot holds the trigger for the 80-frame script.
#[test]
fn the_delta_repeater_waits_for_its_fire_script() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut w = nf_view::load_with_carry_weapons(&iso, "07000005.bin", false, None, &[0x11]).expect("load 07000005");
    for _ in 0..10 { w.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false); }
    arm_and_draw(&mut w, 0x11, 3);
    let shots = tap_shots(&mut w, 120);
    eprintln!("delta 0x11 shots {shots:?}");
    assert!(shots.len() >= 2 && shots[1] - shots[0] >= 80, "the second bolt waits for the 80-frame script: {shots:?}");
    // Sister: the bare record cadence (no view-state gate) is the +0x40 timer's 28.
    let rec = w.sim.as_ref().unwrap().weapons.records[0x11];
    let r = row(0x11, &rec, |_| Some(80));
    assert_eq!(r.tapped[1] - r.tapped[0], 28);
}

#[test]
fn every_weapon_handles_as_its_disc_record() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let w = nf_view::load_with_carry_weapons(&iso, "07000005.bin", false, None, WEAPONS).expect("load 07000005");
    let sim = w.sim.as_ref().unwrap();
    let rows: Vec<Row> = WEAPONS.iter().map(|&id| {
        let rec = &sim.weapons.records[id as usize];
        row(id, rec, |h| sim.scripts.get(&h).map(|s| s.length as u32))
    }).collect();
    assert_eq!(rows.len(), PINNED.len());
    for (r, e) in rows.iter().zip(PINNED) {
        let got = (r.id, r.held.as_slice(), r.tapped.as_slice(), r.pull, r.clip, r.reload, r.reload_key, r.zoom, r.zoom_ticks);
        assert_eq!(got, *e, "weapon {:#04x}", r.id);
    }
    // Sister: the same pipeline on an edited record moves its row — the P2K with +0x40 = 10 taps at
    // 0, 12, 24 (the press on tick 10 meets a live timer) instead of 0, 10, 20, and +0x92 = 8 moves
    // the clip.
    let mut rec = sim.weapons.records[2];
    rec[0x40..0x44].copy_from_slice(&10u32.to_le_bytes());
    rec[0x92..0x94].copy_from_slice(&8u16.to_le_bytes());
    let edited = row(2, &rec, |h| sim.scripts.get(&h).map(|s| s.length as u32));
    assert_ne!(edited.tapped, rows[1].tapped, "a slower +0x40 must move the tapped cadence");
    assert_ne!(edited.clip, rows[1].clip, "the clip follows +0x92");
}
