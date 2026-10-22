//! door-hint-boot-count: the 3-per-BOOT `DoorMsg` cap crosses the level load through the
//! scene-to-scene handoff.
//!
//! The game's rule, all cited:
//! - `Check_Target` 0x1a7810's class-0x1c arm shows `Door_DoMsg` 0x135f18's 0x200000b "Press
//!   Action to open door" hint only while the .sdata global `DoorMsg` 0x30cd04 < 3, and
//!   increments it on every SHOWN hint [M disasm 0x1a7b88 lw — the `DoorMsg < 3` gate,
//!   0x1a7ba8 lw + 0x1a7bc0 sw — the `DoorMsg + 1`; the gate short-circuits BEFORE the
//!   `Door_DoMsg` call, so a hint the cap blocks never reaches the door].
//! - A scan of the whole ACTION.ELF (this task) finds exactly THREE gp-relative references to
//!   0x30cd04 (gp 0x314670, immediate 0x8694): the two loads and the one store above, all inside
//!   `Check_Target` — and no absolute `lui`-based reference. No other reader, no other writer, no
//!   reset anywhere in the on-foot ELF [M the gp-relative scan].
//! - `DoorMsg`'s image bytes in the ELF are 00 00 00 00 [D ACTION.ELF file offset 0x206c04] — a
//!   boot (a NEW process) starts at 0, and since the only writer is the +1, only a boot resets it.
//!   A level load, the pause-menu Restart (`C_GCPAUSE_Handler` 0x204408:139 → `ResetMap_
//!   LevelToLoad(BaseMap, 0, 1)`, a plain cold level load) and the campaign transition are all
//!   level loads — the counter survives every one of them.
//! - The counter is NOT in any save: the game's `.sdata` is process memory outside `Player_
//!   RamSave`'s buffers [M 0x195670's fields]; the port's `RamSave` carries
//!   health/switch/inventory/channels/mission only, no hud.
//!
//! The port keeps the count in the per-level Sim's HUD, so the scene-to-scene handoff must carry
//! it: every scene replacement in the app (`App::swap_in_scene` — level loads, both Restart arms,
//! the campaign transition; the weapon-asset warm reload; the MP late-join replay) calls
//! `HudState::carry_boot_globals`. This test drives that exact shape minus the GPU halves this
//! sandbox cannot run: the worker's `load_with_carry` build, then the carry line.
//!
//! Fixtures [D placements, the probe in this file's history]: 07000001 doors 0/1/2 and 07000002
//! door 0 — kind-2 sliders, closed (state 2), no lock channel, ungrouped, live spheres the aim
//! pick reaches from the spawn side at 2.5 units.
//!
//! Opt-in: `NF_ISO`. NF_SILENT everywhere (the ISO tests run under the suite's silent rule).

use nf_game::player::{Input, RamSave};

fn settle(scene: &mut nf_view::scene::Scene, cap: u32) {
    let sim = scene.sim.as_mut().unwrap();
    for _ in 0..cap {
        sim.step_bond(Default::default(), false, false, false);
        if !sim.disabled { break; }
    }
}

/// Yaw/pitch (the port's units: pitch is in π/2 turns) so `bond.look()` points at `target`.
fn face(sim: &mut nf_view::scene::Sim, target: [f32; 3]) {
    let head = sim.bond.head();
    let l = [target[0] - head[0], target[1] - head[1], target[2] - head[2]];
    let len = (l[0] * l[0] + l[1] * l[1] + l[2] * l[2]).sqrt();
    let l = [l[0] / len, l[1] / len, l[2] / len];
    sim.bond.yaw = l[0].atan2(l[2]);
    sim.bond.pitch = (l[1].asin()) / (std::f32::consts::FRAC_PI_2);
}

fn raised(sim: &nf_view::scene::Sim, label: u32) -> Vec<f32> {
    sim.hud.info_msgs.iter().filter(|m| m.label == label).map(|m| m.life).collect()
}

/// Stand 2.5 units from door `di`'s live-sphere centre on the spawn side and face it — the probe's
/// raise direction for every fixture door (a clear line, inside the game's 5-unit gate [M
/// 0x1a7b6c-0x1a7b84]).
fn stand_at_door(sim: &mut nf_view::scene::Sim, di: usize) {
    let (c, _r) = sim.drones.doors.list[di].live_sph();
    let to_spawn = [sim.bond.pos[0] - c[0], 0.0, sim.bond.pos[2] - c[2]];
    let len = (to_spawn[0] * to_spawn[0] + to_spawn[2] * to_spawn[2]).sqrt();
    sim.bond.pos = [c[0] - to_spawn[0] / len * 2.5, c[1], c[2] - to_spawn[2] / len * 2.5];
    face(sim, c);
}

/// Step the sim until the 0x200000b hint raises or the budget ends; returns how many NEW
/// raises landed (the earlier doors' 180-tick messages are still alive in the list — the delta,
/// not the presence, is the signal).
fn aim_hint_raises(sim: &mut nf_view::scene::Sim, di: usize) -> usize {
    stand_at_door(sim, di);
    let before = raised(sim, 0x200_000b).len();
    for _ in 0..12 {
        sim.step_bond(Input::default(), false, false, false);
        let now = raised(sim, 0x200_000b).len();
        if now > before { return now - before; }
    }
    0
}

