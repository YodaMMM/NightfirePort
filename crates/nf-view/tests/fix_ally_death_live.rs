//! The killer of a protected actor, on the real levels. Opt-in via NF_ISO.
//!
//! `DroneFunc_OnInitDeath` 0x145bd0 reads `killer obj+0xFF == 3`, and class 3 is the player
//! [M `Player_Init` 0x1910f8:76]; drones are class 2 [M 0x14d7c8:14]. An actor with prop 0x54 and no
//! prop 0x53 fails the mission only when Bond kills it. Its death channel still applies whoever kills
//! it: `DroneFunc_SetDeathChannel` 0x13e660 writes ch 0x60 from param 12 = 0x60, and param 12 = 0x63
//! is `SetMissionFailReason` by team [M :21-44].
//!
//! The kills here are the same blast with only its owner changed: `grenade.owner` usize::MAX is Bond's
//! throw, a drone index is that guard's [M `Drone_ExplosiveHit` 0x13aab8:20 stamps +0x2B4 with the
//! blast's owner]. `bonds_round_kills…` covers the bullet route.
//!
//! Run: NF_ISO=… cargo test -p nf-view --test fix_ally_death_live -- --nocapture --test-threads=1

#[path = "common/walker.rs"]
mod walker;

use nf_game::player::Input;
use nf_view::scene::{Scene, Sim};
use walker::Walker;

const KILLED_CIVILIANS: u32 = 0x0400_0034;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

/// The 00 Agent roster: the placement gate [M 0x136be0:49] then holds every drone of the census.
fn load(level: u32) -> Scene {
    let mut scene = nf_view::load_with_carry_difficulty(&iso(), &format!("{level:08x}.bin"), false, None, 3).expect("load");
    let sim = scene.sim.as_mut().unwrap();
    // The opening cutscene: the AI freezes under it [M `Drone_CollisionHandler` 0x140968:62].
    for _ in 0..6000 {
        sim.step_bond(Input::default(), false, false, false);
        if !sim.disabled && sim.drones.npc_enabled { break; }
    }
    assert!(sim.drones.npc_enabled, "{level:08x}: the AI runs");
    scene
}

fn step(sim: &mut Sim, ticks: u32) { for _ in 0..ticks { sim.step_bond(Input::default(), false, false, false); } }

fn actor(sim: &Sim, placement: usize) -> usize {
    sim.drones.list.iter().position(|d| d.placement == placement).expect("the census actor is placed")
}

fn a_guard(sim: &Sim) -> usize {
    sim.drones.list.iter().position(|d| d.team == 1 && d.present).expect("a guard exists")
}

/// Writes the actor's own wait channel, the one the level raises when it joins the scene: an idle
/// `WaitSwitch` drone drops every blast, as `DSTATE_Global` has no case 9 [M 0x15b830:59-61].
fn wake(sim: &mut Sim, i: usize) {
    let ch = sim.drones.list[i].switch_channel;
    if ch != 0 {
        sim.drones.channels.set(ch as u32, 1, sim.drones.frame);
        sim.drones.list[i].channel_on = true;
    }
    for _ in 0..300 {
        if sim.drones.list[i].enabled { break; }
        step(sim, 1);
    }
    let d = &sim.drones.list[i];
    assert!(d.enabled, "placement {} wakes, ch {ch:#x}, state {:#x}", d.placement, d.state);
}

/// A blast at the actor's feet, owned by `owner`.
fn blast_at(sim: &mut Sim, pos: [f32; 3], owner: usize, damage: f32) {
    sim.drones.grenades.push(nf_game::grenade::Grenade {
        pos, velocity: [0.0, 0.0, 0.0], speed: 0.0,
        travelled: 0.0, range: 60.0, damage, radius: 1.0, weapon_type: 0x34, mesh: 0x0200_0726, owner, state: 0, bounces: 0, smoke_popped: false, resting: false,
        stuck: false, seek_target: None,
        phase: None,
        burn: 0,
        defused: false,
        loop_handle: 0,
        born: 0,
    });
}

/// Kills the actor with one blast and returns whether the mission failed in the next seconds.
fn kill_and_watch(level: u32, placement: usize, by_bond: bool) -> (bool, u32) {
    let mut scene = load(level);
    let sim = scene.sim.as_mut().unwrap();
    let i = actor(sim, placement);
    assert!(sim.drones.list[i].present && !sim.drones.list[i].corpse, "{level:08x} #{placement} starts alive");
    wake(sim, i);
    assert!(!sim.drones.channels.on(99) && !sim.drones.channels.on(0x60), "{level:08x}: the mission starts open");
    let owner = if by_bond { usize::MAX } else { a_guard(sim) };
    let p = sim.drones.list[i].pos;
    blast_at(sim, [p[0], p[1] + 0.5, p[2]], owner, 100.0);
    let mut dead = false;
    for _ in 0..300 {
        step(sim, 1);
        if sim.drones.list[i].corpse { dead = true; break; }
    }
    assert!(dead, "{level:08x} #{placement}: the blast kills the actor");
    assert_eq!(sim.drones.list[i].hit_by_player, by_bond, "the last hit is the blast's owner [M 0x13aab8:20]");
    // Long enough for the death animation's channel write and the monitor's hook.
    let mut failed = false;
    for _ in 0..600 {
        step(sim, 1);
        if sim.drones.channels.on(99) { failed = true; break; }
    }
    (failed, sim.drones.mission.fail_label)
}

