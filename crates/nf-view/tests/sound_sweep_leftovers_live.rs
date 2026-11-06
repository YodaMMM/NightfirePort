//! sound-sweep-leftovers: the two sound rows the sound-gates sweep marked WRONG and left
//! unfixed, through the real sim path (opt-in via `NF_ISO`).
//!
//! 1. `Player_ChangeSubState` 0x190630:83-85 (disasm 0x1907b4-0x1907e8): the change INTO a
//!    substate in [1..3]∪[5..0x10] runs the hardcoded trio `Sound_StopAllWithId(0x12/0x3d/0xf0)`
//!    — substate 0 skips the outer `0 < param_2`, 4-crouch returns at 0x1907b4 before the calls,
//!    > 0x10 jumps past. The pre-fix port had no substate-change stop at all: a live Laser beam
//!    loop (0x12) survived the swim entry where the game silences it. The port routes every
//!    substate write through `Bond::change_substate` (the game's own funnel), whose outbox the
//!    Sim drains into `drones.audio` next to the `sfx_out` drain.
//! 2. `Bullet_init` 0x12c5a0:286-290: the +2 == 0x45 record — Oddjob's hat ([L SFXOutputData
//!    0x5ef: looped, weight 0]; the record's own +0x60 fire id 0x5ef is the one `FireWeapon`
//!    SKIPS) — plays the LOOPED 0x5ef at the spawn and stores the handle at BU+0xF0;
//!    `Bullet_Delete` 0x12e660:13-16 stops it when the bullet dies (the hat's post-stick expiry
//!    is a silent death but the stop still runs). The pre-fix port played 0x5ef nowhere.

use nf_game::player::Input;

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

/// The camera_surf_eye recipe: the 07000002 fresh boot holds Bond in the intro cut-scene SP #400;
/// the game's own skip action resolves it [M Script_FFwd 0x1dac78].
fn load(iso: &str, level: &str) -> nf_view::scene::Sim {
    let mut scene = nf_view::load(iso, level, false).unwrap_or_else(|e| panic!("load {level}: {e}"));
    let mut sim = scene.sim.take().expect("sim");
    let mut saw_scene = false;
    let mut n = 0u32;
    while n < 400 {
        if sim.disabled { saw_scene = true; } else if saw_scene { break; }
        if sim.disabled { sim.drones.scripts.skip_request = true; }
        sim.step_bond(Input::default(), false, false, false);
        n += 1;
    }
    assert!(saw_scene && !sim.disabled, "the intro cut-scene never released Bond ({n} ticks)");
    sim
}

fn step(sim: &mut nf_view::scene::Sim, input: Input) {
    if sim.disabled { sim.drones.scripts.skip_request = true; }
    sim.step_bond(input, false, false, false);
}

/// The pond's bank [D probe, the fix_swim repro spot], yaw 1.57 walks +X into the pond.
const BANK: [f32; 3] = [-82.3, 5.3, 32.05];

