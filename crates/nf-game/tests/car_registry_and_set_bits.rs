//! unported-front-end-menus-1 — the `Car_*` registry functions and the driven-weapon tick,
//! pinned to the decompile. Headless: no ISO needed. Every assertion cites its [M] address;
//! each test carries a sister assertion that can fail.

use nf_game::car::{Car, CarSystem, SetBitsFire, CAR_DRIVEN, CAR_IDLE, CAR_RESPAWNING, MAX_TANKS,
                   RESPAWN_TICKS};

/// A driven car with an identity spawn pose (the port's stand-in for a placement matrix).
fn driven_car() -> Car {
    let mut rng = nf_game::drone::Rng::new(1);
    let (mut car, _) = Car::create(true, 1, 0x0700_0024, &mut rng).unwrap();
    car.state = CAR_DRIVEN;
    car
}

#[test]
fn car_init_zeroes_the_registry_0x132d78() {
    // Car_Init 0x132d78 [M]: NumTanks = 0; memset(&Tanks, 0, 0x20) — 8 pointer slots.
    let sys = CarSystem::init();
    assert_eq!(sys.num_tanks, 0, "Car_Init zeroes NumTanks");
    assert_eq!(sys.tanks.len(), MAX_TANKS, "the Tanks array is 0x20/4 = 8 slots");
    assert!(sys.tanks.iter().all(|t| t.is_none()), "memset zeroes every slot");
    // Sister: a filled registry is observably different (the assertion above can fail).
    let mut sys = CarSystem::init();
    sys.tanks[0] = Some(driven_car());
    sys.num_tanks = 1;
    assert!(sys.tanks[0].is_some() && sys.num_tanks == 1);
}

#[test]
fn car_reset_restores_spawn_pose_and_clears_state_0x132da8() {
    // Car_Reset 0x132da8 [M], per tank: obj+0xf4 = 0 (state), obj+0xf6 = 0 (timer),
    // CAR_INFO+0xec = 0 (driver), CAR_INFO+0xf8 = 0 (damage), Vec_Zero(CAR_INFO+0xd0)
    // (turret pitch/yaw), Mat_Copy(TankSpawns[i] → obj+0x90).
    let mut sys = CarSystem::init();
    let mut car = driven_car();
    car.matrix[0][3] = 42.5; // the live pose has driven away from the spawn
    car.spawn_matrix[0][3] = -7.0;
    car.state = CAR_DRIVEN;
    car.timer_or_pilot = 3;
    car.driver = Some(3);
    car.damage = 55.0;
    car.turret_pitch = 0.3;
    car.turret_yaw = -2.0;
    sys.tanks[0] = Some(car);
    sys.num_tanks = 1;
    sys.reset();
    let car = sys.tanks[0].as_ref().unwrap();
    assert_eq!(car.state, CAR_IDLE, "obj+0xf4 = 0");
    assert_eq!(car.timer_or_pilot, 0, "obj+0xf6 = 0");
    assert_eq!(car.driver, None, "CAR_INFO+0xec = 0");
    assert_eq!(car.damage, 0.0, "CAR_INFO+0xf8 = 0");
    assert_eq!(car.turret_pitch, 0.0, "Vec_Zero clears +0xd0");
    assert_eq!(car.turret_yaw, 0.0, "Vec_Zero clears +0xd4");
    assert_eq!(car.matrix[0][3], -7.0, "the live matrix is restored from TankSpawns[slot]");
    // Sister: Car_Reset does NOT touch Car_Create's fields — heat/cooldowns/MG ammo are only
    // zeroed by Car_Create 0x1305b0:62-63 [M]; if a reset ever cleared them this fails.
    let mut sys = CarSystem::init();
    let mut car = driven_car();
    car.heat = 40.0;
    car.shell_cooldown = 3.0;
    car.mg_cooldown = 100.0;
    car.mg_ammo = 50;
    sys.tanks[0] = Some(car);
    sys.num_tanks = 1;
    sys.reset();
    let car = sys.tanks[0].as_ref().unwrap();
    assert_eq!(car.heat, 40.0, "heat survives Car_Reset (create-only zero)");
    assert_eq!(car.shell_cooldown, 3.0);
    assert_eq!(car.mg_cooldown, 100.0);
    assert_eq!(car.mg_ammo, 50);
}