/// (level, placement, Bond's kill fails, a guard's kill fails). The props and param 12 are the disc's
/// [D census over `nf_view::dev::LEVELS`]:
/// - 01 #161, 02 #463: prop 0x54 alone, param 12 = 0x60, so a guard's kill fails through the channel;
/// - 07 #349: a party guest, prop 0x53, which fails on any killer;
/// - 06 #494: param 12 = 0x63 with prop 0x53;
/// - 12 #23, #25, 16 #287: prop 0x54 alone, no death channel;
/// - 12 #1: prop 0x54 alone with param 12 = 0x63, so a guard's kill fails too.
const CASES: [(u32, usize, bool, bool); 8] = [
    (0x0700_0001, 161, true, true),
    (0x0700_0002, 463, true, true),
    (0x0700_0006, 494, true, true),
    (0x0700_0007, 349, true, true),
    (0x0700_0012, 23, true, false),
    (0x0700_0012, 25, true, false),
    (0x0700_0012, 1, true, true),
    (0x0700_0016, 287, true, false),
];

#[test]
fn each_protected_actor_fails_the_mission_on_the_games_killers() {
    let mut wrong = Vec::new();
    for (level, placement, bond_fails, guard_fails) in CASES {
        for (by_bond, want) in [(true, bond_fails), (false, guard_fails)] {
            let (failed, label) = kill_and_watch(level, placement, by_bond);
            eprintln!("{level:08x} #{placement} by {}: failed {failed} label {label:#x}", if by_bond { "Bond" } else { "a guard" });
            if failed != want { wrong.push((level, placement, by_bond, failed)); }
            if failed { assert_eq!(label, KILLED_CIVILIANS, "{level:08x} #{placement}: team 2/3 reasons [M 0x13e560]"); }
        }
    }
    assert!(wrong.is_empty(), "(level, placement, by Bond, failed) against the game: {wrong:x?}");
}

/// 07000007's one guest without prop 0x53 in its normal block, #413, takes block 5 [D]: mode 0x15,
/// `DeleteMe` [D `DroneTypeSettings`], so it is gone before anyone can kill it.
#[test]
fn the_07_block_5_guest_deletes_itself() {
    let scene = load(0x0700_0007);
    let sim = scene.sim.as_ref().unwrap();
    assert!(!sim.drones.list[actor(sim, 413)].present);
}

/// The 07000008 ally carries prop 0x54 too, but her skin takes no damage [`Drone::damageable`], so
/// neither killer has a kill to make.
#[test]
fn the_08_ally_survives_both_blasts() {
    for owner_is_bond in [true, false] {
        let mut scene = load(0x0700_0008);
        let sim = scene.sim.as_mut().unwrap();
        let i = actor(sim, 265);
        let owner = if owner_is_bond { usize::MAX } else { a_guard(sim) };
        let p = sim.drones.list[i].pos;
        blast_at(sim, [p[0], p[1] + 0.5, p[2]], owner, 100.0);
        step(sim, 300);
        assert!(!sim.drones.list[i].corpse, "she takes no damage");
        assert!(!sim.drones.channels.on(0x60), "nothing to fail");
    }
}

/// +0x2B4 is the LAST hitter [M 0x13a9f8:23, 0x13aab8:20]. Bond wounds the actor, a guard's blast
/// finishes it: the guard is the killer, so prop 0x54 alone does not fail. The sister order fails.
#[test]
fn the_last_hit_names_the_killer() {
    for (first, second, want_fail) in [(usize::MAX, 0usize, false), (0usize, usize::MAX, true)] {
        let mut scene = load(0x0700_0012);
        let sim = scene.sim.as_mut().unwrap();
        let i = actor(sim, 23);
        wake(sim, i);
        let guard = a_guard(sim);
        let resolve = |o: usize| if o == usize::MAX { o } else { guard };
        let p = sim.drones.list[i].pos;
        let at = [p[0], p[1] + 0.5, p[2]];
        blast_at(sim, at, resolve(first), 0.2);
        let hp = sim.drones.list[i].health;
        for _ in 0..300 { step(sim, 1); if sim.drones.list[i].health < hp { break; } }
        assert!(sim.drones.list[i].health < hp, "the first blast wounds");
        assert!(!sim.drones.list[i].corpse, "the first blast does not kill");
        let p = sim.drones.list[i].pos;
        blast_at(sim, [p[0], p[1] + 0.5, p[2]], resolve(second), 100.0);
        for _ in 0..300 { step(sim, 1); if sim.drones.list[i].corpse { break; } }
        assert!(sim.drones.list[i].corpse, "the second blast kills");
        step(sim, 120);
        assert_eq!(sim.drones.channels.on(99), want_fail, "first {first:x}, second {second:x}");
    }
}

