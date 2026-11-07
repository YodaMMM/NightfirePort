//! weapon-rules: the player weapon rules that sit between the trigger and the switch request, through
//! the real input path (`Sim::step_bond`) on the disc's own records. Opt-in via `NF_ISO`.
//!
//! - the dry gun [M Player_WeaponFiring 0x1a5460 release arm → Player_HandleHasNoAmmo 0x1a2ba8]: trigger
//!   up, no round, nothing carried → the twin (`+0x68 & 0x10`) or `Player_GetBestWeapon` 0x1a9680;
//! - the cycle walk [M Player_WeaponChange 0x1a6f80]: from the request's base, with its own refusals, and
//!   a walk that finds nothing leaves the hand alone;
//! - the view-state gate [M 0x1a5460:631-648] and the aim bit's clears [M Player_Weapon 0x1a24c8:66-99];
//! - the fire press during a guided ride [M 0x1a5460 → Bullet_handle_object_destruction 0x12cd98].

use nf_game::player::Input;
use nf_view::panel;
use nf_view::scene::Scene;

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

fn tick(scene: &mut Scene, input: Input, fire: bool, reload: bool) {
    let sim = scene.sim.as_mut().unwrap();
    sim.drones.health.health = 100.0;
    sim.step_bond(input, fire, reload, false);
}

fn wait_enabled(scene: &mut Scene, cap: u32) {
    for _ in 0..cap {
        if scene.sim.as_ref().map_or(true, |s| !s.disabled) { return; }
        tick(scene, Input::default(), false, false);
    }
}

fn idle(scene: &Scene) -> bool {
    let s = scene.sim.as_ref().unwrap();
    s.gun.switch_state == 0 && s.gun.reload_state == 0 && !s.gun.fire_state && s.gun.switch_request == s.gun.weapon
}

fn wait_idle(scene: &mut Scene, cap: u32) -> bool {
    for _ in 0..cap {
        if idle(scene) { return true; }
        tick(scene, Input::default(), false, false);
    }
    idle(scene)
}

fn give(scene: &mut Scene, id: u8) {
    let q = vec![("do".to_string(), "weapon".to_string()), ("id".to_string(), format!("0x{id:02x}"))];
    let (msg, sw) = panel::apply(scene, &q);
    assert_eq!(sw, Some(id), "the give of {id:#04x} was refused: {msg}");
}

fn draw(scene: &mut Scene, id: u8) {
    for _ in 0..200 {
        tick(scene, Input { switch_to: Some(id), ..Default::default() }, false, false);
        if scene.sim.as_ref().unwrap().gun.weapon == id { break; }
    }
    assert_eq!(scene.sim.as_ref().unwrap().gun.weapon, id, "{id:#04x} never drew");
    assert!(wait_idle(scene, 300), "{id:#04x}'s draw never settled");
}

fn give_and_draw(scene: &mut Scene, id: u8) { give(scene, id); draw(scene, id); }

/// One real press with the release after it, then the refire run out.
fn press(scene: &mut Scene) {
    tick(scene, Input::default(), true, false);
    for _ in 0..16 { tick(scene, Input::default(), false, false); }
}

/// The held family's carried pool and magazine down to `clip` rounds.
fn set_rounds(scene: &mut Scene, clip: i16, carried: i16) {
    let sim = scene.sim.as_mut().unwrap();
    let w = sim.gun.weapon as usize;
    let (slot, t) = (nf_game::pickup::Inventory::clip_slot(&sim.weapons, w), sim.weapons.ammo_type(w));
    sim.drones.inventory.clip[slot] = clip;
    sim.drones.inventory.carried[t] = carried;
}

fn load(iso: &str, weapons: &[u8]) -> Scene {
    let mut scene = nf_view::load_weapons(iso, "07000005.bin", false, weapons).expect("load 07000005");
    wait_enabled(&mut scene, 7200);
    assert!(wait_idle(&mut scene, 400), "the level's first draw never settled");
    scene
}