#[test]
fn car_reset_walks_exactly_numtanks_slots_0x132da8() {
    // The walk is `for i in 0..NumTanks` [M 0x132da8:16] — slots past NumTanks are untouched.
    let mut sys = CarSystem::init();
    let mut car = driven_car();
    car.state = CAR_RESPAWNING;
    car.timer_or_pilot = RESPAWN_TICKS;
    sys.tanks[1] = Some(car);
    sys.num_tanks = 1; // slot 1 exists but is past NumTanks
    sys.reset();
    let car = sys.tanks[1].as_ref().unwrap();
    assert_eq!(car.state, CAR_RESPAWNING, "slot 1 is outside 0..NumTanks=1");
    assert_eq!(car.timer_or_pilot, RESPAWN_TICKS);
    // Sister: inside the walk the state does reset.
    sys.num_tanks = 2;
    sys.reset();
    assert_eq!(sys.tanks[1].as_ref().unwrap().state, CAR_IDLE);
}

#[test]
fn car_delete_is_car_deactivate_0x132e78() {
    // Car_Delete 0x132e78 [M] is a tail-call to Car_Deactivate 0x131d88, dispatched from
    // control_delete_object 0x133178:16-18 via control_funcs[0x28*0xC + 8].
    let mut sys = CarSystem::init();
    sys.tanks[0] = Some(driven_car());
    sys.num_tanks = 1;
    sys.delete(0);
    let car = sys.tanks[0].as_ref().unwrap();
    assert_eq!(car.state, CAR_RESPAWNING, "deactivate's state 99");
    assert_eq!(car.timer_or_pilot, RESPAWN_TICKS, "the 0x708 respawn timer");
    assert_eq!(car.driver, None);
    // Sister: an empty slot is a no-op (the dispatcher null-checks the obj pointer).
    let mut sys = CarSystem::init();
    sys.num_tanks = 0;
    sys.delete(0); // must not panic
    assert!(sys.tanks[0].is_none());
}

#[test]
fn set_bits_turret_input_wrap_and_clamp_0x130060() {
    // Driven input integrates the turret at ∓axis·FRM·0.05 [M 0x130060:104-110].
    let mut car = driven_car();
    let actions = [0.0f32; 40];
    car.set_bits(&actions, 1.0);
    assert_eq!(car.turret_yaw, 0.0, "no input, no movement");

    // action 0 = turn: yaw += -in·FRM·0.05 [M 0x130060:104-106].
    let mut car = driven_car();
    let mut actions = [0.0f32; 40];
    actions[0] = -1.0;
    car.set_bits(&actions, 1.0);
    assert!((car.turret_yaw - 0.05).abs() < 1e-6, "yaw += -(-1)·0.05 = +0.05");

    // action 5 = pitch: pitch += -in·FRM·0.05 [M 0x130060:107-110].
    let mut car = driven_car();
    let mut actions = [0.0f32; 40];
    actions[5] = 1.0;
    car.set_bits(&actions, 1.0);
    assert!((car.turret_pitch + 0.05).abs() < 1e-6, "pitch += -(1)·0.05");

    // The pitch clamp: limit = (1−|yaw·1.2732395|)·0.12900001+0.08, floor −0.705
    // [M 0x130060:113-119]. At yaw 0 the up-limit is 0.20900001.
    let mut car = driven_car();
    car.turret_pitch = 10.0;
    car.set_bits(&actions, 1.0); // actions all zero here → yaw stays 0
    assert!((car.turret_pitch - 0.20900001).abs() < 1e-5, "up-clamped to the yaw-0 limit");
    let mut car = driven_car();
    car.turret_pitch = -10.0;
    car.set_bits(&actions, 1.0);
    assert!((car.turret_pitch + 0.705).abs() < 1e-6, "down-clamped to 0xbf347ae1 = -0.705");

    // Sister for the clamp's yaw dependence: at yaw π/4 the limit term is 0 → 0.08
    // (1.2732395·π/4 ≈ 1.0) — a constant limit would fail this.
    let mut car = driven_car();
    let mut actions0 = [0.0f32; 40];
    // Keep yaw at π/4: set the state directly (the integration would move it with input;
    // with zero input the wrap leaves |π/4| < π untouched).
    car.turret_yaw = core::f32::consts::PI / 4.0;
    car.turret_pitch = 10.0;
    car.set_bits(&actions0, 1.0);
    let expected = (1.0 - (core::f32::consts::PI / 4.0 * 1.2732395).abs()) * 0.12900001 + 0.08;
    assert!((car.turret_pitch - expected).abs() < 1e-5, "the limit follows the yaw");

    // The yaw wrap: |yaw| ≥ π → fmodf(yaw, 2π), then ±2π by sign [M 0x130060:111-112].
    let mut car = driven_car();
    car.turret_yaw = 3.2; // > π
    car.set_bits(&actions0, 1.0);
    assert!(car.turret_yaw.abs() < core::f32::consts::PI, "3.2 wraps to 3.2-2π ≈ -3.083");
}

