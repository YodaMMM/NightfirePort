//! Large object updates read branch by branch against the port (docs/reports/agent-updates-lbl.md).
//! Each test drives a real level and fails on the code before the matching fix. Opt-in: `NF_ISO`.

use nf_game::drone::PlayerBullet;
use nf_game::gt::gt_state;
use nf_game::player::Input;

fn load(level: &str) -> Option<nf_view::scene::Scene> {
    let iso = std::env::var("NF_ISO").ok()?;
    std::env::set_var("NF_SILENT", "1");
    let mut scene = nf_view::load(&iso, level, false).expect("load");
    let sim = scene.sim.as_mut().unwrap();
    for _ in 0..3000 {
        sim.step_bond(Default::default(), false, false, false);
        if !sim.disabled { break; }
    }
    Some(scene)
}

fn round(pos: [f32; 3], dir: [f32; 3], weapon: u16, frame: u32) -> PlayerBullet {
    PlayerBullet {
        pos, dir, speed: 2.0, travelled: 0.0, range: 200.0, damage: 10.0, weapon, first: true, dead: false, bounces: 0,
        homing: false, target: None, gravity: false, guided_cam: false, snd: 0, swoosh: None, born: frame,
    }
}

fn lock_at(sim: &nf_view::scene::Sim, placement: usize) -> usize {
    sim.drones.locks.iter().position(|l| l.placement == placement).expect("the lock placement")
}

/// One laser round per tick through the lock's sphere, for `ticks` ticks.
fn lase(sim: &mut nf_view::scene::Sim, li: usize, weapon: u16, ticks: u32) {
    for _ in 0..ticks {
        let l = &sim.drones.locks[li];
        let (p, f) = (l.pos, l.facing);
        let from = [p[0] + f[0] * 1.5, p[1] + f[1] * 1.5, p[2] + f[2] * 1.5];
        let frame = sim.drones.frame;
        sim.drones.player_bullets.push(round(from, [-f[0], -f[1], -f[2]], weapon, frame));
        sim.drones.health.health = 100.0;
        sim.step_bond(Input::default(), false, false, false);
    }
}

/// `GT_Update` 0x180498:128-148 sums the damage on script A's objects into ext+0xCC every tick, and
/// the death gate takes the gun to state 6 [M :236-258]. The port had no bullet feed at all: only
/// `Explode_Propagate` reached `apply_damage`.
#[test]
fn a_wall_gun_can_be_shot_down_0a() {
    let Some(mut scene) = load("0700000a.bin") else { return };
    let sim = scene.sim.as_mut().unwrap();
    let ti = sim.drones.gt_turrets.iter().position(|t| t.param4 == 18 && t.facing == 1).expect("the wall gun");
    let (gun, fd) = (sim.drones.gt_turrets[ti].pos, sim.drones.gt_turrets[ti].fire_dir);
    sim.bond.pos = [gun[0] + fd[0] * 40.0, 0.0, gun[2] + fd[2] * 40.0];
    let frame = sim.drones.frame;
    sim.drones.channels.set(18, 1, frame);
    for _ in 0..900 {
        sim.drones.health.health = 100.0;
        sim.step_bond(Input::default(), false, false, false);
        if sim.drones.gt_turrets[ti].state == gt_state::COMBAT { break; }
    }
    assert_eq!(sim.drones.gt_turrets[ti].state, gt_state::COMBAT, "the gun deployed");
    let from = [gun[0] + fd[0] * 3.0, gun[1] + fd[1] * 3.0, gun[2] + fd[2] * 3.0];
    // Sister: the turret's own record never counts against it.
    for _ in 0..30 {
        let frame = sim.drones.frame;
        sim.drones.player_bullets.push(round(from, [-fd[0], -fd[1], -fd[2]], nf_game::gt::TURRET_WEAPON_RECORD as u16, frame));
        sim.drones.health.health = 100.0;
        sim.step_bond(Input::default(), false, false, false);
    }
    assert_eq!(sim.drones.gt_turrets[ti].health, 100.0, "record 96 rounds do not hurt the gun");
    for _ in 0..1200 {
        let frame = sim.drones.frame;
        sim.drones.player_bullets.push(round(from, [-fd[0], -fd[1], -fd[2]], 2, frame));
        sim.drones.health.health = 100.0;
        sim.step_bond(Input::default(), false, false, false);
        if sim.drones.gt_turrets[ti].state == gt_state::DESTROYED { break; }
    }
    let t = &sim.drones.gt_turrets[ti];
    assert!(t.health < 0.0, "the rounds wore the gun's 100 health down: {}", t.health);
    assert_eq!(t.state, gt_state::DESTROYED, "the death gate took it");
}