/// The last round of the last magazine: on the trigger's release the hand moves on to the best
/// weapon left, here down the `BestWeapon` table from the SG5 (rank 5) to the P2K (rank 15)
/// [M 0x1a5460 release arm; 0x1a2ba8 tail; 0x1a9680]. Sister: while the trigger stays down nothing
/// switches — the arm is the RELEASE path; and rounds in the pool reload instead.
#[test]
fn a_dry_gun_switches_on_the_release() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = load(&iso, &[0x16, 0x17]);
    give_and_draw(&mut scene, 0x16);
    set_rounds(&mut scene, 1, 0);
    for _ in 0..30 { tick(&mut scene, Input::default(), true, false); }
    {
        let s = scene.sim.as_ref().unwrap();
        assert_eq!((s.gun.clip, s.gun.spare), (0, 0), "the last round went");
        assert_eq!(s.gun.switch_request, 0x16, "a held trigger holds the dry gun [M the arm reads the level at 0]");
    }
    // The last shot's fire script plays out first (the arm wants view state 0).
    for _ in 0..300 {
        tick(&mut scene, Input::default(), false, false);
        if scene.sim.as_ref().unwrap().gun.weapon != 0x16 { break; }
    }
    assert!(wait_idle(&mut scene, 300), "the switch never settled");
    assert_eq!(scene.sim.as_ref().unwrap().gun.weapon, 0x02, "the release moves on to the best weapon left");

    // Sister: rounds in the pool reload, the hand stays.
    give_and_draw(&mut scene, 0x16);
    set_rounds(&mut scene, 1, 30);
    press(&mut scene);
    for _ in 0..300 {
        tick(&mut scene, Input::default(), false, false);
        if scene.sim.as_ref().unwrap().gun.reload_state != 0 { break; }
    }
    let s = scene.sim.as_ref().unwrap();
    assert_eq!((s.gun.weapon, s.gun.switch_request), (0x16, 0x16), "a reload is not a dry gun");
    assert_ne!(s.gun.reload_state, 0, "the auto-reload took the release");
}

/// `Player_GetBestWeapon` hands the family's equipped twin back: the P2K left on its silencer comes
/// back silenced [M 0x1a9680:18-23 — base + BL+0x1BC[base]], where the base id alone would strip it.
#[test]
fn the_best_weapon_comes_back_on_its_twin() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = load(&iso, &[0x02, 0x03, 0x16, 0x17]);
    draw(&mut scene, 2);
    tick(&mut scene, Input { mode_switch: true, ..Default::default() }, false, false);
    assert!(wait_idle(&mut scene, 300), "the silencer swap never settled");
    assert_eq!(scene.sim.as_ref().unwrap().gun.weapon, 0x03);
    give_and_draw(&mut scene, 0x16);
    set_rounds(&mut scene, 1, 0);
    press(&mut scene);
    for _ in 0..300 {
        tick(&mut scene, Input::default(), false, false);
        if scene.sim.as_ref().unwrap().gun.weapon != 0x16 { break; }
    }
    assert!(wait_idle(&mut scene, 300), "the switch never settled");
    assert_eq!(scene.sim.as_ref().unwrap().gun.weapon, 0x03, "the silenced twin comes back");
}

/// The exempt records keep the hand: the fists, and a watch gadget at its empty meter
/// [M 0x1a2ba8:17-30]. The fists are never dry (ammo type 0); the laser's meter is.
#[test]
fn a_spent_gadget_stays_in_hand() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = load(&iso, &[]);
    draw(&mut scene, 0x4e);
    set_rounds(&mut scene, 0, 0);
    for _ in 0..3 { tick(&mut scene, Input::default(), false, false); }
    let s = scene.sim.as_ref().unwrap();
    assert_eq!((s.gun.weapon, s.gun.switch_request), (0x4e, 0x4e), "the recharging laser stays up");
}

/// The step down from a held twin leaves the family [M 0x1a6f80:145-150: the cursor goes to the
/// record's base first]. From the silenced 0x03 the old walk stepped onto 0x02, offered 0x03 again
/// through the twin marker and the press went nowhere. Sister: the step up lands the same.
#[test]
fn cycling_down_leaves_a_silenced_pistol() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = load(&iso, &[0x02, 0x03]);
    draw(&mut scene, 2);
    tick(&mut scene, Input { mode_switch: true, ..Default::default() }, false, false);
    assert!(wait_idle(&mut scene, 300));
    assert_eq!(scene.sim.as_ref().unwrap().gun.weapon, 0x03);
    tick(&mut scene, Input { weapon_cycle: -1, ..Default::default() }, false, false);
    assert_eq!(scene.sim.as_ref().unwrap().gun.switch_request, 1, "the fists are the next weapon down");
    assert!(wait_idle(&mut scene, 300));
    assert_eq!(scene.sim.as_ref().unwrap().gun.weapon, 1);
    tick(&mut scene, Input { weapon_cycle: 1, ..Default::default() }, false, false);
    assert_eq!(scene.sim.as_ref().unwrap().gun.switch_request, 3, "back up the family hands the silenced twin");
}