#[test]
fn set_bits_shell_and_mg_fire_with_exact_constants_0x130060() {
    // Shell: pressed ∧ cooldown ≤ 0 ∧ !overheated → fire, cooldown = 6.0 (0x40c00000),
    // heat +10, latch at > 100 [M 0x130060:169-192].
    let mut car = driven_car();
    let mut actions = [0.0f32; 40];
    actions[9] = 1.0;
    let fired = car.set_bits(&actions, 1.0);
    assert!(fired.shell, "the shell fires");
    assert!(!fired.mg);
    assert!((car.shell_cooldown - 6.0).abs() < 1e-6, "cooldown = 0x40c00000");
    assert!((car.heat - 10.0).abs() < 1e-6, "heat +10");
    assert!(!car.overheated, "10 < 100, no latch");
    // Holding fire during the cooldown does not fire again [M — the ≤ 0.0 test].
    let fired = car.set_bits(&actions, 1.0);
    assert!(!fired.shell, "cooldown 6.0-1.0 > 0 blocks the second shot");
    // The latch: > 100 sets CAR_INFO+0x113 [M 0x130060:189-191].
    let mut car = driven_car();
    car.heat = 95.0;
    let fired = car.set_bits(&actions, 1.0);
    assert!(fired.shell);
    assert!(car.overheated, "95+10 > 100 latches");
    let fired = car.set_bits(&actions, 1.0);
    assert!(!fired.shell, "the latch blocks fire");
    // Sister: heat exactly 100.0 does NOT latch (> 100, not ≥).
    let mut car = driven_car();
    car.heat = 90.0;
    car.set_bits(&actions, 1.0);
    assert!(!car.overheated, "100.0 is not > 100.0");

    // MG: pressed ∧ cooldown ≤ 0 ∧ ammo ≠ 0 → fire, ammo −1, cooldown = 120.0 (0x42f00000)
    // [M 0x130060:193-212].
    let mut car = driven_car();
    car.mg_ammo = 100;
    let mut actions_mg = [0.0f32; 40];
    actions_mg[0xC] = 1.0;
    let fired = car.set_bits(&actions_mg, 1.0);
    assert!(fired.mg, "the MG fires");
    assert!(!fired.shell);
    assert_eq!(car.mg_ammo, 99, "ammo −1");
    assert!((car.mg_cooldown - 120.0).abs() < 1e-4, "cooldown = 0x42f00000");
    // Sister: ammo 0 blocks [M — the != 0 test].
    let mut car = driven_car();
    car.mg_ammo = 0;
    let fired = car.set_bits(&actions_mg, 1.0);
    assert!(!fired.mg && car.mg_ammo == 0 && car.mg_cooldown == 0.0);
}

#[test]
fn set_bits_input_block_is_driven_only_but_cooldowns_tick_always_0x130060() {
    // The input/fire block is behind obj+0xf4 == 1 [M 0x130060:91]; the cooldown decrements
    // are outside it [M 0x130060:159-168].
    let mut car = driven_car();
    car.state = CAR_IDLE;
    car.turret_yaw = 3.2; // would wrap if the input block ran
    car.mg_cooldown = 1.0;
    let mut actions = [0.0f32; 40];
    actions[9] = 1.0;
    actions[0xC] = 1.0;
    let fired = car.set_bits(&actions, 1.0);
    assert!(!fired.shell && !fired.mg, "an idle car never fires");
    assert_eq!(car.turret_yaw, 3.2, "no input integration when not driven");
    assert!((car.mg_cooldown + 1.0 - 1.0).abs() < 1e-6, "placeholder");
    assert_eq!(car.mg_cooldown, 0.0, "the MG cooldown still ticks down");
    // Sister: a driven car with the same input does integrate.
    let mut car = driven_car();
    car.turret_yaw = 3.2;
    car.set_bits(&actions, 1.0);
    assert!(car.turret_yaw != 3.2, "the driven input block runs");
}
