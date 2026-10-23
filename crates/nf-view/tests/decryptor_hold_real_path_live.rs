//! decryptor-hold-real-path (988): the driver's own scenario on the real input path. On 07000012
//! Bond stands where the driver stood — `--bond 7.0,0.5,2.4,3.1416,0` (the red keypad locks beside
//! the door, disc placements m38 #919/#957 [D census: "Lock - KeypadLocked" @(7.01,1.58,0.19) and
//! @(6.74,1.58,0.18), params 0:0x0 kind, 5:0xa out]) — the Decryptor given and equipped through the
//! real equip/switch paths, then the fire input HELD through `step_bond` for the game's scan
//! duration. No direct state pokes: no lock/anim writes, no teleports after the placement; the
//! approach is the real player's (turn/look/forward inputs), the health floor is the walkreal's
//! documented stand-in (this pins the decryptor, not the firefight).
//!
//! The game's model [M]+[L]:
//! - The 0x56's pull is a burst of **999 rounds** (`weapon_data+0x28` mode 0 = 999) at interval
//!   **1 tick** (`+0x40` = 1), `+0x44` = 0 → the round spawns on the fire tick itself [M
//!   Player_WeaponFiring 0x1a5460:688-693] — holding re-fires every tick for ~16.6 s [L
//!   ee_start_05.bin @ weapon_data 0x2c4b58].
//! - A fresh press arms the burst (disasm 0x1a6330 `Input_Action(…,9,4)` edge → 0x1a6380:
//!   `+0x92c <= 0` → `+0x936` = shots_per_pull, `+0x92c` = 1.0); the fire runs only in view
//!   states 0/9/0xb [M 0x1a5460:631-648].
//! - The use gate runs BEFORE `Player_RoundToFire` on every held tick [M 0x1a5460:551-590]: the
//!   crosshair ray's target must be a lock of kind 0/2 in state 0/5 within `weapon_data+0x1C`
//!   (2.0 for the 0x56 [L]) — a refused gate returns before the pull (no fire anim at all).
//! - On a passed gate the round spawns, and the view state 9 suppress only skips the RE-queue
//!   [M 0x1a5460:664-672]; the anim object still ticks every frame [M AnimScriptTick 0x1c2630]
//!   and the case-9 stop check runs every frame [M Player_SetWeaponAnimObj 0x1a2e38:449-456] —
//!   the scan animation LOOPS while held. The port ran its advance and stop check only on
//!   non-fired ticks (the master bug: the held decryptor froze at frame 0 — the driver's "does
//!   not continue its animation").
//! - Each held tick's scan bullet decodes +1 [M Lock_Update 0x1ae3d8:305-315]; the digits solve
//!   at progress 160/120/80/40 (425 contact ticks for the 0x56, the last digit crossing twice
//!   [M :183,348-350,364-372]) and `Lock_Unlock` writes the out channel [M 0x1b0650:193-196].
//!
//! The second test passes the same held-fire chain on 07000014's decryptor lock (disc placement
//! m54 #1100 [D census: kind 0x0, code 2, out ch 0x2c, operative tag 0]).

use nf_view::scene::Sim;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

/// The driver's spot: `--bond 7.0,0.5,2.4,3.1416,0` on 07000012 (facing the keypad wall).
const BOND_SPOT: [f32; 3] = [7.0, 0.5, 2.4];
const BOND_YAW: f32 = 3.1416;
const BOND_PITCH: f32 = 0.0;

/// The 0x56's own record facts [L ee_start_05.bin @ weapon_data 0x2c4b58].
const FIRE_SCRIPT: u32 = 0x0600_05c3; // +0xB0
const RANGE_0X56: f32 = 2.0; // +0x1C