/// A gadget press with no gadget to go to changes nothing [M 0x1a6f80:200 `+0x63 = +0x62`]: the old
/// fallback took the gun out of the hand for `Player_WeaponNone`.
#[test]
fn a_cycle_with_nowhere_to_go_keeps_the_gun() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = load(&iso, &[]);
    draw(&mut scene, 2);
    {
        let s = scene.sim.as_mut().unwrap();
        for id in 0x4a..=0x5e { s.drones.inventory.owned[id] = false; }
    }
    tick(&mut scene, Input { gadget_cycle: 1, ..Default::default() }, false, false);
    let s = scene.sim.as_ref().unwrap();
    assert_eq!((s.gun.weapon, s.gun.switch_request), (2, 2), "the P2K stays in the hand");
}

/// A second press during the holster walks on from the first press's request, and the draw is the
/// latest one [M 0x1a6f80:33-47 — state 2 is not refused; :145 reads +0x63; 0x1a2e38 case 2's tail].
#[test]
fn two_quick_presses_walk_two_weapons() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = load(&iso, &[0x16, 0x17]);
    give_and_draw(&mut scene, 0x16);
    draw(&mut scene, 1);
    tick(&mut scene, Input { weapon_cycle: 1, ..Default::default() }, false, false);
    assert_eq!(scene.sim.as_ref().unwrap().gun.switch_request, 2);
    tick(&mut scene, Input::default(), false, false);
    tick(&mut scene, Input { weapon_cycle: 1, ..Default::default() }, false, false);
    assert_eq!(scene.sim.as_ref().unwrap().gun.switch_request, 0x16, "the second press walks on from the request");
    assert!(wait_idle(&mut scene, 400));
    assert_eq!(scene.sim.as_ref().unwrap().gun.weapon, 0x16, "the P2K was skipped");
}

/// No cycling on a ladder's hands or with the gun raised [M 0x1a6f80:48-60]. Sister: the same press
/// with the aim button up switches.
#[test]
fn hands_and_a_raised_gun_refuse_the_cycle() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = load(&iso, &[]);
    draw(&mut scene, 2);
    let aim = Input { aiming: true, ..Default::default() };
    for _ in 0..5 { tick(&mut scene, aim, false, false); }
    assert!(scene.sim.as_ref().unwrap().aiming, "the P2K has no raise: the bit follows the button");
    tick(&mut scene, Input { aiming: true, weapon_cycle: 1, ..Default::default() }, false, false);
    assert_eq!(scene.sim.as_ref().unwrap().gun.switch_request, 2, "the raised gun refuses the cycle");
    tick(&mut scene, Input::default(), false, false);
    tick(&mut scene, Input { weapon_cycle: 1, ..Default::default() }, false, false);
    assert_ne!(scene.sim.as_ref().unwrap().gun.switch_request, 2, "sister: lowered, the press switches");
    assert!(wait_idle(&mut scene, 300));
    scene.sim.as_mut().unwrap().gun.go_to_hands();
    tick(&mut scene, Input { weapon_cycle: 1, ..Default::default() }, false, false);
    assert_eq!(scene.sim.as_ref().unwrap().gun.switch_request, 0x47, "the hands refuse the cycle");
}

/// A trigger held through the draw does not fire when the draw ends: the press edge passed during
/// the draw and the view-state gate ate it [M 0x1a5460:601-648]. Sister: a fresh press after it fires.
#[test]
fn a_trigger_held_through_the_draw_does_not_fire() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = load(&iso, &[]);
    draw(&mut scene, 1);
    tick(&mut scene, Input { switch_to: Some(2), ..Default::default() }, false, false);
    let mut held = 0;
    for _ in 0..200 {
        tick(&mut scene, Input::default(), true, false);
        held += 1;
        let s = scene.sim.as_ref().unwrap();
        if s.gun.weapon == 2 && s.gun.switch_state == 0 { break; }
    }
    for _ in 0..30 { tick(&mut scene, Input::default(), true, false); }
    assert_eq!(scene.sim.as_ref().unwrap().gun.clip, 7, "no shot from the held trigger ({held} ticks of switch)");
    tick(&mut scene, Input::default(), false, false);
    tick(&mut scene, Input::default(), true, false);
    assert_eq!(scene.sim.as_ref().unwrap().gun.clip, 6, "sister: the fresh press fires");
}