/// Bond's own round on the 07000012 actor at the start of the level: the bullet route stamps the
/// player as the killer [M `Drone_BulletHit` 0x13a9f8:23] and prop 0x54 + param 12 = 0x63 fail the
/// mission, "Killed Civilians".
#[test]
fn bonds_round_kills_the_12_civilian_and_fails_the_mission() {
    let mut w = Walker::new(load(0x0700_0012));
    let i = actor(w.scene.sim.as_ref().unwrap(), 1);
    w.shoot(i, "the 12 civilian");
    let mut failed = false;
    for _ in 0..600 {
        w.tick(Default::default(), false);
        if w.scene.sim.as_ref().unwrap().drones.channels.on(99) { failed = true; break; }
    }
    let sim = w.scene.sim.as_ref().unwrap();
    assert!(sim.drones.list[i].corpse, "the rounds kill him");
    assert!(failed, "killing him fails the mission [M 0x145bd0 reason 1]");
    assert_eq!(sim.drones.mission.fail_label, KILLED_CIVILIANS);
}

fn dxz(a: [f32; 3], b: [f32; 3]) -> f32 { ((a[0] - b[0]).powi(2) + (a[2] - b[2]).powi(2)).sqrt() }

fn steer(scene: &mut Scene, gi: usize, fire: bool) {
    let (turn, look) = {
        let sim = scene.sim.as_ref().unwrap();
        let bp = sim.bond.pos;
        let g = sim.drones.list[gi].pos;
        let dy = g[1] - (bp[1] + sim.bond.eye - sim.bond.crouch_drop);
        let err = (g[0] - bp[0]).atan2(g[2] - bp[2]) - sim.bond.yaw;
        let want_pitch = (dy / dxz(bp, g).max(0.5)).atan() / std::f32::consts::FRAC_PI_2;
        (err.clamp(-0.08, 0.08), (want_pitch - sim.bond.pitch).clamp(-0.08, 0.08))
    };
    scene.sim.as_mut().unwrap().step_bond(Input { turn, look, ..Default::default() }, fire, false, false);
}

/// Stunning an escorted ally fails the mission: `DSTATE_Taser` 0x162b98:39-41, mode +0xC5 0x11/0x0C
/// → reason 0xD, the default arm: ch 99, the label untouched [M 0x13e560].
#[test]
fn tasing_the_01_ally_fails_the_mission_without_a_label() {
    let mut scene = load(0x0700_0001);
    let i = actor(scene.sim.as_ref().unwrap(), 161);
    wake(scene.sim.as_mut().unwrap(), i);
    assert_eq!(scene.sim.as_ref().unwrap().drones.list[i].mode, 0x0c);
    let at = scene.sim.as_ref().unwrap().drones.list[i].pos;
    let mut dev = nf_view::dev::Dev { scenario: None, ..Default::default() };
    dev.teleport(&mut scene, [at[0] - 2.5, at[1], at[2]], 0.0);
    let draw = Input { switch_to: Some(0x4a), ..Default::default() };
    for _ in 0..60 {
        scene.sim.as_mut().unwrap().step_bond(draw, false, false, false);
        if scene.sim.as_ref().unwrap().gun.weapon == 0x4a { break; }
    }
    assert_eq!(scene.sim.as_ref().unwrap().gun.weapon, 0x4a, "the Stunner is in hand");
    for _ in 0..130 { steer(&mut scene, i, false); }
    let label0 = scene.sim.as_ref().unwrap().drones.mission.fail_label;
    // The fall needs more tases than 2 x health [M 0x1469b8:76-88]; a wounded ally falls sooner.
    scene.sim.as_mut().unwrap().drones.list[i].health = 2.0;
    let mut fell = false;
    for t in 0..4000 {
        steer(&mut scene, i, t % 12 == 0);
        if scene.sim.as_ref().unwrap().drones.list[i].state == nf_game::drone::state::TASER { fell = true; break; }
    }
    assert!(fell, "the held tase drops her [M 0x1469b8 case 7]");
    step(scene.sim.as_mut().unwrap(), 2);
    let sim = scene.sim.as_ref().unwrap();
    assert!(sim.drones.channels.on(99), "reason 0xD writes ch 99 [M 0x13e560]");
    assert_eq!(sim.drones.mission.fail_label, label0, "and leaves the label alone");
}