/// Equip 0x56 through the real paths the harness uses (`--give`'s equip_weapon, its owned
/// fallback — the game's own equip refuses an owned gadget on the ammo-type cap, then the
/// contract is to DRAW it through the BL+0x63 switch request [the --give handler] — and the
/// switch settle).
fn equip_decryptor(sim: &mut Sim) {
    let wd = sim.weapons.clone();
    let rounds = wd.clip_size(0x56).max(1);
    if !sim.drones.inventory.equip_weapon(&wd, 0x56, rounds, true) {
        assert!(sim.drones.inventory.owned.get(0x56).copied().unwrap_or(false),
            "the Decryptor must be equippable or already owned (the --give contract)");
    } else {
        sim.drones.inventory.equip_ammo(&wd, 0x56, (rounds * 3).max(rounds));
    }
    sim.gun.switch_request = 0x56;
    let mut equipped = false;
    for _ in 0..400 {
        sim.step_bond(Default::default(), false, false, false);
        if sim.gun.weapon == 0x56 && sim.gun.switch_state == 0 { equipped = true; break; }
    }
    assert!(equipped, "the Decryptor (0x56) must equip; weapon now {:#x}", sim.gun.weapon);
}

/// The kind-0 lock the crosshair picks from `eye` along `dir` — the gate's own stand-in
/// (`pick_decryptor_lock`), used here only to NAME the lock the test asserts on.
fn aimed_lock(sim: &Sim) -> Option<usize> {
    let eye = [sim.bond.pos[0], sim.bond.pos[1] + sim.bond.eye - sim.bond.crouch_drop, sim.bond.pos[2]];
    let dir = sim.bond.look();
    let root = sim.bond.pos;
    nf_game::gadget::pick_decryptor_lock(eye, dir, root, &sim.drones.locks, &sim.world, RANGE_0X56)
}

/// One real-input tick: aim at `target` through the stick (the walkreal's documented aim loop),
/// walk forward when roughly on-heading. `fire` = the trigger held. Returns the yaw error.
fn real_tick(sim: &mut Sim, target: [f32; 3], fire: bool, drive: bool) -> f32 {
    sim.drones.health.health = sim.drones.health.health.max(100.0); // the walker's allowance
    let eye = [sim.bond.pos[0], sim.bond.pos[1] + sim.bond.eye - sim.bond.crouch_drop, sim.bond.pos[2]];
    let (dx, dz, dy) = (target[0] - eye[0], target[2] - eye[2], target[1] - eye[1]);
    let desired = dx.atan2(dz);
    let err = desired - sim.bond.yaw;
    let want_pitch = (dy / (dx * dx + dz * dz).sqrt().max(0.5)).atan() / std::f32::consts::FRAC_PI_2;
    let input = nf_game::player::Input {
        turn: err.clamp(-0.08, 0.08),
        look: (want_pitch - sim.bond.pitch).clamp(-0.08, 0.08),
        forward: if drive && err.abs() < 0.35 { 1.0 } else { 0.0 },
        fire,
        ..Default::default()
    };
    sim.step_bond(input, fire, false, false);
    err
}