/// The reload key is read only with the trigger up [M 0x1a5460: the action-0xE read sits inside the
/// `Input_Action(9, 1) == 0` arm]. Sister: released, the same key reloads.
#[test]
fn the_reload_key_waits_for_the_trigger() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = load(&iso, &[]);
    draw(&mut scene, 2);
    set_rounds(&mut scene, 3, 20);
    for _ in 0..4 { tick(&mut scene, Input::default(), true, true); }
    assert_eq!(scene.sim.as_ref().unwrap().gun.reload_state, 0, "a held trigger keeps the key out");
    assert!(wait_idle(&mut scene, 300), "the fire script never ended");
    tick(&mut scene, Input::default(), false, true);
    assert_ne!(scene.sim.as_ref().unwrap().gun.reload_state, 0, "sister: the key reloads");
}

/// A scoped record's aim bit clears for the reload and the raise re-queues once the machine is idle
/// with the button still held [M Player_Weapon 0x1a24c8:36-47, :66-99, :117-152].
#[test]
fn the_sniper_scope_drops_for_the_reload_and_rises_after_it() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = load(&iso, &[0x1e, 0x1f]);
    give_and_draw(&mut scene, 0x1e);
    set_rounds(&mut scene, 1, 4);
    let aim = Input { aiming: true, ..Default::default() };
    for _ in 0..200 {
        tick(&mut scene, aim, false, false);
        if scene.sim.as_ref().unwrap().aiming { break; }
    }
    assert!(scene.sim.as_ref().unwrap().aiming, "the raise set the aim bit");
    // The key is refused with the scope up [M 0x1a5460: aim ∧ +0x68 & 0x40 reads as activated].
    tick(&mut scene, aim, false, true);
    assert_eq!(scene.sim.as_ref().unwrap().gun.reload_state, 0, "the raised scope refuses the reload key");
    // The last round, an aimed shot: the bit survives the fire script [M the state-9 arm].
    tick(&mut scene, aim, true, false);
    let mut through_fire = true;
    for _ in 0..400 {
        tick(&mut scene, aim, false, false);
        let s = scene.sim.as_ref().unwrap();
        if s.gun.reload_state != 0 { break; }
        through_fire &= s.aiming;
    }
    assert!(through_fire, "the aimed shot keeps the scope up");
    {
        let s = scene.sim.as_ref().unwrap();
        assert_ne!(s.gun.reload_state, 0, "the dry clip reloads with the trigger up");
    }
    tick(&mut scene, aim, false, false);
    assert!(!scene.sim.as_ref().unwrap().aiming, "the reload drops the scope [M 0x1a24c8: state 6 in the scoped set]");
    let mut back = false;
    for _ in 0..600 {
        tick(&mut scene, aim, false, false);
        if scene.sim.as_ref().unwrap().aiming { back = true; break; }
    }
    assert!(back, "with the button held the raise re-queues after the reload");
}

/// The view-state gate covers the aim raise (state 0xF): a press during the raise is spent on the
/// gate — no shot [M 0x1a5460:631-648]. Sister: raised, the press fires.
#[test]
fn no_shot_during_the_scope_raise() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = load(&iso, &[0x1e, 0x1f]);
    give_and_draw(&mut scene, 0x1e);
    let clip = scene.sim.as_ref().unwrap().gun.clip;
    let aim = Input { aiming: true, ..Default::default() };
    tick(&mut scene, aim, false, false);
    tick(&mut scene, aim, false, false);
    assert!(scene.sim.as_ref().unwrap().view_aim_anim.is_some(), "the raise is playing");
    tick(&mut scene, aim, true, false);
    tick(&mut scene, aim, false, false);
    assert_eq!(scene.sim.as_ref().unwrap().gun.clip, clip, "the raise eats the press");
    for _ in 0..200 {
        tick(&mut scene, aim, false, false);
        if scene.sim.as_ref().unwrap().aiming { break; }
    }
    tick(&mut scene, aim, true, false);
    assert_eq!(scene.sim.as_ref().unwrap().gun.clip, clip - 1, "sister: raised, the press fires");
}

/// A fire press on the guided ride blows the rocket where it is and hands the view back
/// [M 0x1a5460 → 0x12cd98 → `Bullet_Delete` 0x12e660:26-30]. The launch press itself, still held,
/// does not. Sister: without the press the rocket flies on.
#[test]
fn the_fire_press_blows_the_guided_rocket() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = load(&iso, &[0x2c, 0x2d]);
    give_and_draw(&mut scene, 0x2c);
    scene.sim.as_mut().unwrap().bond.pitch = 0.5;
    for _ in 0..4 { tick(&mut scene, Input::default(), true, false); }
    assert!(scene.sim.as_ref().unwrap().missile_cam.is_some_and(|c| c.riding), "the guided shot rides");
    for _ in 0..10 { tick(&mut scene, Input::default(), true, false); }
    assert!(scene.sim.as_ref().unwrap().missile_cam.is_some_and(|c| c.riding), "the held launch press is no new press");
    tick(&mut scene, Input::default(), false, false);
    assert!(scene.sim.as_ref().unwrap().missile_cam.is_some_and(|c| c.riding), "sister: released, it flies on");
    tick(&mut scene, Input::default(), true, false);
    tick(&mut scene, Input::default(), false, false);
    assert!(scene.sim.as_ref().unwrap().missile_cam.is_none_or(|c| !c.riding), "the press ended the ride");
    assert!(!scene.sim.as_ref().unwrap().drones.player_bullets.iter().any(|b| b.guided_cam && !b.dead));
}