/// The substate-change trio: a live Laser beam loop 0x12 dies at the swim entry (substate 3 is in
/// the game's stop set [M 0x190630:83-85]); the crouch toggle (4) and the walk (0) stop nothing.
#[test]
fn the_laser_loop_dies_at_the_swim_entry() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    // The Laser 0x4E in Bond's loadout (level_start's own 0x4e slot: clip 500 [M level_start]).
    let inv = nf_game::pickup::Inventory::level_start(0x73, 34);
    let rs = nf_game::player::RamSave::capture(&Default::default(), 0, &inv, &nf_game::triggers::Channels::default());
    let mut scene = nf_view::load_with_carry(&iso, "07000002.bin", false, Some(&rs)).expect("load 07000002");
    let mut sim = scene.sim.take().expect("sim");
    let mut saw_scene = false;
    let mut n = 0u32;
    while n < 400 {
        if sim.disabled { saw_scene = true; } else if saw_scene { break; }
        if sim.disabled { sim.drones.scripts.skip_request = true; }
        sim.step_bond(Input::default(), false, false, false);
        n += 1;
    }
    assert!(saw_scene && !sim.disabled, "the intro never released Bond ({n} ticks)");

    // The helper's branch table [M 0x190630:83-85]: 4-crouch and 0 stop nothing, every
    // [1..3]∪[5..0x10] value queues the trio.
    sim.bond.substate_stops.clear();
    sim.bond.change_substate(4);
    assert!(sim.bond.substate_stops.is_empty(), "the crouch (4) change stops nothing [M 0x1907b4]");
    sim.bond.change_substate(0);
    assert!(sim.bond.substate_stops.is_empty(), "the walk (0) change stops nothing [M `0 < param_2`]");
    sim.bond.change_substate(6);
    assert_eq!(sim.bond.substate_stops, vec![0x12, 0x3d, 0xf0], "the wire (6) entry queues the trio");
    sim.bond.change_substate(0);
    sim.bond.substate_stops.clear();

    // Bond at the pond bank, the Laser equipped through the real switch input.
    sim.bond = nf_game::player::Bond::stand_at(&sim.world, BANK, 1.57);
    let draw = Input { switch_to: Some(0x4e), ..Default::default() };
    for _ in 0..40 { step(&mut sim, draw); }
    assert_eq!(sim.gun.weapon, 0x4e, "the Laser must equip; got {:#x}", sim.gun.weapon);

    // Hold the fire until the beam loop 0x12 is live in the pool [L 0x060004e9 op-1 → 0x12].
    let mut live = false;
    for _ in 0..90 {
        step(&mut sim, Input { forward: 0.0, ..Default::default() }, );
        sim.step_bond(Input::default(), true, false, false);
        if sim.drones.audio.active_ids().contains(&0x12) { live = true; break; }
    }
    assert!(live, "the Laser's beam loop 0x12 never went live (active {:?})",
        sim.drones.audio.active_ids());

    // Walk off the bank into the pond on the real path (the fix_swim recipe): the entry INTO
    // substate 3 must stop the loop.
    let walk = Input { forward: 1.0, ..Default::default() };
    let mut entered = false;
    for _ in 0..600 {
        step(&mut sim, walk);
        if sim.bond.substate == nf_game::player::SS_SWIM { entered = true; break; }
    }
    assert!(entered, "walking off the bank never entered the swim (pos {:?})", sim.bond.pos);
    assert!(!sim.drones.audio.active_ids().contains(&0x12),
        "the beam loop 0x12 survived the swim entry — the game's ChangeSubState trio stops it \
         [M 0x190630:83-85] (active {:?})", sim.drones.audio.active_ids());
}

/// `Bullet_init` 0x12c5a0:286-290: the 0x45 record's spawn plays the looped 0x5ef, and the
/// hat's silent post-stick expiry stops it (`Bullet_Delete` 0x12e660:13-16).
#[test]
fn the_hat_spawn_plays_the_0x5ef_loop_and_the_delete_stops_it() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut sim = load(&iso, "07000001.bin");
    // Equip the hat record through the gun slot (the record is MP-carried; the throw path is the
    // player's real `Player_WeaponInitBullet` → `Bullet_init` chain on a real level). The clip
    // covers the release: `Bullet_init` runs per throw.
    sim.gun.weapon = 0x45;
    sim.gun.clip = 4;
    let mut thrown = false;
    for _ in 0..120 {
        step(&mut sim, Input::default());
        sim.step_bond(Input::default(), true, false, false);
        if !sim.drones.grenades.is_empty() { thrown = true; break; }
    }
    assert!(thrown, "the 0x45 throw never spawned a bullet");
    let g = &sim.drones.grenades[0];
    assert_eq!(g.weapon_type, 0x45, "the thrown record is the hat");
    assert!(sim.drones.audio.live_ids().contains(&0x5ef),
        "the hat spawn never played the 0x5ef loop [M Bullet_init 0x12c5a0:286-290] (live {:?})",
        sim.drones.audio.live_ids());
    assert!(g.loop_handle != 0, "the handle is stored (BU+0xF0) [M 0x12c5a0:290]");

    // The stick + the 1500-tick silent life ([L record 0x45: +0x70 & 0x100; HAT_STICK_LIFE]),
    // then the delete flag: `Bullet_Delete`'s handle stop must retire the loop.
    let mut died = false;
    for _ in 0..2400 {
        step(&mut sim, Input::default());
        if sim.drones.grenades.iter().all(|g| g.state >= 2) || sim.drones.grenades.is_empty() {
            died = true;
            break;
        }
    }
    assert!(died, "the stuck hat never expired");
    assert!(!sim.drones.audio.live_ids().contains(&0x5ef),
        "the 0x5ef loop survived the hat's delete — `Bullet_Delete` stops BU+0xF0 [M 0x12e660:13-16] \
         (live {:?})", sim.drones.audio.live_ids());
}