/// The shared real-path chain: place Bond at `spot`/`yaw`, equip 0x56, settle, then approach the
/// named lock with real inputs until the gate picks it; hold fire and report the scan/anim.
/// Returns (approach ticks, Some(held tick of the unlock), max scan progress, view-time samples
/// while the fire script plays, the report).
fn real_path_chain(
    mut scene: nf_view::scene::Scene,
    spot: [f32; 3],
    yaw: f32,
    pitch: f32,
    lock_of: impl Fn(&Sim) -> Option<usize>,
    out_channel: u32,
) -> (usize, Option<usize>, u32, Vec<(u32, u32, f32)>, String) {
    let sim = scene.sim.as_mut().unwrap();
    // The exact --bond placement (nf-view's own handler: Bond::new(pos, yaw) + the pitch field).
    sim.bond = nf_game::player::Bond::new(spot, yaw);
    sim.bond.pitch = pitch;
    equip_decryptor(sim);

    // The real player's approach: aim at the lock, walk when on-heading, until the gate picks.
    let mut report = String::new();
    let mut walked = 0;
    let mut picked = false;
    for t in 0..480 {
        let Some(li) = lock_of(sim) else { break }; // the pick names the lock
        let lpos = sim.drones.locks[li].pos;
        real_tick(sim, lpos, false, true);
        walked += 1;
        if aimed_lock(sim).is_some() { picked = true; break; }
        let _ = t;
    }
    let li = lock_of(sim);
    report.push_str(&format!("approach: {walked} ticks, gate pick {li:?} (picked {picked}), bond {:?}\n", sim.bond.pos));

    // HOLD the fire input for the game's scan duration (the 0x56 needs 425 contact ticks
    // [M 0x1ae3d8:183,348-350,364-372] + the two-tick collide lag — 900 is headroom).
    let mut unlocked_at = None;
    let mut samples: Vec<(u32, u32, f32)> = Vec::new();
    let mut max_progress = 0u32;
    for t in 0..950u32 {
        let Some(cur) = lock_of(sim) else { break };
        real_tick(sim, sim.drones.locks[cur].pos, true, false);
        if t < 24 {
            let eye = [sim.bond.pos[0], sim.bond.pos[1] + sim.bond.eye - sim.bond.crouch_drop, sim.bond.pos[2]];
            let dir = sim.bond.look();
            let lp = sim.drones.locks[cur].pos;
            let ap = [lp[0] - eye[0], lp[1] - eye[1], lp[2] - eye[2]];
            let dd = (dir[0] * dir[0] + dir[1] * dir[1] + dir[2] * dir[2]).max(1e-9);
            let tt = (ap[0] * dir[0] + ap[1] * dir[1] + ap[2] * dir[2]) / dd;
            let cl = [eye[0] + dir[0] * tt, eye[1] + dir[1] * tt, eye[2] + dir[2] * tt];
            let d = ((cl[0] - lp[0]).powi(2) + (cl[1] - lp[1]).powi(2) + (cl[2] - lp[2]).powi(2)).sqrt();
            let script_now = sim.view_script;
            let target_now = sim.decryptor_target;
            let radius_now = sim.drones.locks[cur].radius;
            report.push_str(&format!("hold t{t}: pitch {:.4} d {d:.3} radius {radius_now:.3} script {script_now:x} time {:.1} target {target_now:?}\n",
                sim.bond.pitch, sim.view_time));
        }
        if t < 40 && sim.view_script == FIRE_SCRIPT { samples.push((t, sim.view_script, sim.view_time)); }
        if t % 40 == 0 && sim.view_script == FIRE_SCRIPT { samples.push((t, sim.view_script, sim.view_time)); }
        for l in sim.drones.locks.iter() {
            if l.kind == 0 { max_progress = max_progress.max(l.scan_progress); }
        }
        if sim.drones.channels.on(out_channel) { unlocked_at = Some(t as usize); break; }
    }
    report.push_str(&format!("hold: unlocked_at {unlocked_at:?}, max_progress {max_progress}, anim samples {:?}\n",
        samples.iter().map(|(t, _, v)| (*t, *v)).collect::<Vec<_>>()));
    (walked, unlocked_at, max_progress, samples, report)
}

/// The nearest kind-0 lock on `out` that is still locked — the one a player walks to and aims
/// at (07000012's two keypads sit 0.27 apart [D census]; the crosshair can only ever catch the
/// nearer one, both open the same door).
fn nearest_lock(sim: &Sim, out: u16) -> Option<usize> {
    let mut best: Option<(usize, f32)> = None;
    for (i, l) in sim.drones.locks.iter().enumerate() {
        if l.kind != 0 || l.out != out || l.unlocked { continue; }
        let d = (l.pos[0] - sim.bond.pos[0]).powi(2) + (l.pos[1] - sim.bond.pos[1]).powi(2) + (l.pos[2] - sim.bond.pos[2]).powi(2);
        if best.is_none_or(|(_, bd)| d < bd) { best = Some((i, d)); }
    }
    best.map(|(i, _)| i)
}