/// `Lock_Update` case 0 reads the laser ids in the kind-1 arm only [M 0x1ae3d8:47-101]. A kind-0
/// keypad looks for 0x56/0x57 [M :162]; the port's burn took any lock in the bullet's way.
#[test]
fn the_laser_does_not_burn_a_keypad_0a() {
    let Some(mut scene) = load("0700000a.bin") else { return };
    let sim = scene.sim.as_mut().unwrap();
    let li = lock_at(sim, 646);
    assert_eq!(sim.drones.locks[li].kind, 0);
    lase(sim, li, 0x4f, 120);
    assert!(!sim.drones.locks[li].burned && !sim.drones.locks[li].unlocked, "the keypad holds");
    assert!(!sim.drones.channels.on(0x31), "its channel stays down");
}

/// Sister: the same rounds on a kind-1 bolt burn through.
#[test]
fn the_laser_burns_a_kind_1_lock_0c() {
    let Some(mut scene) = load("0700000c.bin") else { return };
    let sim = scene.sim.as_mut().unwrap();
    let li = lock_at(sim, 139);
    assert_eq!(sim.drones.locks[li].kind, 1);
    lase(sim, li, 0x4f, 60);
    assert!(sim.drones.locks[li].burned);
    assert!(sim.drones.channels.on(0xe));
    assert!(sim.drones.inventory.keys[1], "`Lock_Unlock` equips the code [M 0x1b0650:19]");
}

/// `Lock_Unlock` 0x1b0650:19 equips the code, and `Lock_Update` case 0 opens a closed lock whose
/// key is on the ring [M 0x1ae3d8:139-147]: 0a's keypads come in pairs, one per door face, with
/// the same code and out channel [D #646/#729], so solving one opens both. Then a dropped channel
/// re-locks both and takes the key back [M case 2, 0x1b0750].
#[test]
fn solving_one_keypad_opens_its_pair_and_a_dropped_channel_relocks_0a() {
    let Some(mut scene) = load("0700000a.bin") else { return };
    let sim = scene.sim.as_mut().unwrap();
    let (a, b) = (lock_at(sim, 646), lock_at(sim, 729));
    assert_eq!(sim.drones.locks[a].code, sim.drones.locks[b].code);
    let code = sim.drones.key_codes.code(sim.drones.locks[a].code);
    sim.drones.keypad = Some((a, nf_game::lock::Keypad::arm(code, false)));
    for c in code { sim.drones.keypad_digit((c - b'0') as u32); }
    for _ in 0..4 { sim.step_bond(Input::default(), false, false, false); }
    assert!(sim.drones.locks[a].unlocked, "the solved keypad");
    assert!(sim.drones.inventory.keys[2], "the key is on the ring");
    assert!(sim.drones.locks[b].unlocked, "its pair opened off the ring");
    assert!(sim.drones.channels.on(0x31));
    // The channel holds on its own now; the drop is what re-locks.
    for _ in 0..30 { sim.step_bond(Input::default(), false, false, false); }
    assert!(sim.drones.locks[a].unlocked && sim.drones.locks[b].unlocked, "nothing re-locks while the channel is up");
    let frame = sim.drones.frame;
    sim.drones.channels.set(0x31, 0, frame);
    for _ in 0..3 { sim.step_bond(Input::default(), false, false, false); }
    assert!(!sim.drones.locks[a].unlocked && !sim.drones.locks[b].unlocked, "both re-locked");
    assert!(!sim.drones.inventory.keys[2], "`Player_UnEquipKey` took the key");
    assert!(!sim.drones.channels.on(0x31));
}