/// The app's level swap minus the GPU halves: the worker's build (`load_with_carry`, exactly what
/// `App::start_async_load`'s job runs — the campaign arm's `RamSave::capture` + `with_mission`) +
/// `App::swap_in_scene`'s carry line (the one line a level swap touches the hud through).
fn load_next_as_app_does(
    iso: &str, name: &str, old: &nf_view::scene::Scene,
) -> nf_view::scene::Scene {
    let save = old.sim.as_ref().map(|s| RamSave::capture(
        &s.drones.health, s.gun.switch_request, &s.drones.inventory, &s.drones.channels)
        .with_mission(&s.drones.mission));
    let mut scene = nf_view::load_with_carry(iso, name, false, save.as_ref()).expect("load the next level");
    if let (Some(o), Some(n)) = (old.sim.as_ref(), scene.sim.as_mut()) { n.hud.carry_boot_globals(&o.hud); }
    scene
}

/// The driver's real-path test: three hints on one level, then the next level loads the app's way
/// and a fresh closed unlocked door hints NOTHING; sister: a fresh session of that level shows
/// one. The cap-blocked door's latch stays clear — the game's `DoorMsg < 3` gate short-circuits
/// before `Door_DoMsg` [M 0x1a7b88-0x1a7b98], so the blocked attempt burns nothing.
#[test]
fn three_hints_on_one_level_the_next_level_shows_no_fresh_hint() {
    let Ok(iso) = std::env::var("NF_ISO") else { eprintln!("NF_ISO not set — skipping"); return };
    std::env::set_var("NF_SILENT", "1");

    // ── Level A (07000001): burn the boot's three hints through the real aim ray. ──
    let mut a = nf_view::load(&iso, "07000001.bin", false).expect("load 07000001");
    settle(&mut a, 3000);
    // The fixtures: doors 0/1/2 are closed sliders with no lock channel, ungrouped [D placements].
    for di in [0usize, 1, 2] {
        let d = &a.sim.as_ref().unwrap().drones.doors.list[di];
        assert_eq!(d.state, nf_game::door::CLOSED, "fixture door {di} starts closed");
        assert_eq!(d.ch_lock, 0, "fixture door {di} is unlocked");
        assert_eq!(d.group, 0, "fixture door {di} is ungrouped");
        assert!(!d.msg_shown, "fixture door {di}'s latch starts clear");
    }
    for di in [0usize, 1, 2] {
        let sim = a.sim.as_mut().unwrap();
        assert_eq!(aim_hint_raises(sim, di), 1, "door {di} raised the 0x200000b hint exactly once through the aim ray");
        let sim = a.sim.as_ref().unwrap();
        assert!(sim.drones.doors.list[di].msg_shown, "door {di}'s +0x74 latch set [M 0x135f18:16-17]");
    }
    assert_eq!(a.sim.as_ref().unwrap().hud.door_msg, 3,
        "three raises = the boot's cap reached [M 0x1a7b88-0x1a7bc8]");

    // ── The next level, loaded the app's way (worker build + the swap's carry line). ──
    let mut b = load_next_as_app_does(&iso, "07000002.bin", &a);
    assert_eq!(b.sim.as_ref().unwrap().hud.door_msg, 3,
        "the boot count crossed the level load — no per-level reset");
    // 07000002 door 0: a fresh closed unlocked slider [D placements — ch_lock 0, group 0, state 2].
    {
        let d = &b.sim.as_ref().unwrap().drones.doors.list[0];
        assert_eq!(d.state, nf_game::door::CLOSED, "the fresh door starts closed");
        assert_eq!(d.ch_lock, 0, "the fresh door is unlocked");
        assert!(!d.msg_shown, "the fresh door's latch starts clear");
    }
    let sim = b.sim.as_mut().unwrap();
    assert_eq!(aim_hint_raises(sim, 0), 0, "no hint at the cap — the boot's three hints were spent on level A");
    let sim = b.sim.as_ref().unwrap();
    assert_eq!(sim.hud.door_msg, 3, "the cap count did not move");
    assert!(!sim.drones.doors.list[0].msg_shown,
        "the blocked attempt burned NOTHING — the DoorMsg < 3 gate precedes Door_DoMsg [M 0x1a7b88-0x1a7b98]");

    // ── Sister: a fresh session of the same level shows one (a new boot starts at 0 [D]). ──
    let mut c = nf_view::load(&iso, "07000002.bin", false).expect("fresh load 07000002");
    settle(&mut c, 3000);
    assert_eq!(c.sim.as_ref().unwrap().hud.door_msg, 0,
        "a fresh session starts at 0 [D the ELF's .sdata image bytes for 0x30cd04 are zero]");
    let sim = c.sim.as_mut().unwrap();
    assert_eq!(aim_hint_raises(sim, 0), 1, "the SAME door raises on a fresh session — the no-hint above was the carried cap, not the door");
    let sim = c.sim.as_ref().unwrap();
    assert_eq!(sim.hud.door_msg, 1, "the fresh session's first hint counts [M 0x1a7bc0]");
    assert!(sim.drones.doors.list[0].msg_shown, "the fresh session's hint latched the door [M 0x135f18:16-17]");
}

/// Sister at the HUD layer (the unit under the app line, kept from door-text-labels): the carry
/// moves BOTH boot globals and nothing else rides along.
#[test]
fn the_carry_moves_exactly_the_boot_globals() {
    let mut old = nf_view::scene::HudState::default();
    old.door_msg = 3;
    old.load_msg = 700;
    old.info_close = 9.0;
    let mut fresh = nf_view::scene::HudState::default();
    fresh.carry_boot_globals(&old);
    assert_eq!((fresh.door_msg, fresh.load_msg), (3, 700), "both boot globals cross");
    assert_eq!(fresh.info_close, 0.0, "the per-level hud fields do NOT cross");
}