#[test]
fn decryptor_12_held_fire_scans_and_opens_the_keypad_lock_from_the_driver_spot() {
    // Difficulty 3: the locks' operative tag (param 6 = 1) must NOT auto-unlock them
    // [M Lock_Create 0x1ae1f8:67-69 — the unlock is Operative (1) only].
    let scene = nf_view::load_with_carry_weapons_difficulty(&iso(), "07000012.bin", false, None, &[0x56], 3)
        .expect("load 07000012 with the Decryptor");
    let sim_probe = scene.sim.as_ref().unwrap();
    let li = sim_probe.drones.locks.iter().position(|l| l.kind == 0 && l.out == 10)
        .expect("07000012 carries the kind-0 keypad locks with out channel 10 [D census m38 #919/#957]");
    let _ = li;
    let lock_of = |sim: &Sim| nearest_lock(sim, 10);
    let (_, unlocked_at, max_progress, samples, report) = real_path_chain(scene, BOND_SPOT, BOND_YAW, BOND_PITCH, lock_of, 10);
    eprintln!("{report}");

    let unlocked_at = unlocked_at.expect("the held decryptor must open the keypad lock (out channel 10)");
    assert!(unlocked_at < 940, "the 0x56's 425-contact scan + settle lands inside the budget (took {unlocked_at})");
    // The digits scanned.
    assert!(max_progress > 0, "the digits must scan while held");
    // The view-model anim keeps advancing while held: the fire script's playhead must leave
    // frame 0 and move across samples (the master bug froze it at 0.0 forever — no tick advanced
    // it and the case-9 stop never ran).
    let fire_samples: Vec<f32> = samples.iter().filter(|(_, s, _)| *s == FIRE_SCRIPT).map(|(_, _, v)| *v).collect();
    assert!(fire_samples.len() >= 3, "the fire script must be playing through the hold (samples {samples:?})");
    assert!(fire_samples.iter().any(|v| *v > 0.0), "the scan anim must advance past frame 0 (samples {fire_samples:?})");
    assert!(fire_samples.windows(2).any(|w| w[0] != w[1]), "the scan anim must keep moving while held (samples {fire_samples:?})");
}

#[test]
fn decryptor_14_held_fire_opens_its_keypad_lock_through_the_same_real_path() {
    // 07000014's decryptor lock: disc placement m54 #1100 "Lock - KeypadLocked"
    // @(-35.84, 2.30, 32.17), params kind 0x0, code 2, out ch 0x2c (44), operative tag 0
    // [D census]. The stand spot is one pace east of the keypad, on its floor.
    let scene = nf_view::load_with_carry_weapons_difficulty(&iso(), "07000014.bin", false, None, &[0x56], 3)
        .expect("load 07000014 with the Decryptor");
    let sim_probe = scene.sim.as_ref().unwrap();
    let lockdump: Vec<String> = sim_probe.drones.locks.iter()
        .map(|l| format!("#{}/kind{}/out{}", l.placement, l.kind, l.out)).collect();
    let l0 = sim_probe.drones.locks.iter().position(|l| l.kind == 0 && l.out == 44)
        .unwrap_or_else(|| panic!("07000014 carries the kind-0 keypad lock with out channel 0x2c [D census m54 #1100]; locks: {lockdump:?}"));
    let lp = sim_probe.drones.locks[l0].pos;
    let spot = [lp[0] + 1.6, lp[1] - 1.0, lp[2]];
    let yaw = (lp[0] - spot[0]).atan2(lp[2] - spot[2]); // face the keypad
    let lock_of = |sim: &Sim| nearest_lock(sim, 44);
    let (_, unlocked_at, _, samples, report) = real_path_chain(scene, spot, yaw, 0.0, lock_of, 44);
    eprintln!("{report}");
    let unlocked_at = unlocked_at.expect("the held decryptor must open 07000014's keypad lock (out ch 0x2c)");
    let fire_samples: Vec<f32> = samples.iter().filter(|(_, s, _)| *s == FIRE_SCRIPT).map(|(_, _, v)| *v).collect();
    assert!(fire_samples.iter().any(|v| *v > 0.0) && fire_samples.windows(2).any(|w| w[0] != w[1]),
        "the scan anim must advance and loop while held (samples {fire_samples:?})");
}