/// The disc's two key pickups sit on 07000046 (#10 id 1, #56 id 5). Key 5 is lock #37's code, a
/// kind-0 lock, and its channel 0x3e is door #38's lock channel [D]: the key alone opens it
/// [M 0x1ae3d8:139-147]. Lock #9 is kind 3: its case 0 does nothing, the key is not enough [M :128].
#[test]
fn a_picked_up_key_opens_its_lock_46() {
    let Some(mut scene) = load("07000046.bin") else { return };
    let sim = scene.sim.as_mut().unwrap();
    let (keyed, other) = (lock_at(sim, 37), lock_at(sim, 9));
    sim.drones.inventory.keys[5] = true;
    sim.drones.inventory.keys[1] = true;
    for _ in 0..4 { sim.step_bond(Input::default(), false, false, false); }
    assert!(sim.drones.locks[keyed].unlocked, "the key on the ring opened #37");
    assert!(sim.drones.channels.on(0x3e), "and raised door #38's lock channel");
    assert!(!sim.drones.locks[other].unlocked, "kind 3 waits for the use press");
}

/// `Sensor_Update` skips both cone tests while `glb_viewer[4]+0x29` is set [M 0x1af618:141-143,
/// :204-206]. Bond stands on a camera's axis through a cut-scene: it must not count him.
#[test]
fn a_camera_is_blind_during_a_cut_scene() {
    let Ok(iso) = std::env::var("NF_ISO") else { return };
    std::env::set_var("NF_SILENT", "1");
    let mut tried = 0;
    for level in ["07000014.bin", "07000012.bin", "0700000b.bin", "0700000a.bin", "07000002.bin", "07000001.bin"] {
        let mut scene = nf_view::load(&iso, level, false).expect("load");
        let sim = scene.sim.as_mut().unwrap();
        for _ in 0..200 {
            if sim.script_camera().is_some() { break; }
            sim.step_bond(Input::default(), false, false, false);
        }
        if sim.script_camera().is_none() || sim.drones.sensors.list.is_empty() { continue; }
        tried += 1;
        let mut max_seen = 0;
        let mut ticks = 0;
        for _ in 0..120 {
            if sim.script_camera().is_none() { break; }
            let s = &sim.drones.sensors.list[0];
            let (p, d, r) = (s.pos, [s.matrix[2][0], s.matrix[2][1], s.matrix[2][2]], s.range * 0.5);
            let at = [p[0] + d[0] * r, p[1] + d[1] * r - 1.0, p[2] + d[2] * r];
            sim.bond.pos = at;
            sim.step_bond(Input::default(), false, false, false);
            max_seen = max_seen.max(sim.drones.sensors.list[0].seen);
            ticks += 1;
        }
        eprintln!("{level}: {ticks} camera ticks, max seen {max_seen}");
        assert!(ticks > 0);
        assert_eq!(max_seen, 0, "{level}: the camera counted Bond during a cut-scene");
    }
    assert!(tried > 0, "no level opened with a script camera and a sensor");
}

/// `RB_Update`'s buoyancy arm: which bodies stand under a water surface at load, on every level
/// that places one. The arm is the only rigid-body draw on the shared stream.
#[test]
fn rigid_bodies_under_water_census() {
    let Ok(iso) = std::env::var("NF_ISO") else { return };
    std::env::set_var("NF_SILENT", "1");
    for level in ["07000001.bin", "07000002.bin", "07000003.bin", "07000004.bin", "07000006.bin", "0700000d.bin"] {
        let scene = nf_view::load(&iso, level, false).expect("load");
        let sim = scene.sim.as_ref().unwrap();
        let under = sim.drones.rbodies.list.iter().filter(|o| o.buoy.is_some_and(|(surface, _)| o.st.trans[1] < surface)).count();
        eprintln!("{level}: {} bodies, {under} under a surface", sim.drones.rbodies.list.len());
    }
}

/// With the laser out of the picture a keypad opens by the decryptor (ids 0x56/0x57) or a typed
/// code. Every level with a closed kind-0 lock must leave Bond a way through.
#[test]
fn keypad_levels_census() {
    let Ok(iso) = std::env::var("NF_ISO") else { return };
    std::env::set_var("NF_SILENT", "1");
    for level in ["0700000a.bin", "0700000b.bin", "07000011.bin", "07000012.bin", "07000014.bin", "07000015.bin"] {
        let scene = nf_view::load(&iso, level, false).expect("load");
        let sim = scene.sim.as_ref().unwrap();
        let inv = &sim.drones.inventory;
        let closed = sim.drones.locks.iter().filter(|l| l.kind == 0 && !l.unlocked).count();
        eprintln!("KP {level}: closed kind-0 {closed}, owns 0x56 {} 0x57 {}, difficulty {}", inv.owned[0x56], inv.owned[0x57], sim.drones.difficulty);
    }
}