/// The last Remote Mine thrown goes to the detonator while it is down: `+0x68 & 0x10` offers the
/// twin 0x38, and `Player_WeaponHasAmmo`'s object walk finds the placed mine [M 0x1a2ba8:37-44;
/// 0x1a16c0:28-51]. Detonating the last one clears the mine family and takes the best weapon.
#[test]
fn the_last_mine_leaves_the_detonator_in_hand() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = load(&iso, &[0x37, 0x38]);
    give_and_draw(&mut scene, 0x37);
    set_rounds(&mut scene, 1, 0);
    tick(&mut scene, Input::default(), true, false);
    let mut landed = false;
    for _ in 0..400 {
        tick(&mut scene, Input::default(), false, false);
        let s = scene.sim.as_ref().unwrap();
        if s.gun.weapon == 0x38 && s.gun.switch_state == 0 { landed = true; break; }
    }
    let s = scene.sim.as_ref().unwrap();
    assert!(landed, "the hand ended on {:#04x} (request {:#04x})", s.gun.weapon, s.gun.switch_request);
    press(&mut scene);
    assert!(wait_idle(&mut scene, 400), "the switch after the detonator never settled");
    let s = scene.sim.as_ref().unwrap();
    assert_eq!(s.gun.weapon, 0x02, "dry, the detonator hands over to the best weapon");
    assert!(!s.drones.inventory.owned[0x37], "the mine family is gone");
}

/// The use gates ahead of the pull [M 0x1a5460]: the Micro-Camera unaimed takes no picture (the press
/// only toggles the watch pair [M 0x1a24c8 mode block]); the Q-Claw outside a type-2 icon's arm runs
/// no fire script. Sister: the camera raised takes it.
#[test]
fn the_gadget_use_gates_refuse_the_press() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = load(&iso, &[]);
    draw(&mut scene, 0x54);
    let before = scene.sim.as_ref().unwrap().last_shot;
    tick(&mut scene, Input { fire: true, ..Default::default() }, true, false);
    let s = scene.sim.as_ref().unwrap();
    assert_eq!(s.last_shot, before, "no picture from the hip");
    assert_eq!(s.gun.switch_request, 0x55, "the same press flips the watch pair");
    assert!(wait_idle(&mut scene, 300));
    draw(&mut scene, 0x54);
    let aim = Input { aiming: true, ..Default::default() };
    for _ in 0..120 {
        tick(&mut scene, aim, false, false);
        if scene.sim.as_ref().unwrap().aiming { break; }
    }
    assert!(scene.sim.as_ref().unwrap().aiming, "the camera raised");
    tick(&mut scene, aim, true, false);
    assert!(scene.sim.as_ref().unwrap().last_shot != before, "sister: raised, it shoots");

    let mut scene = load(&iso, &[]);
    draw(&mut scene, 0x50);
    assert_ne!(scene.sim.as_ref().unwrap().bond.icon_ctx, 2, "the start is outside every claw icon");
    tick(&mut scene, Input::default(), true, false);
    assert!(!scene.sim.as_ref().unwrap().gun.fire_state, "no claw shot outside the icon");
}

/// A laser meter run dry under the held trigger ends the held fire script: the release script plays
/// and the view leaves the fire state, where the regen can run [M 0x1a2e38 case 9's
/// `Player_WeaponHasAmmo`; 0x193600 regens off state 9]. Each unit it regains re-fires under the same
/// pull, so the meter hovers at empty until the release.
#[test]
fn a_dry_laser_leaves_the_fire_state_under_the_held_trigger() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = load(&iso, &[]);
    draw(&mut scene, 0x4e);
    set_rounds(&mut scene, 3, 0);
    let mut left = false;
    for _ in 0..240 {
        tick(&mut scene, Input::default(), true, false);
        left |= !scene.sim.as_ref().unwrap().gun.fire_state;
    }
    assert!(left, "the held script ended with the meter");
}
